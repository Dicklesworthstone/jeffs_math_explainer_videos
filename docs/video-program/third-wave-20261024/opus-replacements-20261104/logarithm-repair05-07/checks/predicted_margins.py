"""Predicted margins for the geometric tests added by this repair.

    python3 -I -B <this folder>/checks/predicted_margins.py

A prediction, not a test run. The checks that count are the #[test]s in the
three chapter files, which build the real text through Kit and read its real
boxes; root runs those. This file only estimates whether they should pass, using
width = CMR10 ems x size x 0.01, first compared with widths measured from the
preview02 pictures. TeX boxes are estimated the same way, more roughly.
"""
import math

EM = {" ": 0.333, ",": 0.278, ":": 0.278, ".": 0.278, ";": 0.278, "=": 0.778, "|": 0.278, "(": 0.389, ")": 0.389,
      "π": 0.57, "→": 1.0, "−": 0.778, "θ": 0.47, "×": 0.778, "+": 0.778, "°": 0.5, "?": 0.472}
for c in "0123456789":
    EM[c] = 0.5
EM.update(a=.5, b=.556, c=.444, d=.556, e=.444, f=.306, g=.5, h=.556, i=.278, j=.306, k=.528, l=.278, m=.833, n=.556,
          o=.5, p=.556, q=.528, r=.392, s=.394, t=.389, u=.556, v=.528, w=.722, x=.528, y=.528, z=.444, L=.625)


def width(text, size):
    return sum(EM[c] for c in text) * size * 0.01


def height(size):
    """Ascender to descender, the tall case."""
    return 0.92 * size * 0.01


print("-- the width rule against measured preview02 widths")
for text, size, seen in [
    ("the circle closes after one lap", 22, 2.8),
    ("a dial that forgets laps", 26, 2.6),
    ("one lap", 20, 0.64),
    ("0: no logarithm", 22, 1.47),
    ("length 2, angle 0.6π", 20, 1.73),
    ("candidate: πi → −πi, gap 0 → −1", 20, 2.9),
    ("principal angle, in π:", 22, 2.01),
    ("branch point", 26, 1.44),
    ("just above", 20, 0.92),
    ("just below → the bottom edge", 20, 2.71),
    ("no continuous logarithm on a loop that winds around 0", 26, 6.37),
]:
    w = width(text, size)
    print(f"  {w:5.2f} predicted  {seen:5.2f} measured  {100 * (w - seen) / seen:+5.1f}%  {text}")

SAFE_X, SAFE_Y = 6.96, 3.85
rows = []


def need(name, margin):
    rows.append((margin, name))


def left_box(text, size, x, y):
    w, h = width(text, size), height(size)
    return x, x + w, y - h / 2, y + h / 2


def mid_box(text, size, x, y):
    w, h = width(text, size), height(size)
    return x - w / 2, x + w / 2, y - h / 2, y + h / 2


# ---- chapter 07: Z centre (-3.4, -0.3), scale 1, W_SCALE 0.8
zx, zy = -3.4, -0.3
top, bottom = zy + 0.8 * math.pi, zy - 0.8 * math.pi
beside, column = zx + 1.65, 0.9
strip_right = zx + 0.8 * math.log(2.6)
strip_left = zx + 0.8 * math.log(0.3)
need("07 strip ends before its labels", beside - strip_right)
r = left_box("just below → bottom edge", 20, beside, bottom + 0.4)[1]
need("07 F-edge bottom label vs rays card", (column - 0.15) - r)
r = left_box("just above → top edge", 20, beside, top - 0.4)[1]
need("07 F-edge top label vs column", (column - 0.15) - r)
for text, size in [("rays at angle θ  →  horizontal lines at height θ", 22),
                   ("circles |z| = r  →  vertical segments at ln r", 22),
                   ("where do the two sides go?", 24),
                   ("angle just above the cut: near π; just below: near −π", 22),
                   ("any curve from 0 out to infinity that never crosses itself", 22)]:
    need(f"07 card on screen: {text[:28]}", SAFE_X - left_box(text, size, column, 0)[1])
for text, sign in [("just above", 1), ("just below", -1)]:
    l, r, b, t = mid_box(text, 20, zx - 2.95, zy + sign * 0.45)
    near = min(abs(b - zy), abs(t - zy))
    outer = zx - math.sqrt(2.3 ** 2 - near ** 2)
    need(f"07 F-sides {text} vs outer circle", (outer - 0.05) - r)
    need(f"07 F-sides {text} vs safe", l + SAFE_X)
    need(f"07 F-sides {text} vs marked line", near - (2.6 * math.sin(0.07) + 0.05))
