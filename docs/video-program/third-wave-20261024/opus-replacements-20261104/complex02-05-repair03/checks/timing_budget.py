"""Do the repaired beats fit the words? A desk estimate, not a measure of any take.

Each cue's fixed animation times are copied by hand from the staged source into OPS. A hook's time
is the character share of the whole cue, which is coarser than `Spoken::at` (that uses the take's
sentence starts and any phrase marks). Cue lengths come from preview02, the SCRATCH voice (the
frozen review's cues.txt), and are also scaled by 0.85 and 1.15 because Oliver's lengths for these
cues were not used here. So "late" below means: on these assumptions the scene reaches this hook
after the voice does. Nothing here is heard, and nothing here is Oliver.

Usage: python3 -I -B timing_budget.py SCRIPT_TSV CUES_TXT
"""

import re
import sys

script = {}
for row in open(sys.argv[1], encoding="utf8").read().splitlines():
    parts = row.split("\t")
    if len(parts) >= 2 and re.fullmatch(r"\d\d_\d+", parts[0]):
        script[parts[0]] = parts[1]
times = {}
for row in open(sys.argv[2], encoding="utf8"):
    m = re.match(r"(\d\d_\d+)\s+([\d.]+) to\s+([\d.]+)", row)
    if m:
        times[m.group(1)] = float(m.group(3)) - float(m.group(2))

F, S, P, U, E = "fixed", "sentence", "phrase", "until", "to_end"
OPS = {
    "02_5": [(F, 0.3), (P, "an eighth of a turn"), (F, 0.7 + 0.4 + 0.25 + 0.4), (S, 2), (F, 0.4)],
    "02_6": [(F, 0.3), (U, "So one plus i", 2.5), (F, 0.4), (S, 1), (F, 0.5), (S, 2), (F, 0.4 + 0.65),
             (P, "and angles add"), (F, 0.65), (P, "Every complex derivative"), (F, 0.5)],
    "03_2": [(F, 0.7), (S, 1), (F, 0.4), (P, "squares each length"), (F, 0.4), (P, "doubles each angle"),
             (F, 0.4), (S, 2), (F, 0.3 + 0.4 + 0.3), (E, 3.0)],
    "03_5": [(F, 0.8), (S, 1), (F, 0.3), (U, "The unit circle", 2.5), (F, 0.4 + 2.0), (P, "each point flips"),
             (F, 0.8), (S, 3), (F, 0.4), (S, 4), (F, 0.8)],
    "03_6": [(F, 1.8), (S, 1), (F, 1.8), (S, 2), (F, 0.3), (U, "So vertical lines", 2.0), (S, 3), (F, 1.2), (E, 1.2)],
    "04_3": [(F, 0.3 + 1.2 + 0.25 + 0.5), (S, 1), (F, 0.4), (S, 2), (F, 0.25), (E, 2.0)],
    "04_4": [(F, 0.3), (U, "Once for a simple zero", 2.0), (P, "twice for a zero of order two"), (F, 0.3),
             (S, 2), (F, 0.3)],
    "04_5": [(F, 0.5 + 1.2 + 0.25), (S, 1), (F, 0.3 + 0.25), (E, 2.0)],
    "04_6": [(F, 0.4 + 0.8 + 0.3 + 0.4 + 0.3), (S, 1), (F, 0.4)],
    "04_7": [(F, 0.3 + 1.4 + 0.3 + 0.4), (S, 1), (F, 0.6 + 0.2), (S, 2), (F, 0.4)],
    "05_2": [(F, 0.4 + 1.2 + 0.4), (S, 2), (F, 0.6), (P, "and at minus one"), (F, 0.3 + 0.4 + 0.4),
             (U, "backwards twice", 2.5)],
    "05_3": [(F, 0.4 + 0.5 + 0.8 + 0.4), (S, 1), (F, 0.4), (P, "and its size"), (F, 0.4), (P, "exact squares"),
             (F, 0.3), (P, "repeating every two pi"), (F, 0.5)],
    "05_7": [(F, 0.4 + 0.5 + 0.8 + 0.4), (S, 1), (F, 0.5), (S, 2), (F, 0.8), (S, 3), (F, 0.6), (S, 4), (F, 0.7),
             (S, 5), (F, 0.3 + 0.8 + 0.4 + 0.3 + 1.8)],
    "05_8": [(F, 0.5), (P, "is the Riemann hypothesis"), (F, 0.4), (S, 2), (F, 0.3 + 0.4)],
    "05_9": [(F, 0.4 + 0.4 + 0.4 + 1.5 + 0.3 + 0.3), (S, 1), (F, 0.5), (P, "in little tiles"),
             (F, 0.3 + 0.3 + 0.5 + 0.2), (P, "nearly square"), (F, 0.5), (S, 4), (F, 0.4)],
}


def sentence_starts(text):
    out = [0]
    for i, c in enumerate(text):
        if c in ".?!" and text[i + 1 : i + 2] == " ":
            out.append(i + 2)
    return out


def walk(cue, length):
    text = script[cue]
    starts = sentence_starts(text)
    share = lambda at: at / len(text) * length  # noqa: E731
    t, rows = 0.0, []
    for op in OPS[cue]:
        if op[0] == F:
            t += op[1]
        elif op[0] == E:
            t += max(length - t, op[1])
        else:
            if op[0] == S:
                label, hook = f"sentence {op[1]}", share(starts[op[1]])
            else:
                assert op[1] in text, op[1]
                label, hook = op[1], share(text.index(op[1]))
            if op[0] == U:
                t += max(hook - t, op[2])
                rows.append((label + " (lands)", t - hook))
            else:
                rows.append((label, max(t - hook, 0.0)))
                t = max(t, hook)
    rows.append(("voice ends", max(t - length, 0.0)))
    return rows


print("late = seconds the scene reaches a hook after the voice, on the assumptions in this file's header")
print(f"{'cue':5} {'len':>5}  {'hook':32} {'x0.85':>6} {'x1.00':>6} {'x1.15':>6}")
worst = {}
for cue in OPS:
    length = times[cue]
    a, b, c = walk(cue, length * 0.85), walk(cue, length), walk(cue, length * 1.15)
    for (label, x), (_, y), (_, z) in zip(a, b, c):
        print(f"{cue:5} {length:5.1f}  {label[:32]:32} {x:6.2f} {y:6.2f} {z:6.2f}")
        worst[(cue, label)] = max(x, y, z)
print()
print("hooks late by over 0.5 s at any scale:")
for (cue, label), x in worst.items():
    if x > 0.5:
        print(f"  {cue} {label}: {x:.2f}")
