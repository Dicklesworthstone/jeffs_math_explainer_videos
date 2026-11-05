#!/usr/bin/env python3
"""What-if check behind the timing recommendations (Mail 4018: "finish your current timing
recommendations"). It writes no answers, no selection and no patch.

Run from the repository root:

    python3 -I -B <folder>/checks/whatif_onsets.py FRESH_OUT_FOLDER

Reads, read-only: the sealed stage09 bundle (its SHA256SUMS is verified; exit 3 on any change), the
report's anchors.json, the binding's pinned script (exit 3 if it differs from the pin), and this
folder's outputs/timing-inspection.json (pinned below; exit 3 if it differs). It imports the sealed
helper only for read_script and kit_sentences, so a mark maps to its sentence exactly as the helper
maps it (sentence n starts at character kit_sentences(text)[n][0]; a mark's character offset is the
decoded length of the text's bytes before it).

For every selectable recording it builds two sets of points:
  current   the recognizer onsets and marks the report already holds;
  adjusted  the same, with every point the inspection classed recognizer_early moved to its
            leave frame (leave_24k), which is what `sample` answers at those values would give.
For each set it checks, in 24 kHz samples (a mark's sample is round(seconds * 24000), as the
report rounds onsets):
  order     0 < start(1) < ... < start(K-1) < length, the helper's refusal in resolved_points,
            where length is ceil(frames / 2) at 24 kHz;
  interval  start(k) <= mark < start(k+1) for the sentence k holding the mark's byte, with
            start(0) = 0 and start(K) = length. This is the L1 rule as Mail 4018 states it. The
            patch is Lavender's; this only counts what the rule would say about these two sets.
It writes whatif.json into FRESH_OUT_FOLDER, which must not exist, and prints a summary.
Arithmetic on recorded values only: not listening, not a heard boundary, not a selection.
"""

import hashlib
import importlib.util
import json
import os
import sys

P = "docs/video-program/third-wave-20261024/opus-replacements-20261104"
BUNDLE = f"{P}/logarithm-stage09-audio-preparation01"
HERE = f"{P}/logarithm-stage09-timing-inspection01"
INSPECTION = f"{HERE}/outputs/timing-inspection.json"
INSPECTION_SHA256 = "7d19519935c529227b42d2baf94f5614c841061526e4be7b2b9f8896b86a81de"
RATE_OUT = 24000


def sha_file(p):
    with open(p, "rb") as f:
        return hashlib.sha256(f.read()).hexdigest()


def verify_seal():
    bad = []
    for line in open(f"{BUNDLE}/SHA256SUMS", encoding="utf8").read().splitlines():
        want, rel = line.split("  ", 1)
        if not os.path.isfile(f"{BUNDLE}/{rel}") or sha_file(f"{BUNDLE}/{rel}") != want:
            bad.append(rel)
    return bad


def check(starts, marks, length):
    """starts: sentence starts for sentences 1..K-1; marks: [(id, k, sample)]."""
    bounds = [0] + starts + [length]
    order = [f"start {i} ({bounds[i]}) not after {bounds[i - 1]}" for i in range(1, len(bounds)) if bounds[i] <= bounds[i - 1]]
    outside = [{"id": mid, "sentence": k, "mark_24k": m, "start_24k": bounds[k], "next_start_24k": bounds[k + 1]}
               for mid, k, m in marks if not bounds[k] <= m < bounds[k + 1]]
    return order, outside


