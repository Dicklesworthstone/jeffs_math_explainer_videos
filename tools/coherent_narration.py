#!/usr/bin/env python3
"""Generate coherent takes and connect reviewed sample spans to Narrator cues.

Generation reuses narration_cache's actual EOS and asset checks. Assembly never
guesses alignments: a director supplies measured sentence ends in source samples.
All selected target PCM is retained, in order, with optional authored silence.
A contextual pickup may exclude only its explicitly recorded following context.
Output is
the existing per-cue WAV/report.json interface, not a new rendering path.
Technical validation is not a listening judgment.
"""
from __future__ import annotations

import argparse
import importlib.util
import json
import os
from pathlib import Path
import re
import sys
import wave

REPO = Path(__file__).resolve().parents[1]

_spec = importlib.util.spec_from_file_location("narration_cache", Path(__file__).with_name("narration_cache.py"))
cache = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(cache)


def read_script(path):
    result = {}
    for number, row in enumerate(Path(path).read_text().splitlines(), 1):
        if not row.strip() or row.startswith("#"):
            continue
        cells = row.split("\t")
        if len(cells) != 2 or not re.fullmatch(r"\d\d_\d+", cells[0]) or not cells[1].strip():
            raise ValueError(f"invalid script row {number}")
        if cells[0] in result:
            raise ValueError(f"duplicate script cue {cells[0]}")
        result[cells[0]] = cells[1]
    if not result:
        raise ValueError("empty script")
    return result


def sentences(text):
    # Matches the existing kit's punctuation boundary contract.
    return [s.strip() for s in re.findall(r".*?[.?!](?= |$)|.+$", text) if s.strip()]


def direction_words(text):
    return [m.lower().replace('’',"'") for m in re.findall(r"[A-Za-z0-9]+(?:['’][A-Za-z]+)?",text)]


def direction_map_path(value, topic):
    """Resolve durable map references independently of the caller's cwd."""
    if not isinstance(value, str) or not value.strip():
        raise ValueError('direction map path must be repository-relative')
    path = Path(value)
    if path.is_absolute() or '..' in path.parts:
        raise ValueError('direction map path must be repository-relative')
    resolved = (REPO / path).resolve()
    if not resolved.is_relative_to(REPO.resolve()):
        raise ValueError('direction map path escapes repository')
    if not isinstance(topic, str) or not re.fullmatch(r'[a-z][a-z0-9_]*', topic):
        raise ValueError('direction map needs a safe topic identifier')
    topic_path = Path('explainer_corpus') / topic
    if (not path.is_relative_to(topic_path)
            or not resolved.is_relative_to(REPO.resolve() / topic_path)):
        raise ValueError('direction map must stay inside its topic tree')
    return resolved


def direction_contract(script_path, plan):
    """Opted-in plans bind every direction before generation and assembly.

    Plans without a direction contract remain readable unless the caller requires
    one. Bound maps must live in the repository; archived absolute-path plans
    require their retained adapter or an explicit metadata-only rebind.
    This validates authored coverage, not the quality or application of pauses.
    """
    contract=plan.get('direction_contract')
    if contract is None:return None
    if not isinstance(contract,dict) or contract.get('schema_version')!=1 or contract.get('script_sha256')!=cache.digest(script_path):
        raise ValueError('direction contract does not bind this script')
    source=contract['map'];path=direction_map_path(source['path'], contract['topic'])
    if cache.digest(path)!=source['sha256']:
        raise ValueError('direction map changed')
    data=json.loads(path.read_text());script=read_script(script_path);resolved=[];seen=set()
    if not data['overrides']:raise ValueError('empty direction contract')
    for i,item in enumerate(data['overrides']):
        if item['video']!=contract['topic'] or item['cue'] not in script:
            raise ValueError('direction names a different topic or unknown cue; no silent filtering')
        text,phrase=direction_words(script[item['cue']]),direction_words(item['after'])
        hits=[at+len(phrase)-1 for at in range(len(text)-len(phrase)+1) if phrase and text[at:at+len(phrase)]==phrase]
        if 'word_index' in item:hits=[at for at in hits if at==item['word_index']]
        if len(hits)!=1:raise ValueError('direction phrase is missing or ambiguous')
        key=(item['cue'],hits[0])
        if key in seen:raise ValueError('duplicate direction boundary')
        seen.add(key);resolved.append({'override_index':i,'cue':item['cue'],'end_word_index':hits[0],
                                      'direction':item})
    return {'contract':contract,'resolved':resolved,
            'claim':'Authored coverage only. No acoustic or listening acceptance.'}


