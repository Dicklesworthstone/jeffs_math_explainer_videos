#!/usr/bin/env python3
"""Run from the repo root. Where each bound chapter 11 hold lands after the patch.

A hold row names a file hash, a line and a marker. This walks each file from the cue's say to the next say.
Pass a further Rust file as argv[1] to report it too.
"""
import hashlib
import json
import re
import sys

ROOT = "docs/video-program/third-wave-20261024/opus-replacements-20261104/"
B = ROOT + "algebra-picture-review-11_permutations/"
MAP = "explainer_corpus/abstract_algebra_expanded/docs/narration-direction-map-oliver24.json"
FILES = [("baseline", B + "baseline/permutations.rs"), ("candidate", B + "candidate/permutations.rs")]
if len(sys.argv) > 1:
    FILES.append(("follow-up", sys.argv[1]))
rows = [r for r in json.load(open(MAP, encoding="utf-8"))["visual_holds"] if r["cue"].startswith("11_")]
print("bound sha", {r["sha256"] for r in rows})
for label, path in FILES:
    data = open(path, "rb").read()
    lines = data.decode("utf-8").splitlines()
    print(f"\n== {label}: sha256 {hashlib.sha256(data).hexdigest()}")
    says = [(i + 1, m.group(1)) for i, row in enumerate(lines) if (m := re.search(r'k\.say\(stage, "(11_\d+)"\)', row))]
    holds = [i + 1 for i, row in enumerate(lines) if row.strip() == "k.hold(stage)?;"]
    print("k.hold lines in file:", holds)
    for r in rows:
        at = next(n for n, c in says if c == r["cue"])
        nxt = next(n for n, c in says if c == r["next_cue"])
        inside = [h for h in holds if at < h < nxt]
        ok = len(inside) == 1
        waits = lines[inside[0]].strip() if ok else ""
        print(f"  {r['cue']}->{r['next_cue']} ({r['kind']}): say {at}, hold {inside}, next say {nxt}; bound line {r['line']} {'still' if inside == [r['line']] else 'moved by ' + str(inside[0] - r['line']) if ok else 'BROKEN'}; line after the hold: {waits!r}")
    intervals = {}
    for r in rows:
        at = next(n for n, c in says if c == r["cue"])
        nxt = next(n for n, c in says if c == r["next_cue"])
        intervals[r["cue"]] = "\n".join(lines[at - 1 : nxt])
    if label == "baseline":
        first_intervals = intervals
    print("  say-to-next-say text unchanged from baseline, per row:", {c: intervals[c] == first_intervals[c] for c in intervals})
    a = next(n for n, c in says if c == "11_24")
    b = next(n for n, c in says if c == "11_25")
    print("  11_24->11_25: k.hold between its says:", [h for h in holds if a < h < b], "| map row:", any(r["cue"] == "11_24" for r in rows))
