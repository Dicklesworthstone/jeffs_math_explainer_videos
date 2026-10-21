#!/usr/bin/env python3
"""Facts oracle for "How to Learn a Secret About a Crowd Without Learning Each
Person's Answer".

Run from this folder (`./x randomized_response facts-check` does, with numpy
through uv; chapter 03's eight classes need numpy):

    uv run --no-project --with numpy python3 -I -B oracle.py

stdout: one `KEY<TAB>VALUE` line per fact, the same keys `survey facts` prints
(src/facts.rs), in the rows of docs/fact-sheet.md: R-F (computed), R-X (the
edges of the always/never sentences).

stderr: cross-checks (lines starting with "#") and a summary; exit 1 if any
cross-check failed.

How the values are obtained, independently of the Rust model (which counts
each coin pair's route, tallies a class whose coins land on average, convolves
two binomial counts for the yes distribution and runs the rule over base-4
coin outcomes for repeated asks):
  * the rule as a lookup table of the two coins' faces ("HH", "HT", "TH",
    "TT"), written out here, not computed from a route;
  * the class on average by giving every person the coin outcomes in turn,
    group by group, and counting answers per (row, answer) cell;
  * the yes count's exact distribution by multiplying the class's people in
    one at a time: a dynamic program over people, each adding their own
    outcome counts (no binomial coefficients), with Python's big integers;
  * Bayes for the answers' meaning by counting (person, coin outcome) pairs;
  * sampling variance by enumerating one random person's (truth, coin) joint
    distribution and taking E[X^2] - E[X]^2 of their contribution;
  * repeated asks by itertools.product over every coin outcome of every ask,
    and RAPPOR's 75-of-100 example by exact binomial likelihoods (big
    integers: 4^100 is no obstacle here);
  * chapter 03's eight real classes by numpy's own PCG64DXSM(3).random_raw(),
    two bits a student looked up in the written-out rule table (the Rust side
    draws through franken_manim's port of the same generator);
  * the estimate's mean, its mean rounded up at zero, and the "one in k"
    readings straight from the person-by-person distribution;
  * chapter 05's dial by counting a spinner's slices and the second coin's
    faces with its own inline rule (a truthful slice gives the truth, any
    other slice the face, heads for yes), equivalent to the rule table but
    not read from it; the coins' variance as the variance of one person's
    share over those outcomes; the fixed class's miss as the square root of
    20 times it, rounded to the nearest whole by an integer square root.

The 13-chapter video's added rows, by the same independent means:
  * R-F17, chapter 01's chart: each person's yes chance from the weighted
    routes (`channel`), summed seat by seat for every number of copiers;
  * R-F18, the class of forty: solving E[yes] = n(1 - t)/2 + k t for k;
    and the person-by-person distribution for every k at twenty;
  * R-F19, the misses: tails of the person-by-person distribution; three
    hundred people by the same dynamic program (big integers), at 60 and at
    0 copiers;
  * R-F20, the sampled share: one person's enumerated joint distribution,
    and the smallest number of coin answers found by counting upward;
  * R-F21, copiers who always say no: the dynamic program with those seats
    answering yes in none of the four outcomes, and one person's share
    contribution enumerated for the share's average;
  * R-F22, a hundred answers: exact binomial likelihoods, and the zeros
    after the point counted by multiplying by ten;
  * R-F23: the dynamic program's estimate variance at each checked size and
    every number of copiers;
  * R-X6 to R-X8: RAPPOR's words typed here from the paper (arXiv
    1407.6981v2, read 2026-10-20), not from the Rust constants.
"""
import math
import sys
from fractions import Fraction
from itertools import product

_EMITTED = {}
_FAILED = []
_CHECKS = [0]

# The class (src/model.rs): twenty seats in rows of five; these four copied.
CLASS_SIZE = 20
COLUMNS = 5
COPIED_SEATS = (2, 6, 13, 17)
# Chapter 04's person is asked this many times.
ASKS = 3
# RAPPOR's example: yeses out of answers.
RAPPOR = (75, 100)
WORST_LISTED = 5
MAX_ASKS = 10
# Chapter 03's real classes: seed 3 (the chapter's number), eight draws.
SEED = 3
RUNS = 8
# Chapter 05's dial: a spinner of four equal slices; each setting's name and
# how many slices say "tell the truth" (the rest hand over to the second coin).
SLICES = 4
DIAL = (("pure", 0), ("down", 1), ("ours", 2), ("up", 3), ("direct", 4))
# Chapter 04's class of forty and its yeses; the seeded class it shows (the
# fifth of chapter 03's eight, counted from zero).
FORTY = (40, 18)
SHOWN_RUN = 4
# Chapter 07's three hundred; chapter 09's class sizes; chapter 13's sizes.
BIG_CLASS = 300
ASKED = (20, 100, 300)
CHECKED_SIZES = (1, 2, 7, 20, 24)

# The rule, written out: (first coin, second coin) -> answer for someone who
# copied, answer for someone who didn't; and what decided it.
OUTCOMES = ("HH", "HT", "TH", "TT")
RULE = {
    "HH": (True, False, "truth"),
    "HT": (True, False, "truth"),
    "TH": (True, True, "forced yes"),
    "TT": (False, False, "forced no"),
}


def emit(key, value):
    if key in _EMITTED:
        raise SystemExit(f"oracle: {key} emitted twice")
    _EMITTED[key] = str(value)
    print(f"{key}\t{value}")