def review_directions(contract, selected, manifest):
    if contract is None:return None
    cues={c for unit in selected for c in unit['line_ids']}
    expected={r['override_index'] for r in contract['resolved'] if r['cue'] in cues}
    reviews=manifest.get('direction_dispositions',[])
    if len(reviews)!=len(expected) or {r['override_index'] for r in reviews}!=expected:
        raise ValueError('every selected authored direction needs an explicit assembly disposition')
    for row in reviews:
        if row['kind'] not in ('retained_source','authored_silence','contextual_reread','native_visual_hold') or not row.get('reason','').strip():
            raise ValueError('direction disposition needs a concrete action and reason')
        evidence=row['evidence']
        if cache.digest(evidence['path'])!=evidence['sha256']:
            raise ValueError('direction disposition evidence changed')
    return {'binding':contract,'dispositions':reviews,
            'claim':'Every selected direction is accounted for; truth of auditory judgments is separate.'}


def check_dispositions(directions, lines):
    """A disposition must agree with the assembled samples at its boundary.

    Inserted silence needs a nonzero sentence gap or a bound intra-sentence
    insertion; retained_source needs neither; a visual hold ends its cue.
    contextual_reread is not checkable from samples and stays a reviewed claim.
    """
    if directions is None:
        if any(row.get('intra_sentence_pauses') for rows in lines.values() for row in rows):
            raise ValueError('intra-sentence pauses require bound directions')
        return
    kinds={r['override_index']:r['kind'] for r in directions['dispositions']}
    resolved={r['override_index']:r for r in directions['binding']['resolved']}
    inserted=set()
    for cue, rows in lines.items():
        count=0
        for row in rows:
            end=count+len(direction_words(row['sentence']))
            for pause in row.get('intra_sentence_pauses',[]):
                index=pause['override_index'];binding=resolved.get(index)
                if (type(index) is not int or index in inserted or binding is None
                        or binding['cue']!=cue or not count<=binding['end_word_index']<end-1
                        or kinds.get(index)!='authored_silence'):
                    raise ValueError('intra-sentence pause has wrong, duplicate or undisposed direction')
                inserted.add(index)
            count=end
    for r in directions['binding']['resolved']:
        if r['override_index'] not in kinds:continue
        ends,count={},0
        for row in lines[r['cue']]:
            count+=len(direction_words(row['sentence']));ends[count-1]=row['gap_after_samples']
        kind,gap=kinds[r['override_index']],ends.get(r['end_word_index'])
        if r['override_index'] in inserted:gap=1
        if ((kind=='authored_silence' and not gap) or (kind=='retained_source' and gap)
                or (kind=='native_visual_hold' and r['end_word_index']!=count-1)):
            raise ValueError(f"direction disposition contradicts assembled samples at override {r['override_index']}")


def read_units(script_path, units_path, require_directions=False):
    script = read_script(script_path)
    plan = json.loads(Path(units_path).read_text())
    if plan.get("schema_version") != 1 or plan.get("script_sha256") != cache.digest(script_path):
        raise ValueError("unit plan does not bind current script bytes")
    units = plan["units"]
    ids, covered = set(), []
    for unit in units:
        uid, cues = unit["id"], unit["line_ids"]
        if not re.fullmatch(r"[A-Za-z0-9_-]{1,60}", uid) or uid in ids:
            raise ValueError("duplicate or unsafe unit ID")
        ids.add(uid)
        if not cues or any(c not in script for c in cues) or len({c[:2] for c in cues}) != 1:
            raise ValueError(f"invalid or cross-chapter cue group {uid}")
        text = " ".join(script[c] for c in cues)
        if unit["text"] != text or not 1 <= len(text.split()) <= 100:
            raise ValueError(f"unit {uid} must contain exact script text, at most 100 words")
        covered.extend(cues)
    if covered != list(script):
        raise ValueError("unit plan omits, repeats or reorders script cues")
    if require_directions and plan.get('direction_contract') is None:
        raise ValueError('this pipeline requires a direction contract; a legacy plan cannot opt out here')
    direction_contract(script_path,plan)
    return script, units


