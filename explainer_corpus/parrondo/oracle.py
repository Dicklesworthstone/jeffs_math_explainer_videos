#!/usr/bin/env python3
"""Facts oracle for "Two Losing Games Can Win — Here Is the Catch".

Run from this folder (`./x parrondo facts-check` does):

    python3 -I -B oracle.py

stdout: one `KEY<TAB>VALUE` line per fact, the same keys `parrondo facts`
prints (src/facts.rs).

stderr: cross-checks (lines starting with "#") and a summary; exit 1 if any
cross-check failed.

How the values are obtained, independently of the Rust model (which writes
every coin as a whole number of 200ths, finds a game's long-run shares by the
spanning-tree formula and its average per round as three times the net flow,
3 (p0 p1 p2 − q0 q1 q2) / W):
  * long-run shares by solving π P = π, Σ π = 1 with Gaussian elimination
    over fractions.Fraction, and the average per round as Σ π_i (2 p_i − 1);
  * a periodic schedule (AB, AABB, ABB, AAB) as the product of its one-round
    matrices, solved the same way, with the per-round average taken phase by
    phase as the shares move through the period;
  * the smallest whole-number crowds (26 for B, 1,418 for the mix) by trying
    every size from 1 up, not by a least common multiple;
  * the crowd's first rounds and its settling exactly, round by round;
  * the dollars after 100 and 1,000 rounds by an exact dynamic programme on
    integer weights (every chance is a whole number of 200ths), not floats;
  * where the mix's A share stops winning by exact bisection on the sign of
    the Gaussian-elimination average, not on the two products;
  * the one simulated player by its own SplitMix64 (seed 1999, fixed before
    any run was looked at), the generator being a contract both sides share;
  * the sign rule, the equal net flow on every edge and the spanning-tree
    shares checked on all 729 games whose three chances are tenths 1..9.

Rounding everywhere: half away from zero, exactly, on fractions (the crowd's
second round has 5.85 players at corner 1, shown as 5.9).

Assumptions (transparent; the video states them): capital is any whole
number, negatives allowed, play never stops; "divides by three" means
remainder 0 with Python's floor remainder (−1 leaves 2); the mix is a fair
coin choosing A or B independently before every round; "wins" means a
positive long-run average per round.
"""
import sys
from decimal import ROUND_HALF_UP, Decimal, localcontext
from fractions import Fraction
from itertools import product

_EMITTED = {}
_FAILED = []
_CHECKS = [0]

MODULUS = 3
HANDICAP = Fraction(1, 200)
ROUND = (Fraction(1, 2), Fraction(1, 10), Fraction(3, 4))  # A, bad, good
PUBLISHED = tuple(x - HANDICAP for x in ROUND)
ORDINARY = (Fraction(49, 100), Fraction(48, 100))
SEED = 1999
START = 7
ROUNDS = 100_000
SHORT = 1000
FIRST = 8
HORIZON = 100
LONG_HORIZON = 1000
SETTLE_LOOK = 200
SHARES = (Fraction(1, 20), Fraction(9, 10))
REMAINDERS = (6, 7, 8, 9, 10, 600, -1, -3)
# chapter 16: rounds of each schedule played one at a time in the checks
# (the scene shows 24), A's shares either side of the crossings and the
# whole percentages nearest them; chapter 17's crowds' start (corner 0)
# and window of dollars won or lost
PLAYED_CHECK = 120
CROWD_START = 0
EDGE_SHARES = ((70, 1000), (71, 1000), (847, 1000), (848, 1000))
WHOLE_EDGES = ((7, 100), (85, 100))
WINDOW = (-100, 140)
# chapter 08's real crowd: rounds shown one by one, and rounds counted
REAL_SHOWN = 8
REAL_ROUNDS = 1000
# chapter 11's split crowd under the mix, round by round from B's 10, 4, 12
MIX_SHIFT = 6
# Ethier & Lee, Electron. J. Probab. 14 (2009), paper 62: the capital-
# dependent games at ε = 0 (ρ = 1/3), means per game played
ETHIER_LEE = {"mix": Fraction(18, 709), "AB": Fraction(0), "AABB": Fraction(4, 163),
              "ABB": Fraction(2416, 35601), "AAB": Fraction(32, 1609)}


def emit(key, value):
    if key in _EMITTED:
        raise SystemExit(f"oracle: {key} emitted twice")
    if isinstance(value, bool):
        value = "true" if value else "false"
    _EMITTED[key] = str(value)
    print(f"{key}\t{value}")


def check(name, ok):
    _CHECKS[0] += 1
    if not ok:
        _FAILED.append(name)
    print(f"# {'ok  ' if ok else 'FAIL'} {name}", file=sys.stderr)


def csv(values):
    return ",".join(str(x) for x in values)


# ------------------------------------------------------------- formatting

def fixed(x, places):
    """x rounded half away from zero to `places` decimals, exactly."""
    x = Fraction(x)
    scaled = abs(x) * 10**places
    whole = scaled.numerator // scaled.denominator
    if scaled - whole >= Fraction(1, 2):
        whole += 1
    sign = "-" if x < 0 and whole else ""
    digits = str(whole).rjust(places + 1, "0")
    return sign + (digits[:-places] + "." + digits[-places:] if places else digits)


def signed(x, places):
    text = fixed(x, places)
    return "+" + text if Fraction(x) > 0 and text.strip("0.") else text


def pct(x, places=1):
    return fixed(100 * Fraction(x), places)


def exact_pct(x):
    """A percentage with at most one decimal, printed without a trailing .0."""
    y = 100 * Fraction(x)
    if (10 * y).denominator != 1:
        raise SystemExit(f"oracle: {x} is not a whole number of tenths of a percent")
    text = fixed(y, 1)
    return text[:-2] if text.endswith(".0") else text


def cents(x, places=2):
    return signed(100 * Fraction(x), places)


def fracs(values):
    return csv(Fraction(v) for v in values)


def decimal_sqrt(x):
    """The square root of a fraction to 60 significant digits."""
    with localcontext() as ctx:
        ctx.prec = 60
        return (Decimal(x.numerator) / Decimal(x.denominator)).sqrt()


def dec_fixed(d, places):
    """A positive Decimal rounded half up to `places` decimals, refused if
    it is within 1e-12 of a tie (floats on the Rust side then agree)."""
    if not far_from_boundary(Fraction(d), places, Fraction(1, 10**12)):
        raise SystemExit(f"oracle: {d} is too close to a rounding boundary at {places} places")
    return str(d.quantize(Decimal(1).scaleb(-places), rounding=ROUND_HALF_UP))


# -------------------------------------------------------------- the games

def b_game(coins):
    a, bad, good = coins
    return (bad, good, good)


def a_game(coins):
    return (coins[0],) * 3


def mix_of(g, h, share_g=Fraction(1, 2)):
    return tuple(share_g * x + (1 - share_g) * y for x, y in zip(g, h))


def step_matrix(game):
    m = [[Fraction(0)] * 3 for _ in range(3)]
    for i, p in enumerate(game):
        m[i][(i + 1) % 3] += p
        m[i][(i - 1) % 3] += 1 - p
    return m


def matmul(a, b):
    return [[sum(a[i][k] * b[k][j] for k in range(3)) for j in range(3)] for i in range(3)]


def vecmat(v, m):
    return [sum(v[i] * m[i][j] for i in range(3)) for j in range(3)]


def stationary(m):
    """π m = π, Σ π = 1, by Gaussian elimination over fractions."""
    rows = [[m[j][i] - (1 if i == j else 0) for j in range(3)] for i in range(3)]
    rows[2] = [Fraction(1)] * 3
    rhs = [Fraction(0), Fraction(0), Fraction(1)]
    for c in range(3):
        r = next(r for r in range(c, 3) if rows[r][c] != 0)
        rows[c], rows[r] = rows[r], rows[c]
        rhs[c], rhs[r] = rhs[r], rhs[c]
        for r in range(3):
            if r != c and rows[r][c] != 0:
                f = rows[r][c] / rows[c][c]
                rows[r] = [x - f * y for x, y in zip(rows[r], rows[c])]
                rhs[r] -= f * rhs[c]
    return [rhs[i] / rows[i][i] for i in range(3)]


def shares(game):
    return stationary(step_matrix(game))


def win_at(game, weights):
    return sum(w * p for w, p in zip(weights, game))


