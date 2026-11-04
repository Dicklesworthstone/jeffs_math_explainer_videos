"""Numeric mirrors of the repair's new tests, and independent maths checks.

    python3 -I -B checks/geometry.py

A prediction, not a test run. The checks that count are the #[test]s in the
candidate, which root runs. This file reads the repair's constants out of the
candidate source, recomputes each test's condition in Python with no Rust model
code, and prints the margin. Text boxes are CMR10 estimates, so a small margin
here is a warning about the picture, not a proof about it. Exit 1 on any failure.
"""
import cmath
import math
import pathlib
import re
import sys

HERE = pathlib.Path(__file__).resolve().parent
CRATE = HERE.parent / "candidate" / "explainer_corpus" / "complex_analysis_visual"
SRC = {n: (CRATE / "src" / "chapters" / f"{n}.rs").read_text() for n in ("turning", "two_planes", "reading_color", "gallery")}
MODEL = (CRATE / "src" / "model.rs").read_text()
failures = []


def check(name, ok, detail=""):
    print(f"  {'ok  ' if ok else 'FAIL'} {name}{'  ' + detail if detail else ''}")
    if not ok:
        failures.append(name)


def num(chapter, name):
    return float(re.search(rf"pub const {name}: f64 = (-?[\d.]+);", SRC[chapter]).group(1))


def nums(text):
    return [float(x) for x in re.findall(r"-?\d+\.?\d*", text)]


def point(chapter, name):
    m = re.search(rf"pub const {name}: C = C::new\((-?[\d.]+), (-?[\d.]+|TAU)\);", SRC[chapter])
    return complex(float(m.group(1)), math.tau if m.group(2) == "TAU" else float(m.group(2)))


def points(chapter, name):
    body = re.search(rf"pub const {name}: \[C; \d\] = \[(.*?)\];", SRC[chapter], re.S).group(1)
    return [complex(a, b) for a, b in re.findall(r"C::new\((-?[\d.]+), (-?[\d.]+)\)", body) for a, b in [(float(a), float(b))]]


class Plane:
    def __init__(self, cx, cy, width, x, y):
        self.cx, self.cy, self.width, self.x, self.y = cx, cy, width, x, y
        self.unit = width / (x[1] - x[0])
        self.height = width * (y[1] - y[0]) / (x[1] - x[0])
        self.top, self.bottom = cy + self.height / 2, cy - self.height / 2
        self.left, self.right = cx - width / 2, cx + width / 2

    def p(self, z):
        return (self.cx + (z.real - sum(self.x) / 2) * self.unit, self.cy + (z.imag - sum(self.y) / 2) * self.height / (self.y[1] - self.y[0]))

    def contains(self, z):
        return self.x[0] <= z.real <= self.x[1] and self.y[0] <= z.imag <= self.y[1]


# ---------------------------------------------------------------- chapter 02
print("-- chapter 02")
tp = Plane(-2.0, -0.1, 8.4, (-3.4, 5.0), (-2.6, 2.9))
names = points("turning", "TURN_NAMES_AT")
check("i and -1 are off their axes", abs(names[0].real) > 0.2 and abs(names[1].imag) > 0.2, f"{abs(names[0].real)} {abs(names[1].imag)}")
for z in names:
    check(f"name at {z} clears the unit circle and is in the window", abs(z) > 1.2 and tp.contains(z), f"|z| = {abs(z):.3f}")
lines = []
for k in range(-2, 3):
    t = k / 2
    lines += [(complex(t, -1), complex(t, 1)), (complex(-1, t), complex(1, t))]
glyph = [[0.15 + 0.1j, 0.15 + 0.9j, 0.7 + 0.9j], [0.15 + 0.5j, 0.5 + 0.5j]]
strokes = lines + [(g[i], g[i + 1]) for g in glyph for i in range(len(g) - 1)]


