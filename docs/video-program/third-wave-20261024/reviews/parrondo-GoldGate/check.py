#!/usr/bin/env python3
"""GoldGate's independent checks for the parrondo review (2026-10-24).

Standard library only, no import of the author's code:

    python3 -I -B check.py [ORACLE_STDOUT]

Methods differ on purpose from the author's (model.rs: closed-form spanning
trees; oracle.py: Gaussian elimination on the 3x3 chain and the period
product):

- stationary weights from the constant-net-flow equations around the
  triangle (pi_i p_i - pi_{i+1} q_{i+1} = J on every edge), solved by Cramer's
  rule, then verified exactly (pi P = pi) and against float power iteration;
- spanning-tree weights for K06 by brute-force enumeration of in-trees;
- periodic schedules from the 3L-state (corner, phase) chain, solved
  exactly, plus a float Cesaro average of the expected gain;
- real random crowds by exact binomial convolution;
- horizons by an exact Fraction DP (100 rounds) and a float DP (1,000);
- SplitMix64 re-implemented from its published constants.

With ORACLE_STDOUT (the author's `oracle.py` facts, key<TAB>value), every key
computed here is compared with the author's value. Exit status 1 if any
compared key differs or any internal assertion fails.
"""

import math
import sys
from fractions import Fraction as F
from itertools import product

FAILED = []
MINE = {}


def ok(label, cond):
    print(("ok   " if cond else "FAIL ") + label)
    if not cond:
        FAILED.append(label)


def fixed(x, d):
    """x to d decimals, half away from zero (as a reader rounds)."""
    x = F(x)
    s = -1 if x < 0 else 1
    q = (abs(x) * 10**d * 2 + 1) // 2
    txt = str(q).rjust(d + 1, "0")
    body = txt if d == 0 else txt[:-d] + "." + txt[-d:]
    return ("-" if s < 0 and q != 0 else "") + body


def pct(x, d=1):
    return fixed(F(x) * 100, d)


def frac_list(xs):
    return ",".join(str(F(x)) for x in xs)


# ------------------------------------------------------------------ games

HALF = F(1, 2)
ROUND = (HALF, F(1, 10), F(3, 4))
EPS = F(1, 200)


def coins(eps):
    a, bad, good = (c - eps for c in ROUND)
    return a, bad, good


def game_a(eps):
    a, _, _ = coins(eps)
    return (a, a, a)


def game_b(eps):
    _, bad, good = coins(eps)
    return (bad, good, good)


def mixture(g, h, share_g=HALF):
    return tuple(share_g * x + (1 - share_g) * y for x, y in zip(g, h))


def stationary_flux(p):
    """Weights and the common net flow J of a triangle game (forward
    chances p), from J on every edge plus normalisation."""
    q = [1 - x for x in p]
    a1, b1 = p[0] / q[1], -1 / q[1]
    a2, b2 = a1 * p[1] / q[2], (b1 * p[1] - 1) / q[2]
    # closure: (a2 p2 - q0) pi0 + (b2 p2 - 1) J = 0
    # norm:    (1 + a1 + a2) pi0 + (b1 + b2) J = 1
    m11, m12, r1 = a2 * p[2] - q[0], b2 * p[2] - 1, F(0)
    m21, m22, r2 = 1 + a1 + a2, b1 + b2, F(1)
    det = m11 * m22 - m12 * m21
    pi0 = (r1 * m22 - m12 * r2) / det
    j = (m11 * r2 - r1 * m21) / det
    pi = (pi0, a1 * pi0 + b1 * j, a2 * pi0 + b2 * j)
    return pi, j


def step(v, p):
    """One round: v over corners -> v' (the crowd evolution)."""
    out = [0, 0, 0]
    for i in range(3):
        out[(i + 1) % 3] += v[i] * p[i]
        out[(i - 1) % 3] += v[i] * (1 - p[i])
    return out


def mean_of(p):
    pi, j = stationary_flux(p)
    direct = sum(w * (2 * x - 1) for w, x in zip(pi, p))
    assert direct == 3 * j, "mean must be three times the net flow"
    return direct


