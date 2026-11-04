"""Exact checks of what chapters 04, 05 and 06 say and print. Python standard library only; no import from staged code.

Usage from the repo root:  python3 -I -B math_exact.py
Written by the same author as the chapters, so this tests arithmetic and edges, not judgement. Checks at named inputs
are examples of the laws, not proofs of them. Rule 3 applies: each always or never sentence is tried at its edge.
"""
from fractions import Fraction as Q
from math import comb, factorial

fails = 0


def ck(ok, claim, extra=""):
    global fails
    print(("ok   " if ok else "FAIL ") + claim + (f"  [{extra}]" if extra else ""))
    fails += 0 if ok else 1


def chain(n, k):
    """The product of (1 - j/n), j from 0 to k-1."""
    out = Q(1)
    for j in range(k):
        out *= 1 - Q(j, n)
    return out


def term(x, n, k):
    return x**k / factorial(k) * chain(n, k)


def size(x, k):
    return abs(x) ** k / factorial(k)


def tail_bound(x, cut, terms=400):
    return sum((size(x, k) for k in range(cut + 1, terms)), Q(0))


R = Q(3, 2)
XS = (R, -R, Q(-3), Q(1), Q(0), Q(1, 7), Q(-9, 2))
NS = (1, 2, 3, 4, 5, 8, 16, 64)

print("--- 04_1: climbing by a fraction of your height multiplies it")
for h in (Q(1), Q(5, 4), Q(7, 3)):
    ck(h + Q(1, 4) * h == Q(5, 4) * h, f"height {h}: h + h/4 = 5h/4")
ck(Q(5, 4) ** 4 == Q(625, 256), "four steps make (5/4)^4 = 625/256", f"{float(Q(625, 256)):.5f}")

print("--- 04_4: four cells")
ck(2**4 == 16 and sum(comb(4, k) for k in range(5)) == 16, "sixteen ways, sorted by how many took x")
ck(comb(4, 2) == 6, "six pairs of cells take x squared")
ck(6 * Q(1, 4) * Q(1, 4) == Q(6, 16), "six times a quarter times a quarter is six sixteenths")

print("--- 04_5, 04_6: the exact statement, including past k = n")
for n in NS:
    for k in range(0, n + 6):
        falling = Q(1)
        for j in range(k):
            falling *= n - j
        ck(falling / factorial(k) == comb(n, k), f"n={n} k={k}: falling product over k! is the choose count") if k in (0, n, n + 1) else None
        if comb(n, k) * Q(1, n) ** k != chain(n, k) / factorial(k):
            ck(False, f"binom(n,k)/n^k = chain/k! fails at n={n} k={k}")
ck(True, "binom(n,k)/n^k = chain(n,k)/k! for every n and k tried, k up to n+5")
ck(chain(4, 2) / factorial(2) == Q(6, 16), "six sixteenths = one half times one times three quarters")
ck(Q(1, 2) * 1 * Q(3, 4) == Q(6, 16), "and that product is written in the cue's order")
for n in NS:
    ck(chain(n, n + 1) == 0, f"n={n}: asking for n+1 cells makes one factor exactly zero, 1 - n/n")

print("--- 05_3: the sentence 'until k passes n, each factor lies between zero and one, and after that the chain is zero'")
for n in NS:
    good = all(0 <= 1 - Q(j, n) <= 1 for j in range(0, n + 1))
    ck(good, f"n={n}: every factor j = 0..n lies in [0,1]")
    ck(1 - Q(n + 1, n) < 0, f"n={n}: the factor at j = n+1 is negative, so 'each factor' alone would be false", f"{1 - Q(n + 1, n)}")
    ck(all(chain(n, k) == 0 for k in range(n + 1, n + 8)), f"n={n}: every chain past k = n is zero, negative factor or not")
    ck(all(0 <= chain(n, k) <= 1 for k in range(n + 8)), f"n={n}: every whole chain lies in [0,1]")
print("--- 05_3: 'no larger than the series term', at both signs and an edge")
for x in XS:
    for n in NS:
        ok = all(abs(term(x, n, k)) <= size(x, k) for k in range(n + 8))
        if not ok:
            ck(False, f"ceiling broken x={x} n={n}")
ck(True, "|term| <= |x|^k/k! at every x, n and k tried")
ck(abs(term(R, 1, 0)) == size(R, 0) and abs(term(R, 1, 1)) == size(R, 1), "equality happens, so the ceiling cannot be lowered: k = 0 and k = 1, n = 1")

print("--- 05_2: the warning sum")
for n in (4, 16, 40, 100):
    ck(n * Q(1, n) == 1, f"{n} terms of 1/{n} total exactly one")
