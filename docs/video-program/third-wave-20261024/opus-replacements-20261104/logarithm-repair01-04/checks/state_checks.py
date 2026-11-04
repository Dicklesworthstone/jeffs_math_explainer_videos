#!/usr/bin/env python3
"""Does the picture say what is true at each moment? Asked of both trees.

Reads the snapshot rows that checks/run_against_stub.py leaves in SCRATCH_DIR. Each
claim names a review finding and a moment (cue, sentence), and asks about what the
scene has on screen then: which texts, which strokes, where. The positions are real
scene geometry from the real chapter code. The text boxes and the clock are estimates.

A claim must hold on the candidate at every speaking rate. It must fail on the base at
the middle rate, or the check cannot tell a repair from no repair. This is not picture
acceptance: it reads no frame.

Usage: python3 -I -B checks/state_checks.py SCRATCH_DIR
"""

import math
import sys
from collections import defaultdict
from pathlib import Path

RATES = ("2.68", "2.3", "3.1")
ONE_X, STEP, OX, OY = -3.6, 2.0, -3.2, -0.4
D_Y, C_Y = 0.3, 0.62
END = "before-next"


def load(path):
    moments = defaultdict(lambda: {"texts": [], "strokes": []})
    for row in path.read_text().splitlines():
        parts = row.split("\t")
        _prefix, cue, sentence, _t, kind = parts[:5]
        key = (cue, sentence)
        if kind == "text":
            x, y, w, h = (float(n) for n in parts[5:9])
            moments[key]["texts"].append((parts[9], x, y, w, h))
        else:
            paint, half = parts[5], float(parts[6])
            pts = [tuple(float(n) for n in p.split(",")) for p in parts[7].split(";")]
            moments[key]["strokes"].append((paint, half, pts))
    return moments


def segment_distance(pts, box):
    _, cx, cy, w, h = box
    best = float("inf")
    for (x0, y0), (x1, y1) in zip(pts, pts[1:]):
        steps = max(1, int(math.hypot(x1 - x0, y1 - y0) / 0.01))
        for k in range(steps + 1):
            t = k / steps
            x, y = x0 + (x1 - x0) * t, y0 + (y1 - y0) * t
            best = min(best, math.hypot(max(abs(x - cx) - w / 2, 0.0), max(abs(y - cy) - h / 2, 0.0)))
    return best


class Now:
    """What is on screen at one moment."""

    def __init__(self, moment):
        self.texts, self.strokes = moment["texts"], moment["strokes"]

    def has(self, part):
        return any(part in t[0] for t in self.texts)

    def text(self, exact):
        return next((t for t in self.texts if t[0] == exact), None)

    def like(self, part):
        return next((t for t in self.texts if part in t[0]), None)

    def label(self, exact, y):
        return next(t for t in self.texts if t[0] == exact and abs(t[2] - y) < 0.01)

    def paint(self, paint):
        return [s for s in self.strokes if s[0] == paint]

    def level(self, paint, y, within=0.01):
        """Straight level strokes at height y, as (left, right) centre lines."""
        out = []
        for _, _, pts in self.paint(paint):
            if len(pts) == 2 and abs(pts[0][1] - y) < within and abs(pts[1][1] - y) < within:
                out.append(tuple(sorted((pts[0][0], pts[1][0]))))
        return sorted(out)

    def clear_of(self, box, paint, minimum=0.03):
        """Whether every `paint` stroke keeps `minimum` from the text box."""
        return all(segment_distance(pts, box) - half >= minimum for _, half, pts in self.paint(paint))


def box_gap(a, b):
    return max(abs(a[1] - b[1]) - (a[3] + b[3]) / 2, abs(a[2] - b[2]) - (a[4] + b[4]) / 2)


def gaps(pieces, half):
    return [b[0] - a[1] - 2 * half for a, b in zip(pieces, pieces[1:])]


def ruler_x(n):
    return -5.6 + 2.6 * math.log(n)


def red_bar_on_negative_y(n):
    for _, _, pts in n.paint("red"):
        on_axis = all(abs(p[0] - OX) < 0.02 for p in pts)
        if on_axis and min(p[1] for p in pts) < OY - 0.1 and max(p[1] for p in pts) - min(p[1] for p in pts) > 1.0:
            return True
    return False


