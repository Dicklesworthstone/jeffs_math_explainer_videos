#!/usr/bin/env python3
"""Exact arithmetic examples of the complete diagonal identity, not a general proof, plus the grid's fact rows."""
from fractions import Fraction
from math import factorial, comb
import json
import sys

checks = 0
for m in range(13):
    for x in [Fraction(-2), Fraction(-1, 2), Fraction(0), Fraction(1, 3), Fraction(2)]:
        for y in [Fraction(-2), Fraction(-1, 2), Fraction(0), Fraction(1, 3), Fraction(2)]:
            diagonal = sum((x**k * y**(m-k) / (factorial(k)*factorial(m-k)) for k in range(m+1)), Fraction())
            assert diagonal == (x+y)**m/factorial(m)
            checks += 1
    for k in range(m+1):
        assert Fraction(factorial(m),factorial(k)*factorial(m-k)) == comb(m,k)
print(json.dumps({"exact_diagonal_examples":checks,"max_degree":12,"displayed_complete_degrees":[0,1,2,3],"native_comparison":"not run","general_proof":"binomial theorem plus absolute convergence; independent review open"},indent=2), file=sys.stderr)

# Fact rows. Each cell comes through the binomial route C(i+j, i)/(i+j)!, not the native i!j!.
SIDE = 4
rows = [("product.side", SIDE), ("product.cells", SIDE**2)]
for i in range(SIDE):
    for j in range(SIDE):
        value = Fraction(comb(i + j, i), factorial(i + j))
        assert value.numerator == 1
        rows.append((f"product.cell.{i}.{j}.denominator", value.denominator))
for m in range(SIDE):
    rows.append((f"product.diagonal.{m}.cells", sum(1 for i in range(SIDE) for j in range(SIDE) if i + j == m)))
    rows.append((f"product.total.{m}.denominator", sum(comb(m, k) for k in range(m + 1)) * factorial(m) // 2**m))

# 03_7. E(1) between an exact partial sum and that sum plus a bound on its tail, so the printed digits are proved.
partial = sum((Fraction(1, factorial(k)) for k in range(25)), Fraction())
tail = Fraction(2, factorial(25))
assert f"{float(partial):.5f}" == f"{float(partial + tail):.5f}" == "2.71828"
rows.append(("product.e", f"{float(partial):.5f}"))
LOW, HIGH, STEPS = -2, 3, 4
marks = [Fraction(j, STEPS) for j in range(LOW * STEPS, HIGH * STEPS + 1)]
whole = [q for q in marks if q >= 0 and q.denominator == 1]
fractions = [q for q in marks if q > 0 and q.denominator != 1]
negatives = [q for q in marks if q < 0]
assert len(whole) + len(fractions) + len(negatives) == len(marks)
rows.extend([
    ("product.ladder.low", LOW),
    ("product.ladder.high", HIGH),
    ("product.ladder.steps", STEPS),
    ("product.ladder.whole", len(whole)),
    ("product.ladder.fractions", len(fractions)),
    ("product.ladder.negatives", len(negatives)),
])
assert len(rows) == len(dict(rows)) == 33
for key, value in rows:
    print(f"{key}\t{value}")
