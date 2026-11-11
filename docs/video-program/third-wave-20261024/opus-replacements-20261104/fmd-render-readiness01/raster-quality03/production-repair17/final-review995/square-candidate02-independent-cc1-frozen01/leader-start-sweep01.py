"""Does candidate02's L5 test still hold for nearby LEADER_START values? (CC1; reuses square-geometry01's helpers
by import-free copy: the nest, leader and test rules as in the candidate test.)

usage: python3 -I -B leader-start-sweep01.py
"""
import math


def children(q):
    a, b, c, d = q
    mid = lambda p, r: ((p[0] + r[0]) / 2, (p[1] + r[1]) / 2)
    ab, bc, cd, da, ce = mid(a, b), mid(b, c), mid(c, d), mid(d, a), mid(a, c)
    return [[a, ab, ce, da], [ab, b, bc, ce], [ce, bc, c, cd], [da, ce, cd, d]]


def pt_seg(p, a, b):
    ab, ap = (b[0] - a[0], b[1] - a[1]), (p[0] - a[0], p[1] - a[1])
    t = max(0.0, min(1.0, (ap[0] * ab[0] + ap[1] * ab[1]) / (ab[0] ** 2 + ab[1] ** 2)))
    return math.hypot(p[0] - a[0] - t * ab[0], p[1] - a[1] - t * ab[1])


def cross(u, w, z):
    return (w[0] - u[0]) * (z[1] - u[1]) - (w[1] - u[1]) * (z[0] - u[0])


def seg_gap(a, b, p, q):
    if cross(a, b, p) * cross(a, b, q) < 0 and cross(p, q, a) * cross(p, q, b) < 0:
        return 0.0
    return min(pt_seg(a, p, q), pt_seg(b, p, q), pt_seg(p, a, b), pt_seg(q, a, b))


Q0 = [(-4.6, -2.1), (-0.6, -2.1), (-0.6, 1.9), (-4.6, 1.9)]
nest = [Q0, children(Q0)[2]]
for idx in (0, 3, 1, 2):
    nest.append(children(nest[-1])[idx])
last = nest[-1]
c = ((last[0][0] + last[2][0]) / 2, (last[0][1] + last[2][1]) / 2)
off = (0.33, -0.46)
away = math.hypot(*off)
u = (off[0] / away, off[1] / away)
reach, cap = 5 * 0.005 + 1.5 * 0.005, 1.5 * 0.005
for start in (0.0705, 0.072, 0.075, 0.08):
    frm, to = (c[0] + u[0] * start, c[1] + u[1] * start), (c[0] + u[0] * (away - 0.12), c[1] + u[1] * (away - 0.12))
    touched, ok = [], True
    for k, q in enumerate(nest):
        for i in range(4):
            g = seg_gap(frm, to, q[i], q[(i + 1) % 4])
            if g < reach:
                touched.append((k, i))
            elif not g > reach + 0.02:
                ok = False
    rim = start > 0.07 and start - cap < 0.07
    print(f"start {start}: rim {rim}, overlap {0.07 - (start - cap):.4f} ({(0.07 - (start - cap)) * 270:.1f} px@4K), "
          f"touched {touched}, as expected {touched == [(3, 1), (4, 1), (5, 0), (5, 1)]}, no near miss {ok}")
