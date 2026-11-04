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
assert len(rows) == len(dict(rows)) == 26
for key, value in rows:
    print(f"{key}\t{value}")
