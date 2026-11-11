"""Geometry mirror for the proposed 08_5 repairs (L5, L11, L12) against build268 loop_sums.rs 8a13e8a1.

Read-only, no render. Mirrors proof_square / proof_children / the nested index path [0, 3, 1, 2] /
c / delta / the zoom of goursat_proof exactly, then measures clearances (scene units; 67.5 tile px
per unit at 960x540) for candidate label positions. Glyph boxes are ESTIMATES (CM ems x size x 0.01,
calibrated on crops s0357-slot4 and s0395-slot1: 'c' at 27 pt is about 7 tile px = 0.105 units).

usage: python3 -I -B geometry-check01.py
"""
import cmath
import math

PX = 67.5


def mid(a, b):
    return (a + b) / 2


def children(v):
    if len(v) == 4:
        a, b, c, d = v
        ab, bc, cd, da = mid(a, b), mid(b, c), mid(c, d), mid(d, a)
        centre = mid(a, c)
        return [[a, ab, centre, da], [ab, b, bc, centre], [centre, bc, c, cd], [da, centre, cd, d]]
    a, b, c = v
    ab, bc, ca = mid(a, b), mid(b, c), mid(c, a)
    return [[a, ab, ca], [ab, b, bc], [ca, bc, c], [ab, bc, ca]]


def diameter(v):
    return max(abs(p - q) for p in v for q in v)


def perimeter(v):
    return sum(abs(v[(i + 1) % len(v)] - v[i]) for i in range(len(v)))


def edges(v):
    return [(v[i], v[(i + 1) % len(v)]) for i in range(len(v))]


def seg_point(p, a, b):
    ab = b - a
    t = 0.0 if abs(ab) == 0 else max(0.0, min(1.0, ((p - a) * ab.conjugate()).real / abs(ab) ** 2))
    return abs(p - (a + t * ab))


def seg_seg(a, b, c, d):
    def ccw(p, q, r):
        return ((q - p) * (r - p).conjugate()).imag
    if ccw(a, b, c) * ccw(a, b, d) < 0 and ccw(c, d, a) * ccw(c, d, b) < 0:
        return 0.0
    return min(seg_point(a, c, d), seg_point(b, c, d), seg_point(c, a, b), seg_point(d, a, b))


def box_seg(centre, half, a, b):
    """Distance from an axis-aligned box (centre, half=(hx,hy)) to a segment (0 if they meet)."""
    hx, hy = half
    corners = [centre + complex(sx * hx, sy * hy) for sx, sy in ((-1, -1), (1, -1), (1, 1), (-1, 1))]
    inside = lambda p: abs(p.real - centre.real) <= hx and abs(p.imag - centre.imag) <= hy
    if inside(a) or inside(b):
        return 0.0
    return min(seg_seg(corners[i], corners[(i + 1) % 4], a, b) for i in range(4))


def box_circle(centre, half, c0, r):
    hx, hy = half
    dx = max(abs(centre.real - c0.real) - hx, 0.0)
    dy = max(abs(centre.imag - c0.imag) - hy, 0.0)
    near = math.hypot(dx, dy)
    far = max(math.hypot(centre.real - c0.real + sx * hx, centre.imag - c0.imag + sy * hy)
              for sx in (-1, 1) for sy in (-1, 1))
    if near >= r:
        return near - r
    if far <= r:
        return r - far
    return 0.0


# ---- the nest, exactly as goursat_proof builds it ----
SQ_C, SIDE = complex(-2.6, -0.1), 4.0
h = SIDE / 2
square = [SQ_C + complex(-h, -h), SQ_C + complex(h, -h), SQ_C + complex(h, h), SQ_C + complex(-h, h)]
q1 = children(square)[2]
nest = [square, q1]
current = q1
for index in [0, 3, 1, 2]:
    current = children(current)[index]
    nest.append(current)
c = mid(current[0], current[2])
delta = diameter(current) * 1.8
print("c =", c, " delta =", round(delta, 4), " Q5 side =", abs(current[1] - current[0]))
old_label = c + complex(0.95, 0.12)
print("old c label", old_label, "distance", round(abs(old_label - c), 3), "=", round(abs(old_label - c) * PX), "px")

nest_edges = [e for sq in nest for e in edges(sq)]
C_HALF = (0.06, 0.06)
DOT_R = 0.07


def label_report(p):
    e = min(box_seg(p, C_HALF, a, b) for a, b in nest_edges)
    circ = box_circle(p, C_HALF, c, delta)
    dotc = box_circle(p, C_HALF, c, DOT_R)
    return e, circ, dotc


e, circ, dotc = label_report(old_label)
print("old label clearances: edges %.3f circle %.3f dot-dist %.3f" % (e, circ, abs(old_label - c)))
# edges crossed by the straight path from c to the old label
crossed = sum(1 for a, b in nest_edges if seg_seg(c, old_label, a, b) == 0.0)
print("old label: nest edges met by the straight dot-label path:", crossed)

# ---- L5 search: label outside the delta circle, near, with a leader that meets no nest edge ----
best = []
for i in range(-60, 61):
    for j in range(-60, 61):
        p = c + complex(i * 0.01, j * 0.01)
        d = abs(p - c)
        if not (0.4 <= d <= 0.75):
            continue
        e, circ, dotc = label_report(p)
        if e < 0.10 or circ < 0.12:
            continue
        u = (p - c) / d
        # leader from just past the dot rim to just short of the glyph
        for s0 in (0.12, 0.14, 0.16):
            a = c + u * s0
            b = p - u * 0.12
            le = min(seg_seg(a, b, x, y) for x, y in nest_edges)
            if le < 0.02:
                continue
            best.append((min(e, circ), d, p, s0, le, e, circ))
            break