def check_stationary(name, p):
    pi, _ = stationary_flux(p)
    ok(f"{name}: weights sum to 1 and pi P = pi exactly", sum(pi) == 1 and step(pi, p) == list(pi))
    v = [1.0, 0.0, 0.0]
    pf = [float(x) for x in p]
    for _ in range(4000):
        v = step(v, pf)
    ok(f"{name}: float power iteration agrees to 1e-12", all(abs(v[i] - float(pi[i])) < 1e-12 for i in range(3)))
    return pi


def in_trees(p):
    """Brute-force spanning in-tree weights of the triangle chain."""
    q = [1 - x for x in p]
    out = []
    for root in range(3):
        others = [i for i in range(3) if i != root]
        total = F(0)
        for choice in product((1, -1), repeat=2):
            nxt = {i: (i + c) % 3 for i, c in zip(others, choice)}
            weight = F(1)
            for i, c in zip(others, choice):
                weight *= p[i] if c == 1 else q[i]
            good = True
            for start in others:
                seen, at = set(), start
                while at != root:
                    if at in seen:
                        good = False
                        break
                    seen.add(at)
                    at = nxt[at]
                if not good:
                    break
            if good:
                total += weight
        out.append(total)
    return out


def products(p):
    fwd = p[0] * p[1] * p[2]
    back = (1 - p[0]) * (1 - p[1]) * (1 - p[2])
    return fwd, back


# ------------------------------------------------------------------ 1. the two regimes

print("== 1. the two handicap regimes (stationary weights by net-flow equations)")
for label, eps in (("round", F(0)), ("pub", EPS)):
    a, b = game_a(eps), game_b(eps)
    mix = mixture(a, b)
    for gname, g in (("a", a), ("b", b), ("mix", mix)):
        pi = check_stationary(f"{label}.{gname}", g)
        m = mean_of(g)
        print(f"   {label}.{gname}: coins {frac_list(g)}  weights {frac_list(pi)} "
              f"({', '.join(pct(w) for w in pi)} %)  mean {m} = {fixed(m * 100, 2)} cents")
        MINE[f"{label}.{gname}.weights"] = frac_list(pi)
        MINE[f"{label}.{gname}.mean"] = str(m)
    # the dial: B wins good - (good - bad) * share0; fair share
    _, bad, good = coins(eps)
    fair = (good - HALF) / (good - bad)
    MINE[f"{label}.b.fair_share"] = str(fair)
    pib, _ = stationary_flux(b)
    pim, _ = stationary_flux(mix)
    b_win = good - (good - bad) * pib[0]
    ok(f"{label}: B's win from the dial equals sum pi_i p_i", b_win == sum(w * x for w, x in zip(pib, b)))
    MINE[f"{label}.b.win"] = str(b_win)
    b_in_mix = good - (good - bad) * pim[0]
    MINE[f"{label}.mix.b_rounds.win"] = str(b_in_mix)
    mix_win = sum(w * x for w, x in zip(pim, mix))
    MINE[f"{label}.mix.win"] = str(mix_win)
    ok(f"{label}: mix win = (A coin + B-in-mix)/2 (game decomposition)", mix_win == (coins(eps)[0] + b_in_mix) / 2)
    die = (bad + 2 * good) / 3
    MINE[f"{label}.die.win"] = str(die)
    print(f"   {label}: B fair share {fair} ({pct(fair)} %), B win {pct(b_win, 2)} %, B rounds in mix "
          f"{pct(b_in_mix, 2)} %, mix win {pct(mix_win, 2)} %, die-chosen {die} ({pct(die)} %)")
    for gname, g in (("a", a), ("b", b), ("mix", mix)):
        fwd, back = products(g)
        MINE[f"{label}.products.{gname}"] = frac_list((fwd, back))
        trees = in_trees(g)
        w = sum(trees)
        pi, j = stationary_flux(g)
        ok(f"{label}.{gname}: in-tree weights / W = flux weights", [t / w for t in trees] == list(pi))
        ok(f"{label}.{gname}: K06 mean = 3 (fwd - back) / W", mean_of(g) == 3 * (fwd - back) / w)
        ok(f"{label}.{gname}: sign rule (mean > 0 iff fwd > back)", (mean_of(g) > 0) == (fwd > back))