def check(name, ok):
    _CHECKS[0] += 1
    if not ok:
        _FAILED.append(name)
    print(f"# {'ok  ' if ok else 'FAIL'} {name}", file=sys.stderr)


def csv(values):
    return ",".join(str(v) for v in values)


def says(copied, outcome):
    yes_if_copied, yes_if_not, _ = RULE[outcome]
    return yes_if_copied if copied else yes_if_not


def percent(q):
    """Nearest whole percent, halves up, exactly."""
    return math.floor(Fraction(q) * 100 + Fraction(1, 2))


# ------------------------------------------------------------ the rule

def p_yes(copied):
    return Fraction(sum(says(copied, o) for o in OUTCOMES), len(OUTCOMES))


def ratio(said_yes):
    """P(said | truth = said) / P(said | truth = other)."""
    def p(copied):
        return Fraction(sum(says(copied, o) == said_yes for o in OUTCOMES), len(OUTCOMES))
    return p(said_yes) / p(not said_yes)


def channel(t):
    """The dial: P(yes | copied), P(yes | didn't) for truth chance t, fair second coin,
    computed by weighting the three routes."""
    return t + (1 - t) / 2, (1 - t) / 2


def estimate(yes, n, t):
    """Solve E[yes] = n (1 - t)/2 + k t for k; None when nobody tells the truth."""
    try:
        return (yes - n * (1 - t) / 2) / t
    except ZeroDivisionError:
        return None


def worst_ratio(t):
    yc, yd = channel(t)
    ratios = []
    for a, b in ((yc, yd), (1 - yd, 1 - yc)):
        if b == 0:
            return None
        ratios.append(a / b)
    return max(ratios)


# ------------------------------------------------------------ the class

def average_class():
    """Every seat's outcome, handing out HH, HT, TH, TT in turn within each group."""
    turn = {True: 0, False: 0}
    seats = []
    for seat in range(CLASS_SIZE):
        copied = seat in COPIED_SEATS
        seats.append((copied, OUTCOMES[turn[copied] % len(OUTCOMES)]))
        turn[copied] += 1
    return seats, turn


def cells(seats):
    """(row, answer) -> [didn't, copied] counts; row is 'truth' or 'coin'."""
    out = {}
    for copied, outcome in seats:
        row = "truth" if RULE[outcome][2] == "truth" else "coin"
        key = (row, says(copied, outcome))
        out.setdefault(key, [0, 0])[int(copied)] += 1
    return out


# ------------------------------------------------------------ scatter

def yes_distribution(people, copiers_say_no=False):
    """ways[y] over all 4^len(people) coin outcomes, one person at a time; with
    copiers_say_no, those who copied answer yes in none of their outcomes."""
    ways = [1]
    for copied in people:
        yes_ways = 0 if copied and copiers_say_no else sum(says(copied, o) for o in OUTCOMES)
        no_ways = len(OUTCOMES) - yes_ways
        nxt = [0] * (len(ways) + 1)
        for y, w in enumerate(ways):
            nxt[y] += w * no_ways
            nxt[y + 1] += w * yes_ways
        ways = nxt
    total = len(OUTCOMES) ** len(people)
    return [Fraction(w, total) for w in ways], total


def moments(dist, f):
    m = sum(p * f(y) for y, p in enumerate(dist))
    return m, sum(p * f(y) ** 2 for y, p in enumerate(dist)) - m * m


def person_variance(share, t):
    """One random person's contribution to the estimated share, (yes - (1-t)/2)/t:
    its variance over (truth, route, answer), enumerated."""
    joint = []
    for copied, w in ((True, share), (False, 1 - share)):
        for route, pr in (("truth", t), ("forced yes", (1 - t) / 2), ("forced no", (1 - t) / 2)):
            yes = copied if route == "truth" else route == "forced yes"
            joint.append((w * pr, (Fraction(int(yes)) - (1 - t) / 2) / t))
    mean = sum(p * x for p, x in joint)
    return sum(p * x * x for p, x in joint) - mean * mean, mean


# ------------------------------------------------------------ asking again

def repeated(asks, memoized=False):
    """sequence (tuple of answers) -> [didn't, copied] outcome counts."""
    out = {}
    for copied in (False, True):
        if memoized:
            runs = ((o,) * asks for o in OUTCOMES)
        else:
            runs = product(OUTCOMES, repeat=asks)
        for run in runs:
            seq = tuple(says(copied, o) for o in run)
            out.setdefault(seq, [0, 0])[int(copied)] += 1
    return out


def posterior(prior, copied_ways, didnt_ways):
    num = prior * copied_ways
    return num / (num + (1 - prior) * didnt_ways)


