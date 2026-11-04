#!/usr/bin/env python3
"""Separate exact-rational author checks for finite displayed examples."""
from fractions import Fraction
from math import factorial

facts = [
    ("slide.a", 2), ("slide.b", 3), ("slide.product", 2 * 3),
    ("growth.base", 2), ("growth.first", 3), ("growth.second", 2),
    ("growth.first_factor", 2 ** 3), ("growth.second_factor", 2 ** 2),
    ("growth.total_factor", 2 ** 5),
]
for n in (1, 2, 4, 16, 64, 1024):
    facts.append((f"limit.{n}", f"{float(Fraction(n + 1, n) ** n):.5f}"))
facts.append(("series.order", 8))
facts.append(("series.sum", f"{float(sum((Fraction(1, factorial(k)) for k in range(9)), Fraction())):.5f}"))
for key, value in facts:
    print(f"{key}\t{value}")