round_b, round_mix = game_b(0), mixture(game_a(0), game_b(0))
pub_a, pub_b = game_a(EPS), game_b(EPS)
pub_mix = mixture(pub_a, pub_b)
ok("round B is exactly fair; weights 5/13, 2/13, 6/13", mean_of(round_b) == 0 and stationary_flux(round_b)[0] == (F(5, 13), F(2, 13), F(6, 13)))
ok("round mix mean 18/709 (Ethier & Lee)", mean_of(round_mix) == F(18, 709))
ok("published: A -1/100, B -73443/8446300, mix +223/14200",
   (mean_of(pub_a), mean_of(pub_b), mean_of(pub_mix)) == (F(-1, 100), F(-73443, 8446300), F(223, 14200)))
cost = mean_of(round_mix) - mean_of(pub_mix)
MINE["handicap.mix_cost"] = str(cost)
toward = (F(5, 13) - stationary_flux(round_mix)[0][0]) / (F(5, 13) - F(1, 3))
MINE["round.mix.toward_thirds.pct"] = pct(toward, 0)
print(f"   handicap costs the mix {fixed(cost * 100, 2)} cents; the mix moves {pct(toward, 1)} % of the way to thirds")
halfway = (mean_of(pub_a) + mean_of(pub_b)) / 2
print(f"   halfway between A and B: {fixed(halfway * 100, 2)} cents (the attractive wrong answer)")
thirds = F(1, 3) * coins(EPS)[1] + F(2, 3) * coins(EPS)[2]
print(f"   03_2: published coins with equal thirds win {pct(thirds, 2)} %")

# 10_3's claim, in numbers: where the good coin's wins happen in B's crowd
for label, g in (("round", round_b), ("pub", pub_b)):
    pi, _ = stationary_flux(g)
    at1, at2 = pi[1] * g[1], pi[2] * g[2]
    print(f"   10_3 check ({label} B): good-coin wins at corner 1 vs 2 = {pct(at1 / (at1 + at2))} % vs "
          f"{pct(at2 / (at1 + at2))} %; every win at corner 2 lands on corner 0 (2 -> 0 is forward)")

# random games: K06 and equal net flow on every edge, not just these three
seed = 12345


def lcg():
    global seed
    seed = (seed * 6364136223846793005 + 1442695040888963407) % 2**64
    return seed >> 33


bad_k06 = 0
for _ in range(400):
    g = tuple(F(1 + lcg() % 96, 97) for _ in range(3))
    trees = in_trees(g)
    w = sum(trees)
    pi = [t / w for t in trees]
    flows = [pi[i] * g[i] - pi[(i + 1) % 3] * (1 - g[(i + 1) % 3]) for i in range(3)]
    fwd, back = products(g)
    m = sum(x * (2 * c - 1) for x, c in zip(pi, g))
    if not (flows[0] == flows[1] == flows[2] and m == 3 * flows[0] == 3 * (fwd - back) / w):
        bad_k06 += 1
ok("400 random games: equal net flow on all edges, mean = 3 J = 3 (fwd - back)/W", bad_k06 == 0)

# ------------------------------------------------------------------ 2. crowds

print("== 2. the expected crowd (26) and the mix crowd (1,418)")


def smallest_crowd(g):
    pi, _ = stationary_flux(g)
    n = 1
    while True:
        cells = [n * w * c for w, c in zip(pi, g)] + [n * w * (1 - c) for w, c in zip(pi, g)]
        if all(x.denominator == 1 for x in cells):
            return n
        n += 1


for key, g in (("crowd", round_b), ("mixcrowd", round_mix)):
    n = smallest_crowd(g)
    pi, j = stationary_flux(g)
    counts = [n * w for w in pi]
    fwd = [c * x for c, x in zip(counts, g)]
    back = [c * (1 - x) for c, x in zip(counts, g)]
    arrive = step(counts, g)
    nets = [fwd[i] - back[(i + 1) % 3] for i in range(3)]
    MINE[f"{key}.size"] = str(n)
    MINE[f"{key}.counts"] = frac_list(counts)
    MINE[f"{key}.forward"] = frac_list(fwd)
    MINE[f"{key}.back"] = frac_list(back)
    print(f"   {key}: smallest whole crowd {n}; counts {frac_list(counts)}; forward {frac_list(fwd)}; "
          f"back {frac_list(back)}; arrivals {frac_list(arrive)}; net per edge {frac_list(nets)}; "
          f"winners {sum(fwd)}, losers {sum(back)}")
    ok(f"{key}: one round leaves the counts unchanged and conserves the total", arrive == counts and sum(arrive) == n)
    if key == "crowd":
        MINE["crowd.winners"] = str(sum(fwd))
    else:
        MINE["mixcrowd.net"] = frac_list(nets)
        MINE["mixcrowd.gain"] = str(sum(nets))
        MINE["mixcrowd.winners"] = str(sum(fwd))
        MINE["mixcrowd.losers"] = str(sum(back))

