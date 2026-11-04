#!/usr/bin/env python3
"""Numbers and layout of the staged chapter 06-10 repair, mirrored in Python.

Nothing here runs the crate. It repeats the arithmetic of the Rust tests the repair adds, so a
wrong tolerance shows now rather than at root's first build, and it boxes the new labels with
an estimated text width (0.46 em per character, em = size x 0.01 scene units, TeX estimated by
hand) to find overlaps. An estimate is not a picture: the first preview must still be read.

Reads nothing; writes nothing. Exit 1 on any failure.
"""
import cmath
import math
import sys

fails = []


def check(ok, what):
    if not ok:
        fails.append(what)
        print("FAIL:", what)


# ---------------------------------------------------------------- one_derivative
Z0 = 1 + 1j
d = 2 * Z0
for k in range(4):
    step = cmath.rect(0.3, 0.35 + math.tau / 4 * k)
    image = step * cmath.rect(1 + (abs(d) - 1) * 1.0, cmath.phase(d) * 1.0)
    check(abs(image - d * step) < 1e-12, f"fan {k} ends as f'(z0) h")
    w = Z0**2 + image
    check(-1.6 <= w.real <= 1.6 and 0.4 <= w.imag <= 3.6, f"fan {k} image in the output window: {w}")
    z = Z0 + step
    check(-0.2 <= z.real <= 2.2 and -0.2 <= z.imag <= 2.2, f"fan {k} step in the input window")


def flip_at(z, t):
    return complex(z.real, t * z.imag)


def squash_at(z, t):
    return complex(z.real, t * z.imag - (t - 1))


for x, y in ((0.7, 0.7), (1.3, 0.7), (1.3, 1.3), (0.7, 1.3)):
    z = complex(x, y)
    check(flip_at(z, 1.0) == z and flip_at(z, -1.0) == z.conjugate(), "flip runs from the point to its conjugate")
    check(squash_at(z, 1.0) == z, "squash starts at the point")
    check(abs(squash_at(z, 2.0) - complex(x, 2.0 * y - 1.0)) < 1e-12, "squash ends on the tall rectangle")
    # the flip and the squash stay in the mirror window, x in [-0.6, 2.6], y in [-1.6, 1.6]
    for t in (1.0, 0.5, 0.0, -0.5, -1.0):
        check(-1.6 <= t * y <= 1.6, "flip stays in the window")
    for t in (1.0, 1.5, 2.0):
        check(-1.6 <= t * y - (t - 1) <= 1.6, f"squash stays in the window at t={t}: {t * y - (t - 1)}")

# ---------------------------------------------------------------- loop_sums
STEPS = 24
pts = [0j]
for k in range(STEPS):
    a = cmath.rect(1, math.tau * k / STEPS)
    b = cmath.rect(1, math.tau * (k + 1) / STEPS)
    mid = cmath.rect(1, math.tau * (k + 0.5) / STEPS)
    pts.append(pts[-1] + mid * (b - a))
