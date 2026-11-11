"""geometry-cand03.py: independent CC0 geometry for CC2's boundary candidate03 (no compile, no render).

Every number comes from the file under test: the constants (SMALL_R, MID_R, SHRUNK_R, HOLE, BOUND_R,
A_RIDE, RHO_RIDE, A_AT, A_NAME_FROM_CENTRE, A_NAME_HIDDEN, A_NAME_EASE), the two planes
(close_up(), picture()), the line widths at their call sites, the two shrink tweens, and the test
module's PX, A_HALF, CLEAR and widened boxes. The three formula bodies (shrink_names, a_leader,
a_name_shown) are bound by exact text: if the source differs from what this script evaluates, it
stops. Box model only: label ink boxes are the test module's, the dashes are taken as a full circle.

usage: python3 -I -B geometry-cand03.py CANDIDATE_BOUNDARY_RS
exit: 0 when every check passes
"""
import ast
import math
import operator
import os
import re
import sys

src = open(sys.argv[1], encoding="utf-8").read()
film, _, tests = src.partition("\n#[cfg(test)]\nmod tests {")
assert tests, "no test module"
results = []


def check(name, ok, detail=""):
    results.append((name, bool(ok)))
    print(("PASS " if ok else "FAIL ") + name + ("  [" + detail + "]" if detail else ""))


OPS = {ast.Add: operator.add, ast.Sub: operator.sub, ast.Mult: operator.mul, ast.Div: operator.truediv}


def arith(expr, names=None):
    """Evaluate a Rust f64 literal expression: numbers, + - * /, parentheses, known names."""
    expr = expr.replace("_f64", "")

    def ev(n):
        if isinstance(n, ast.Expression):
            return ev(n.body)
        if isinstance(n, ast.Constant) and isinstance(n.value, (int, float)):
            return float(n.value)
        if isinstance(n, ast.BinOp) and type(n.op) in OPS:
            return OPS[type(n.op)](ev(n.left), ev(n.right))
        if isinstance(n, ast.UnaryOp) and isinstance(n.op, ast.USub):
            return -ev(n.operand)
        if isinstance(n, ast.Name) and names and n.id in names:
            return names[n.id]
        raise ValueError("not a literal expression: " + expr)
    return ev(ast.parse(expr, mode="eval"))


def const(name, text=film):
    m = re.search(r"(?:pub )?const %s: f64 = ([^;]+);" % name, text)
    assert m, name
    return arith(m.group(1), {"PX": PX} if text is tests else None)


def const_c(name):
    m = re.search(r"pub const %s: C = C::new\(([^,]+), ([^)]+)\);" % name, film)
    assert m, name
    return complex(arith(m.group(1)), arith(m.group(2)))


def plane(fn):
    m = re.search(r"pub fn %s\(\) -> Plane \{[^}]*?Plane::new\(v\(([^,]+), ([^)]+)\), ([^,]+), \(([^,]+), ([^)]+)\), \(([^,]+), ([^)]+)\)\)" % fn, film, re.S)
    assert m, fn
    cx, cy, width, x0, x1, y0, y1 = (arith(g) for g in m.groups())
    # draw.rs Plane::new: height follows the width, so a unit is the same length both ways
    return {"centre": complex(cx, cy), "unit": width / (x1 - x0), "x": (x0, x1), "y": (y0, y1)}


def body(fn):
    m = re.search(r"pub fn %s\([^)]*\) -> [^{]+\{\n(.*?)\n\}\n" % fn, film, re.S)
    assert m, fn
    return m.group(1)


PX = 270.0
m = re.search(r"const PX: f64 = ([\d.]+);", tests)
PX = float(m.group(1))
A_HALF = tuple(arith(g, {"PX": PX}) for g in re.search(r"const A_HALF: \(f64, f64\) = \(([^,]+), ([^)]+)\);", tests).groups())
CLEAR = float(re.search(r"const CLEAR: f64 = ([\d.]+);", tests).group(1))
rho_half = re.findall(r"const RHO_HALF: \(f64, f64\) = \(([^,]+), ([^)]+)\);", tests)
check("tests: every RHO_HALF is the same", len(set(rho_half)) == 1 and len(rho_half) >= 2, str(rho_half))
RHO_HALF = tuple(arith(g, {"PX": PX}) for g in rho_half[0])
AN = re.search(r"const A_NAME_HALF: \(f64, f64\) = \(([^,]+), ([^)]+)\);", tests).groups()
A_NAME_HALF = tuple(arith(g, {"PX": PX}) for g in AN)