v = [F(26), F(0), F(0)]
traj = []
for k in range(1, 401):
    v = step(v, round_b)
    ok_total = sum(v) == 26
    if not ok_total:
        ok(f"total stays 26 at round {k}", False)
    traj.append(v)
for k in range(1, 7):
    MINE[f"crowd.round.{k}"] = ",".join(fixed(x, 1) for x in traj[k - 1])
target = [10, 4, 12]
far = [k for k in range(1, 401) if any(abs(x - t) >= HALF for x, t in zip(traj[k - 1], target))]
MINE["crowd.within_half_from"] = str(far[-1] + 1)
print("   expected crowd from (26, 0, 0) under round B, rounds 1-6: "
      + "; ".join(MINE[f"crowd.round.{k}"] for k in range(1, 7)))
print(f"   within half a player of (10, 4, 12) at every corner from round {far[-1] + 1} on (checked to round 400)")
share0 = [float(x[0] / 26) for x in traj[:10]]
print("   share at corner 0, rounds 1-10: " + ", ".join(f"{100 * s:.1f}" for s in share0))


def transient_eigs(g):
    """The two eigenvalues besides 1: sum -1 (no self-moves), product
    fwd + back (the determinant of the cycle matrix)."""
    fwd, back = products(g)
    disc = 1 - 4 * float(fwd + back)
    if disc >= 0:
        r = math.sqrt(disc)
        return ((-1 + r) / 2, (-1 - r) / 2)
    return ("complex", math.sqrt(float(fwd + back)))


for name, g in (("round B", round_b), ("round mix", round_mix), ("pub B", pub_b), ("pub mix", pub_mix)):
    e = transient_eigs(g)
    assert e[0] != "complex", name
    print(f"   {name}: transient eigenvalues {e[0]:.4f}, {e[1]:.4f} (negative: the crowd sloshes, flipping sign each round)")

print("   11_5 check, the mix started from B's settled crowd (10, 4, 12), round coins, shares %:")
v = [F(10), F(4), F(12)]
rows = []
for k in range(0, 9):
    rows.append((k, [pct(x / 26) for x in v]))
    v = step(v, round_mix)
for k, s in rows:
    print(f"     round {k}: corner 0 {s[0]}, corner 1 {s[1]}, corner 2 {s[2]}")
first = step([F(10), F(4), F(12)], round_mix)
ok("11_5: after ONE mix round corner 0 is already 9/26 = 34.6 % and corner 1 overshoots to 7.5/26 = 28.8 %",
   first == [F(9), F(15, 2), F(19, 2)])

# ------------------------------------------------------------------ 3. real random players

print("== 3. 26 real random players at (10, 4, 12), round B, one round")


def binom(n, p):
    return [math.comb(n, k) * p**k * (1 - p) ** (n - k) for k in range(n + 1)]


x0, x1, x2 = binom(10, F(1, 10)), binom(4, F(3, 4)), binom(12, F(3, 4))
# arrivals: c0 = (4 - X1) + X2, c1 = X0 + (12 - X2), c2 = (10 - X0) + X1
stay = sum(x0[a] * x1[a + 2] * x2[a + 8] for a in range(0, 3) if a + 2 <= 4 and a + 8 <= 12)
exact_split = x0[1] * x1[3] * x2[9]
c0 = {}
winners = {}
for a, b, c in product(range(11), range(5), range(13)):
    pr = x0[a] * x1[b] * x2[c]
    c0[4 - b + c] = c0.get(4 - b + c, 0) + pr
    winners[a + b + c] = winners.get(a + b + c, 0) + pr
