"""Exact Chapman bias -(N+1)C(N-M,n+1)/C(N+1,n+1) versus expected recaptures
E[k] = Mn/N, compared with N*exp(-E[k]) (memo section 2.4; Robson-Regier rule).
Standard library only.
"""
from math import comb, exp
from fractions import Fraction as F
for N, M, n in [(1000, 64, 64), (1000, 50, 50), (1000, 100, 100), (10000, 200, 200), (10000, 1000, 1000)]:
    b = -F((N + 1) * comb(N - M, n + 1), comb(N + 1, n + 1))
    mu = F(M * n, N)
    print(f"N={N} M=n={M}: E[k]={float(mu):.3f}, Chapman exact bias={float(b):.4f} ({100*float(b)/N:.3f}% ), exp(-E[k])*N={exp(-float(mu))*N:.4f}")
