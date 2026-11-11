"""CC1's independent mirror of CC0's square candidate02 delta (read-only; no compile, no render).

Rebuilds the 08_5 nest from the source rules (proof_square, proof_children, Q_1 = child 2, then 0, 3, 1, 2), the L5
leader (C_NAME_OFFSET, LEADER_START, 0.12 short of the glyph), and checks, with the TRUE stroke widths (Q_0 and Q_1 5,
Q_2..Q_5 3, leader 1.5, dot radius 0.07, round caps half the width; width w is w/100 scene units):
  * where along the leader its ink overlaps nest ink, and which sides are crossed or touched;
  * the test's own classification (reach = 5*0.005 + cap, margin 0.02) and its expected touched set;
  * how much of Q_5 shows outside the dot (Q_5's half side is smaller than the dot's radius);
  * the delta-circle crossing; and, in the magnified picture, where the redrawn c name sits.
Then binds the candidate02 test's source strings against the candidate file itself (goursat body counts/order).

usage: python3 -I -B square-geometry01.py CANDIDATE02_LOOP_SUMS_RS
"""
import math
import sys

PX4K, PX960 = 270.0, 67.5


def px(u):
    return f"{u:.4f} u = {u * PX960:.1f} px@960 = {u * PX4K:.1f} px@4K"


def children(q):
    a, b, c, d = q
    mid = lambda p, r: ((p[0] + r[0]) / 2, (p[1] + r[1]) / 2)
    ab, bc, cd, da, ce = mid(a, b), mid(b, c), mid(c, d), mid(d, a), mid(a, c)
    return [[a, ab, ce, da], [ab, b, bc, ce], [ce, bc, c, cd], [da, ce, cd, d]]


def sub(p, q):
    return (p[0] - q[0], p[1] - q[1])


def norm(p):
    return math.hypot(*p)


def pt_seg(p, a, b):
    ab, ap = sub(b, a), sub(p, a)
    t = max(0.0, min(1.0, (ap[0] * ab[0] + ap[1] * ab[1]) / (ab[0] ** 2 + ab[1] ** 2)))
    return norm(sub(p, (a[0] + t * ab[0], a[1] + t * ab[1])))


def cross(u, w, z):
    return (w[0] - u[0]) * (z[1] - u[1]) - (w[1] - u[1]) * (z[0] - u[0])


def seg_gap(a, b, p, q):
    if cross(a, b, p) * cross(a, b, q) < 0 and cross(p, q, a) * cross(p, q, b) < 0:
        return 0.0
    return min(pt_seg(a, p, q), pt_seg(b, p, q), pt_seg(p, a, b), pt_seg(q, a, b))


def sides(q):
    return [(q[i], q[(i + 1) % len(q)]) for i in range(len(q))]


# the nest, as goursat_proof draws it
cx, cy, h = -2.6, -0.1, 2.0
Q0 = [(cx - h, cy - h), (cx + h, cy - h), (cx + h, cy + h), (cx - h, cy + h)]
nest = [Q0, children(Q0)[2]]
for idx in (0, 3, 1, 2):
    nest.append(children(nest[-1])[idx])
width = [5.0, 5.0, 3.0, 3.0, 3.0, 3.0]
last = nest[-1]
c = ((last[0][0] + last[2][0]) / 2, (last[0][1] + last[2][1]) / 2)
print("nest (bl, tr):")
for k, q in enumerate(nest):
    print(f"  Q_{k}: {q[0]} .. {q[2]}  stroke {width[k]}")
print(f"c = {c}")

# the L5 leader
off, start_s, short = (0.33, -0.46), 0.075, 0.12
name = (c[0] + off[0], c[1] + off[1])
away = norm(sub(name, c))
u = (off[0] / away, off[1] / away)
at = lambda s: (c[0] + u[0] * s, c[1] + u[1] * s)
end_s = away - short
frm, to = at(start_s), at(end_s)
lead_half, dot_r = 1.5 / 200, 0.07
print(f"\nL5 leader: s {start_s}..{end_s:.4f} along u = ({u[0]:.4f}, {u[1]:.4f}); name {away:.4f} from c")
print(f"  start cap reaches {start_s - lead_half:.4f} from c: {px(dot_r - (start_s - lead_half))} inside the dot's rim")

# ink overlaps along the leader, true widths
print("  sides whose ink the leader's ink meets (true widths), with the s-span of the overlap:")
overlap_hi = 0.0
for k, q in enumerate(nest):
    for i, (a, b) in enumerate(sides(q)):
        reach = width[k] / 200 + lead_half
        g = seg_gap(frm, to, a, b)
        span = [s / 1e4 for s in range(int(start_s * 1e4), int(end_s * 1e4) + 1) if pt_seg(at(s / 1e4), a, b) < reach]
        if span:
            overlap_hi = max(overlap_hi, span[-1])
            kind = "CROSSES (centre lines meet)" if g == 0.0 else f"touches (centre-line gap {g:.4f})"
            print(f"    Q_{k} side {i} ({'bottom right top left'.split()[i]}): {kind}; ink overlap s {span[0]:.4f}..{span[-1]:.4f}")
print(f"  the leader's ink is on nest ink from the rim to s = {overlap_hi:.4f}: {px(overlap_hi - dot_r)} past the rim; "
      f"clear for the remaining {px(end_s - overlap_hi)}")