for k in range(STEPS // 2 + 1):
    check(abs(pts[k + STEPS // 2] - pts[k]) < 1e-9, f"second lap lies over the first at {k}")
check(abs(pts[STEPS // 2]) < 1e-9, "the chain is back at its start after half the steps")
check(24 / 96 == 0.25, "finer step share")
# the circulation head points anticlockwise
at, r, a1 = (1.0, 2.0), 0.26, 5.4
tip = (at[0] + r * math.cos(a1), at[1] + r * math.sin(a1))
prev = (at[0] + r * math.cos(a1 - 0.35), at[1] + r * math.sin(a1 - 0.35))
cross = (tip[0] - at[0]) * (tip[1] - prev[1]) - (tip[1] - at[1]) * (tip[0] - prev[0])
check(cross > 0, "circulation head points anticlockwise")
# the shrunk step's labels stay clear of each other
scale = 2.4
mid = cmath.rect(1, math.tau * 3.5 / 24)
a, b = cmath.rect(1, math.tau * 3 / 24), cmath.rect(1, math.tau * 4 / 24)
dz, prod = b - a, mid * (b - a)
for t in (1.0, 0.25):
    def label(w):
        q = w * t
        tipv = (2.2 + scale * q.real, -0.6 + scale * q.imag)
        u = q / abs(q) * 0.5
        return tipv[0] + u.real, tipv[1] + u.imag
    (x1, y1), (x2, y2) = label(dz), label(prod)
    check(abs(y1 - y2) > 0.3 or abs(x1 - x2) > 0.8, f"step and product names clear at t={t}")

# ---------------------------------------------------------------- boundary
for n in (8, 16):
    for k in range(n // 2):
        z1 = cmath.rect(0.5, math.tau * k / n)
        z2 = cmath.rect(0.5, math.tau * (k + n // 2) / n)
        check(abs(1 / (1 + z1**2) - 1 / (1 + z2**2)) < 1e-12, "opposite samples share a value")
step = 4.6 / 920
for x in (-1.0, 0.0, 1.0):
    k = (x + 1.6) / step
    check(abs(k - round(k)) < 1e-9, f"{x} on the curve grid")
check(abs(1 / (1 + 2.0**2) - 0.2) < 1e-12, "the bump at 2 is 0.2, which the row says it matches")
check(abs(abs(1j - 1) - math.sqrt(2)) < 1e-12, "the segment from 1 to i is root 2 long")
check(1.0 + math.sqrt(2) < 2.5 and 1.0 - math.sqrt(2) > -2.5, "the root 2 circle fits the picture")
# the lifted picture
top, bottom = -0.2 + 2.8, -0.2 - 2.8
check(top + 0.18 < 3.0 and bottom >= -3.0, f"picture window {bottom}..{top}")

# ---------------------------------------------------------------- rigidity
check(abs(abs(cmath.sin(3j)) - 10.018) < 0.01, "the climb ends on the 10.018 reading")
unit = 5.6 / 0.84
check(1 / (12 * math.pi) * unit > 0.12, "the cross leaves the innermost zero showing")
print(f"innermost zero at {1 / (12 * math.pi) * unit:.3f} scene units; cross arm 0.12")
check(0.0 + 2.8 <= 3.05 and 0.0 - 2.8 >= -3.0, "sin(1/z) plate window")

# ---------------------------------------------------------------- arrows with buff 0
for name, length, thick in (
    ("06_1 step", 0.45 * 4.6 / 2.4, 4.0),
    ("08_1 product", 2 * math.sin(math.pi / 24) * 2.4, 4.0),
    ("08_2 chain", 2 * math.sin(math.pi / 24) * 3.0, 3.0),
    ("08_3 column", 2 * math.sin(math.pi / 24) * 4.8 / 6.27, 3.0),
    ("08_4 row", 2 * math.sin(math.pi / 24) * 0.45, 3.0),
    ("07_5 step", 0.6 * 1.35, 4.0),
):
    shaft = min(thick * 0.015, 0.1 * length)
    print(f"{name:14} L = {length:.3f}, shaft {shaft:.3f}, at 4K {shaft * 270:.1f} px wide")
    check(length > 0, name)


# ---------------------------------------------------------------- label boxes
def box(text, size, x, y, left=False, tex_w=None):
    w = tex_w if tex_w is not None else len(text) * 0.46 * size * 0.01
    h = size * 0.012
    x0 = x if left else x - w / 2
    return (text, x0, x0 + w, y - h / 2, y + h / 2)


def pinned(xlim=7.11, ylim=4.0, *boxes):
    for b in boxes:
        check(b[1] >= -xlim and b[2] <= xlim and b[3] >= -ylim and b[4] <= ylim, f"off the frame: {b}")


def clear(*boxes):
    for i, a in enumerate(boxes):
        for b in boxes[i + 1 :]:
            if a[1] < b[2] and b[1] < a[2] and a[3] < b[4] and b[3] < a[4]:
                check(False, f"overlap: {a[0]!r} and {b[0]!r}")


SCENES = {
    "06_1": [
        box("quotient", 36, 0.0, 3.1, tex_w=3.6),
        box("settles", 32, 2.75, 3.1, tex_w=1.2),
        box("f(z)=z2", 28, 1.05, 2.0, tex_w=1.5),
        box("Im out", 20, 2.4, 1.88),
        box("z", 30, -3.6, 2.0, tex_w=0.2),
        box("change", 20, 2.4 + 0.92 * 1.4375, -0.6 + 0.72 * 1.4375),
    ],
    "06_2": [
        box("the ratio", 22, 5.85, 1.55),
        box("live re", 22, 4.98, 1.12, left=True, tex_w=0.5),
        box("+", 22, 5.62, 1.12),
        box("live im", 22, 5.82, 1.12, left=True, tex_w=0.5),
        box("i", 22, 6.38, 1.12),
        box("along a spiral", 20, 5.85, 0.72),
    ],
    "06_3": [
        box("the image", 18, 2.75, 1.45, left=True),
        box("exact stretch and turn", 18, 2.75, 1.15, left=True),
        box("z0 name", 26, 0.96, -2.1, tex_w=0.5),
    ],
    "06_3 fan": [
        box("times f'", 30, 3.3, 2.3, tex_w=1.4),
        box("a stretch and a turn", 22, 3.45, 1.35),
    ],
    "06_5": [
        box("z=x+iy", 34, -2.2, 3.35, tex_w=1.9),
        box("f=u+iv", 34, 2.2, 3.35, tex_w=1.9),
        box("step right", 26, -1.2, 2.55),
        box("right tex", 34, 1.2, 2.55, tex_w=1.5),
        box("ux", 24, -1.0, 2.08, tex_w=0.35),
        box("is how fast u changes as x changes", 22, -0.7, 2.08, left=True),
        box("step up", 26, -1.2, 1.35),
        box("up tex", 34, 1.6, 1.35, tex_w=3.4),
        box("must agree", 24, -3.2, 1.95),
    ],
    "06_7": [
        box("symbols", 32, -2.4, -1.45, tex_w=2.9),
        box("=", 34, -0.55, -1.45, tex_w=0.3),
        box("numbers", 32, 1.35, -1.45, tex_w=2.9),
        box("= x (2+2i)", 34, 4.8, -1.45, tex_w=2.2),
        box("stretch turn", 24, 0.2, -2.7, tex_w=5.0),
    ],
    "06_9": [
        box("x+iy to x+2iy", 30, -2.6, 2.85, tex_w=2.8),
        box("no complex derivative", 22, 4.2, -0.25),
    ],
    "07_1": [
        box("real line in", 18, -3.4 + 0.75 * 1.923, -0.15 - 0.16 * 1.923),
        box("imaginary in", 18, -3.4 + 0.4 * 1.923, -0.15 + 1.12 * 1.923),
        box("real line out", 18, 3.4 + 0.8 * 1.923, -0.15 - 0.16 * 1.923),
        box("imaginary out", 18, 3.4 - 0.72 * 1.923, -0.15 - 0.16 * 1.923),
        box("180", 26, 3.4 + 0.42 * 1.923, -0.15 + 0.42 * 1.923),
        box("90", 26, -3.4 + 0.62 * 1.923, -0.15 + 0.62 * 1.923),
        box("kept in", 22, -3.4 - 0.4 * 1.923, -0.15 + 0.9 * 1.923),
        box("kept out", 22, 3.4 + 0.48 * 1.923, -0.15 - 0.54 * 1.923),
    ],
    "07_6": [box("|z|2: one point, no disk", 20, 2.2, -2.4, left=True)],
    "07_5": [box("|z|2 x jagged g, |g| <= 1 = M: derivative 0 at 0", 20, 4.2, 1.55)],
    "08_1": [
        box("sum", 32, 0.0, 3.15, tex_w=4.2),
        box("Im", 20, -3.6, 2.28),
        box("shrink", 20, 0.0, 2.55),
        box("Re", 20, -0.86, -0.4),
        box("unit circle", 20, -3.6 + 2 * 1.35 * math.cos(-1.1), -0.4 + 2 * 1.35 * math.sin(-1.1)),
        box("product name", 22, 1.11, -0.89, tex_w=1.0),
        box("step name", 22, 1.30, 0.08, tex_w=0.45),
    ],
    "08_2": [
        box("second lap, over the first", 20, 5.3, -0.4),
        box("the chain, radius 1.5 about (2.4, -0.4)", 0, 2.4, -0.4, tex_w=3.0),
    ],
    "09_1": [
        box("split", 34, 2.0, 3.2, tex_w=4.6),
        box("rest", 20, 4.0, 2.2),
        box("result", 36, 4.0, 1.1, tex_w=4.0),
        box("Re", 20, -0.26, -0.2),
    ],
    "09_3": [
        box("bump", 28, -5.0, 3.0, tex_w=2.2),
        box("Im", 20, -3.3, 2.78),
        box("formula", 34, 2.0, 3.2, tex_w=4.0),
        box("values", 22, 4.0, 1.86),
        box("pair up", 18, 4.0, -0.75),
        box("avg8", 24, 4.0, -1.4),
        box("avg16", 24, 4.0, -1.95),
    ],
    "09_6": [
        box("coef", 32, 2.0, 3.25, tex_w=6.2),
        box("theta", 24, 4.0, 2.55, tex_w=1.6),
        box("values", 22, 4.0, 2.1),
        box("slope", 18, 1.35, -2.55),
        box("read1", 26, 4.0, -2.55, tex_w=3.0),
        box("next", 18, 1.5, -3.4),
        box("read2", 26, 4.0, -3.4, tex_w=3.0),
        box("list", 26, -3.3, -3.45, tex_w=7.5),
        box("turned", 20, 4.0, -1.95),
    ],
    "09_9": [
        box("row 1", 22, 3.9, 2.55),
        box("60 terms at 2.6: 212 (outside: it blows up)", 22, 3.9, 2.0),
        box("works", 24, 3.9, 1.45),
    ],
    "10_2": [
        box("identity", 34, 4.3, 2.6, tex_w=3.2),
        box("real", 22, 4.3, 1.8, tex_w=2.7),
        box("Re", 20, -0.06, -0.3),
    ],
    "10_4": [
        box("sin(1/z)", 30, -4.7, 3.1, tex_w=1.5),
        box("Im", 20, -2.8, 2.98),
        box("Re", 20, 0.24, 0.0),
        box("zeros", 20, 4.3, 2.6),
        box("decimals", 20, 4.3, 2.2, tex_w=3.5),
    ],
    "10_6": [
        box("sin z", 30, -5.1, 2.625, tex_w=0.9),
        box("Im", 20, -2.8, 2.505),
        box("Re", 20, 0.44, -0.3),
        box("size", 24, 2.7, 0.6, left=True),
        box("live", 24, 3.95, 0.6, left=True, tex_w=0.7),
        box("bound", 22, 4.3, 1.4),
    ],
}
for name, boxes in SCENES.items():
    pinned(7.11, 4.0, *boxes)
    clear(*boxes)
print(f"label sets boxed: {len(SCENES)}")


# ---------------------------------------------------------------- cards against rings
# carded() pads its text by 0.5 x 0.35 scene units, so a short label's card is much larger than
# the label. Each card must keep clear of the ring it names and the marks beside it.
def card_box(chars, size, cx, cy, unit, text_w=None):
    """Half-extents in plane units around the plane point (cx, cy)."""
    w = (text_w if text_w is not None else chars * 0.46 * size * 0.01) + 0.5
    h = size * 0.012 + 0.35
    return cx - w / 2 / unit, cx + w / 2 / unit, cy - h / 2 / unit, cy + h / 2 / unit


def ring_clear(name, b, centre, r):
    """No point of the card's rectangle lies on the ring about centre."""
    x0, x1, y0, y1 = b
    cx, cy = centre
    nearest = math.hypot(max(x0 - cx, 0, cx - x1), max(y0 - cy, 0, cy - y1))
    farthest = max(math.hypot(x - cx, y - cy) for x in (x0, x1) for y in (y0, y1))
    check(nearest > r or farthest < r, f"{name}: the card lies across the ring of radius {r:.3f}")


U9 = 1.12  # picture(): 5.6 scene units over 5 plane units
ROOT2 = math.sqrt(2)
for name, chars, size, at, rings in (
    ("a little beyond", 15, 18, (1.7, -0.3), [((0, 0), 0.8), ((0, 0), 0.5)]),
    ("r = 1/2", 7, 20, (-1.05, 0.3), [((0, 0), 0.5)]),
    ("a = 0", 5, 20, (0.85, -0.72), [((0, 0), 0.5)]),
    ("r = 2", 5, 20, (-2.0, 1.85), [((0, 0), 2.0)]),
    ("i", 1, 22, (-0.42, 1.3), [((0, 0), 1.0), ((1, 0), ROOT2)]),
    ("-i", 2, 22, (-0.42, -1.3), [((0, 0), 1.0), ((1, 0), ROOT2)]),
    ("radius 1", 8, 20, (1.5, 0.7), [((0, 0), 1.0)]),
    ("root 2", 10, 20, (0.95, 0.95), [((1, 0), ROOT2)]),
    ("share at i", 4, 24, (0.72, 1.0), [((0, 1), 0.22), ((0, 0), 2.0)]),
    ("share at -i", 4, 24, (0.72, -1.0), [((0, -1), 0.22), ((0, 0), 2.0)]),
):
    b = card_box(chars, size, at[0], at[1], U9)
    check(b[0] >= -2.5 and b[1] <= 2.5 and b[2] >= -2.5 and b[3] <= 2.5, f"{name}: card leaves the picture {b}")
    for centre, r in rings:
        ring_clear(name, b, centre, r)
# the point 1's card stays below the letter a, which sits at 1 + 0.18 - 0.2i
b = card_box(1, 22, 1.0, -0.62, U9)
check(b[3] < -0.3, f"the card at 1 touches the name a: {b}")
# chapter 07: the cards at 0 and at 1 keep clear of their dots; scene units, relative to the dot
for name, dx, dy, r in (("0", 0.45, -0.38, 0.11), ("1", 0.0, -0.4, 0.08)):
    x0, x1, y0, y1 = card_box(1, 22, dx, dy, 1.0)
    nearest = math.hypot(max(x0, 0, -x1), max(y0, 0, -y1))
    check(nearest > r - 0.03, f"chapter 07's card at {name} covers its dot: {nearest:.3f} against {r}")
# chapter 10: the sine plate, 0.75 scene units per plane unit
b = card_box(2, 22, 0.85, 3.0, 0.75)
check(b[0] > 0.0 and b[3] <= 3.5, f"the card at 3i: {b}")
print("cards checked against their rings")

print("failures:", len(fails))
sys.exit(1 if fails else 0)