def main():
    # R-F1: the class
    n, copiers = CLASS_SIZE, len(COPIED_SEATS)
    share = Fraction(copiers, n)
    emit("R-F1.class", n)
    emit("R-F1.columns", COLUMNS)
    emit("R-F1.rows", -(-n // COLUMNS))
    emit("R-F1.copied", copiers)
    emit("R-F1.copied_seats", csv(COPIED_SEATS))
    emit("R-F1.copied_share", share)

    # R-F2: the rule
    emit("R-F2.coins", len(OUTCOMES[0]))
    emit("R-F2.coin_outcomes", len(OUTCOMES))
    emit("R-F2.yes_outcomes.copied", sum(says(True, o) for o in OUTCOMES))
    emit("R-F2.yes_outcomes.didnt", sum(says(False, o) for o in OUTCOMES))
    emit("R-F2.p_yes.copied", p_yes(True))
    emit("R-F2.p_yes.didnt", p_yes(False))
    emit("R-F2.p_no.copied", 1 - p_yes(True))
    emit("R-F2.p_no.didnt", 1 - p_yes(False))
    yes_share = share * p_yes(True) + (1 - share) * p_yes(False)
    emit("R-F2.yes_share", yes_share)
    check("Dwork and Roth 2.3: yes share = 1/4 + p/2", yes_share == Fraction(1, 4) + share / 2)
    routes = [Fraction(sum(RULE[o][2] == r for o in OUTCOMES), len(OUTCOMES))
              for r in ("truth", "forced yes", "forced no")]
    emit("R-F2.route_chances", csv(routes))
    check("the dial at 1/2 is the counted rule", channel(Fraction(1, 2)) == (p_yes(True), p_yes(False)))

    # R-F3: the class on average
    seats, turn = average_class()
    check("each group is a multiple of four", all(v % len(OUTCOMES) == 0 for v in turn.values()))
    c = cells(seats)
    get = lambda row, said: c.get((row, said), [0, 0])
    truthful = sum(get("truth", s)[0] + get("truth", s)[1] for s in (True, False))
    emit("R-F3.truthful", truthful)
    emit("R-F3.truthful_share", Fraction(truthful, n))
    emit("R-F3.truthful.yes", sum(get("truth", True)))
    emit("R-F3.truthful.no", sum(get("truth", False)))
    emit("R-F3.forced", sum(get("coin", True)) + sum(get("coin", False)))
    emit("R-F3.forced.yes", sum(get("coin", True)))
    emit("R-F3.forced.no", sum(get("coin", False)))
    emit("R-F3.forced.yes.copied", get("coin", True)[1])
    emit("R-F3.forced.no.copied", get("coin", False)[1])
    yes = sum(get(r, True)[0] + get(r, True)[1] for r in ("truth", "coin"))
    no = n - yes
    emit("R-F3.yes", yes)
    emit("R-F3.no", no)
    emit("R-F3.counts_seen", len({True, False}))
    check("seven of twenty is the average yes share", Fraction(yes, n) == yes_share)

    # R-F4: undo the coins, the survey's way: solve E[yes] = n/4 + k/2 for k
    t = Fraction(1, 2)
    forced_yes = n * (1 - t) / 2
    k = (yes - forced_yes) / t
    emit("R-F4.forced_yes", forced_yes)
    emit("R-F4.forced_yes_share", forced_yes / n)
    emit("R-F4.left", yes - forced_yes)
    emit("R-F4.factor", 1 / t)
    emit("R-F4.estimate", k)
    emit("R-F4.matches_truth", "true" if k == copiers else "false")
    emit("R-F4.twice_yes", yes / t)
    emit("R-F4.half_class", n * t)
    check("Dwork and Roth: estimate p as twice the yes share minus 1/2",
          k / n == 2 * Fraction(yes, n) - Fraction(1, 2))

    # R-F5: both answers' ratios
    emit("R-F5.ratio.yes", ratio(True))
    emit("R-F5.ratio.no", ratio(False))
    emit("R-F5.worst_ratio", worst_ratio(t))
    emit("R-F5.epsilon", f"{math.log(worst_ratio(t)):.4f}")
    check("both ratios equal the worst", ratio(True) == ratio(False) == worst_ratio(t))

    # R-F6: who the yeses and noes are, counted person by coin outcome
    pairs = [(seat in COPIED_SEATS, o) for seat in range(n) for o in OUTCOMES]
    yes_pairs = [cp for cp, o in pairs if says(cp, o)]
    no_pairs = [cp for cp, o in pairs if not says(cp, o)]
    emit("R-F6.yes.copied", get("truth", True)[1] + get("coin", True)[1])
    emit("R-F6.no.copied", get("truth", False)[1] + get("coin", False)[1])
    given_yes = Fraction(sum(yes_pairs), len(yes_pairs))
    given_no = Fraction(sum(no_pairs), len(no_pairs))
    check("counted on average = Bayes over every coin outcome (yes)",
          given_yes == Fraction(get("truth", True)[1] + get("coin", True)[1], yes))
    check("counted on average = Bayes over every coin outcome (no)",
          given_no == Fraction(get("truth", False)[1] + get("coin", False)[1], no))
    emit("R-F6.posterior.yes", given_yes)
    emit("R-F6.posterior.no", given_no)
    emit("R-F6.percent.prior", percent(share))
    emit("R-F6.percent.yes", percent(given_yes))
    emit("R-F6.percent.no", percent(given_no))

    # R-F11: chapter 02's eighty outcomes, tallied pair by pair
    emit("R-F11.class_outcomes", len(pairs))
    emit("R-F11.others", n - copiers)
    emit("R-F11.outnumber", Fraction(n - copiers, copiers))
    for said, name, chosen in ((True, "yes", yes_pairs), (False, "no", no_pairs)):
        emit(f"R-F11.{name}.copied", sum(chosen))
        emit(f"R-F11.{name}.didnt", len(chosen) - sum(chosen))
        emit(f"R-F11.{name}", len(chosen))
        share_said = Fraction(sum(chosen), len(chosen))
        # Bayes from the rule table's likelihoods, not from the pairs
        like_c = Fraction(sum(says(True, o) == said for o in OUTCOMES), len(OUTCOMES))
        like_d = Fraction(sum(says(False, o) == said for o in OUTCOMES), len(OUTCOMES))
        check(f"a {name}: pairs = Bayes", share_said == share * like_c / (share * like_c + (1 - share) * like_d))
        emit(f"R-F11.posterior.{name}", share_said)
        emit(f"R-F11.average.{name}", Fraction(len(chosen), len(OUTCOMES)))
        emit(f"R-F11.average.{name}.copied", Fraction(sum(chosen), len(OUTCOMES)))
    check("on average the pairs are chapter 01's class", Fraction(len(yes_pairs), len(OUTCOMES)) == yes)

    # R-F12: odds, and a class where half copied (odd seats this time)
    def odds(q):
        return q / (1 - q)
    emit("R-F12.odds.prior", odds(share))
    emit("R-F12.odds.yes", odds(given_yes))
    emit("R-F12.odds.no", odds(given_no))
    check("a yes triples the odds, a no divides them by three",
          odds(given_yes) == 3 * odds(share) and odds(given_no) * 3 == odds(share))

    def yes_share(seats):
        chosen = [seat in seats for seat in range(n) for o in OUTCOMES if says(seat in seats, o)]
        return Fraction(sum(chosen), len(chosen))

    def no_share(seats):
        chosen = [seat in seats for seat in range(n) for o in OUTCOMES if not says(seat in seats, o)]
        return Fraction(sum(chosen), len(chosen))

    half = set(range(1, n, 2))
    emit("R-F12.half_class", len(half))
    emit("R-F12.posterior.yes_at_half", yes_share(half))
    check("at half the class a yes means the copier's own 3/4", yes_share(half) == p_yes(True))
    emit("R-X5.posterior.yes_none_copied", yes_share(set()))
    emit("R-X5.posterior.no_all_copied", no_share(set(range(n))))

    # R-F7: the scatter of a real class of twenty
    people = [seat in COPIED_SEATS for seat in range(n)]
    dist, total = yes_distribution(people)
    check("the distribution sums to one", sum(dist) == 1)
    mean, var = moments(dist, lambda y: y)
    est_mean, est_var = moments(dist, lambda y: 2 * y - Fraction(n, 2))
    emit("R-F7.outcomes", total)
    emit("R-F7.mean_yes", mean)
    emit("R-F7.var_yes", var)
    emit("R-F7.var_estimate", est_var)
    emit("R-F7.sd_estimate", f"{math.sqrt(est_var):.2f}")
    check("the estimate is right on average", est_mean == copiers)
    for other in range(n + 1):
        d, _ = yes_distribution([True] * other + [False] * (n - other))
        m, v = moments(d, lambda y: 2 * y - Fraction(n, 2))
        if m != other or v != Fraction(3 * n, 4):
            check(f"unbiased with variance 3n/4 at {other} copiers", False)
            break
    else:
        check("unbiased with variance 3n/4 for every number of copiers 0..20", True)
    per_person, _ = person_variance(Fraction(0), t)
    per_person_copier, _ = person_variance(Fraction(1), t)
    emit("R-F7.coin_variance_per_person", per_person)
    check("the coins' variance is the same for both truths", per_person == per_person_copier)
    check("the mean yes count is a whole number", mean.denominator == 1)
    exactly = dist[int(mean)]
    emit("R-F7.p_exactly_right", exactly)
    emit("R-F7.percent.exactly_right", percent(exactly))
    negative = sum(p for y, p in enumerate(dist) if 2 * y - Fraction(n, 2) < 0)
    emit("R-F7.p_negative", negative)
    emit("R-F7.percent.negative", percent(negative))

    # R-F8: independent draws (with replacement, or from a large population)
    # at the class's share
    coins_var, mean_share = person_variance(share, t)
    check("one random person's contribution is unbiased", mean_share == share)
    direct = share * (1 - share)
    emit("R-F8.variance_n1.direct", direct)
    emit("R-F8.variance_n1.coins", coins_var)
    emit("R-F8.cost", coins_var / direct)
    emit("R-F8.cost_2dp", f"{float(coins_var / direct):.2f}")
    # the coins' part alone: the enumerated total less the direct variance
    emit("R-F8.added_cost", (coins_var - direct) / direct)
    big_p = channel(t)[0]
    emit("R-F8.warner_extra", big_p * (1 - big_p) / (2 * big_p - 1) ** 2)
    check("coins = direct + Warner's term", coins_var == direct + big_p * (1 - big_p) / (2 * big_p - 1) ** 2)

    # R-F13: chapter 03's eight real classes, replayed with numpy
    import numpy as np
    bits = np.random.PCG64DXSM(SEED)
    yeses = []
    for _ in range(RUNS):
        raw = int(bits.random_raw())
        yeses.append(sum(says(seat in COPIED_SEATS, OUTCOMES[(raw >> (2 * seat)) & 3]) for seat in range(n)))
    estimates = [estimate(y, n, t) for y in yeses]
    emit("R-F13.seed", SEED)
    emit("R-F13.runs", RUNS)
    emit("R-F13.yes", csv(yeses))
    emit("R-F13.estimates", csv(estimates))
    emit("R-F13.lowest", min(estimates))
    emit("R-F13.highest", max(estimates))
    emit("R-F13.on_four", sum(e == copiers for e in estimates))
    emit("R-F13.below_zero", sum(e < 0 for e in estimates))

    # R-F14: the exact distribution, read for chapter 03
    est_of = lambda y: estimate(y, n, t)
    mean_est = sum(p * est_of(y) for y, p in enumerate(dist))
    check("the estimate averages the true count", mean_est == copiers)
    emit("R-F14.estimate_mean", mean_est)
    emit("R-F14.outcomes_trillions", total // 10 ** 12)
    nearest = lambda q: math.floor(Fraction(q) + Fraction(1, 2))
    emit("R-F14.one_in.exactly", nearest(1 / exactly))
    emit("R-F14.one_in.negative", nearest(1 / negative))
    clamped = sum(p * max(est_of(y), 0) for y, p in enumerate(dist))
    check("rounding negatives up raises the average", clamped > mean_est)
    emit("R-F14.clamp_at", 0)
    emit("R-F14.clamped_mean", clamped)
    emit("R-F14.clamped_mean_2dp", f"{float(clamped):.2f}")
    answer_var = p_yes(True) * (1 - p_yes(True))
    check("both truths' answers vary alike", answer_var == p_yes(False) * (1 - p_yes(False)))
    check("twenty answers make the count's variance", answer_var * n == var)
    emit("R-F14.answer_variance", answer_var)
    emit("R-F14.sd_rounded", round(math.sqrt(est_var)))
    cost = coins_var / direct
    check("almost six: just under its nearest whole number", nearest(cost) > cost)
    emit("R-F14.cost_rounded", nearest(cost))

    # R-F9: asking again
    rep = repeated(ASKS)
    emit("R-F9.asks", ASKS)
    emit("R-F9.outcomes", len(OUTCOMES) ** ASKS)
    by = [[0, 0] for _ in range(ASKS + 1)]
    for seq, cnt in rep.items():
        by[sum(seq)][0] += cnt[0]
        by[sum(seq)][1] += cnt[1]
    emit("R-F9.by_yeses.didnt", csv(b[0] for b in by))
    emit("R-F9.by_yeses.copied", csv(b[1] for b in by))
    all_yes = rep[(True,) * ASKS]
    emit("R-F9.all_yes.copied", all_yes[1])
    emit("R-F9.all_yes.didnt", all_yes[0])
    emit("R-F9.ratios", csv(Fraction(b[1], b[0]) for b in by))
    check("every pattern's ratio is 3^(2y - k), whatever the order",
          all(Fraction(c[1], c[0]) == Fraction(3) ** (2 * sum(s) - ASKS) for s, c in rep.items()))
    after = posterior(share, all_yes[1], all_yes[0])
    emit("R-F9.posterior.all_yes", after)
    emit("R-F9.percent.all_yes", percent(after))
    majority = Fraction(sum(c[1] for s, c in rep.items() if 2 * sum(s) > ASKS), len(OUTCOMES) ** ASKS)
    emit("R-F9.majority_right", majority)
    emit("R-F9.percent.majority_right", percent(majority))
    worst = []
    for k in range(1, WORST_LISTED + 1):
        worst.append(max(Fraction(c[1], c[0]) for c in repeated(k).values()))
    emit("R-F9.worst_ratio_by_asks", csv(worst))
    check("each fresh ask multiplies the worst ratio by three",
          all(w == Fraction(3) ** (i + 1) for i, w in enumerate(worst)))
    memo = repeated(ASKS, memoized=True)
    emit("R-F10.memoized.ratio", Fraction(memo[(True,) * ASKS][1], memo[(True,) * ASKS][0]))
    check("a memoized answer gives only all-yes or all-no",
          all(len(set(s)) == 1 for s in memo))
    check("a memoized all-yes stays at three for every ask count to ten",
          all(Fraction(repeated(k, True)[(True,) * k][1], repeated(k, True)[(True,) * k][0]) == 3
              for k in range(1, MAX_ASKS + 1)))
    emit("R-F9.ratio.all_yes", Fraction(all_yes[1], all_yes[0]))
    emit("R-F9.all_no.didnt", rep[(False,) * ASKS][0])
    emit("R-F9.all_no.copied", rep[(False,) * ASKS][1])
    # a majority for each fixed truth: more yeses from a copier, more noes otherwise
    right_copied = sum(c[1] for s, c in rep.items() if 2 * sum(s) > ASKS)
    right_didnt = sum(c[0] for s, c in rep.items() if 2 * sum(s) < ASKS)
    emit("R-F9.majority.outcomes", right_copied)
    emit("R-F9.majority_right.didnt", Fraction(right_didnt, len(OUTCOMES) ** ASKS))
    check("a majority is right as often whichever the truth", right_copied == right_didnt)
    memo_all_yes = memo[(True,) * ASKS]
    emit("R-F10.memoized.outcomes", sum(c[1] for c in memo.values()))
    check("one flip: as many outcomes for each truth", sum(c[0] for c in memo.values()) == len(OUTCOMES))
    emit("R-F10.memoized.all_yes.copied", memo_all_yes[1])
    emit("R-F10.memoized.all_yes.didnt", memo_all_yes[0])
    # chapter 04's class: every seat with every outcome of its three asks
    class_yes = [0, 0]
    class_memo = [0, 0]
    for seat in range(CLASS_SIZE):
        copied = seat in COPIED_SEATS
        for run in product(OUTCOMES, repeat=ASKS):
            if all(says(copied, o) for o in run):
                class_yes[int(copied)] += 1
        for o in OUTCOMES:
            if all(says(copied, x) for x in (o,) * ASKS):
                class_memo[int(copied)] += 1
    emit("R-F15.all_yes.copied", class_yes[1])
    emit("R-F15.all_yes.didnt", class_yes[0])
    emit("R-F15.all_yes", sum(class_yes))
    check("the class's three-yes outcomes give the closed-form chance",
          Fraction(class_yes[1], sum(class_yes)) == after)
    memo_after = Fraction(class_memo[1], sum(class_memo))
    emit("R-F10.memoized.posterior", memo_after)
    check("a memoized three yeses leave one yes's chance",
          memo_after == posterior(share, sum(says(True, o) for o in OUTCOMES), sum(says(False, o) for o in OUTCOMES)))
    emit("R-F15.posterior.all_yes_at_half", posterior(Fraction(1, 2), all_yes[1], all_yes[0]))
    emit("R-F15.posterior.all_yes_none_copied", posterior(Fraction(0), all_yes[1], all_yes[0]))

    # R-F16: the dial, every (slice, second-coin face) outcome of the spinner
    emit("R-F16.slices", SLICES)
    emit("R-F16.outcomes", 2 * SLICES)
    ratios, wobbles, same_ratio, ratio_form, same_wobble, wobble_form = {}, {}, [], [], [], []
    misses, miss_rounding = {}, []
    for name, k in DIAL:
        dial = Fraction(k, SLICES)
        spins = [(s < k, face) for s in range(SLICES) for face in "HT"]

        def spun(copied, spin):
            truthful, face = spin
            return copied if truthful else face == "H"

        said_yes = {c: sum(spun(c, spin) for spin in spins) for c in (True, False)}
        said_no = {c: len(spins) - said_yes[c] for c in (True, False)}
        emit(f"R-F16.{name}.truth", dial)
        emit(f"R-F16.{name}.truth_slices", k)
        emit(f"R-F16.{name}.yes.copied", said_yes[True])
        emit(f"R-F16.{name}.yes.didnt", said_yes[False])
        r_yes = Fraction(said_yes[True], said_yes[False]) if said_yes[False] else None
        r_no = Fraction(said_no[False], said_no[True]) if said_no[True] else None
        same_ratio.append(r_yes == r_no)
        ratio_form.append(r_yes == ((1 + dial) / (1 - dial) if dial < 1 else None))
        ratios[name] = r_yes
        emit(f"R-F16.{name}.ratio", "unbounded" if r_yes is None else r_yes)
        # one person's share of the estimate, (answer - (1 - t)/2) / t, over the spins
        wobble = None
        if dial > 0:
            per_truth = []
            for c in (True, False):
                xs = [(spun(c, spin) - (1 - dial) / 2) / dial for spin in spins]
                mean_x = sum(xs) / len(xs)
                per_truth.append(sum((x - mean_x) ** 2 for x in xs) / len(xs))
            same_wobble.append(per_truth[0] == per_truth[1])
            wobble = per_truth[0]
            wobble_form.append(wobble == (1 - dial * dial) / (4 * dial * dial))
        wobbles[name] = wobble
        emit(f"R-F16.{name}.coin_variance", "none" if wobble is None else wobble)
        # the fixed class of n: its count's variance and usual miss (the square root)
        class_var = None if wobble is None else n * wobble
        emit(f"R-F16.{name}.class_variance", "none" if class_var is None else class_var)
        emit(f"R-F16.{name}.miss_2dp", "none" if class_var is None else f"{math.sqrt(class_var):.2f}")
        if class_var is None:
            emit(f"R-F16.{name}.miss_rounded", "none")
        else:
            # nearest whole to sqrt(p/q), halves up: the largest m with (2m - 1)^2 q <= 4p,
            # found by the integer square root of 4pq
            p, q = class_var.numerator, class_var.denominator
            nearest_root = (math.isqrt(4 * p * q) // q + 1) // 2
            miss_rounding.append(abs(nearest_root - math.sqrt(class_var)) <= 0.5)
            misses[name] = nearest_root
            emit(f"R-F16.{name}.miss_rounded", nearest_root)
    check("the dial: a yes and a no carry the same ratio at every setting", all(same_ratio))
    check("the dial: the ratio is (1 + t)/(1 - t), and unbounded at t = 1", all(ratio_form))
    check("the dial: the coins' variance is the same for both truths, and (1 - t^2)/(4t^2)",
          all(same_wobble) and all(wobble_form) and len(wobble_form) == len(DIAL) - 1)
    check("the dial's middle is chapter 01's coin: 3 times, 3/4 a person",
          ratios["ours"] == ratio(True) and wobbles["ours"] == per_person)
    check("the dial's edges: at 0 no estimate, at 1 nobody who didn't copy says yes",
          estimate(yes, n, Fraction(0)) is None and ratios["direct"] is None and ratios["pure"] == 1)
    check("the dial's misses: each exact nearest root within 1/2 of the square root; at 1/2, chapter 03's",
          all(miss_rounding) and len(misses) == len(DIAL) - 1
          and n * wobbles["ours"] == est_var and misses["ours"] == round(math.sqrt(est_var)))
    emit("R-F16.up.wobble_vs_ours", wobbles["up"] / wobbles["ours"])
    emit("R-F16.up.wobble_vs_ours_nearest_quarter", Fraction(nearest(4 * wobbles["up"] / wobbles["ours"]), 4))
    emit("R-F16.down.wobble_vs_ours", wobbles["down"] / wobbles["ours"])
    emit("R-F16.down.ratio_under", math.floor(ratios["down"]) + 1)
    emit("R-F16.class.count_variance", n * wobbles["ours"])
    check("a fixed class's count: n people's coins, chapter 03's estimate variance", n * wobbles["ours"] == est_var)
    emit("R-F16.class.share_variance", wobbles["ours"] / n)

    # R-F17: chapter 01's chart, each seat's yes chance from the weighted routes
    def average_yes(truth, k):
        yc, yd = channel(truth)
        return sum(yc if seat < k else yd for seat in range(n))
    alone = {average_yes(Fraction(0), k) for k in range(n + 1)}
    check("a coin alone gives the same yeses whoever copied", len(alone) == 1)
    emit("R-F17.coin_alone.average_yes", alone.pop())
    emit("R-F17.direct.average_yes", average_yes(Fraction(1), copiers))
    emit("R-F17.rule.average_yes", average_yes(t, copiers))
    check("the rule's line at the class is chapter 01's seven", average_yes(t, copiers) == yes)
    emit("R-F17.rule.at_none", average_yes(t, 0))
    emit("R-F17.rule.at_all", average_yes(t, n))

    # R-F18: the class of forty, solved; the recipe unbiased for every k
    forty, forty_yes = FORTY
    emit("R-F18.class", forty)
    emit("R-F18.yes", forty_yes)
    emit("R-F18.forced_yes", forty * (1 - t) / 2)
    emit("R-F18.left", forty_yes - forty * (1 - t) / 2)
    emit("R-F18.estimate", estimate(forty_yes, forty, t))
    unbiased = all(moments(yes_distribution([True] * k + [False] * (n - k))[0],
                           lambda y: estimate(y, n, t))[0] == k for k in range(n + 1))
    emit("R-F18.unbiased_every_k", "true" if unbiased else "false")
    emit("R-F18.shown.yes", yeses[SHOWN_RUN])
    emit("R-F18.shown.estimate", estimates[SHOWN_RUN])
    check("the shown class misses, inside 0 to n", estimates[SHOWN_RUN] != copiers and 0 <= estimates[SHOWN_RUN] <= n)

    # R-F19: the misses, read from the person-by-person distribution
    miss_by = round(math.sqrt(est_var))
    def p_miss(by):
        return sum(p for y, p in enumerate(dist) if abs(estimate(y, n, t) - copiers) >= by)
    emit("R-F19.miss_by", miss_by)
    emit("R-F19.miss_at_least", p_miss(miss_by))
    emit("R-F19.miss_at_least.over", Fraction(2, 5))
    check("more than two in five miss by four or more, fewer than half",
          Fraction(2, 5) < p_miss(miss_by) < Fraction(1, 2))
    emit("R-F19.percent.miss_at_least", percent(p_miss(miss_by)))
    emit("R-F19.miss_at_least_twice", p_miss(2 * miss_by))
    emit("R-F19.one_in.miss_at_least_twice", nearest(1 / p_miss(2 * miss_by)))
    check("estimates are even, so missing by more than sqrt(15) is missing by 4 or more",
          p_miss(Fraction(387, 100)) == p_miss(miss_by))
    big_copied = BIG_CLASS * share
    big_vars = set()
    for k in (int(big_copied), 0):
        big_dist, _ = yes_distribution([True] * k + [False] * (BIG_CLASS - k))
        m, v = moments(big_dist, lambda y: 2 * y - Fraction(BIG_CLASS, 2))
        check(f"three hundred with {k} copiers: right on average", m == k)
        big_vars.add(v)
    check("three hundred's estimate variance is the same whoever copied", len(big_vars) == 1)
    big_var = big_vars.pop()
    big_miss = math.isqrt(big_var.numerator)
    check("three hundred's variance is a whole square", big_var.denominator == 1 and big_miss ** 2 == big_var)
    emit("R-F19.big.class", BIG_CLASS)
    emit("R-F19.big.copied", big_copied)
    emit("R-F19.big.variance", big_var)
    emit("R-F19.big.miss", big_miss)
    emit("R-F19.big.miss_share", Fraction(big_miss, BIG_CLASS))
    emit("R-F19.big.percent.miss_share", percent(Fraction(big_miss, BIG_CLASS)))
    class_share = math.sqrt(est_var) / n
    emit("R-F19.class.miss_share_2dp", f"{class_share:.2f}")
    emit("R-F19.class.percent.miss_share", f"{100 * class_share:.0f}")
    emit("R-F19.class.one_in.miss_share", round(1 / class_share))

    # R-F20: twenty drawn from a big school; the share in percentage points
    direct_n = direct / n
    emit("R-F20.direct.points_2dp", f"{100 * math.sqrt(direct_n):.2f}")
    emit("R-F20.direct.points", round(100 * math.sqrt(direct_n)))
    emit("R-F20.coins.points_2dp", f"{100 * math.sqrt(coins_var / n):.2f}")
    emit("R-F20.coins.points", round(100 * math.sqrt(coins_var / n)))
    m = 1
    while coins_var / m > direct_n:
        m += 1
    emit("R-F20.match.exact", coins_var / direct_n)
    emit("R-F20.match", m)
    emit("R-F20.match.points_2dp", f"{100 * math.sqrt(coins_var / m):.2f}")
    check("the matching count over twenty is the cost", Fraction(coins_var / direct_n) / n == coins_var / direct)

    # R-F21: the four who copied always say no; the others follow the rule
    broken, _ = yes_distribution(people, copiers_say_no=True)
    broken_yes, _ = moments(broken, lambda y: y)
    broken_est, _ = moments(broken, lambda y: estimate(y, n, t))
    emit("R-F21.average_yes", broken_yes)
    emit("R-F21.twice_yes", broken_yes / t)
    emit("R-F21.average_estimate", broken_est)
    emit("R-F21.bias", broken_est - copiers)
    # one random person's contribution to the estimated share, enumerated
    contribution = Fraction(0)
    for copied, w in ((True, share), (False, 1 - share)):
        for o in OUTCOMES:
            said = False if copied else says(False, o)
            contribution += w * Fraction(1, len(OUTCOMES)) * (int(said) - (1 - t) / 2) / t
    emit("R-F21.share_mean", contribution)
    check("the share's average is the class's over n", contribution * n == broken_est)
    for size in ASKED:
        k = int(size * share)
        d, _ = yes_distribution([True] * k + [False] * (size - k), copiers_say_no=True)
        mean_est, _ = moments(d, lambda y: estimate(y, size, t))
        check(f"{size} asked: the estimated share still averages {contribution}", mean_est / size == contribution)
    emit("R-F21.asked", csv(ASKED))

    # R-F22: a hundred fresh answers, seventy-five yes, from even odds
    ry, rn = RAPPOR
    like_c = Fraction(3, 4) ** ry * Fraction(1, 4) ** (rn - ry)
    like_d = Fraction(1, 4) ** ry * Fraction(3, 4) ** (rn - ry)
    ratio_many = like_c / like_d
    no_many = like_d / (like_c + like_d)
    emit("R-F22.asks", rn)
    emit("R-F22.yes", ry)
    emit("R-F22.no", rn - ry)
    emit("R-F22.excess", ry - (rn - ry))
    emit("R-F22.ratio", ratio_many)
    emit("R-F22.ratio_digits", len(str(ratio_many)))
    emit("R-F22.posterior_no", no_many)
    zeros, x = 0, no_many
    while x * 10 < 1:
        x *= 10
        zeros += 1
    emit("R-F22.zeros", zeros)
    check("the decimal: 23 zeros, then 139", f"{float(no_many):.30f}".startswith("0." + "0" * zeros + "139"))

    # R-F23: a fixed class's estimate variance is 3n/4 for every n and every k
    every = all(moments(yes_distribution([True] * k + [False] * (size - k))[0],
                        lambda y: 2 * y - Fraction(size, 2))[1] == Fraction(3 * size, 4)
                for size in CHECKED_SIZES for k in range(size + 1))
    check("variance 3n/4 at every checked size and every number of copiers", every)
    emit("R-F23.per_person", per_person)
    emit("R-F23.checked_sizes", csv(CHECKED_SIZES))

    # R-X: the edges
    emit("R-X1.direct.worst_ratio", "unbounded" if worst_ratio(Fraction(1)) is None else worst_ratio(Fraction(1)))
    emit("R-X1.pure_coin.worst_ratio", worst_ratio(Fraction(0)))
    emit("R-X1.pure_coin.estimate", "none" if estimate(yes, n, Fraction(0)) is None else estimate(yes, n, Fraction(0)))
    check("a pure coin's answers are as likely from everyone", channel(Fraction(0))[0] == channel(Fraction(0))[1])
    emit("R-X2.estimate.no_yes", (0 - forced_yes) / t)
    emit("R-X2.estimate.all_yes", (n - forced_yes) / t)
    emit("R-X4.average_needs_multiple_of", len(OUTCOMES))
    ry, rn = RAPPOR
    like_yes = math.comb(rn, ry) * 3 ** ry          # copied: 3/4 each yes, 1/4 each no (x 4^n)
    like_no = math.comb(rn, ry) * 3 ** (rn - ry)    # didn't: 1/4 each yes, 3/4 each no
    no_after = Fraction(like_no, like_yes + like_no)
    emit("R-X3.rappor.no_after_75_of_100", f"{float(no_after):.2e}")
    check("RAPPOR 1: No after 75 of 100 is 1/(3^50 + 1)", no_after == Fraction(1, 3 ** 50 + 1))
    # RAPPOR's own words, typed from arXiv 1407.6981v2 (read 2026-10-20)
    emit("R-X6.rappor.compliance", "an assumption that may not hold for human subjects")
    emit("R-X7.rappor.year", 2014)
    emit("R-X7.rappor.memoized", "Permanent randomized response")
    emit("R-X7.rappor.each_report", "Instantaneous randomized response")
    emit("R-X7.rappor.deployment", "limited deployment in Google's Chrome Web browser")
    emit("R-X8.rappor.prior", "irrespective of any attacker's prior knowledge")

    print(f"# {_CHECKS[0]} checks, {len(_FAILED)} failed", file=sys.stderr)
    return 1 if _FAILED else 0


if __name__ == "__main__":
    sys.exit(main())
