"""report-compare01.py: do the rebind269 report (b4a4c91e, the 347 captions' pinned input) and the rebind313
report (69cb32ee, the successor delivery02's OLD report) agree on everything the caption timing reads?

cc1-caption-repair347.py / explainer_kit captions.rs read, per cue: each row's sentence, out_samples and
gap_after_samples, and the report's meta.rate. The cue offsets come from the timelines, which are not here.
Also lists every other field that differs, so the difference between the two reports is named.

usage: python3 -I -B report-compare01.py REPORT269 REPORT313 OUT
"""
import hashlib
import json
import sys

A, B, OUT = sys.argv[1:4]
lines = []
say = lines.append
ha, hb = (hashlib.sha256(open(p, "rb").read()).hexdigest() for p in (A, B))
a, b = json.load(open(A)), json.load(open(B))
say("269 %s  313 %s" % (ha, hb))
say("rate %s / %s" % (a["meta"]["rate"], b["meta"]["rate"]))
say("cues %d / %d, same keys %s" % (len(a["lines"]), len(b["lines"]), list(a["lines"]) == list(b["lines"])))
CAP = ("sentence", "out_samples", "gap_after_samples")
cap_diff, other = [], {}
rows = 0
for cue in a["lines"]:
    ra, rb = a["lines"][cue], b["lines"].get(cue, [])
    if len(ra) != len(rb):
        cap_diff.append((cue, "row count %d / %d" % (len(ra), len(rb))))
        continue
    for i, (x, y) in enumerate(zip(ra, rb)):
        rows += 1
        for k in CAP:
            if x.get(k) != y.get(k):
                cap_diff.append((cue, i, k, x.get(k), y.get(k)))
        for k in set(x) | set(y):
            if k not in CAP and x.get(k) != y.get(k):
                other.setdefault(k, []).append("%s[%d]" % (cue, i))
say("sentence rows compared %d; caption-timing fields that differ: %d" % (rows, len(cap_diff)))
for d in cap_diff[:20]:
    say("  DIFF %s" % (d,))
for k in sorted(other):
    say("other field %-20s differs in %3d rows, e.g. %s" % (k, len(other[k]), other[k][:4]))
meta = sorted(k for k in set(a["meta"]) | set(b["meta"]) if a["meta"].get(k) != b["meta"].get(k))
say("meta fields that differ: %s" % meta)
say("RESULT: %s" % ("caption inputs from the two reports are identical" if not cap_diff and a["meta"]["rate"] == b["meta"]["rate"]
                    else "caption inputs differ"))
with open(OUT, "x", encoding="utf-8") as f:
    f.write("\n".join(lines) + "\n")
print("\n".join(lines))