def hunter_tip(n):
    return [max(s[2], key=lambda p: p[1])[0] for s in n.paint("hero") if len(s[2]) == 4]


def product_drawn(n):
    blue = n.level("blue", 0.1)
    teal = n.level("teal", -0.08)
    good_blue = any(abs(b[0] - ruler_x(1)) < 0.05 and abs(b[1] - ruler_x(2)) < 0.05 for b in blue)
    good_teal = any(abs(b[0] - ruler_x(2)) < 0.05 and abs(b[1] - ruler_x(8)) < 0.05 for b in teal)
    return good_blue and good_teal and n.has("2 × 4 = 8")


def one_bar_from_one_to_ten(n):
    bars = n.level("blue", 0.0)
    one, ten = n.text("1"), n.text("10")
    return len(bars) == 1 and abs(bars[0][0] - one[1]) <= 0.08 and abs(bars[0][1] - ten[1]) <= 0.08


def closing_rows(n):
    a, b = n.like("2^{10}"), n.like("doublings")
    return a is not b and a[1] - a[3] / 2 > 1.9 and b[1] - b[3] / 2 > 1.9


# finding, cue, sentence, what must be true there, the test
CLAIMS = [
    ("1.12", "01_1", END, "the title does not ask the question before the voice does",
     lambda n: not n.has("Where does three go?")),
    # Both trees draw the first length twice, one exactly on the other, so the check counts places.
    ("1.4", "01_1", "4", "the three doublings show as three lengths with visible gaps",
     lambda n: len(set(n.level("blue", 0.0))) == 3 and min(gaps(sorted(set(n.level("blue", 0.0))), 0.03)) >= 0.05),
    ("1.5", "01_2", END, "the half steps show as separate lengths",
     lambda n: len(n.level("blue", -0.28)) == 3 and min(gaps(n.level("blue", -0.28), 0.03)) >= 0.05),
    ("1.5", "01_2", END, '"one step" clears every blue stroke',
     lambda n: n.clear_of(n.text("one step"), "blue")),
    ("1.8", "01_3", END, "the hunter rests at least 0.15 away from the true place of 3",
     lambda n: len(hunter_tip(n)) == 1 and abs(hunter_tip(n)[0] - (ONE_X + STEP * math.log2(3))) >= 0.15),
    ("1.6", "01_4", END, "both compressed rows keep visible gaps between their pieces",
     lambda n: all(min(gaps(n.level(c, y), 0.025)) >= 0.04 for c, y in (("blue", -2.35), ("hero", -2.7)))),
    ("1.7", "01_4", END, '"19 doublings" ends inside the safe frame',
     lambda n: n.text("19 doublings")[1] + n.text("19 doublings")[3] / 2 <= 6.86),
    ("1.9", "01_5", END, "the keyboard line replaces the power row",
     lambda n: n.has("Hz") and not n.like("% apart")),
    ("1.3", "01_6", "4", "no base-two claim survives into the base-ten sentence",
     lambda n: not any(n.has(s) for s in (r"\mathrm{pos}(x)", r"\mathrm{pos}(3)", r"\log_2 8", r"\log_2 3"))),
    ("1.10", "01_6", "4", "the third sentence has a concrete picture: a step of ×4",
     lambda n: n.has("one step = ×4")),
    ("1.1", "01_6", END, "one blue bar under the ruler, from the 1 mark to the 10 mark", one_bar_from_one_to_ten),
    ("1.2", "01_6", END, 'the labels "8" and "10" stay clearly apart',
     lambda n: box_gap(n.text("8"), n.text("10")) >= 0.2),
    ("1.13", "01_6", END, "the marks did not move: 8 is still three old steps along",
     lambda n: abs(n.text("8")[1] - (ONE_X + 3 * STEP)) < 1e-6),
    ("2.4", "02_1", "2", "the full-length caption states its unit",
     lambda n: n.has("one full length = 1")),
    ("2.1", "02_2", END, "the gold line leaves D's 8 whole",
     lambda n: n.clear_of(n.label("8", D_Y - 0.58), "hero")),
    ("2.1", "02_4", END, "the red line leaves C's 6 whole",
     lambda n: n.clear_of(n.label("6", C_Y + 0.58), "red")),
    ("2.2", "02_4", END, "the red end tick leaves D's 10 whole",
     lambda n: n.clear_of(n.label("10", D_Y - 0.58), "red")),
    ("2.2", "02_5", "2", "the overflow's red marks and words have left",
     lambda n: not n.paint("red") and not n.has("D stops at 10")),
    ("2.1", "02_5", END, "the gold line leaves C's 6 whole",
     lambda n: n.clear_of(n.label("6", C_Y + 0.58), "hero")),
    ("2.7", "02_5", END, "C's name is on screen",
     lambda n: any(t[0] == "C" and abs(t[1]) < 6.86 for t in n.texts)),
    ("2.9", "02_5", END, '"one mark" has a bracket over the three rows',
     lambda n: any(len(s[2]) == 4 and abs(s[2][0][1] - s[2][3][1]) > 1.0 for s in n.paint("dim"))),
    ("3.1", "03_2", "1", '"one lap = ×10" has gone with its arc',
     lambda n: not n.has("one lap = ×10")),
    ("3.3", "03_2", "3", "no gold tick shows the landing before the turn",
     lambda n: not n.paint("hero")),
    ("3.3", "03_2", "4", "the dial says 2.4 on landing",
     lambda n: n.text("2.4") is not None),
    ("3.1", "03_2", END, '"one lap = ×10" serves the sentence about the extra lap, by the counter',
     lambda n: n.text("one lap = ×10")[1] - n.text("one lap = ×10")[3] / 2 > 1.3),
    ("3.4", "03_3", "1", "the title has moved on to the laps",
     lambda n: n.has("Laps the dial forgets") and not n.has("Multiplying is turning")),
    ("3.5", "03_3", "1", '"the dial shows only the mark" has lasted through the first sentence',
     lambda n: n.has("the dial shows only the mark")),
    ("3.3", "03_3", "2", "the floor caption names the 2.4 mark",
     lambda n: n.has("the 2.4 mark: one place on the dial")),
    ("3.6", "03_3", "2", "the floor caption clears the tilted tick",
     lambda n: n.clear_of(n.like("one "), "hero")),
    ("3.10", "03_3", "2", "one dashed drop, not four overlaid",
     lambda n: len(n.paint("faint")) == 16),
    ("3.9", "03_4", "0", "the readings clear the helix",
     lambda n: all(n.clear_of(n.text(t), "purple", 0.05) for t in ("0.24", "2.4", "24", "240"))),
    ("3.7", "03_4", "2", "the right panel has its caption while the voice describes it",
     lambda n: n.has("inputs a lap apart, one output")),
    ("3.2", "03_4", END, "the spiral's caption clears the helix",
     lambda n: n.clear_of(n.text("a spiral, and its shadow"), "purple", 0.05)),
    ("4.5", "04_2", "4", "the tangent has run up the curve by the end of the climbing sentence",
     lambda n: any(len(s[2]) == 2 and abs(s[2][0][1] - s[2][1][1]) > 0.5 for s in n.paint("blue"))),
    ("4.10", "04_3", "1", "the stale law has left",
     lambda n: not n.has(r"e^{a+b}")),
    ("4.11", "04_3", "1", "the formula has not arrived before its name is spoken",
     lambda n: not n.has(r"\ln(e^{x})")),
    ("4.9", "04_3", "3", "a ring marks the point (0, 1) for ln 1 = 0",
     lambda n: any(len(s[2]) == 33 and abs(sum(p[1] for p in s[2][:32]) / 32 - (OY + 1)) < 0.01 and s[1] > 0.01
                   for s in n.paint("hero"))),
    ("4.2", "04_3", END, "ln 2's label clears the tick labels 1 and 2",
     lambda n: all(box_gap(n.like(r"\ln 2 \approx"), t) >= 0.05 for t in n.texts if t[0] in ("1", "2"))),
    ("4.11", "04_3", END, "the words natural logarithm meet the symbol ln",
     lambda n: n.has("ln: the natural logarithm")),
    ("4.1", "04_4", "2", "no red bar is left on ln's outputs, the negative y-axis",
     lambda n: not red_bar_on_negative_y(n)),
    ("4.1", "04_4", "2", "the bar lies on the negative x-axis: the inputs with no logarithm",
     lambda n: any(b[0] < OX - 2.0 and abs(b[1] - OX) < 0.02 for b in n.level("red", OY, 0.02))),
    ("4.1", "04_4", "2", "the dashed floor has become ln's wall, on the y-axis",
     lambda n: len([s for s in n.paint("red") if all(abs(p[0] - OX) < 0.02 for p in s[2])]) >= 20),
    ("4.1", "04_4", "2", "the red words say what the red marks now mean",
     lambda n: n.has("no real logarithm") and not n.has("0 and below")),
    ("4.6", "04_4", "2", "the triangle's legs are labelled after the flip",
     lambda n: n.text("rise") is not None and n.text("run") is not None),
    # 4.8 is not here. Both trees have lost the name by this moment: the base during the 2.6 s
    # flip, the candidate as it starts. Only the order can tell them apart, so a chapter test
    # pins the order and checks/stub_sensitivity.py proves that test bites.
    ("4.16", "04_4", "4", "the triangle has gone before the tangent crosses it",
     lambda n: not any(len(s[2]) == 4 for s in n.paint("blue"))),
    ("4.17", "04_4", END, "ln 1.01 uses approximately-equals throughout",
     lambda n: n.has(r"\ln 1.01 \approx 0.00995")),
    ("4.12", "04_5", "1", "the ruler's spine is up by the end of the first sentence",
     lambda n: any(len(s[2]) == 2 and abs(s[2][0][1] - 0.4) < 0.01 and abs(s[2][1][1] - 0.4) < 0.01
                   for s in n.paint("green"))),
    ("4.3", "04_5", "3", "the product law is drawn: blue 1 to 2, teal 2 to 8, labelled 2 × 4 = 8", product_drawn),
    ("4.18", "04_5", END, "the base-two word uses a true subscript",
     lambda n: n.has(r"\log_2:") and not n.has("log₂:")),
    ("4.14", "04_7", END, "the card names the rule of seventy and the dot says ×1000",
     lambda n: n.has("rule of seventy") and n.text("×1000") is not None),
    ("4.18", "04_7", END, "the drop at the doubling time has a number at its foot",
     lambda n: n.text("13.9") is not None),
    ("4.4", "04_7", END, "the closing rows start well right of the dashed ×2 line, in two rows", closing_rows),
]


