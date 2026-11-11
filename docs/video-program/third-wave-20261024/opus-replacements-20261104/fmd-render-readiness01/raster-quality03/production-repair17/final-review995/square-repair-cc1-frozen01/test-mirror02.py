"""Python mirror of the four new tests in candidate01-loop_sums.rs (no compile, no render).

Each assertion is copied one for one with the same thresholds; the source-binding asserts are run
against the candidate file's goursat_proof body exactly as goursat_body() slices it. Also checks the
mod.rs supplement lists for duplicates of the two new entries.

usage: python3 -I -B test-mirror02.py CANDIDATE_LOOP_SUMS_RS CANDIDATE_MOD_RS
"""
import re
import sys

SQ = complex(-2.6, -0.1)
WALK_START, KEPT_QUARTER = 1, 3
KEPT_NAME_OFFSET = complex(0.55, -0.48)
C_NAME_OFFSET = complex(0.33, -0.46)


def children(v):
    m = lambda a, b: (a + b) / 2
    if len(v) == 4:
        a, b, c, d = v
        ab, bc, cd, da, ce = m(a, b), m(b, c), m(c, d), m(d, a), m(a, c)
        return [[a, ab, ce, da], [ab, b, bc, ce], [ce, bc, c, cd], [da, ce, cd, d]]
    raise ValueError


def diameter(v):
    return max(abs(p - q) for p in v for q in v)


def perimeter(v):
    return sum(abs(v[(i + 1) % len(v)] - v[i]) for i in range(len(v)))


def area(p):
    return sum((p[i].conjugate() * p[(i + 1) % len(p)]).imag for i in range(len(p))) / 2


def pts(p, a, b):
    ab, ap = b - a, p - a
    t = min(1.0, max(0.0, (ap.real * ab.real + ap.imag * ab.imag) / (ab.real ** 2 + ab.imag ** 2)))
    return abs(p - (a + ab * t))


def gap(a, b, p, q):
    side = lambda u, w, z: (w - u).real * (z - u).imag - (w - u).imag * (z - u).real
    if side(a, b, p) * side(a, b, q) < 0 and side(p, q, a) * side(p, q, b) < 0:
        return 0.0
    return min(pts(a, p, q), pts(b, p, q), pts(p, a, b), pts(q, a, b))


def sides(p):
    return [(p[i], p[(i + 1) % len(p)]) for i in range(len(p))]


def cuts(s):
    m = lambda a, b: (a + b) / 2
    a, b, c, d = s
    return [[m(a, b), m(d, c)], [m(d, a), m(b, c)]]


def c_name_and_leader(c):
    at = c + C_NAME_OFFSET
    u = (at - c) / abs(at - c)
    return at, [c + u * 0.18, at - u * 0.12]


def drawn_nest():
    h = 2.0
    square = [SQ + complex(-h, -h), SQ + complex(h, -h), SQ + complex(h, h), SQ + complex(-h, h)]
    nest = [square, children(square)[2]]
    for i in [0, 3, 1, 2]:
        nest.append(children(nest[-1])[i])
    cur = nest[-1]
    c = (cur[0] + cur[2]) / 2
    s = perimeter(square) / perimeter(cur)
    return nest, c, [SQ + (z - c) * s for z in cur], SQ


src = open(sys.argv[1], encoding="utf-8").read()
body = src.split("fn goursat_proof(")[1].split("pub struct LoopSums")[0]
results = []


def check(name, ok, detail=""):
    results.append((name, bool(ok)))
    print(("PASS " if ok else "FAIL ") + name + (("  " + detail) if detail else ""))


# test 1: the_kept_square_is_a_bisection_quarter_that_holds_c
_, _, enl, centre = drawn_nest()
near = lambda a, b: abs(a - b) < 1e-12
qs = children(enl)
kept = qs[KEPT_QUARTER]
check("every quarter has c as a corner", all(any(near(z, centre) for z in q) for q in qs))
check("no quarter concentric", all(abs((q[0] + q[2]) / 2 - centre) > 0.5 for q in qs),
      "min centre offset %.3f" % min(abs((q[0] + q[2]) / 2 - centre) for q in qs))
check("kept shares a corner with Q_n", any(near(z, w) for z in kept for w in enl))
check("kept halves diameter", abs(diameter(kept) * 2 - diameter(enl)) < 1e-12)
check("kept halves perimeter", abs(perimeter(kept) * 2 - perimeter(enl)) < 1e-12)
d, k = enl[2] - enl[0], kept[2] - kept[0]
check("kept diagonal is its diameter", abs(abs(k) - diameter(kept)) < 1e-12)
check("kept diagonal parallel to D_n", abs(d.real * k.imag - d.imag * k.real) < 1e-12)
check("kept diagonal apart from D_n line", gap(enl[0], enl[2], kept[0], kept[2]) > 1.0,
      "%.3f" % gap(enl[0], enl[2], kept[0], kept[2]))