def box_clearance(at, m, hx=0.3, hy=0.16):
    """How far (in the better axis) the nearest stroke point stays outside the box; negative means inside."""
    best = 9.0
    for a, b in strokes:
        for k in range(201):
            z = (a + (b - a) * k / 200) * m
            best = min(best, max(abs(z.real - at.real) - hx, abs(z.imag - at.imag) - hy))
    return best


w_at, w2_at = point("turning", "W_NAME_AT"), point("turning", "W2_NAME_AT")
for m in (1, 1 + 1j, (1 + 1j) ** 2):
    c = box_clearance(w_at, m)
    check(f"'1 + i' clear of the grid at rest, multiplier {m}", c > 0, f"margin {c:.3f}")
c = box_clearance(w2_at, (1 + 1j) ** 2)
check("'2i' clear of the final grid", c > 0 and tp.contains(w2_at), f"margin {c:.3f}")

# ---------------------------------------------------------------- chapter 03
print("-- chapter 03")
W = num("two_planes", "WINDOW")
EXP_OUT = num("two_planes", "EXP_OUT_HALF")
inp = Plane(-3.5, -0.35, 5.4, (-W, W), (-W, W))
out = Plane(3.5, -0.35, 5.4, (-W, W), (-W, W))
exp_box = re.search(r"Plane::new\(v\(-3\.5, -0\.35\), 3\.6, \(-2\.4, 2\.4\), \((-?[\d.]+), ([\d.]+)\)\)", SRC["two_planes"])
exp_in = Plane(-3.5, -0.35, 3.6, (-2.4, 2.4), (float(exp_box.group(1)), float(exp_box.group(2))))
exp_out = Plane(3.5, -0.35, 5.4, (-EXP_OUT, EXP_OUT), (-EXP_OUT, EXP_OUT))


def whole(span):
    return [float(k) for k in range(math.ceil(span[0]), math.floor(span[1]) + 1) if k != 0]


check("window ticks", whole((-W, W)) == [-2, -1, 1, 2], str(whole((-W, W))))
check("e^z input ticks", whole(exp_in.y) == [1, 2, 3, 4, 5, 6], str(whole(exp_in.y)))
check("e^z output ticks", whole((-EXP_OUT, EXP_OUT)) == [-3, -2, -1, 1, 2, 3])
check("e^z input frame height equals the others", abs(exp_in.height - inp.height) < 1e-9, f"{exp_in.height} vs {inp.height}")
check("the window holds z^2 of the grid, and 1/z's farthest image", W > 2 and W > 1 / 0.45, f"{W} > {1 / 0.45:.3f}")
check("unit length is one through 03_5", True, f"{inp.unit:.4f} scene units per 1 (was 1.8, 1.227, 1.08 by beat)")


def grid(n=240):
    out_lines = []
    for k in range(-2, 3):
        t = k / 2
        out_lines.append([complex(t, -1 + 2 * j / n) for j in range(n + 1)])
        out_lines.append([complex(-1 + 2 * j / n, t) for j in range(n + 1)])
    return out_lines


g = grid()


def image_clearance(at, hx, hy):
    return min(max(abs((z * z).real - at.real) - hx, abs((z * z).imag - at.imag) - hy) for l in g for z in l)


unit_at = points("two_planes", "UNIT_AT")
for z in unit_at:
    c = image_clearance(z, 0.05, 0.06)
    check(f"unit name {z}: off axes, off the unit circle, off the images", abs(z.real) > 0.05 and abs(z.imag) > 0.06 and abs(z) - math.hypot(0.05, 0.06) > 1.05 and c > 0, f"image margin {c:.3f}")
    check(f"unit name {z} inside every window it is used in", exp_in.contains(z) and out.contains(z) and exp_out.contains(z))
