"""Independent exact oracle for the apportionment film (Adding a Seat).

    python3 -I -B oracle.py            # KEY<TAB>VALUE lines on stdout
    python3 -I -B oracle.py --check    # also print cross-checks to stderr

Every number the film shows or says is printed here from exact rational
arithmetic (fractions.Fraction); nothing is a float. The Rust facts
(src/facts.rs, src/model.rs) must print the same keys with the same values;
tools/facts_check.py compares them.

Independence from the Rust side, on purpose:
- divisor methods here are solved the textbook way, by searching for a
  divisor d (exact bisection on rationals) whose rounded quotients add up to
  the house size; the Rust side instead walks the list of seat openings
  ("signposts") in order. Two formulations, one answer.
- Hamilton here sorts exact Fraction tails; the Rust side compares integer
  remainders.
- the quota method here tests eligibility with Fraction quotas and ceil; the
  Rust side uses integer cross-multiplication.
Ties at a cutoff raise an error in both: the film's examples have none.

Historical and published numbers cannot be computed; they are printed as
pinned literals with their source in a comment, and the Rust side pins the
same literals (Facts::pin) so a typo on either side fails the comparison.
"""

import sys
from fractions import Fraction as F
from math import floor, ceil

TOWNS = ["ash", "birch", "cedar"]
POP = {"ash": 4000, "birch": 9000, "cedar": 26000}
# the census taken later (chapter 13): Ash shrinks, Birch and Cedar grow
POP2 = {"ash": 3900, "birch": 9500, "cedar": 26100}
# the same census one town at a time (chapter 13 moves the cards in three
# steps: Birch grows, then Cedar grows, then Ash shrinks to POP2)
BIRCH_GROWN = {"ash": POP["ash"], "birch": POP2["birch"], "cedar": POP["cedar"]}
GROWN = {"ash": POP["ash"], "birch": POP2["birch"], "cedar": POP2["cedar"]}
# chapter 12 counts whole seat tiles in the stretch of house sizes 6 to 13
STRETCH = (6, 13)
# chapter 15's try-it-yourself: Cedar grows, Ash and Birch stay the same
PUZZLE = {"ash": 4000, "birch": 9000, "cedar": 30000}
# the four-town map that breaks Webster's quota (chapter 11)
W4 = ["a", "b", "c", "d"]
POP4 = {"a": 2000, "b": 5000, "c": 8000, "d": 27000}
# Goelz, Peters and Procaccia (2024), Table 1, house size 10: five states,
# three population profiles (arXiv 2202.11061v2, p. 9)
GPP = {
    "A": [824, 44, 44, 44, 44],
    "B": [824, 44, 44, 44, 222],
    "C": [824, 1, 1, 44, 44],
}
GPP_TABLE_QUOTAS = {  # as printed in Table 1 (two decimals)
    "A": ["8.24", "0.44", "0.44", "0.44", "0.44"],
    "B": ["6.99", "0.37", "0.37", "0.37", "1.88"],
    "C": ["9.02", "0.01", "0.01", "0.48", "0.48"],
}

out = []


def emit(key, value):
    out.append(f"{key}\t{value}")


def dec(x, places):
    """Exact decimal rounding, halves away from zero (x >= 0 here)."""
    x = F(x)
    scale = 10**places
    n = floor(x * scale + F(1, 2))
    if places == 0:
        return str(n)
    s = str(n).rjust(places + 1, "0")
    return s[:-places] + "." + s[-places:]


def comma(n):
    return f"{n:,}"


def mixed(x):
    """A Fraction as a whole number and a proper fraction: '9 3/4'."""
    x = F(x)
    whole, rest = divmod(x.numerator, x.denominator)
    if rest == 0:
        return str(whole)
    part = F(rest, x.denominator)
    frac = f"{part.numerator}/{part.denominator}"
    return frac if whole == 0 else f"{whole} {frac}"


