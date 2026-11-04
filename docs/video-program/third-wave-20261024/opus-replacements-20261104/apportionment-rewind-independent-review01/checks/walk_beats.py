"""Walk 12_8's staged beats in source order with lag carried, at several voice rates, 30 and 60 fps.

    python3 -I -B walk_beats.py      (from the repo root)

Frames as the pinned engine counts them: ceil(run_time x fps) on the f64's exact rational value. Phrases by
Spoken::at with no marks: sentence start by character share, phrase by character share inside its sentence.
wait_until waits with kit's frames_covering (ceil of seconds x fps - 1e-6). Not Oliver; a real take's sentence
onsets will move these.
"""
import math
import pathlib
from fractions import Fraction as Fr

STAGE = pathlib.Path("narration_lab/third-wave-20261024/root/elevenlabs-apportionment-rebuild-20261104/combined-repair-stage03/explainer_corpus/apportionment")
script = dict(l.split("\t", 1) for l in (STAGE / "script.tsv").read_text().splitlines() if l and not l.startswith("#"))
s = script["12_8"]
offs = [0] + [i + 2 for i in range(len(s) - 1) if s[i] in ".?!" and s[i + 1] == " " and i + 2 < len(s)]


def at(phrase, length):
    i = s.index(phrase)
    j = max(n for n, o in enumerate(offs) if o <= i)
    frm, to = offs[j], offs[j + 1] if j + 1 < len(offs) else len(s)
    t0, t1 = length * Fr(frm, len(s)), length * Fr(to, len(s))
    return t0 + (t1 - t0) * Fr(i - frm, max(to - frm, 1))


# ("w", phrase) waits; ("p", label, seconds...) plays one call per seconds value, in source order, lines 860-1019
BEATS = [
    ("p", "ranks and waits out", 0.3),
    ("w", "Could a passed tile ever be left out"),
    ("p", "past-now fills out", 0.4),
    ("p", "marker to now", 1.0),
    ("p", "now label", 0.3),
    ("w", "Suppose one were"),
    ("p", "suppose caption", 0.4),
    ("w", "Go back to the last moment"),
    ("p", "ink marker in", 0.3),
    ("p", "rewind", *([0.12] * 14)),
    ("p", "then and legend", 0.3),
    ("p", "ring and ?", 0.3),
    ("w", "a tile that ends later than the missing one"),
    ("p", "later ring, compare, label", 0.4),
    ("w", "or to the very start if it never did"),
    ("p", "or 0", 0.3),
    ("w", "Every tile handed out since then"),
    ("p", "need label", 0.3),
    ("p", "seven chips", 0.9),
    ("w", "and the missing tile too"),
    ("p", "label morph", 0.3),
    ("p", "red chip", 0.3),
    ("w", "had not been entered at that moment"),
    ("p", "compare out", 0.2),
    ("p", "veils, wait label, grey key", 0.4),
    ("p", "then pulse", 0.5),
    ("w", "or the rule would have picked a tile ending sooner"),
    ("p", "rings pulse", 0.6),
    ("p", "later ring and label out", 0.2),
    ("p", "later veil", 0.2),
    ("w", "And every one of them ends by now"),
    ("p", "after veils and band", 0.4),
    ("p", "now pulse", 0.5),
]
WORDS = len(s.split())


def walk(length, fps):
    frame, lags, marks = 0, {}, {}
    for b in BEATS:
        if b[0] == "w":
            t = at(b[1], length)
            need = math.ceil(max(t - Fr(frame, fps), 0) * fps - Fr(1, 10 ** 6)) if t > Fr(frame, fps) else 0
            lags[b[1]] = float(Fr(frame, fps) - t)
            frame += need
            marks[b[1]] = Fr(frame, fps)
        else:
            frame += sum(math.ceil(Fr(r) * fps) for r in b[2:])
            marks[b[1]] = Fr(frame, fps)
    return lags, marks


for wps in ("2.0", "2.6", "3.0", "3.25", "3.5", "3.75", "4.0"):
    length = Fr(WORDS) / Fr(wps) + Fr(2, 5)
    for fps in (30, 60):
        lags, marks = walk(length, fps)
        late = {k[:24]: round(v, 2) for k, v in lags.items() if v > 0}
        ring, miss = marks["ring and ?"], at("the missing one", length)
        end = marks["now pulse"]
        print(f"{wps:>4s} w/s {fps} fps: rewind ends {float(marks['rewind']):6.2f}, ring in {float(ring):6.2f}, says 'the missing one' {float(miss):6.2f}, last beat ends {float(end):6.2f} of {float(length):6.2f}; late: {late or 'none'}")
for wps in [Fr(x, 100) for x in range(300, 900, 5)]:
    length = Fr(WORDS) / wps + Fr(2, 5)
    _, marks = walk(length, 30)
    if marks["ring and ?"] >= at("the missing one", length):
        print(f"the ring would first miss the words 'the missing one' at about {float(wps):.2f} words a second")
        break