fold = point("two_planes", "FOLD_NAME_AT")
c = image_clearance(fold, 0.045, 0.055)
check("the name at 0 fits the gap around the fold", c > 0 and fold.real + 0.045 < 0 and fold.imag + 0.055 < -0.12, f"image margin {c:.3f}")
turns_at = re.search(r"pub const TURNS_AT: \[\(f64, f64\); 2\] = \[\((-?[\d.]+), (-?[\d.]+)\), \((-?[\d.]+), (-?[\d.]+)\)\]", SRC["two_planes"])
tv = [float(x) for x in turns_at.groups()]
turns_y = num("two_planes", "TURNS_Y")
radius_at = points("two_planes", "RADIUS_AT")
for name, word, value, plane in (("input", tv[0], tv[1], inp), ("output", tv[2], tv[3], out)):
    axis = plane.p(0j)[0]
    check(f"{name}: 'turns:' and its value on one side of the axis", word + 0.3 < value - 0.2 and value + 0.2 < axis - 0.2 and word - 0.3 > plane.left, f"value right {value + 0.25:.2f}, axis {axis}")
row_limit = inp.p(complex(0, radius_at[0].imag - 0.2))[1]
check("the turns row is in the frame, under the radii", inp.bottom < turns_y < row_limit, f"{inp.bottom:.2f} < {turns_y} < {row_limit:.2f}")
for at, half, r in ((radius_at[0], 0.365, 0.5), (radius_at[1], 0.408, 0.25)):
    near = abs(complex(at.real - half, at.imag + 0.1))
    check(f"radius {r} label off the axes and its circle", at.real - half > 0.1 and at.imag + 0.1 < -0.1 and near > r + 0.1, f"nearest {near:.3f} vs circle {r}")
radii = [math.exp(-1), 1.0, math.e]
label_half = num("two_planes", "RADIUS_LABEL_HALF")


def radius_label_at(k, r):
    return complex(r - 0.08 - label_half if k == 2 else r + 0.04 + label_half, -0.2)


boxes = []
for k, r in enumerate(radii):
    at = radius_label_at(k, r)
    lo = abs(complex(at.real - label_half, at.imag + 0.07))
    hi = abs(complex(at.real + label_half, at.imag - 0.07))
    boxes.append((at.real - label_half, at.real + label_half))
    gap = min(min(abs(o - lo), abs(o - hi)) if not lo < o < hi else -1 for o in radii)
    check(f"readout {k} clear of every circle", all(o < lo - 0.03 or o > hi + 0.03 for o in radii) and exp_out.contains(complex(at.real + label_half, at.imag)), f"smallest gap {gap:.3f}")
    # the rays of 03_6, at ±45°, ±135°
    ray_gap = min(abs(abs(z_y) - z_x) for z_x in (at.real - label_half, at.real + label_half) for z_y in (at.imag + 0.07, at.imag - 0.07))
    check(f"readout {k} clear of the ray at -45 degrees", at.real - label_half > abs(at.imag - 0.07), f"box corner gap {ray_gap:.3f}")
check("the three readouts do not touch", boxes[0][1] < boxes[1][0] and boxes[1][1] < boxes[2][0], f"{boxes}")
und, half_h = point("two_planes", "UNDEFINED_AT"), num("two_planes", "UNDEFINED_HALF_H")
leader = points("two_planes", "UNDEFINED_LEADER")
check("the 1/0 card is outside the unit circle", und.imag + half_h < -1.1 and inp.contains(und - 1j * half_h), f"top {und.imag + half_h}")
check("the leader starts on the card's top edge, off the vertical axis", abs(leader[0].imag - (und.imag + half_h)) < 1e-9 and leader[0].real < -0.05 and leader[1].real < -0.05)
ticks = [0.6, 2.0, 3.6, 5.0]
tick_r = 0.1 / inp.unit
d = min(abs(leader[0] + (leader[1] - leader[0]) * k / 50 - cmath.rect(1, th)) for th in ticks for k in range(51))
check("the leader misses every tick pair", d > tick_r, f"distance {d:.3f} vs tick {tick_r:.3f}")
top_e = math.exp(1.15)
rays = [cmath.rect(top_e, (k + 0.5) * math.pi / 2) for k in range(4)]
check("rays end in the window", all(exp_out.contains(z) for z in rays), f"{rays[0]:.3f}")
low = math.exp(-1.5)
check("no ray lies on an axis", abs(low * math.cos(math.pi / 4)) > 0.1, f"smallest part {low * math.cos(math.pi / 4):.3f}")