def mean(game):
    pi = shares(game)
    return sum(w * (2 * p - 1) for w, p in zip(pi, game))


def tree_shares(game):
    """The spanning-tree formula (the Rust model's method), for the sweep."""
    p = game
    q = [1 - x for x in game]
    w = (q[1] * p[2] + p[1] * p[2] + q[1] * q[2],
         p[0] * q[2] + p[0] * p[2] + q[0] * q[2],
         q[0] * p[1] + p[0] * p[1] + q[0] * q[1])
    total = sum(w)
    return [x / total for x in w]


def products(game):
    f = game[0] * game[1] * game[2]
    b = (1 - game[0]) * (1 - game[1]) * (1 - game[2])
    return f, b


def periodic(games):
    """Per round played, over a schedule repeated forever."""
    m = step_matrix(games[0])
    for g in games[1:]:
        m = matmul(m, step_matrix(g))
    pi = stationary(m)
    v = pi
    total = Fraction(0)
    for g in games:
        total += sum(v[i] * (2 * g[i] - 1) for i in range(3))
        v = vecmat(v, step_matrix(g))
    return total / len(games), v == pi


def cycle(games):
    """Each round's starting shares once the schedule has settled into its
    cycle, exactly: π of the period's matrix, carried round by round."""
    m = step_matrix(games[0])
    for g in games[1:]:
        m = matmul(m, step_matrix(g))
    v, out = stationary(m), []
    for g in games:
        out.append(v)
        v = vecmat(v, step_matrix(g))
    return out


def played(games, start, rounds):
    """The shares before each of `rounds` rounds of the schedule repeated,
    exactly, from `start`."""
    out, v = [], list(start)
    for k in range(rounds):
        out.append(v)
        v = vecmat(v, step_matrix(games[k % len(games)]))
    return out


def gain(game, v):
    """One round's average gain when the crowd's shares are v."""
    return sum(v[i] * (2 * game[i] - 1) for i in range(3))


def schedule(pattern, coins):
    return [a_game(coins) if c == "A" else b_game(coins) for c in pattern]


# --------------------------------------------------------------- the crowd

def flows(game, weights, n):
    forward = [n * w * p for w, p in zip(weights, game)]
    back = [n * w * (1 - p) for w, p in zip(weights, game)]
    return forward, back


def smallest_whole_crowd(game):
    weights = shares(game)
    n = 1
    while True:
        forward, back = flows(game, weights, n)
        if all(x.denominator == 1 for x in forward + back):
            return n
        n += 1


def evolve(v, game):
    return vecmat(v, step_matrix(game))


# ------------------------------------------------------------ the player

MASK = (1 << 64) - 1


class SplitMix64:
    def __init__(self, seed):
        self.state = seed & MASK

    def next(self):
        self.state = (self.state + 0x9E3779B97F4A7C15) & MASK
        z = self.state
        z = ((z ^ (z >> 30)) * 0xBF58476D1CE4E5B9) & MASK
        z = ((z ^ (z >> 27)) * 0x94D049BB133111EB) & MASK
        return z ^ (z >> 31)


def run(kind, coins, rounds, seed=SEED, start=START):
    """One simulated player: the dollars after every round, and how many
    rounds began at a multiple of three. Every chance is a whole number of
    200ths; a toss wins when the draw mod 200 is below it."""
    gen = SplitMix64(seed)
    a, b = a_game(coins), b_game(coins)
    dollars, at_zero, path = start, 0, [start]
    for _ in range(rounds):
        if kind == "mix":
            game = a if gen.next() % 2 == 0 else b
        else:
            game = a if kind == "A" else b
        r = dollars % MODULUS
        at_zero += r == 0
        heads = game[r] * 200
        if heads.denominator != 1:
            raise SystemExit("oracle: a chance is not a whole number of 200ths")
        dollars += 1 if gen.next() % 200 < heads.numerator else -1
        path.append(dollars)
    return path, at_zero


# ------------------------------------------------------------- finite play

def horizon(game, rounds, start=0):
    """Exact distribution of the dollars after `rounds`, on integer weights
    scaled by 200 per round: (ahead, behind, level) as fractions."""
    up = [int(p * 200) for p in game]
    if any(Fraction(u, 200) != p for u, p in zip(up, game)):
        raise SystemExit("oracle: a chance is not a whole number of 200ths")
    offset = rounds
    dist = [0] * (2 * rounds + 1)
    dist[start + offset] = 1
    lo = hi = start + offset
    for _ in range(rounds):
        new = [0] * (2 * rounds + 1)
        for i in range(lo, hi + 1):
            w = dist[i]
            if w:
                r = (i - offset) % MODULUS
                new[i + 1] += w * up[r]
                new[i - 1] += w * (200 - up[r])
        dist = new
        lo, hi = lo - 1, hi + 1
    total = 200**rounds
    ahead = sum(dist[offset + 1:])
    behind = sum(dist[:offset])
    level = dist[offset]
    return Fraction(ahead, total), Fraction(behind, total), Fraction(level, total), ahead + behind + level == total


def net_distribution(game, rounds, start):
    """Exact distribution of the dollars won or lost after `rounds`, from
    `start` dollars, on integer weights scaled by 200 per round (a dict,
    net -> weight, and its total 200**rounds). Independent of horizon(),
    which indexes an array of absolute dollars from a start of 0."""
    up = [int(p * 200) for p in game]
    if any(Fraction(u, 200) != p for u, p in zip(up, game)):
        raise SystemExit("oracle: a chance is not a whole number of 200ths")
    dist = {0: 1}
    for _ in range(rounds):
        new = {}
        for net, w in dist.items():
            r = (start + net) % MODULUS
            new[net + 1] = new.get(net + 1, 0) + w * up[r]
            new[net - 1] = new.get(net - 1, 0) + w * (200 - up[r])
        dist = new
    return dist, 200**rounds


def categories(dist, total):
    """Ahead of the start, behind it, level with it."""
    return (Fraction(sum(w for k, w in dist.items() if k > 0), total),
            Fraction(sum(w for k, w in dist.items() if k < 0), total),
            Fraction(dist.get(0, 0), total))


def moments(dist, total):
    """The mean and the variance of the net, exactly."""
    m = Fraction(sum(k * w for k, w in dist.items()), total)
    return m, Fraction(sum(k * k * w for k, w in dist.items()), total) - m * m


def window_mass(dist, total):
    lo, hi = WINDOW
    return Fraction(sum(w for k, w in dist.items() if lo <= k <= hi), total)


def poly_mul(a, b):
    out = [Fraction(0)] * (len(a) + len(b) - 1)
    for i, x in enumerate(a):
        for j, y in enumerate(b):
            out[i + j] += x * y
    return out


def poly_trim(a):
    a = list(a)
    while len(a) > 1 and a[-1] == 0:
        a.pop()
    return a


def poly_rem(a, b):
    a, b = poly_trim(a), poly_trim(b)
    while len(a) >= len(b) and any(a):
        f = a[-1] / b[-1]
        shift = len(a) - len(b)
        for i, y in enumerate(b):
            a[i + shift] -= f * y
        a = poly_trim(a[:-1]) if len(a) > 1 else [Fraction(0)]
    return a


def poly_at(a, x):
    return sum(c * x**k for k, c in enumerate(a))


def sturm_roots(a, lo, hi):
    """How many distinct real roots the polynomial a has in (lo, hi]
    (Sturm's theorem; a has no root at lo)."""
    seq = [poly_trim(a), poly_trim([k * c for k, c in enumerate(a)][1:])]
    while any(seq[-1]) and len(seq[-1]) > 1:
        seq.append([-c for c in poly_rem(seq[-2], seq[-1])])
    def changes(x):
        signs = [s for s in ((poly_at(p, x) > 0) - (poly_at(p, x) < 0) for p in seq) if s]
        return sum(1 for s, t in zip(signs, signs[1:]) if s != t)
    return changes(lo) - changes(hi)


def bisect(f, lo, hi, steps=60):
    """A sign change of f in [lo, hi], exactly, to 2^-steps."""
    flo = f(lo) > 0
    for _ in range(steps):
        mid = (lo + hi) / 2
        if (f(mid) > 0) == flo:
            lo = mid
        else:
            hi = mid
    return lo, hi


