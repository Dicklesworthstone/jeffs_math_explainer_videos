#!/usr/bin/env python3
"""Exact finite author check of the Stokes script's box signs, not a proof."""
from fractions import Fraction as Q
from math import prod


def interval_power(a, b, exponent):
    return (b ** (exponent + 1) - a ** (exponent + 1)) / (exponent + 1)


def wedge_sign(order):
    inversions = sum(x > y for i, x in enumerate(order) for y in order[i + 1 :])
    return (-1) ** inversions


cases = 0
for n in range(1, 9):
    # Unequal, non-symmetric rational intervals and cross-coordinate factors.
    lo = [Q(-i - 2, i + 3) for i in range(n)]
    hi = [lo[i] + Q(i + 3, i + 2) for i in range(n)]
    for missing in range(n):
        basis = [i for i in range(n) if i != missing]
        form_sign = (-1) ** missing
        # f_i includes cubic and mixed terms; each term is a coefficient and
        # an exponent tuple. The same f_i is integrated on both faces.
        powers1 = tuple(2 if j == missing else (j + missing) % 3 for j in range(n))
        powers2 = tuple(3 if j == missing else (j + 1) % 2 for j in range(n))
        terms = [(Q(2 * missing + 1, 3), powers1), (Q(-3, 5), powers2)]
        volume = Q(0)
        for coefficient, powers in terms:
            p = powers[missing]
            derivative = coefficient * p
            for j in range(n):
                derivative *= interval_power(lo[j], hi[j], powers[j] - (j == missing))
            volume += form_sign * wedge_sign([missing] + basis) * derivative
        boundary = Q(0)
        for side, x in [(-1, lo[missing]), (1, hi[missing])]:
            # Outward-first orientation: upper (-1)^i, lower (-1)^(i+1)
            # relative to the remaining coordinates in their original order.
            face_orientation = side * (-1) ** missing
            for coefficient, powers in terms:
                face = coefficient * x ** powers[missing]
                face *= prod((interval_power(lo[j], hi[j], powers[j]) for j in basis), start=Q(1))
                boundary += face_orientation * form_sign * face
        assert volume == boundary, (n, missing, volume, boundary)
        assert volume != -boundary, (n, missing, "reversing the boundary sign went undetected")
        cases += 1
print(f"{cases} nonlinear missing-coordinate terms: exact box and oriented-face integrals agree (n=1..8)")
print("Every checked term detects a reversed boundary sign; finite author check, not independent proof or review.")