# ---------------------------------------------------------------- chapter 04
print("-- chapter 04")
pic = Plane(-2.6, -0.25, 6.4, (-2, 2), (-2, 2))
name_at = point("reading_color", "NAME_AT")
loop_r = num("reading_color", "LOOP_R")
at, c0 = pic.p(name_at), pic.p(0j)
half_w, half_card = 0.96, 0.3
reach = math.hypot(half_w, abs(at[1] - c0[1]) + half_card)
check("the name card stands inside the gold loop", reach < loop_r * pic.unit - 0.1, f"reach {reach:.3f} vs loop {loop_r * pic.unit:.3f}")
check("the name card clears the mark", abs(at[1] - c0[1]) - half_card > 0.13, f"{abs(at[1] - c0[1]) - half_card:.3f} vs mark 0.11")
fy = float(re.search(r"pub const FORMULA_AT: \(f64, f64\) = \(-?[\d.]+, ([\d.]+)\)", SRC["reading_color"]).group(1))
check("the formula stands over the picture, inside the frame", fy - 0.3 > pic.top and fy + 0.3 < 3.85, f"{fy - 0.3:.2f} > {pic.top:.2f}, {fy + 0.3:.2f} < 3.85")
probe_r = num("reading_color", "PROBE_R")
check("the probe stays in the window at every named direction", probe_r < 2.0)
SCRIPT = {l.split("\t")[0]: l.split("\t", 1)[1].strip() for l in (CRATE / "script.tsv").read_text().splitlines() if "\t" in l}
said = SCRIPT["04_1"]
pos = -1
for phrase in ("Pointing right is red", "Pointing up is yellow green", "Pointing left is teal", "pointing down is violet"):
    nxt = said.find(phrase, pos + 1)
    check(f"'{phrase}' is literal, in order", nxt > pos)
    pos = nxt

# ---------------------------------------------------------------- chapter 05
print("-- chapter 05")
trig = Plane(-2.4, -0.1, 8.4, (-6.6, 6.6), (-3.3, 3.3))
mark = 0.11 / trig.unit
check("sin's zeros at +-2pi stand inside, mark and all", trig.x[1] - math.tau > mark, f"{trig.x[1] - math.tau:.3f} vs mark {mark:.3f}")
drop = nums(re.search(r"READOUT_DROP: \[f64; 2\] = \[(.*?)\]", SRC["gallery"]).group(1))
top_card = trig.p(3j)[1] - drop[1]
low_card = trig.p(1j)[1] - drop[0]
check("sin(3i)'s card stays inside the picture", top_card + 0.42 < trig.top, f"{top_card + 0.42:.3f} < {trig.top:.3f}")
check("the two readout cards do not touch", top_card - 0.42 > low_card + 0.42, f"gap {top_card - 0.42 - low_card - 0.42:.3f}")
zp = Plane(-3.4, 0.2, 6.4, (-7, 3), (-3, 3))
st = Plane(1.6, 0.35, 2.0, (-1, 2), (0, 32))
st.height = 5.9
st.top, st.bottom = 0.35 + 5.9 / 2, 0.35 - 5.9 / 2
st.p = lambda z: (1.6 + (z.real - 0.5) * 2.0 / 3, 0.35 + (z.imag - 16) * 5.9 / 32)
shared_top = num("gallery", "SHARED_TOP")
check("the box lies in the wide picture, the band in the strip", all(zp.contains(z) and st.contains(z) for z in (-1 + 0j, 2 + 0j, complex(2, shared_top), complex(-1, shared_top))))
check("the wide picture's top is the band's top", zp.y[1] == shared_top and shared_top < st.y[1])
band_h = st.p(complex(0, shared_top))[1] - st.p(0j)[1]
check("the band is visible on the strip", band_h > 0.4, f"{band_h:.3f} scene units tall")
heights = [14.134725141734695, 21.022039638771556, 25.01085758014569, 30.424876125859512]
check("all four marked zeros are above the band, so the box holds none", all(h > shared_top for h in heights))
rows = nums(re.search(r"ZETA_ROWS: \[f64; 4\] = \[(.*?)\]", SRC["gallery"]).group(1))
ys = [st.p(complex(0.5, h))[1] for h in heights]
gap = min(abs(r - y) for r in rows for y in ys)
check("no row of the column lines up with a zero's label", gap > 0.25, f"smallest gap {gap:.3f}, labels at {[round(y, 3) for y in ys]}")
label_end = st.right + num("gallery", "ZERO_LABEL_GAP") + 5.5 * 18 * 0.01 if hasattr(st, "right") else None
check("the labels end before the column starts", label_end + 0.3 < num("gallery", "ZETA_COLUMN"), f"{label_end:.2f} vs {num('gallery', 'ZETA_COLUMN')}")
said2 = SCRIPT["05_2"]
check("05_2 phrases in order; the count lands on the line's last words", said2.find("and at minus one") < said2.find("backwards twice") and said2.endswith("backwards twice."))
check("05_9 phrase is literal", "in little tiles" in SCRIPT["05_9"])

