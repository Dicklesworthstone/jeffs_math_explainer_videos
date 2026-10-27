#!/usr/bin/env python3
"""Facts oracle for "How Much Can Five Serial Numbers Reveal?".

Run from this folder (`./x german_tank facts-check` will, once root registers
the crate):

    python3 -I -B oracle.py

stdout: one `KEY<TAB>VALUE` line per fact, the keys `tank facts` prints
(src/facts.rs) once the binary exists. stderr: cross-checks (lines starting
with "#") and a summary; exit 1 if any cross-check failed.

The model throughout: boxes numbered 1, 2, ..., N without skips; a handful is
k different boxes; every k-box handful is equally likely. A recipe turns a
handful into an estimate of N.

How values are obtained, independently of the Rust model (which uses closed
forms: P(M = m) = C(m-1, k-1)/C(N, k), E[M] = k(N+1)/(k+1), and a sum-of-
subsets table for the doubling recipe):
  * every distribution over handfuls by brute-force enumeration of
    itertools.combinations, including all 13,019,909 handfuls of five from 71;
  * where enumeration is too large (N = 100 to 200 or more), the chance that
    all k numbers are at most m as a sequential draw product, prod (m-i)/(N-i),
    never as a ratio of binomials, and checked against enumeration on small N;
  * the unbiased function of the largest number by solving the triangular
    system E[g(M)] = N count by count, never from the closed form;
  * every always/never sentence of the script at its edges, by enumeration,
    plus negative controls that must fail (a recipe without the -1, a recipe
    that divides by k+1, equal per-box chances without equal handfuls).
"""
import sys
from fractions import Fraction as F
from itertools import combinations, permutations
from math import comb, isqrt

_EMITTED = {}
_FAILED = []
_CHECKS = [0]


def emit(key, value):
    if key in _EMITTED:
        raise SystemExit(f"fact {key} emitted twice")
    _EMITTED[key] = value
    print(f"{key}\t{value}")


def check(name, ok):
    _CHECKS[0] += 1
    print(f"# {'ok  ' if ok else 'FAIL'} {name}", file=sys.stderr)
    if not ok:
        _FAILED.append(name)


def q(x):
    """A rational as the facts print it: an integer, or a/b in lowest terms."""
    x = F(x)
    return str(x.numerator) if x.denominator == 1 else f"{x.numerator}/{x.denominator}"


def csv(xs):
    return ",".join(q(x) if isinstance(x, F) else str(x) for x in xs)


def flag(b):
    return "true" if b else "false"


def decimal1(x):
    """x to one decimal place, only when that is exact (28.6, 84.2)."""
    x = F(x)
    if (x * 10).denominator != 1:
        raise SystemExit(f"{x} has no exact one-decimal form")
    whole, tenth = divmod(int(x * 10), 10)
    return f"{whole}.{tenth}"


def nearest(x):
    """Round half away from zero (what the voice says for "about")."""
    x = F(x)
    return int(x + F(1, 2)) if x >= 0 else -int(-x + F(1, 2))


def about1(x):
    """A positive x rounded half up to one decimal place (a panel's "≈ 166.8")."""
    if F(x) < 0:
        raise SystemExit(f"{x}: about1 is for positive values")
    tenths = nearest(F(x) * 10)
    return f"{tenths // 10}.{tenths % 10}"


def decimal2(x):
    """x to two decimal places, only when that is exact (4.25, 2.25)."""
    x = F(x)
    if (x * 100).denominator != 1 or x < 0:
        raise SystemExit(f"{x} has no exact positive two-decimal form")
    whole, hundredths = divmod(int(x * 100), 100)
    return f"{whole}.{hundredths:02d}"


def exact_sqrt(x):
    """The square root of a rational that is a perfect square (9/4 -> 3/2)."""
    x = F(x)
    a, b = isqrt(x.numerator), isqrt(x.denominator)
    if a * a != x.numerator or b * b != x.denominator:
        raise SystemExit(f"{x} is not a perfect square")
    return F(a, b)


def sqrt_about1(x):
    """The square root of a positive x rounded half up to one decimal place,
    exactly: the tenths t with ((2t - 1)/20)^2 <= x < ((2t + 1)/20)^2, from
    the integer square root of 400x (a panel's "≈ 11.7")."""
    if F(x) < 0:
        raise SystemExit(f"{x}: sqrt_about1 is for positive values")
    r = isqrt(int(F(x) * 400))
    odd = r if r % 2 == 1 else r - 1
    tenths = (odd + 1) // 2
    return f"{tenths // 10}.{tenths % 10}"


# ------------------------------------------------------------------ recipes
# Each recipe is written from its spoken definition, not from a formula sheet.


def gaps_of(hand):
    """The unseen numbers below the smallest and between consecutive seen ones."""
    s = sorted(hand)
    prev, out = 0, []
    for x in s:
        out.append(x - prev - 1)
        prev = x
    return out


def gap_recipe(hand):
    """The largest number plus the average of the gaps below it."""
    g = gaps_of(hand)
    return max(hand) + F(sum(g), len(g))


def doubling_recipe(hand):
    """Twice the average of the numbers, minus one."""
    return 2 * F(sum(hand), len(hand)) - 1


def doubling_of_sum(s, k):
    """The doubling recipe for k numbers that add up to s."""
    return 2 * F(s, k) - 1


def spread_recipe(hand):
    """Numbering start unknown: the numbers from smallest to largest, plus one
    average inner gap at each end."""
    s = sorted(hand)
    inner = [b - a - 1 for a, b in zip(s, s[1:])]
    return (s[-1] - s[0] + 1) + 2 * F(sum(inner), len(inner))


def smallest_recipe(hand):
    """(k + 1) times the smallest, minus one: unbiased, a negative control for
    'no unbiased recipe misses less'."""
    return (len(hand) + 1) * min(hand) - 1


def handfuls(n, k):
    return combinations(range(1, n + 1), k)


# --------------------------------------------------- handfuls drawn by computer
# The handfuls the video draws (chapters 02 and 08) come from SplitMix64, one
# stream per experiment, seeded with SEED plus the experiment's number, and
# are shown in the order drawn: the first ones, never chosen. src/model.rs
# implements the same generator separately; `tank facts` must print the same
# handfuls.

SEED = 20261024
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

    def below(self, n):
        """A whole number in 0..n-1, every one equally likely: values from the
        incomplete last block of n are thrown away and redrawn."""
        limit = (1 << 64) - (1 << 64) % n
        while True:
            x = self.next()
            if x < limit:
                return x % n


def drawn_handfuls(stream, n, k, count):
    """`count` fair handfuls of k different boxes from 1..n, each drawn one
    box at a time (a repeat is redrawn), sorted."""
    rng = SplitMix64(SEED + stream)
    out = []
    for _ in range(count):
        seen = []
        while len(seen) < k:
            x = 1 + rng.below(n)
            if x not in seen:
                seen.append(x)
        out.append(tuple(sorted(seen)))
    return out


def mean_over(n, k, recipe):
    total, count = F(0), 0
    for h in handfuls(n, k):
        total += recipe(h)
        count += 1
    return total / count


def mae_over(n, k, recipe):
    total, count = F(0), 0
    for h in handfuls(n, k):
        total += abs(recipe(h) - n)
        count += 1
    return total / count


def mse_over(n, k, recipe):
    total, count = F(0), 0
    for h in handfuls(n, k):
        total += (recipe(h) - n) ** 2
        count += 1
    return total / count


# -------------------------------------------------- distributions at scale


def max_and_sum_counts(n, k):
    """Brute force: how many handfuls have each largest number, each sum, and
    each pair (largest, sum)."""
    by_max, by_sum, joint, count = {}, {}, {}, 0
    for h in handfuls(n, k):
        m, s = h[-1], sum(h)
        by_max[m] = by_max.get(m, 0) + 1
        by_sum[s] = by_sum.get(s, 0) + 1
        joint[m, s] = joint.get((m, s), 0) + 1
        count += 1
    return by_max, by_sum, joint, count


def all_at_most(m, n, k):
    """Chance that all k numbers of a fair handful from 1..n are at most m,
    drawing them one at a time: (m/n)((m-1)/(n-1))..."""
    p = F(1)
    for i in range(k):
        if m - i <= 0:
            return F(0)
        p *= F(m - i, n - i)
    return p


def upper_bound(m, k, level):
    """The largest count n for which a handful topped by m or less still has
    chance at least `level`: the top of the one-sided range."""
    n = m
    while all_at_most(m, n + 1, k) >= level:
        n += 1
    return n


def risks_of_largest(n, k, recipe):
    """(average, average miss, average squared miss) of a recipe of the
    largest number over the fair k-handfuls from 1..n, with the chance that
    the largest is m taken as all_at_most(m) - all_at_most(m - 1)."""
    mean = mae = mse = F(0)
    below = F(0)
    for mm in range(k, n + 1):
        upto = all_at_most(mm, n, k)
        p, below = upto - below, upto
        e = recipe(mm)
        mean += p * e
        mae += p * abs(e - n)
        mse += p * (e - n) ** 2
    return mean, mae, mse


# --------------------------------------------------- the staircase, proved
# Polynomials in the count N and the handful size k, as {(power of N, power
# of k): coefficient}; an identity of polynomials holds for every N and k.


def poly(*terms):
    out = {}
    for c, i, j in terms:
        out[i, j] = out.get((i, j), F(0)) + F(c)
    return {e: c for e, c in out.items() if c != 0}


def padd(a, b, sign=1):
    out = dict(a)
    for e, c in b.items():
        out[e] = out.get(e, F(0)) + sign * c
    return {e: c for e, c in out.items() if c != 0}


def pmul(a, b):
    out = {}
    for (i, j), c in a.items():
        for (u, v), d in b.items():
            out[i + u, j + v] = out.get((i + u, j + v), F(0)) + c * d
    return {e: c for e, c in out.items() if c != 0}


def falling(top, kk):
    """top (a polynomial) times (top - 1) ... times (top - kk + 1)."""
    out = poly((1, 0, 0))
    for i in range(kk):
        out = pmul(out, padd(top, poly((-i, 0, 0))))
    return out


