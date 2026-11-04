"""Independent exact checks of what sections A-C say and print. Python standard library only; no import from staged code.

Usage:  python3 -I -B math_exact.py
The derivations and code are my own. I had seen the earlier review's check output first, so this is independent
in code and route, not blind to its conclusions. Each check states the claim in the cue's or the screen's words.
Edges of every always/never claim are tested.
"""
from fractions import Fraction as Q
from math import comb, exp, factorial, floor
import random

fails = 0


def ck(ok, claim, extra=""):
    global fails
    print(("ok   " if ok else "FAIL ") + claim + (f"  [{extra}]" if extra else ""))
    fails += 0 if ok else 1


def poly_at(vertices, t):
    """Piecewise-linear interpolation through exact (t, y) vertices."""
    for (a, ya), (b, yb) in zip(vertices, vertices[1:]):
        if a <= t <= b:
            return ya + (yb - ya) * (t - a) / (b - a)
    raise ValueError(t)


def euler(n):
    return [(Q(k, n), (1 + Q(1, n)) ** k) for k in range(n + 1)]


print("--- A. rate law")
p4, p8, p16 = euler(4), euler(8), euler(16)
line = lambda t: 1 + t
ck(p4[1][1] == line(p4[1][0]) == Q(5, 4), "01_2: both first steps land at the same height", "5/4")
ck(p4[1][1] > 1, "01_2: the right rule's next slope is steeper: slope = height 5/4 > 1")
ck(all(p4[k + 1][1] - p4[k][1] > p4[k][1] - p4[k - 1][1] for k in range(1, 4)), "01_3: the right drawing keeps steepening (each rise exceeds the last)")
ck(all(p4[k][1] - line(p4[k][0]) > 0 for k in range(2, 5)), "01_3: after the first step the two drawings separate, right above left")
grid = [Q(k, 16) for k in range(17)]
# Every drawing's first step has slope 1, so a finer drawing lies ON the coarser one until its first step ends.
# My first version asserted "strictly above at every t>0"; that is false and was my error, not the cue's.
ck(all(poly_at(p8, t) == poly_at(p4, t) for t in grid if t <= Q(1, 8)) and all(poly_at(p8, t) > poly_at(p4, t) for t in grid if t > Q(1, 8)),
   "01_4: 'the new one climbs slightly higher': 8 steps equal 4 up to t=1/8, then strictly above")
ck(all(poly_at(p16, t) == poly_at(p8, t) for t in grid if t <= Q(1, 16)) and all(poly_at(p16, t) > poly_at(p8, t) for t in grid if t > Q(1, 16)),
   "01_4: 16 steps equal 8 up to t=1/16, then strictly above")
g48 = max(poly_at(p8, t) - poly_at(p4, t) for t in grid)
g816 = max(poly_at(p16, t) - poly_at(p8, t) for t in grid)
ck(g816 < g48, "01_4: 'brings the new drawings closer together': largest gap shrinks", f"{float(g48):.4f} then {float(g816):.4f}")
fine = [Q(k, 64) for k in range(65)]
ck(all(float(poly_at(p16, t)) < exp(float(t)) + 1e-12 for t in fine), "A: every stepped drawing stays under the true curve (they only approach it from below)")
ck(float(p16[-1][1]) < exp(1) < 3, "A: fixed axis top 3 holds the 16-step endpoint and e", f"{float(p16[-1][1]):.6f} < {exp(1):.6f}")
for label, value, want in (("height after step 2", p4[2][1], "1.56"), ("after step 3", p4[3][1], "1.95"), ("after step 4", p4[4][1], "2.44")):
    ck(f"{float(value):.2f}" == want, f"01_3 read-out {label} prints {want}", f"exact {value} = {float(value)}")
for n, want in ((4, "2.441"), (8, "2.566"), (16, "2.638")):
    ck(f"{float(euler(n)[-1][1]):.3f}" == want, f"01_4 endpoint for {n} steps prints {want}", f"{float(euler(n)[-1][1]):.6f}")
ck("last step" and f"{float(p4[3][1]):.2f}" == "1.95", "01_3: last step's slope is the height before it: 1.95, then height 2.44")
# the tie that could separate Rust and Python
ck(float(euler(8)[1][1]) == 1.125, "facts: rate.8.1 is an exact binary tie at 1.125", "round-half-even prints 1.12 in Rust and Python; that row is exported, never drawn")

print("--- B. coefficients")
a = [Q(1)]
for k in range(30):
    a.append(a[k] / (k + 1))  # coefficient of x^k in f' is (k+1)a[k+1]; f' = f forces (k+1)a[k+1] = a[k]