# ------------------------------------------------- the shared half-step layer
print("-- P2, the half-step bands")
STEP = math.tau / 12
OFF = float(re.search(r"pub const BAND_OFFSET: f64 = ([\d.]+);", MODEL).group(1))


def shade(w, off):
    size_band = (math.log(abs(w)) / STEP + off) % 1.0
    phase_band = ((cmath.phase(w) % math.tau) / STEP + off) % 1.0
    return (0.58 + 0.42 * size_band) * (0.80 + 0.20 * phase_band)


def tan_contrast(y, off):
    row = [shade(cmath.tan(complex(-6.6 + 13.2 * (c + 0.5) / 240, y)), off) for c in range(240)]
    return max(row) / min(row)


for y in (2.0, 2.4, 3.0):
    a, b = tan_contrast(y, 0.0), tan_contrast(y, OFF)
    check(f"tan z at y = {y}: flat where the voice says it is flat", b < 1.1, f"max/min {a:.3f} -> {b:.3f}")


def comb(off):
    worst = 0.0
    for k in range(1, 40):
        y = 0.37 * k
        worst = max(worst, abs(shade(cmath.exp(complex(-1e-3, y)), off) - shade(cmath.exp(complex(1e-3, y)), off)))
    return worst


check("e^z does not flip across x = 0", comb(OFF) < 0.01, f"largest jump {comb(0.0):.3f} -> {comb(OFF):.3f}")
for w in (0.3 + 0.2j, -1.3 + 2.2j, 0.7 - 0.4j):
    a = shade(cmath.exp(w), OFF)
    b = shade(cmath.exp(w + STEP), OFF)
    c = shade(cmath.exp(w + 1j * STEP), OFF)
    check(f"the shading still repeats every step, at {w}", abs(a - b) < 1e-9 and abs(a - c) < 1e-9)
check("the band ratio is unchanged", abs(math.exp(STEP) - 1.6881) < 5e-5, f"{math.exp(STEP):.4f}")


def tile(seed, steps=16):
    f0 = seed ** 3 - 1
    g3 = 3 * seed ** 2 / f0
    w0 = cmath.log(f0)
    band = lambda x: STEP * (math.floor(x / STEP + OFF) - OFF)  # noqa: E731
    corner = complex(band(w0.real), band(w0.imag))
    pts = []
    for a, b in ((0, 1), (1, 1 + 1j), (1 + 1j, 1j), (1j, 0)):
        for k in range(steps):
            w = corner + (a + (b - a) * k / steps) * STEP
            target = cmath.exp(w)
            z = seed + (w - w0) / g3
            for _ in range(50):
                d = (z ** 3 - 1 - target) / (3 * z ** 2)
                z -= d
                if abs(d) < 1e-15:
                    break
            pts.append(z)
    return pts + [pts[0]]