def main():
    if len(sys.argv) != 2 or os.path.exists(sys.argv[1]):
        print("usage: from the repository root, whatif_onsets.py FRESH_OUT_FOLDER (must not exist)", file=sys.stderr)
        return 2
    bad = verify_seal()
    if bad:
        print("STALE: the sealed stage09 bundle differs: " + ", ".join(bad))
        return 3
    if sha_file(INSPECTION) != INSPECTION_SHA256:
        print(f"STALE: {INSPECTION} is not the inspection output this check was written for")
        return 3
    spec = importlib.util.spec_from_file_location("stage09_prepare_audio", f"{BUNDLE}/prepare_audio.py")
    helper = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(helper)
    binding = json.load(open(f"{BUNDLE}/bindings/logarithm-stage09-oliver.json"))
    script_path = f"{binding['tree']}/explainer_corpus/{binding['topic']}/script.tsv"
    if sha_file(script_path) != binding["pins"].get(script_path):
        print(f"STALE: {script_path} differs from the binding's pin")
        return 3
    script = helper.read_script(script_path)
    anchors = json.load(open(f"{BUNDLE}/report/anchors.json"))
    rows = {r["id"]: r for r in json.load(open(INSPECTION))["rows"]}
    early = {i for i, r in rows.items() if r.get("class") == "recognizer_early"}
    result = {"current": {"order": [], "outside": []}, "adjusted": {"order": [], "outside": []}}
    moved_marks_needed = []
    recordings = 0
    for e in anchors["recordings"]:
        recordings += 1
        text = script[e["cue"]]
        spans = helper.kit_sentences(text)
        length = (e["frames"] + 1) // 2
        sents = e["sentences"][1:]
        if len(sents) != len(spans) - 1 or any(s.get("status") != "supported" for s in sents):
            print(f"REFUSED: {e['label']}: sentence rows do not match the script's sentences or are unsupported")
            return 1
        cur_starts = [s["sample_24k"] for s in sents]
        adj_starts = [rows[s["id"]]["leave_24k"] if s["id"] in early else s["sample_24k"] for s in sents]
        cur_marks, adj_marks = [], []
        btext = text.encode()
        for m in e["phrase_anchors"]:
            if m.get("status") != "supported":
                continue
            ch = len(btext[: m["byte"]].decode())
            k = max(n for n, (a, _b) in enumerate(spans) if a <= ch)
            cur = round(m["asr_start_s"] * RATE_OUT)
            adj = rows[m["id"]]["leave_24k"] if m["id"] in early else cur
            cur_marks.append((m["id"], k, cur))
            adj_marks.append((m["id"], k, adj))
            if k >= 1 and sents[k - 1]["id"] in early and adj < adj_starts[k - 1]:
                moved_marks_needed.append({"mark": m["id"], "sentence_onset": sents[k - 1]["id"],
                                           "mark_24k": adj, "adjusted_onset_24k": adj_starts[k - 1]})
        for name, starts, marks in (("current", cur_starts, cur_marks), ("adjusted", adj_starts, adj_marks)):
            order, outside = check(starts, marks, length)
            result[name]["order"] += [f"{e['label']}: {o}" for o in order]
            result[name]["outside"] += outside
    marks_total = sum(1 for e in anchors["recordings"] for m in e["phrase_anchors"] if m.get("status") == "supported")
    summary = {"recordings": recordings, "supported_marks": marks_total,
               "early_points_moved": len(early),
               "early_sentence_onsets": sum(1 for i in early if i.startswith("onset:")),
               "early_marks": sum(1 for i in early if i.startswith("anchor:")),
               **{f"{name}_order_problems": len(result[name]["order"]) for name in result},
               **{f"{name}_marks_outside_their_sentence": len(result[name]["outside"]) for name in result},
               "marks_left_before_a_moved_onset": len(moved_marks_needed)}
    os.makedirs(sys.argv[1])
    with open(f"{sys.argv[1]}/whatif.json", "x", encoding="utf8") as f:
        f.write(json.dumps({"claim": "Arithmetic on recorded values only: not listening, not a selection, not an answers file.",
                            "inspection_sha256": INSPECTION_SHA256, "stage09_seal_sha256": sha_file(f"{BUNDLE}/SHA256SUMS"),
                            "summary": summary, "result": result, "marks_left_before_a_moved_onset": moved_marks_needed},
                           indent=1, ensure_ascii=False) + "\n")
    print(json.dumps(summary, indent=1))
    for name in result:
        for o in result[name]["order"]:
            print(f"{name} order: {o}")
        for o in result[name]["outside"]:
            print(f"{name} outside: {o}")
    for o in moved_marks_needed:
        print(f"mark before moved onset: {o}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