l, r, b, t = mid_box("blocked", 20, zx - 1.3, zy + 0.75)
need("07 F-blocked corner vs unit circle", math.hypot(r - zx, b - zy) - 1.05)
l, r, b, t = left_box("z → 0: off to the left", 18, -6.7, zy)
need("07 F-zero note vs safe", l + SAFE_X)
need("07 F-zero note vs strip", (strip_left - 0.1) - r)
wb = left_box("branch point", 26, column, -3.05)
ab = left_box("any curve from 0 out to infinity that never crosses itself", 22, column, -2.55)
need("07 F-bp word vs card above", ab[2] - wb[3])
need("07 F-bp word vs bottom of safe", wb[2] + SAFE_Y)
for text, y in [("top edge: included", top), ("bottom edge: left out", bottom)]:
    l, r, b, t = left_box(text, 20, beside, y)
    need(f"07 V1 {text} vs strip", l - strip_right)
    need(f"07 V1 {text} vs column", (column - 0.15) - r)
r = left_box("principal angle, in π:", 22, column, 0.85)[1]
need("07 F-gap words vs live angle", (column + 2.2) - (r + 0.1))
need("07 F-gap not too wide", 0.5 - ((column + 2.2) - r))

# ---- chapter 06: W centre (0.9, -2.7), scale .38; Z centre (-4.1, -0.3), scale 1.25, half_w 2.9
wx, wy, ws = 0.9, -2.7, 0.38
jx, j0 = 1.95, -1.3
j_top, j_bot = j0 + ws * math.pi, j0 - ws * math.pi
print(f"-- 06 E7: arrow {j_top:.3f} to {j_bot:.3f}, length/scale = {(j_top - j_bot) / ws:.6f} (2π = {2 * math.pi:.6f})")
l, r, b, t = left_box("candidate: πi → −πi, gap 0 → −1", 20, jx + 0.25, j0)
need("06 E7 words vs gauge", (5.6 - 0.25 - 0.1) - r)
need("06 E7 inset bottom vs safe", (j_bot - 0.12) + SAFE_Y)
for text in ["πi", "0", "−πi"]:
    ll, rr, _, _ = mid_box(text, 18, jx - 0.4, 0)
    need(f"06 E7 label {text} vs tall axis", ll - (wx + 0.2))
    need(f"06 E7 label {text} vs inset axis/words", min(jx - 0.08, l) - rr)
stub, bracket, lap_x = wx - 0.8, wx - 0.9, wx - 1.35
for text in ["π", "2π", "3π", "4π"]:
    ll, _, _, _ = mid_box(text + "i", 18, wx - 0.55, 0)
    need(f"06 E1 tick {text}i vs bracket stub", ll - (stub + 0.05))
zl, zr, _, _ = mid_box("0", 18, wx - 0.6, wy)
need("06 E3 zero label vs bracket stub", zl - (stub + 0.05))
need("06 E3 zero label vs base line", (wx - 1.2 * ws - 0.03) - zr)
l, r, _, _ = mid_box("one lap", 20, lap_x, 0)
need("06 E1 word vs bracket", (bracket - 0.05) - r)
need("06 E1 word vs number's panel", l - (-4.1 + 2.9 + 0.05))
tb = mid_box("no continuous logarithm on a loop that winds around 0", 26, 1.2, -3.55)
differ_bottom = -3.0 - 0.27  # the formula's box, tall exponent; measured near -3.18
need("06 E10 theorem vs formula (formula estimated)", differ_bottom - tb[3])
need("06 E10 theorem vs tall axis", (wy - 0.4 * ws) - tb[3])
need("06 E10 theorem vs bottom of safe", tb[2] + SAFE_Y)
need("06 E10 theorem vs right of safe", SAFE_X - tb[1])
need("06 E10 theorem vs the number z", tb[0] - mid_box("the number z", 22, -4.1, -3.25)[1])
l, r, b, t = left_box("0 is not inside", 20, -4.1 + 0.15, -0.3 - 1.6)
need("06 E6 note vs axis", l - (-4.1 + 0.05))
need("06 E6 note vs side circle", (-0.3 - 1.25 * 0.8) - t)
need("06 E6 note vs unit circle", (-0.3 - 1.25) - t)
tw = mid_box("its logarithm", 22, wx - 1.0, wy + ws * 4.4 * math.pi)
four = mid_box("4πi", 18, wx - 0.55, wy + ws * 4 * math.pi)
need("06 E9 name vs 4πi", tw[2] - four[3])
need("06 E9 name vs axis", (wx - 0.1) - tw[1])
need("06 E9 name vs number's panel", tw[0] - (-4.1 + 2.9))
need("06 E8 short trail vs stroke", 0.02 - ws * 2 * math.pi * 0.005)