def evaluate(moments, entry):
    cue, sentence, function = entry[1], entry[2], entry[4]
    key = (cue, sentence)
    if key not in moments:
        return False, "no such moment"
    try:
        return bool(function(Now(moments[key]))), ""
    except (AttributeError, TypeError, StopIteration, ValueError, IndexError) as error:
        return False, type(error).__name__


def main():
    out = Path(sys.argv[1])
    base = load(out / "base-2.68.tsv")
    candidates = {rate: load(out / f"candidate-{rate}.tsv") for rate in RATES}
    trouble = 0
    print("finding\tcue\tsentence\tbase\tcandidate at " + ", ".join(RATES) + "\tclaim")
    for entry in CLAIMS:
        finding, cue, sentence, words = entry[:4]
        was, why = evaluate(base, entry)
        now = [evaluate(candidates[rate], entry)[0] for rate in RATES]
        shown = "/".join("pass" if ok else "FAIL" for ok in now)
        note = f" ({why})" if why else ""
        print(f"{finding}\t{cue}\t{sentence}\t{'pass' if was else 'fail'}{note}\t{shown}\t{words}")
        if not all(now):
            trouble += 1
        if was:
            print("  ^ NOT SENSITIVE: the base passes this claim too")
            trouble += 1
    print(f"{len(CLAIMS)} claims, {trouble} problems")
    sys.exit(1 if trouble else 0)


main()
