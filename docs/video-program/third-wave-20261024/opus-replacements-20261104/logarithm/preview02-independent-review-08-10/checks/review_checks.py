#!/usr/bin/env python3
"""Independent checks behind the logarithm preview02 08-10 picture review.

Run from the repository root:

    python3 -I -B docs/video-program/third-wave-20261024/opus-replacements-20261104/logarithm/preview02-independent-review-08-10/checks/review_checks.py

Reads the source and the extracted frames. Writes nothing. Python mirrors of the crate's
arithmetic, not the crate itself: nothing here was compiled or rendered. Text widths use the
CMR10 table in EMS with width = ems * size * 0.01, an estimate good to a few hundredths.
"""
import math
import sys
from pathlib import Path

try:
    from PIL import Image, ImageChops
except ImportError:
    sys.exit("needs Pillow for the frame sections")

FRAMES = Path(
    "narration_lab/third-wave-20261024/root/five-film-release-20261027/logarithm-opus-rebuild-frames02"
)
CRATE = Path("explainer_corpus/logarithm_mysteries")
TAU = math.tau
I2 = [[1.0, 0.0], [0.0, 1.0]]
J = [[0.0, -1.0], [1.0, 0.0]]


def mmul(a, b):
    return [
        [a[0][0] * b[0][0] + a[0][1] * b[1][0], a[0][0] * b[0][1] + a[0][1] * b[1][1]],
        [a[1][0] * b[0][0] + a[1][1] * b[1][0], a[1][0] * b[0][1] + a[1][1] * b[1][1]],
    ]


def mscale(a, k):
    return [[a[0][0] * k, a[0][1] * k], [a[1][0] * k, a[1][1] * k]]


def madd(a, b):
    return [[a[0][0] + b[0][0], a[0][1] + b[0][1]], [a[1][0] + b[1][0], a[1][1] + b[1][1]]]


def inverse(a):
    d = a[0][0] * a[1][1] - a[0][1] * a[1][0]
    return [[a[1][1] / d, -a[0][1] / d], [-a[1][0] / d, a[0][0] / d]]


def rotation(t):
    return [[math.cos(t), -math.sin(t)], [math.sin(t), math.cos(t)]]


def mexp(x):
    """model::mexp: scale, 30 Taylor terms, square."""
    norm = max(abs(v) for r in x for v in r)
    s = 0
    while norm / 2.0**s > 0.25:
        s += 1
    small = mscale(x, 1.0 / 2.0**s)
    total, term = I2, I2
    for k in range(1, 31):
        term = mscale(mmul(term, small), 1.0 / k)
        total = madd(total, term)
    for _ in range(s):
        total = mmul(total, total)
    return total


def generator(a, b, tilt_degrees):
    """model::ellipse_generator: 2 pi P J P^-1, P = R(tilt) diag(a, b)."""
    p = mmul(rotation(math.radians(tilt_degrees)), [[a, 0.0], [0.0, b]])
    return mscale(mmul(mmul(p, J), inverse(p)), TAU)


def apply(m, p):
    return [m[0][0] * p[0] + m[0][1] * p[1], m[1][0] * p[0] + m[1][1] * p[1]]


def border(name, m, clip=3.0):
    """draw::clipped_grid at n = 4 for the lines k = +-3, as doing_nothing.rs:126 calls it."""
    n, samples = 4, 48
    print(f"  {name}, clip {clip}")
    for k in (-3, 3):
        for vertical in (False, True):
            ts = [-n + n * j / samples for j in range(2 * samples + 1)]
            inside = [t for t in ts if abs(t) <= 3.0 + 1e-12]
            kept = []
            for t in inside:
                q = apply(m, [float(k), t] if vertical else [t, float(k)])
                if abs(q[0]) <= clip and abs(q[1]) <= clip:
                    kept.append(t)
            axis = "x" if vertical else "y"
            where = f"t {min(kept):+.2f} .. {max(kept):+.2f}" if kept else "nothing"
            print(f"    line {axis} = {k:+d}: {len(kept)}/{len(inside)} samples kept, {where}")


