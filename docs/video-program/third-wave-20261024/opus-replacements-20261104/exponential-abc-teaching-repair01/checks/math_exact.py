"""Exact checks of what the repaired cues newly say and print. Python standard library only; no import from staged code.

Usage from the repo root:  python3 -I -B math_exact.py
These are my own derivations, written by the same author as the repair, so they test arithmetic and edges, not the
repair's judgement. The checks at named inputs are examples of the laws, not proofs of them.
"""
from fractions import Fraction as Q
from math import factorial

fails = 0


def ck(ok, claim, extra=""):
    global fails
    print(("ok   " if ok else "FAIL ") + claim + (f"  [{extra}]" if extra else ""))
    fails += 0 if ok else 1


R = Q(3, 2)


def t(x, k):
    return abs(x) ** k / factorial(k)


print("--- 02_1 to 02_3: the columns, the equations, the factorials")
c = {0: Q(1)}
for n in range(1, 8):
    c[n] = c[n - 1] / n
    ck(n * c[n] == c[n - 1], f"column {n - 1}: {n} times the coefficient of x^{n} equals the coefficient above it", f"{c[n]}")
    ck(c[n] == Q(1, factorial(n)), f"the chain's divisions give 1/{n}!")
ck([c[i] for i in range(5)] == [1, 1, Q(1, 2), Q(1, 6), Q(1, 24)], "02_3 s2: 'one, one, one half, one sixth, one twenty-fourth'")
for k in range(1, 5):
    product = 1
    for i in range(1, k + 1):
        product *= i
    ck(product == factorial(k), f"the running product 1...{k} printed under column {k} is {k}!", str(product))

print("--- 02_4: differentiating a polynomial of degree N lands N terms and misses one power")
for N in (4, 5, 6):
    derivative = {k - 1: k * c[k] for k in range(1, N + 1)}
    ck(all(derivative[k] == c[k] for k in range(N)), f"degree {N}: the derivative's first {N} columns match the function's")
    ck(N not in derivative and c[N] != 0, f"degree {N}: the derivative has no x^{N}, and the function does: the hole is real")

print("--- 02_5: ratio, halving and tail, at R = 3/2 and at the edge of the hypothesis")
ratios = {i: t(R, i) / t(R, i - 1) for i in range(1, 30)}
ck(all(ratios[i] == R / i for i in ratios), "every printed ratio is R / index exactly")
ck([ratios[i] for i in range(1, 9)] == [Q(3, 2), Q(3, 4), Q(1, 2), Q(3, 8), Q(3, 10), Q(1, 4), Q(3, 14), Q(3, 16)], "the ratio row reads 3/2, 3/4, 1/2, 3/8, 3/10, 1/4, 3/14, 3/16")
ck(ratios[2] > Q(1, 2) and ratios[3] == Q(1, 2), "halving starts at index 3, with equality there: the sign is non-strict for a reason")
for radius in (Q(1, 2), Q(1), R, Q(2), Q(5, 2), Q(7)):
    # The smallest N the hypothesis N + 2 >= 2R allows.
    N = 0
    while N + 2 < 2 * radius:
        N += 1
    first = t(radius, N + 1)
    ck(all(t(radius, k) <= first / 2 ** (k - N - 1) for k in range(N + 1, N + 60)), f"R={radius}, N={N}: every tail term sits under the halving ceiling")
    tail = sum((t(radius, k) for k in range(N + 1, N + 80)), Q(0))
    ck(tail <= 2 * first, f"R={radius}, N={N}: the tail, truncated at 80 terms, is at most the printed bound")
    ck(t(radius / 2, N + 1) <= first and t(-radius, N + 1) == first, f"R={radius}: an inner input's term is no larger, and -R's equals R's")
    if N > 0:
        M = N - 1
        ck(t(radius, M + 2) > t(radius, M + 1) / 2, f"edge, R={radius}: one degree below the hypothesis the first ceiling step already fails")
for n, printed in ((4, "0.12657"), (8, "0.00022")):
    bound = 2 * t(R, n + 1)
    p = Q(printed)
    ck(p >= bound and p - bound < Q(1, 100000), f"degree {n}: {printed} is 2 t_{n + 1} rounded up at the fifth place", f"exact {float(bound):.7f}")
    tail = sum((t(R, k) for k in range(n + 1, 60)), Q(0))
    ck(tail < bound, f"degree {n}: the true tail is under it", f"{float(tail):.7f}")
ck(2 * t(R, 5) == Q(81, 640), "degree 4: the bound is 81/640 before rounding")
ck(t(R, 5) * Q(16) * Q(19, 10) < Q(2), "zoom: the tallest inset bar stays under two scene units at the 1.9 scale", f"{float(t(R, 5) * 16 * Q(19, 10)):.3f}")
for k in range(1, 10):
    ck(k * R ** (k - 1) / factorial(k) == t(R, k - 1), f"02_5 s4: the derivative of term {k} has the size of term {k - 1}")
