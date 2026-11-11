"""Mirror of candidate03's test `the_preview_arrow_reaches_the_loop_head_first`, applied to the
outline that arrow-mirror01.py builds from fmn a399d626's Arrow (anchors and line midpoints only;
fmn's point records add no point outside them). Read-only; no compile.

Cases: build268 (default buff 0.25) must FAIL; candidate (buff 0.0) must PASS; a reversed arrow
(head at zero, buff 0.0) must FAIL the head-first check; the right arrow measured against the
wrong angle must FAIL.

usage: python3 -I -B test-mirror01.py ARROW_MIRROR.py
"""
import math
import sys

code = open(sys.argv[1], encoding="utf-8").read().split("\nO = lp_p(")[0]
ns = {}
exec(compile(code, sys.argv[1], "exec"), ns)
build, lp_p = ns["build"], ns["lp_p"]

R_PLANE, ARG, UNIT = 0.8, 0.9, 1.5 / 2.4


def outline_points(poly):
    pts = []
    for a, b in zip(poly, poly[1:]):
        pts += [a, ((a[0] + b[0]) / 2, (a[1] + b[1]) / 2), b]
    return pts


def run_test(poly, arg=ARG):
    pa, pb = lp_p(0.0, 0.0), lp_p(R_PLANE * math.cos(arg), R_PLANE * math.sin(arg))
    dx, dy = pb[0] - pa[0], pb[1] - pa[1]
    length = math.sqrt(dx * dx + dy * dy)
    if abs(length - R_PLANE * UNIT) >= 1e-12:
        return False, "length %r" % length
    pts = []
    for q in outline_points(poly):
        qx, qy = q[0] - pa[0], q[1] - pa[1]
        pts.append(((qx * dx + qy * dy) / length, (qy * dx - qx * dy) / length))
    lo = min(p[0] for p in pts)
    hi = max(p[0] for p in pts)
    if not (abs(lo) < 1e-6 and abs(hi - length) < 1e-6):
        return False, "range %.6f to %.6f of %.6f" % (lo, hi, length)
    for along, across in pts:
        if along > hi - 1e-6 and not abs(across) < 1e-6:
            return False, "farthest point off the axis: along %.6f across %.6f" % (along, across)
    return True, "range %.2e to %.6f of %.6f, farthest point on the axis" % (lo, hi, length)


O = lp_p(0.0, 0.0)
END = lp_p(R_PLANE * math.cos(ARG), R_PLANE * math.sin(ARG))
cases = [
    ("build268 default buff 0.25", build(O, END, 0.25)["poly"], ARG, False),
    ("candidate buff 0.0", build(O, END, 0.0)["poly"], ARG, True),
    ("reversed arrow buff 0.0 (head at zero)", build(END, O, 0.0)["poly"], ARG, False),
    ("right arrow, test at angle 1.2", build(O, END, 0.0)["poly"], 1.2, False),
]
ok_all = True
for name, poly, arg, expect in cases:
    ok, why = run_test(poly, arg)
    status = "as expected" if ok == expect else "UNEXPECTED"
    ok_all &= ok == expect
    print("%-42s test %s (%s) -- %s" % (name, "PASS" if ok else "FAIL", why, status))
print("ALL CASES AS EXPECTED:", ok_all)