# ---- chapter 05: ARROWS centre (-3.4, -.9), scale 1.25; OUT centre (2.7, -.15), scale .85; IN centre (-4.9, -.15), scale .3
ax, ay, a_s = -3.4, -0.9, 1.25
ox, oy, o_s = 2.7, -0.15, 0.85
ring_top = oy + o_s * math.e
zw_tip = (ax + a_s * 1.8 * 1.12 * math.cos(math.radians(80)), ay + a_s * 1.8 * 1.12 * math.sin(math.radians(80)))
zw_l = mid_box("zw", 28, *zw_tip)[0]
ti = left_box("2i", 26, ax + a_s * -0.55, ay + a_s * 2.0)
ni = left_box("i", 26, ax + a_s * -0.2, ay + a_s * 1.15)
need("05 D2 2i vs axis", (ax - 0.05) - ti[1])
need("05 D2 2i vs zw", zw_l - ti[1])
need("05 D3 i vs axis", (ax - 0.05) - ni[1])
need("05 D3 i corner vs unit circle", math.hypot(ni[1] - ax, ni[2] - ay) - (a_s + 0.02))
for text, size in [("the circle closes after one lap", 22), ("a dial that forgets laps", 26)]:
    l, r, b, t = left_box(text, size, ox + 0.15, 2.6)
    need(f"05 D4 line vs axis: {text[:16]}", l - (ox + 0.1))
    need(f"05 D4 line vs outer ring: {text[:16]}", b - (ring_top + 0.05))
    need(f"05 D4 line vs safe: {text[:16]}", SAFE_X - r)
    need(f"05 D4 line top vs title: {text[:16]}", 3.2 - t)
gx = ox + o_s * 2 * math.cos(0.6 * math.pi)
gl, gr, gb, gt = mid_box("length 2, angle 0.6π", 20, gx - 0.5, 2.4)
need("05 D6 words vs axis", (ox - 0.05) - gr)
need("05 D6 words vs outer ring", gb - (ring_top + 0.05))
lw = left_box("laps:", 24, ox + 2.35, 1.7)
ray_x = ox + o_s * math.exp(1.2) * math.cos(math.pi / 4)
need("05 D7 laps vs ray end", lw[0] - (ray_x + 0.05))
need("05 D7 laps vs count", (ox + 3.15 - 0.05) - lw[1])
need("05 D7 count vs safe", SAFE_X - (ox + 3.15 + 0.4))
al, ar, ab_, at = mid_box("each lap drawn a little further out, to keep the laps apart", 18, ox - 0.4, -3.5)
ol = mid_box("output", 22, ox + 2.6, -3.25)[0]
need("05 D8 aid vs output", (ol - 0.05) - ar)
need("05 D8 aid vs axis", (oy - 3.0) - at)
need("05 D8 aid vs bottom of safe", ab_ + SAFE_Y)
need("05 D8 aid vs left of safe", al + SAFE_X)
l, r, b, t = mid_box("0: no logarithm", 22, ox + 2.2, -2.6)
need("05 D5 word vs outer ring", math.hypot(l - ox, t - oy) - (o_s * math.e + 0.1))
need("05 D5 word vs -45° ray end", (oy - o_s * math.exp(1.2) * math.sin(math.pi / 4)) - t)
need("05 D5 word vs right of safe", SAFE_X - r)
need("05 D5 word vs bottom of safe", b + SAFE_Y)
tx, ty = l + 0.15, t + 0.04
angle = math.atan2(ty - oy, tx - ox)
off = abs(angle - round(angle / (math.pi / 4)) * math.pi / 4)
print(f"-- 05 D5: leader at {math.degrees(angle):.1f}°, {math.degrees(off):.1f}° from the nearest ray")
need("05 D5 leader vs nearest ray (rad, need > 5°)", off - math.radians(5))
line_end = -4.9 + 0.3 * 1.2
need("05 D1 words vs gridline end", (-4.9 + 0.3 * 0.55 + 0.35) - (line_end + 0.05))
z0 = mid_box("0", 18, -4.9 + 0.85, -0.15 + 0.2)
need("05 D9 0 vs level link", z0[2] - (-0.15 + 0.02))
each_em = 0.466 + (0.389 + 0.57 + 0.778 + 0.5 + 0.57 + 0.528 + 0.389 + 0.345) * 0.7 + 1.33 + 1.28 + 0.28 + 0.67 + 0.528 + 2.83
each_w = each_em * 30 * 0.01
need("05 D13 rule vs strip (TeX estimated)", (-2.3 - each_w / 2) - (line_end + 0.05))
need("05 D13 rule vs outer ring (TeX estimated)", (ox - o_s * math.e - 0.05) - (-2.3 + each_w / 2))

print("-- predicted margin of each inequality, tightest first (negative would fail)")
rows.sort()
for margin, name in rows:
    print(f"  {margin:+6.3f}  {name}")
print(f"{len(rows)} inequalities; tightest {rows[0][0]:+.3f}; failing {sum(m < 0 for m, _ in rows)}")