def sqrt_2dp(x):
    """√x rounded half up to two decimals, by Decimal at 50 digits (the Rust
    side uses an integer square root instead)."""
    from decimal import Decimal, ROUND_HALF_UP, localcontext
    with localcontext() as ctx:
        ctx.prec = 50
        return str(Decimal(x).sqrt().quantize(Decimal("0.01"), rounding=ROUND_HALF_UP))


def num(x):
    """A whole number with thousands commas, else one decimal place."""
    x = F(x)
    if x.denominator == 1:
        return comma(x.numerator)
    whole, frac = dec(x, 1).split(".")
    return comma(int(whole)) + "." + frac


# ------------------------------------------------------------------ shares

def shares(pop, names, h):
    total = sum(pop[t] for t in names)
    return {t: F(h * pop[t], total) for t in names}


# ---------------------------------------------------------------- Hamilton

def hamilton(pop, names, h):
    q = shares(pop, names, h)
    seats = {t: floor(q[t]) for t in names}
    left = h - sum(seats.values())
    tails = sorted(names, key=lambda t: q[t] - floor(q[t]), reverse=True)
    if 0 < left < len(names):
        a, b = tails[left - 1], tails[left]
        if q[a] - floor(q[a]) == q[b] - floor(q[b]):
            raise ValueError(f"Hamilton tie at h={h}")
    for t in tails[:left]:
        seats[t] += 1
    return seats


# ----------------------------------------------------------- divisor rules

def round_jefferson(x):
    return floor(x)


def round_adams(x):
    return ceil(x)


def round_webster(x):
    return floor(x + F(1, 2))


def round_hill(x):
    n = floor(x)
    return n + 1 if x * x >= n * (n + 1) else n


ROUND = {
    "jefferson": round_jefferson,
    "adams": round_adams,
    "webster": round_webster,
    "hill": round_hill,
}


def divisor_method(rule, pop, names, h, steps=400):
    """Find d with sum(round(p/d)) == h by exact bisection; error on a tie."""
    r = ROUND[rule]

    def total(d):
        return sum(r(F(pop[t]) / d) for t in names)

    lo, hi = F(1, 10**6), F(sum(pop.values()) * 10)  # total(lo) huge, total(hi) small
    for _ in range(steps):
        mid = (lo + hi) / 2
        s = total(mid)
        if s == h:
            return {t: r(F(pop[t]) / mid) for t in names}, mid
        if s > h:
            lo = mid
        else:
            hi = mid
    raise ValueError(f"{rule}: no divisor gives exactly {h} seats (a tie)")


def try_divisor(rule, pop, names, h):
    try:
        return divisor_method(rule, pop, names, h)[0]
    except ValueError:
        return None


def divisor_window(rule, pop, names, h):
    """The exact interval of divisors giving h seats, found from its edges:
    each town's seat-opening divisors, tested at midpoints (oracle-side)."""
    seats, d = divisor_method(rule, pop, names, h)
    # every divisor that changes some town's rounding
    edges = set()
    for t in names:
        for k in range(1, 60):
            if rule == "jefferson":
                edges.add(F(pop[t], k))
            elif rule == "webster":
                edges.add(F(pop[t]) / (k - F(1, 2)))
    edges = sorted(edges)
    below = max(e for e in edges if e <= d)
    above = min(e for e in edges if e > d)
    return seats, below, above


# ------------------------------------------------- Balinski-Young quota rule

def quota_method(pop, names, house):
    """Seats awarded one at a time, highest population over (seats + 1)
    first, skipping any town whose next seat would put it above the ceiling
    of its share at the new house size."""
    seats = {t: 0 for t in names}
    history = {0: dict(seats)}
    total = sum(pop[t] for t in names)
    for h in range(1, house + 1):
        eligible = [t for t in names if seats[t] + 1 <= ceil(F(h * pop[t], total))]
        best = max(eligible, key=lambda t: F(pop[t], seats[t] + 1))
        ties = [t for t in eligible if F(pop[t], seats[t] + 1) == F(pop[best], seats[best] + 1)]
        if len(ties) > 1:
            return history, h  # stop: the next seat has no single winner
        seats[best] += 1
        history[h] = dict(seats)
    return history, None


