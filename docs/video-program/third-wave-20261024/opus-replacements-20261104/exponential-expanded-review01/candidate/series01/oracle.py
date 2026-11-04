#!/usr/bin/env python3
"""Exact polynomial arithmetic; no recurrence or native-model import."""
from fractions import Fraction
from math import ceil, factorial

RADIUS = Fraction(3, 2)
SEGMENTS = 60
rows = [
    ("factorials.plot.radius", "1.500000"),
    ("factorials.plot.top", "5.000000"),
    ("factorials.plot.segments", SEGMENTS),
]
for n in range(5):
    denominator = factorial(n)
    rows.append((f"factorials.coefficient.{n}", "1" if denominator == 1 else f"1/{denominator}"))
    rows.append((f"factorials.denominator.{n}", denominator))
    for j in range(SEGMENTS + 1):
        x = RADIUS * j / SEGMENTS
        y = sum((x**k / factorial(k) for k in range(n + 1)), Fraction())
        rows.append((f"factorials.curve.{n}.{j}.x", f"{float(x):.6f}"))
        rows.append((f"factorials.curve.{n}.{j}.y", f"{float(y):.5f}"))
        assert 0 <= x <= RADIUS and 0 <= y <= 5
        scaled = y * 200000
        assert scaled.denominator != 1 or scaled.numerator % 2 == 0, "an exact tie at the fifth place"
n = 4
ratio = RADIUS / (n + 2)
bound = RADIUS**(n+1) / factorial(n+1) / (1 - ratio)
assert bound == Fraction(27,320) and ratio == Fraction(1, 4)
rounded_up = Fraction(ceil(bound * 100000), 100000)
assert rounded_up >= bound
rows.extend([
    ("factorials.tail.order", n),
    ("factorials.tail.ratio", f"{float(ratio):.2f}"),
    ("factorials.tail.bound", f"{float(rounded_up):.5f}"),
])
assert len(rows) == len(dict(rows)) == 626
for key, value in rows:
    print(f"{key}\t{value}")
