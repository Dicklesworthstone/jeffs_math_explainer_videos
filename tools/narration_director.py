#!/usr/bin/env python3
"""Script-first cadence proposals compared with FW words and native cue timing.

This is a read-only diagnostic, not a naturalness gate or an audio editor.
`plan --manifest INPUT.json --out MAP.json` drafts text-only boundary proposals.
Review semantic roles/ranges (or add exact phrase overrides) before `audit`.
`audit --manifest INPUT.json --map MAP.json --out NEW_DIRECTORY` emits JSON and
a local video review page. Uses prosody_qa's existing waveform primitives.
Run with `uv run --no-project --with numpy python3 -I tools/narration_director.py`.

Manifest: {videos:[{id,script,report,timeline,master,master_sha256,asr:[paths]}]}.
Map: {schema_version:1,scripts:{id:sha256},policy:{role:[min,max]},overrides:[]}
Overrides: {video,cue,after:"unique exact phrase",role,range_s:[min,max],reason}.
Ranges are editorial hypotheses. No score can certify listening acceptance.
"""
from __future__ import annotations

import argparse
import difflib
import hashlib
import html
import importlib.util
import json
import math
from pathlib import Path
import re
import sys
import wave

import numpy as np

sys.dont_write_bytecode = True
_spec = importlib.util.spec_from_file_location("prosody_qa", Path(__file__).with_name("prosody_qa.py"))
qa = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(qa)

WORDS = re.compile(r"[A-Za-z0-9]+(?:['’][A-Za-z]+)?")
# Uncalibrated starting proposals, deliberately distinct from acceptance gates.
POLICY = {"connected": [0.0, 0.28], "clause": [0.12, 0.65],
          "thought": [0.35, 1.1], "question": [0.65, 1.65],
          "cue_transition": [0.45, 1.4], "chapter_transition": [0.7, 2.2]}


def digest(path):
    with Path(path).open("rb") as handle:
        return hashlib.file_digest(handle, "sha256").hexdigest()


def load(path):
    return json.loads(Path(path).read_text())


def write_new(path, data):
    with Path(path).open("x") as handle:
        json.dump(data, handle, indent=2, allow_nan=False)
        handle.write("\n")


def tokens(text):
    return [m.group().lower().replace("’", "'") for m in WORDS.finditer(text)]