def recurrence_proof():
    """The algebra behind 'every count forces the next value', checked as
    identities of polynomials, not at sample counts.

    Unbiased at N - 1 and at N, the totals of g over all handfuls are
    (N - 1) C(N - 1, k) and N C(N, k); the handfuls of N boxes are those of
    N - 1 boxes plus the C(N - 1, k - 1) handfuls that contain N, whose
    largest is N. So C(N - 1, k - 1) g(N) = N C(N, k) - (N - 1) C(N - 1, k):
    g(N) has a coefficient of at least 1 and is fixed by the values below
    it, and g(k) = k by the single handful of k boxes. Dividing by
    C(N - 1, k - 1) with
      (a) k C(N, k) = N C(N - 1, k - 1)        [k! C(N, k) = N (N - 1)...(N - k + 1)]
      (b) k C(N - 1, k) = (N - k) C(N - 1, k - 1)
    gives k g(N) = N^2 - (N - 1)(N - k) = (k + 1) N - k        (c)
    = k (N + (N - k)/k): the gap recipe at largest N             (d)
    and the share of handfuls containing N, C(N - 1, k - 1)/C(N, k) = k/N by
    (a). (a) and (b) are checked for k = 1..12 as polynomials in N (their
    falling factorials, times k!); (c) and (d) in N and k together."""
    n_ = poly((1, 1, 0))
    k_ = poly((1, 0, 1))
    one = poly((1, 0, 0))
    ok = True
    for kk in range(1, 13):
        # k! C(N, k) = falling(N, k); k! C(N - 1, k - 1) = k falling(N - 1, k - 1)
        n_minus_1 = padd(n_, one, -1)
        lhs_a = pmul(poly((kk, 0, 0)), falling(n_, kk))
        rhs_a = pmul(n_, pmul(poly((kk, 0, 0)), falling(n_minus_1, kk - 1)))
        lhs_b = pmul(poly((kk, 0, 0)), falling(n_minus_1, kk))
        rhs_b = pmul(padd(n_, poly((kk, 0, 0)), -1), pmul(poly((kk, 0, 0)), falling(n_minus_1, kk - 1)))
        ok = ok and lhs_a == rhs_a and lhs_b == rhs_b
    n_sq = pmul(n_, n_)
    c_lhs = padd(n_sq, pmul(padd(n_, one, -1), padd(n_, k_, -1)), -1)
    c_rhs = padd(pmul(padd(k_, one), n_), k_, -1)
    d_rhs = padd(pmul(k_, n_), padd(n_, k_, -1))
    return ok and c_lhs == c_rhs and c_rhs == d_rhs


def forced_values(k, top):
    """Solve E[g(M)] = n for n = k, k+1, ..., top, one unknown g(n) at a time:
    the only recipe of the largest number that is unbiased at every count."""
    g = {}
    for n in range(k, top + 1):
        weights = {}
        for h in handfuls(n, k):
            weights[h[-1]] = weights.get(h[-1], 0) + 1
        total = sum(weights.values())
        known = sum(w * g[m] for m, w in weights.items() if m != n)
        g[n] = (n * total - known) / F(weights[n])
    return g


# ------------------------------------------------------------------- facts


