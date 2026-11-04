#!/usr/bin/env python3
"""Exact polynomial arithmetic; no recurrence or native-model import."""
from fractions import Fraction
from math import factorial

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
        rows.append((f"factorials.curve.{n}.{j}.y", f"{float(y):.6f}"))
        assert 0 <= x <= RADIUS and 0 <= y <= 5
n = 4
bound = RADIUS**(n+1) / factorial(n+1) / (1 - RADIUS / (n+2))
assert bound == Fraction(27,320)
rows.extend([("factorials.tail.order", n), ("factorials.tail.bound", f"{float(bound):.5f}")])
assert len(rows) == len(dict(rows)) == 625
for key, value in rows:
    print(f"{key}\t{value}")