ok("joint distribution sums to 1", sum(c0.values()) == 1)
print(f"   P(the counts come back exactly 10, 4, 12)        = {float(stay):.4f}  ({stay})")
print(f"   P(the six flows are exactly 1/9, 3/1, 9/3)       = {float(exact_split):.4f}")
print(f"   P(exactly 13 winners)                            = {float(winners[13]):.4f}")
print(f"   P(exactly 1 of the 10 at corner 0 wins)          = {float(x0[1]):.4f}")
m0 = sum(k * p for k, p in c0.items())
sd0 = math.sqrt(float(sum((k - m0) ** 2 * p for k, p in c0.items())))
print(f"   corner 0 after the round: mean {m0}, standard deviation {sd0:.2f} players, "
      f"P(9..11) = {float(sum(c0.get(k, 0) for k in (9, 10, 11))):.3f}")
pi_b = (F(5, 13), F(2, 13), F(6, 13))
multi = F(math.factorial(26), math.factorial(10) * math.factorial(4) * math.factorial(12)) \
    * pi_b[0]**10 * pi_b[1]**4 * pi_b[2]**12
sd_st = math.sqrt(26 * 5 / 13 * 8 / 13)
print(f"   long-run (stationary) crowd of 26 independent players: P(exactly 10, 4, 12) = {float(multi):.4f}; "
      f"corner-0 count 10 +/- {sd_st:.2f} (binomial 26, 5/13)")
ok("a real crowd keeps 10, 4, 12 for one round less than one time in ten", stay < F(1, 10))

M64 = (1 << 64) - 1


class SplitMix64:
    def __init__(self, seed):
        self.x = seed & M64

    def next(self):
        self.x = (self.x + 0x9E3779B97F4A7C15) & M64
        z = self.x
        z = ((z ^ (z >> 30)) * 0xBF58476D1CE4E5B9) & M64
        z = ((z ^ (z >> 27)) * 0x94D049BB133111EB) & M64
        return z ^ (z >> 31)


gen = SplitMix64(26)
corners = [0] * 26
seen = []
for k in range(1, 31):
    for i in range(26):
        p = round_b[corners[i]]
        win = gen.next() % p.denominator < p.numerator
        corners[i] = (corners[i] + (1 if win else -1)) % 3
    if k >= 24:
        seen.append((k, [corners.count(c) for c in range(3)]))
print("   illustration (one seeded draw, SplitMix64 seed 26): 26 real players from corner 0, counts at rounds 24-30: "
      + "; ".join(f"{k}: {c}" for k, c in seen))

# ------------------------------------------------------------------ 4. schedules

print("== 4. mixture vs periodic schedules ((corner, phase) chain, exact)")


def solve_stationary(P):
    """Exact left null vector of a stochastic matrix (Fraction Gaussian
    elimination on (P - I)^T with a row of ones)."""
    n = len(P)
    A = [[P[j][i] - (1 if i == j else 0) for j in range(n)] for i in range(n)]
    A[-1] = [F(1)] * n
    b = [F(0)] * (n - 1) + [F(1)]
    M = [row[:] + [b[i]] for i, row in enumerate(A)]
    for c in range(n):
        piv = next(r for r in range(c, n) if M[r][c] != 0)
        M[c], M[piv] = M[piv], M[c]
        for r in range(n):
            if r != c and M[r][c] != 0:
                f = M[r][c] / M[c][c]
                M[r] = [x - f * y for x, y in zip(M[r], M[c])]
    return [M[i][n] / M[i][i] for i in range(n)]


def schedule_mean(games):
    L = len(games)
    n = 3 * L
    P = [[F(0)] * n for _ in range(n)]
    for k, g in enumerate(games):
        for i in range(3):
            s = 3 * k + i
            P[s][3 * ((k + 1) % L) + (i + 1) % 3] += g[i]
            P[s][3 * ((k + 1) % L) + (i - 1) % 3] += 1 - g[i]
    pi = solve_stationary(P)
    ok_sum = sum(pi) == 1
    assert ok_sum
    return sum(pi[3 * k + i] * (2 * g[i] - 1) for k, g in enumerate(games) for i in range(3))


