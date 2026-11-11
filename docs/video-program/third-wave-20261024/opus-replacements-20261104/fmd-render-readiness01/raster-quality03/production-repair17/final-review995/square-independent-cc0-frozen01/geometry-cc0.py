"""geometry-cc0.py: independent CC0 geometry for the CC1 08_5 candidate (no compile, no render).

Recomputes the drawn nest, c, the L5 name and leader, the L12 quarter picture and the L11
"perimeter P_n" pair from the constants in candidate01-loop_sums.rs, and converts to pixels.
Stroke widths use fmn's width x 0.01 units; glyph boxes are CM metric estimates (labelled).
"""
import math

PX960 = 960 / (8 * 16 / 9)   # 67.5 px per unit at 960 x 540
PX4K = PX960 * 4

def mid(a, b):
    return ((a[0] + b[0]) / 2, (a[1] + b[1]) / 2)

def children(q):
    a, b, c, d = q
    ab, bc, cd, da, ce = mid(a, b), mid(b, c), mid(c, d), mid(d, a), mid(a, c)
    return [[a, ab, ce, da], [ab, b, bc, ce], [ce, bc, c, cd], [da, ce, cd, d]]

def sides(q):
    return [(q[i], q[(i + 1) % len(q)]) for i in range(len(q))]

def pseg(p, a, b):
    ab = (b[0] - a[0], b[1] - a[1]); ap = (p[0] - a[0], p[1] - a[1])
    t = max(0.0, min(1.0, (ap[0] * ab[0] + ap[1] * ab[1]) / (ab[0] ** 2 + ab[1] ** 2)))
    return math.hypot(p[0] - a[0] - t * ab[0], p[1] - a[1] - t * ab[1])

def box_seg(cx, cy, hw, hh, a, b, n=40):
    """Smallest distance from a glyph box (centre, half sizes) to a segment, by sampling the box edge."""
    pts = []
    for i in range(n + 1):
        f = i / n
        pts += [(cx - hw + 2 * hw * f, cy - hh), (cx - hw + 2 * hw * f, cy + hh),
                (cx - hw, cy - hh + 2 * hh * f), (cx + hw, cy - hh + 2 * hh * f)]
    return min(pseg(p, a, b) for p in pts)

def px(u):
    return f"{u:.3f} u = {u * PX960:.1f} px@960 = {u * PX4K:.1f} px@4K"

# the drawn nest, as goursat_proof builds it
SQ = [(-4.6, -2.1), (-0.6, -2.1), (-0.6, 1.9), (-4.6, 1.9)]
nest = [SQ, children(SQ)[2]]
widths = [5.0, 5.0]
for k in [0, 3, 1, 2]:
    nest.append(children(nest[-1])[k]); widths.append(3.0)
cur = nest[-1]
c = mid(cur[0], cur[2])
side5 = cur[1][0] - cur[0][0]
diam5 = math.hypot(side5, side5)
delta = 1.8 * diam5
print("Q_k boxes:")
for k, q in enumerate(nest):
    print(f"  Q_{k}: x {q[0][0]:.4f}..{q[1][0]:.4f}  y {q[0][1]:.4f}..{q[2][1]:.4f}  stroke {widths[k]}")
print(f"c = ({c[0]:.4f}, {c[1]:.4f}); Q_5 side {side5:.4f} = {side5 * PX960:.1f} px@960; delta {delta:.4f}")
print(f"dot radius 0.07 vs c to Q_5 edge {side5 / 2:.4f}: the dot covers Q_5's edges (pre-existing)")