best.sort(key=lambda r: (-round(r[0], 2), r[1]))
print("\nL5 candidates (min(edge, circle) clearance desc, then distance):")
for r in best[:8]:
    m, d, p, s0, le, e, circ = r
    print("  label (%.3f, %.3f) off c (%+.3f, %+.3f) dist %.3f = %d px | edge %.3f (%d px) circle %.3f (%d px) | leader s0 %.2f, leader-edge %.3f"
          % (p.real, p.imag, p.real - c.real, p.imag - c.imag, d, round(d * PX), e, round(e * PX), circ, round(circ * PX), s0, le))

# ---- the magnified view (zoom of current about c to SQUARE_CENTRE) ----
zoom_scale = perimeter(square) / perimeter(current)
zoom = lambda z: SQ_C + (z - c) * zoom_scale
enl = [zoom(z) for z in current]
centre = zoom(c)
print("\nenlarged", [(round(z.real, 3), round(z.imag, 3)) for z in enl], "centre", centre)
quarters = children(enl)
for k, q in enumerate(quarters):
    has_c = any(abs(z - centre) < 1e-12 for z in q)
    print("quarter %d: corners %s  c is a corner: %s  diam %.4f (parent %.4f)  perim %.4f (parent %.4f)"
          % (k, [(round(z.real, 2), round(z.imag, 2)) for z in q], has_c, diameter(q), diameter(enl), perimeter(q), perimeter(enl)))
old_half = [centre + (z - centre) * 0.5 for z in enl]
print("old concentric half-square equals a quarter?", any(all(abs(a - b) < 1e-9 for a, b in zip(old_half, q)) for q in quarters))

K = 3  # proposed kept quarter: top-left
kept = quarters[K]
kept_diag = (kept[0], kept[2])
diag = (enl[0], enl[2])
cuts = [(mid(enl[0], enl[1]), mid(enl[3], enl[2])), (mid(enl[0], enl[3]), mid(enl[1], enl[2]))]
# the cut halves are exactly the quarters' shared (interior) edges
shared = set()
for q in quarters:
    for a, b in edges(q):
        on_outer = any(seg_point(a, x, y) < 1e-9 and seg_point(b, x, y) < 1e-9 for x, y in edges(enl))
        if not on_outer:
            shared.add(tuple(sorted([(round(a.real, 9), round(a.imag, 9)), (round(b.real, 9), round(b.imag, 9))])))
print("interior quarter edges:", len(shared), "(expect 4 = two cuts, each split at c)")
print("kept diagonal parallel to D_n line:", abs(((kept_diag[1] - kept_diag[0]) / (diag[1] - diag[0])).imag) < 1e-12,
      " separation %.3f, length ratio %.3f" % (seg_seg(*kept_diag, *diag), abs(kept_diag[1] - kept_diag[0]) / abs(diag[1] - diag[0])))

strokes_two_halves = edges(enl) + cuts + [kept_diag, diag] + edges(kept)
labels = {
    "c (unchanged, -0.22,+0.28)": (centre + complex(-0.22, 0.28), (0.06, 0.06)),
    "D_n (unchanged, -1.7,1.2)": (complex(-1.7, 1.2), (0.15, 0.12)),
    "Q_{n+1} proposed": (complex(-4.05, 1.42), (0.26, 0.13)),
    "perimeter P_n pair (y -2.5)": (complex(-2.6, -2.5), (0.62, 0.11)),
}
print("\n'Two halves' picture label clearances (min over outline, cuts, both diagonals, kept outline):")
for name, (p, half) in labels.items():
    m = min(box_seg(p, half, a, b) for a, b in strokes_two_halves)
    dotd = box_circle(p, half, centre, DOT_R)
    inside_kept = kept[0].real <= p.real <= kept[2].real and kept[0].imag <= p.imag <= kept[2].imag
    print("  %-30s clearance %.3f (%d px)  dot %.3f  inside kept quarter: %s" % (name, m, round(m * PX), dotd, inside_kept))

# ---- L11: the resting |z - c| segment from the bottom-right corner instead of the bottom-left ----
for start in (0, 1):
    seg = (centre, enl[start])
    overlap = seg_point(enl[start], *diag) < 1e-9
    print("\nwalk start corner %d: |z-c| segment on the D_n diagonal: %s; length %.3f = %.2f D_n"
          % (start, overlap, abs(enl[start] - centre), abs(enl[start] - centre) / diameter(enl)))
    for name in ("c (unchanged, -0.22,+0.28)", "D_n (unchanged, -1.7,1.2)", "perimeter P_n pair (y -2.5)"):
        p, half = labels[name]
        print("   %-30s clearance to resting segment %.3f (%d px)" % (name, box_seg(p, half, *seg), round(box_seg(p, half, *seg) * PX)))
# the walk stays counterclockwise from either start
for start in (0, 1):
    order = [enl[(start + k) % 4] for k in range(4)]
    area = sum((order[i].conjugate() * order[(i + 1) % 4]).imag for i in range(4)) / 2
    print("start %d: oriented area %.3f (positive = counterclockwise)" % (start, area))
