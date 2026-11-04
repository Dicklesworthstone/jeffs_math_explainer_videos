#!/usr/bin/env python3
"""Apply the S2 picture-column corrections to a copy of the storyboard.

    python3 -I -B storyboard_corrections.py CANONICAL_STORYBOARD OUT_STORYBOARD

Each correction must match exactly once inside the picture cell of its row. The
words, id, features and hold cells of every row stay byte-identical, so script.tsv
cannot change. An existing output that differs is refused, never overwritten.

12_5, 12_8 and 12_9 are review item S2, updated to what staged03 merged. 04_2 is
beyond S2: the same kind of drift, from root's sixteenth subtotal repair.
"""
import hashlib
import re
import sys
from pathlib import Path

CORRECTIONS = [
    ("04_2", "sixteenth.rs format string",
     'Whole cells unchanged; "1 + 3 + 10 = 14";',
     'Whole cells unchanged; "whole parts: 1 + 3 + 10 = 14";'),
    ("12_5", "both_promises.rs pulse_all on the first tiles",
     'each town\'s first tile flashes, and its length appears',
     'each town\'s first tile pulses, with no growth, and its length appears'),
    ("12_8", "no tile is named until the rewind has cleared the later fills",
     'and no fill after "then" is ever shown beside the supposition.',
     'and no tile is named as missing while a fill after "then" is in view.'),
    ("12_8", "n3-rewind: words at the supposition, rewind at Go back, ring after it",
     'At "Suppose one were" the seven fills after 6 fade, leaving the real run at 6 (0, 2, 4), and Cedar\'s fifth '
     'tile (6 to 7 1/2), the wait 12_2 and 12_7 showed, is ringed red with "?"; "suppose: a passed tile not handed '
     'out". At "Go back" a second, ink marker slides 13 → 6, "then", with "solid: real up to then, hollow: only '
     'supposed".',
     'At "Suppose one were" only the words "suppose: a passed tile not handed out". At "Go back" a second, ink '
     'marker walks 13 → 6 while the seven fills after 6 leave one per house size, newest first, so every state is '
     'the real run at the marker and the walk ends on the real run at 6 (0, 2, 4); then "then", with "solid: real '
     'up to then, hollow: only supposed", and Cedar\'s fifth tile (6 to 7 1/2), the wait 12_2 and 12_7 showed, is '
     'ringed red with "?".'),
    ("12_8", "repair02: labelled chips",
     'seven hollow chips under the axis, one per seat; at "and the missing tile too" an eighth, red.',
     'seven hollow chips under the axis, one per seat, labelled "handed out since then"; at "and the missing tile '
     'too" the label becomes "since then, plus the missing tile" and an eighth chip appears, red.'),
    ("12_8", "grey-key: a separate key with the first veils",
     'with "6 to 7 1/2: not entered at 6", and the ink marker pulses.',
     'with "6 to 7 1/2: not entered at 6" and, arriving with the first grey, a separate key "grey: ruled out" (the '
     'words already up stay still), and the ink marker pulses.'),
    ("12_8", "repair02: red items brought to the front after the band",
     'only the whole tiles between the markers stay clear. Pulses recolor',
     'only the whole tiles between the markers stay clear, with the red ring, "?" and wait caption kept above the '
     'band. Pulses recolor'),
    ("12_9", "repair02: chips count first, rings land with the bounds",
     'At "So all those tiles fit" hero rings on the whole tiles inside the band; the missing one keeps its red ring, '
     'so the rings count 0, 1, 4 from here on. At "one more of them" beside the chips "7 seats from then to now: '
     '7 + 1 = 8 tiles" in red, and the red chip pulses. At "But each town\'s row fits at most" under them "Ash 0',
     'At "So all those tiles fit" the chips pulse: they are the count. At "one more of them" the label beside the '
     'chips becomes "7 seats from then to now: 7 + 1 = 8 tiles" in red, and the red chip pulses. At "But each '
     'town\'s row fits at most" hero rings on the whole tiles inside the band, the missing one keeping its red ring '
     'on top, so the rings count 0, 1, 4; under the chips "Ash 0'),
]


def cells(line):
    return re.split(r"(?<!\\)\|", line)


def correct(text):
    lines = text.split("\n")
    for cue, _, old, new in CORRECTIONS:
        hits = [i for i, line in enumerate(lines) if line.startswith(f"| {cue} |")]
        if len(hits) != 1:
            raise SystemExit(f"{cue}: not exactly one row")
        row = cells(lines[hits[0]])
        if len(row) != 7:
            raise SystemExit(f"{cue}: {len(row)} cells")
        if row[3].count(old) != 1:
            raise SystemExit(f"{cue}: correction does not match its picture cell once: {old[:60]!r}")
        if "|" in new:
            raise SystemExit(f"{cue}: a bare pipe in new text")
        row[3] = row[3].replace(old, new)
        lines[hits[0]] = "|".join(row)
    return "\n".join(lines)


def main(argv):
    source, out = Path(argv[0]), Path(argv[1])
    before = source.read_text()
    after = correct(before)
    a, b = before.split("\n"), after.split("\n")
    if len(a) != len(b):
        raise SystemExit("line count changed")
    changed = []
    for x, y in zip(a, b):
        if x == y:
            continue
        cx, cy = cells(x), cells(y)
        if [c for i, c in enumerate(cx) if i != 3] != [c for i, c in enumerate(cy) if i != 3]:
            raise SystemExit("a cell other than the picture changed")
        changed.append(cx[1].strip())
    if out.exists() and out.read_text() != after:
        print(f"DIFFERS, not overwritten: {out}")
        return 1
    out.parent.mkdir(parents=True, exist_ok=True)
    if not out.exists():
        out.write_text(after)
    print(f"{hashlib.sha256(before.encode()).hexdigest()}  input storyboard")
    print(f"{hashlib.sha256(after.encode()).hexdigest()}  corrected storyboard")
    print(f"{len(CORRECTIONS)} corrections in rows {changed}; every other cell and line unchanged")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