# L5: the name and leader
off = (0.33, -0.46)
name = (c[0] + off[0], c[1] + off[1])
dist = math.hypot(*off)
u = (off[0] / dist, off[1] / dist)
lead0 = (c[0] + 0.18 * u[0], c[1] + 0.18 * u[1])
lead1 = (name[0] - 0.12 * u[0], name[1] - 0.12 * u[1])
print(f"\nL5 name at ({name[0]:.4f}, {name[1]:.4f}), {px(dist)} from c")
edges = [(s, widths[k]) for k, q in enumerate(nest) for s in sides(q)]
gh = 0.0585   # half of the 'c' glyph box at 27 pt (cmmi c 0.433 x 0.431 em x 0.27), estimate
ink_edge = min(box_seg(name[0], name[1], gh, gh, a, b) - w * 0.005 for (a, b), w in edges)
print(f"  name glyph box to nearest nest edge ink: {px(ink_edge)}")
near_c = min(math.hypot(name[0] + sx * gh - c[0], name[1] + sy * gh - c[1]) for sx in (-1, 1) for sy in (-1, 1))
print(f"  name glyph box to delta-circle ink: {px(near_c - delta - 0.015)}")
# where the ray from c crosses the shared x = -2.1 edge (right edges of Q_3, Q_4, Q_5)
s_cross = (-2.1 - c[0]) / u[0]
y_cross = c[1] + s_cross * u[1]
print(f"  ray from c crosses x = -2.1 at s = {s_cross:.4f} (y = {y_cross:.4f}); dot rim at s = 0.07; leader starts at s = 0.18")
print(f"  gap dot rim -> leader start along the ray: {px(0.18 - 0.07)}; the x = -2.1 edge lies inside that gap")
lead_edge = min(min(pseg(lead0, a, b), pseg(lead1, a, b), min(pseg(p, lead0, lead1) for p in (a, b))) for (a, b), w in edges)
print(f"  leader centre line to nearest nest edge centre line: {px(lead_edge)}")
print(f"  ... minus stroke half-widths (edge 3 -> 0.015, leader 1.5 -> 0.0075): ink gap {px(lead_edge - 0.0225)}")
print(f"  test threshold 0.02 u is below the half-width sum 0.0225 u: a touching leader would still pass")
s_circle = delta
print(f"  leader crosses the delta-circle at s = {s_circle:.4f} (leader spans s 0.18..{dist - 0.12:.4f})")

# L12: the magnified quarter picture
cen = (-2.6, -0.1)
EN = SQ   # zoom maps Q_5 onto the outer square exactly (side 4 about SQUARE_CENTRE)
quarters = children(EN)
kept = quarters[3]
print(f"\nL12 kept quarter (index 3): {kept}")
print(f"  shares corner {kept[3]} with Q_n; c = {cen} is its corner {kept[1]}")
dn = (EN[0], EN[2]); kd = (kept[0], kept[2])
lines = [(EN[i], EN[(i + 1) % 4], 5.0) for i in range(4)]
cuts = [(mid(EN[0], EN[1]), mid(EN[3], EN[2]), 2.0), (mid(EN[3], EN[0]), mid(EN[1], EN[2]), 2.0)]
kept_edges = [(a, b, 5.0) for a, b in sides(kept)]
all_strokes = lines + cuts + kept_edges + [(dn[0], dn[1], 3.0), (kd[0], kd[1], 4.0)]
labels = {
    "c (26 pt, est 0.113 x 0.112)": ((cen[0] - 0.22, cen[1] + 0.28), 0.0565, 0.056),
    "D_n (25 pt, est 0.31 x 0.21)": ((-1.7, 1.2), 0.155, 0.105),
    "Q_{n+1} (24 pt, est 0.50 x 0.22)": ((kept[3][0] + 0.55, kept[3][1] - 0.48), 0.25, 0.11),
}
for nm, ((x, y), hw, hh) in labels.items():
    g = min(box_seg(x, y, hw, hh, a, b) - w * 0.005 for a, b, w in all_strokes)
    print(f"  {nm} at ({x:.2f}, {y:.2f}): glyph box to nearest stroke ink {px(g)}")
print(f"  kept diagonal parallel to D_n, 1.414 u apart; kept D and P exactly half of Q_n's")

# L11: "perimeter P_n" (estimated CM metrics; the scene measures the real widths)
word_w = 4.172 * 20 * 0.01           # cmr10 p e r i m e t e r
name_w = (0.642 + 0.600 * 0.7) * 26 * 0.01
total = word_w + 0.14 + name_w
left = -2.6 - total / 2
print(f"\nL11 pair: word {word_w:.3f} u, P_n {name_w:.3f} u, total {total:.3f} u, x {left:.3f}..{left + total:.3f}")
word_mid = (0.669 - 0.194) / 2 * 20 * 0.01     # box centre above baseline: 'i' dot top, 'p' descender
name_mid = (0.683 - 0.150) / 2 * 26 * 0.01     # P top, subscript n bottom
print(f"  centre-aligned at y = -2.5: P_n baseline sits {px(name_mid - word_mid)} below the word's baseline (estimate)")
top = -2.5 + (0.669 * 20 * 0.01 - word_mid)
print(f"  pair top {top:.3f}; square bottom edge ink at {-2.1 - 0.025:.3f}: gap {px(-2.1 - 0.025 - top)}")
# resting walk segment vs D_n name
rest = (cen, EN[1])
print(f"\nL11 resting |z - c| segment c -> bottom-right corner, length {math.dist(*rest):.4f} = D_n/2 ({math.dist(EN[0], EN[2]) / 2:.4f})")
print(f"  D_n name box to resting segment: {px(box_seg(-1.7, 1.2, 0.155, 0.105, *rest) - 0.025)}")