SMALL_R, MID_R, SHRUNK_R, HOLE, BOUND_R = (const(n) for n in ["SMALL_R", "MID_R", "SHRUNK_R", "HOLE", "BOUND_R"])
A_RIDE, RHO_RIDE, A_NAME_EASE = const("A_RIDE"), const("RHO_RIDE"), const("A_NAME_EASE")
A_AT, A_NAME_FROM_CENTRE = const_c("A_AT"), const_c("A_NAME_FROM_CENTRE")
hid = re.search(r"pub const A_NAME_HIDDEN: \[f64; 2\] = \[([^,]+), ([^\]]+)\];", film).groups()
A_NAME_HIDDEN = (arith(hid[0]), arith(hid[1]))
cp, pic = plane("close_up"), plane("picture")
U = cp["unit"]
print("source constants: SMALL_R %.3f MID_R %.3f SHRUNK_R %.3f HOLE %.3f BOUND_R %.3f A_RIDE %.5f RHO_RIDE %.5f"
      % (SMALL_R, MID_R, SHRUNK_R, HOLE, BOUND_R, A_RIDE, RHO_RIDE))
print("close_up unit %.5f scene per plane unit; picture unit %.5f; PX %.0f; CLEAR %.3f (%.2f px)"
      % (U, pic["unit"], PX, CLEAR, CLEAR * PX))

# the formulas, bound by exact text
SHRINK_NAMES = """    let unit = close_up().unit();
    [C::new(0.0, r + A_RIDE / unit), C::new(r + RHO_RIDE / unit, 0.0)]"""
A_LEADER = """    let unit = close_up().unit();
    let foot = A_RIDE - 12.5 / 270.0;
    [
        C::new(0.0, (HOLE + 0.015 + 0.02) / unit),
        C::new(0.0, r + (foot - 0.02) / unit),
    ]"""
A_NAME_SHOWN = """    let [lo, hi] = A_NAME_HIDDEN;
    ((lo - r).max(r - hi) / A_NAME_EASE).clamp(0.0, 1.0)"""
check("formula shrink_names is the one evaluated here", body("shrink_names") == SHRINK_NAMES)
check("formula a_leader is the one evaluated here", body("a_leader") == A_LEADER)
check("formula a_name_shown is the one evaluated here", body("a_name_shown") == A_NAME_SHOWN)


def shrink_names(r):
    return complex(0.0, r + A_RIDE / U), complex(r + RHO_RIDE / U, 0.0)


def a_leader(r):
    foot = A_RIDE - 12.5 / 270.0
    return complex(0.0, (HOLE + 0.015 + 0.02) / U), complex(0.0, r + (foot - 0.02) / U)


def a_name_shown(r):
    lo, hi = A_NAME_HIDDEN
    return min(1.0, max(0.0, max(lo - r, r - hi) / A_NAME_EASE))


# line widths at their call sites (width x 0.01 scene, half on each side; round caps on open ends)
def width_at(pattern):
    m = re.search(pattern, film)
    assert m, pattern
    return float(m.group(1)) * 0.01


W_SMALL = width_at(r"let small = circle_in\(stage, &cp, SMALL_R, output\(\), ([\d.]+), 1\.0\);")
W_HOLE = width_at(r"\.radius\(HOLE\)\s*\.style\(line_style\(warn\(\), ([\d.]+)\)\)")
W_DASH = width_at(r"\.add\(polyline\(pts\)\.style\(line_style\(color, ([\d.]+)\)\)\.build\(\)\),")
W_LEAD = width_at(r"\.style\(faded_line\(Palette::ink\(\), ([\d.]+), 0\.6\)\)")
W_DISK = width_at(r"let unit = p\.circle\(stage, ZERO, 1e-3, hero\(\), ([\d.]+)\)\?;")
print("widths (scene): small %.3f hole %.3f dashes %.3f leader %.3f disks %.3f" % (W_SMALL, W_HOLE, W_DASH, W_LEAD, W_DISK))
tw = re.findall(r"stage\.play\(tween_linear\(radius, (MID_R|SHRUNK_R), ([\d.]+)\)\?\)\?;", film)
check("shrink tweens read from the source", [t[0] for t in tw] == ["MID_R", "SHRUNK_R"], str(tw))
T1, T2 = float(tw[0][1]), float(tw[1][1])