def integer_words(n):
    small = "zero one two three four five six seven eight nine ten eleven twelve thirteen fourteen fifteen sixteen seventeen eighteen nineteen".split()
    tens = "zero ten twenty thirty forty fifty sixty seventy eighty ninety".split()
    if n < 20:
        return [small[n]]
    if n < 100:
        return [tens[n // 10]] + (integer_words(n % 10) if n % 10 else [])
    for value, name in ((1000000000, "billion"), (1000000, "million"), (1000, "thousand"), (100, "hundred")):
        if n >= value:
            return integer_words(n // value) + [name] + (integer_words(n % value) if n % value else [])
    raise ValueError("invalid integer")


def asr_words(result):
    """Expand written integers without inventing sub-word timestamps."""
    words = []
    for group, seg in enumerate(result["segments"]):
        a, b = float(seg["start_sec"]), float(seg["end_sec"])
        if not math.isfinite(a + b) or a < 0 or b < a:
            raise ValueError("invalid ASR timestamp")
        text = re.sub(r"(?<=\d),(?=\d)", "", seg["text"])
        ts = tokens(text)
        expanded = [w for t in ts for w in (integer_words(int(t)) if t.isdigit() and len(t) <= 10 else [t])]
        for token in expanded:
            words.append({"text": token, "start": a, "end": b, "group": group,
                          "coarse": len(expanded) != 1})
    if any(b["start"] < a["start"] - 0.001 for a, b in zip(words, words[1:])):
        raise ValueError("nonmonotone ASR timestamps")
    return words


def align(expected, observed):
    """Only exact matching blocks. Unmatched words stay unmatched."""
    mapping = {}
    for a, b, n in difflib.SequenceMatcher(a=expected, b=observed, autojunk=False).get_matching_blocks():
        for j in range(n):
            mapping[a + j] = b + j
    return mapping


def draft_map(manifest):
    return {"schema_version": 1, "scripts": {v["id"]: digest(v["script"]) for v in manifest["videos"]},
            "policy": POLICY, "overrides": [],
            "claim": "Text-only heuristic proposals. Author context-specific overrides before repair; no listening acceptance."}


def validate_map(manifest, direction):
    if direction.get("schema_version") != 1:
        raise ValueError("unsupported director map")
    for v in manifest["videos"]:
        if direction["scripts"].get(v["id"]) != digest(v["script"]):
            raise ValueError("director map binds a different script: " + v["id"])
    ids = [v["id"] for v in manifest["videos"]]
    if len(ids) != len(set(ids)) or any(r["video"] not in ids for r in direction["overrides"]):
        raise ValueError("duplicate video or override for unknown video")
    seen_holds = set()
    for hold in direction.get('visual_holds', []):
        key = (hold['video'], hold['cue'], hold['next_cue'])
        if hold['video'] not in ids or key in seen_holds or not hold.get('reason'):
            raise ValueError('invalid or duplicate visual hold')
        seen_holds.add(key)
        if digest(hold['evidence']['path']) != hold['evidence']['sha256']:
            raise ValueError('visual hold evidence changed')
        video = next(v for v in manifest['videos'] if v['id'] == hold['video'])
        cues = list(qa.read_tsv(video['script']))
        if (hold['cue'], hold['next_cue']) not in set(zip(cues,cues[1:])):
            raise ValueError('visual hold does not name adjacent cues')
    if set(POLICY) - direction["policy"].keys():
        raise ValueError("missing default boundary policy")
    for bounds in [*direction["policy"].values(), *[r["range_s"] for r in direction["overrides"]]]:
        if len(bounds) != 2 or not all(math.isfinite(n) for n in bounds) or not 0 <= bounds[0] <= bounds[1] <= 30:
            raise ValueError("invalid proposed pause range")


def directions(video, script, direction):
    rows = {}
    cues = list(script)
    for ci, (cue, text) in enumerate(script.items()):
        matches = list(WORDS.finditer(text))
        for i, word in enumerate(matches):
            punctuation = text[word.end():matches[i + 1].start() if i + 1 < len(matches) else len(text)]
            role = "connected"
            if "?" in punctuation:
                role = "question"
            elif any(p in punctuation for p in ".!"):
                role = "thought"
            elif any(p in punctuation for p in ",;:—–"):
                role = "clause"
            if i + 1 == len(matches) and ci + 1 < len(cues):
                if cues[ci + 1][:2] != cue[:2]:
                    role = "chapter_transition"
                elif role != "question":
                    role = "cue_transition"
            rows[(cue, i)] = {"role": role, "range_s": direction["policy"][role],
                              "reason": "Text/punctuation default; semantic review pending", "basis": "heuristic"}
    used = set()
    for override in direction["overrides"]:
        if override["video"] != video:
            continue
        cue = override["cue"]
        text = script[cue]
        phrase = tokens(override["after"])
        ts = tokens(text)
        hits = [i + len(phrase) - 1 for i in range(len(ts) - len(phrase) + 1) if ts[i:i + len(phrase)] == phrase]
        if "word_index" in override:
            hits = [i for i in hits if i == override["word_index"]]
        if len(hits) != 1 or not phrase or not override.get("reason"):
            raise ValueError(f"override must name one unambiguous phrase and reason: {video}/{cue}/{phrase}")
        key = (cue, hits[0])
        if key in used or key not in rows:
            raise ValueError("duplicate or missing override boundary")
        used.add(key)
        rows[key] = {k: override[k] for k in ("role", "range_s", "reason")}
        rows[key]["basis"] = "authored"
    return rows


def native_timeline(directory, report):
    """Integer frames plus exact WAV lengths; never rounded cue TSV durations."""
    cues, windows, offset, inputs = {}, [], 0.0, []
    for path in sorted(Path(directory).glob("*.timeline.tsv")):
        fields = [line.split("\t") for line in path.read_text().splitlines()]
        fps = int(next(r[1] for r in fields if r[0] == "fps"))
        frames = int(next(r[1] for r in fields if r[0] == "frames"))
        if fps <= 0 or frames <= 0:
            raise ValueError("invalid native chapter timing")
        inputs.append({"path": str(path.resolve()), "sha256": digest(path)})
        for row in fields:
            if row[0] == "line":
                cue, start, wav = row[1], offset + int(row[2]) / fps, Path(row[4])
                if cue in cues or cue not in report["lines"]:
                    raise ValueError("duplicate or unknown native cue")
                with wave.open(str(wav)) as handle:
                    length = handle.getnframes() / handle.getframerate()
                cues[cue] = {"start": start, "duration": length, "wav": str(wav),
                             "chapter": path.stem.split(".")[0]}
            elif row[0] in ("listen", "breath"):
                a = offset + int(row[1]) / fps
                windows.append((a, a + float(row[2]), row[0]))
            elif row[0] == "mark" and row[1] == "predict":
                a = offset + int(row[2]) / fps
                windows.append((a, a, "predict"))
        offset += frames / fps
    if set(cues) != set(report["lines"]):
        raise ValueError("native timeline does not cover every narration cue")
    return cues, windows, offset, inputs


def quiet_runs(x, rate, relative):
    times, db = qa.levels(x, rate)
    if not len(db):
        return []
    threshold = min(-38.0, float(np.percentile(db, 95)) - relative)
    return [(max(0.0, float(times[a] - qa.STEP)), min(len(x) / rate, float(times[b - 1] + qa.STEP)))
            for a, b in qa.runs_of(db < threshold) if (b - a) * qa.STEP >= 0.03]


def edge_quiet(runs, duration, tail):
    if tail:
        return next((duration - a for a, b in reversed(runs) if b >= duration - 0.025), 0.0)
    return next((b for a, b in runs if a <= 0.025), 0.0)


def boundary_quiet(runs, left, right):
    # ASR smears silence into adjacent words. Locate nearby low-energy candidates,
    # retaining their endpoints and sensitivity range, without declaring safe cuts.
    mid = (left["end"] + right["start"]) / 2
    candidates = [(a, b) for a, b in runs if a >= left["start"] - 0.03 and b <= right["end"] + 0.03
                  and a <= mid + 0.18 and b >= mid - 0.18]
    if not candidates:
        return 0.0, None
    a, b = min(candidates, key=lambda r: abs((r[0] + r[1]) / 2 - mid))
    return b - a, [a, b]


def classify(observed, proposed, protected=False, uncertain=False):
    if protected:
        return "protected_visual_hold"
    if uncertain:
        return "alignment_review"
    lo, hi = observed
    if hi < proposed[0] - 0.08:
        return "missing_pause"
    if lo > proposed[1] + 0.12:
        return "unexpected_pause"
    return "within_proposal"


def audit_video(video, direction):
    script = qa.read_tsv(video["script"])
    report = load(video["report"])
    rate = report["meta"]["rate"]
    if report["meta"]["script_sha256"] != digest(video["script"]):
        raise ValueError("narration report/script mismatch")
    if digest(video["master"]) != video["master_sha256"]:
        raise ValueError("review master changed")
    native, windows, duration, inputs = native_timeline(video["timeline"], report)
    inputs += [{"path": str(Path(video[k]).resolve()), "sha256": digest(video[k])} for k in ("script", "report", "master")]
    by_source, source_hashes = {}, {}
    for cue, sentences in report["lines"].items():
        if " ".join(s["sentence"] for s in sentences) != script[cue]:
            raise ValueError("report does not reproduce canonical cue text")
        cursor = 0
        cue_word = 0
        for sentence in sentences:
            source = str(Path(sentence["source"]).resolve())
            if source in source_hashes and source_hashes[source] != sentence["source_sha256"]:
                raise ValueError("contradictory source hashes")
            source_hashes[source] = sentence["source_sha256"]
            if sentence.get("gap_after_samples", 0) < 0 or sentence["out_samples"] != sentence["source_out_sample"] - sentence["source_in_sample"]:
                raise ValueError("source map contains a trim or inconsistent sample interval")
            count = len(tokens(sentence["sentence"]))
            row = dict(sentence, cue=cue, cue_start=cursor / rate, cue_word=cue_word)
            by_source.setdefault(source, []).append(row)
            cursor += sentence["out_samples"] + sentence.get("gap_after_samples", 0)
            cue_word += count
        if abs(cursor / rate - native[cue]["duration"]) > 1 / rate:
            raise ValueError("cue duration differs from source map")
    asr = {}
    for path in video["asr"]:
        inputs.append({"path": str(Path(path).resolve()), "sha256": digest(path)})
        for line in Path(path).read_text().splitlines():
            row = json.loads(line)
            source = str(Path(row["input"]).resolve())
            if source in by_source and row["status"] == "ok":
                if source in asr and asr[source]["result"] != row["report"]["result"]:
                    raise ValueError("ambiguous ASR reports; select one explicitly")
                asr[source] = {"result": row["report"]["result"], "path": str(path)}
    words, alignment = {}, []
    for source, parts in by_source.items():
        if digest(source) != source_hashes[source]:
            raise ValueError("source waveform changed: " + source)
        inputs.append({"path": source, "sha256": source_hashes[source]})
        if source not in asr:
            alignment.append({"source": source, "status": "missing_asr",
                              "expected_words": sum(len(tokens(p["sentence"])) for p in parts), "matched_words": 0})
            continue
        expected = [w for part in parts for w in tokens(part["sentence"])]
        observed = asr_words(asr[source]["result"])
        mapping = align(expected, [w["text"] for w in observed])
        alignment.append({"source": source, "asr": asr[source]["path"], "expected_words": len(expected),
                          "matched_words": len(mapping), "status": "exact" if expected == [w["text"] for w in observed] else "partial",
                          "expected": " ".join(p["sentence"] for p in parts), "observed": asr[source]["result"]["transcript"],
                          "binding": "historical ASR path linked to currently hash-verified source; ASR did not emit an input hash"})
        index = 0
        for part in parts:
            a, b = part["source_in_sample"] / rate, part["source_out_sample"] / rate
            for j, token in enumerate(tokens(part["sentence"])):
                if index in mapping:
                    obs = observed[mapping[index]]
                    context = sum(mapping.get(k) == mapping[index] + k - index for k in range(max(0, index - 2), min(len(expected), index + 3)))
                    valid = obs["start"] >= a - 0.08 and obs["end"] <= b + 0.08 and context >= min(3, len(expected))
                    words[(part["cue"], part["cue_word"] + j)] = dict(obs, source=source, expected=token,
                        start=obs["start"] - a + part["cue_start"], end=obs["end"] - a + part["cue_start"],
                        observed_index=mapping[index], valid=valid)
                index += 1
    signals = {}
    for cue, n in native.items():
        x, sr = qa.read_wav(n["wav"])
        if sr != rate:
            raise ValueError("cue sample rate changed")
        # Reconstruct exactly; prevents a same-duration changed cue from being scored.
        with wave.open(n["wav"]) as w:
            if w.getnchannels() != 1 or w.getsampwidth() != 2:
                raise ValueError("unsupported cue format")
            actual = w.readframes(w.getnframes())
        inputs.append({"path": n["wav"], "sha256": digest(n["wav"])})
        rebuilt = bytearray()
        for part in report["lines"][cue]:
            with wave.open(part["source"]) as w:
                if w.getnchannels() != 1 or w.getsampwidth() != 2 or w.getframerate() != rate:
                    raise ValueError("unsupported source format")
                w.setpos(part["source_in_sample"])
                rebuilt.extend(w.readframes(part["source_out_sample"] - part["source_in_sample"]))
            rebuilt.extend(b"\0\0" * part.get("gap_after_samples", 0))
        if actual != rebuilt:
            raise ValueError("cue PCM does not reconstruct from source: " + cue)
        signals[cue] = {rel: quiet_runs(x, sr, rel) for rel in (24, 30)}
    proposed = directions(video["id"], script, direction)
    holds = {(h['cue'], h['next_cue']): h for h in direction.get('visual_holds',[]) if h['video']==video['id']}
    for hold in holds.values():
        inputs.append({k:hold['evidence'][k] for k in ('path','sha256')})
    sequence = [(cue, i) for cue, text in script.items() for i in range(len(tokens(text)))]
    boundaries = []
    for left_key, right_key in zip(sequence, sequence[1:]):
        cue, index = left_key
        nxt, _ = right_key
        d = proposed[left_key]
        row = {"video": video["id"], "cue": cue, "word_index": index, "next_cue": nxt, **d,
               "left_context": " ".join(tokens(script[cue])[max(0, index - 7):index + 1]),
               "right_context": " ".join(tokens(script[nxt])[right_key[1]:right_key[1] + 8])}
        left, right = words.get(left_key), words.get(right_key)
        if not left or not right or not left["valid"] or not right["valid"]:
            row.update(kind="alignment_review", time=native[cue]["start"], observed_s=None,
                       severity=0.0, uncertainty="unmatched/coarse boundary or sentence-edge timing conflict")
            boundaries.append(row)
            continue
        cross = cue != nxt
        time = native[cue]["start"] + left["end"]
        scene = native[nxt]["start"] - native[cue]["start"] - native[cue]["duration"] if cross else 0.0
        protected = cross and any(time - 0.1 <= a <= native[nxt]["start"] + 0.05 for a, _, _ in windows)
        hold = holds.get((cue,nxt)) if cross else None
        if hold:
            protected = True
            row['visual_hold'] = hold
        estimates, intervals = [], []
        for rel in (24, 30):
            if cross:
                a = edge_quiet(signals[cue][rel], native[cue]["duration"], True)
                b = edge_quiet(signals[nxt][rel], native[nxt]["duration"], False)
                estimates.append(a + scene + b)
                intervals.append(None)
            else:
                length, interval = boundary_quiet(signals[cue][rel], left, right)
                estimates.append(length)
                intervals.append(interval)
        observed_range = [min(estimates), max(estimates)]
        uncertain = (left["source"] == right["source"] and (right["observed_index"] != left["observed_index"] + 1
                     or left["group"] == right["group"]))
        if cross and scene < -1 / rate:
            kind = "overlapping_cues"
        else:
            kind = classify(observed_range, d["range_s"], protected, uncertain)
            if cross and kind == 'unexpected_pause':
                kind = 'scene_wait_review'
        deficit = max(0.0, d["range_s"][0] - observed_range[1])
        excess = max(0.0, observed_range[0] - d["range_s"][1])
        severity = max(deficit, excess) * (2 if d["basis"] == "authored" else 1)
        row.update(kind=kind, time=time, observed_s=observed_range,
                   asr_gap_s=native[nxt]["start"] + right["start"] - time,
                   scene_added_s=scene, waveform_candidates=intervals,
                   same_source=left["source"] == right["source"], severity=severity,
                   counterfactual_without_scene_s=[v - scene for v in observed_range],
                   uncertainty="low-energy proxy at two thresholds; consonants/breaths and ASR alignment need inspection")
        boundaries.append(row)
    pace = []
    for cue, text in script.items():
        matched = [words.get((cue, i)) for i in range(len(tokens(text)))]
        matched = [w for w in matched if w and w["valid"]]
        if len(matched) < 4:
            continue
        lo, hi = matched[0]["start"], matched[-1]["end"]
        if hi <= lo:
            continue
        quiet = sum(max(0, min(b, hi) - max(a, lo)) for a, b in signals[cue][30])
        articulation = max(0.1, hi - lo - quiet)
        number_words = set("zero one two three four five six seven eight nine ten hundred thousand million billion half quarter tenth fifth percent equals minus plus divided square probability delta epsilon".split())
        weight = sum(t in number_words for t in tokens(text))
        pace.append({"cue": cue, "time": native[cue]["start"] + lo, "text": text,
                     "word_coverage": len(matched) / len(tokens(text)),
                     "words_per_active_minute": len(tokens(text)) * 60 / articulation,
                     "technical_tokens": weight, "technical_tokens_per_active_second": weight / articulation})
    base = float(np.median([p["words_per_active_minute"] for p in pace])) if pace else None
    for row in pace:
        row["relative_to_own_video_median"] = row["words_per_active_minute"] / base if base else None
        row["kind"] = "rushed_technical_candidate" if (row["word_coverage"] >= .85 and row["technical_tokens"] >= 4
                       and row["relative_to_own_video_median"] > 1.15) else "rate_observation"
    counts = {kind: sum(row["kind"] == kind for row in boundaries) for kind in sorted({row["kind"] for row in boundaries})}
    coverage = {"expected_words": len(sequence), "exact_matched_words": len(words),
                "usable_timed_words": sum(w["valid"] for w in words.values()),
                "missing_asr_sources": sum(a["status"] == "missing_asr" for a in alignment)}
    # An audit must not quietly bind changing inputs in a shared checkout.
    for item in inputs:
        if digest(item["path"]) != item["sha256"]:
            raise ValueError("input changed during audit: " + item["path"])
    return {"id": video["id"], "master": video["master"], "master_sha256": video["master_sha256"],
            "duration": duration, "cue_count": len(script), "inputs": inputs, "alignment": alignment,
            "boundaries": boundaries, "pace": pace, "counts": counts, "coverage": coverage,
            "median_words_per_active_minute": base,
            "claim": "Diagnostic proposals only. No audio was edited and no listening acceptance is inferred."}


def review_page(results):
    cards = []
    for video in results:
        candidates = [r for r in video["boundaries"] if r["kind"] in ("missing_pause", "unexpected_pause", "overlapping_cues", "scene_wait_review")]
        findings = sorted((r for r in candidates if r['basis'] == 'authored'), key=lambda r:r['severity'],reverse=True)[:8]
        findings += sorted((r for r in candidates if r['basis'] != 'authored'),key=lambda r:r['severity'],reverse=True)[:4]
        buttons = []
        for row in findings:
            t = max(0, row["time"] - 5)
            label = f"{int(row['time']//60)}:{int(row['time']%60):02d} · {row['kind'].replace('_',' ')} · {row['cue']}"
            measured = f"{row['observed_s'][0]:.2f}–{row['observed_s'][1]:.2f}s; proposed {row['range_s'][0]}–{row['range_s'][1]}s"
            buttons.append(f'<li><button data-time="{t:.3f}">{html.escape(label)}</button> '
                f'{html.escape(measured)}<p>{html.escape(row["left_context"])} <b>|</b> {html.escape(row["right_context"])}</p>'
                f'<small>{html.escape(row["basis"] + ": " + row["reason"])} '
                f'Assembly wait: {row["scene_added_s"]:.2f}s. '
                f'{"Fresh master timestamp crosscheck available." if row.get("master_excerpt_checks") else "Projected from source-take ASR."}</small></li>')
        bursts = []
        for row in video['pace']:
            if row['kind'] != 'rushed_technical_candidate':
                continue
            label = f"{int(row['time']//60)}:{int(row['time']%60):02d} · {row['cue']} · dense/fast candidate"
            bursts.append(f'<li><button data-time="{max(0,row["time"]-2):.3f}">{html.escape(label)}</button>'
                          f'<p>{html.escape(row["text"])}</p><small>{row["relative_to_own_video_median"]:.2f}× this video’s median active-word-rate proxy; {row["technical_tokens"]} technical tokens. This is a triage heuristic, not a speed limit.</small></li>')
        coverage=video['coverage']
        coverage_text=f"{video['cue_count']} cues; {coverage['usable_timed_words']} of {coverage['expected_words']} words have usable projected timing. {video['counts'].get('alignment_review',0)} boundaries require alignment review."
        cards.append(f'<section><h2>{html.escape(video["id"])}</h2><video controls preload="metadata" src="{html.escape(Path(video["master"]).as_uri())}"></video>'
                     f'<p>{coverage_text} <a href="{html.escape(Path(video["master"]).as_uri())}">Open original cut</a></p>'
                     f'<h3>Pause candidates: semantic direction first, then long gaps</h3><ol>{"".join(buttons)}</ol>'
                     f'<h3>Dense delivery</h3><ul>{"".join(bursts) if bursts else "<li>No cue met this conservative relative-rate heuristic; this does not certify the pacing.</li>"}</ul></section>')
    return '''<!doctype html><meta charset="utf-8"><title>Narration direction review</title>
<style>body{font:17px system-ui;max-width:1000px;margin:40px auto;background:#111821;color:#edf1f5;padding:20px}a{color:#9bd3ef}video{width:100%;max-height:480px}section{margin:50px 0}li{margin:25px 0}p{line-height:1.5}button{font:inherit;background:#294662;color:white;border:1px solid #648aad;padding:8px;cursor:pointer}small{color:#acbdce}</style>
<h1>Narration: measured timing versus intended phrasing</h1>
<p>Original delivered videos. These are ranked diagnostic candidates, not confirmed audible defects. Pause ranges are editorial proposals. Known visual holds are protected. A scene-wait flag calls for inspecting its visual task; it is not a request to shorten it. No speech has been changed.</p>''' + "".join(cards) + '''
<script>document.querySelectorAll('button').forEach(b=>b.onclick=()=>{const v=b.closest('section').querySelector('video');document.querySelectorAll('video').forEach(x=>x.pause());v.currentTime=Number(b.dataset.time);v.play();});</script>'''


def verify_master_excerpts(manifest, results):
    """Compare projected take timings with fresh ASR on the actual scored master.

    Same-engine, different-input/context corroboration is not independent hearing.
    Moving timestamps lower localization confidence; they do not prove a defect.
    """
    verification = manifest.get("verification")
    if not verification:
        return {"status": "not_requested", "boundaries": []}
    receipt = load(verification["receipt"])
    raw = Path(verification["raw"])
    if receipt["exit"] != 0 or digest(raw) != receipt["raw_sha256"]:
        raise ValueError("master verification failed or changed")
    entries = [json.loads(line) for line in raw.read_text().splitlines()]
    by_path = {str(Path(r["input"]).resolve()): r for r in entries}
    if len(by_path) != len(entries) or len(entries) != len(receipt["excerpts"]):
        raise ValueError("missing or duplicate master excerpt result")
    videos = {v["id"]: v for v in manifest["videos"]}
    output = {v["id"]: v for v in results}
    checks = []
    for excerpt in receipt["excerpts"]:
        v = videos[excerpt["video"]]
        if excerpt["master_sha256"] != v["master_sha256"] or digest(excerpt["excerpt"]) != excerpt["excerpt_sha256"]:
            raise ValueError("excerpt does not bind this master or input")
        row = by_path[str(Path(excerpt["excerpt"]).resolve())]
        if row["status"] != "ok":
            raise ValueError("master excerpt ASR was not successful")
        observed = asr_words(row["report"]["result"])
        script = qa.read_tsv(v["script"])
        timeline, _, _, _ = native_timeline(v["timeline"], load(v["report"]))
        a, b = excerpt["master_start_s"], excerpt["master_start_s"] + excerpt["duration_s"]
        keys, expected = [], []
        for cue, text in script.items():
            n = timeline[cue]
            if n["start"] + n["duration"] < a or n["start"] > b:
                continue
            for i, token in enumerate(tokens(text)):
                keys.append((cue, i))
                expected.append(token)
        mapping = align(expected, [w["text"] for w in observed])
        indexes = {key: i for i, key in enumerate(keys)}
        for boundary in output[v["id"]]["boundaries"]:
            if boundary["observed_s"] is None or not a + .5 < boundary["time"] < b - .5:
                continue
            index = indexes.get((boundary["cue"], boundary["word_index"]))
            if index is None or index not in mapping or index + 1 not in mapping:
                continue
            left, right = observed[mapping[index]], observed[mapping[index+1]]
            if mapping[index+1] != mapping[index]+1 or left["group"] == right["group"]:
                continue
            context = range(max(0,index-2), min(len(expected),index+4))
            if sum(mapping.get(k) == mapping[index] + k - index for k in context) < min(4,len(expected)):
                continue
            fresh = [a + left["end"], a + right["start"]]
            prior = [boundary["time"], boundary["time"] + boundary["asr_gap_s"]]
            drift = max(abs(x-y) for x,y in zip(fresh,prior))
            evidence = {"excerpt": excerpt["excerpt"], "fresh_boundary_edges": fresh,
                        "projected_boundary_edges": prior, "max_edge_shift_s": drift,
                        "status": "timing_stable_within_250ms" if drift <= .25 else "timestamp_context_sensitive"}
            prior_kind = boundary["kind"]
            boundary.setdefault("master_excerpt_checks", []).append(evidence)
            if drift > .25 and boundary["cue"] == boundary["next_cue"]:
                boundary.setdefault("pre_crosscheck_kind", prior_kind)
                boundary["kind"] = "alignment_review"
                boundary["uncertainty"] += "; ASR boundary moved over 250 ms with excerpt/mix context; localization requires review"
            checks.append(dict(video=v["id"],cue=boundary["cue"],word_index=boundary["word_index"],
                               prior_kind=prior_kind,**evidence))
    for result in results:
        result["counts"] = {kind: sum(r["kind"] == kind for r in result["boundaries"])
                            for kind in sorted({r["kind"] for r in result["boundaries"]})}
    return {"status": "diagnostic_crosscheck", "receipt_sha256": digest(verification["receipt"]),
            "raw_sha256": digest(raw), "boundaries": checks,
            "claim": "Fresh FW on scored master excerpts checks localization sensitivity; same engine, not listening acceptance."}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("command", choices=("plan", "audit"))
    parser.add_argument("--manifest", type=Path, required=True)
    parser.add_argument("--map", type=Path)
    parser.add_argument("--out", type=Path, required=True)
    args = parser.parse_args()
    manifest = load(args.manifest)
    if args.command == "plan":
        write_new(args.out, draft_map(manifest))
        return
    if not args.map:
        parser.error("audit requires --map")
    direction = load(args.map)
    validate_map(manifest, direction)
    # Resolve every authored location before reading media or creating outputs.
    for video in manifest["videos"]:
        directions(video["id"], qa.read_tsv(video["script"]), direction)
    args.out.mkdir(parents=True, exist_ok=False)
    results = []
    for video in manifest["videos"]:
        result = audit_video(video, direction)
        results.append(result)
        print(json.dumps({"video": video["id"], "cues": result["cue_count"], "counts": result["counts"]}), flush=True)
    verification = verify_master_excerpts(manifest, results)
    for result in results:
        write_new(args.out / (result["id"] + ".json"), result)
    write_new(args.out / "master-verification.json", verification)
    write_new(args.out / "summary.json", {"manifest_sha256": digest(args.manifest), "map_sha256": digest(args.map),
        "tool_sha256": digest(__file__), "measurement_tool_sha256": digest(Path(__file__).with_name("prosody_qa.py")),
        "videos": [{k: r[k] for k in ("id", "cue_count", "duration", "counts", "coverage")} for r in results],
        "status": "diagnostic_candidates_only", "listening": "not_performed"})
    with (args.out / "review.html").open("x") as handle:
        handle.write(review_page(results))


if __name__ == "__main__":
    main()