# the test's classification
reach_t = 5.0 * 0.005 + lead_half
touched, near = [], []
least_other = (9.0, None)
for k, q in enumerate(nest):
    for i, (a, b) in enumerate(sides(q)):
        g = seg_gap(frm, to, a, b)
        if g < reach_t:
            touched.append((k, i))
        else:
            if g <= reach_t + 0.02:
                near.append((k, i, g))
            if g < least_other[0]:
                least_other = (g, (k, i))
print(f"  test classification (reach {reach_t}): touched {touched}; near misses {near}; "
      f"least untouched gap {least_other[0]:.4f} at Q_{least_other[1][0]} side {least_other[1][1]} "
      f"(margin over reach+0.02: {least_other[0] - reach_t - 0.02:+.4f})")
print(f"  test expects [(3, 1), (4, 1), (5, 0), (5, 1)]: {touched == [(3, 1), (4, 1), (5, 0), (5, 1)]}")
clear_from = at(0.25)
print(f"  from s 0.25 every side clear by reach+0.02: "
      f"{all(seg_gap(clear_from, to, a, b) > reach_t + 0.02 for q in nest for a, b in sides(q))}")
delta = 1.8 * max(norm(sub(p, r)) for p in last for r in last)
print(f"  delta-circle radius {delta:.4f}: crosses the leader once at s = {delta:.4f} "
      f"({start_s < delta < end_s})")

# Q_5 against the dot
half5 = (last[2][0] - last[0][0]) / 2
w5 = width[5] / 200
corner = math.hypot(half5, half5)
t_out = math.sqrt(dot_r ** 2 - half5 ** 2) if dot_r > half5 else 0.0
print(f"\nQ_5: half side {half5:.4f} < dot radius {dot_r}: each side's centre line is inside the dot for |t| < {t_out:.4f};"
      f" its ink shows only as corner stubs {half5 - t_out:.4f} long (plus the stroke's half width) on each arm;"
      f" corners stand {corner:.4f} from c, so their ink reaches {px(corner + w5 - dot_r)} beyond the rim")
print(f"  Q_5's right side lies on x = {last[1][0]:.4f}, the same line as Q_3's and Q_4's right sides "
      f"(x = {nest[3][1][0]:.4f}, {nest[4][1][0]:.4f})")

# L12: the redrawn c name in the magnified picture
scale = (4 * 4.0) / (4 * (last[1][0] - last[0][0]))
centre = (cx, cy)
enl = [(centre[0] + (p[0] - c[0]) * scale, centre[1] + (p[1] - c[1]) * scale) for p in last]
kept = children(enl)[3]
cname = (centre[0] - 0.22, centre[1] + 0.28)
inside = kept[0][0] < cname[0] < kept[2][0] and kept[0][1] < cname[1] < kept[2][1]
print(f"\nL12: magnified Q_n {enl[0]}..{enl[2]}; kept quarter {kept[0]}..{kept[2]}; c name at {cname}, "
      f"inside the kept fill: {inside}")
glyph_half = 0.06  # 'c' at 26: about 0.12 x 0.12 (estimate)
strokes = [(enl[i], enl[(i + 1) % 4], 5.0) for i in range(4)] + [(kept[i], kept[(i + 1) % 4], 5.0) for i in range(4)]
mid = lambda p, r: ((p[0] + r[0]) / 2, (p[1] + r[1]) / 2)
strokes += [(mid(enl[0], enl[1]), mid(enl[3], enl[2]), 2.0), (mid(enl[3], enl[0]), mid(enl[1], enl[2]), 2.0)]
strokes += [(enl[0], enl[2], 3.0), (kept[0], kept[2], 4.0)]
least = min(pt_seg(cname, a, b) - w / 200 - glyph_half for a, b, w in strokes)
print(f"  c name glyph-box (estimated half {glyph_half}) ink clearance to the nearest L12 stroke: {px(least)}")

# source bindings of the changed tests, against the candidate file
src = open(sys.argv[1], encoding="utf-8").read()
body = src.split("fn goursat_proof(")[1].split("pub struct LoopSums")[0]
redraw = 'kit.tex(stage, "c", 26.0, output(), &[], centre_name_at)?'
fill = body.find("proof_fill(stage, kept, hero()),")
shown = body.find("fade_in_all(stage, &quarter_picture, 0.5)?;")
print("\nsource bindings (candidate file):")
print(f"  goursat body ends before the tests: {'mod tests' not in body}")
print(f"  centre_name_at defined: {'let centre_name_at = v(centre.re - 0.22, centre.im + 0.28);' in body}")
print(f"  redraw occurrences in body: {body.count(redraw)} (test wants 2)")
print(f"  fill < redraw < fade-in: {0 <= fill < body.find(redraw, fill) < shown}")
print(f"  dot redraw between fill and fade-in: {'dot(stage, v(centre.re, centre.im), 0.07, output()),' in body[fill:shown]}")
kept_vec = body[body.find("quarter_picture.extend(["):shown]
order = [kept_vec.find(s) for s in ("proof_fill(stage, kept", "proof_path(stage, kept", "dot(stage", redraw, "KEPT_TEX")]
print(f"  draw order inside quarter_picture (fill, outline, dot, c name, Q_(n+1) name): {order == sorted(order)}")
print(f"  later film uses of centre_name after L12: {body[shown:].count('centre_name')} (the square_picture list only)")