print("A. The grid border at t = 1 (finding F5)")
border("gens[0], 2 pi J: frames 25-35", mexp(mscale(J, TAU)))
border("gens[1], ellipse 2 by 1: frames 48-58", mexp(generator(2.0, 1.0, 0.0)))
border("gens[2], ellipse 1.6 by 0.7 tilted 30: frames 66-87", mexp(generator(1.6, 0.7, 30.0)))
print("  the same three with a tolerance of 1e-9 on the clip")
border("gens[0]", mexp(mscale(J, TAU)), 3.0 + 1e-9)
border("gens[1]", mexp(generator(2.0, 1.0, 0.0)), 3.0 + 1e-9)
border("gens[2]", mexp(generator(1.6, 0.7, 30.0)), 3.0 + 1e-9)
stretch = mexp([[math.log(1.5), 0.0], [0.0, math.log(0.6)]])
print(f"  stretch flow at t = 1: the line x = 2 lands at {2 * stretch[0][0]!r}; kept: {2 * stretch[0][0] <= 3.0}")

print("B. The matrix card widths as drawn (finding F1): draw.rs dx = size*0.011, w = dx + size*0.0095")
EMS = {".": 0.278, "−": 0.778, " ": 0.333, "r": 0.392, "a": 0.5, "d": 0.556, "i": 0.278, "n": 0.556, "s": 0.394}


def ems(text):
    return sum(EMS.get(ch, 0.5) for ch in text)


def width(entry, size):
    return ems(entry) * size * 0.01


def clearances(rows, size, dx, w):
    """Gap between the two entries of each row; each bracket against its column's widest entry."""
    gaps = [2 * dx - (width(left, size) + width(right, size)) / 2 for left, right in rows]
    left = w - (dx + max(width(row[0], size) for row in rows) / 2)
    right = w - (dx + max(width(row[1], size) for row in rows) / 2)
    return gaps, left, right


def report(label, rows, size, dx, w, at=None):
    gaps, left, right = clearances(rows, size, dx, w)
    collides = min(gaps) < 0.05 or left < 0.0 or right < 0.0
    span = "" if at is None else f", brackets at {at - w:.2f} .. {at + w:.2f}"
    print(
        f"  {label}: row gaps {gaps[0]:+.3f} {gaps[1]:+.3f}, left bracket {left:+.3f}, "
        f"right bracket {right:+.3f}{span}" + ("  COLLIDES" if collides else "")
    )


CARDS = (
    ("09_1 rotation card", 4.4, 34.0, [["0.5", "−0.866"], ["0.866", "0.5"]]),
    ("e_card", 4.6, 30.0, [["0.905", "−0.97"], ["0.776", "0.517"]]),
    ("x_card", 4.6, 30.0, [["0.3", "−1"], ["0.8", "−0.1"]]),
    ("target card", 5.4, 30.0, [["0", "−1"], ["1", "0"]]),
    ("pair", 4.9, 26.0, [["a", "−b"], ["b", "a"]]),
    ("identity card", 4.6, 32.0, [["1", "0"], ["0", "1"]]),
)
for label, at, size, rows in CARDS:
    report(label, rows, size, size * 0.011, size * 0.011 + size * 0.0095, at)

print("C. e^X for the 09_2 velocity, values on the card (finding F1: the numbers are right)")
X = [[0.3, -1.0], [0.8, -0.1]]
E = mexp(X)
print(f"  e^X = [[{E[0][0]:.6f}, {E[0][1]:.6f}], [{E[1][0]:.6f}, {E[1][1]:.6f}]]")
disc = (X[0][0] + X[1][1]) ** 2 - 4 * (X[0][0] * X[1][1] - X[0][1] * X[1][0])
im = math.sqrt(-disc) / 2
print(f"  Im of its eigenvalue: {im:.5f} radians per unit time = {im / math.pi:.4f} pi = {im / TAU:.4f} laps (finding F8)")

print("D. Rates in 09_4 agree with pi/2 + 2 pi k")
for rate, k in ((0.25, 0), (1.25, 1), (-0.75, -1)):
    print(f"  rate {rate:+.2f}: {TAU * rate:+.5f}; pi/2 + 2 pi ({k:+d}) = {math.pi / 2 + TAU * k:+.5f}")