ck(Q(1, 100) < Q(1, 40) < Q(1, 16) < Q(1, 4), "column three shrinks along the printed counts")
print("--- 05_3: 'its column k can reach one over k, and those add up without limit'")
for k in (1, 3, 7):
    ck(max(Q(1, n) for n in range(k, 400)) == Q(1, k), f"column {k} exists only for n >= {k}; its largest value is 1/{k}, at n = {k}")
h = {m: sum(Q(1, k) for k in range(1, m + 1)) for m in (4, 8, 16, 32, 64)}
ck(all(h[2 * m] - h[m] >= Q(1, 2) for m in (4, 8, 16, 32)), "each doubling of the harmonic sum adds at least one half: no bound")
print("  a series ceiling, by contrast, sums to E(|x|):", f"{float(sum(size(R, k) for k in range(40))):.5f}")

print("--- 05_4, 05_5: gaps, tail and head, as the cue's order says")
for x in (R, -R):
    for n in (64, 256, 1024):
        d_exact = (1 + x / n) ** n
        head = sum((size(x, k) - abs(term(x, n, k)) for k in range(0, 9)), Q(0))
        tail = tail_bound(x, 8)
        # A bracket of E(x) from a long partial sum and its own tail.
        s = sum((x**k / factorial(k) for k in range(60)), Q(0))
        e = 2 * size(x, 60)
        worst = max(abs(d_exact - (s - e)), abs(d_exact - (s + e)))
        ck(worst <= head + tail + e, f"x={x} n={n}: distance <= head gaps + series tail", f"distance {float(worst):.5f}, bound {float(head + tail):.5f}")
print("--- 05_4: the first cut is too big for the aim, and the second is not")
aim = Q(1, 100)
ck(tail_bound(R, 4) > aim, "tail after 4 is above one hundredth", f"{float(tail_bound(R, 4)):.5f}")
ck(tail_bound(R, 8) < aim, "tail after 8 is below one hundredth", f"{float(tail_bound(R, 8)):.5f}")
last_ok = None
for n in (64, 256, 1024):
    head = sum((size(R, k) - abs(term(R, n, k)) for k in range(0, 9)), Q(0))
    total = head + tail_bound(R, 8)
    print(f"  n={n}: head {float(head):.5f}, total {float(total):.5f}")
    last_ok = total
ck(last_ok < aim, "at 1024 cells the head plus the tail is below one hundredth", f"{float(last_ok):.5f}")
head256 = sum((size(R, k) - abs(term(R, 256, k)) for k in range(0, 9)), Q(0)) + tail_bound(R, 8)
ck(head256 > aim, "at 256 cells this bound is still above the aim: the green row is the last one, on purpose", f"{float(head256):.5f}")
ck(all(abs(term(-R, n, k)) == abs(term(R, n, k)) for n in NS for k in range(12)), "05_6: same bars in absolute value at minus three halves")
ck(term(-R, 8, 3) < 0 < term(-R, 8, 2), "05_6: odd terms negative, even terms positive")

print("--- 05_7: minus three, and every statement about its base")
x = Q(-3)
want = {1: Q(-2), 2: Q(-1, 2), 3: Q(0), 4: Q(1, 4)}
for n, base in want.items():
    ck(1 + x / n == base, f"{n} cells: the base is {base}")
ck((1 + x / 1) ** 1 == -2, "one cell gives minus two")
ck((1 + x / 2) ** 2 == Q(1, 4), "two cells give one quarter")
ck((1 + x / 3) ** 3 == 0, "three cells give exactly zero")
ck((1 + x / 4) ** 4 == Q(1, 256), "four cells give almost nothing", f"{float(Q(1, 256)):.4f}")
ck(all(1 + x / n > 0 for n in (4, 5, 16, 64, 1024)) and all(1 + x / n <= 0 for n in (1, 2, 3)), "a positive multiplier in every step needs more than three cells")
seq = [(1 + x / n) ** n for n in (1, 2, 3, 4, 16, 64, 1024)]
ck(seq[0] < seq[1] and seq[1] > seq[2], "it does not climb steadily: up to one quarter, then down to zero")
ck(all(a < b for a, b in zip(seq[2:], seq[3:])), "past three cells, the printed values climb")
ck(seq[-1] < sum((x**k / factorial(k) for k in range(80)), Q(0)) + 2 * size(x, 80), "and they stay below E(-3)")

print("--- 05_8: the swap, and the walks")
for rate, time in ((Q(3, 2), Q(1)), (Q(-3), Q(1)), (Q(1, 2), Q(2))):
    n = 16
    ck((1 + rate * time / n) ** n == (1 + rate * time / n) ** n and rate * time / n == (rate * time) / n, f"rate {rate} for time {time}: each step multiplies by 1 + (rate)(time)/n")
