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
# Separate exact rational formula, not a translation of the Rust Euler loop.
facts.extend([("rate.start", "1.000000"), ("rate.duration", "1.000000")])
counts = (4, 8, 16)
for n in counts:
    facts.append((f"rate.{n}.steps", n))
    for k in range(n + 1):
        t = Fraction(k, n)
        y = Fraction(n + 1, n) ** k
        linear = 1 + t
        for field, value in (("t", t), ("proportional", y), ("constant", linear)):
            facts.append((f"rate.{n}.{k}.{field}", f"{float(value):.6f}"))
        facts.append((f"rate.{n}.{k}.right_readout", f"height {float(y):.2f}: slope {float(y):.2f}"))
        facts.append((f"rate.{n}.{k}.left_readout", f"height {float(linear):.2f}: slope 1.00"))
    previous = Fraction(n + 1, n) ** (n - 1)
    endpoint = Fraction(n + 1, n) ** n
    facts.append((f"rate.{n}.last_step", f"last step: slope {float(previous):.2f}, height now {float(endpoint):.2f}"))
for k in range(1, len(counts) + 1):
    ns = counts[:k]
    names = ", ".join(map(str, ns))
    ends = ", ".join(f"{float(Fraction(n + 1, n) ** n):.3f}" for n in ns)
    facts.append((f"rate.refinement.{k}", f"{names} steps: {ends}"))
assert len(dict(facts)) == len(facts), "duplicate fact key"

for key, value in facts:
    print(f"{key}\t{value}")
