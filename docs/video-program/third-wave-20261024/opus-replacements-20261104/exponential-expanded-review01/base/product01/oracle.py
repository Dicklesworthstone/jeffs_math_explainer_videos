#!/usr/bin/env python3
"""Exact arithmetic examples of the complete diagonal identity, not a general proof."""
from fractions import Fraction
from math import factorial, comb
import json

checks = 0
for m in range(13):
    for x in [Fraction(-2), Fraction(-1, 2), Fraction(0), Fraction(1, 3), Fraction(2)]:
        for y in [Fraction(-2), Fraction(-1, 2), Fraction(0), Fraction(1, 3), Fraction(2)]:
            diagonal = sum((x**k * y**(m-k) / (factorial(k)*factorial(m-k)) for k in range(m+1)), Fraction())
            assert diagonal == (x+y)**m/factorial(m)
            checks += 1
    for k in range(m+1):
        assert Fraction(factorial(m),factorial(k)*factorial(m-k)) == comb(m,k)
print(json.dumps({"exact_diagonal_examples":checks,"max_degree":12,"displayed_complete_degrees":[0,1,2,3],"native_comparison":"not run","general_proof":"binomial theorem plus absolute convergence; independent review open"},indent=2))
