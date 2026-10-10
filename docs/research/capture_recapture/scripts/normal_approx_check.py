"""Explain Feller's "x ~ 0.04" and "about 0.03" (3rd ed. p. 46).

Normal approximation to the *binomial* (no finite-population factor), with and
without continuity correction, for M = n = 1000, k = 100 at N = 8500 and 12000.
Compare with the exact hypergeometric tails 0.03468 and 0.02887 (verify.py).
Standard library only.
"""
from statistics import NormalDist as ND
from math import sqrt

for N, side in [(8500, 'le'), (12000, 'ge')]:
    p = 1000 / N
    mu = 1000 * p
    sd = sqrt(1000 * p * (1 - p))
    if side == 'le':
        print(N, 'binomial-normal P(K<=100) cc', ND().cdf((100.5 - mu) / sd),
              'nocc', ND().cdf((100 - mu) / sd))
    else:
        print(N, 'binomial-normal P(K>=100) cc', 1 - ND().cdf((99.5 - mu) / sd),
              'nocc', 1 - ND().cdf((100 - mu) / sd))
print('coverage of Feller 8500-12000 from exact tails: 1-0.034677-0.028875 =',
      1 - 0.034677 - 0.028875)