for a, b in cuts(enl):
    check("cut through c", pts(centre, a, b) < 1e-12)
    for u, w in [(a, centre), (centre, b)]:
        owners = sum(1 for q in qs if any((near(s, u) and near(t, w)) or (near(s, w) and near(t, u)) for s, t in sides(q)))
        check("cut half shared by 2 quarters", owners == 2, str(owners))
inside = lambda p: kept[0].real < p.real < kept[2].real and kept[0].imag < p.imag < kept[2].imag
kept_name = kept[3] + KEPT_NAME_OFFSET
c_name, d_name = centre + complex(-0.22, 0.28), complex(-1.7, 1.2)
check("names inside/outside kept", inside(kept_name) and inside(c_name) and not inside(d_name))
strokes = sides(enl) + [tuple(x) for x in cuts(enl)] + [(enl[0], enl[2]), (kept[0], kept[2])]
for p, r in [(kept_name, 0.28), (c_name, 0.085), (d_name, 0.17)]:
    m = min(pts(p, a, b) for a, b in strokes)
    check("name %s clear by > %.3f" % (p, r), m > r, "min %.3f" % m)
check("body binds proof_children(&enlarged)", "let quarters = proof_children(&enlarged);" in body)
check("body binds KEPT_QUARTER", "let kept = &quarters[KEPT_QUARTER];" in body)
check("old concentric map gone", "(*z - centre).scale(0.5)" not in body)

# test 2: the_walk_rests_off_the_diagonal_and_stays_counterclockwise
walk = [enl[(WALK_START + i) % 4] for i in range(4)]
check("walk ccw", area(walk) > 0, "%.3f" % area(walk))
start = walk[0]
check("start off the diagonal", pts(start, enl[0], enl[2]) > 1.0, "%.3f" % pts(start, enl[0], enl[2]))
check("|start-c| = D/2", abs(abs(start - centre) * 2 - diameter(enl)) < 1e-12)
check("D_n name clear of resting segment", pts(complex(-1.7, 1.2), centre, start) > 1.0,
      "%.3f" % pts(complex(-1.7, 1.2), centre, start))
for s in ["enlarged[(WALK_START + k) % 4]", "let moving = walk_corners;", "&[centre, walk_corners[0]]"]:
    check("body has " + s, s in body)

# test 3: the_name_of_c_is_near_its_dot_and_clear_of_the_nest
nest, c, _, _ = drawn_nest()
edges = [e for q in nest for e in sides(q)]
delta = 1.8 * diameter(nest[-1])
name, (frm, to) = c_name_and_leader(c)
glyph = 0.085
away = abs(name - c)
check("0.4 <= away < 0.7", 0.4 <= away < 0.7, "%.3f (%.0f px)" % (away, away * 67.5))
m = min(pts(name, a, b) for a, b in edges)
check("name clear of nest edges > glyph+0.06", m > glyph + 0.06, "min %.3f" % m)
check("name clear of delta circle", away - glyph > delta + 0.1, "%.3f vs %.3f" % (away - glyph, delta + 0.1))
check("leader starts past rim, ends short of glyph", abs(frm - c) > 0.15 and abs(to - name) > glyph)
lg = min(gap(frm, to, a, b) for a, b in edges)
check("leader meets no nest edge (> 0.02)", lg > 0.02, "min %.3f (%.1f px)" % (lg, lg * 67.5))
for s in ["let (name_at, leader) = c_name_and_leader(c);", "&[point, point_leader, point_name]",
          "&[point, point_leader, point_name, neighbourhood, small]"]:
    check("body has " + s, s in body)
crossing = abs(frm - c) < delta < abs(to - c)
print("info: leader crosses the delta-circle (drawn later, dim leader over green ring):", crossing)

# test 4: the_repaired_square_picture_leaves_before_the_triangle
st = body.find("let mut square_picture = vec![")
cl = st + body[st:].find("fade_out_all(stage, &square_picture, 0.4)?;")
check("square picture cleared before triangle", st >= 0 and cl < body.find("let triangle = proof_triangle();"))
listed = body[st:cl]
for piece in ["perimeter_word,", "perimeter_name,", "square_picture.extend(quarter_picture);"]:
    check("listed " + piece, piece in listed)

# existing phrase-binding test must still hold: no new wait_for, count unchanged
print("info: line.wait_for( calls in body:", body.count("line.wait_for("))

# mod.rs supplements: no duplicate of the new entries
mod = open(sys.argv[2], encoding="utf-8").read()
text_block = mod.split("pub const TEXT: &[&str] = &[")[1].split("];")[0]
tex_block = mod.split("pub const TEX: &[&str] = &[")[1].split("];")[0]
check("TEXT has PERIMETER_WORD once", text_block.count("loop_sums::PERIMETER_WORD") == 1 and '"perimeter"' not in text_block)
check("TEX has KEPT_TEX once", tex_block.count("loop_sums::KEPT_TEX") == 1 and "Q_{n+1}" not in tex_block)
print("\n%d checks, %d failed" % (len(results), sum(1 for _, ok in results if not ok)))