def within_quota(seats, pop, names, h):
    q = shares(pop, names, h)
    return all(floor(q[t]) <= seats[t] <= ceil(q[t]) for t in names)


# ------------------------------------------------------------------- facts

def main(check):
    total = sum(POP.values())
    for t in TOWNS:
        emit(f"pop.{t}", comma(POP[t]))
    emit("pop.total", comma(total))

    # shares, whole parts, tails (in 39ths: populations are thousands)
    for h in (14, 15, 16, 17):
        q = shares(POP, TOWNS, h)
        for t in TOWNS:
            emit(f"share.{h}.{t}", dec(q[t], 2))
            emit(f"whole.{h}.{t}", floor(q[t]))
            tail = q[t] - floor(q[t])
            emit(f"tail39.{h}.{t}", tail * 39)  # an integer: 39 ths of a seat
            emit(f"tail.{h}.{t}", dec(tail, 2))
        emit(f"leftover.{h}", h - sum(floor(q[t]) for t in TOWNS))
    # 15 seats in thirteenths (Ash 7/13, Birch 6/13)
    q15 = shares(POP, TOWNS, 15)
    emit("tail13.15.ash", (q15["ash"] - floor(q15["ash"])) * 13)
    emit("tail13.15.birch", (q15["birch"] - floor(q15["birch"])) * 13)
    emit("std_divisor.15", comma(total // 15))
    # the speed of a tail: share gained per added seat
    for t in TOWNS:
        emit(f"speed39.{t}", F(POP[t] * 39, total))
        emit(f"speed.{t}", dec(F(POP[t], total), 2))

    # naive rounding to the nearest whole seat
    for h in (14, 15, 16):
        q = shares(POP, TOWNS, h)
        r = {t: floor(q[t] + F(1, 2)) for t in TOWNS}
        for t in TOWNS:
            emit(f"naive.{h}.{t}", r[t])
        emit(f"naive.{h}.total", sum(r.values()))

    # Hamilton
    for h in (14, 15, 16, 17):
        s = hamilton(POP, TOWNS, h)
        for t in TOWNS:
            emit(f"hamilton.{h}.{t}", s[t])
    # how often does adding a seat cost someone a seat, from 1 seat to 100
    events = []
    for h in range(1, 100):
        a, b = hamilton(POP, TOWNS, h), hamilton(POP, TOWNS, h + 1)
        losers = [t for t in TOWNS if b[t] < a[t]]
        if losers:
            events.append((h, losers))
    emit("alabama.additions", 99)
    emit("alabama.count", len(events))
    emit("alabama.loser", ",".join(sorted({t for _, ls in events for t in ls})))
    emit("alabama.houses", ",".join(f"{h}to{h + 1}" for h, _ in events))
    # Hamilton never leaves quota (checked, not assumed)
    hq = all(within_quota(hamilton(POP, TOWNS, h), POP, TOWNS, h) for h in range(1, 101))
    emit("hamilton.within_quota_1_100", "yes" if hq else "no")

    # Jefferson: the people-per-seat window at 15 seats, an example divisor
    seats, lo, hi = divisor_window("jefferson", POP, TOWNS, 15)
    emit("jefferson.15.window_low", num(lo))
    emit("jefferson.15.window_high", num(hi))
    d = F(2300)
    js = {t: floor(F(POP[t]) / d) for t in TOWNS}
    emit("jefferson.example_divisor", comma(2300))
    for t in TOWNS:
        emit(f"jefferson.at2300.{t}", dec(F(POP[t]) / d, 2))
    emit("jefferson.at2300.total", sum(js.values()))
    q = shares(POP, TOWNS, 15)
    for t in TOWNS:
        emit(f"at2600.{t}", dec(q[t], 2))  # dividing by 2,600 is the share
    emit("at2600.whole_total", sum(floor(q[t]) for t in TOWNS))
    # what dropping the fraction costs at 2,300 people a seat, as a percent
    for t in ("ash", "cedar"):
        x = F(POP[t]) / d
        emit(f"jefferson.drop_pct.{t}", dec((x - floor(x)) / x * 100, 1))
    for h in (14, 15, 16, 17):
        s, _ = divisor_method("jefferson", POP, TOWNS, h)
        for t in TOWNS:
            emit(f"jefferson.{h}.{t}", s[t])

    # the opening list (Jefferson signposts): k-th seat of a town opens when
    # people-per-seat falls to population / k. Listed here independently by
    # sorting every (value, town, k) exactly; ties are reported.
    openings = sorted(
        ((F(POP[t], k), t, k) for t in TOWNS for k in range(1, 20)),
        key=lambda e: (-e[0], TOWNS.index(e[1])),
    )
    for i, (val, t, k) in enumerate(openings[:19], 1):
        emit(f"opening.{i}", f"{t}{k}@{comma(floor(val + F(1, 2)))}")
    first_tie = next(
        i for i in range(1, len(openings)) if openings[i][0] == openings[i - 1][0]
    )
    emit("opening.first_tie_rank", first_tie)  # 1-based rank of the earlier tied entry
    emit("opening.first_tie_value", comma(int(openings[first_tie][0])))

    # Adams, Webster, Hill
    for rule, houses in (("adams", (15,)), ("webster", (15, 16, 17)), ("hill", (15, 16, 17))):
        for h in houses:
            s, _ = divisor_method(rule, POP, TOWNS, h)
            for t in TOWNS:
                emit(f"{rule}.{h}.{t}", s[t])
    # every rule's result at 15 seats against the quota bounds
    # scans over 1..100 seats: a house size where a rule ties at its cutoff
    # has no single answer; it is skipped and counted, never broken by fiat
    for rule in ("jefferson", "adams", "webster", "hill"):
        bad, ties, upper, lower = 0, [], 0, 0
        for h in range(1, 101):
            s = try_divisor(rule, POP, TOWNS, h)
            if s is None:
                ties.append(h)
                continue
            q = shares(POP, TOWNS, h)
            if not within_quota(s, POP, TOWNS, h):
                bad += 1
            upper += any(s[t] > ceil(q[t]) for t in TOWNS)
            lower += any(s[t] < floor(q[t]) for t in TOWNS)
        emit(f"{rule}.quota_breaks_1_100", bad)
        emit(f"{rule}.above_quota_1_100", upper)
        emit(f"{rule}.below_quota_1_100", lower)
        emit(f"{rule}.tie_houses_1_100", ",".join(map(str, ties)) or "none")
        if rule == "hill":
            below = [h for h in range(1, 101) if h not in ties and any(
                try_divisor(rule, POP, TOWNS, h)[t] < floor(shares(POP, TOWNS, h)[t]) for t in TOWNS)]
            emit("hill.below_quota_houses_1_100", ",".join(map(str, below)))
            s3 = try_divisor(rule, POP, TOWNS, 3)
            for t in TOWNS:
                emit(f"hill.3.{t}", s3[t])
            emit("share.3.cedar", dec(shares(POP, TOWNS, 3)["cedar"], 2))

    # Hill rounds up past √(n(n+1)) between n and n + 1
    for n in range(4):
        emit(f"hill.threshold.{n}", sqrt_2dp(n * (n + 1)))

    # Webster leaves quota: the four-town map
    total4 = sum(POP4.values())
    for t in W4:
        emit(f"w4.pop.{t}", comma(POP4[t]))
    emit("w4.pop.total", comma(total4))
    q4 = shares(POP4, W4, 14)
    # rounding to the nearest lifts each small share by ceil - share, in thirds
    lifts = [(ceil(q4[t]) - q4[t]) * 3 for t in W4[:3]]
    assert all(x.denominator == 1 for x in lifts), lifts
    for t, x in zip(W4[:3], lifts):
        emit(f"w4.lift_thirds.{t}", x)
    emit("w4.lift_thirds.total", sum(lifts))
    for t in W4:
        emit(f"w4.share.14.{t}", dec(q4[t], 2))
    s4, lo4, hi4 = divisor_window("webster", POP4, W4, 14)
    for t in W4:
        emit(f"w4.webster.14.{t}", s4[t])
    emit("w4.webster.14.within_quota", "yes" if within_quota(s4, POP4, W4, 14) else "no")
    emit("w4.window_low", num(lo4))
    emit("w4.window_high", num(hi4))
    # (Hamilton has no single answer here: three tails tie at two thirds)
    # with three towns Webster stayed within quota at every house size here
    emit("webster.three_town_quota_breaks_1_100",
         sum(1 for h in range(1, 101)
             if not within_quota(divisor_method("webster", POP, TOWNS, h)[0], POP, TOWNS, h)))

    # quota method (Balinski and Young, PNAS 1974, p. 4604): Jefferson's
    # priority among the towns still within their upper quota
    hist, first_tie = quota_method(POP, TOWNS, 100)
    last = max(hist)
    for h in (14, 15, 16, 17):
        for t in TOWNS:
            emit(f"quotamethod.{h}.{t}", hist[h][t])
    ok = all(within_quota(hist[h], POP, TOWNS, h) for h in range(1, last + 1))
    mono = all(hist[h + 1][t] >= hist[h][t] for h in range(0, last) for t in TOWNS)
    emit("quotamethod.first_tie_house", first_tie if first_tie else "none")
    emit("quotamethod.checked_through", last)
    emit("quotamethod.within_quota_checked", "yes" if ok else "no")
    emit("quotamethod.never_takes_back_checked", "yes" if mono else "no")
    # at 15 seats Cedar is capped: its ceiling is exactly 10
    emit("quotamethod.15.cedar_ceiling", ceil(shares(POP, TOWNS, 15)["cedar"]))
    emit("quotamethod.15.birch_priority", comma(POP["birch"] // 4))
    emit("quotamethod.15.ash_priority", comma(POP["ash"] // 2))
    # chapter 12's tiles: town t's k-th seat spans house sizes (k-1)P/p to kP/p
    for t in TOWNS:
        emit(f"tile.len.{t}", mixed(F(total, POP[t])))
    emit("tile.end.cedar.11", mixed(F(11 * total, POP["cedar"])))
    emit("tile.end.birch.4", mixed(F(4 * total, POP["birch"])))
    a, b = STRETCH
    emit("stretch.from", a)
    emit("stretch.to", b)
    emit("stretch.seats", b - a)
    inside = 0
    for t in TOWNS:
        length = F(total, POP[t])
        # whole tiles in [a, b]: k with (k-1)L >= a and kL <= b
        n = max(0, floor(F(b) / length) - ceil(F(a) / length))
        emit(f"stretch.inside.{t}", n)
        inside += n
    emit("stretch.inside.total", inside)
    # chapter 12's picture of the supposition: the run as it really went up to
    # 6, the seat handed out there, and the tile supposed left out
    emit("stretch.need", b - a + 1)
    for t in TOWNS:
        emit(f"stretch.then.{t}", hist[a][t])
        emit(f"stretch.now.{t}", hist[b][t])
    later = [t for t in TOWNS if hist[a][t] == hist[a - 1][t] + 1]
    assert len(later) == 1
    later_seat = hist[a][later[0]]
    later_end = F(later_seat * total, POP[later[0]])
    emit("stretch.later.town", later[0])
    emit("stretch.later.seat", later_seat)
    emit("stretch.later.end", mixed(later_end))
    # the next tile, k = seats + 1, spans k-1 to k tile lengths: not yet entered
    # at a, ending sooner than the tile the rule took, and passed by b
    waiting = [t for t in TOWNS
               if F(hist[a][t] * total, POP[t]) >= a
               and F((hist[a][t] + 1) * total, POP[t]) < later_end
               and F((hist[a][t] + 1) * total, POP[t]) <= b]
    assert len(waiting) == 1
    missing_seat = hist[a][waiting[0]] + 1
    emit("stretch.missing.town", waiting[0])
    emit("stretch.missing.seat", missing_seat)
    emit("stretch.missing.start", mixed(F((missing_seat - 1) * total, POP[waiting[0]])))
    emit("stretch.missing.end", mixed(F(missing_seat * total, POP[waiting[0]])))

    # the population paradox: a later census, still 16 seats
    total2 = sum(POP2.values())
    for t in TOWNS:
        emit(f"census2.pop.{t}", comma(POP2[t]))
    emit("census2.pop.total", comma(total2))
    q2 = shares(POP2, TOWNS, 16)
    for t in TOWNS:
        emit(f"census2.share.16.{t}", dec(q2[t], 2))
    h2 = hamilton(POP2, TOWNS, 16)
    for t in TOWNS:
        emit(f"census2.hamilton.16.{t}", h2[t])
    for rule in ("jefferson", "webster"):
        s1, _ = divisor_method(rule, POP, TOWNS, 16)
        s2, _ = divisor_method(rule, POP2, TOWNS, 16)
        for t in TOWNS:
            emit(f"census2.{rule}.16.{t}", s2[t])
        # growing Cedar never loses while shrinking Ash gains, under divisors
        emit(f"census2.{rule}.paradox", "yes" if (s2["cedar"] < s1["cedar"] and s2["ash"] > s1["ash"]) else "no")

    # the signpost lists the film moves its cards through: Jefferson before
    # the census (opening.*), with only the growth, and at the census;
    # Webster before and with chapter 15's grown Cedar
    def ranked(value, pops, count):
        entries = sorted(
            ((value(pops[t], k), t, k) for t in TOWNS for k in range(1, count + 2)),
            key=lambda e: (-e[0], TOWNS.index(e[1])),
        )
        return entries[:count]

    jeff = lambda p, k: F(p, k)
    webs = lambda p, k: F(2 * p, 2 * k - 1)
    for name, value, pops, count in (
        ("census2.birch", jeff, BIRCH_GROWN, 19),
        ("census2.grown", jeff, GROWN, 19),
        ("census2", jeff, POP2, 19),
        ("webster", webs, POP, 17),
        ("puzzle.webster", webs, PUZZLE, 17),
    ):
        for i, (val, t, k) in enumerate(ranked(value, pops, count), 1):
            emit(f"{name}.opening.{i}", f"{t}{k}@{comma(floor(val + F(1, 2)))}")
    emit("puzzle.pop.cedar", comma(PUZZLE["cedar"]))
    # a lottery that keeps the quota promise and gives Ash its share on
    # average must give Ash 2 seats with probability equal to its tail
    qa = F(15 * POP["ash"], sum(POP.values()))
    emit("lottery.15.ash.two_pct", dec((qa - floor(qa)) * 100, 0))
    emit("lottery.15.ash.one_pct", dec((1 - (qa - floor(qa))) * 100, 0))
    sp, _ = divisor_method("webster", PUZZLE, TOWNS, 15)
    for t in TOWNS:
        emit(f"puzzle.webster.15.{t}", sp[t])
    # Ash's next Webster signpost after its fifteen seats (shown in 15_4)
    w15, _ = divisor_method("webster", POP, TOWNS, 15)
    emit("webster.15.ash_next_signpost", comma(floor(webs(POP["ash"], w15["ash"] + 1) + F(1, 2))))
    if check:
        s15, _ = divisor_method("webster", POP, TOWNS, 15)
        assert sp["ash"] <= s15["ash"] and sp["birch"] <= s15["birch"], "a grown Cedar cannot raise Ash or Birch"
        print(f"check: puzzle Webster 15 {[sp[t] for t in TOWNS]} vs {[s15[t] for t in TOWNS]}", file=sys.stderr)

    # Goelz, Peters and Procaccia's five-state proof, house size 10
    for prof, pops in GPP.items():
        tot = sum(pops)
        emit(f"gpp.{prof}.total", tot)
        for i, p in enumerate(pops, 1):
            qq = F(10 * p, tot)
            emit(f"gpp.{prof}.pop.{i}", p)
            emit(f"gpp.{prof}.share.{i}", dec(qq, 2))
            emit(f"gpp.{prof}.floor.{i}", floor(qq))
            emit(f"gpp.{prof}.ceil.{i}", ceil(qq))
        if check:
            printed = [dec(F(10 * p, tot), 2) for p in pops]
            assert printed == GPP_TABLE_QUOTAS[prof], (prof, printed)

    # pinned history (cannot be computed; sources in the storyboard notes)
    emit("history.seaton.house_before", 299)   # Young, census PDF p. 13, quoting C. W. Seaton
    emit("history.seaton.house_after", 300)
    emit("history.seaton.alabama_before", 8)
    emit("history.seaton.alabama_after", 7)
    emit("history.seaton.range_low", 275)
    emit("history.seaton.range_high", 350)
    emit("history.ny1830.quota", "38.593")      # Young, census PDF p. 10, Table 4
    emit("history.ny1830.seats", 40)
    emit("history.hill_law_year", 1941)          # Young, census PDF p. 15
    emit("history.quota_method_year", 1974)      # Balinski and Young, PNAS 71(11), Nov 1974, p. 4604
    emit("history.by_impossibility_year", 1982)  # Balinski and Young 1982, as described in GPP p. 9
    emit("history.gpp_year", 2024)               # GPP v2, accepted by Operations Research, June 2024

    if check:
        cross_checks()


def cross_checks():
    """Printed to stderr: independent consistency checks, not facts."""
    say = lambda *a: print(*a, file=sys.stderr)
    # Hamilton tails at a house size add up to the leftover count
    for h in range(1, 101):
        q = shares(POP, TOWNS, h)
        assert sum(q[t] - floor(q[t]) for t in TOWNS) == h - sum(floor(q[t]) for t in TOWNS)
    say("ok tails add up to the leftover count, 1..100")
    # every divisor method is house monotone on 1..100 (the list argument)
    for rule in ROUND:
        prev = {t: 0 for t in TOWNS}
        for h in range(1, 101):
            try:
                s, _ = divisor_method(rule, POP, TOWNS, h)
            except ValueError:
                say(f"note {rule}: tie at {h} (skipped)")
                continue
            assert all(s[t] >= prev[t] for t in TOWNS), (rule, h)
            prev = s
    say("ok divisor rules never take a seat back on 1..100 (ties skipped)")
    # GPP: the quota bounds that drive the proof
    A, B, C = (shares(dict(enumerate(GPP[k])), range(5), 10) for k in "ABC")
    assert floor(A[0]) == 8 and ceil(A[0]) == 9
    assert ceil(B[0]) == 7 and ceil(B[4]) == 2
    assert floor(C[0]) == 9
    say("ok GPP bounds: A city 8 or 9; B city <= 7, fifth <= 2; C city >= 9")
    # Webster's four-town failure is a lower-quota failure of the big town
    q4 = shares(POP4, W4, 14)
    s4, _ = divisor_method("webster", POP4, W4, 14)
    assert q4["d"] == 9 and s4["d"] == 8
    say("ok four-town map: share exactly 9, Webster gives 8")


if __name__ == "__main__":
    main("--check" in sys.argv[1:])
    print("\n".join(out))
