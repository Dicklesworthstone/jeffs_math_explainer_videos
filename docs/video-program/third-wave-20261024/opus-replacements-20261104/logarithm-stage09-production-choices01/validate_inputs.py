#!/usr/bin/env python3
"""Validate inputs/selection.json and inputs/answers.json with the combined reviewed helper, read-only.

Run from the repository root:

    python3 -I -B docs/video-program/third-wave-20261024/opus-replacements-20261104/logarithm-stage09-production-choices01/validate_inputs.py

The helper is Lavender's combined staged file (L1 + plan binding, 08a26927, reviewed in
logarithm-stage09-mark-interval-review01). Nothing is decoded, written or heard. Steps:
  V1 the helper's CLI `report --check` against the sealed stage09 report (read-only);
  V2 load_binding, then load_selection for the whole film, as decode and prepare call them;
  V3 check_answers, as prepare calls it: applied answers, and no blocking question left open;
  V4 resolved_points with the cue text for every chosen recording at its decoded length ceil(n/2), as
     prepare calls it: the sentence ends strictly increase and every mark lies inside its sentence;
  V5 every applied answer moves its onset later, by the measured amount, and nothing else changes.
"""

import hashlib
import importlib.util
import json
import os
import subprocess
import sys

P = "docs/video-program/third-wave-20261024/opus-replacements-20261104"
HERE = f"{P}/logarithm-stage09-production-choices01"
HELPER = f"{P}/logarithm-stage09-plan-binding-fix01/staged/prepare_audio.py"
HELPER_SHA = "08a269279c47f3a3639ac2458b378135ecca620af9fd246d6b9291eaabd28b14"
STAGE09 = f"{P}/logarithm-stage09-audio-preparation01"
BINDING = f"{STAGE09}/bindings/logarithm-stage09-oliver.json"
SEL, ANS = f"{HERE}/inputs/selection.json", f"{HERE}/inputs/answers.json"
RATE = 24000
results = {"pass": 0, "fail": 0}


def sha_file(p):
    with open(p, "rb") as f:
        return hashlib.sha256(f.read()).hexdigest()


def report(name, ok, detail):
    results["pass" if ok else "fail"] += 1
    print(f"{name} {'PASS' if ok else 'FAIL'}: {detail}")


def main():
    if sha_file(HELPER) != HELPER_SHA:
        print(f"STALE: {HELPER}")
        return 3
    print(f"helper {HELPER_SHA[:8]}; selection {sha_file(SEL)[:8]}; answers {sha_file(ANS)[:8]}")
    r = subprocess.run([sys.executable, "-I", "-B", HELPER, "--binding", BINDING, "report", "--check", f"{STAGE09}/report"],
                       capture_output=True, text=True)
    report("V1", r.returncode == 0, (r.stdout or r.stderr).strip())

    spec = importlib.util.spec_from_file_location("validate_combined_prepare_audio", HELPER)
    h = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(h)
    b = h.load_binding(BINDING)
    _files, questions, recordings, inputs = h.build_report(b)
    try:
        sel, chosen = h.load_selection(b, SEL, inputs, None)
    except h.Refused as e:
        report("V2", False, str(e))
        return 1
    names = {}
    for cue, t in chosen.items():
        names[os.path.basename(t["rel"])[-14:-4]] = names.get(os.path.basename(t["rel"])[-14:-4], 0) + 1
    report("V2", len(chosen) == 80 and b["sha256"] == sel["binding_sha256"],
           f"binding {b['sha256'][:8]}; load_selection accepts {len(chosen)} cues in script order; {names}")

    try:
        applied = h.check_answers(b, ANS, questions, recordings, chosen)
    except h.Refused as e:
        report("V3", False, str(e))
        return 1
    blocking = [q["id"] for q in questions if q["blocking"]]
    open_q = [q["id"] for q in questions if q["id"] not in applied]
    report("V3", all(a["decision"] == "sample" for a in applied.values()),
           f"check_answers applies {len(applied)} sample answers (of {len(json.load(open(ANS))['answers']) - len(questions)} in the file; "
           f"the rest name recordings not selected); blocking questions {blocking} do not touch the selection; "
           f"open advisory questions on the selection: {[q['id'] for q in questions if not q['blocking'] and q['id'] in open_q and set(q['files']) & {os.path.basename(t['rel']) for t in chosen.values()}]}")

    by_file = {(e["cue"], e["file"]): e for e in recordings}
    refused, n_on, n_marks, moved, lead, room = [], 0, 0, [], (9e9, ""), (9e9, "")
    for cue, t in chosen.items():
        entry = by_file[(cue, os.path.basename(t["rel"]))]
        frames24 = h.expected_frames(t)[0]
        try:
            starts, ends, marks = h.resolved_points(entry, applied, frames24, inputs["script"][cue])
        except h.Refused as e:
            refused.append(f"{cue}: {e}")
            continue
        _s0, _e0, marks0 = h.resolved_points(entry, {}, frames24, inputs["script"][cue])
        n_on += len(ends) - 1
        n_marks += len(marks)
        for s, (sample, _why) in zip(entry["sentences"][1:], starts):
            if s["id"] in applied:
                moved.append(sample - s["sample_24k"])
            elif sample != s["sample_24k"]:
                refused.append(f"{s['id']}: changed without an answer")
        if not any(m["id"] in applied for m in entry["phrase_anchors"]) and marks != marks0:
            refused.append(f"{cue}: a mark changed without an answer")
        bounds = [x / RATE for x in [0] + ends[:-1]] + [frames24 / RATE]
        opens = h.sentence_opens(inputs["script"][cue])
        for m in marks:
            j = max(k for k, o in enumerate(opens) if o <= m["byte"])
            lead = min(lead, (round(m["seconds"] - bounds[j], 6), f"{cue} b{m['byte']}"))
            room = min(room, (round(bounds[j + 1] - m["seconds"], 6), f"{cue} b{m['byte']}"))
    report("V4", not refused, f"resolved_points accepts all {len(chosen)} chosen recordings at ceil(n/2): {n_on} onsets, {n_marks} marks; "
                              f"smallest lead of a mark over its sentence onset {lead[0]} s ({lead[1]}), smallest room {room[0]} s ({room[1]}); "
                              f"refused {refused[:2]}")
    report("V5", len(moved) == len(applied) and all(d > 0 for d in moved) and not any(k.startswith("anchor:") for k in applied),
           f"{len(moved)} onsets moved by their answers, all later: {min(moved) / RATE:.2f} to {max(moved) / RATE:.2f} s "
           f"(median {sorted(moved)[len(moved) // 2] / RATE:.2f} s); no mark answer applies to the selection")
    print(f"RESULT {'PASS' if results['fail'] == 0 else 'FAIL'}: {results['pass']} pass, {results['fail']} fail")
    return 0 if results["fail"] == 0 else 1


if __name__ == "__main__":
    sys.exit(main())