def request_text(unit, script, context_after=None):
    """Following context is the next canonical cue, never invented coverage."""
    if context_after is None:
        return unit["text"]
    cues = list(script)
    at = cues.index(unit["line_ids"][-1]) + 1
    if at == len(cues) or context_after != cues[at]:
        raise ValueError("following context must be the next canonical script cue")
    text = unit["text"] + " " + script[context_after]
    if len(text.split()) > 100:
        raise ValueError("target plus following context exceeds 100 words")
    return text


def generate(args):
    script, units = read_units(args.script, args.units, getattr(args, "require_directions", False))
    selected = [u for u in units if not args.chapter or u["line_ids"][0].startswith(args.chapter + "_")]
    if args.unit:
        if len(set(args.unit)) != len(args.unit) or set(args.unit) - {u["id"] for u in selected}:
            raise ValueError("repair selection contains duplicate or unknown units")
        selected = [u for u in selected if u["id"] in args.unit]
    if not selected:
        raise ValueError("selected chapter has no units")
    env = cache.child_environment(os.environ)
    options = {"profile": "balanced", "route": "in-process", "normalization": "verbatim", "voice_cache": False}
    def capture():
        return cache.make_context(args.ftts, args.voice, args.model, Path(__file__), args.script,
                                  args.lexicon, env, options)
    context = capture()
    inputs = {"script": cache.identity(args.script), "units": cache.identity(args.units)}
    directions=direction_contract(args.script,json.loads(args.units.read_text()))
    following = {}
    if args.following_context:
        cues = list(script)
        for u in selected:
            at = cues.index(u["line_ids"][-1]) + 1
            if at == len(cues):
                raise ValueError("final unit has no canonical following context")
            following[u["id"]] = cues[at]
    plan = {u["id"]: [{"j": 0, "reads": [{"tag": f"{u['id']}_s{args.seed}", "seed": args.seed,
                      "text": u["text"], "tts": request_text(u, script, following.get(u["id"]))}]}] for u in selected}
    jobs = cache.bind_plan(plan, context)
    # This directory is an immutable run, not a silently refreshed plan.
    args.out.mkdir(parents=True, exist_ok=False)
    cache.write_new(args.out / "plan.json", {"inputs": inputs, "units": selected, "plan": plan,
                    "request_budget": len(jobs), "retry_budget": 0, "seed": args.seed,
                    "following_context": following,"direction_contract":directions})
    def verify_inputs():
        if capture() != context or inputs != {"script": cache.identity(args.script), "units": cache.identity(args.units)}:
            raise ValueError("generation inputs changed; preserve attempts, do not qualify outputs")
        if direction_contract(args.script,json.loads(args.units.read_text()))!=directions:
            raise ValueError('generation direction contract changed')
    results = cache.run_batch(jobs, args.out / "takes", env, 1, 300, verify_inputs)
    cache.write_new(args.out / "result.json", {"units": [{"id": u["id"], "tag": j["tag"], "state": r["state"]}
                    for u, j, r in zip(selected, jobs, results)], "claim": "generation integrity only; content and listening pending"})
    return int(any(r["state"] == "failed" for r in results))


def wav_pcm(path):
    info = cache.wav_identity(path)
    with wave.open(str(path), "rb") as w:
        return info, w.readframes(w.getnframes())


