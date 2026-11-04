"""Predicted fit of the text this repair adds or moves.

    python3 -I -B checks/predicted_margins.py

A prediction, not a measurement: width = CMR10 ems x size x 0.01, the rule that was
checked against measured widths in logarithm-repair05-07/checks/predicted_margins.py
(within about 8%). TeX boxes are rougher still. Nothing here has been rendered.
Exit 1 if a margin is negative.
"""
import pathlib
import re
import sys

HERE = pathlib.Path(__file__).resolve().parent
CH = HERE.parent / "candidate" / "explainer_corpus" / "complex_analysis_visual" / "src" / "chapters"
GALLERY = (CH / "gallery.rs").read_text()
PLANES = (CH / "two_planes.rs").read_text()
COLOR = (CH / "reading_color.rs").read_text()

EM = {" ": 0.333, ",": 0.278, ":": 0.278, ".": 0.278, ";": 0.278, "=": 0.778, "|": 0.278, "(": 0.389, ")": 0.389,
      "π": 0.57, "→": 1.0, "−": 0.778, "×": 0.778, "+": 0.778, "°": 0.5, "?": 0.472, "/": 0.5, "'": 0.278, "…": 1.0,
      "≤": 0.778, ">": 0.778, "³": 0.4, "R": 0.736, "I": 0.361, "N": 0.75}
for c in "0123456789":
    EM[c] = 0.5
EM.update(a=.5, b=.556, c=.444, d=.556, e=.444, f=.306, g=.5, h=.556, i=.278, j=.306, k=.528, l=.278, m=.833, n=.556,
          o=.5, p=.556, q=.528, r=.392, s=.394, t=.389, u=.556, v=.528, w=.722, x=.528, y=.528, z=.444, L=.625, T=.722)


def width(text, size):
    return sum(EM[c] for c in text) * size * 0.01


def height(size):
    return 0.92 * size * 0.01


def const(src, name):
    return re.search(rf'pub const {name}: &str =\s*"([^"]*)";', src).group(1)


SAFE_X, SAFE_Y = 6.96, 3.85
rows = []


def need(name, margin):
    rows.append((margin, name))


# ---- 03
t = const(PLANES, "RESCALED")
need("03_6 caption fits the frame", SAFE_X - width(t, 18) / 2)
need("03_6 caption above the safe bottom", -3.5 - height(18) / 2 + SAFE_Y)
word, value = width("turns:", 22), width("0.50", 28)
for name, wx, vx, axis, left in (("input", -5.0, -4.2, -3.5, -6.2), ("output", 2.0, 2.8, 3.5, 0.8)):
    need(f"03_2 {name}: word clear of the frame", wx - word / 2 - left)
    need(f"03_2 {name}: word clear of its value", (vx - value / 2) - (wx + word / 2))
    need(f"03_2 {name}: value clear of the axis", axis - (vx + value / 2))
unit = 5.4 / 4.6
und = width(const(PLANES, "UNDEFINED"), 20) + 0.5
need("03_5 1/0 card inside the input frame, left", (-3.5 + -0.55 * unit - und / 2) - (-6.2))
need("03_5 1/0 card clear of the axis tick below it", (-0.35 + (-1.6 - 0.24) * unit) - (-0.35 - 2 * unit + 0.06))

# ---- 04
right = 4.55
t = const(COLOR, "RECIPE_RANGE")
need("04_7 range note fits the frame", SAFE_X - right - width(t, 20) / 2)
need("04_7 range note clear of the picture", right - width(t, 20) / 2 - 0.6)
t = "at i: −1 turn, at −i: −1 turn"
need("04_7 per-pole note fits the frame", SAFE_X - right - width(t, 24) / 2)
need("04_7 per-pole note clear of the picture", right - width(t, 24) / 2 - 0.6)
need("04_7 notes do not touch, vertically", (0.6 - height(24) / 2) - (-0.2 + height(20) / 2))
for name, text in (("zero of order 2", const(COLOR, "ORDER_TWO")), ("simple pole", const(COLOR, "SIMPLE_POLE"))):
    need(f"04 card '{name}': half width within the estimate the test uses", 0.96 - (width(text, 22) + 0.5) / 2)
legend_top = 2.2 + height(20) / 2
need("04 wheel arrow clear of 'positive'", 2.2 + 0.88 * 0.435 - legend_top)
need("04 wheel arrow inside the safe top", SAFE_Y - (2.2 + 0.88))

# ---- 05
zcol = 4.15
for name in ("CONVERGES", "ONE_POLE", "CRITICAL"):
    need(f"05_7 {name} fits right of the zero labels", SAFE_X - zcol - width(const(GALLERY, name), 22))
need("05_7 trivial zeros fit", SAFE_X - zcol - width("zeros at −2, −4, −6, …", 22))
rh = max(width("Riemann hypothesis:", 26), width("open", 26)) + 0.5
need("05_8 RH card fits the frame", SAFE_X - (zcol + 1.2) - rh / 2)
need("05_8 RH card clear of the zero labels", (zcol + 1.2) - rh / 2 - (2.72 + width("1/2 + 30.42i", 18)))
label = width("1/2 + 30.42i", 18)
need("05_7 zero labels end before the column", zcol - (2.72 + label))
need("05_7 top zero label inside the safe top", SAFE_Y - (3.01 + height(18) / 2))
caption = width(const(GALLERY, "SQUASHED") + "32", 18)
need("05_7 strip caption fits", SAFE_X - (1.6 + caption / 2))
need("05_7 strip caption clear of the wide plot's foot", (0.2 - 1.92) - (-2.85 + height(18) / 2))
need("05_8 strip caption clear of the lower third's top", (-2.85 - height(18) / 2) - (-3.14))
box = width(const(GALLERY, "OUT_BOX"), 18)
need("05_1 box name within its box's width", 3.4 - box)
need("05_1 box name clear of the count above it", (1.75 - 0.14) - (1.1 + 0.18 + height(18) / 2))
# the mosaic: rows at centre 1.55 and -1.35, panels 2.6 tall
top_row_top, top_row_foot, low_row_top = 1.55 + 1.3, 1.55 - 1.3, -1.35 + 1.3
need("05_9 tallest name, the fraction at 20, clear of its panel", 3.15 - 0.42 / 2 - top_row_top)
need("05_9 tallest name inside the safe top", SAFE_Y - (3.15 + 0.42 / 2))
need("05_9 lower names under the top row", top_row_foot - (0.1 + 0.2 / 2))
need("05_9 lower names over the lower row", 0.1 - 0.2 / 2 - low_row_top)

for margin, name in sorted(rows):
    print(f"  {margin:6.3f}  {name}")
bad = [n for m, n in rows if m < 0]
print(f"\n{len(rows)} margins, {len(bad)} negative" + (": " + "; ".join(bad) if bad else ""))
sys.exit(1 if bad else 0)