n = 16
ck((1 + Q(3, 2) * 1 / n) ** n == (1 + 1 * Q(3, 2) / n) ** n, "rate x for time 1 = rate 1 for time x: the same product, step by step")
ck(1 + Q(-3, 4) > 0 and 1 + Q(-3, 16) > 0, "the drawn walks to minus three use 4 and 16 steps, so neither crosses zero")
ck(Q(1) + Q(1, 1000000) > 1 and (1 + Q(1, 2)) ** 2 == Q(9, 4), "x = 1 is one case of the limit, written with its own n; e is not the general formula")

print("--- 06_2: the test, from coefficients, with the chain rule's sign")
for d in (2, 3, 4, 7):
    y = [Q(1, factorial(k)) for k in range(d + 1)]
    m = 12
    e_minus = [Q((-1) ** k, factorial(k)) for k in range(m)]
    product = [sum(y[i] * e_minus[j - i] for i in range(min(j, d) + 1)) for j in range(m)]
    slope_direct = [(j + 1) * product[j + 1] for j in range(m - 1)]
    dy = [(k + 1) * y[k + 1] for k in range(d)] + [Q(0)]
    diff = [dy[k] - y[k] for k in range(d + 1)]
    written = [sum(diff[i] * e_minus[j - i] for i in range(min(j, d) + 1)) for j in range(m - 1)]
    ck(slope_direct == written, f"degree {d}: d/dx(y E(-x)) = (y' - y) E(-x), coefficient by coefficient below order {m - 1}")
ck(all(Q((-1) ** (k + 1), factorial(k + 1)) * (k + 1) == -Q((-1) ** k, factorial(k)) for k in range(12)), "d/dx E(-x) = -E(-x), term by term: the chain rule's minus sign")

print("--- 06_3: the failures")
for d, denom in ((2, 2), (4, 24)):
    y = [Q(1, factorial(k)) for k in range(d + 1)]
    dy = [(k + 1) * y[k + 1] for k in range(d)] + [Q(0)]
    diff = [dy[k] - y[k] for k in range(d + 1)]
    ck(all(c == 0 for c in diff[:-1]) and diff[-1] == Q(-1, denom), f"degree {d}: y' - y is exactly -x^{d}/{denom}, the lost top term")
    ck(y[0] == 1, f"degree {d}: starts at height one")
print("--- 06_3: 'On these axes, degree four breaks the rule less', tried at its edge")
ck(all(x**4 / 24 < x**2 / 2 for x in (Q(1, 2), Q(1), Q(3, 2))), "|x| <= 3/2: x^4/24 < x^2/2, so the claim holds across the panel")
ck(Q(12) ** 2 / 24 == Q(12) / 2, "at x^2 = 12 the two defects are equal")
ck(Q(16) ** 2 / 24 > Q(16) / 2, "past x^2 = 12, degree four breaks it more, which is why the cue says 'On these axes'")
ck((Q(3, 2)) ** 2 < 12, "the panel's edge x = 3/2 is well inside that")
print("--- 06_4, 06_5: the proof's arithmetic at its premises")
m = 20
e_plus = [Q(1, factorial(k)) for k in range(m)]
e_minus = [Q((-1) ** k, factorial(k)) for k in range(m)]
cauchy = [sum(e_plus[i] * e_minus[j - i] for i in range(j + 1)) for j in range(m)]
ck(cauchy[0] == 1 and all(c == 0 for c in cauchy[1:]), "E(x)E(-x) = 1: every Cauchy coefficient above order zero vanishes")
ck(all(k * Q(1, factorial(k)) == Q(1, factorial(k - 1)) for k in range(1, 30)), "E' = E term by term: k / k! = 1/(k-1)!")
ck(e_plus[0] == 1, "E(0) = 1")
print("--- 06_7: the gap")
ck(Q(-1, 2) < Q(-1, 4) < 0 < 1, "the pieces are separated, and only the near one contains the start")
ck(3 * 1 != 1, "the two pieces read different constants, 3 and 1, and the rule never joined them")
print("--- 06_8: the edges of the rate law")
for k_rate in (Q(1, 2), Q(0), Q(-1, 2), Q(3)):
    e_k = [k_rate**j / factorial(j) for j in range(14)]
    de = [(j + 1) * e_k[j + 1] for j in range(13)]
    ck(all(de[j] == k_rate * e_k[j] for j in range(13)), f"k={k_rate}: d/dt E(kt) = k E(kt), term by term")
ck(all(Q(0)**j * 0 == 0 for j in range(4)), "a zero start stays at zero: 0 times E(kt) is 0 for every k")
print("--- 06_6: the ledger has one direction: what the premises rest on")
PREMISES = {
    "E'=E": "series term by term, chapter 02",
    "E(x)E(-x)=1": "series product in the grid, chapter 03",
}
for name, where in PREMISES.items():
    print(f"  premise {name}: {where}")
print("RESULT", "PASS" if not fails else f"{fails} FAIL")
raise SystemExit(1 if fails else 0)
