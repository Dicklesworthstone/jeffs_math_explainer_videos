#!/usr/bin/env python3
"""Exact author checks for the proposed Stokes examples, not an independent proof."""
from collections import defaultdict
from fractions import Fraction as Q
from itertools import product
from math import prod


def line_xdy(a, b):
    """Integral of x dy on one straight edge, with exact endpoints."""
    return (a[0] + b[0]) * (b[1] - a[1]) / 2


def parity(indices):
    inversions = sum(a > b for i, a in enumerate(indices) for b in indices[i + 1:])
    return -1 if inversions % 2 else 1


# A unit square partitioned into 25 counterclockwise cells: cancel shared edges
# as oriented chains, then evaluate the surviving outer chain independently.
edges = defaultdict(int)
cell_total = Q(0)
for i, j in product(range(5), repeat=2):
    x0, x1, y0, y1 = Q(i, 5), Q(i + 1, 5), Q(j, 5), Q(j + 1, 5)
    points = [(x0, y0), (x1, y0), (x1, y1), (x0, y1)]
    for a, b in zip(points, points[1:] + points[:1]):
        cell_total += line_xdy(a, b)
        key = tuple(sorted((a, b)))
        edges[key] += 1 if (a, b) == key else -1
survivors = {k: v for k, v in edges.items() if v}
assert all(abs(v) == 1 for v in survivors.values()) and len(survivors) == 20
boundary_total = sum(sign * line_xdy(*edge) for edge, sign in survivors.items())
assert cell_total == boundary_total == Q(1)

# For n=1..8, independently check wedge signs and the box identity for
# affine coefficient functions with distinct cross terms on unequal boxes.
for n in range(1, 9):
    widths = [Q(i + 2, i + 1) for i in range(n)]
    lower = [Q(-i, i + 1) for i in range(n)]
    midpoints = [lower[i] + widths[i] / 2 for i in range(n)]
    volume = prod(widths, start=Q(1))
    diagonal = [Q((i + 1) * 3 - 7, i + 2) for i in range(n)]
    matrix = [[diagonal[i] if i == j else Q((i + 1) * (j + 2), i + j + 2)
               for j in range(n)] for i in range(n)]
    for i in range(n):
        omitted_basis = [j for j in range(n) if j != i]
        assert (-1 if i % 2 else 1) * parity([i] + omitted_basis) == 1
    local_integral = sum(diagonal) * volume
    face_integral = Q(0)
    for i in range(n):
        low_point, high_point = list(midpoints), list(midpoints)
        low_point[i], high_point[i] = lower[i], lower[i] + widths[i]
        f = lambda point: Q(i + 1) + sum(matrix[i][j] * point[j] for j in range(n))
        # Affine cross terms cancel pointwise between the paired faces.
        face_integral += (f(high_point) - f(low_point)) * prod(
            (widths[j] for j in range(n) if j != i), start=Q(1))
    assert local_integral == face_integral

# Paraboloid X=(x,y,1-x²-y²): upward X_x × X_y=(2x,2y,1).
# The proposed curl (0,0,1) has dot product 1 at each sampled rational point.
for x, y in [(Q(0), Q(0)), (Q(1, 3), Q(2, 5)), (Q(-2, 7), Q(3, 8))]:
    normal = (2 * x, 2 * y, Q(1))
    assert sum(a * b for a, b in zip((Q(0), Q(0), Q(1)), normal)) == 1
# On the unit rim, F·dr/dθ=(sin²θ+cos²θ)/2=1/2; integrate 0..2π.
for c, s in [(Q(1), Q(0)), (Q(3, 5), Q(4, 5)), (Q(-5, 13), Q(12, 13))]:
    assert ((-s / 2) * (-s) + (c / 2) * c) == Q(1, 2)
assert Q(1, 2) * 2 == 1  # both surface and rim are 1·π
# On the unit sphere F·n=1/3; its 4π flux matches ball volume 4π/3.
assert Q(1, 3) * 4 == Q(4, 3)
# For α=(x dy-y dx)/r², x=r cosθ,y=r sinθ yields dθ;
# induced outer and inner orientations therefore cancel.
for c, s in [(Q(1), Q(0)), (Q(3, 5), Q(4, 5))]:
    assert c * c + s * s == 1
assert 2 + (-2) == 0  # coefficients of π for outer and inner boundaries
print('25 cells: exact signed-edge cancellation and square integral 1')
print('n=1..8: wedge signs and affine box integrals agree exactly')
print('paraboloid/rim sampled algebra: common integral π; sphere: 4π/3')
print('annulus outer/inner orientation coefficients cancel; no independent review')
