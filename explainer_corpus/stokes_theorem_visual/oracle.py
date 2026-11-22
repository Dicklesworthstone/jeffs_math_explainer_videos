"""Independent-style author enumeration for the draft scenes, not peer review.

The rows from steps. to fv. (chapters 08 to 12 and 19 to 21, CC3 scratch draft) are recomputed here by
routes other than src/facts.rs: covectors instead of the x-times-dy formula, the shoelace area instead of
midpoint sampling, Simpson's rule in exact fractions instead of monomial integrals, and a face-cell
incidence matrix instead of the per-face loop.
"""
from collections import Counter
from fractions import Fraction as F
import math

squares = [((0,0),(1,0),(1,1),(0,1)), ((1,0),(2,0),(2,1),(1,1))]
edges = Counter()
for corners in squares:
    for a, b in zip(corners, corners[1:] + corners[:1]):
        key = tuple(sorted((a, b)))
        edges[key] += 1 if (a, b) == key else -1
assert sum(value != 0 for value in edges.values()) == 6
print('two_squares.directed_edges\t8')
print('two_squares.canceled_edges\t2')
print('two_squares.outer_edges\t6')
print('interval.internal_endpoint_pairs\t2')


def num(x):
    """The facts' spelling: integers without a decimal point, other values as Rust prints f64 or a fraction."""
    if isinstance(x, F):
        return str(x.numerator) if x.denominator == 1 else '%d/%d' % (x.numerator, x.denominator)
    return str(int(x)) if float(x).is_integer() else repr(float(x))


# 08: the covector of x dy at p is (0, x); its value on a step is the dot product.
def covector(p):
    return (0.0, p[0])


def apply(c, d):
    return c[0] * d[0] + c[1] * d[1]


p, d = (2.0, 1.0), (1.0, 0.5)
print('steps.value\t' + num(apply(covector(p), d)))
print('steps.reversed\t' + num(apply(covector(p), (-1.0, -0.5))))
print('steps.doubled\t' + num(apply(covector(p), (2.0, 1.0))))
print('steps.sideways\t' + num(apply(covector(p), (1.0, 0.0))))
print('steps.at_x3\t' + num(apply(covector((3.0, 2.0)), d)))

# 09: the shoelace area of the inscribed n-gon, checked against (n/2) sin(2 pi/n).
for n in (4, 8, 16, 64):
    pts = [(math.cos(2 * math.pi * k / n), math.sin(2 * math.pi * k / n)) for k in range(n)]
    area = 0.5 * sum(pts[k][0] * pts[(k + 1) % n][1] - pts[(k + 1) % n][0] * pts[k][1] for k in range(n))
    assert abs(area - n / 2 * math.sin(2 * math.pi / n)) < 1e-12
    print('circle.sum.%d\t%.2f' % (n, area))
assert abs(0.5 * 64 * math.sin(2 * math.pi / 64) - math.pi) < 0.01

# 10: the signed area as a 2x2 determinant, rows in step order.
def det(a, b):
    return a[0] * b[1] - b[0] * a[1]


u, v = (2.0, 0.0), (1.0, 1.5)
print('areas.uv\t' + num(det(u, v)))
print('areas.vu\t' + num(det(v, u)))
slide = lambda t: (1 + 2 * t, 1.5 - 1.5 * t)
print('areas.halfway\t' + num(det(u, slide(0.5))))
print('areas.flat\t' + num(det(u, slide(1.0))))
# dx^dx is the determinant of a matrix with two equal rows.
for a, b in ((u, v), ((1, 2), (3, 4)), ((-2, 5), (7, 1))):
    assert a[0] * b[0] - b[0] * a[0] == 0


# 11/12: Simpson's rule is exact for these polynomials (degree at most 2 in each variable).
def simpson(f):
    return (f(F(0)) + 4 * f(F(1, 2)) + f(F(1))) / 6


def cube_integral(h):
    return simpson(lambda x: simpson(lambda y: simpson(lambda z: h(x, y, z))))


g = lambda x, y, z: y * y + x * z
dg_dy = lambda x, y, z: 2 * y
inside = cube_integral(dg_dy)
face_y1 = simpson(lambda x: simpson(lambda z: g(x, F(1), z)))
face_y0 = -simpson(lambda x: simpson(lambda z: g(x, F(0), z)))
print('cube.interior\t' + num(inside))
print('cube.face_y1\t' + num(face_y1))
print('cube.face_y0\t' + num(face_y0))
print('cube.boundary\t' + num(face_y1 + face_y0))
assert face_y1 + face_y0 == inside


# 19: edge integrals by Simpson along each edge of [-1/2, 1/2]^2, counterclockwise from the right edge.
def edges_of_cell():
    h = F(1, 2)
    yield (lambda s: (h, s - h)), (1, 0), (0, 1)          # right, upward
    yield (lambda s: (h - s, h)), (0, 1), (-1, 0)         # top, leftward
    yield (lambda s: (-h, h - s)), (-1, 0), (0, -1)       # left, downward
    yield (lambda s: (s - h, -h)), (0, -1), (1, 0)        # bottom, rightward


def counts(field):
    cross, along = [], []
    for path, n, t in edges_of_cell():
        cross.append(simpson(lambda s: sum(a * b for a, b in zip(field(*path(s)), n))))
        along.append(simpson(lambda s: sum(a * b for a, b in zip(field(*path(s)), t))))
    return cross, along


spread = lambda x, y: (x / 2, y / 2)
turn = lambda x, y: (-y / 2, x / 2)
c, a = counts(spread)
print('cell.spreading.cross\t' + ','.join(num(q) for q in c))
print('cell.spreading.net_outflow\t' + num(sum(c)))
print('cell.spreading.circulation\t' + num(sum(a)))
c, a = counts(turn)
print('cell.turning.along\t' + ','.join(num(q) for q in a))
print('cell.turning.net_outflow\t' + num(sum(c)))
print('cell.turning.circulation\t' + num(sum(a)))
assert sum(counts(spread)[0]) == 1 and sum(counts(turn)[1]) == 1  # div of spread, curl of turn, unit area

# 20/21: incidence matrix M (faces x cells): +1 where the flux leaves, -1 where it arrives.
before = [F(4), F(2), F(1), F(1)]
faces = [(0, 1), (0, 2), (1, 3), (2, 3)]
M = [[(1 if c == i else -1 if c == j else 0) for c in range(4)] for i, j in faces]
flux = [F(1, 10) * sum(M[f][c] * before[c] for c in range(4)) for f in range(4)]
after = [before[c] - sum(M[f][c] * flux[f] for f in range(4)) for c in range(4)]
assert all(sum(row) == 0 for row in M)  # each face: one cell loses what the other gains


def dec(q):
    s = '%.2f' % float(q)
    return s.rstrip('0').rstrip('.') if '.' in s else s


print('fv.flux\t' + ','.join(dec(q) for q in flux))
print('fv.after\t' + ','.join(dec(q) for q in after))
print('fv.total_before\t' + dec(sum(before)))
print('fv.total_after\t' + dec(sum(after)))
inflow = F(1, 4)
print('fv.inflow_cell\t' + dec(after[1] + inflow))
print('fv.open_total\t' + dec(sum(after) + inflow))
# Stability (not a fact row): on a periodic square grid the checkerboard mode is multiplied by 1 - 8k per
# step, so |1 - 8k| <= 1 needs k <= 1/4; the factor used is 1/10.
k = F(1, 10)
assert abs(1 - 8 * k) <= 1 and abs(1 - 8 * (F(1, 4) + F(1, 100))) > 1
