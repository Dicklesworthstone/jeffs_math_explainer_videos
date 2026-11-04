#!/usr/bin/env python3
"""Author check with rational arithmetic in Q(sqrt(2), i); not independent review."""
from fractions import Fraction as F
from pathlib import Path
import hashlib


def add(a, b):
    return (a[0] + b[0], a[1] + b[1])


def mul(a, b):
    return (a[0] * b[0] + 2 * a[1] * b[1], a[0] * b[1] + a[1] * b[0])


def neg(a):
    return (-a[0], -a[1])


def cmul(a, b):
    return (add(mul(a[0], b[0]), neg(mul(a[1], b[1]))),
            add(mul(a[0], b[1]), mul(a[1], b[0])))


def poly_mul(a, b):
    out = [(F(0), F(0)) for _ in range(len(a) + len(b) - 1)]
    for i, x in enumerate(a):
        for j, y in enumerate(b):
            out[i + j] = add(out[i + j], mul(x, y))
    return out


def poly_add(a, b):
    assert len(a) == len(b)
    return [add(x, y) for x, y in zip(a, b)]


zero, one, sqrt2 = (F(0), F(0)), (F(1), F(0)), (F(0), F(1))
half = (F(1, 2), F(0))
quarter_sqrt2 = (F(0), F(1, 4))
q_plus, q_minus = [one, sqrt2, one], [one, neg(sqrt2), one]
n_plus, n_minus = [half, quarter_sqrt2], [half, neg(quarter_sqrt2)]
assert poly_mul(q_plus, q_minus) == [one, zero, zero, zero, one]
assert poly_add(poly_mul(n_plus, q_minus), poly_mul(n_minus, q_plus)) == [one, zero, zero, zero]

# Each numerator is +/- sqrt(2)/8 times its denominator's derivative + 1/4.
eighth_sqrt2 = (F(0), F(1, 8))
constant_quarter = (F(1, 4), F(0))
assert poly_add(poly_mul([eighth_sqrt2], [sqrt2, (F(2), F(0))]), [constant_quarter, zero]) == n_plus
assert poly_add(poly_mul([neg(eighth_sqrt2)], [neg(sqrt2), (F(2), F(0))]), [constant_quarter, zero]) == n_minus

# Upper poles u=(1+i)/sqrt(2), v=(-1+i)/sqrt(2), encoded in Q(sqrt(2),i).
sqrt2_half = (F(0), F(1, 2))
u, v = (sqrt2_half, sqrt2_half), (neg(sqrt2_half), sqrt2_half)
for z in (u, v):
    square = cmul(z, z)
    assert cmul(square, square) == (neg(one), zero)
assert (add(u[0], v[0]), add(u[1], v[1])) == (zero, sqrt2)
# Since r^4=-1, 1/(4r^3)=-r/4. Therefore 2i times the sum is sqrt(2)/2.
quarter = (F(1, 4), F(0))
residue_sum = (zero, neg(mul(sqrt2, quarter)))
assert cmul((zero, (F(2), F(0))), residue_sum) == (sqrt2_half, zero)

root = Path.cwd()
for relative in ("explainer_corpus/complex_analysis_visual/script.tsv",
                 "explainer_corpus/complex_analysis_visual/src/chapters/integral.rs"):
    path = root / relative
    print(f"sha256 {relative} {hashlib.sha256(path.read_bytes()).hexdigest()}")
print("PASS exact factorization and partial-fraction numerator identity")
print("PASS exact numerator decomposition into logarithmic derivatives plus 1/4")
print("PASS exact upper-root fourth powers, residue sum, and pi/sqrt(2) coefficient")
print("LIMIT: author algebra check only; no listening, picture, sync, or independent review")