print("E. Residue after the 10_3 fade (not counted as a defect): difference from frame 92")
base = Image.open(FRAMES / "10_doing_nothing/frame-0092.png").convert("RGB")
box = (100, 200, 1240, 700)
for f in (86, 87, 88, 89, 90, 91):
    img = Image.open(FRAMES / f"10_doing_nothing/frame-{f:04d}.png").convert("RGB")
    hist = ImageChops.difference(img.crop(box), base.crop(box)).convert("L").histogram()
    print(f"  frame {f}: largest difference {max(i for i, c in enumerate(hist) if c)} of 255")

print("F. 08 frames 198 and 199: what appears at the start of the 08_4 fade (finding F12)")
a = Image.open(FRAMES / "08_product_rule/frame-0198.png").convert("RGB")
b = Image.open(FRAMES / "08_product_rule/frame-0199.png").convert("RGB")
print(f"  full-frame difference bbox {ImageChops.difference(a, b).convert('L').point(lambda v: 255 if v > 24 else 0).getbbox()}")

print("G. Reading load at the end of chapter 10 (finding F16)")
source = (CRATE / "src/chapters/doing_nothing.rs").read_text()
textbook = "turning rate of e^(tX) = imaginary part of an eigenvalue of X (complex vectors allowed)"
precisely = "no eigenvalue of A in (−∞, 0] ⇒ one X with e^X = A, all eigenvalues −π < Im < π; real if A is (Higham; Gallier Thm 3.11)"
for text in (textbook, precisely):
    print(f"  present in source: {text in source}; {len(text)} characters")
total = len(textbook) + len(precisely) + len("precisely: ")
print(f"  {total} characters on screen from 90.0 s to 98.9 s (scratch timing): {total / 8.9:.1f} per second")

print(
    "H. Proposed card widths (finding F1, uncompiled): dx = max(size*0.011, (widest + 0.25)/2), "
    "w = dx + max(size*0.0095, widest/2 + 0.10)"
)
for label, at, size, rows in CARDS:
    widest = max(width(e, size) for row in rows for e in row)
    dx = max(size * 0.011, (widest + 0.25) / 2)
    report(label, rows, size, dx, dx + max(size * 0.0095, widest / 2 + 0.10), at)
print("  rounding to two places with the present widths does not repair the cards")
for label, size, rows in (
    ("rotation card, rounded", 34.0, [["0.5", "−0.87"], ["0.87", "0.5"]]),
    ("e_card, rounded", 30.0, [["0.91", "−0.97"], ["0.78", "0.52"]]),
):
    report(label, rows, size, size * 0.011, size * 0.011 + size * 0.0095)


def to_scene(x):
    return (x - 640) / 640 * 7.11


def lit_columns(chapter, frame, rows, columns, test):
    img = Image.open(FRAMES / f"{chapter}/frame-{frame:04d}.png").convert("RGB")
    return [x for x in range(*columns) if any(test(img.getpixel((x, y))) for y in range(*rows))]


print("I. Native measurements for findings F2, F3, F8")
bright = lambda p: max(p) > 70
xs = lit_columns("09_matrix_logs", 220, (345, 366), (900, 1280), bright)
print(f"  F2 dictionary, 09 frame 220: lit columns {xs[0]} .. {xs[-1]} of 0 .. 1279")
green = lambda p: p[1] > p[0] + 25 and p[1] > p[2] + 25
purple = lambda p: p[2] > p[1] + 25 and p[0] > p[1] + 10
g = lit_columns("10_doing_nothing", 170, (272, 292), (900, 1280), green)
u = lit_columns("10_doing_nothing", 170, (272, 292), (900, 1280), purple)
print(f"  F3, 10 frame 170: green rule ends at column {g[-1]}, purple label starts at column {u[0]}: {g[-1] - u[0] + 1} columns shared")
xs = lit_columns("10_doing_nothing", 200, (318, 340), (640, 1280), bright)
now = to_scene(xs[-1])
extra = width("radians ", 22.0)
print(f"  F8 spiral line, 10 frame 200: right edge scene {now:.2f}; 'radians ' adds {extra:.2f}: {now + extra:.2f}, safe area ends 6.96")
