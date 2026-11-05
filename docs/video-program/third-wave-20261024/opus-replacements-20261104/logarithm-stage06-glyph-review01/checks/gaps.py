"""The spacing and film-size numbers REVIEW.md quotes, each measured on a page.

    python3 -B .../checks/gaps.py     (from the repo root; PIL needs site packages)

Part 1: gap between two named glyphs, from ink components at level 0.5. Controls are in the same job and at the same sheet size.
Part 2: each new label's extent at its film size, scaled from its row's ink width on the sheet (Tex rows are drawn at 36, Text at 34).
These are extents in frame units and say nothing about what else is on screen at the time.
"""
import csv
import sys
from pathlib import Path

from PIL import Image

BUNDLE = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(BUNDLE / "checks"))
from touch_test import PAGES, components  # noqa: E402

PX_PER_UNIT = 135.0


def pieces(page, box):
    x0, y0, x1, y1 = box
    im = Image.open(PAGES / f"frame_{page:06d}.png").convert("L").crop(box)
    return [(x0 + c[0], y0 + c[1], x0 + c[2], y0 + c[3]) for c in components(im, 0.5) if c[4] >= 3]


def gap(page, box, left_index=0, right_index=1):
    found = sorted(pieces(page, box))
    return found[right_index][0] - found[left_index][2] - 1


print("# gap between minus and the character after it, px on the sheet at size 36")
print(f"  row 560 so -1 = 1 (product_rule.rs:343): {gap(47, (120, 465, 200, 510))} px  <- minus follows \\text")
print(f"  row 498 -1 = i dot i (control, leading minus): {gap(41, (50, 478, 110, 525))} px")
print(f"  row 565 L(z) - ell (control, a real subtraction): {gap(47, (540, 870, 610, 915))} px")

print("\n# gap between the previous glyph and a colon, px on the sheet at size 36")
for row, name, page, box in [
    (512, "X = 0:", 42, (150, 695, 220, 745)),
    (513, "X = 2 pi J:", 42, (225, 770, 280, 815)),
    (557, "(-inf, 0]:", 47, (365, 225, 410, 265)),
    (586, "z:", 49, (60, 870, 110, 910)),
    (559, "S\\text{: } (control, the delta string)", 47, (230, 390, 290, 440)),
]:
    print(f"  row {row} {name}: {gap(page, box)} px")

print("\n# new labels at film size")
rows = {}
with open(BUNDLE / "coverage/row-map.tsv", newline="", encoding="utf-8") as f:
    for r in csv.DictReader(f, delimiter="\t", quoting=csv.QUOTE_NONE):
        rows.setdefault(r["registry_row"], r)


def width(row, size):
    r = rows[str(row)]
    sheet = 36.0 if r["kind"] == "Tex" else 34.0
    return (int(r["right"]) - int(r["left"]) + 1) * size / sheet / PX_PER_UNIT


# centred labels: (row, size, centre x); left-anchored: (row, size, left x)
differ = width(565, 28)
print(f"  row 565 DIFFER size 28, centred at 1.2 (lifting_loops.rs:220): width {differ:.2f}, x {1.2 - differ / 2:.2f} to {1.2 + differ / 2:.2f}")
single = width(559, 28)
print(f"  row 559 singular S size 28, centred at 0 (unreachable.rs:1662): width {single:.2f}, x {-single / 2:.2f} to {single / 2:.2f}")
positive = width(575, 22)
print(f"  row 575 e^tp size 22, left at 2.2 (unreachable.rs:749,1309): width {positive:.2f}, x 2.20 to {2.2 + positive:.2f}")
ours = width(360, 22)
name = width(497, 22)
left = 0.9 + 0.7 - 0.2
print(f"  row 360 'ours: carried along' size 22, left at {left:.2f} (lifting_loops.rs:316,537): width {ours:.2f}, right {left + ours:.2f}")
print(f"  row 497 (ell) size 22, left at ours_right + 0.12 (lifting_loops.rs:634): width {name:.2f}, x {left + ours + 0.12:.2f} to {left + ours + 0.12 + name:.2f}")
print("  frame half width 7.11; the film's stated safe area is 6.90 (lifting_loops.rs:38)")

print("\n# DIFFER against the tall axis's foot: an estimate from the sheet, not a picture")
r = rows["565"]
top, bottom, left_px, right_px = int(r["top"]), int(r["bottom"]), int(r["left"]), int(r["right"])
scale = 28.0 / 36.0
half_px = (bottom - top + 1) * scale / 2
foot = -2.7 + -0.4 * 0.38  # W.p(0, -0.4), lifting_loops.rs:44-45,242
axis_x = 0.9
sheet_x = left_px + (axis_x - (1.2 - differ / 2)) / differ * (right_px - left_px + 1)
print(f"  axis foot at y {foot:.3f}, x {axis_x}; that x falls at sheet x {sheet_x:.0f}, between L (443-470) and its paren (476-486)")
for name, ink_top in (("paren of L(z), sheet y 875", 875), ("L, sheet y 878", 878)):
    y = -3.0 + (half_px - (ink_top - top) * scale) / PX_PER_UNIT
    print(f"  {name}: top of ink at y {y:.3f}, {foot - y:.3f} units below the foot, {(foot - y) * PX_PER_UNIT:.1f} px at 1080p")