ck(all(a[k] == Q(1, factorial(k)) for k in range(31)), "02_3: previous over next whole number gives 1/k! through degree 30")
ck(a[:5] == [Q(1), Q(1), Q(1, 2), Q(1, 6), Q(1, 24)], "02_3: one, one, one half, one sixth, one twenty-fourth")
ck(2 * a[2] == a[1] and 3 * a[3] == a[2] and 4 * a[4] == a[3], "02_2/table: 2b=a, 3c=b, 4d=c, the shift the derivative row draws")
deriv = [(k + 1) * a[k + 1] for k in range(4)]
ck(deriv == a[:4] and len(deriv) == 4, "02_4: a degree-4 truncation's derivative matches through x^3 and has no x^4 term, so the top power is lost")
ck(a[4] != 0, "02_4: 'cannot finish the job': the top coefficient of f is nonzero while f' has none there")
d_series = [(k + 1) * a[k + 1] for k in range(30)]
ck(d_series == a[:30], "02_5: the derivative series IS the same series re-indexed, so 'the same argument works' is exact")


def tail_true(R, N, terms=200):
    """Exact sum of R^k/k! for k = N+1 to N+terms, a lower estimate of the tail; adequate to 40+ places here."""
    s = Q(0)
    t = Q(R) ** (N + 1) / factorial(N + 1)
    for k in range(N + 1, N + terms + 1):
        s += t
        t = t * R / (k + 1)
    return s


def bound(R, N):
    R = Q(R)
    return R ** (N + 1) / factorial(N + 1) / (1 - R / (N + 2))


print("--- B. tail bound, TEX[1] and TEX[2]")
R, N = Q(3, 2), 4
ck(R / (N + 2) == Q(1, 4), "read-out: ratio is 1/4 at R=3/2, N=4")
ck(bound(R, N) == Q(27, 320), "TEX[2] at R=3/2, N=4 is 27/320 = 0.084375 exactly")
ck(Q(floor(bound(R, N) * 100000) + 1, 100000) == Q(8438, 100000), "read-out: rounded up at five places, 0.08438")
sup = tail_true(R, N)
ck(sup < bound(R, N) < Q(8438, 100000), "the printed 0.08438 bounds the true tail at the worst x", f"tail {float(sup):.6f} < 0.084375 < 0.08438")
ck(abs(float(sup) - (exp(1.5) - sum(1.5 ** k / factorial(k) for k in range(5)))) < 1e-12, "that tail is e^1.5 minus the degree-4 sum at x=R")
rng = random.Random(20261104)
bad = 0
for _ in range(400):
    Rt = Q(rng.randint(1, 40), rng.randint(1, 8))
    Nt = rng.randint(0, 25)
    if Nt + 2 > Rt and tail_true(Rt, Nt, 120) > bound(Rt, Nt):
        bad += 1
ck(bad == 0, "TEX[2] holds on 400 random (R, N) with N+2 > R, exact arithmetic")
ck(all(Q(5, 1) / (k + 1) <= Q(1, 2) for k in range(9, 40)) and Q(5, 1) / 9 > Q(1, 2), "TEX[1] edge: R=5: holds exactly from k+1 >= 2R = 10, fails at k+1 = 9")
ck(Q(5) / 10 == Q(1, 2), "TEX[1] edge: equality at k+1 = 2R, so the printed non-strict sign is right")
ck(bound(Q(10), 4) < 0, "TEX[2] edge: without N+2 > R the formula goes negative, so the printed hypothesis is needed", f"{float(bound(Q(10), 4)):.1f}")
# x<0 on the same interval: the bound must still hold; alternating signs make the error smaller
xs = [Q(k, 8) for k in range(-12, 13)]
def err(x, N, terms=120):
    t = Q(x) ** (N + 1) / factorial(N + 1)
    s = Q(0)
    for k in range(N + 1, N + terms):
        s += t
        t = t * x / (k + 1)
    return abs(s)
ck(all(err(x, 4) <= sup for x in xs) and max(err(x, 4) for x in xs) == err(Q(3, 2), 4), "uniform on |x|<=3/2: the worst sampled |x| is at x=+R, negatives are smaller")

