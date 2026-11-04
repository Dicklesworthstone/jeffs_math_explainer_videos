#!/usr/bin/env python3
"""Exact-rational checks of the claims A-C rest on, including where the screen text fails."""
from fractions import Fraction as F
from itertools import product
from math import comb, e as EULER, factorial

bad = 0


def check(name, ok, detail=""):
    global bad
    bad += not ok
    print(("ok   " if ok else "FAIL ") + name + (f"  {detail}" if detail else ""))


# Series: coefficient matching, as polynomial identities, not samples.
a = [F(1)]
for k in range(12):
    a.append(a[k] / (k + 1))
check("matching gives a_k = 1/k! through degree 12", all(a[k] == F(1, factorial(k)) for k in range(13)))
derivative = [(k + 1) * a[k + 1] for k in range(12)]
check("derivative row equals function row, shifted", derivative == a[:12])
top = 8
p = [F(1, factorial(k)) for k in range(top + 1)]
dp = [(k + 1) * p[k + 1] for k in range(top)]
check("a finite polynomial misses only its top term", dp == p[:top] and len(dp) == len(p) - 1)

# Series: the displayed tail bound against the true supremum.
R, N = F(3, 2), 4
bound = R ** (N + 1) / factorial(N + 1) / (1 - R / (N + 2))
check("displayed bound at R = 3/2, N = 4 is 27/320", bound == F(27, 320), str(bound))
tail = sum(R**k / factorial(k) for k in range(N + 1, 60))
check("true supremum of the tail sits below it", tail < bound, f"tail {float(tail):.6f}, bound {float(bound):.6f}")
check("0.08437 is below the exact bound, so it is not a rounded-up bound", F(8437, 100000) < bound)
check("0.08437 still exceeds the true tail", F(8437, 100000) > tail)

# Series: what the screen TeX states with no hypothesis.
R, k = 10, 1
holds = F(R, k + 1) <= F(1, 2)
print(f"note R/(k+1) <= 1/2 at R = {R}, k = {k}: {holds} (TEX[1] shows it with no condition; it needs k+1 >= 2R)")
check("TEX[1] is false without a condition", not holds)
R, N = F(10), 4
denominator = 1 - R / (N + 2)
print(f"note 1 - R/(N+2) at R = 10, N = 4 is {denominator}; TEX[2] would claim a tail at most a negative number")
check("TEX[2] is false without N+2 > R", denominator < 0)
print(f"note at R = 3/2, N = 4 the geometric ratio is {F(3, 2) / 6}, not the one half the voice names")

# Product: every cell, diagonal and total.
SIDE = 4
cells = {(i, j): F(1, factorial(i) * factorial(j)) for i, j in product(range(SIDE), repeat=2)}
denominators = [factorial(i) * factorial(j) for j in range(SIDE) for i in range(SIDE)]
print("cell denominators, row by row:", denominators)
for m in range(SIDE):
    weights = [cells[(i, m - i)] * factorial(m) for i in range(m + 1)]
    check(f"degree {m} diagonal is (x+y)^{m}/{factorial(m)}", weights == [comb(m, i) for i in range(m + 1)])
check(
    "factorial weights are binomial through degree 24",
    all(F(factorial(m), factorial(k) * factorial(m - k)) == comb(m, k) for m in range(25) for k in range(m + 1)),
)
complete = all(all(i < SIDE and m - i < SIDE for i in range(m + 1)) for m in range(SIDE))
degree_four_cut = not all(i < SIDE and SIDE - i < SIDE for i in range(SIDE + 1))
check("complete diagonals in a 4 by 4 corner are degrees 0 to 3", complete and degree_four_cut)
check("the corner holds 16 products", len(cells) == 16)

# Rate: the stepped endpoints stay below e and rise; the 4-step value is exact.
ends = [F(n + 1, n) ** n for n in (4, 8, 16)]
check("rate endpoints rise and stay below e", ends[0] < ends[1] < ends[2] and float(ends[2]) < EULER)
check("4-step endpoint is 625/256", ends[0] == F(625, 256))


# Product closing claims, in exact truncated series.
def E(t, n=40):
    return sum(t**k / factorial(k) for k in range(n))


x, y = F(1, 3), F(-1, 2)
gap = abs(E(x) * E(y) - E(x + y))
check("E(x)E(y) = E(x+y) to within truncation at x = 1/3, y = -1/2", gap < F(1, 10**30), f"{float(gap):.2e}")
check("E(x)E(-x) = 1 to within truncation", abs(E(x) * E(-x) - 1) < F(1, 10**30))
check("E(x) = E(x/2)^2 to within truncation", abs(E(y) - E(y / 2) ** 2) < F(1, 10**30))
print("RESULT", "PASS" if not bad else f"{bad} FAILED")
