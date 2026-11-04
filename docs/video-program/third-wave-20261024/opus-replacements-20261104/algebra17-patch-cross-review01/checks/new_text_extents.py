"""Where the patch's new text lands, from CMR10 advance widths, calibrated on the render.

Run from the repo root with python3 -B (PIL needs site packages). Reads only.

Width in units = em width of the string x size x 0.01, then checked against spans the lint
printed and against pixel widths in the frames. Nothing here renders Rust text, and kerning
is ignored; the calibration shows the size of that error.
"""
from pathlib import Path

from PIL import Image

FRAMES = Path(
    "narration_lab/third-wave-20261024/root/five-film-release-20261027/algebra-combined-repair-frames01/17_applications"
)
# CMR10 advance widths in em, from TeX's font metrics. The checkout holds no loose font file
# to measure, so these stand in for it and the calibration below says how well they do.
LOWER = dict(zip("abcdefghijklmnopqrstuvwxyz", (0.5, 0.5556, 0.4444, 0.5556, 0.4444, 0.3056, 0.5, 0.5556, 0.2778, 0.3056, 0.5278, 0.2778, 0.8333, 0.5556, 0.5, 0.5556, 0.5278, 0.3917, 0.3944, 0.3889, 0.5556, 0.5278, 0.7222, 0.5278, 0.5278, 0.4444)))
UPPER = dict(zip("ABCDEFGHIJKLMNOPQRSTUVWXYZ", (0.75, 0.7083, 0.7222, 0.7639, 0.6806, 0.6528, 0.7847, 0.75, 0.3611, 0.5139, 0.7778, 0.625, 0.9167, 0.75, 0.7778, 0.6806, 0.7778, 0.7361, 0.5556, 0.7222, 0.75, 0.75, 1.0278, 0.75, 0.75, 0.6111)))
OTHER = {" ": 0.3333, ".": 0.2778, ",": 0.2778, ":": 0.2778, ";": 0.2778, "-": 0.3333, "?": 0.4722, "×": 0.7778, "→": 1.0, "+": 0.7778, "=": 0.7778}


def em(s):
    return sum(LOWER.get(c) or UPPER.get(c) or OTHER.get(c) or 0.5 for c in s)


def width(s, size):
    return em(s) * size * 0.01


print("\n# calibration, against what the render and the lint report")
for s, size, seen, where in (
    ("forgets the column", 24, 1.96, "lint span -1.53..0.43"),
    ("The quotient keeps exactly what it was built to keep", 26, 6.04, "lint span -3.02..3.02"),
    ("Can arithmetic catch a copying error?", 36, 544 / 90, "frame 231, 368 to 912 px"),
    ("3. Why does changing representative within a coset keep it?", 27, 643 / 90, "frame 830, 318 to 961 px"),
    ("Equal pieces that cover the group", 25, 334 / 90, "frame 779, 698 to 1032 px"),
):
    w = width(s, size)
    print(f"  {w:5.2f} model, {seen:5.2f} seen ({w / seen:5.3f})  {where}")

print("\n# new or changed text: centre x, y, size; left and right edges against the safe area +-6.96")
items = [
    ("Valid only if the total is divisible by 11", 0.0, -2.2, 28, "c"),
    ("weight difference × value difference", 0.0, -3.05, 24, "c"),
    ("Distinct positions: the weight difference is nonzero", 0.0, -3.05, 24, "c"),
    ("Distinct symbols: the value difference is nonzero", 0.0, -3.05, 24, "c"),
    ("Two nonzero factors: this swap is detected", 0.0, -3.05, 24, "c"),
    ("Group: how moves combine", 2.5, 1.9, 25, "c"),
    ("Subgroup: moves that stay together", 2.5, 1.1, 25, "c"),
    ("Isomorphism: same structure, new description", 2.5, 0.3, 25, "c"),
    ("Cosets: equal pieces that cover the group", 2.5, -0.5, 25, "c"),
    ("Normal subgroup: pieces inherit an operation", 2.5, -1.3, 25, "c"),
    ("1. Which residues does 6 generate on the twelve-clock?", -3.6, 2.0, 27, "l"),
    ("2. Can a four-clock be isomorphic to two two-position clocks?", -3.6, 0.75, 27, "l"),
    ("3. Why does changing representative within a coset keep it?", -3.6, -0.5, 27, "l"),
    ("positions holding a 1", 3.2, 2.8, 22, "c"),
    ("1s in the received word", 3.2, 2.8, 22, "c"),
    ("each row: a coset of 16 words", 3.3, 0.35, 24, "c"),
    ("syndrome", 4.95, 0.1, 22, "c"),
    ("sent", -1.48, 0.03, 22, "c"),
    ("decoded", -3.96, 0.03, 22, "c"),
    ("forgets the column", -0.55, -3.7, 24, "c"),
    ("The quotient keeps exactly what it was designed to keep", 0.0, -3.7, 26, "c"),
]
for s, x, y, size, mode in items:
    w = width(s, size)
    left, right = (x, x + w) if mode == "l" else (x - w / 2, x + w / 2)
    flag = "" if -6.96 <= left and right <= 6.96 else "  OVER SAFE"
    print(f"  {left:6.2f} .. {right:5.2f}  w {w:4.2f}  y {y:5.2f}  size {size}  {s}{flag}")

print("\n# clearances that decide a fit, in units")
print(f"  17_11 captions: clock's '3' label right edge -0.94 + half glyph; widest caption left edge {2.5 - max(width(t, 25) for t, *_ in items[5:10]) / 2:.2f}")
print(f"  17_16 heading vs key: heading right edge {3.2 + width('positions holding a 1', 22) / 2:.2f} (or {3.2 + width('1s in the received word', 22) / 2:.2f} with the correction); the key's first box starts at {5.0 - 0.25:.2f}, its '4' at y 2.5")
print(f"  17_15 'rows_are' left edge {3.3 - width('each row: a coset of 16 words', 24) / 2:.2f}; the word's last box ends at {-5.4 + 0.75 * 6 + 0.3:.2f}")
tex_em = 0.5 + 0.833 + 0.52 + 1 + 0.389 + 1.889 + 0.333 + 1.0 + 0.389
print(f"  17_5 label '7 Delta d (mod 11)' size 20, by TeX metrics: width {tex_em * 0.2:.2f}, so x {-6.05 - tex_em * 0.1:.2f} .. {-6.05 + tex_em * 0.1:.2f}; safe edge -6.96, first column digit about -5.05")

print("\n# a tex width check on the render: frame 358's 7x6+6x4=66 at size 34")
im = Image.open(FRAMES / "frame-0358.png").convert("L")
band = im.crop((300, 360, 980, 396))
xs = [x for x in range(band.width) if max(band.crop((x, 0, x + 1, band.height)).getdata()) > 90]
print(f"  ink {300 + xs[0]} to {300 + xs[-1]} px, {(xs[-1] - xs[0]) / 90:.2f} units")
print(f"  so 'x 7' labels at y 0.35 (px {360 - 0.35 * 90:.0f}) sit above that row (px 362 top) and below the lane at y 0.9 (px {360 - 0.9 * 90:.0f})")