def far_from_boundary(x, places, margin=Fraction(1, 10**9)):
    """x is not within `margin` of a rounding boundary at `places` decimals:
    floats on the Rust side round it the same way."""
    scaled = Fraction(x) * 10**places
    frac_part = scaled - (scaled.numerator // scaled.denominator)
    return abs(frac_part - Fraction(1, 2)) > margin * 10**places


# ------------------------------------------------------------------ facts

def facts():
    out = {}
    pub_a, pub_b = a_game(PUBLISHED), b_game(PUBLISHED)
    pub_mix = mix_of(pub_a, pub_b)
    r_a, r_b = a_game(ROUND), b_game(ROUND)
    r_mix = mix_of(r_a, r_b)

    # the rules (01, 02)
    emit("rules.modulus", MODULUS)
    emit("rules.handicap", HANDICAP)
    emit("coins.published", fracs(PUBLISHED))
    emit("coins.published.heads.pct", csv(exact_pct(p) for p in PUBLISHED))
    emit("coins.published.tails.pct", csv(exact_pct(1 - p) for p in PUBLISHED))
    # the paper itself (13_10's card): Harmer & Abbott, Nature 402, 864
    # (1999), doi:10.1038/47220
    emit("source.volume", 402)
    emit("source.page", 864)
    emit("source.year", 1999)
    emit("coins.round", fracs(ROUND))
    emit("coins.round.heads.pct", csv(exact_pct(p) for p in ROUND))
    emit("coins.round.tails.pct", csv(exact_pct(1 - p) for p in ROUND))
    emit("remainder.examples", csv(f"{n}:{n % MODULUS}" for n in REMAINDERS))

    # game A's hundred rounds (01_6 to 01_8)
    hundred = 100
    emit("a.hundred.wins", exact_pct(PUBLISHED[0]))
    emit("a.hundred.losses", exact_pct(1 - PUBLISHED[0]))
    emit("a.hundred.net", hundred * (2 * PUBLISHED[0] - 1))

    # the player (01, 03, 04, 05, 07, 17)
    emit("run.seed", SEED)
    emit("run.start", START)
    emit("run.rounds", ROUNDS)
    runs = {}
    for kind in ("A", "B", "mix"):
        path, at_zero = run(kind, PUBLISHED, ROUNDS)
        runs[kind] = (path, at_zero)
        key = kind.lower()
        emit(f"run.{key}.net", path[-1] - START)
        emit(f"run.{key}.at0.rounds", at_zero)
        emit(f"run.{key}.at0.pct", pct(Fraction(at_zero, ROUNDS)))
        emit(f"run.{key}.low", min(path))
        emit(f"run.{key}.high", max(path))
    emit("run.a.first", csv(runs["A"][0][:FIRST + 1]))
    emit("run.b.net.short", runs["B"][0][SHORT] - START)
    out["runs"] = runs

    # B by equal thirds, and B's verdict (03)
    third = Fraction(1, 3)
    pub_thirds = win_at(pub_b, [third] * 3)
    emit("pub.b.thirds.win", pub_thirds)
    emit("pub.b.thirds.win.pct", pct(pub_thirds))
    emit("thirds.pct", pct(third))
    pb = shares(pub_b)
    emit("pub.b.weights", fracs(pb))
    emit("pub.b.weights.pct", csv(pct(w) for w in pb))
    emit("pub.b.win", win_at(pub_b, pb))
    emit("pub.b.mean", mean(pub_b))
    emit("pub.b.mean.cents", cents(mean(pub_b)))
    emit("pub.a.mean", mean(pub_a))
    emit("pub.a.mean.cents", cents(mean(pub_a)))
    # the dashed line of 01_9 and its "about a thousand dollars below"
    emit("run.a.expected.net", ROUNDS * mean(pub_a))

    # why the mix looks hopeless (04): ordinary coins and the halfway average
    mixed = sum(ORDINARY) / 2
    emit("ordinary.coins.pct", csv(exact_pct(p) for p in ORDINARY))
    emit("ordinary.mixed.pct", exact_pct(mixed))
    halfway = (mean(pub_a) + mean(pub_b)) / 2
    emit("pub.halfway.mean", halfway)
    emit("pub.halfway.mean.cents", cents(halfway))

    # the mix with the published coins (04, 05, 13)
    pm = shares(pub_mix)
    emit("pub.mix.coins", fracs(pub_mix[:2]))
    emit("pub.mix.coins.pct", csv(exact_pct(p) for p in pub_mix[:2]))
    emit("pub.mix.weights", fracs(pm))
    emit("pub.mix.weights.pct", csv(pct(w) for w in pm))
    emit("pub.mix.win", win_at(pub_mix, pm))
    emit("pub.mix.win.pct", pct(win_at(pub_mix, pm)))
    emit("pub.mix.mean", mean(pub_mix))
    emit("pub.mix.mean.cents", cents(mean(pub_mix)))
    emit("pub.mix.b_rounds.win", win_at(pub_b, pm))
    emit("pub.mix.b_rounds.win.pct", pct(win_at(pub_b, pm)))
    good, bad = PUBLISHED[2], PUBLISHED[1]
    threshold = (good - Fraction(1, 2)) / (good - bad)
    emit("pub.b.fair_share", threshold)
    emit("pub.b.fair_share.pct", pct(threshold))

    # the round-number coins (07 to 12)
    r_thirds = win_at(r_b, [third] * 3)
    emit("round.b.thirds.win", r_thirds)
    emit("round.b.thirds.win.pct", pct(r_thirds))
    rb = shares(r_b)
    emit("round.b.weights", fracs(rb))
    emit("round.b.weights.pct", csv(pct(w) for w in rb))
    emit("round.b.win", win_at(r_b, rb))
    emit("round.b.mean", mean(r_b))
    slope = ROUND[2] - ROUND[1]
    emit("round.b.line", fracs((ROUND[2], slope)))
    emit("round.b.line.pct", csv((exact_pct(ROUND[2]), exact_pct(slope))))
    emit("round.b.fair_share", (ROUND[2] - Fraction(1, 2)) / slope)
    die = third * ROUND[1] + 2 * third * ROUND[2]
    emit("round.die.win", die)
    emit("pub.die.win", third * PUBLISHED[1] + 2 * third * PUBLISHED[2])

    # the crowd of 26 (08, 09)
    n = smallest_whole_crowd(r_b)
    emit("crowd.size", n)
    counts = [n * w for w in rb]
    forward, back = flows(r_b, rb, n)
    emit("crowd.counts", csv(counts))
    emit("crowd.forward", csv(forward))
    emit("crowd.back", csv(back))
    # 14_12: no edge of B's settled crowd carries a net flow
    emit("crowd.net", csv(forward[i] - back[(i + 1) % 3] for i in range(3)))
    # arrivals at each corner by the corner they come from, lowest first
    arrivals = []
    for j in range(3):
        parts = []
        for i in range(3):
            if (i + 1) % 3 == j:
                parts.append(forward[i])
            elif (i - 1) % 3 == j:
                parts.append(back[i])
        arrivals.append("+".join(str(x) for x in parts))
    emit("crowd.arrivals", csv(arrivals))
    emit("crowd.winners", sum(forward))
    emit("crowd.share0.pct", pct(rb[0]))
    v = [Fraction(n), Fraction(0), Fraction(0)]
    history = []
    for k in range(1, SETTLE_LOOK + 1):
        v = evolve(v, r_b)
        history.append(v)
    for k in range(1, 7):
        emit(f"crowd.round.{k}", csv(fixed(x, 1) for x in history[k - 1]))
    half = Fraction(1, 2)
    settle = next(k for k in range(1, SETTLE_LOOK + 1)
                  if all(max(abs(x - c) for x, c in zip(history[j - 1], counts)) < half
                         for j in range(k, SETTLE_LOOK + 1)))
    emit("crowd.within_half_from", settle)
    out["settle"] = (history, counts, settle)
    out["crowd"] = (n, counts, forward, back)
    # 10_4: the good coin wins at corners 1 and 2; the share of those wins at 2
    emit("crowd.good_wins_at2", forward[2] / (forward[1] + forward[2]))

    # 08_7: 26 real players started at 10, 4, 12, each tossing a real coin
    # every round (one SplitMix64 stream, players in a fixed order)
    gen = SplitMix64(SEED)
    corners = [c for c, k in enumerate(counts) for _ in range(int(k))]
    real = []
    for _ in range(REAL_ROUNDS):
        for k, c in enumerate(corners):
            heads = r_b[c] * 200
            if heads.denominator != 1:
                raise SystemExit("oracle: a chance is not a whole number of 200ths")
            corners[k] = (c + 1) % 3 if gen.next() % 200 < heads.numerator else (c - 1) % 3
        real.append(tuple(corners.count(i) for i in range(3)))
    target = tuple(int(c) for c in counts)
    for k in range(1, REAL_SHOWN + 1):
        emit(f"realcrowd.round.{k}", csv(real[k - 1]))
    emit("realcrowd.rounds", REAL_ROUNDS)
    emit("realcrowd.exact", sum(1 for c in real if c == target))
    emit("realcrowd.mean", csv(fixed(Fraction(sum(c[i] for c in real), REAL_ROUNDS), 1) for i in range(3)))
    out["realcrowd"] = (real, target)

    # 11_5: the mix's split crowd, round by round from B's settled 10, 4, 12
    v = [Fraction(c) for c in counts]
    shift = []
    for k in range(1, MIX_SHIFT + 1):
        v = evolve(v, r_mix)
        shift.append([x / n for x in v])
        emit(f"mixshift.round.{k}", csv(pct(x) for x in shift[-1]))
    out["mixshift"] = shift

    # the mix with the round-number coins (11, 12, 14)
    rm = shares(r_mix)
    emit("round.mix.coins", fracs(r_mix[:2]))
    emit("round.mix.coins.pct", csv(exact_pct(p) for p in r_mix[:2]))
    emit("round.mix.tails.pct", csv(exact_pct(1 - p) for p in r_mix[:2]))
    emit("round.mix.weights", fracs(rm))
    emit("round.mix.weights.pct", csv(pct(w) for w in rm))
    emit("round.mix.win", win_at(r_mix, rm))
    emit("round.mix.win.pct", pct(win_at(r_mix, rm)))
    emit("round.mix.win.pct2", pct(win_at(r_mix, rm), 2))
    emit("round.mix.b_rounds.win", win_at(r_b, rm))
    emit("round.mix.b_rounds.win.pct", pct(win_at(r_b, rm)))
    emit("round.mix.b_rounds.win.pct2", pct(win_at(r_b, rm), 2))
    emit("round.mix.a_rounds.win", win_at(r_a, rm))
    emit("round.mix.mean", mean(r_mix))
    emit("round.mix.mean.cents", cents(mean(r_mix)))
    toward = (rb[0] - rm[0]) / (rb[0] - third)
    emit("round.mix.toward_thirds.pct", pct(toward, 0))
    cost = mean(r_mix) - mean(pub_mix)
    emit("handicap.mix_cost", cost)
    emit("handicap.mix_cost.cents", fixed(100 * cost, 2))

    m = smallest_whole_crowd(r_mix)
    mcounts = [m * w for w in rm]
    mforward, mback = flows(r_mix, rm, m)
    emit("mixcrowd.size", m)
    emit("mixcrowd.counts", csv(mcounts))
    emit("mixcrowd.forward", csv(mforward))
    emit("mixcrowd.back", csv(mback))
    net = [mforward[i] - mback[(i + 1) % 3] for i in range(3)]
    emit("mixcrowd.net", csv(net))
    emit("mixcrowd.gain", sum(mforward) - sum(mback))
    emit("mixcrowd.winners", sum(mforward))
    emit("mixcrowd.losers", sum(mback))
    out["mixcrowd"] = (m, mcounts, mforward, mback, net)

    # 14_10, 14_11: 1,418 real players started at 490, 360, 568, tossing
    # real coins (08_7's stream and player order), counted as crossings
    gen = SplitMix64(SEED)
    corners = [c for c, k in enumerate(mcounts) for _ in range(int(k))]
    crossings = []
    for _ in range(REAL_ROUNDS):
        moved = ([0, 0, 0], [0, 0, 0])
        for k, c in enumerate(corners):
            heads = r_mix[c] * 200
            if heads.denominator != 1:
                raise SystemExit("oracle: a chance is not a whole number of 200ths")
            win = gen.next() % 200 < heads.numerator
            moved[0 if win else 1][c] += 1
            corners[k] = (c + 1) % 3 if win else (c - 1) % 3
        crossings.append((moved, tuple(corners.count(i) for i in range(3))))
    (real_f, real_b), after = crossings[0]
    rnet = [real_f[i] - real_b[(i + 1) % 3] for i in range(3)]
    emit("realmix.round.1.forward", csv(real_f))
    emit("realmix.round.1.back", csv(real_b))
    emit("realmix.round.1.net", csv(rnet))
    emit("realmix.round.1.after", csv(after))
    emit("realmix.round.1.gain", sum(rnet))
    emit("realmix.rounds", REAL_ROUNDS)
    totals = [sum(f[i] - b[(i + 1) % 3] for (f, b), _ in crossings) for i in range(3)]
    emit("realmix.mean.net", csv(fixed(Fraction(t, REAL_ROUNDS), 1) for t in totals))
    emit("realmix.mean.gain", fixed(Fraction(sum(totals), REAL_ROUNDS), 1))
    out["realmix"] = (crossings, rnet)

    # the two products (15)
    fb, bb = products(r_b)
    emit("round.products.b", fracs((fb, bb)))
    fm, bm = products(r_mix)
    emit("round.products.mix", fracs((fm, bm)))
    common = max(fm.denominator, bm.denominator)
    while common % fm.denominator or common % bm.denominator:
        common += 1
    emit("round.products.mix.common", f"{fm * common}/{common},{bm * common}/{common}")
    # 15_12: the one line on the mix's crowd, net × (p2 + q1 q2) = w0 ×
    # (p0 p1 p2 − q0 q1 q2), in players on both sides
    factor = r_mix[2] + (1 - r_mix[1]) * (1 - r_mix[2])
    emit("round.mix.sign.factor", factor)
    emit("round.products.mix.diff", fm - bm)
    emit("round.mix.sign.sides", net[0] * factor)
    out["sign_line"] = (factor, fm - bm)
    for name, game in (("a", pub_a), ("b", pub_b), ("mix", pub_mix)):
        f, b = products(game)
        emit(f"pub.products.{name}", fracs((f, b)))
        emit(f"pub.products.{name}.fixed", csv((fixed(f, 4), fixed(b, 4))))

    # schedules (16)
    for pattern in ("AB", "AABB", "ABB", "AAB"):
        value, _ = periodic(schedule(pattern, ROUND))
        emit(f"round.sched.{pattern.lower()}", value)
    for pattern in ("AB", "AABB"):
        value, _ = periodic(schedule(pattern, PUBLISHED))
        emit(f"pub.sched.{pattern.lower()}", value)
        emit(f"pub.sched.{pattern.lower()}.cents", cents(value))

    # A's share in the mix (16_5)
    def mix_mean(share):
        return mean(mix_of(pub_a, pub_b, share))

    low = bisect(mix_mean, Fraction(0), Fraction(1, 2))
    high = bisect(mix_mean, Fraction(1, 2), Fraction(1))
    emit("pub.share.low.pct", pct(low[0]))
    emit("pub.share.high.pct", pct(high[0]))
    for s in SHARES:
        emit(f"pub.share.{s.numerator}_{s.denominator}.cents", cents(mix_mean(s)))
    out["crossings"] = (low, high, mix_mean)

    # 16: what each schedule's rounds meet, played round by round from B's
    # settled crowd and in the cycle each settles into (exact)
    emit("sched.a_share", csv([Fraction(1, 2)] + [Fraction(p.count("A"), len(p)) for p in ("AB", "AABB")]))
    ab_games, aabb_games = schedule("AB", PUBLISHED), schedule("AABB", PUBLISHED)
    pb = shares(pub_b)
    ab_played = played(ab_games, pb, PLAYED_CHECK)
    aabb_played = played(aabb_games, pb, PLAYED_CHECK)
    emit("pub.sched.ab.first.b.at0.pct", pct(ab_played[1][0]))
    ab_cycle, aabb_cycle = cycle(ab_games), cycle(aabb_games)
    emit("pub.sched.ab.cycle.a.pct2", csv(pct(x, 2) for x in ab_cycle[0]))
    emit("pub.sched.ab.cycle.b.pct2", csv(pct(x, 2) for x in ab_cycle[1]))
    emit("pub.sched.ab.cycle.b.pct", csv(pct(x) for x in ab_cycle[1]))
    emit("pub.sched.ab.b.cents", cents(gain(pub_b, ab_cycle[1])))
    emit("pub.sched.aabb.b.at0.pct", csv(pct(aabb_cycle[k][0]) for k in (2, 3)))
    emit("pub.sched.aabb.b.cents", csv(cents(gain(pub_b, aabb_cycle[k])) for k in (2, 3)))
    a_coin = PUBLISHED[0]
    from2, from1 = a_coin * ab_cycle[0][2], (1 - a_coin) * ab_cycle[0][1]
    emit("pub.sched.ab.refill.pct2", csv(pct(x, 2) for x in (from2, from1, ab_cycle[1][0])))
    plain = (ORDINARY[0], ORDINARY[1], ORDINARY[1])
    plain_sched = (mean(mix_of(a_game(plain), b_game(plain))), periodic(schedule("AB", plain))[0],
                   periodic(schedule("AABB", plain))[0])
    emit("ordinary.sched.cents", csv(cents(x) for x in plain_sched))
    out["schedules"] = (ab_played, aabb_played, ab_cycle, aabb_cycle, (from2, from1), plain_sched)
    # 16_12 to 16_14: the crossings rounded, and the exact strict bounds
    emit("pub.share.points", csv(SHARES))
    emit("pub.share.crossings.fine", csv(pct(x[0], 2) for x in (low, high)))
    emit("pub.share.crossings.whole", csv(pct(x[0], 0) for x in (low, high)))
    edges = [Fraction(n, d) for n, d in EDGE_SHARES]
    edge_means = [mix_mean(s) for s in edges]
    emit("pub.share.edge.pct", csv(pct(s) for s in edges))
    emit("pub.share.edge.cents", csv(cents(m, 4) for m in edge_means))
    emit("pub.share.edge.sign", csv((m > 0) - (m < 0) for m in edge_means))
    whole_means = [mix_mean(Fraction(n, d)) for n, d in WHOLE_EDGES]
    emit("pub.share.whole.cents", csv(cents(m, 4) for m in whole_means))
    out["share_edges"] = (edges, edge_means, whole_means)

    # finite play (17)
    ahead, behind, level, whole = horizon(pub_mix, HORIZON)
    emit("horizon.short.mix", csv(pct(x) for x in (ahead, behind, level)))
    long_mix = horizon(pub_mix, LONG_HORIZON)
    long_a = horizon(pub_a, LONG_HORIZON)
    long_b = horizon(pub_b, LONG_HORIZON)
    emit("horizon.long.mix.ahead.pct", pct(long_mix[0]))
    emit("horizon.long.a.behind.pct", pct(long_a[1]))
    emit("horizon.long.b.behind.pct", pct(long_b[1]))
    out["horizon"] = ((ahead, behind, level, whole), long_mix, long_a, long_b)

    # 17: the crowd's exact distributions of dollars won or lost
    emit("horizon.rounds", csv((HORIZON, LONG_HORIZON)))
    emit("horizon.rounds.ratio", LONG_HORIZON // HORIZON)
    emit("horizon.start", CROWD_START)
    emit("run.short", SHORT)
    dists = [net_distribution(pub_mix, n, CROWD_START) for n in (HORIZON, LONG_HORIZON)]
    cats = [categories(d, t) for d, t in dists]
    emit("horizon.long.mix", csv(pct(x) for x in cats[1]))
    d100, t100 = dists[0]
    top = max(d100, key=lambda k: d100[k])
    emit("horizon.short.mix.tallest", f"{top}:{pct(Fraction(d100[top], t100))}")
    b7, t7 = net_distribution(pub_b, LONG_HORIZON, START)
    windows = [window_mass(d, t) for d, t in dists + [(b7, t7)]]
    emit("horizon.window.pct", csv(pct(x) for x in windows))
    moms = [moments(d, t) for d, t in dists]
    emit("horizon.mix.mean", csv(signed(m, 2) for m, _ in moms))
    sds = [decimal_sqrt(v) for _, v in moms]
    emit("horizon.mix.spread", csv(dec_fixed(s, 1) for s in sds))
    emit("horizon.mix.spread2", csv(dec_fixed(s, 2) for s in sds))
    ratio = decimal_sqrt(moms[1][1] / moms[0][1])
    emit("horizon.mix.spread.ratio", dec_fixed(ratio, 2))
    emit("horizon.mix.spread.ratio.fine", dec_fixed(ratio, 4))
    root10 = decimal_sqrt(Fraction(LONG_HORIZON, HORIZON))
    emit("horizon.rounds.sqrt.fine", dec_fixed(root10, 4))
    at_rate = [n * mean(pub_mix) for n in (HORIZON, LONG_HORIZON)]
    emit("horizon.mix.at_rate", csv(fixed(x, 2) for x in at_rate))
    costs = [r - m for r, (m, _) in zip(at_rate, moms)]
    emit("horizon.mix.start_cost", fixed(costs[0], 2))
    lucky_net = runs["B"][0][SHORT] - START
    ahead7 = Fraction(sum(w for k, w in b7.items() if k > 0), t7)
    lucky = Fraction(sum(w for k, w in b7.items() if k >= lucky_net), t7)
    emit("horizon.long.b.from_start", csv(pct(x) for x in (ahead7, lucky)))
    out["finite"] = (dists, cats, top, (b7, t7), windows, moms, sds, ratio, root10, costs, ahead7, lucky)

    # the closing puzzle (18): switching by the count, both ways round
    reverse = (PUBLISHED[1], PUBLISHED[0], PUBLISHED[0])
    swapped = (PUBLISHED[0], PUBLISHED[2], PUBLISHED[2])
    emit("pub.reverse.mean", mean(reverse))
    emit("pub.reverse.mean.cents", cents(mean(reverse), 1))
    emit("pub.swapped.mean", mean(swapped))
    emit("pub.swapped.mean.cents", cents(mean(swapped), 1))
    out["closing"] = (reverse, swapped)
    return out


# ------------------------------------------------------------ cross-checks

def cross_checks(out):
    pub_a, pub_b = a_game(PUBLISHED), b_game(PUBLISHED)
    pub_mix = mix_of(pub_a, pub_b)
    r_a, r_b = a_game(ROUND), b_game(ROUND)
    r_mix = mix_of(r_a, r_b)

    # every game on the grid of tenths: the Rust model's formulas hold
    grid = [Fraction(k, 10) for k in range(1, 10)]
    tree_bad = flow_bad = sign_bad = fair = converse_bad = line_bad = 0
    for game in product(grid, repeat=3):
        pi = shares(game)
        tree_bad += tree_shares(game) != pi
        edge = [pi[i] * game[i] - pi[(i + 1) % 3] * (1 - game[(i + 1) % 3]) for i in range(3)]
        flow_bad += not (edge[0] == edge[1] == edge[2] and mean(game) == 3 * edge[0])
        f, b = products(game)
        sgn = (mean(game) > 0) - (mean(game) < 0)
        sign_bad += sgn != (f > b) - (f < b)
        fair += f == b
        # 15_1, the converse: the first two balances fix the weights; the
        # third holds exactly when the products match, and then those
        # weights are the settled crowd and the net flow is zero
        p, q = game, [1 - x for x in game]
        w = [Fraction(1), p[0] / q[1], p[0] * p[1] / (q[1] * q[2])]
        third = w[2] * p[2] == w[0] * q[0]
        balanced = [x / sum(w) for x in w]
        converse_bad += third != (f == b) or (third and (balanced != pi or edge[0] != 0))
        # 15_7, 15_8: the one line, with both of its other factors positive
        line_bad += (edge[0] * (p[2] + q[1] * q[2]) != pi[0] * (f - b)
                     or not (p[2] + q[1] * q[2] > 0 and pi[0] > 0))
    emit("sign.games", 9**3)
    emit("sign.violations", sign_bad)
    emit("sign.fair_games", fair)
    check("729 games: the spanning-tree shares equal the eliminated ones", tree_bad == 0)
    check("15_1, 729 games: the third balance holds exactly when the products match, and then the "
          "balanced weights are the settled crowd with zero net flow", converse_bad == 0 and fair > 0)
    check("729 games: every edge carries the same net flow, and the average is three times it", flow_bad == 0)
    check("729 games: the average's sign is the sign of forward minus backward product", sign_bad == 0)
    check("15_7, 729 games: net × (p2 + q1 q2) = w0 × (p0 p1 p2 − q0 q1 q2), with p2 + q1 q2 and w0 "
          "positive", line_bad == 0)
    # 15_1's condition at its edge: chances from 0 to 1 in tenths, certain
    # coins included (the shares by the spanning-tree formula, checked
    # stationary). The one line still holds; whenever the products differ
    # both other factors are positive, so the sign rule survives; the
    # factor vanishes only with both products 0 and no net flow.
    edge_games = edge_bad = vanishing = 0
    for game in product([Fraction(k, 10) for k in range(11)], repeat=3):
        pi = tree_shares(game)
        p, q = game, [1 - x for x in game]
        edge = [pi[i] * p[i] - pi[(i + 1) % 3] * q[(i + 1) % 3] for i in range(3)]
        f, b = products(game)
        factor = p[2] + q[1] * q[2]
        edge_games += 1
        vanishing += factor == 0
        edge_bad += (vecmat(pi, step_matrix(game)) != pi or not edge[0] == edge[1] == edge[2]
                     or edge[0] * factor != pi[0] * (f - b)
                     or (f != b and not (factor > 0 and pi[0] > 0))
                     or (factor == 0 and not (f == b == 0 and edge[0] == 0)))
    check(f"15_1's edge, {edge_games} games with certain coins: the one line holds, the sign rule "
          f"survives, and the {vanishing} games with a vanishing factor have no net flow",
          edge_bad == 0 and vanishing > 0)
    check("the sweep has fair games to test the equality case (A's fair coin among them)", fair > 0)

    # game A treats every corner alike, whatever its coin
    check("a coin that ignores the money gives each corner a third", all(shares((p,) * 3) == [Fraction(1, 3)] * 3 for p in grid))
    # two losing ordinary coins mix into a losing coin, at every share
    losing = [Fraction(k, 100) for k in range(1, 50)]
    cases = winners = 0
    for p, q in product(losing, repeat=2):
        for s in (Fraction(1, 10), Fraction(1, 2), Fraction(9, 10)):
            cases += 1
            winners += mean(mix_of((p,) * 3, (q,) * 3, s)) >= 0
    emit("ordinary.sweep.cases", cases)
    emit("ordinary.sweep.winning", winners)
    check("every mix of two losing ordinary coins loses", winners == 0)

    # the mix's gain is all in B's rounds; A's rounds win exactly A's coin
    for coins, game in ((ROUND, r_mix), (PUBLISHED, pub_mix)):
        pi = shares(game)
        a, b = a_game(coins), b_game(coins)
        check(f"the mix's win is half A's rounds and half B's rounds (A coin {coins[0]})",
              win_at(game, pi) == (win_at(a, pi) + win_at(b, pi)) / 2 and win_at(a, pi) == coins[0])
    # B's chance is one line in its share at corner 0
    for coins in (ROUND, PUBLISHED):
        b = b_game(coins)
        ok = all(win_at(b, (w0, (1 - w0) / 2, (1 - w0) / 2)) == coins[2] - (coins[2] - coins[1]) * w0
                 for w0 in (Fraction(0), Fraction(1, 3), Fraction(5, 13), Fraction(1)))
        check(f"B wins its good coin minus 65 points per unit of share at corner 0 (good {coins[2]})", ok)
    check("round coins: B is fair at share 5/13 exactly", shares(r_b)[0] == Fraction(5, 13) and mean(r_b) == 0)

    # Ethier & Lee pins, and stationarity over each period
    check("Ethier & Lee: the round-coin mix averages 18/709", mean(r_mix) == ETHIER_LEE["mix"])
    for pattern in ("AB", "AABB", "ABB", "AAB"):
        value, back = periodic(schedule(pattern, ROUND))
        check(f"Ethier & Lee: {pattern} averages {ETHIER_LEE[pattern]} per round, and the shares return after a period",
              value == ETHIER_LEE[pattern] and back)
    ab, _ = periodic(schedule("AB", PUBLISHED))
    aabb, _ = periodic(schedule("AABB", PUBLISHED))
    check("published coins: strict alternation loses while AABB and the random mix win (the caption's 'any sequence' is false)",
          ab < 0 < aabb and mean(pub_mix) > 0)

    # crowds
    n, counts, forward, back = out["crowd"]
    check("the crowd of 26: one round sends every player on and the counts return",
          [forward[2] + back[1], forward[0] + back[2], forward[1] + back[0]] == counts and sum(counts) == 26)
    check("13 of 26 win: B is fair with the round coins", sum(forward) * 2 == n)
    m, mcounts, mforward, mback, net = out["mixcrowd"]
    check("the mix's crowd of 1,418: counts return, 12 net on every edge, gain 36 = 727 - 691",
          [mforward[2] + mback[1], mforward[0] + mback[2], mforward[1] + mback[0]] == mcounts
          and net == [12, 12, 12] and sum(mforward) - sum(mback) == 36 == 3 * net[0])
    check("the gain per round of the crowd of 1,418 is 1,418 times 18/709", Fraction(36, m) == mean(r_mix))
    # 14_9's control: B's 10, 4, 12 under the mix's coins is not settled;
    # its edges disagree and each corner changes by net in minus net out
    uf = [c * p for c, p in zip(counts, r_mix)]
    ub = [c * (1 - p) for c, p in zip(counts, r_mix)]
    unet = [uf[i] - ub[(i + 1) % 3] for i in range(3)]
    moved = evolve(counts, r_mix)
    check("control 14_9: an unsettled crowd (B's 10, 4, 12 under the mix) has edges that disagree, "
          "and each corner changes by net in minus net out",
          len(set(unet)) > 1 and all(moved[i] - counts[i] == unet[(i - 1) % 3] - unet[i] for i in range(3)))
    # 14_10, 14_11: real rounds against the split crowd
    real_moves, rnet = out["realmix"]
    now = [int(c) for c in mcounts]
    kept = True
    for (f, b), corners in real_moves:
        nets = [f[i] - b[(i + 1) % 3] for i in range(3)]
        kept = kept and sum(corners) == m and all(
            corners[i] - now[i] == nets[(i - 1) % 3] - nets[i] for i in range(3))
        now = list(corners)
    check("14_10: in every real round each corner changes by net in minus net out, and all 1,418 stay", kept)
    check("control 14_10: the first real round's edges disagree and it loses dollars; one round is not the "
          "expected flow", len(set(rnet)) > 1 and sum(rnet) < 0)
    # 15_12, 15_13: the one line on the mix's crowd, and a player's average
    factor, diff = out["sign_line"]
    check("15_12: on the mix's crowd both sides of the one line agree, and three times the net flow per "
          "player is the mix's average, 3 w0 (P − Q) / (p2 + q1 q2)",
          net[0] * factor == mcounts[0] * diff and 3 * net[0] / m == mean(r_mix)
          == 3 * shares(r_mix)[0] * diff / factor)
    history, counts, settle = out["settle"]
    check("the crowd keeps all 26 players every round", all(sum(v) == 26 for v in history))
    worst = [max(abs(x - c) for x, c in zip(history[j - 1], counts)) for j in (settle - 1, settle)]
    check(f"the settling round {settle} is not a rounding accident: before it {float(worst[0]):.6f}, at it {float(worst[1]):.6f}",
          worst[0] - Fraction(1, 2) > Fraction(1, 10**6) and Fraction(1, 2) - worst[1] > Fraction(1, 10**6))
    check("10_4: most of the good coin's wins in B's crowd happen at corner 2, and a win there lands on corner 0",
          forward[2] > forward[1] and (2 + 1) % 3 == 0)
    real, target = out["realcrowd"]
    check("08_7: the real crowd keeps 26 players, rarely shows exactly 10, 4, 12, and averages 10.0, 4.0, 12.0",
          all(sum(c) == 26 for c in real) and sum(1 for c in real if c == target) < REAL_ROUNDS // 10
          and [fixed(Fraction(sum(c[i] for c in real), REAL_ROUNDS), 1) for i in range(3)] == ["10.0", "4.0", "12.0"])
    check("control: the split crowd never moves off 10, 4, 12; the real crowd does in its first round",
          evolve([Fraction(c) for c in target], r_b) == [Fraction(c) for c in target] and real[0] != target)
    limit = shares(r_mix)[0]
    zero = [s[0] for s in out["mixshift"]]
    crossings = sum(1 for x, y in zip(zero, zero[1:]) if (x - limit) * (y - limit) < 0)
    check(f"11_5: corner 0's share drops at once to near its limit, then sloshes across it ({crossings} crossings in {MIX_SHIFT} rounds)",
          zero[0] < shares(r_b)[0] and abs(zero[0] - limit) < Fraction(1, 200) and crossings >= 2)

    # the closing puzzle answers from the picture
    reverse, swapped = out["closing"]
    f, b = products(reverse)
    check("B at corner 0 and A elsewhere: every coin below a half, forward product smaller, loses",
          all(p < Fraction(1, 2) for p in reverse) and f < b and mean(reverse) < 0)
    f, b = products(swapped)
    check("A at corner 0 and B elsewhere wins", f > b and mean(swapped) > 0)

    # finite play
    (ahead, behind, level, whole), long_mix, long_a, long_b = out["horizon"]
    check("the 100-round distribution sums to one", whole and long_mix[3] and long_a[3] and long_b[3])
    rounded = [ahead, behind, level, long_mix[0], long_a[1], long_b[1]]
    check("every horizon share is far from a rounding boundary at one decimal of a percent (floats agree)",
          all(far_from_boundary(100 * x, 1) for x in rounded))
    check("over 1,000 rounds most players of A alone and of B alone are behind, most of the mix ahead",
          long_a[1] > Fraction(1, 2) and long_b[1] > Fraction(1, 2) and long_mix[0] > Fraction(1, 2))
    low, high, mix_mean = out["crossings"]
    check("A's share: the mix wins strictly between the crossings and loses outside",
          mix_mean(low[0]) < 0 < mix_mean(low[1]) and mix_mean(high[0]) > 0 > mix_mean(high[1])
          and mix_mean(Fraction(1, 20)) < 0 and mix_mean(Fraction(9, 10)) < 0)
    check("the crossings are far from a rounding boundary (floats agree)",
          far_from_boundary(100 * low[0], 1) and far_from_boundary(100 * high[0], 1))

    # 16: the schedules, round by round and in their cycles
    ab_played, aabb_played, ab_cycle, aabb_cycle, (from2, from1), plain_sched = out["schedules"]
    line = Fraction(49, 130)
    check("a B round breaks even with 49/130 of the crowd on corner 0, wins below it and loses above, "
          "however corners 1 and 2 split the rest",
          all(gain(pub_b, (line, s, 1 - line - s)) == 0 for s in (Fraction(0), Fraction(1, 5), 1 - line))
          and gain(pub_b, (line - Fraction(1, 100), 0, 1 - line + Fraction(1, 100))) > 0
          > gain(pub_b, (line + Fraction(1, 100), 0, 1 - line - Fraction(1, 100))))
    check("an A round loses exactly a cent at any shares (A's coin is the same at every corner)",
          all(gain(pub_a, v) == Fraction(-1, 100) for v in ab_played[:6] + ab_cycle))
    for label, sched, phases in (("AB", schedule("AB", PUBLISHED), ab_cycle),
                                 ("AABB", schedule("AABB", PUBLISHED), aabb_cycle)):
        returns = vecmat(phases[-1], step_matrix(sched[-1])) == phases[0]
        moved = all(vecmat(phases[k], step_matrix(sched[k])) == phases[k + 1] for k in range(len(sched) - 1))
        avg = sum(gain(g, v) for g, v in zip(sched, phases)) / len(sched)
        check(f"{label}: the cycle's shares come back after a period and its rounds average to the "
              "schedule's per round played", returns and moved and avg == periodic(sched)[0])
    ab_b = [v[0] for k, v in enumerate(ab_played) if k % 2 == 1]
    check(f"16_6: from B's settled crowd, alternation's B rounds meet strictly more of the crowd on corner 0 "
          f"each time ({len(ab_b)} B rounds), the first {pct(ab_b[0])} %, and all below the cycle's",
          all(x < y for x, y in zip(ab_b, ab_b[1:])) and all(x < ab_cycle[1][0] for x in ab_b)
          and far_from_boundary(100 * ab_b[0], 1))
    check("16_6: the fifth B round meets less than 49/130 and the sixth more (the visible dots do not "
          "separate the fifth from the line, so the voice names no round)", ab_b[4] < line < ab_b[5])
    tv = [sum(abs(x - y) for x, y in zip(v, ab_cycle[k % 2])) / 2 for k, v in enumerate(ab_played)]
    check("16_6: from the sixth B round on every B round loses: the crowd's total-variation distance to the "
          f"cycle there ({float(100 * tv[11]):.3f} points) is below the cycle's margin over the line "
          f"({float(100 * (ab_cycle[1][0] - line)):.3f}), and it never grows from round to round",
          tv[11] < ab_cycle[1][0] - line and all(y <= x for x, y in zip(tv, tv[1:])))
    aabb_b = [v[0] for k, v in enumerate(aabb_played) if k % 4 in (2, 3)]
    aabb_tv = [sum(abs(x - y) for x, y in zip(v, aabb_cycle[k % 4])) / 2 for k, v in enumerate(aabb_played)]
    check("16_9: AABB's B rounds meet less than 49/130 on corner 0 from the first, in the cycle (36.0 and "
          "33.3 %) and, by the never-growing distance to the cycle, ever after",
          all(x < line for x in aabb_b) and max(aabb_cycle[2][0], aabb_cycle[3][0]) + aabb_tv[-1] < line
          and all(y <= x for x, y in zip(aabb_tv, aabb_tv[1:])))
    check("16_7: B's round leaves the cycle's A shares, and A refills corner 0 with 49.5 % of corner 2 "
          "and 50.5 % of corner 1",
          vecmat(ab_cycle[1], step_matrix(pub_b)) == ab_cycle[0] and from2 + from1 == ab_cycle[1][0])
    shown = [Fraction(x) for x in (pct(ab_cycle[0][2], 2), pct(ab_cycle[0][1], 2), pct(from2, 2), pct(from1, 2),
                                   pct(ab_cycle[1][0], 2))]
    check("16_7's printed sums: 49.5 % × 68.95 % rounds to 34.13 %, 50.5 % × 7.59 % to 3.83 %, and 34.13 % + "
          "3.83 % is 37.96 %",
          fixed(PUBLISHED[0] * shown[0], 2) == pct(from2, 2) and fixed((1 - PUBLISHED[0]) * shown[1], 2)
          == pct(from1, 2) and shown[2] + shown[3] == shown[4])
    averaged = tuple((x + y) / 2 for x, y in zip(pub_a, pub_b))
    check("control 16_8: averaging AB's two chances at each corner gives the coin-toss mix's coins, which "
          "predict a win; AB played round by round loses",
          averaged == pub_mix and mean(averaged) > 0 > periodic(schedule("AB", PUBLISHED))[0])
    check("16_9, 16_10: two and two wins, a little less than the coin-toss mix",
          0 < periodic(schedule("AABB", PUBLISHED))[0] < mean(pub_mix))
    plain_bad = 0
    for p in (Fraction(k, 100) for k in range(1, 100, 7)):
        for q in (Fraction(k, 100) for k in range(2, 100, 9)):
            for pattern in ("AB", "BA", "AABB", "ABB", "AAB", "ABAB"):
                value, _ = periodic(schedule(pattern, (p, q, q)))
                want = (pattern.count("A") * (2 * p - 1) + pattern.count("B") * (2 * q - 1)) / len(pattern)
                plain_bad += value != want
    check("16_11: with coins that ignore the money every schedule averages its count of each coin, "
          "whatever the order (a sweep of coins and six patterns), so half and half loses exactly 3 cents",
          plain_bad == 0 and all(x == Fraction(-3, 100) for x in plain_sched))
    # 16_12 to 16_14: the curve's two crossings, rounded and exact
    edges, edge_means, whole_means = out["share_edges"]
    lin = [(pub_b[i], pub_a[i] - pub_b[i]) for i in range(3)]
    fwd_poly, back_poly = [Fraction(1)], [Fraction(1)]
    for c0, c1 in lin:
        fwd_poly = poly_mul(fwd_poly, [c0, c1])
        back_poly = poly_mul(back_poly, [1 - c0, -c1])
    verdict = [x - y for x, y in zip(fwd_poly, back_poly)]
    check("16_13: the mix's verdict at A's share s, p0 p1 p2 − q0 q1 q2, a cubic in s, has exactly two "
          "roots between 0 and 1 (Sturm), negative at both ends",
          sturm_roots(verdict, Fraction(0), Fraction(1)) == 2 and poly_at(verdict, 0) < 0
          and poly_at(verdict, 1) < 0)
    check("16_13: the verdict's sign is the mix's sign at every edge share and both points",
          all((poly_at(verdict, s) > 0) == (mix_mean(s) > 0) for s in edges + list(SHARES)))
    check("16_14: rounded crossings are not the strict bounds: 7.0 % loses, 7.1 % wins, 84.7 % wins, 84.8 % "
          "loses, and 7 % and 85 % both lose",
          [(m > 0) - (m < 0) for m in edge_means] == [-1, 1, 1, -1] and all(m < 0 for m in whole_means))
    check("16_14: the crossings at two decimals and at none are far from a rounding boundary (floats agree)",
          all(far_from_boundary(100 * x[0], 2) and far_from_boundary(100 * x[0], 0) for x in (low, high)))

    # 17: what the exact distributions are
    dists, cats, top, (b7, t7), windows, moms, sds, ratio, root10, costs, ahead7, lucky = out["finite"]
    check("17_3: after an even number of rounds every total is even (odd mass 0), and each distribution "
          "holds the whole crowd",
          all(k % 2 == 0 for d, _ in dists + [(b7, t7)] for k in d)
          and all(sum(d.values()) == t for d, t in dists + [(b7, t7)]))
    check("17_3: the window from −100 to +140 dollars holds every total after 100 rounds and all but less "
          "than 0.01 % after 1,000 (the mix from 0, B from 7)",
          windows[0] == 1 and all(1 - w < Fraction(1, 10**4) for w in windows[1:]))
    d100, t100 = dists[0]
    check(f"17_3: the tallest bar after 100 rounds is unique, at {top} dollars",
          sum(1 for w in d100.values() if w == d100[top]) == 1)
    settled = csv(pct(w) for w in shares(pub_mix))
    sums = [[Fraction(sum(w for k, w in d.items() if (CROWD_START + k) % MODULUS == c), t) for c in range(3)]
            for d, t in dists]
    check("17_3: added up by corner, the bars after 100 and after 1,000 rounds are the mix's settled crowd "
          "at one decimal, the most on corner 2, far from a rounding boundary",
          all(csv(pct(x) for x in s) == settled and s[2] == max(s) for s in sums)
          and all(far_from_boundary(100 * x, 1) for s in sums for x in s))
    corner2 = [k for k in d100 if (CROWD_START + k) % MODULUS == 2 and Fraction(d100[k], t100) > Fraction(1, 500)]
    check("17_3: single corner-2 bars are not all taller than their neighbours, so only the sums are claimed",
          any(d100[k] < d100.get(k - 2, 0) or d100[k] < d100.get(k + 2, 0) for k in corner2))
    old = out["horizon"]
    check("17_4: two exact programmes agree on ahead, behind and level after 100 and 1,000 rounds of the mix",
          cats[0] == old[0][:3] and cats[1][0] == old[1][0])
    check("17_13: neither horizon makes being ahead certain (behind > 0), and after one round only 29.5 % "
          "are ahead, so 'always better than even' is false",
          all(c[1] > 0 for c in cats)
          and categories(*net_distribution(pub_mix, 1, CROWD_START))[0] == pub_mix[0] < Fraction(1, 2))
    check("17_5: starting on corner 0 costs the crowd the same once, at 100 and at 1,000 rounds (within "
          "half a cent), and 1.57 − 0.28 = 1.29 as printed",
          abs(costs[0] - costs[1]) < Fraction(1, 200)
          and Fraction(fixed(costs[0] + moms[0][0], 2)) - Fraction(fixed(costs[0], 2))
          == Fraction(fixed(moms[0][0], 2)))
    check("17_2: the crowd starts on corner 0", CROWD_START % MODULUS == 0)
    check("17_7: more of the crowd is ahead after 1,000 rounds than after 100, and fewer level",
          cats[1][0] > cats[0][0] and cats[1][2] < cats[0][2])
    rate = (moms[1][0] - moms[0][0]) / (LONG_HORIZON - HORIZON)
    check(f"17_8: from 100 to 1,000 rounds the crowd's average grows by {cents(rate, 4)} cents a round, "
          "the mix's 1.57 to two decimals", cents(rate) == cents(mean(pub_mix)))
    check("17_9: the spread's ratio is close to √10 but not equal (the square root is a long-run law)",
          dec_fixed(ratio, 2) == dec_fixed(root10, 2) and dec_fixed(ratio, 4) != dec_fixed(root10, 4))
    floats = [(100 * x, 1) for x in list(cats[1]) + windows + [ahead7, lucky, Fraction(d100[top], t100)]]
    floats += [(m, 2) for m, _ in moms] + [(costs[0], 2), (100 * ab_played[1][0], 1)]
    check("every new float fact is far from a rounding boundary (Rust's floats print the same digits)",
          all(far_from_boundary(x, p) for x, p in floats))
    check(f"17_10: from 7 dollars, B alone for 1,000 rounds: {pct(ahead7)} % ahead, {pct(lucky)} % at "
          "least +16 (more than one in eight)", Fraction(1, 8) < lucky < ahead7 < Fraction(1, 2))

    # the player's runs: what the voice says about them
    runs = out["runs"]
    a_path, a_zero = runs["A"]
    b_path, b_zero = runs["B"]
    m_path, m_zero = runs["mix"]
    a_net, b_net, m_net = (p[-1] - START for p in (a_path, b_path, m_path))
    check("01_5: the run's first eight rounds of A end above the start", a_path[FIRST] > START)
    check("01_9: A's run ends within 100 dollars of a thousand below", abs(a_net + ROUNDS // 100) <= 100)
    check("03_4: B's run ends below its start, less far than A's", a_net < b_net < 0)
    check("03_5: B's run spends more than 38 % of its rounds at a multiple of three",
          Fraction(b_zero, ROUNDS) > Fraction(38, 100))
    check("04_6: the mix's run ends well over a thousand above (at least 1,250)", m_net >= 1250)
    check("05_2: the mix's run spends about 34.5 % at a multiple of three (34.25 to 34.75)",
          Fraction(3425, 10000) <= Fraction(m_zero, ROUNDS) <= Fraction(3475, 10000))
    check("07_3: A's run spends about 33.5 % at a multiple of three (33.25 to 33.75)",
          Fraction(3325, 10000) <= Fraction(a_zero, ROUNDS) <= Fraction(3375, 10000))
    check("17_1: B's run is 16 dollars ahead after 1,000 rounds", b_path[SHORT] - START == 16)
    check("every run moves one dollar a round", all(abs(x - y) == 1 for p in (a_path, b_path, m_path) for x, y in zip(p, p[1:])))

    # negative controls: the checks above can fail
    pb = shares(pub_b)
    swapped_weights = [pb[2], pb[1], pb[0]]
    check("control: B's weights with corners 0 and 2 swapped are not stationary",
          vecmat(swapped_weights, step_matrix(pub_b)) != swapped_weights)
    check("control: no crowd smaller than 26 has whole-number flows",
          all(any(x.denominator != 1 for x in sum(flows(r_b, shares(r_b), k), [])) for k in range(1, 26)))
    halfway = (mean(pub_a) + mean(pub_b)) / 2
    check("control: averaging the two games' averages predicts a loss, the mix wins", halfway < 0 < mean(pub_mix))
    die = Fraction(1, 3) * ROUND[1] + Fraction(2, 3) * ROUND[2]
    check("control: the same coins chosen by a die win (8/15); chosen by the money, fair", die > Fraction(1, 2) == win_at(r_b, shares(r_b)))
    check("control: rounding half to even would show 5.85 as 5.8; half away from zero shows 5.9",
          round(Fraction(585, 100), 1) == Fraction(58, 10) and fixed(Fraction(585, 100), 1) == "5.9")
    other = run("B", PUBLISHED, SHORT, seed=SEED + 1)[0]
    check("control: another seed gives another path (the run is one draw, not the law)", other != b_path[:SHORT + 1])


def main():
    out = facts()
    cross_checks(out)
    print(f"# {_CHECKS[0] - len(_FAILED)}/{_CHECKS[0]} cross-checks passed", file=sys.stderr)
    return 1 if _FAILED else 0


if __name__ == "__main__":
    sys.exit(main())
