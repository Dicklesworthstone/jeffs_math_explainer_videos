"""cue-binding01.py: do the 579 delivery captions (347) still sit on the successor's schedule?

Text-only reads, no media:
1. maps every caption of the 571 native export and of the 579 delivery set to its script cue (word walk over
   script.tsv, which both sets equal per cue), and checks that each cue's first start and last end agree;
2. checks every cue's first caption start against master993's chapter start + the cue's timeline frame:
   for 01/08/09 from root's combined01 timelines (the successor's), for every other chapter from master993's own
   timeline269 copies where present locally;
3. compares the combined01 01/09 timelines row by row with the timeline269 tables the 347 tool pins
   (narration paths reduced to file names).

usage: python3 -I -B cue-binding01.py PR17_DIR SCRIPT_TSV OUT
"""
import hashlib
import json
import re
import sys
from pathlib import Path

D, SCRIPT, OUT = Path(sys.argv[1]), Path(sys.argv[2]), sys.argv[3]
lines, bad = [], []
say = lines.append
FPS = 60


def sha(p):
    return hashlib.sha256(Path(p).read_bytes()).hexdigest()


def srt(p):
    out = []
    for block in Path(p).read_text(encoding="utf-8").strip().split("\n\n"):
        rows = block.split("\n")
        a, b = re.findall(r"(\d+):(\d+):(\d+),(\d+)", rows[1])
        ms = [int(h) * 3600000 + int(m) * 60000 + int(s) * 1000 + int(x) for h, m, s, x in (a, b)]
        out.append((ms[0], ms[1], " ".join(rows[2:])))
    return out


def cues():
    order = []
    for row in SCRIPT.read_text(encoding="utf-8").splitlines():
        if re.match(r"^\d\d_\d+\t", row):
            cue, text = row.split("\t", 1)
            order.append((cue, text.split()))
    return order


def bind(caps, order):
    """cue -> (first start, last end, captions)"""
    out, i = {}, 0
    for cue, words in order:
        got, first = [], i
        while len(got) < len(words):
            got += caps[i][2].split()
            i += 1
        if got != words:
            bad.append("%s: captions do not end on the cue's last word" % cue)
        out[cue] = (caps[first][0], caps[i - 1][1], i - first)
    if i != len(caps):
        bad.append("%d captions left over" % (len(caps) - i))
    return out


def timeline(p):
    rows = {}
    for row in Path(p).read_text().splitlines():
        cols = row.split("\t")
        if cols[0] == "line":
            rows[cols[1]] = int(cols[2])
    return rows


def normal(p):
    return [("\t".join(r.split("\t")[:-1] + [r.split("\t")[-1].rsplit("/", 1)[-1]]) if r.startswith("line\t") else r)
            for r in Path(p).read_text().splitlines()]


native, delivery = D / "complex-analysis-master993.en.srt", D / "cc1-caption-repair347-candidate.en.srt"
say("native  %s %s" % (native.name, sha(native)))
say("delivery %s %s" % (delivery.name, sha(delivery)))
say("script %s" % sha(SCRIPT))
order = cues()
n, d = srt(native), srt(delivery)
bn, bd = bind(n, order), bind(d, order)
say("cues %d; native captions %d, delivery captions %d" % (len(order), len(n), len(d)))
moved = [c for c, _ in order if bn[c][:2] != bd[c][:2]]
say("cues whose first start or last end differ between 571 and 579: %d %s" % (len(moved), moved[:10]))
bad += ["%s moved" % c for c in moved]
more = {c: bd[c][2] - bn[c][2] for c, _ in order if bd[c][2] != bn[c][2]}
say("cues with more captions in 579: %d (net +%d) %s" % (len(more), sum(more.values()), more))

summ = json.loads((D / "completed-master993-summary.json").read_text())
start = {r["chapter"]: r["start_frame"] for r in summ["rows"]}
TL = {"01_the_bump": D / "final-review995/combined-timeline-terminal01/01_the_bump.timeline.tsv",
      "08_loop_sums": D / "final-review995/combined-timeline-terminal01/08_loop_sums.timeline.tsv",
      "09_boundary": D / "final-review995/combined-timeline-terminal01/09_boundary.timeline.tsv"}
T269 = {"01_the_bump": D / "preview274-review305/preview/01_the_bump.timeline.tsv",
        "09_boundary": D / "preview274-review305/preview/09_boundary.timeline.tsv",
        "11_counting": D / "preview274-review305/preview/11_counting.timeline.tsv"}
PINS = {"01_the_bump": "6149d6fa5e0e7086f1a61a727b0d9492bf32952815e378ffb82dd6cb973197ef",
        "09_boundary": "e05a6679f8105d7bec1647088c276d52871753770a3af4fbc99c3e3af5bffb76",
        "11_counting": "29a3afd345fa3a376e5c163d5bf5e01267a866e2305bdd27c6413a5bab46807d"}
say("== cue starts on the successor's schedule (combined01 timelines for 01/08/09; timeline269 for 11)")
for ch, p in list(TL.items()) + [("11_counting", T269["11_counting"])]:
    rows = timeline(p)
    off = 0
    for cue, frame in rows.items():
        want = round((start[ch] + frame) / FPS * 1000)
        got_n, got_d = bn[cue][0], bd[cue][0]
        if abs(got_n - want) > 1 or abs(got_d - want) > 1:
            off += 1
            bad.append("%s: timeline %d ms, captions %d / %d" % (cue, want, got_n, got_d))
    say("%-13s %s: %d cues, starts off by > 1 ms: %d" % (ch, "combined01" if ch in TL else "timeline269", len(rows), off))
say("== combined01 timelines against the timeline269 tables pinned by the 347 tool")
for ch in ("01_the_bump", "09_boundary"):
    pin = sha(T269[ch]) == PINS[ch]
    same = normal(TL[ch]) == normal(T269[ch])
    say("%-13s timeline269 copy = 347 pin: %s; all rows equal but narration folder: %s" % (ch, pin, same))
    if not (pin and same):
        bad.append("%s timeline differs" % ch)
say("08_loop_sums: no local copy of the 347-pinned timeline269 table (e2276be4); bound by its 5 cue starts above")
say("RESULT: %s" % ("no mismatch" if not bad else "; ".join(bad[:12])))
with open(OUT, "x", encoding="utf-8") as f:
    f.write("\n".join(lines) + "\n")
print("\n".join(lines))
