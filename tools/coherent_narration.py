#!/usr/bin/env python3
"""Generate coherent takes and connect reviewed sample spans to Narrator cues.

Generation reuses narration_cache's actual EOS and asset checks. Assembly never
guesses alignments: a director supplies measured sentence ends in source samples.
All source PCM is retained, in order, with optional authored silence. Output is
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


def read_units(script_path, units_path):
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
    return script, units


def generate(args):
    script, units = read_units(args.script, args.units)
    selected = [u for u in units if not args.chapter or u["line_ids"][0].startswith(args.chapter + "_")]
    if not selected:
        raise ValueError("selected chapter has no units")
    env = cache.child_environment(os.environ)
    options = {"profile": "balanced", "route": "in-process", "normalization": "verbatim", "voice_cache": False}
    def capture():
        return cache.make_context(args.ftts, args.voice, args.model, Path(__file__), args.script,
                                  args.lexicon, env, options)
    context = capture()
    inputs = {"script": cache.identity(args.script), "units": cache.identity(args.units)}
    plan = {u["id"]: [{"j": 0, "reads": [{"tag": f"{u['id']}_s{args.seed}", "seed": args.seed,
                      "text": u["text"], "tts": u["text"]}]}] for u in selected}
    jobs = cache.bind_plan(plan, context)
    # This directory is an immutable run, not a silently refreshed plan.
    args.out.mkdir(parents=True, exist_ok=False)
    cache.write_new(args.out / "plan.json", {"inputs": inputs, "units": selected, "plan": plan,
                    "request_budget": len(jobs), "retry_budget": 0, "seed": args.seed})
    def verify_inputs():
        if capture() != context or inputs != {"script": cache.identity(args.script), "units": cache.identity(args.units)}:
            raise ValueError("generation inputs changed; preserve attempts, do not qualify outputs")
    results = cache.run_batch(jobs, args.out / "takes", env, 1, 300, verify_inputs)
    cache.write_new(args.out / "result.json", {"units": [{"id": u["id"], "tag": j["tag"], "state": r["state"]}
                    for u, j, r in zip(selected, jobs, results)], "claim": "generation integrity only; content and listening pending"})
    return int(any(r["state"] == "failed" for r in results))


def wav_pcm(path):
    info = cache.wav_identity(path)
    with wave.open(str(path), "rb") as w:
        return info, w.readframes(w.getnframes())


def unit_spans(unit, script, record):
    wav_path = Path(record["wav"]).resolve(strict=True)
    receipt_path = Path(record["generation_receipt"]).resolve(strict=True)
    receipt = json.loads(receipt_path.read_text())
    info, pcm = wav_pcm(wav_path)
    if info != receipt["wav"] or info["sha256"] != record["sha256"]:
        raise ValueError("source audio no longer matches generation receipt")
    if receipt["returncode"] != 0 or receipt["request"]["submitted_text"] != unit["text"]:
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
    if not ends or ends[-1] != info["frames"] or any(b <= a for a, b in zip([0] + ends, ends)):
        raise ValueError("source spans must cover every sample once, in increasing order")
    if any(n < 0 or n > 24000 * 5 for n in gaps):
        raise ValueError("authored inserted silence must be between zero and five seconds")
    start = 0
    rows = []
    for (cue, sentence), end, gap in zip(parts, ends, gaps):
        rows.append((cue, {"sentence": sentence, "tts": sentence, "start": start / 24000,
                     "end": end / 24000, "pause_cut_s": 0.0, "out_samples": end - start,
                     "gap_after_samples": gap, "source": str(wav_path), "source_sha256": info["sha256"],
                     "source_in_sample": start, "source_out_sample": end, "unit_id": unit["id"]},
                     pcm[start * 2:end * 2] + b"\0\0" * gap))
        start = end
    return rows


def assemble(args):
    script, units = read_units(args.script, args.units)
    manifest = json.loads(args.spans.read_text())
    if manifest.get("schema_version") != 1 or manifest.get("units_sha256") != cache.digest(args.units):
        raise ValueError("span manifest does not bind current unit plan")
    review = manifest["boundary_review"]
    if not review.get("reviewer") or not review.get("basis") or review.get("listening") not in ("unreviewed", "reviewed"):
        raise ValueError("record boundary inspection basis and truthful listening status")
    selected = [u for u in units if not args.chapter or u["line_ids"][0].startswith(args.chapter + "_")]
    records = manifest["units"]
    if not selected or [r["id"] for r in records] != [u["id"] for u in selected]:
        raise ValueError("span manifest omits, repeats or reorders selected units")
    lines, audio = {}, {}
    for unit, record in zip(selected, records):
        for cue, row, pcm in unit_spans(unit, script, record):
            lines.setdefault(cue, []).append(row)
            audio[cue] = audio.get(cue, b"") + pcm
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
              "spans_sha256": cache.digest(args.spans), "boundary_review": review,
              "claim": "sample-preserving assembly; listening and visual synchronization are separate"}, "lines": lines}
    cache.write_new(args.out / "report.json", report)
    cache.write_new(args.out / "assembly.json", {"schema_version": 1, "outputs": outputs,
                    "report_sha256": cache.digest(args.out / "report.json"), "manifest": manifest})
    verify(args.out)
    print(json.dumps({"assembled_cues": len(lines), "chapter": args.chapter, "out": str(args.out)}))
    return 0


def verify(out):
    receipt = json.loads((out / "assembly.json").read_text())
    report = json.loads((out / "report.json").read_text())
    if receipt["report_sha256"] != cache.digest(out / "report.json"):
        raise ValueError("report hash changed")
    if set(receipt["outputs"]) != set(report["lines"]):
        raise ValueError("report/output cue coverage differs")
    for cue, rows in report["lines"].items():
        rebuilt = b""
        for row in rows:
            info, pcm = wav_pcm(Path(row["source"]))
            if info["sha256"] != row["source_sha256"]:
                raise ValueError("source changed after assembly")
            rebuilt += pcm[row["source_in_sample"] * 2:row["source_out_sample"] * 2]
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
    for p in (gen, assemble_parser):
        p.add_argument("--out", type=Path, required=True)
        p.add_argument("--chapter", help="two-digit chapter prefix; omitted means all chapters")
    for option in ("ftts", "voice", "model"):
        gen.add_argument("--" + option, required=True)
    gen.add_argument("--lexicon", type=Path, required=True)
    gen.add_argument("--seed", type=int, default=2)
    assemble_parser.add_argument("--spans", type=Path, required=True)
    v = commands.add_parser("verify")
    v.add_argument("out", type=Path)
    args = parser.parse_args()
    if args.command == "generate":
        if not 0 < args.seed < 2**64:
            raise ValueError("use an explicit positive seed")
        return generate(args)
    if args.command == "assemble":
        return assemble(args)
    if args.command == "verify":
        verify(args.out)
        print("sample reconstruction verified; listening is separate")
        return 0
    script, units = read_units(args.script, args.units)
    print(json.dumps({"cues": len(script), "units": len(units)}))
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except (OSError, ValueError, KeyError, TypeError, wave.Error) as error:
        print(f"coherent-narration: {error}", file=sys.stderr)
        raise SystemExit(1) from error
