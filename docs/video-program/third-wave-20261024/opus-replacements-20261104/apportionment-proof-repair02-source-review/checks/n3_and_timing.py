#!/usr/bin/env python3
"""Reviewer's own check for the repair02 source review. Run from the repo root
with python3 -I -B.

Part 1: exact fractions from the three populations, no crate import. Does every
state of the proposed rewind equal a real run, with no hole in any row?

Part 2: `Spoken::at` (draw.rs) re-implemented for scratch voice, against picture
durations copied by hand from the source. How much of each word window does the
picture use? Not native evidence: no build, no render, no measured Oliver take.
"""
import csv
import math
from fractions import Fraction as Fr

POP = {"Ash": 4000, "Birch": 9000, "Cedar": 26000}
FRAC = {t: Fr(p, sum(POP.values())) for t, p in POP.items()}
THEN, NOW = 6, 13
MISSING = ("Cedar", 5)

seats = {t: 0 for t in POP}
history = {0: dict(seats)}
handed = {}
for h in range(1, 17):
    for t in sorted(POP, key=lambda t: -Fr(POP[t], seats[t] + 1)):
        if seats[t] + 1 <= math.ceil(h * FRAC[t]):
            seats[t] += 1
            handed[h] = (t, seats[t])
            history[h] = dict(seats)
            break

print("Part 1: the proposed rewind, newest hand-out first")
state = {t: set(range(1, history[NOW][t] + 1)) for t in POP}
for h in range(NOW, THEN, -1):
    t, k = handed[h]
    state[t].remove(k)
    assert all(state[x] == set(range(1, history[h - 1][x] + 1)) for x in POP), h
    assert all(state[x] == set(range(1, len(state[x]) + 1)) for x in POP), ("hole", h)
    print(f"  undo h{h:2d} {t} {k:2d} -> marker {h - 1:2d}: {[len(state[x]) for x in POP]} = real run at {h - 1}")
assert MISSING[1] not in state[MISSING[0]]
print("every state is the real run at the marker, no row has a hole: yes")
print(f"at landing {[len(state[x]) for x in POP]}; {MISSING[0]} {MISSING[1]} is unfilled when its ring lands: yes")
print(f"{MISSING[0]} {MISSING[1]} is filled until h7 is undone, the last step: its fill leaves just before the marker reaches {THEN}")


def frames(rt, fps):
    return math.ceil(rt * fps - 1e-6)


def anim(fps, *rts):
    return sum(frames(r, fps) for r in rts) / fps


text = {}
for row in csv.reader(open("explainer_corpus/apportionment/script.tsv"), delimiter="\t"):
    if row and row[0].startswith("12_"):
        text[row[0]] = row[-1]
cues = {}
render = "narration_lab/third-wave-20261024/root/five-film-release-20261027/apportionment-opus-rebuild-preview02/render/"
for row in csv.reader(open(render + "12_both_promises.cues.tsv"), delimiter="\t"):
    cues[row[0]] = (float(row[1]), float(row[2]))


def at(cue, phrase):
    """Seconds from the cue start: char share of the sentence, sentence by char share of the line."""
    s = text[cue]
    i = s.index(phrase)
    starts = [0] + [j + 2 for j in range(len(s) - 2) if s[j] in ".?!" and s[j + 1] == " "]
    k = max(n for n, st in enumerate(starts) if st <= i)
    a = starts[k]
    b = starts[k + 1] if k + 1 < len(starts) else len(s)
    length = cues[cue][1]
    return length * a / len(s) + length * (b - a) / len(s) * (i - a) / max(b - a, 1)


# (cue, phrase, next phrase or None for cue end, baseline rts, candidate rts, rec rts or None)
REC_STEPS = (0.12,) * 14
W = [
    ("12_8", "Suppose one were", "Go back to the last moment", (0.4, 0.4), (0.4, 0.4), (0.4,)),
    ("12_8", "Go back to the last moment", "a tile that ends later than the missing one", (0.3, 1.6, 0.3), (0.3, 1.6, 0.3), (0.3,) + REC_STEPS + (0.3, 0.3)),
    ("12_8", "Every tile handed out since then", "and the missing tile too", (0.9,), (0.3, 0.9), None),
    ("12_8", "and the missing tile too", "had not been entered at that moment", (0.3,), (0.3, 0.3), None),
    ("12_8", "had not been entered at that moment", "or the rule would have picked", (0.2, 0.4, 0.5), (0.2, 0.4, 0.4, 0.5), None),
    ("12_9", "So all those tiles fit inside the stretch", "one more of them than seats added", (1.0,), (0.6,), None),
    ("12_9", "one more of them than seats added", "But each town's row fits at most", (0.4, 0.6), (0.4, 0.6), None),
    ("12_9", "But each town's row fits at most", "and the fractions add up to one", (1.2,), (1.0, 1.2), None),
]
print()
print("Part 2: word windows on scratch lengths. tol = how much faster than scratch the voice could run here before the picture passes the next phrase")
print(f"{'phrase':44s} {'starts':>8s} {'window':>7s} {'base':>5s} {'cand':>5s} {'tol':>5s} {'N3':>5s} {'tol':>5s}")
for cue, phrase, nxt, base, cand, rec in W:
    t0, t1 = at(cue, phrase), at(cue, nxt)
    win = t1 - t0
    b = anim(30, *base)
    c60, c30 = anim(60, *cand), anim(30, *cand)
    c = max(c30, c60)
    line = f"{phrase[:43]:44s} {cues[cue][0] + t0:8.2f} {win:7.2f} {b:5.2f} {c:5.2f} {win / c:5.2f}"
    if rec:
        r = max(anim(30, *rec), anim(60, *rec))
        line += f" {r:5.2f} {win / r:5.2f}"
    print(line)
print("fps note: 14 steps of 0.12 s cost", round(anim(30, *REC_STEPS), 3), "s at 30 fps and", round(anim(60, *REC_STEPS), 3), "s at 60 fps")