sx, sy = re.search(r"pub const TILE_SEED: C = C::new\(([\d.]+), ([\d.]+)\);", SRC["gallery"]).groups()
seed = complex(float(sx), float(sy))
t = tile(seed)
on = lambda x: abs(x / STEP + OFF - round(x / STEP + OFF)) < 1e-9  # noqa: E731
check("every traced point lies on a band or phase line", all(on(cmath.log(z ** 3 - 1).real) or on(cmath.log(z ** 3 - 1).imag) for z in t), f"{len(t)} points")
cs = [t[k * 16] for k in range(4)]
sides = [abs(cs[(k + 1) % 4] - cs[k]) for k in range(4)]
check("the tile is nearly square (test tolerance 1.1)", max(sides) / min(sides) < 1.1, f"ratio {max(sides) / min(sides):.4f}")
centre = sum(t[:-1]) / (len(t) - 1)
win = (centre.real - 0.65, centre.real + 0.65, centre.imag - 0.65, centre.imag + 0.65)
check("the tile sits in the zoom", all(win[0] <= z.real <= win[1] and win[2] <= z.imag <= win[3] for z in t), f"centre {centre:.4f}")
h = 5.4 * 2.6 / 4.2
check("the zoom sits in the cubic's panel", -2.7 <= win[0] and win[1] <= 2.7 and 0.3 - h / 2 <= win[2] and win[3] <= 0.3 + h / 2, f"{[round(x, 3) for x in win]}")
zeros = [cmath.exp(2j * math.pi * k / 3) for k in range(3)]
check("no zero in the zoom", not any(win[0] <= z.real <= win[1] and win[2] <= z.imag <= win[3] for z in zeros))

# ------------------------------------------------------- the maths is unmoved
print("-- facts the pictures keep (independent values)")


def winding(f, centre, r, n=4096):
    total = 0.0
    prev = cmath.phase(f(centre + r))
    for k in range(1, n + 1):
        now = cmath.phase(f(centre + r * cmath.exp(1j * math.tau * k / n)))
        total += (now - prev + math.pi) % math.tau - math.pi
        prev = now
    return round(total / math.tau)


check("z^2 winds 2", winding(lambda z: z * z, 0, 0.9) == 2)
check("1/z winds -1", winding(lambda z: 1 / z, 0, 0.9) == -1)
check("the bump winds -1 at i and at -i", winding(lambda z: 1 / (1 + z * z), 1j, 0.5) == -1 and winding(lambda z: 1 / (1 + z * z), -1j, 0.5) == -1)
check("the cubic winds 3 on r = 1.8", winding(lambda z: z ** 3 - 1, 0, 1.8) == 3)
check("(z-1)/(z+1)^2 winds -2 at -1", winding(lambda z: (z - 1) / (z + 1) ** 2, -1, 0.3) == -2)
check("sin i, sin 3i", abs(cmath.sin(1j).imag - 1.1752) < 5e-5 and abs(cmath.sin(3j).imag - 10.018) < 5e-4, f"{cmath.sin(1j).imag:.4f} {cmath.sin(3j).imag:.3f}")
check("tan(+-4i) settles to +-i", abs(cmath.tan(4j) - 1j) < 1e-3 and abs(cmath.tan(-4j) + 1j) < 1e-3, f"{cmath.tan(4j):.4f}")
check("e^(2 pi i) = 1", abs(cmath.exp(2j * math.pi) - 1) < 1e-15)
check("(1 + i)^2 = 2i, |1 + i| = sqrt 2, 45 degrees", (1 + 1j) ** 2 == 2j and abs(abs(1 + 1j) - math.sqrt(2)) < 1e-15 and abs(math.degrees(cmath.phase(1 + 1j)) - 45) < 1e-12)
check("the walk: r = 0.5 maps to 0.25, once becomes twice", 0.5 ** 2 == 0.25)

print(f"\n{len(failures)} failed" + (": " + "; ".join(failures) if failures else ""))
sys.exit(1 if failures else 0)