def cesaro(games, rounds):
    v = [1.0, 0.0, 0.0]
    gain = 0.0
    for t in range(rounds):
        g = [float(x) for x in games[t % len(games)]]
        gain += sum(v[i] * (2 * g[i] - 1) for i in range(3))
        v = step(v, g)
    return gain / rounds


def pattern(s, eps):
    return [game_a(eps) if c == "A" else game_b(eps) for c in s]


for s, want in (("AB", F(0)), ("AABB", F(4, 163)), ("ABB", F(2416, 35601)), ("AAB", F(32, 1609))):
    got = schedule_mean(pattern(s, F(0)))
    MINE[f"round.sched.{s.lower()}"] = str(got)
    ok(f"round coins, {s} per round played = {want} (Ethier & Lee pin)", got == want)
for s in ("AB", "AABB"):
    got = schedule_mean(pattern(s, EPS))
    MINE[f"pub.sched.{s.lower()}"] = str(got)
    ces = cesaro(pattern(s, EPS), 200000)
    ok(f"published {s}: exact {fixed(got * 100, 3)} cents; float Cesaro average over 200,000 rounds agrees to 1e-5",
       abs(ces - float(got)) < 1e-5)
print("   AB at eps = 0 over rho (A fair; B p0 = rho^2/(1+rho^2), p1 = p2 = 1/(1+rho)):")
all_zero = True
for rho in (F(1, 5), F(1, 4), F(1, 3), F(2, 5), F(1, 2), F(2, 3), F(3, 4), F(3, 2), F(2), F(3)):
    b = (rho**2 / (1 + rho**2), 1 / (1 + rho), 1 / (1 + rho))
    a = (HALF, HALF, HALF)
    ab, abb, aabb = schedule_mean([a, b]), schedule_mean([a, b, b]), schedule_mean([a, a, b, b])
    all_zero &= ab == 0
    print(f"     rho {str(rho):>4}: AB {str(ab):>3}   ABB {float(abb):+.5f}   AABB {float(aabb):+.5f}")
ok("AB is exactly fair at eps = 0 for every rho tried (10 values)", all_zero)
tiny = F(1, 10**6)
slope = schedule_mean(pattern("AB", tiny)) / tiny
ok(f"AB slope at eps -> 0: {float(slope):.5f} vs -228/169 = {float(F(-228, 169)):.5f}", abs(slope - F(-228, 169)) < F(1, 10**4))
MINE["pub.mix.mean"] = str(mean_of(pub_mix))

print("   the share of A in the random mix (published coins):")


def mix_mean(share_a):
    return mean_of(mixture(pub_a, pub_b, share_a))


def bisect(lo, hi, n=40):
    up = mix_mean(lo) > 0
    for _ in range(n):
        mid = (lo + hi) / 2
        if (mix_mean(mid) > 0) == up:
            lo = mid
        else:
            hi = mid
    return (lo + hi) / 2


lo_x, hi_x = bisect(F(0), F(1, 2)), bisect(F(1, 2), F(1))
MINE["pub.share.low.pct"] = pct(lo_x)
MINE["pub.share.high.pct"] = pct(hi_x)
MINE["pub.share.1_20.cents"] = fixed(mix_mean(F(1, 20)) * 100, 2)
MINE["pub.share.9_10.cents"] = fixed(mix_mean(F(9, 10)) * 100, 2)
print(f"     crossings {float(lo_x) * 100:.3f} % and {float(hi_x) * 100:.3f} %; at 5 % {fixed(mix_mean(F(1, 20)) * 100, 3)} "
      f"cents, at 90 % {fixed(mix_mean(F(9, 10)) * 100, 3)} cents; at 50 % {fixed(mix_mean(HALF) * 100, 3)}")

# ------------------------------------------------------------------ 5. finite play

print("== 5. finite horizon (exact Fraction DP to 100 rounds; float DP to 1,000)")


def horizon_exact(g, rounds, start):
    dist = {start: F(1)}
    for _ in range(rounds):
        nxt = {}
        for x, w in dist.items():
            p = g[x % 3]
            nxt[x + 1] = nxt.get(x + 1, 0) + w * p
            nxt[x - 1] = nxt.get(x - 1, 0) + w * (1 - p)
        dist = nxt
    ahead = sum(w for x, w in dist.items() if x > start)
    behind = sum(w for x, w in dist.items() if x < start)
    return ahead, behind, dist.get(start, F(0))