def facts():
    # 01: the five boxes, and the one certain thing
    hand = (14, 27, 33, 51, 60)
    k, m = len(hand), max(hand)
    emit("G-F1.numbers", csv(hand))
    first = 1
    emit("G-F1.first", first)
    # counting up with nothing skipped: the box made after the first
    emit("G-F1.second", first + 1)
    emit("G-F1.handful", k)
    emit("G-F1.largest", m)
    emit("G-F1.at_least", min(n for n in range(1, 10 * m) if all(x <= n for x in hand)))
    thousand = 1000
    emit("G-F1.thousand", thousand)
    # the zoomed line's marks: the first number, then each quarter of the
    # extent short of its end (which has its own label)
    emit("G-F1.zoom_marks", csv([first] + list(range(thousand // 4, thousand, thousand // 4))))

    # 02: what fair handfuls from a big and a small company look like
    big, small = 1000, 70
    big_hands = drawn_handfuls(1, big, k, 10)
    small_hands = drawn_handfuls(2, small, k, 10)
    emit("G-F2.seed", SEED)
    emit("G-F2.big", big)
    emit("G-F2.small", small)
    emit("G-F2.draws", len(big_hands))
    emit("G-F2.big_handfuls", ";".join(csv(h) for h in big_hands))
    emit("G-F2.big_lowest_largest", min(h[-1] for h in big_hands))
    # the mark every row from a thousand reaches past (02_5's "six
    # hundred"): a round number the voice says, under the lowest largest
    over = 600
    emit("G-F2.over", over)
    emit("G-F2.big_largest_over_600", flag(all(h[-1] > over for h in big_hands)))
    emit("G-F2.small_handfuls", ";".join(csv(h) for h in small_hands))
    p_big = all_at_most(m, big, k)
    emit("G-F2.big_all_at_most_60", q(p_big))
    emit("G-F2.big_all_at_most_60_under_one_in_a_million", flag(p_big < F(1, 10**6)))
    emit("G-F2.small_all_at_most_60.percent", nearest(100 * all_at_most(m, small, k)))
    # how many of the ten rows shown from seventy stay at sixty or below
    # (02_7's "Three of these ten do"): one draw's count, not the chance
    emit("G-F2.small_fit_shown", sum(1 for h in small_hands if h[-1] <= m))
    # the average spacing of neighbouring numbers, counting from zero: one
    # more than a gap's average size, (N - k)/(k + 1) + 1; the round number
    # 02_8 says the big company's spacing is over
    spacing_over = 150
    emit("G-F2.spacing_over", spacing_over)
    emit("G-F2.big_spacing", q(F(big - k, k + 1) + 1))
    emit("G-F2.big_spacing_over_150", flag(F(big - k, k + 1) + 1 > spacing_over))
    emit("G-F2.small_spacing", q(F(small - k, k + 1) + 1))
    emit("G-F2.small_spacing_about", nearest(F(small - k, k + 1) + 1))
    emit("G-F2.friends_spacing", q(F(m, k)))
    # what the scenes display: the five that leave the bin (the line's 84
    # dots, stream 0), each shown row's own average spacing (its largest over
    # five), the panel's rounded averages, the one-in-a-million threshold
    emit("G-F2.bin_dots", 84)
    emit("G-F2.bin_handful", csv(drawn_handfuls(0, 84, k, 1)[0]))
    emit("G-F2.big_row_spacing", decimal1(F(big_hands[0][-1], k)))
    emit("G-F2.small_row_spacing", decimal1(F(small_hands[0][-1], k)))
    emit("G-F2.big_spacing_shown", about1(F(big - k, k + 1) + 1))
    emit("G-F2.small_spacing_shown", about1(F(small - k, k + 1) + 1))
    emit("G-F2.million", 10**6)

    # 03: the gaps, one at a time, and the first estimate
    g = gaps_of(hand)
    est = gap_recipe(hand)
    emit("G-F3.gaps", csv(g))
    emit("G-F3.unseen", sum(g))
    emit("G-F3.gap_count", len(g))
    emit("G-F3.average_gap", q(F(sum(g), k)))
    emit("G-F3.estimate", q(est))
    check("03: the five gaps hold 55 boxes, 11 a gap, and 60 + 11 = 71", g == [13, 12, 5, 17, 8] and est == 71)
    # what 03_1 to 03_3 say, one gap at a time: the first two numbers and
    # the gaps below and between them, then the last three gaps
    emit("G-F3.first_seen", hand[0])
    emit("G-F3.second_seen", hand[1])
    emit("G-F3.first_gap", g[0])
    emit("G-F3.second_gap", g[1])
    emit("G-F3.last_gaps", csv(g[2:]))
    # a company whose count we know (chapter 02's seventy): the first drawn
    # handful's gaps, its top gap among them
    first = small_hands[0]
    emit("G-F3.known_handful", csv(first))
    emit("G-F3.known_gaps", csv(gaps_of(first)))
    emit("G-F3.known_top_gap", small - first[-1])
    # how many gaps lie above the largest number (03_5, 03_6, 03_8: "one
    # more"): counted on the known handful, where the top gap can be seen
    with_top = [b - a - 1 for a, b in zip((0,) + tuple(first), tuple(first) + (small + 1,))]
    emit("G-F3.top_gaps", len(with_top) - len(gaps_of(first)))
    check("03: the known handful's gaps and its one top gap fill seventy",
          sum(with_top) + k == small and with_top[-1] == small - first[-1])

    # 04: what the count might be (04_1: sixty, seventy-one, five hundred),
    # the bin's fair handfuls lined up as "every handful of five" (the
    # chapter 02 stream drawn on past its first handful), the edge handful
    # 1..5, and the recipe in numbers
    far = 500
    emit("G-F4.far_count", far)
    check("04: each count named is possible, at least the largest, in order", m <= est < far)
    ghosts = drawn_handfuls(0, 84, k, 6)
    emit("G-F4.ghost_handfuls", ";".join(csv(h) for h in ghosts))
    check("04: the rows open with the bin's own handful, six different fair handfuls of the 84 dots",
          ghosts[0] == drawn_handfuls(0, 84, k, 1)[0] and len(set(ghosts)) == 6
          and all(len(set(h)) == k and 1 <= h[0] and h[-1] <= 84 for h in ghosts))
    edge = (1, 2, 3, 4, 5)
    emit("G-F4.edge_handful", csv(edge))
    emit("G-F4.edge_estimate", q(gap_recipe(edge)))
    emit("G-F4.edge_gaps", csv(gaps_of(edge)))
    # 04_5's sentence, from the gaps' definition: for every handful of five
    # the gaps below the largest hold the largest minus five
    check("04: the gaps below the largest hold the largest minus k (k<=5, N<=14)",
          all(sum(gaps_of(h)) == max(h) - kk for kk in range(1, 6) for n in range(kk, 15) for h in handfuls(n, kk)))

    # 05: the 71-box company's handfuls, counted as ordered draws of five
    # over the orders of five (08's brute force recounts them), the whole
    # millions 05_1 says and that there are more
    ordered, orders = 1, 1
    for i in range(k):
        ordered *= 71 - i
        orders *= i + 1
    all71 = ordered // orders
    check("05: ordered draws of five divide evenly by the orders of five", ordered % orders == 0)
    millions = all71 // 10**6
    emit("G-F5.company", 71)
    emit("G-F5.handfuls", all71)
    emit("G-F5.millions", millions)
    emit("G-F5.handfuls_over_millions", flag(all71 > millions * 10**6))
    # 05, 06, 07: the tiny company, five boxes, a pair seen
    tn, tk = 5, 2
    pairs = list(handfuls(tn, tk))
    tiny = [gap_recipe(p) for p in pairs]
    emit("G-F5.boxes", tn)
    emit("G-F5.seen", tk)
    emit("G-F5.pairs", len(pairs))
    emit("G-F5.pair_list", ";".join(csv(p) for p in pairs))
    emit("G-F5.first_pair", csv(pairs[0]))
    emit("G-F5.last_pair", csv(pairs[-1]))
    check("05: the pairs run from the two smallest boxes to the two largest",
          pairs[0] == (1, 2) and pairs[-1] == (tn - 1, tn) and len(pairs) == comb(tn, tk))
    emit("G-F5.estimates", csv(tiny))
    # the table shows a whole estimate as it is and a half to one decimal
    emit("G-F5.estimates_shown", ",".join(q(e) if e.denominator == 1 else decimal1(e) for e in tiny))
    emit("G-F5.average", q(sum(tiny) / len(tiny)))
    emit("G-F5.largest_average", q(F(sum(p[-1] for p in pairs), len(pairs))))
    # the worked pairs, from the line: what is unseen below the largest
    low, high = (1, 2), (2, 5)
    high_unseen = [b for b in range(1, max(high)) if b not in high]
    emit("G-F5.example_low_pair", csv(low))
    emit("G-F5.example_low", q(gap_recipe(low)))
    emit("G-F5.example_high_pair", csv(high))
    emit("G-F5.example_high_largest", max(high))
    emit("G-F5.example_high_unseen", csv(high_unseen))
    emit("G-F5.example_high_unseen_count", len(high_unseen))
    emit("G-F5.example_high_gaps", csv(gaps_of(high)))
    emit("G-F5.example_high", q(gap_recipe(high)))
    emit("G-F5.example_high_decimal", decimal1(gap_recipe(high)))
    check("05: two and five leave three unseen boxes in two non-empty gaps; one and two leave none",
          len(high_unseen) == max(high) - tk and all(x > 0 for x in gaps_of(high))
          and gaps_of(low) == [0, 0] and gap_recipe(low) == max(low))
    # NEGATIVE: adding the unseen count without dividing by the gaps, or
    # dividing by the pairs' size plus one, misses the average of five
    check("NEGATIVE 05: without the share per gap, or over three gaps, the ten don't average five",
          sum(max(p) + sum(gaps_of(p)) for p in pairs) / len(pairs) != tn
          and sum(max(p) + F(sum(gaps_of(p)), tk + 1) for p in pairs) / len(pairs) != tn)
    tiny_gaps = []
    for p in pairs:
        gl = gaps_of(p) + [tn - p[-1]]
        tiny_gaps.append(gl)
    emit("G-F6.gap_lists", ";".join(csv(gl) for gl in tiny_gaps))
    emit("G-F6.unseen", tn - tk)
    emit("G-F6.positions", tk + 1)
    emit("G-F6.position_averages", csv([F(sum(gl[i] for gl in tiny_gaps), len(pairs)) for i in range(tk + 1)]))
    # every split of three unseen boxes into three gaps appears exactly once
    splits = sorted(
        (a, b, tn - tk - a - b) for a in range(tn - tk + 1) for b in range(tn - tk + 1 - a)
    )
    emit("G-F6.one_pair_per_list", flag(sorted(tuple(gl) for gl in tiny_gaps) == splits))
    # the friends: five numbers cut the unseen into six gaps, five of them seen
    emit("G-F6.friends_positions", k + 1)
    emit("G-F6.friends_seen_gaps", len(g))
    # the worked rows' lists, read off the table by pair
    by_pair = {tuple(p): gl for p, gl in zip(pairs, tiny_gaps)}
    emit("G-F6.low_gaps", csv(by_pair[tuple(low)]))
    emit("G-F6.high_gaps", csv(by_pair[tuple(high)]))
    # trading the gaps below and above, written out as the reversed ends
    low_traded = [by_pair[tuple(low)][-1]] + by_pair[tuple(low)][1:-1] + [by_pair[tuple(low)][0]]
    emit("G-F6.low_traded", csv(low_traded))
    partner_of = {}
    for i, gl in enumerate(tiny_gaps):
        t = [gl[-1]] + gl[1:-1] + [gl[0]]
        partner_of[i] = [j for j, other in enumerate(tiny_gaps) if other == t]
    emit("G-F6.low_partner", csv(pairs[partner_of[pairs.index(tuple(low))][0]]))
    emit("G-F6.partners", csv([partner_of[i][0] + 1 for i in range(len(pairs))]))
    columns = [[gl[i] for gl in tiny_gaps] for i in range(tk + 1)]
    emit("G-F6.column_sums", csv([sum(c) for c in columns]))
    for name, c in zip(("below", "between", "above"), columns):
        emit(f"G-F6.{name}_values", csv(sorted(c)))
    emit("G-F6.each_chance", q(F(1, len(pairs))))
    emit("G-F6.top_average", q(F(sum(columns[-1]), len(pairs))))
    check("06: each row has exactly one partner by trading below and above, and partners pair off",
          all(len(v) == 1 for v in partner_of.values())
          and all(partner_of[partner_of[i][0]][0] == i for i in partner_of))
    check("06: the three columns hold the same ten numbers",
          sorted(columns[0]) == sorted(columns[1]) == sorted(columns[2]))
    # NEGATIVE: if a pair's chance grew with its larger number, the hidden
    # top gap would average less than the gap below, so the trade argument
    # needs every pair equally likely
    weight = {tuple(p): p[-1] for p in pairs}
    total_w = sum(weight.values())
    w_avg = [F(sum(weight[tuple(p)] * gl[i] for p, gl in zip(pairs, tiny_gaps)), total_w) for i in range(tk + 1)]
    check("NEGATIVE 06: weighting pairs by their larger number, the top gap averages 3/4, the gap below 9/8",
          w_avg[-1] == F(3, 4) and w_avg[0] == F(9, 8))
    # 07: the general proof's pieces, and the one-box handful
    # each gap's average, read off the tiny company's table and the friends'
    # handful: (count - handful) / (handful + 1)
    emit("G-F7.gap_average_formula", f"(N-{k})/{k + 1}")
    # one box: the largest plus the one gap below it, read as a multiple of m
    one = (7,)
    slope = gap_recipe(one) - gap_recipe((6,))
    emit("G-F7.one_box_recipe", f"{q(slope)}m{q(gap_recipe(one) - slope * 7)}")
    one_box_ok = all(mean_over(n, 1, gap_recipe) == n for n in range(1, 30))
    emit("G-F7.one_box_unbiased", flag(one_box_ok))
    emit("G-F7.general_unbiased", flag(all(mean_over(n, kk, gap_recipe) == n
                                          for kk in range(1, 6) for n in range(kk, 22))))
    # the tiny company again: each pair's guess for its top gap is the
    # average of the gaps it can see; its top gap is what the count leaves
    guesses = [F(sum(gaps_of(p)), len(gaps_of(p))) for p in pairs]
    tops = [tn - p[-1] for p in pairs]
    emit("G-F7.tiny_guesses", csv([q(x) for x in guesses]))
    emit("G-F7.tiny_guesses_shown", ",".join(q(x) if x.denominator == 1 else decimal1(x) for x in guesses))
    emit("G-F7.tiny_tops", csv(tops))
    emit("G-F7.tiny_agree", sum(1 for x, t in zip(guesses, tops) if x == t))
    emit("G-F7.tiny_guess_average", q(sum(guesses) / len(pairs)))
    emit("G-F7.tiny_top_average", q(F(sum(tops), len(pairs))))
    check("07: row by row, estimate - count = guess - top gap in the tiny company",
          all(gap_recipe(p) - tn == x - t for p, x, t in zip(pairs, guesses, tops)))
    # one box: its number, the one gap below it, its two gaps, its estimate
    one_box = (37,)
    emit("G-F7.one_box_handful", len(one_box))
    emit("G-F7.one_box", one_box[0])
    emit("G-F7.one_box_below", gaps_of(one_box)[0])
    emit("G-F7.one_box_gaps", len(gaps_of(one_box)) + 1)
    emit("G-F7.one_box_estimate", q(gap_recipe(one_box)))
    check("07: one box doubles and subtracts one, for every number to 40",
          all(gap_recipe((m,)) == 2 * m - 1 for m in range(1, 41)))

    # 08: a company of 71 boxes, every handful of five (brute force)
    n71 = 71
    by_max, by_sum, joint, count = max_and_sum_counts(n71, k)
    emit("G-F8.count", n71)
    emit("G-F8.handfuls", count)
    check("05: the 71-box handfuls counted as draws equal 08's brute force", all71 == count)
    gap_dist = {}
    for mm, c in by_max.items():
        e = mm + F(mm - k, k)
        gap_dist[e] = gap_dist.get(e, 0) + c
    mean71 = sum(e * c for e, c in gap_dist.items()) / count
    over = sum(c for e, c in gap_dist.items() if e > n71)
    exact = sum(c for e, c in gap_dist.items() if e == n71)
    mae71 = sum(abs(e - n71) * c for e, c in gap_dist.items()) / count
    emit("G-F8.average", q(mean71))
    emit("G-F8.overestimate_share", q(F(over, count)))
    emit("G-F8.overestimate_percent", nearest(100 * F(over, count)))
    emit("G-F8.exactly_right", q(F(exact, count)))
    emit("G-F8.exactly_right_one_in", nearest(F(count, exact)))
    emit("G-F8.highest_estimate", decimal1(max(gap_dist)))
    emit("G-F8.lowest_estimate", q(min(gap_dist)))
    emit("G-F8.average_miss", q(mae71))
    emit("G-F8.average_miss_about", nearest(mae71))
    check("08: the 71-box estimates average 71", mean71 == n71)
    # twenty fresh handfuls from the 71-box company, drawn by computer
    again = drawn_handfuls(3, n71, k, 20)
    again_est = [gap_recipe(h) for h in again]
    emit("G-F8.again_draws", len(again))
    emit("G-F8.again_handfuls", ";".join(csv(h) for h in again))
    emit("G-F8.again_estimates", ",".join(decimal1(e) for e in again_est))
    emit("G-F8.again_lowest", decimal1(min(again_est)))
    emit("G-F8.again_lowest_about", nearest(min(again_est)))
    emit("G-F8.again_highest", decimal1(max(again_est)))
    emit("G-F8.again_average", q(sum(again_est) / len(again_est)))
    emit("G-F8.again_average_about", nearest(sum(again_est) / len(again_est)))
    emit("G-F8.again_exact", sum(1 for e in again_est if e == n71))
    # 08_3: the handfuls counted by their largest number, from the brute
    # force: one stack per largest number, the smallest and the next, and
    # the count itself (said as "over nine hundred thousand")
    emit("G-F8.stacks", len(by_max))
    emit("G-F8.topped_by_smallest", by_max[k])
    emit("G-F8.next_largest", k + 1)
    emit("G-F8.topped_by_next", by_max[k + 1])
    emit("G-F8.topped_by_count", by_max[n71])
    emit("G-F8.topped_by_count_over", 100000 * (by_max[n71] // 100000))
    # 08_6: how far the two sides of 71 miss on average; 08_7: the spread
    overs = [(e, c) for e, c in gap_dist.items() if e > n71]
    unders = [(e, c) for e, c in gap_dist.items() if e < n71]
    overshoot = sum((e - n71) * c for e, c in overs) / sum(c for _, c in overs)
    undershoot = sum((n71 - e) * c for e, c in unders) / sum(c for _, c in unders)
    emit("G-F8.overshoot", q(overshoot))
    emit("G-F8.overshoot_about", nearest(overshoot))
    emit("G-F8.undershoot", q(undershoot))
    emit("G-F8.average_miss_decimal", about1(mae71))
    var71 = sum((e - n71) ** 2 * c for e, c in gap_dist.items()) / count
    emit("G-F8.variance", q(var71))
    emit("G-F8.sd_about", sqrt_about1(var71))

    # 09: how sure; the chance a handful of five tops out at 60 or less
    for n in (71, 100):
        emit(f"G-F9.at_most_60.{n}", q(all_at_most(m, n, k)))
        emit(f"G-F9.at_most_60.{n}.percent", nearest(100 * all_at_most(m, n, k)))
    emit("G-F9.second_count", 100)
    level = F(1, 20)
    top5 = upper_bound(m, k, level)
    emit("G-F9.level", q(level))
    emit("G-F9.level_percent", nearest(100 * level))
    emit("G-F9.confidence_percent", nearest(100 * (1 - level)))
    emit("G-F9.range", csv([m, top5]))
    emit("G-F9.range_top", top5)
    emit("G-F9.first_rare", top5 + 1)
    emit("G-F9.first_rare.percent_below", flag(all_at_most(m, top5 + 1, k) < level))
    # the two counts either side of the cutoff, shown to a tenth of a percent
    for n in (top5, top5 + 1):
        emit(f"G-F9.at_most_60.{n}", q(all_at_most(m, n, k)))
        emit(f"G-F9.at_most_60.{n}.percent1", about1(100 * all_at_most(m, n, k)))
    # 09_8's division at the range's top: handfuls from the first sixty, and
    # from the whole count, as counts (not the product above)
    emit("G-F9.from_largest", comb(m, k))
    emit("G-F9.from_range_top", comb(top5, k))
    ten = 10
    emit("G-F9.ten_boxes", ten)
    emit("G-F9.ten_boxes_range", csv([m, upper_bound(m, ten, level)]))
    emit("G-F9.ten_boxes_top", upper_bound(m, ten, level))
    # 09_5, 09_6: the coverage experiment. With the count fixed at 71, each of
    # 08's twenty drawn handfuls builds its own range the same way
    ranges = [(h[-1], upper_bound(h[-1], k, level)) for h in again]
    covers = [lo <= n71 <= hi for lo, hi in ranges]
    misses = [lo for (lo, _), c in zip(ranges, covers) if not c]
    emit("G-F9.again_ranges", ";".join(csv(r) for r in ranges))
    emit("G-F9.again_cover", sum(covers))
    emit("G-F9.again_miss", len(misses))
    emit("G-F9.again_miss_largest", csv(misses))
    # and over every handful of the 71-box company (the brute-force counts)
    cover71 = sum(F(c, count) for mm, c in by_max.items() if upper_bound(mm, k, level) >= n71)
    emit("G-F9.cover_71", q(cover71))
    emit("G-F9.cover_71_percent", nearest(100 * cover71))
    # coverage: the range built this way holds the true count in at least
    # 95 of every 100 handfuls, for every count 5..150
    worst = None
    for n in range(k, 151):
        cover = sum(F(comb(mm - 1, k - 1), comb(n, k)) for mm in range(k, n + 1) if upper_bound(mm, k, level) >= n)
        worst = cover if worst is None or cover < worst else worst
    emit("G-F9.worst_coverage_at_least_95", flag(worst >= F(19, 20)))

    # 10: the doubling recipe, earned from the middle number. The numbers 1
    # to N balance at their average, the middle; the count read back from
    # the middle, as a multiple of it (two counts give the slope)
    middle = lambda n: F(sum(range(1, n + 1)), n)
    emit("G-F10.middle_71", q(middle(n71)))
    slope = F(n71 - 70, 1) / (middle(n71) - middle(70))
    emit("G-F10.from_middle", f"{q(slope)}*middle{q(n71 - slope * middle(n71))}")
    check("10: every count from 1 to 200 is twice its middle number minus one",
          all(2 * middle(n) - 1 == n for n in range(1, 201)))
    # the doubling recipe read as a multiple of the handful's average
    a1, a2 = (1, 2, 3, 4, 5), (1, 2, 3, 4, 10)
    avg = lambda h: F(sum(h), len(h))
    slope = (doubling_recipe(a2) - doubling_recipe(a1)) / (avg(a2) - avg(a1))
    emit("G-F10.recipe", f"{q(slope)}*average{q(doubling_recipe(a1) - slope * avg(a1))}")
    total = sum(hand)
    emit("G-F10.sum", total)
    emit("G-F10.average", q(F(total, k)))
    emit("G-F10.doubling", q(doubling_recipe(hand)))
    # over every handful of the 71-box company (08's brute force), a fair
    # handful's average averages the middle number
    emit("G-F10.average_of_averages_71", q(sum(F(s, k) * c for s, c in by_sum.items()) / count))
    emit("G-F10.doubling_unbiased",
         flag(all(mean_over(n, kk, doubling_recipe) == n for kk in range(1, 6) for n in range(kk, 17))))
    silly = (2, 3, 4, 5, 60)
    emit("G-F10.others", k - 1)
    emit("G-F10.silly_others", csv(silly[:-1]))
    emit("G-F10.silly_handful", csv(silly))
    emit("G-F10.silly_sum", sum(silly))
    emit("G-F10.silly_average_decimal", decimal1(F(sum(silly), k)))
    emit("G-F10.silly_doubling", q(doubling_recipe(silly)))
    emit("G-F10.silly_doubling_decimal", decimal1(doubling_recipe(silly)))
    emit("G-F10.silly_gap", q(gap_recipe(silly)))
    # how often doubling lands below the handful's own largest number, from
    # the brute-force count of every (largest, sum) pair
    below = F(sum(c for (mx, s), c in joint.items() if doubling_of_sum(s, k) < mx), count)
    emit("G-F10.below_largest_71", q(below))
    emit("G-F10.below_largest_71_one_in", nearest(1 / below))
    emit("G-F10.below_largest_71_percent1", about1(100 * below))
    dbl = {s: doubling_of_sum(s, k) for s in by_sum}
    emit("G-F10.doubling_range_71", csv([min(dbl.values()), max(dbl.values())]))
    # 10's purple bars: the handfuls counted by sum, three sums to a bar so
    # each bar is 1.2 wide like a green one, centred on the gap recipe's
    # value for largest m (bar m holds the sums nearest 3m)
    bars = {}
    for s, c in by_sum.items():
        bars[nearest(F(s, 3))] = bars.get(nearest(F(s, 3)), 0) + c
    tallest = max(bars, key=lambda mm: bars[mm])
    emit("G-F10.bars_71", len(bars))
    emit("G-F10.tallest_bar_71", bars[tallest])
    emit("G-F10.tallest_bar_71_at", q(tallest + F(tallest - k, k)))
    check("10: the purple bars hold every handful once, at 5..115 with no bar empty",
          sum(bars.values()) == count and sorted(bars) == list(range(5, 116)))
    dbl_mean = sum(dbl[s] * c for s, c in by_sum.items()) / count
    dbl_mae = sum(abs(dbl[s] - n71) * c for s, c in by_sum.items()) / count
    dbl_var = sum((dbl[s] - n71) ** 2 * c for s, c in by_sum.items()) / count
    emit("G-F10.doubling_average_71", q(dbl_mean))
    emit("G-F10.doubling_miss_71", q(dbl_mae))
    emit("G-F10.doubling_miss_71_about", nearest(dbl_mae))
    emit("G-F10.doubling_miss_71_decimal", about1(dbl_mae))
    emit("G-F10.gap_miss_71_about", nearest(mae71))
    emit("G-F10.doubling_variance_71", q(dbl_var))
    emit("G-F10.doubling_sd_about", sqrt_about1(dbl_var))
    emit("G-F10.variance_ratio", q(dbl_var / var71))
    check("08: the doubling recipe averages 71 at 71", dbl_mean == n71)
    check("10: at 71 doubling misses more by both widths (14.35 > 9.23; 316.8 > 135.8), each average miss "
          "squared under its variance", dbl_mae > mae71 and dbl_var > var71
          and dbl_mae ** 2 < dbl_var and mae71 ** 2 < var71)
    check("10: the variance of doubling is (N+1)(N-5)/15 at 71, 7/3 of the gap recipe's",
          dbl_var == F(72 * 66, 15) and dbl_var / var71 == F(7, 3))
    check("NEGATIVE 10: the two widths of doubling differ as shown (14.3 average miss, 17.8 standard deviation)",
          about1(dbl_mae) == "14.3" and sqrt_about1(dbl_var) == "17.8" != about1(dbl_mae))

    # 11: given the largest, the others don't depend on the count. First on
    # pairs: companies of 5, 6 and 8 boxes, the pairs topped by 5
    def others_given(n, kk, top):
        seen = {}
        for h in handfuls(n, kk):
            if h[-1] == top:
                seen[h[:-1]] = seen.get(h[:-1], 0) + 1
        tot = sum(seen.values())
        return {o: F(c, tot) for o, c in seen.items()}
    pair_top, companies = tn, (tn, 6, 8)
    topped = {n: others_given(n, tk, pair_top) for n in companies}
    emit("G-F11.pair_top", pair_top)
    emit("G-F11.pair_companies", csv(companies))
    emit("G-F11.pair_more", csv(companies[1:]))
    emit("G-F11.pair_more_pairs", csv([comb(n, tk) for n in companies[1:]]))
    emit("G-F11.pairs_topped", len(topped[tn]))
    emit("G-F11.pair_smaller", csv([o[0] for o in sorted(topped[tn])]))
    each = set(p for g in topped.values() for p in g.values())
    emit("G-F11.pair_each", q(each.pop()) if len(each) == 1 else "unequal")
    emit("G-F11.pair_same_for_every_company", flag(all(g.keys() == topped[tn].keys() for g in topped.values())))
    emit("G-F11.pair_shares", csv([F(sum(1 for h in handfuls(n, tk) if h[-1] == pair_top), comb(n, tk))
                                   for n in companies]))
    # (and on a smaller copy of the friends' case: largest 9 of four seen,
    # counts 9..16)
    base = others_given(9, 4, 9)
    same = all(others_given(n, 4, 9) == base for n in range(9, 17))
    emit("G-F11.others_same_for_every_count", flag(same))
    emit("G-F11.others_sets", comb(m - 1, k - 1))
    emit("G-F11.others_sets_over", 10000 * (comb(m - 1, k - 1) // 10000))
    # the chance that the largest of a fair handful is 60, at three counts:
    # all at most 60 minus all at most 59, as sequential draws; and the
    # chance of one particular group of four given it (one ordered draw of
    # those five boxes, times the 5! orders, over that chance)
    counts11 = (m, n71, 1000)
    emit("G-F11.counts", csv(counts11))
    top_chance = lambda n: all_at_most(m, n, k) - all_at_most(m - 1, n, k)

    def one_handful(n):
        p = F(1)
        for i in range(k):
            p *= F(i + 1, n - i)
        return p
    for n in counts11:
        emit(f"G-F11.top_chance.{n}", q(top_chance(n)))
        emit(f"G-F11.each_group.{n}", q(one_handful(n) / top_chance(n)))
    emit("G-F11.top_chance.60.one_in", nearest(1 / top_chance(m)))
    emit("G-F11.top_chance.71.one_in", nearest(1 / top_chance(n71)))
    emit("G-F11.top_chance.1000.millions", nearest(1 / top_chance(1000) / 10 ** 6))
    check("11: with exactly 60 boxes the largest is 60 with chance 5/60 (the edge count)",
          top_chance(m) == F(k, m))
    check("11: given the largest 60, one group of four has chance 1/C(59, 4) at every count 60..300, "
          "while the chance of the largest 60 falls",
          {one_handful(n) / top_chance(n) for n in range(m, 301)} == {F(1, comb(m - 1, k - 1))}
          and all(top_chance(n + 1) < top_chance(n) for n in range(m, 300)))
    # the gaps below sixty hold 55 numbers whatever the other four are (all
    # 455,126 groups), the strange handful's among them
    emit("G-F11.silly_gaps", csv(gaps_of(silly)))
    emit("G-F11.silly_unseen", sum(gaps_of(silly)))
    check("11: for every group of four below 60, the gaps below 60 total 55 = 60 - 5",
          all(sum(gaps_of(g + (m,))) == m - k for g in combinations(range(1, m), k - 1)))

    # 12: the tiny company's ten pairs again, with the doubling recipe. For a
    # pair, twice the average is the total: the recipe read as a multiple of
    # the pair's total (two pairs give the slope)
    a1, a2 = (1, 2), (1, 5)
    slope = (doubling_recipe(a2) - doubling_recipe(a1)) / (sum(a2) - sum(a1))
    emit("G-F12.pair_doubling", f"{q(slope)}*total{q(doubling_recipe(a1) - slope * sum(a1))}")
    dbl = [doubling_recipe(p) for p in pairs]
    emit("G-F12.doubling_estimates", csv(dbl))
    emit("G-F12.doubling_average", q(sum(dbl) / len(dbl)))
    emit("G-F12.doubling_range", csv([min(dbl), max(dbl)]))
    # the pairs sorted by their largest number, each group's doubling
    # estimates and their average
    groups = {}
    for p in pairs:
        groups.setdefault(p[-1], []).append(doubling_recipe(p))
    tops = sorted(groups)
    avg_of = {t: sum(v) / len(v) for t, v in groups.items()}
    emit("G-F12.groups", len(tops))
    emit("G-F12.group_tops", csv(tops))
    emit("G-F12.group_sizes", csv([len(groups[t]) for t in tops]))
    for t in tops:
        emit(f"G-F12.ends_in_{t}", csv(groups[t]))
    for t in tops[1:]:
        emit(f"G-F12.ends_in_{t}_average", q(avg_of[t]))
    emit("G-F12.ends_in_5_average_decimal", decimal1(avg_of[5]))
    emit("G-F12.group_averages", csv([avg_of[t] for t in tops]))
    emit("G-F12.group_averages_shown", ",".join(q(avg_of[t]) if avg_of[t].denominator == 1 else decimal1(avg_of[t])
                                                for t in tops))
    emit("G-F12.grouped_is_gap_recipe", flag(all(avg_of[p[-1]] == gap_recipe(p) for p in pairs)))
    # a group's average is the same in every company that could have sold
    # its pairs (given the largest, the smaller is equally likely to be any
    # number below it: chapter 11), here for companies of up to 12 boxes
    def group_mean(n, kk, t, recipe):
        vals = [F(recipe(h)) for h in handfuls(n, kk) if h[-1] == t]
        return sum(vals) / len(vals)
    emit("G-F12.group_average_same_for_every_count",
         flag(all(group_mean(n, tk, t, doubling_recipe) == avg_of[t] for t in tops for n in range(t, 13))))
    # each pair is one chance in ten, so a group's chance is its pairs in
    # ten; weighted by those chances the group averages give the ten pairs'
    # average back, and weighted equally they do not
    chance = {t: F(len(groups[t]), len(pairs)) for t in tops}
    emit("G-F12.top_group_chance", q(chance[tops[-1]]))
    weighted = sum(chance[t] * avg_of[t] for t in tops)
    equal = sum(avg_of.values()) / len(tops)
    emit("G-F12.weighted_average", q(weighted))
    emit("G-F12.equal_weight_average", q(equal))
    emit("G-F12.equal_weight_average_shown", decimal2(equal))
    check("12: the groups weighted by their chances average the ten estimates' 5; weighted equally, 17/4",
          weighted == sum(dbl) / len(dbl) == tn and equal == F(17, 4) != tn)
    # the misses as arrows from the count: the one group whose arrows point
    # both ways, and the largest group whose arrows all point one way
    signed = {t: [e - tn for e in groups[t]] for t in tops}
    both = [t for t in tops if min(signed[t]) < 0 < max(signed[t])]
    one_way = [t for t in tops if len(groups[t]) > 1 and t not in both]
    emit("G-F12.cancel_group", both[0])
    emit("G-F12.cancel_misses", csv(signed[both[0]]))
    emit("G-F12.same_side_group", max(one_way))
    emit("G-F12.same_side_misses", csv(signed[max(one_way)]))
    emit("G-F12.same_side_average_miss", q(abs(avg_of[max(one_way)] - tn)))
    check("12: one group's arrows point both ways (topped by 4: -1, 0, 1, its average misses by 0); the group "
          "topped by 5's all point one way (its average misses by their average, 3/2)",
          both == [4] and avg_of[4] == tn
          and abs(avg_of[5] - tn) == sum(abs(x) for x in signed[5]) / len(signed[5]) == F(3, 2))
    d_mae, g_mae = mae_over(tn, tk, doubling_recipe), mae_over(tn, tk, gap_recipe)
    d_mse, g_mse = mse_over(tn, tk, doubling_recipe), mse_over(tn, tk, gap_recipe)
    emit("G-F12.doubling_miss", decimal1(d_mae))
    emit("G-F12.grouped_miss", decimal1(g_mae))
    emit("G-F12.doubling_squared_miss", q(d_mse))
    emit("G-F12.grouped_squared_miss", q(g_mse))
    emit("G-F12.grouped_squared_miss_shown", decimal2(g_mse))
    emit("G-F12.doubling_sd_about", sqrt_about1(d_mse))
    root = exact_sqrt(g_mse)
    emit("G-F12.grouped_sd", q(root))
    emit("G-F12.grouped_sd_decimal", decimal1(root))
    # for squares, the drop is exactly the spread of the estimates inside
    # their groups, weighted by the groups' chances
    inside = sum(chance[t] * sum((e - avg_of[t]) ** 2 for e in groups[t]) / len(groups[t]) for t in tops)
    emit("G-F12.spread_inside", q(inside))
    check("12: averaging inside the groups lowers the average squared miss by exactly the spread inside them "
          "(3 - 9/4 = 3/4), and the average miss from 7/5 to 6/5",
          d_mse - g_mse == inside == F(3, 4) and (d_mae, g_mae) == (F(7, 5), F(6, 5)))

    # 13: unbiased must hold at every count. A recipe that always says 71 is
    # exactly right when the count is 71, and biased at every other count
    always = lambda h: F(n71)
    emit("G-F13.always", n71)
    check("13: always saying 71 averages 71 over the 71-box handfuls, and misses the count on average by "
          "71 - N at every other count (enumerated, counts 5..12)",
          sum(always(None) * c for c in by_max.values()) / count == n71
          and all(mean_over(n, k, always) - n == n71 - n != 0 for n in range(k, 13)))
    # the staircase for pairs: in a company of n boxes, the pairs topped by n
    # carry the one value no smaller company forces
    companies = list(range(tk, tn + 1))
    emit("G-F13.smallest_company", companies[0])
    emit("G-F13.next_company", companies[1])
    emit("G-F13.predict_company", companies[2])
    emit("G-F13.last_company", companies[-1])
    emit("G-F13.companies", csv(companies))
    emit("G-F13.company_pairs", csv([comb(n, tk) for n in companies]))
    emit("G-F13.pairs_from_3", comb(3, tk))
    emit("G-F13.pairs_from_4", comb(4, tk))
    for n in companies[1:]:
        emit(f"G-F13.new_pairs.{n}", sum(1 for h in handfuls(n, tk) if h[-1] == n))
    new_values = {n: len({h[-1] for h in handfuls(n, tk)} - {h[-1] for h in handfuls(n - 1, tk)}) for n in companies}
    emit("G-F13.new_values_per_count", new_values[tn] if len(set(new_values.values())) == 1 else "unequal")
    forced = forced_values(tk, tn)
    emit("G-F13.forced", csv([forced[mm] for mm in range(tk, tn + 1)]))
    emit("G-F13.forced_at_2", q(forced[2]))
    emit("G-F13.forced_at_3_decimal", decimal1(forced[3]))
    emit("G-F13.forced_at_4", q(forced[4]))
    emit("G-F13.forced_at_5", q(forced[5]))
    emit("G-F13.forced_at_5_decimal", decimal1(forced[5]))
    emit("G-F13.forced_shown", ",".join(q(forced[mm]) if forced[mm].denominator == 1 else decimal1(forced[mm])
                                        for mm in range(tk, tn + 1)))
    emit("G-F13.company_total.4", sum(forced[h[-1]] for h in handfuls(4, tk)))
    forced5 = forced_values(k, 40)
    emit("G-F13.forced_is_gap_recipe", flag(all(forced5[mm] == mm + F(mm - k, k) for mm in forced5)))
    # the share of handfuls topped by the count itself: k/N, never zero (60
    # boxes, five seen: chapter 11's 1/12)
    emit("G-F13.share_at_60", q(F(comb(m - 1, k - 1), comb(m, k))))
    emit("G-F13.recurrence_proved", flag(recurrence_proof()))
    # any unbiased recipe, even three times the smaller number minus one,
    # averages inside its groups to the same staircase
    a1, a2 = (1, 5), (2, 5)
    slope = smallest_recipe(a2) - smallest_recipe(a1)
    emit("G-F13.smaller_recipe", f"{q(slope)}*smaller{q(smallest_recipe(a1) - slope * a1[0])}")
    sm = [smallest_recipe(p) for p in pairs]
    emit("G-F13.smaller_estimates", csv(sm))
    emit("G-F13.smaller_range", csv([min(sm), max(sm)]))
    emit("G-F13.smaller_average", q(sum(sm) / len(sm)))
    emit("G-F13.smaller_grouped", csv([group_mean(tn, tk, t, smallest_recipe) for t in tops]))
    emit("G-F13.smaller_miss", q(mae_over(tn, tk, smallest_recipe)))
    emit("G-F13.smaller_miss_decimal", decimal1(mae_over(tn, tk, smallest_recipe)))
    emit("G-F13.smaller_squared_miss", q(mse_over(tn, tk, smallest_recipe)))
    check("13: three times the smaller minus one averages 5 over the ten pairs, and inside its groups gives "
          "the staircase 2, 7/2, 5, 13/2",
          sum(sm) / len(sm) == tn
          and [group_mean(tn, tk, t, smallest_recipe) for t in tops] == [forced[t] for t in tops])
    # a biased recipe of the largest, seven sixths of it minus one: on the
    # friends' boxes, and over every handful of the 71-box company (counted
    # by enumeration above)
    factor = F(k + 2, k + 1)
    shrunk = lambda mm: factor * mm - 1
    emit("G-F13.shrunk_factor", q(factor))
    emit("G-F13.shrunk_recipe", f"{q(factor)}*largest-1")
    emit("G-F13.shrunk_friends", q(shrunk(m)))
    s_mean = sum(shrunk(mm) * c for mm, c in by_max.items()) / count
    s_mae = sum(abs(shrunk(mm) - n71) * c for mm, c in by_max.items()) / count
    s_mse = sum((shrunk(mm) - n71) ** 2 * c for mm, c in by_max.items()) / count
    gap_mse = sum((e - n71) ** 2 * c for e, c in gap_dist.items()) / count
    check("13: the sequential-draw chances of the largest give the 71-box enumeration's averages for the "
          "7/6 recipe", risks_of_largest(n71, k, shrunk) == (s_mean, s_mae, s_mse))
    emit("G-F13.shrunk_average_71", q(s_mean))
    emit("G-F13.shrunk_bias_71", q(s_mean - n71))
    emit("G-F13.shrunk_miss_71", q(s_mae))
    emit("G-F13.shrunk_miss_71_shown", about1(s_mae))
    emit("G-F13.shrunk_squared_71", q(s_mse))
    emit("G-F13.shrunk_squared_71_shown", about1(s_mse))
    emit("G-F13.gap_squared_71_shown", about1(gap_mse))
    emit("G-F13.shrunk_misses_less_squared", flag(s_mse < gap_mse))
    emit("G-F13.shrunk_misses_less", flag(s_mae < mae71))
    # where it loses: with exactly k boxes the gap recipe is always right.
    # Counts 5 to 300 where the 7/6 recipe does not miss less than the gap
    # recipe, squared or plain, and its bias at each, -(N + 1)/36
    edge = tuple(range(1, k + 1))
    emit("G-F13.edge_count", k)
    emit("G-F13.gap_at_edge", q(gap_recipe(edge)))
    emit("G-F13.shrunk_at_edge", q(shrunk(k)))
    emit("G-F13.shrunk_at_edge_shown", about1(shrunk(k)))
    loses_sq, loses_plain, bias_ok = [], [], True
    for n in range(k, 301):
        g_mean_n, g_mae_n, g_mse_n = risks_of_largest(n, k, lambda mm: mm + F(mm - k, k))
        s_mean_n, s_mae_n, s_mse_n = risks_of_largest(n, k, shrunk)
        if s_mse_n >= g_mse_n:
            loses_sq.append(n)
        if s_mae_n >= g_mae_n:
            loses_plain.append(n)
        bias_ok = bias_ok and g_mean_n == n and s_mean_n - n == -F(n + 1, 36)
    check("13: at every count 5..300 the gap recipe averages the count and the 7/6 recipe misses it on "
          "average by -(N + 1)/36", bias_ok)
    check("13: the 7/6 recipe loses at the edge: at 5 boxes it says 29/6 where the gap recipe is exactly 5",
          shrunk(k) == F(29, 6) and gap_recipe(edge) == k and 5 in loses_sq and 5 in loses_plain)
    emit("G-F13.shrunk_loses_squared_at", csv(loses_sq))
    emit("G-F13.shrunk_loses_plain_at", csv(loses_plain))

    # 14: unfair draws. The same five boxes, bought at one shop that only ever
    # received boxes 1 to 71 of the 200 the company made: a fair handful from
    # the shop, so the recipe averages the shop's 71 (every handful of 1..71,
    # enumerated above for chapter 08)
    emit("G-F14.shop_boxes", n71)
    emit("G-F14.made", 200)
    emit("G-F14.shop_handful", csv(hand))
    emit("G-F14.shop_estimate", q(gap_recipe(hand)))
    emit("G-F14.shop_average", q(mean71))
    emit("G-F14.shop_short_by", q(200 - mean71))
    late = (158, 166, 173, 189, 200)
    late_lo = 151
    emit("G-F14.late_from", late_lo)
    emit("G-F14.late_handful", csv(late))
    emit("G-F14.late_estimate", q(gap_recipe(late)))
    emit("G-F14.late_first_gap", gaps_of(late)[0])
    emit("G-F14.late_other_gaps", csv(gaps_of(late)[1:]))
    # the late pool's average, by enumeration of the 50 boxes 151..200
    late_total, late_n = F(0), 0
    for h in combinations(range(late_lo, 201), k):
        late_total += gap_recipe(h)
        late_n += 1
    emit("G-F14.late_average", q(late_total / late_n))
    # in a fair draw from 200, all five above 150 (the smallest gap at least
    # 150): five draws in a row from the 50 boxes above 150
    above = F(1)
    for i in range(k):
        above *= F(200 - (late_lo - 1) - i, 200 - i)
    emit("G-F14.fair_all_above", q(above))
    emit("G-F14.fair_all_above_under_one_in_1000", flag(above < F(1, 1000)))

    # 15: numbering from an unknown start
    s = sorted(hand)
    inner = [b - a - 1 for a, b in zip(s, s[1:])]
    emit("G-F15.inner_gaps", csv(inner))
    emit("G-F15.inner_count", len(inner))
    emit("G-F15.hidden_count", len(s) + 1 - len(inner))
    emit("G-F15.inner_unseen", sum(inner))
    emit("G-F15.inner_average", q(F(sum(inner), len(inner))))
    emit("G-F15.inner_average_decimal", decimal1(F(sum(inner), len(inner))))
    emit("G-F15.span_numbers", s[-1] - s[0] + 1)
    emit("G-F15.spread_estimate", q(spread_recipe(hand)))
    # unbiased whatever the first number is: shift the numbering and enumerate
    ok = True
    for start in (1, 7, 10001):
        for n in range(k, 16):
            tot, cnt = F(0), 0
            for h in combinations(range(start, start + n), k):
                tot += spread_recipe(h)
                cnt += 1
            ok &= tot / cnt == n
    emit("G-F15.spread_unbiased", flag(ok))
    # two deliveries: boxes 1-5 or 6-10, each half the time
    deliveries = [(1, 2, 3, 4, 5), (6, 7, 8, 9, 10)]
    emit("G-F15.deliveries", ";".join(csv(d) for d in deliveries))
    emit("G-F15.delivery_estimates", csv([gap_recipe(d) for d in deliveries]))
    per_box = {b: F(sum(b in d for d in deliveries), 2) for b in range(1, 11)}
    emit("G-F15.deliveries_each_box", q(per_box[1]) if len(set(per_box.values())) == 1 else "unequal")
    emit("G-F15.deliveries_average", q(sum(gap_recipe(d) for d in deliveries) / 2))
    emit("G-F15.deliveries_made", 10)

    # 16: a sixth box
    six = hand + (90,)
    emit("G-F16.sixth", 90)
    emit("G-F16.size", len(six))
    emit("G-F16.unseen", sum(gaps_of(six)))
    emit("G-F16.gaps", csv(gaps_of(six)))
    emit("G-F16.estimate", q(gap_recipe(six)))
    emit("G-F16.average_gap", q(F(sum(gaps_of(six)), len(six))))


# ------------------------------------------------------------ cross-checks


def cross_checks():
    # the always/never sentences of the script, at their edges
    never_below = all(gap_recipe(h) >= max(h) for kk in range(1, 7) for n in range(kk, 13) for h in handfuls(n, kk))
    check("the gap recipe never guesses below the largest number (k<=6, N<=12)", never_below)
    equal_only_at_edge = all(
        (gap_recipe(h) == max(h)) == (h == tuple(range(1, kk + 1)))
        for kk in range(1, 6) for n in range(kk, 12) for h in handfuls(n, kk)
    )
    check("it equals the largest exactly when the handful is 1..k", equal_only_at_edge)
    check("unbiased at every count (k<=6, N<=22, enumeration)",
          all(mean_over(n, kk, gap_recipe) == n for kk in range(1, 7) for n in range(kk, 23)))
    check("unbiased at every count (closed form vs product, k=5, N<=400)",
          all(sum(F(comb(mm - 1, 4), comb(n, 5)) * (mm + F(mm - 5, 5)) for mm in range(5, n + 1)) == n
              for n in range(5, 401)))
    # each gap position averages (N-k)/(k+1), the top one included
    pos_ok = True
    for kk in range(1, 5):
        for n in range(kk, 12):
            hs = list(handfuls(n, kk))
            lists = [gaps_of(h) + [n - h[-1]] for h in hs]
            for i in range(kk + 1):
                pos_ok &= F(sum(gl[i] for gl in lists), len(hs)) == F(n - kk, kk + 1)
            # and the lists are exactly the compositions, once each
            pos_ok &= len(set(map(tuple, lists))) == len(hs) == comb(n, kk)
    check("every gap position averages (N-k)/(k+1); handfuls <-> gap lists", pos_ok)
    # 06's trade and 07's guess, at every small count: trading any two gap
    # positions maps the lists onto themselves; the average of the k seen
    # gaps averages what the top gap does; estimate - count is guess - top
    trade_ok = True
    for kk in range(1, 5):
        for n in range(kk, 12):
            lists = [tuple(gaps_of(h) + [n - h[-1]]) for h in handfuls(n, kk)]
            every = set(lists)
            for i in range(kk + 1):
                for j in range(i + 1, kk + 1):
                    swapped = set()
                    for gl in lists:
                        t = list(gl)
                        t[i], t[j] = t[j], t[i]
                        swapped.add(tuple(t))
                    trade_ok &= swapped == every
            hs = list(handfuls(n, kk))
            guess = [F(sum(gaps_of(h)), kk) for h in hs]
            top = [n - h[-1] for h in hs]
            trade_ok &= sum(guess) / len(hs) == F(sum(top), len(hs)) == F(n - kk, kk + 1)
            trade_ok &= all(gap_recipe(h) - n == x - t for h, x, t in zip(hs, guess, top))
    check("trading gap positions keeps the lists; the guess averages the top gap's (N-k)/(k+1) (k<=4, N<=11)",
          trade_ok)
    # NEGATIVE: guessing the top gap with the largest seen gap instead of
    # their average is biased high, so the averaging matters
    widest = lambda h: max(h) + max(gaps_of(h))
    check("NEGATIVE 07: largest plus the widest seen gap averages above N (k=2..4, N=k+2..11)",
          all(mean_over(n, kk, widest) > n for kk in range(2, 5) for n in range(kk + 2, 12)))
    check("the doubling recipe is unbiased (k<=5, N<=18)",
          all(mean_over(n, kk, doubling_recipe) == n for kk in range(1, 6) for n in range(kk, 19)))
    check("with a pair, doubling never goes below the largest (edge of 08's claim)",
          all(doubling_recipe(h) >= h[-1] for n in range(2, 20) for h in handfuls(n, 2)))
    check("with three, doubling can go below the largest", doubling_recipe((1, 2, 5)) < 5)
    # Rao-Blackwell: averaging doubling within a largest gives the gap recipe,
    # and never raises the average miss (k=2..4, N<=12)
    rb_ok = True
    for kk in range(2, 5):
        for n in range(kk, 13):
            groups = {}
            for h in handfuls(n, kk):
                groups.setdefault(h[-1], []).append(doubling_recipe(h))
            for mm, vals in groups.items():
                rb_ok &= sum(vals) / len(vals) == mm + F(mm - kk, kk)
            rb_ok &= mae_over(n, kk, gap_recipe) <= mae_over(n, kk, doubling_recipe)
            rb_ok &= mse_over(n, kk, gap_recipe) <= mse_over(n, kk, doubling_recipe)
    check("grouping the doubling recipe by the largest gives the gap recipe and smaller misses", rb_ok)
    # other unbiased recipes miss at least as much (plain and squared)
    rivals_ok = True
    for kk in (2, 3, 5):
        for n in range(kk + 1, 14):
            base_mae, base_mse = mae_over(n, kk, gap_recipe), mse_over(n, kk, gap_recipe)
            rivals = [doubling_recipe, smallest_recipe]
            if kk >= 2:
                rivals.append(spread_recipe)
            for lam in (F(1, 4), F(1, 2), F(3, 4)):
                rivals.append(lambda h, lam=lam: lam * gap_recipe(h) + (1 - lam) * doubling_recipe(h))
            for r in rivals:
                rivals_ok &= mean_over(n, kk, r) == n
                rivals_ok &= base_mae <= mae_over(n, kk, r) and base_mse <= mse_over(n, kk, r)
    check("seven unbiased rivals are unbiased and miss at least as much", rivals_ok)
    check("the forced recipe of the largest is the gap recipe (k=1..4, N<=25)",
          all(all(v == mm + F(mm - kk, kk) for mm, v in forced_values(kk, 25).items()) for kk in range(1, 5)))
    # the other numbers given the largest: uniform over sets below it, any count
    ok = True
    for kk, top in ((3, 7), (4, 9), (5, 8)):
        sets = None
        for n in range(top, top + 7):
            got = {}
            for h in handfuls(n, kk):
                if h[-1] == top:
                    got[h[:-1]] = got.get(h[:-1], 0) + 1
            ok &= len(set(got.values())) == 1 and len(got) == comb(top - 1, kk - 1)
            sets = got.keys() if sets is None else sets
            ok &= set(got.keys()) == set(sets)
    check("given the largest, the others are uniform below it whatever the count", ok)
    # NEGATIVE 11: draws that favour bigger numbers (each box drawn in turn
    # with chance proportional to its number) make the others given the
    # largest depend on the count, and not uniform; fair draws of the same
    # sizes do not
    def weighted(n, kk):
        out = {}
        for order in permutations(range(1, n + 1), kk):
            p, left = F(1), F(n * (n + 1), 2)
            for x in order:
                p *= x / left
                left -= x
            key = tuple(sorted(order))
            out[key] = out.get(key, 0) + p
        return out

    def given(dist, top):
        sub = {h[:-1]: p for h, p in dist.items() if h[-1] == top}
        tot = sum(sub.values())
        return {o: p / tot for o, p in sub.items()}
    for kk, top in ((2, 4), (3, 6)):
        w = [given(weighted(n, kk), top) for n in range(top, top + 4)]
        fair = [given({h: F(1, comb(n, kk)) for h in handfuls(n, kk)}, top) for n in range(top, top + 4)]
        check(f"NEGATIVE 11: weighted draws (k={kk}, largest {top}): the others' chances change with the count "
              f"and are unequal; fair draws' never do",
              all(a != w[0] for a in w[1:]) and len(set(w[0].values())) > 1
              and all(a == fair[0] for a in fair) and len(set(fair[0].values())) == 1)
    # 10's always/never sentences at their edges: doubling never falls below
    # the largest for one box or a pair, and does for some handful of three
    # or more from a company of 4 or more boxes
    check("10: doubling is never below the largest for handfuls of 1 or 2 (N<=14)",
          all(doubling_recipe(h) >= h[-1] for kk in (1, 2) for n in range(kk, 15) for h in handfuls(n, kk)))
    check("10: doubling falls below the largest for some handful of 3 to 6 from every count above the size",
          all(any(doubling_recipe(h) < h[-1] for h in handfuls(n, kk)) for kk in range(3, 7) for n in range(kk + 1, 13)))
    check("10: at the edge N = k every handful is 1..N and doubling says N",
          all(doubling_recipe(tuple(range(1, n + 1))) == n for n in range(1, 30)))
    # the two recipes' average squared misses, by enumeration: doubling's is
    # (N+1)(N-k)/(3k), (k+2)/3 of the gap recipe's, equal for one box and
    # zero at N = k
    ratio_ok = True
    for kk in range(1, 6):
        for n in range(kk, 14):
            d, g = mse_over(n, kk, doubling_recipe), mse_over(n, kk, gap_recipe)
            ratio_ok &= d == F((n + 1) * (n - kk), 3 * kk) and d == F(kk + 2, 3) * g
            ratio_ok &= (d == g) == (kk == 1 or n == kk)
    check("10: doubling's average squared miss is (N+1)(N-k)/(3k) = (k+2)/3 of the gap recipe's (k<=5, N<=13)",
          ratio_ok)
    # 12's sentences at their edges, for the doubling recipe and the rival
    # three-times-the-smaller recipe, k = 2..4, N = k..12: inside every group
    # the average's miss is at most the average of the misses, equal exactly
    # when no two misses point opposite ways; its squared miss falls short of
    # the average squared miss by exactly the spread inside the group (zero
    # only when the group's estimates agree); weighted by their chances the
    # groups give back the overall average, and weighted equally they never
    # do once N > k
    def by_top(n, kk, recipe):
        out = {}
        for h in handfuls(n, kk):
            out.setdefault(h[-1], []).append(F(recipe(h)))
        return out
    jensen_ok = weights_ok = True
    for recipe in (doubling_recipe, smallest_recipe):
        for kk in range(2, 5):
            for n in range(kk, 13):
                groups = by_top(n, kk, recipe)
                for vals in groups.values():
                    avg, misses = sum(vals) / len(vals), [v - n for v in vals]
                    plain = sum(abs(x) for x in misses) / len(misses)
                    squared = sum(x * x for x in misses) / len(misses)
                    spread = sum((v - avg) ** 2 for v in vals) / len(vals)
                    one_way = min(misses) >= 0 or max(misses) <= 0
                    jensen_ok &= abs(avg - n) <= plain and (abs(avg - n) == plain) == one_way
                    jensen_ok &= squared - (avg - n) ** 2 == spread and (spread == 0) == (len(set(vals)) == 1)
                weighted_mean = sum(F(len(v), comb(n, kk)) * sum(v) / len(v) for v in groups.values())
                equal_mean = sum(sum(v) / len(v) for v in groups.values()) / len(groups)
                weights_ok &= weighted_mean == n and (equal_mean == n) == (n == kk)
    check("12: inside every group the average misses by at most the average miss (equal exactly when the "
          "misses are one-sided) and by exactly the spread less squared (doubling and 3*smaller-1, k=2..4, N<=12)",
          jensen_ok)
    check("12: groups weighted by their chances average N; weighted equally they miss N whenever N > k", weights_ok)
    # NEGATIVE 12: averaging does not always help strictly. A recipe of the
    # largest is unchanged by it, and with a single box or N = k every
    # recipe of the handful already is one
    check("NEGATIVE 12: averaging changes nothing for the gap recipe itself (its groups agree), for one box, "
          "or at N = k (k<=5, N<=12)",
          all(len(set(v)) == 1 for kk in range(1, 5) for n in range(kk, 13)
              for v in by_top(n, kk, gap_recipe).values())
          and all(mae_over(n, 1, doubling_recipe) == mae_over(n, 1, gap_recipe) for n in range(1, 13))
          and all(mse_over(kk, kk, doubling_recipe) == mse_over(kk, kk, gap_recipe) == 0 for kk in range(1, 6)))
    # 11 carried into 12: with fair draws a group's average is the same at
    # every count, so it is a recipe; with draws that favour bigger numbers
    # (weighted() above) the doubling average given the largest changes with
    # the count, so "replace it by its group's average" would need N
    fair_same = all(len({sum(by_top(n, kk, doubling_recipe)[t]) / len(by_top(n, kk, doubling_recipe)[t])
                         for n in range(t, 13)}) == 1 for kk in range(2, 5) for t in range(kk, 13))
    check("12: with fair draws each group's doubling average is the same in every company it fits (k=2..4, N<=12)",
          fair_same)
    for kk, top in ((2, 4), (3, 6)):
        avgs = []
        for n in range(top, top + 4):
            sub = {h: p for h, p in weighted(n, kk).items() if h[-1] == top}
            avgs.append(sum(p * doubling_recipe(h) for h, p in sub.items()) / sum(sub.values()))
        check(f"NEGATIVE 12: weighted draws (k={kk}, largest {top}): the doubling average given the largest "
              f"changes with the count", len(set(avgs)) == len(avgs))
    # 13: the staircase. recurrence_proof() checks the algebra as polynomial
    # identities (every N, every k); the enumerations below are controls
    check("13: the recurrence and its solution hold as polynomial identities (recurrence_proof)",
          recurrence_proof())
    wrong = padd(pmul(poly((1, 1, 0)), poly((1, 1, 0))),
                 pmul(padd(poly((1, 1, 0)), poly((1, 0, 0)), -1), padd(poly((1, 1, 0)), poly((1, 0, 1)))), -1)
    check("NEGATIVE 13: the polynomial check refuses a wrong step, N^2 - (N - 1)(N + k) != (k + 1)N - k",
          wrong != padd(pmul(padd(poly((1, 0, 1)), poly((1, 0, 0))), poly((1, 1, 0))), poly((1, 0, 1)), -1))
    check("13: the share of handfuls topped by N is k/N, never zero (k<=6, N<=30)",
          all(F(sum(1 for h in handfuls(n, kk) if h[-1] == n), comb(n, kk)) == F(kk, n) > 0
              for kk in range(1, 7) for n in range(kk, 31) if comb(n, kk) <= 600000))
    # control: change the staircase at one value and the recipe is biased at
    # exactly the counts from that value on
    perturb_ok = True
    for kk in (2, 3):
        base = forced_values(kk, 14)
        for bump in range(kk, 13):
            g = dict(base)
            g[bump] += 1
            for n in range(kk, 15):
                biased = mean_over(n, kk, lambda h: g[h[-1]]) != n
                perturb_ok &= biased == (n >= bump)
    check("13: raising the staircase at one value biases the recipe at that count and every larger one, and "
          "at no smaller one (k=2,3, N<=14)", perturb_ok)
    # NEGATIVE 13: with no new value at a count there is nothing to solve for.
    # Pairs, the largest capped at 4: forced to 2, 7/2, 5 by counts 2..4, the
    # count of 5 then averages 22/5, so no unbiased recipe of it exists
    capped = forced_values(2, 4)
    check("NEGATIVE 13: a recipe of min(largest, 4) for pairs, forced by counts 2..4, averages 22/5 at 5 boxes",
          mean_over(5, 2, lambda h: capped[min(h[-1], 4)]) == F(22, 5))
    rival_ok = True
    for kk in range(2, 6):
        for n in range(kk, 14):
            rival_ok &= mean_over(n, kk, smallest_recipe) == n
            groups = by_top(n, kk, smallest_recipe)
            rival_ok &= all(sum(v) / len(v) == t + F(t - kk, kk) for t, v in groups.items())
            rival_ok &= mae_over(n, kk, gap_recipe) <= mae_over(n, kk, smallest_recipe)
            rival_ok &= mse_over(n, kk, gap_recipe) <= mse_over(n, kk, smallest_recipe)
    check("13: (k+1)*smaller - 1 is unbiased, averages inside its groups to the staircase, and misses at least "
          "as much, plain and squared (k=2..5, N<=13)", rival_ok)
    # the sequential-draw chance equals enumeration on small cases
    check("all-at-most product == enumeration (N<=14, k<=4)",
          all(all_at_most(mm, n, kk) == F(sum(1 for h in handfuls(n, kk) if h[-1] <= mm), comb(n, kk))
              for kk in range(1, 5) for n in range(kk, 15) for mm in range(0, n + 1)))
    # the 71-box brute force agrees with the binomial closed form
    by_max, by_sum, joint, count = max_and_sum_counts(71, 5)
    check("71 boxes: handfuls by largest == C(m-1, 4)",
          all(by_max[mm] == comb(mm - 1, 4) for mm in range(5, 72)) and count == comb(71, 5))
    # one-sided range: the bound found by the product is the last count whose
    # binomial ratio C(60,5)/C(n,5) is still at least 1/20
    top = upper_bound(60, 5, F(1, 20))
    check(f"range top {top}: binomial ratio >= 1/20 there and < 1/20 one count later",
          F(comb(60, 5), comb(top, 5)) >= F(1, 20) > F(comb(60, 5), comb(top + 1, 5)))

    # 08's and 09's sentences at their edges
    est71 = {mm: mm + F(mm - 5, 5) for mm in by_max}
    check("08: the 71-box estimates are 67 values 6/5 apart, 5 to 84.2, one per largest number",
          sorted(est71.values()) == [5 + F(6, 5) * j for j in range(67)] and len(set(est71.values())) == 67)
    check("08: over nine hundred thousand and under a million handfuls top out at 71",
          900000 < by_max[71] < 1000000)
    check("08: only the largest 60 gives exactly 71, so the hero bar is one stack",
          [mm for mm, e in est71.items() if e == 71] == [60])
    check("08: the variance is (N-k)(N+1)/(k(k+2)) at N=71 (66*72/35), and 11.65 <= its root < 11.75",
          sum((e - 71) ** 2 * by_max[mm] for mm, e in est71.items()) / count == F(66 * 72, 35)
          and F(1165, 100) ** 2 <= F(66 * 72, 35) < F(1175, 100) ** 2)
    check("08: sqrt_about1 at its edges (1.1 exactly; 1.15 rounds up; just under 1.15 rounds down)",
          sqrt_about1(F(121, 100)) == "1.1" and sqrt_about1(F(529, 400)) == "1.2"
          and sqrt_about1(F(528, 400)) == "1.1" and sqrt_about1(F(1, 1000)) == "0.0")
    # 08_6's "Three in twenty was lucky": three or more exact estimates in
    # twenty fair draws has chance under 1/20 (two or more would not)
    exact = F(by_max[60], count)
    at_least = lambda j: 1 - sum(comb(20, i) * exact ** i * (1 - exact) ** (20 - i) for i in range(j))
    check("08: three or more exactly-71 estimates in twenty draws has chance under 1/20, two or more over it",
          at_least(3) < F(1, 20) < at_least(2))
    # NEGATIVE 08: the balance point is the mean, not the middle handful, and
    # the standard deviation is not the average miss
    acc, middle = 0, None
    for mm in sorted(by_max):
        acc += by_max[mm]
        if middle is None and 2 * acc >= count:
            middle = est71[mm]
    mae = sum(abs(e - 71) * by_max[mm] for mm, e in est71.items()) / count
    check("NEGATIVE 08: the middle handful's estimate is 74.6, not the balance point 71",
          middle == F(373, 5) != 71)
    check("NEGATIVE 08: the average miss squared falls short of the variance (9.23^2 < 135.8)",
          mae * mae < F(66 * 72, 35))
    # 09: every count from the cutoff on is rare (the chance only falls), and
    # 107 and 108 sit just either side of one in twenty
    p = lambda n, kk=5: all_at_most(60, n, kk)
    check("09: from 108 to 400 boxes, a handful this low has chance under 1/20, falling",
          all(p(n) < F(1, 20) and p(n + 1) < p(n) for n in range(108, 400)))
    check("09: just over 5% at 107 (under 6%), just under at 108 (over 4%)",
          F(1, 20) < p(107) < F(6, 100) and F(4, 100) < p(108) < F(1, 20))
    check("09: with ten boxes the range's top is 79: chance >= 1/20 at 79, < 1/20 at 80",
          p(79, 10) >= F(1, 20) > p(80, 10))
    check("09: a range covers 71 exactly when its handful's largest is 40 or more",
          all((upper_bound(mm, 5, F(1, 20)) >= 71) == (mm >= 40) for mm in range(5, 72)))
    # NEGATIVE 09: a range from the largest to the estimate is not a 95% range
    # (it covers 71 only when the estimate overshoots or hits it)
    to_estimate = sum(F(by_max[mm], count) for mm, e in est71.items() if e >= 71)
    check("NEGATIVE 09: the range from the largest to the estimate covers 71 only about 62% of the time (< 95%)",
          F(6, 10) < to_estimate < F(62, 100))
    # NEGATIVE 09: (60/N)^5 is a first guess that puts the cutoff at 109
    guess_top = max(n for n in range(60, 200) if F(60, n) ** 5 >= F(1, 20))
    check("NEGATIVE 09: the first guess (60/N)^5 >= 1/20 gives 109, not the exact 107", guess_top == 109 != top)
    # NEGATIVE 09: the 95% is not a probability about the count. Give the
    # count a prior (equally likely over 60..200, or over 60..1000) and the
    # chance that it is at most 107 given this handful changes with the
    # prior, and is 95% for neither
    posts = []
    for last in (200, 1000):
        like = {n: F(1, comb(n, 5)) for n in range(60, last + 1)}
        posts.append(sum(w for n, w in like.items() if n <= top) / sum(like.values()))
    check("NEGATIVE 09: the posterior chance of N <= 107 depends on the prior (91.8% vs 91.2%), never 95%",
          posts[0] != posts[1] and all(abs(x - F(19, 20)) > F(1, 100) for x in posts))

    # negative controls: these must be detected as wrong
    no_minus_one = lambda h: max(h) * (1 + F(1, len(h)))
    check("NEGATIVE: largest*(1+1/k) without the -1 is biased high by exactly 1",
          all(mean_over(n, kk, no_minus_one) == n + 1 for kk in (2, 3, 5) for n in range(kk, 12)))
    k_plus_one = lambda h: max(h) + F(sum(gaps_of(h)), len(h) + 1)
    check("NEGATIVE: dividing the gaps by k+1 is biased low",
          all(mean_over(n, kk, k_plus_one) < n for kk in (2, 3, 5) for n in range(kk + 1, 12)))
    deliveries = [(1, 2, 3, 4, 5), (6, 7, 8, 9, 10)]
    check("NEGATIVE: equal per-box chances without equal handfuls: average 8, not 10",
          sum(gap_recipe(d) for d in deliveries) / 2 == 8)
    check("NEGATIVE: a shop of 40 boxes averages 40, not the company's 200",
          mean_over(40, 5, gap_recipe) == 40)
    shop71 = sum((mm + F(mm - 5, 5)) * c for mm, c in by_max.items()) / count
    check("NEGATIVE: the friends' shop of 71 boxes averages 71, not the company's 200",
          shop71 == 71 != 200)
    # the drawn handfuls are fair-handful shaped: k different boxes in 1..n,
    # sorted; the generator's values stay below n; and the streams differ
    ok = True
    for stream, n in ((1, 1000), (2, 70), (3, 71), (4, 7)):
        for h in drawn_handfuls(stream, n, 5, 50):
            ok &= len(set(h)) == 5 and list(h) == sorted(h) and 1 <= h[0] and h[-1] <= n
    rng = SplitMix64(SEED)
    ok &= all(0 <= rng.below(n) < n for n in (1, 2, 3, 70, 71, 1000, 2**63 + 1) for _ in range(200))
    ok &= drawn_handfuls(1, 1000, 5, 3) != drawn_handfuls(2, 1000, 5, 3)
    check("drawn handfuls: five different boxes in range, sorted; streams differ", ok)
    # over many drawn handfuls, every box from a company of 7 turns up about
    # equally often (5/7 of draws each; a loose band, not a test of randomness)
    tally = {b: 0 for b in range(1, 8)}
    for h in drawn_handfuls(4, 7, 5, 7000):
        for b in h:
            tally[b] += 1
    check("drawn handfuls from 7: each box in 5000 +- 250 of 7000",
          all(abs(c - 5000) <= 250 for c in tally.values()))
    # permutation invariance: a handful is a set, its order never matters
    check("recipes ignore the order of the numbers",
          all(gap_recipe(p) == gap_recipe((14, 27, 33, 51, 60)) and doubling_recipe(p) == 73
              for p in permutations((14, 27, 33, 51, 60))))


def main():
    facts()
    cross_checks()
    print(f"# {_CHECKS[0]} cross-checks, {len(_FAILED)} failed; {len(_EMITTED)} facts", file=sys.stderr)
    return 1 if _FAILED else 0


if __name__ == "__main__":
    sys.exit(main())