def render_source_span(pcm, start, end, pauses):
    """Insert explicit zeros without deleting/reordering speech; not a safe-cut verdict."""
    chunks=[];previous=start
    for pause in pauses:
        at,added=pause['source_sample'],pause['inserted_samples']
        lo,hi=pause['source_zero']
        if (any(type(n) is not int for n in (at,added,lo,hi))
                or not previous<at<end or not 0<added<=24000*5
                or not 0<lo<at<hi<len(pcm)//2):
            raise ValueError('invalid intra-sentence source position or silence length')
        if (any(pcm[lo*2:hi*2]) or pcm[(lo-1)*2:lo*2]==b'\0\0'
                or pcm[hi*2:(hi+1)*2]==b'\0\0'):
            raise ValueError('intra-sentence exact-zero run changed or is incomplete')
        chunks.extend((pcm[previous*2:at*2],b'\0\0'*added));previous=at
    chunks.append(pcm[previous*2:end*2])
    return b''.join(chunks)


def unit_spans(unit, script, record):
    wav_path = Path(record["wav"]).resolve(strict=True)
    receipt_path = Path(record["generation_receipt"]).resolve(strict=True)
    receipt = json.loads(receipt_path.read_text())
    info, pcm = wav_pcm(wav_path)
    if info != receipt["wav"] or info["sha256"] != record["sha256"]:
        raise ValueError("source audio no longer matches generation receipt")
    context_after = record.get("context_after")
    if receipt["returncode"] != 0 or receipt["request"]["submitted_text"] != request_text(unit, script, context_after):
        raise ValueError("generation request does not contain this unit's script")
    cache.validate_generation(receipt["generation"], receipt["request"], receipt["completion"])
    if receipt["completion"]["event"] != "run_complete" or receipt["completion"]["exit_code"] != 0:
        raise ValueError("generation has no successful terminal event")
    if receipt["generation"]["output"]["samples"] != info["frames"]:
        raise ValueError("generation sample count differs from source WAV")
    parts = [(cue, sentence) for cue in unit["line_ids"] for sentence in sentences(script[cue])]
    ends, gaps = record["sentence_ends"], record["gap_after_samples"]
    if len(ends) != len(parts) or len(gaps) != len(parts):
        raise ValueError("each scripted sentence needs one measured end and authored gap")
    if any(type(n) is not int for n in ends + gaps):
        raise ValueError("sample boundaries and gaps must be integers")
    target_end = info["frames"]
    if context_after is not None:
        target_end = record.get("target_end_sample")
        if type(target_end) is not int or not 0 < target_end < info["frames"]:
            raise ValueError("contextual pickup needs a measured target prefix before the source end")
    elif "target_end_sample" in record:
        raise ValueError("a target crop requires bound canonical following context")
    if not ends or ends[-1] != target_end or any(b <= a for a, b in zip([0] + ends, ends)):
        raise ValueError("source spans must cover every selected target sample once, in increasing order")
    if any(n < 0 or n > 24000 * 5 for n in gaps):
        raise ValueError("authored inserted silence must be between zero and five seconds")
    pauses=record.get('intra_sentence_pauses',[])
    if (not isinstance(pauses,list) or any(not isinstance(p,dict) for p in pauses)
            or any(type(p.get('source_sample')) is not int for p in pauses)):
        raise ValueError('invalid intra-sentence pause list')
    if any(b['source_sample']<=a['source_sample'] for a,b in zip(pauses,pauses[1:])):
        raise ValueError('intra-sentence pauses must be ordered without duplicate cuts')
    if any(not 0<p['source_sample']<target_end or p['source_sample'] in ends for p in pauses):
        raise ValueError('intra-sentence insertion must be inside one selected sentence')
    start = 0
    rows = []
    for (cue, sentence), end, gap in zip(parts, ends, gaps):
        within=[p for p in pauses if start<p['source_sample']<end]
        rendered=render_source_span(pcm,start,end,within)
        rows.append((cue, {"sentence": sentence, "tts": sentence, "start": start / 24000,
                     "end": end / 24000, "pause_cut_s": 0.0, "out_samples": len(rendered)//2,
                     "gap_after_samples": gap, "source": str(wav_path), "source_sha256": info["sha256"],
                     "source_in_sample": start, "source_out_sample": end, "unit_id": unit["id"],
                     **({'intra_sentence_pauses':within} if within else {})},
                     rendered + b"\0\0" * gap))
        start = end
    return rows


def check_boundary_review(review):
    """Validate the recorded claim, not whether any listening actually occurred."""
    if (not isinstance(review, dict)
            or any(not isinstance(review.get(key), str) or not review[key].strip()
                   for key in ('reviewer', 'basis'))
            or review.get('listening') not in ('unreviewed', 'reviewed')):
        raise ValueError('record boundary inspection basis and truthful listening status')


def assemble(args):
    script, units = read_units(args.script, args.units, getattr(args, "require_directions", False))
    manifest = json.loads(args.spans.read_text())
    if manifest.get("schema_version") != 1 or manifest.get("units_sha256") != cache.digest(args.units):
        raise ValueError("span manifest does not bind current unit plan")
    review = manifest.get("boundary_review")
    check_boundary_review(review)
    selected = [u for u in units if not args.chapter or u["line_ids"][0].startswith(args.chapter + "_")]
    records = manifest["units"]
    if not selected or [r["id"] for r in records] != [u["id"] for u in selected]:
        raise ValueError("span manifest omits, repeats or reorders selected units")
    directions=review_directions(direction_contract(args.script,json.loads(args.units.read_text())),selected,manifest)
    lines, audio = {}, {}
    for unit, record in zip(selected, records):
        for cue, row, pcm in unit_spans(unit, script, record):
            lines.setdefault(cue, []).append(row)
            audio[cue] = audio.get(cue, b"") + pcm
    check_dispositions(directions, lines)
    args.out.mkdir(parents=True, exist_ok=False)
    outputs = {}
    for cue, pcm in audio.items():
        path = args.out / f"{cue}.wav"
        with path.open("xb") as handle:
            with wave.open(handle, "wb") as w:
                w.setparams((1, 2, 24000, 0, "NONE", "not compressed"))
                w.writeframes(pcm)
        outputs[cue] = cache.wav_identity(path)
    report = {"meta": {"rate": 24000, "adapter": "coherent-narration-v1",
              "script_sha256": cache.digest(args.script), "units_sha256": cache.digest(args.units),
              "spans_sha256": cache.digest(args.spans), "boundary_review": review,"direction_contract":directions,
              "claim": "sample-preserving assembly; listening and visual synchronization are separate"}, "lines": lines}
    cache.write_new(args.out / "report.json", report)
    cache.write_new(args.out / "assembly.json", {"schema_version": 1, "outputs": outputs,
                    "report_sha256": cache.digest(args.out / "report.json"), "manifest": manifest})
    verify(args.out, args.script, args.units,
           require_directions=getattr(args, "require_directions", False), chapter=args.chapter)
    print(json.dumps({"assembled_cues": len(lines), "chapter": args.chapter, "out": str(args.out)}))
    return 0


def verify(out, script_path=None, units_path=None, *, require_directions=False, chapter=None):
    """Reconstruct PCM and, when supplied, recheck the caller's source contract.

    A report's own hashes cannot establish which script/plan the caller expects.
    Supplying both paths checks full-film coverage by default, or exactly the
    explicit chapter. This is technical verification, never listening approval.
    """
    if (script_path is None) != (units_path is None):
        raise ValueError('verification needs both script and unit plan')
    if (require_directions or chapter is not None) and script_path is None:
        raise ValueError('required directions or chapter scope need external script and unit plan')
    receipt = json.loads((out / "assembly.json").read_text())
    report = json.loads((out / "report.json").read_text())
    if receipt["report_sha256"] != cache.digest(out / "report.json"):
        raise ValueError("report hash changed")
    if set(receipt["outputs"]) != set(report["lines"]):
        raise ValueError("report/output cue coverage differs")
    review = report['meta'].get('boundary_review')
    if review != receipt['manifest'].get('boundary_review'):
        raise ValueError('boundary review differs between report and manifest')
    check_boundary_review(review)
    directions = report["meta"].get("direction_contract")
    if directions is not None:
        # Even reconstruction-only checks must not silently drop a disposition
        # from the report's declared set of selected direction boundaries.
        review_directions(directions['binding'], [{'line_ids': list(report['lines'])}],
                          {'direction_dispositions': directions['dispositions']})
    if script_path is not None:
        script, units = read_units(script_path, units_path, require_directions)
        if report['meta'].get('rate') != 24000 or report['meta'].get('adapter') != 'coherent-narration-v1':
            raise ValueError('assembly metadata does not describe this adapter and sample rate')
        if (report['meta'].get('script_sha256') != cache.digest(script_path)
                or report['meta'].get('units_sha256') != cache.digest(units_path)):
            raise ValueError('assembly does not bind the supplied script and unit plan')
        selected = [u for u in units if chapter is None or u['line_ids'][0].startswith(chapter + '_')]
        expected_cues = [cue for u in selected for cue in u['line_ids']]
        if not selected or set(report['lines']) != set(expected_cues):
            raise ValueError('assembly cue coverage differs from the requested script/chapter')
        manifest = receipt['manifest']
        records = manifest['units']
        if (manifest.get('schema_version') != 1 or manifest.get('units_sha256') != cache.digest(units_path)
                or [r['id'] for r in records] != [u['id'] for u in selected]):
            raise ValueError('assembly manifest differs from the requested unit plan/scope')
        contract = direction_contract(script_path, json.loads(Path(units_path).read_text()))
        expected_directions = review_directions(contract, selected, manifest)
        if directions != expected_directions:
            raise ValueError('assembly direction binding differs from the supplied contract')
        expected_lines = {}
        for unit, record in zip(selected, records):
            for cue, row, _ in unit_spans(unit, script, record):
                expected_lines.setdefault(cue, []).append(row)
        if report['lines'] != expected_lines:
            raise ValueError('assembly sentence/source spans differ from the supplied script and generation receipts')
    check_dispositions(directions,report['lines'])
    if directions:
        source = directions["binding"]["contract"]["map"]
        bound = [(direction_map_path(source["path"], directions["binding"]["contract"]["topic"]), source["sha256"])] + [(r["evidence"]["path"], r["evidence"]["sha256"])
                                                         for r in directions["dispositions"]]
        if any(cache.digest(p) != h for p, h in bound):
            raise ValueError("direction map or disposition evidence changed after assembly")
    for cue, rows in report["lines"].items():
        rebuilt = b""
        for row in rows:
            info, pcm = wav_pcm(Path(row["source"]))
            if info["sha256"] != row["source_sha256"]:
                raise ValueError("source changed after assembly")
            rendered=render_source_span(pcm,row['source_in_sample'],row['source_out_sample'],row.get('intra_sentence_pauses',[]))
            if len(rendered)//2!=row['out_samples']:
                raise ValueError('reported span sample count differs from reconstruction')
            rebuilt += rendered
            rebuilt += b"\0\0" * row["gap_after_samples"]
        actual_info, actual_pcm = wav_pcm(out / f"{cue}.wav")
        if actual_info != receipt["outputs"][cue] or rebuilt != actual_pcm:
            raise ValueError(f"cue {cue} does not reconstruct sample-for-sample")
    return True


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    commands = parser.add_subparsers(dest="command", required=True)
    check = commands.add_parser("check")
    gen = commands.add_parser("generate")
    assemble_parser = commands.add_parser("assemble")
    for p in (check, gen, assemble_parser):
        p.add_argument("--script", type=Path, required=True)
        p.add_argument("--units", type=Path, required=True)
        p.add_argument("--require-directions", action="store_true",
                       help="refuse a plan without a hash-bound direction_contract (third-wave callers)")
    for p in (gen, assemble_parser):
        p.add_argument("--out", type=Path, required=True)
        p.add_argument("--chapter", help="two-digit chapter prefix; omitted means all chapters")
    for option in ("ftts", "voice", "model"):
        gen.add_argument("--" + option, required=True)
    gen.add_argument("--lexicon", type=Path, required=True)
    gen.add_argument("--seed", type=int, default=2)
    gen.add_argument("--unit", action="append", help="generate only this existing unit; repeat for a bounded repair")
    gen.add_argument("--following-context", action="store_true",
                     help="append the next canonical cue as synthesis-only context; assembly needs a measured target prefix")
    assemble_parser.add_argument("--spans", type=Path, required=True)
    v = commands.add_parser("verify")
    v.add_argument("out", type=Path)
    v.add_argument("--script", type=Path, help="external expected script; requires --units")
    v.add_argument("--units", type=Path, help="external expected unit plan; requires --script")
    v.add_argument("--require-directions", action="store_true",
                   help="require and revalidate external script/plan/direction coverage")
    v.add_argument("--chapter", help="verify exactly this chapter; default with script/units is the whole film")
    args = parser.parse_args()
    if args.command == "generate":
        if not 0 < args.seed < 2**64:
            raise ValueError("use an explicit positive seed")
        return generate(args)
    if args.command == "assemble":
        return assemble(args)
    if args.command == "verify":
        verify(args.out, args.script, args.units, require_directions=args.require_directions, chapter=args.chapter)
        scope = "supplied script/unit/generation binding plus " if args.script else ""
        if args.require_directions:
            scope += "required direction coverage plus "
        print(scope + "sample reconstruction verified; listening is separate")
        return 0
    script, units = read_units(args.script, args.units, args.require_directions)
    print(json.dumps({"cues": len(script), "units": len(units)}))
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except (OSError, ValueError, KeyError, TypeError, wave.Error) as error:
        print(f"coherent-narration: {error}", file=sys.stderr)
        raise SystemExit(1) from error