inputs = [R * j / 30 for j in range(-30, 31)]
ck(all(t(x, k) <= t(R, k) for x in inputs for k in range(10)), "02_5 s5: across the sweep no bar passes its ceiling")
ck(all(t(x, 0) == 1 for x in inputs), "the sweep's edge: bar 0 stays at one, including at x = 0")
ck(all(t(Q(0), k) == 0 for k in range(1, 10)), "the sweep's other edge: at x = 0 every later bar is zero, and the live bar clamps instead of failing")


print("--- 02_6: the promises, in payment order, against the brief's section order")
order = ["why adding inputs multiplies outputs", "why the stepped approximations reach this curve", "why it is the only solution"]
SLICE = "docs/video-program/third-wave-20261024/opus-replacements-20261104/exponential-abc-teaching-repair01/candidate/series01/script-slice.tsv"
line = next(l for l in open(SLICE, encoding="utf-8") if l.startswith("02_6"))
positions = [line.index(x) for x in order]
ck(positions == sorted(positions), "the clauses come in the order C (product), D (Euler limit), E (uniqueness)")


def enclose(x, degree=40):
    """E(x) between exact bounds: a partial sum and its tail bound, with both signs handled."""
    assert degree + 2 >= 2 * abs(x)
    partial = sum((x**k / factorial(k) for k in range(degree + 1)), Q(0))
    width = 2 * t(x, degree + 1)
    return partial - width, partial + width


def overlap(a, b):
    return a[0] <= b[1] and b[0] <= a[1]


def times(a, b):
    assert a[0] > 0 and b[0] > 0
    return a[0] * b[0], a[1] * b[1]


print("--- 03_7: named inputs, in exact brackets. Examples of the law, not its proof")
one = enclose(Q(1))
ck(Q(271828, 100000) < one[0] and one[1] < Q(271829, 100000), "E(1) lies between 2.71828 and 2.71829, so the printed five places are its digits", f"width {float(one[1] - one[0]):.1e}")
ck(one[1] - one[0] < Q(1, 10**20), "the bracket is tight enough for the checks below to mean something")
ck(overlap(enclose(Q(2)), times(one, one)), "E(2) = e times e")
ck(overlap(enclose(Q(3)), times(times(one, one), one)), "E(3) = e cubed")
half = enclose(Q(1, 2))
ck(half[0] > 0 and overlap(times(half, half), one), "E(1/2) is positive and its square is e")
quarter = enclose(Q(3, 4))
root = enclose(Q(1, 4))
ck(overlap(quarter, times(times(root, root), root)), "fractions: E(3/4) is E(1/4) cubed, and E(1/4) is the positive fourth root of e")
ck(overlap(times(root, times(root, times(root, root))), one), "fractions: E(1/4) to the fourth is e")
for x in (Q(1, 4), Q(1), Q(2)):
    pos, neg = enclose(x), enclose(-x)
    ck(neg[0] > 0 and overlap(times(pos, neg), (Q(1), Q(1))), f"reciprocals: E({-x}) is positive and E({x}) E({-x}) = 1")
ck(overlap(enclose(Q(0)), (Q(1), Q(1))), "edge: E(0) = 1, so the zero mark is earned without the law")
ck((-half[0]) ** 2 == half[0] ** 2, "edge: squares alone do not pick the root. A negative number has the same square, which is why TEX[12] prints E>0")
# Rational powers leave the irrationals open: that gap is exactly what the continuity clause fills.
ck(all(Q(p, q) ** 2 != 2 for p in range(1, 60) for q in range(1, 60)), "edge: no fraction squares to 2 in this search, so the marks alone cannot reach every input: continuity is doing work")

print("--- 03_7 line and the product's guides")
LOW, HIGH, STEPS = -2, 3, 4
marks = [Q(j, STEPS) for j in range(LOW * STEPS, HIGH * STEPS + 1)]
whole = [m for m in marks if m >= 0 and m.denominator == 1]
fractions = [m for m in marks if m > 0 and m.denominator != 1]
negatives = [m for m in marks if m < 0]
ck(sorted(whole + fractions + negatives) == marks, "the three groups cover the marks once each", f"{len(whole)}+{len(fractions)}+{len(negatives)}")
x_of = lambda u: Q(-4) + Q(8, 5) * (u - LOW)
ck(x_of(Q(LOW)) == -4 and x_of(Q(HIGH)) == 4, "the line runs from x = -4 to x = 4, in frame")
centres = [(Q(-42, 10) + Q(135, 100) * i, Q(14, 10) - (3 - i)) for i in range(4)]
a, b = (Q(-455, 100), Q(-186, 100)), (Q(2, 10), Q(166, 100))
slope = (b[1] - a[1]) / (b[0] - a[0])
miss = [abs(a[1] + slope * (x - a[0]) - y) for x, y in centres]
ck(max(miss) < Q(1, 100), "03_2's line runs through the centres of all four degree-three cells", f"largest miss {float(max(miss)):.4f}")
print("RESULT", "PASS" if not fails else f"{fails} FAIL")
raise SystemExit(1 if fails else 0)