def horizon_float(g, rounds, start):
    pf = [float(x) for x in g]
    off = rounds
    dist = [0.0] * (2 * rounds + 1)
    dist[off] = 1.0
    for _ in range(rounds):
        nxt = [0.0] * len(dist)
        for i, w in enumerate(dist):
            if w:
                p = pf[(start - off + i) % 3]
                nxt[i + 1] += w * p
                nxt[i - 1] += w * (1 - p)
        dist = nxt
    return math.fsum(dist[off + 1:]), math.fsum(dist[:off]), dist[off]


h = horizon_exact(pub_mix, 100, 0)
ok("100-round exact distribution sums to 1", sum(h) == 1)
MINE["horizon.short.mix"] = ",".join(pct(x) for x in h)
print(f"   mix, 100 rounds from 0 dollars: ahead {float(h[0]) * 100:.3f} %, behind {float(h[1]) * 100:.3f} %, level {float(h[2]) * 100:.3f} %")
h7 = horizon_exact(pub_mix, 100, 7)
print(f"   (same from 7 dollars, the player's start: ahead {float(h7[0]) * 100:.3f}, behind {float(h7[1]) * 100:.3f}, level {float(h7[2]) * 100:.3f})")
for name, g, which in (("mix", pub_mix, 0), ("a", pub_a, 1), ("b", pub_b, 1)):
    r0 = horizon_float(g, 1000, 0)
    r7 = horizon_float(g, 1000, 7)
    word = "ahead" if which == 0 else "behind"
    MINE[f"horizon.long.{name}.{word}.pct"] = pct(F(r0[which]).limit_denominator(10**12))
    print(f"   {name}, 1,000 rounds: {word} {r0[which] * 100:.3f} % from 0 dollars, {r7[which] * 100:.3f} % from 7")

# ------------------------------------------------------------------ 6. the seeded player

print("== 6. the seeded player (SplitMix64 re-implemented; contract: seed 1999, start 7, "
      "mix draws its deciding coin first, even = A, toss = draw mod 200 < chance in 200ths)")


def run(play, rounds=100000, seed=1999, start=7):
    gen = SplitMix64(seed)
    a = [int(x * 200) for x in pub_a]
    b = [int(x * 200) for x in pub_b]
    now, at0, path = start, 0, [start]
    for _ in range(rounds):
        if play == "A":
            g = a
        elif play == "B":
            g = b
        else:
            g = a if gen.next() % 2 == 0 else b
        r = now % 3
        at0 += r == 0
        now += 1 if gen.next() % 200 < g[r] else -1
        path.append(now)
    return path, at0


runs = {}
for play in ("A", "B", "Mix"):
    path, at0 = run(play)
    runs[play] = path
    k = play.lower()
    MINE[f"run.{k}.net"] = str(path[-1] - 7)
    MINE[f"run.{k}.at0.rounds"] = str(at0)
    MINE[f"run.{k}.low"] = str(min(path))
    MINE[f"run.{k}.high"] = str(max(path))
    print(f"   {play}: net {path[-1] - 7:+d}, rounds at a multiple of 3 {at0} ({at0 / 1000:.2f} %), low {min(path)}, high {max(path)}")
MINE["run.a.first"] = ",".join(str(x) for x in runs["A"][:9])
MINE["run.b.net.short"] = str(runs["B"][1000] - 7)
print(f"   A's first nine values {MINE['run.a.first']}; B after 1,000 rounds {runs['B'][1000] - 7:+d}")

# ------------------------------------------------------------------ 7. closing puzzle

print("== 7. the closing puzzle (switch by the count)")
_, bad, good = coins(EPS)
a_coin = coins(EPS)[0]
reverse = (bad, a_coin, a_coin)
swapped = (a_coin, good, good)
MINE["pub.reverse.mean"] = str(mean_of(reverse))
MINE["pub.swapped.mean"] = str(mean_of(swapped))
print(f"   B at 0, A elsewhere: {mean_of(reverse)} = {fixed(mean_of(reverse) * 100, 2)} cents; "
      f"A at 0, B elsewhere: {mean_of(swapped)} = {fixed(mean_of(swapped) * 100, 2)} cents")
