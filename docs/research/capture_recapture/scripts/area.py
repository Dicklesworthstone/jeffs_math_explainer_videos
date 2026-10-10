"""Numerically check the identity sum_N P(K=k | N,M,n) = Mn/(k(k-1)) for k >= 2
(area under the likelihood curve; memo section 2.2). Exact Fractions, partial
sums to N=2000 and N=20000. Standard library only.
"""
from math import comb
from fractions import Fraction as F
# Check: sum_{N} C(M,k)C(N-M,n-k)/C(N,n) = M n / (k (k-1)) for k>=2 (exact partial sums + tail bound)
def S(M, n, k, Nmax):
    return sum(F(comb(M, k) * comb(N - M, n - k), comb(N, n)) for N in range(M + n - k, Nmax + 1))
for (M, n, k) in [(10, 10, 3), (5, 4, 2), (7, 5, 2), (6, 9, 4), (12, 8, 5)]:
    target = F(M * n, k * (k - 1))
    for Nmax in (2000, 20000):
        s = S(M, n, k, Nmax)
        print(M, n, k, Nmax, float(s), float(target), float(target - s))