# radii: a fine grid plus every 60 fps frame of both tweens
radii = sorted({SHRUNK_R + (SMALL_R - SHRUNK_R) * k / 4000 for k in range(4001)}
               | {SMALL_R + (MID_R - SMALL_R) * f / (T1 * 60) for f in range(int(round(T1 * 60)) + 1)}
               | {MID_R + (SHRUNK_R - MID_R) * f / (T2 * 60) for f in range(int(round(T2 * 60)) + 1)})


def box(c, half):
    return (c.real - half[0], c.imag - half[1], c.real + half[0], c.imag + half[1])


def box_inmost(b):
    """Least distance from the centre (0) to the box: 0 if it contains the centre."""
    dx = max(b[0], 0.0, -b[2])
    dy = max(b[1], 0.0, -b[3])
    return math.hypot(dx, dy)


def box_outmost(b):
    return max(math.hypot(x, y) for x in (b[0], b[2]) for y in (b[1], b[3]))


def box_gap(p, q):
    dx = max(p[0] - q[2], q[0] - p[2], 0.0)
    dy = max(p[1] - q[3], q[1] - p[3], 0.0)
    return math.hypot(dx, dy)


least = {}


def note(key, value, r):
    if key not in least or value < least[key][0]:
        least[key] = (value, r)


ring_out = HOLE + W_HOLE / 2
dash_in = BOUND_R * U - W_DASH / 2
foots = set()
for r in radii:
    R = r * U
    a_c, rho_c = (z * U for z in shrink_names(r))
    a_b, rho_b = box(a_c, A_HALF), box(rho_c, RHO_HALF)
    line_out, line_in = R + W_SMALL / 2, R - W_SMALL / 2
    for nm, b in (("a", a_b), ("rho", rho_b)):
        note(nm + " / circle line (drawn gap)", box_inmost(b) - line_out, r)
        note(nm + " / dashes", dash_in - box_outmost(b), r)
        note(nm + " / hole ring", box_inmost(b) - ring_out, r)
    note("a / rho", box_gap(a_b, rho_b), r)
    low, high = (abs(z) * U for z in a_leader(r))
    foots.add(round(low, 12))
    cap = W_LEAD / 2
    note("leader foot / hole ring (cap counted)", (low - cap) - ring_out, r)
    note("leader top / a ink (cap counted)", a_b[1] - (high + cap), r)
    note("leader crosses the circle line: below its inner edge", line_in - low, r)
    note("leader crosses the circle line: above its outer edge", high - line_out, r)
    note("leader / rho box (cap counted)", rho_b[0] - cap, r)
    note("leader visible inside the circle", line_in - (low - cap), r)
    note("a nearer the line than the hole: hole gap minus line gap (px)", ((box_inmost(a_b) - ring_out) - (box_inmost(a_b) - line_out)) * PX, r)
print("\n09_1 shrink, %d radii (grid + every 60 fps frame of %.1f s and %.1f s), least values in native px:" % (len(radii), T1, T2))
for k, (val, r) in least.items():
    print("  %-62s %8.2f px at r = %.4f" % (k, val * (PX if not k.endswith("(px)") else 1), r))