print("--- B. chart")
vals = [[sum(Q(j, 40) ** k / factorial(k) for k in range(n + 1)) for j in range(61)] for n in range(5)]
ck(all(0 <= y <= 5 for row in vals for y in row), "02_3: all five partial sums stay inside the fixed 0..5 axis", f"highest {float(vals[4][-1]):.6f}")
ck(all(vals[n + 1][j] > vals[n][j] for n in range(4) for j in range(1, 61)), "02_3: each new curve lies above the last for x>0, so the dimmed curve stays visible")
ck(float(vals[4][-1]) < exp(1.5) < 5, "02_3: e^1.5 also fits, so the top label does not lie about the limit")
ck(max(vals[4][j] - vals[3][j] for j in range(61)) == vals[4][60] - vals[3][60], "02_3: the last visible change is largest at the right end", f"{float(vals[4][60] - vals[3][60]):.4f} in y, about {float(vals[4][60] - vals[3][60]) * 0.8:.3f} scene units")

print("--- C. product grid, cell by cell")
cells = {(i, j): factorial(i) * factorial(j) for i in range(4) for j in range(4)}
want = [[1, 1, 2, 6], [1, 1, 2, 6], [2, 2, 4, 12], [6, 6, 12, 36]]
ck(all(cells[(i, j)] == want[j][i] for i in range(4) for j in range(4)), "03_1: sixteen denominators, read by hand from x^i y^j / (i! j!)")
ck(len(cells) == 16, "03_1: 'sixteen products'")
for m in range(4):
    in_grid = [(i, j) for (i, j) in cells if i + j == m]
    ck(len(in_grid) == m + 1, f"03_2/03_4: degree {m} is complete in the 4x4 corner", f"{m + 1} squares")
for m in range(4, 7):
    in_grid = [(i, j) for (i, j) in cells if i + j == m]
    ck(len(in_grid) < m + 1, f"03_4: degree {m} is NOT complete in the corner, so 'through degree three' is the exact edge", f"{len(in_grid)} of {m + 1}")
for m in range(0, 25):
    ok = all(Q(1, factorial(k) * factorial(m - k)) == Q(comb(m, k), factorial(m)) for k in range(m + 1))
    if not ok:
        ck(False, f"diagonal identity at m={m}")
ck(True, "03_4/TEX[3]: m!/(k!(m-k)!) is C(m,k) coefficient by coefficient through m=24")


def diag(x, y, m):
    return sum((Q(x) ** k * Q(y) ** (m - k) / (factorial(k) * factorial(m - k)) for k in range(m + 1)), Q(0))


def E(x, terms=80):
    s, t = Q(0), Q(1)
    for k in range(terms):
        s += t
        t = t * x / (k + 1)
    return s


pts = [Q(-3, 2), Q(-1, 3), Q(0), Q(2, 7), Q(5, 4)]
ck(all(diag(x, y, m) == (Q(x) + y) ** m / factorial(m) for x in pts for y in pts for m in range(13)), "TEX[7]: the diagonal sums to (x+y)^m/m! exactly, degrees 0..12, 25 rational pairs")
ck(all(x ** 2 / 2 + x * y + y ** 2 / 2 == Q(1, 2) * (x ** 2 + 2 * x * y + y ** 2) == (x + y) ** 2 / 2 for x in pts for y in pts), "TEX[0]=TEX[1]=TEX[2], the three forms 03_3 morphs through")
ck(all(abs(E(x) * E(y) - E(x + y)) < Q(1, 10 ** 40) for x in pts for y in pts), "TEX[4]: E(x)E(y)=E(x+y) to 40 places on rational pairs")
ck(all(abs(E(x) * E(-x) - 1) < Q(1, 10 ** 40) for x in pts), "TEX[5]: E(x)E(-x)=E(0)=1")
ck(all(abs(E(x) - E(x / 2) ** 2) < Q(1, 10 ** 40) for x in pts), "TEX[10]: E(x)=E(x/2)^2")
ck(all(E(x) > 0 for x in (Q(-20), Q(-5), Q(-1), Q(0), Q(1))), "TEX[6]: E(x)>0, including x=-20 where a low partial sum would mislead", "exact sum of 80 terms")
s4 = sum(Q(-5) ** k / Q(factorial(k)) for k in range(5))
s5 = sum(Q(-5) ** k / Q(factorial(k)) for k in range(6))
ck(s5 < 0 < s4, "edge: at x=-5 the partial sums change sign, so positivity is a property of the sum, not of any partial sum", f"S4={float(s4):.2f}, S5={float(s5):.2f}")
ck(E(Q(1, 2)) > E(Q(1, 3)) > E(Q(0)) > E(Q(-1, 2)), "03_6: increasing on sampled points")
print("RESULT", "PASS" if not fails else f"{fails} FAIL")