ok("18_6: every coin of the reverse rule is below one half", all(c < HALF for c in reverse))

# ------------------------------------------------------------------ 8. opening layout and timing arithmetic

print("== 8. chapter 01 arithmetic from the_puzzle.rs constants (no frame was rendered or seen)")
# the_puzzle.rs:119-125 LONG window; 4K frame is 8 scene units tall (FRAME_Y_RADIUS 4) on 2160 px
cx, cy, W, H = 0.8, 0.15, 10.6, 4.0
px = 2160 / 8.0
left = cx - W / 2
unit_y = H / 1400.0
a1000 = runs["A"][:1001]
lo, hi = min(a1000), max(a1000)
short_x = left + W * 1000 / 100000
box_w = max(short_x - left, 0.3) + 0.2
box_h = (hi - lo) * unit_y + 0.3
print(f"   first 1,000 rounds of A: dollars {lo} to {hi}; drawn {short_x - left:.3f} units wide "
      f"({(short_x - left) * px:.0f} px at 4K) and {(hi - lo) * unit_y:.3f} units tall ({(hi - lo) * unit_y * px:.0f} px)")
print(f"   the 01_10 outline is {box_w:.2f} x {box_h:.2f} units; the run inside it fills "
      f"{100 * (short_x - left) / box_w:.0f} % of its width and {100 * (hi - lo) * unit_y / box_h:.0f} % of its height")
print(f"   01_5's climb (7 -> 12, five dollars) on the long plot: {5 * unit_y:.4f} units = {5 * unit_y * px:.1f} px at 4K")

# 01_9 phrase timing with the kit's scratch voice (SCRATCH_WPS 2.6; starts by character share)
line = ("Here's one simulated player, playing game A for a hundred thousand rounds. The path wobbles, but sinks, "
        "ending about a thousand dollars below where it started.")
speech = len(line.split()) / 2.6
length = speech + 0.4
s2 = line.index("The path")
starts = [0.0, speech * (s2 - 1) / (len(line) - 1)]  # kit: shares of the trimmed sentences' characters


def at(phrase):
    i = line.index(phrase)
    j = 0 if i < s2 else 1
    a0, t0 = (0, starts[0]) if j == 0 else (s2, starts[1])
    a1, t1 = (s2, starts[1]) if j == 0 else (len(line), length)
    return t0 + (t1 - t0) * (i - a0) / (a1 - a0)


t_run = max(1.8, at("for a hundred thousand rounds")) + 0.1
t_end = at("ending about")
frac = (t_end - t_run) / 7.0
k = int(round(frac * 100000))
print(f"   01_9 (scratch estimate): line {length:.1f} s; draw from {t_run:.1f} s to {t_run + 7:.1f} s; "
      f"'ending about' at {t_end:.1f} s, when {100 * frac:.0f} % is drawn and the token reads about "
      f"{runs['A'][k]:+d} dollars; the end readout fades in after {t_run + 7:.1f} s")

# ------------------------------------------------------------------ compare with the author's oracle

if len(sys.argv) > 1:
    print("== comparison with the author's oracle stdout")
    theirs = {}
    with open(sys.argv[1], encoding="utf-8") as fh:
        for row in fh:
            if "\t" in row:
                key, val = row.rstrip("\n").split("\t", 1)
                theirs[key] = val
    same = differ = absent = 0
    for key in sorted(MINE):
        if key not in theirs:
            absent += 1
            continue
        a, b = MINE[key].replace("+", ""), theirs[key].replace("+", "")
        if a == b:
            same += 1
        else:
            differ += 1
            print(f"DIFF {key}: mine {MINE[key]!r}, theirs {theirs[key]!r}")
    print(f"   {same} keys agree, {differ} differ; {absent} of mine have no oracle key")
    if differ:
        FAILED.append("oracle comparison")

print(f"== {'all checks passed' if not FAILED else str(len(FAILED)) + ' FAILED: ' + '; '.join(FAILED)}")
sys.exit(1 if FAILED else 0)