check("a and rho clear the circle line by CLEAR", min(least["a / circle line (drawn gap)"][0], least["rho / circle line (drawn gap)"][0]) > CLEAR)
check("a and rho clear the dashes by CLEAR", min(least["a / dashes"][0], least["rho / dashes"][0]) > CLEAR)
check("a and rho clear the hole ring by CLEAR", min(least["a / hole ring"][0], least["rho / hole ring"][0]) > CLEAR)
check("a and rho clear each other by CLEAR", least["a / rho"][0] > CLEAR)
check("leader foot never moves", len(foots) == 1, "%d distinct" % len(foots))
check("leader clear of the hole ring (drawn)", least["leader foot / hole ring (cap counted)"][0] > 0)
check("leader stops short of a (drawn)", least["leader top / a ink (cap counted)"][0] > 0)
check("leader always crosses the circle line", min(least["leader crosses the circle line: below its inner edge"][0],
                                                   least["leader crosses the circle line: above its outer edge"][0]) > 0)
check("leader clear of rho by CLEAR", least["leader / rho box (cap counted)"][0] > CLEAR)
a_rest = (shrink_names(SHRUNK_R)[0] - A_AT) * U
print("  a at rest: %.2f px above A_AT; shrunk line vs hole ring drawn gap %.2f px"
      % (abs(a_rest) * PX, ((SHRUNK_R * U - W_SMALL / 2) - ring_out) * PX))
check("shrunk circle keeps a drawn gap from the hole ring", (SHRUNK_R * U - W_SMALL / 2) - ring_out > 0)

# the leader's look, start / hold / rest: its length inside and outside the circle
print("\nleader at the three held radii (px): inside the circle / across the line / outside it / total drawn")
for r in (SMALL_R, MID_R, SHRUNK_R):
    R = r * U
    low, high = (abs(z) * U for z in a_leader(r))
    cap = W_LEAD / 2
    inside = (R - W_SMALL / 2) - (low - cap)
    across = W_SMALL
    outside = (high + cap) - (R + W_SMALL / 2)
    a_c = shrink_names(r)[0] * U
    print("  r %.3f: %6.1f / %5.1f / %5.1f / %6.1f   (a's centre %.0f px above point a, line top %.0f px)"
          % (r, inside * PX, across * PX, outside * PX, (high - low + 2 * cap) * PX, a_c.imag * PX, (R + W_SMALL / 2) * PX))

# 09_7 and 09_9: a's name stands A_NAME_FROM_CENTRE from each centre; the disks grow from 0.001
pu = pic["unit"]
a28 = box(A_NAME_FROM_CENTRE * pu, A_NAME_HALF)
inm, outm = box_inmost(a28), box_outmost(a28)
bad_shown, near_band, worst_partial = [], [], (9.9, None)
for k in range(20001):
    r = 1e-3 + (1.5 - 1e-3) * k / 20000
    R = r * pu
    gap_out = inm - (R + W_DISK / 2)
    gap_in = (R - W_DISK / 2) - outm
    gap = max(gap_out, gap_in)       # > 0: the line is clear of the box on one side
    shown = a_name_shown(r)
    if gap <= CLEAR:
        near_band.append(r)
        if shown > 1e-12:
            bad_shown.append(r)
    if shown > 1e-12:
        worst_partial = min(worst_partial, (gap, r))
print("\n09_7 / 09_9 growths (picture unit %.3f; a@28 box %.1f x %.1f px):" % (pu, 2 * A_NAME_HALF[0] * PX, 2 * A_NAME_HALF[1] * PX))
print("  line within CLEAR of a's box for r in [%.4f, %.4f]; hidden range %s, eased %.2f either side"
      % (near_band[0], near_band[-1], A_NAME_HIDDEN, A_NAME_EASE))
print("  least gap while a shows at all: %.2f px at r = %.4f" % (worst_partial[0] * PX, worst_partial[1]))
check("a hidden wherever a growing line is within CLEAR", not bad_shown, "%d radii shown" % len(bad_shown))
ends = {"bump.radius0": 1.0, "recentre1.radius": 1.414}
check("a fully shown at 0.001 and at both growth ends (facts 1, 1.414)", all(a_name_shown(r) == 1.0 for r in (1e-3, 1.0, 1.414)))

failed = sum(1 for _, ok in results if not ok)
print("\nfile:", os.path.basename(sys.argv[1]))
print("%d checks, %d failed" % (len(results), failed))
sys.exit(1 if failed else 0)
