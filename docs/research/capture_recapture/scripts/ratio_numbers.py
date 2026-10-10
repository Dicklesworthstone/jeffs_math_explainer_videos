"""On-screen numbers for the ratio "seesaw" (memo section 2.2).

Prints numerator (N-M)(N-n), denominator N(N-M-n+k) and their difference,
which equals Mn - kN exactly. Feller's data (M=n=1000, k=100) at
N = 9999, 10000, 10001, and the toy (M=n=10, k=3) at N = 33, 34.
Standard library only.
"""
from fractions import Fraction as F

M, n, k = 1000, 1000, 100
for N in (9999, 10000, 10001):
    a = (N - M) * (N - n)
    b = N * (N - M - n + k)
    print(N, a, b, a - b, M * n - k * N)
M, n, k = 10, 10, 3
for N in (33, 34):
    a = (N - M) * (N - n)
    b = N * (N - M - n + k)
    print(N, a, b, F(a, b))
