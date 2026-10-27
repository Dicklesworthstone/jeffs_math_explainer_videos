#!/usr/bin/env python3
"""Independent values for every fact `inspection_paradox facts` prints.

    python3 -I oracle.py              # key<TAB>value lines, for tools/facts_check.py
    python3 -I oracle.py --self-test  # edge cases, negative controls and cross-checks

Standard library only, exact arithmetic (fractions.Fraction). The Rust model
computes the average wait from the gaps with the closed form
sum(g^2) / (2 sum(g)). This oracle never uses that form for the example's
numbers. It builds the bus times from the two routes and averages over the
hour second by second: a person turning up at the middle of each of the 3,600
seconds, waiting until the next bus of either route. Every bus time is a whole
minute and the wait falls in a straight line between buses, so the midpoint
average over seconds is exact, not an approximation. The closed form appears
here only as a cross-check in --self-test and in the always/never sweep (F-X1),
whose proof is the video's own argument.

Values print the way the video shows them: whole numbers as "6", fractions
reduced as "13/3", durations in minutes as "m:ss" ("4:20"), lists joined
with commas, booleans as "true"/"false".
"""

import itertools
import math
import random
import sys
from bisect import bisect_left, bisect_right
from collections import Counter
from fractions import Fraction as Q

# The continuing example: two routes, each every 12 minutes, the red bus
# two minutes after the blue one; one hour on the clock.
PERIOD = 12
OFFSET = 2
HOUR = 60
EVEN_OFFSET = 6
SECONDS = 60  # samples a minute for the exact midpoint average

# The bound sweep (F-X1): every multiset of up to this many gaps, each a whole
# number of minutes from 0 to this length, with at least one nonzero gap.
SWEEP_MAX_GAP = 12
SWEEP_MAX_COUNT = 4

# Completely random buses (chapter 09): the same average gap, in minutes; and a
# timetable with more spread than random buses at that average (three at once).
RANDOM_MEAN = Q(6)
CLUMPED_GAPS = [0, 0, 18]

# Chapter 10: two classes; a profiler's one long call and five short ones (ms).
CLASS_SIZES = [2, 10]
LONG_CALL_MS = 10
SHORT_CALL_MS = 2
SHORT_CALLS = 5

# Historical literals, from the sources named in docs/brief.md.
FELD_GROFMAN_YEAR = 1977  # Research in Higher Education 6, 215-222 (1977)
TERADA_YEAR = 1922  # Masuda & Hiraoka, arXiv:2004.14081, abstract and section 1
THIS_YEAR = 2026

# Chapter 11: someone who reads the timetable and turns up this early; then
# every passenger of the hour (one a minute) doing the same.
EARLY = 1

# Chapter 12's transfer puzzle: the red bus this many minutes after the blue.
PUZZLE_OFFSET = 4


def fmt(x):
    x = Q(x)
    return str(x.numerator) if x.denominator == 1 else f"{x.numerator}/{x.denominator}"


def clock(minutes):
    """Minutes as m:ss; refuses a value that is not a whole number of seconds."""
    s = Q(minutes) * 60
    if s.denominator != 1:
        raise ValueError(f"{minutes} min is not a whole number of seconds")
    s = s.numerator
    return f"{s // 60}:{s % 60:02d}"


def csv(xs):
    return ",".join(fmt(x) for x in xs)


# ----------------------------------------------------------------- timetable

def route(start, until=HOUR):
    """A route's bus times in [0, until): start, start + PERIOD, ..."""
    start %= PERIOD
    return list(range(start, until, PERIOD))


def bus_times(offset):
    """Both routes' buses in the hour, sorted (two buses at once appear twice)."""
    return sorted(route(0) + route(offset))


def extended(offset):
    """Bus times from the previous hour's last buses to the next hour's first."""
    t = bus_times(offset)
    return sorted([x - HOUR for x in t] + t + [x + HOUR for x in t])


def hour_gaps(offset):
    """The gap after each bus of the hour, ending at the next bus (the next
    hour's first bus closes the last gap)."""
    t = bus_times(offset) + [HOUR]
    return [b - a for a, b in zip(t, t[1:])]


def gap_before_each_bus(offset):
    t = extended(offset)
    out = []
    for x in bus_times(offset):
        i = bisect_left(t, x)
        out.append(x - t[i - 1])
    return out


def moments():
    """The middle of every second of the hour, in minutes."""
    return [Q(2 * s + 1, 2 * SECONDS) for s in range(HOUR * SECONDS)]


def wait_at(t, ext):
    return ext[bisect_right(ext, t)] - t


def since_last(t, ext):
    return t - ext[bisect_right(ext, t) - 1]


def gap_around(t, ext):
    i = bisect_right(ext, t)
    return ext[i] - ext[i - 1]


def time_average(f, offset):
    ext = extended(offset)
    ms = moments()
    return sum((f(t, ext) for t in ms), Q(0)) / len(ms)


# -------------------------------------------------------------- the people

def cycle_average_wait(gaps):
    """A timetable that repeats the whole-minute gaps `gaps` forever, averaged
    second by second over one cycle (no closed form)."""
    times = list(itertools.accumulate([0] + list(gaps[:-1])))
    total = sum(gaps)
    ext = sorted([t - total for t in times] + times + [t + total for t in times])
    ms = [Q(2 * s + 1, 2 * SECONDS) for s in range(total * SECONDS)]
    return sum((wait_at(t, ext) for t in ms), Q(0)) / len(ms)


def planners(offset):
    """Chapter 11: the hour's passengers (one at the middle of every minute),
    each ready at their dot, standing EARLY minutes before the first bus at
    or after ready + EARLY. Returns (ready, turns up, bus) for each."""
    ext = extended(offset)
    out = []
    for k in range(HOUR):
        ready = Q(2 * k + 1, 2)
        bus = ext[bisect_left(ext, ready + EARLY)]
        out.append((ready, bus - EARLY, bus))
    return out


def slider_wait(tenths):
    """The average wait over the hour with the red bus `tenths`/10 minutes
    after the blue (07_5's slider between whole minutes). Every bus falls on
    a whole second (a tenth of a minute is 6 s), so in integer half-seconds:
    a person at the middle of each second of the hour, waiting until the
    next bus strictly after them. Returns minutes."""
    blue = [720 * k for k in range(-1, 7)]
    red = [720 * k + 6 * tenths for k in range(-1, 7)]
    halves = sorted(2 * b for b in set(blue + red))
    total = 0
    for s in range(HOUR * SECONDS):
        m = 2 * s + 1
        total += halves[bisect_right(halves, m)] - m
    return Q(total, 2 * SECONDS * HOUR * SECONDS)


def readout(minutes):
    """A moving readout: minutes to the nearest whole second, halves up."""
    s = Q(minutes) * 60
    if s < 0:
        raise ValueError(f"{minutes} min is negative")
    whole = (2 * s.numerator + s.denominator) // (2 * s.denominator)
    return f"{whole // 60}:{whole % 60:02d}"


def middle_of_minute_people(offset, start, end):
    """A person at the middle of every minute in [start, end), with the wait."""
    ext = extended(offset)
    return [(Q(2 * k + 1, 2), wait_at(Q(2 * k + 1, 2), ext)) for k in range(start, end)]


def area_by_seconds(offset, a, b):
    """The waiting the moments in [a, b) hold: wait x time, summed second by second."""
    ext = extended(offset)
    return sum((wait_at(t, ext) for t in moments() if a <= t < b), Q(0)) / SECONDS


def triangle_tiles(scale):
    """Copies of a small right triangle that tile one `scale` times as large,
    counted on the lattice: the upright cells and the inverted ones."""
    upright = sum(1 for i in range(scale) for j in range(scale) if i + j < scale)
    inverted = sum(1 for i in range(scale) for j in range(scale) if i + j < scale - 1)
    return upright, inverted


# -------------------------------------------------------- the bound's sweep

def closed_form_wait(gaps):
    return Q(sum(g * g for g in gaps), 2 * sum(gaps))


def sweep():
    cases = violations = equalities = equal_and_even = 0
    for n in range(1, SWEEP_MAX_COUNT + 1):
        for gaps in itertools.combinations_with_replacement(range(SWEEP_MAX_GAP + 1), n):
            if sum(gaps) == 0:
                continue
            cases += 1
            wait = closed_form_wait(gaps)
            half = Q(sum(gaps), 2 * n)
            even = len(set(gaps)) == 1
            if wait < half:
                violations += 1
            if wait == half:
                equalities += 1
                equal_and_even += even
            elif even:
                equal_and_even -= 1_000_000  # all gaps the same but no equality breaks the claim
    return cases, violations, equalities, equal_and_even == equalities


# ------------------------------------------------------------------ facts

def facts():
    out = []

    def add(key, value):
        out.append((key, value))

    ext = extended(OFFSET)
    blue, red = route(0), route(OFFSET)
    gaps = hour_gaps(OFFSET)
    short, long_ = min(gaps), max(gaps)
    mean_gap = Q(sum(gaps), len(gaps))

    # 01, 02: the stop, the sign, the gaps, five in six
    add("F-T1.routes", 2)
    add("F-T1.period", PERIOD)
    add("F-T1.offset", OFFSET)
    add("F-T1.hour", HOUR)
    add("F-T1.cycles_per_hour", HOUR // PERIOD)
    add("F-T1.blue_times", csv(blue))
    add("F-T1.red_times", csv(red))
    add("F-T1.buses_per_hour", len(bus_times(OFFSET)))
    add("F-T1.hour_gaps", csv(gaps))
    add("F-T1.gaps", csv(gaps[:2]))
    add("F-T1.short", short)
    add("F-T1.long", long_)
    add("F-T1.short_count", gaps.count(short))
    add("F-T1.long_count", gaps.count(long_))
    add("F-T1.mean_gap", fmt(mean_gap))
    add("F-T1.half_mean_gap", fmt(mean_gap / 2))
    add("F-T1.long_share_of_gaps", fmt(Q(gaps.count(long_), len(gaps))))
    in_long = time_average(lambda t, e: 1 if gap_around(t, e) == long_ else 0, OFFSET)
    add("F-T1.long_minutes", fmt(in_long * HOUR))
    add("F-T1.long_time_share", fmt(in_long))

    # 01, 04: the average wait, and by kind of gap
    wait = time_average(wait_at, OFFSET)
    add("F-W1.mean_wait", fmt(wait))
    add("F-W1.mean_wait_clock", clock(wait))
    add("F-W1.extra_clock", clock(wait - mean_gap / 2))

    def mean_wait_in(kind):
        ms = [t for t in moments() if gap_around(t, ext) == kind]
        return sum((wait_at(t, ext) for t in ms), Q(0)) / len(ms)

    short_wait, long_wait = mean_wait_in(short), mean_wait_in(long_)
    add("F-W1.short_mean_wait", fmt(short_wait))
    add("F-W1.long_mean_wait", fmt(long_wait))
    add("F-W1.naive_mean_of_waits", fmt((short_wait + long_wait) / 2))
    # 12_5: the train's allowance, the longest wait; turning up just after a
    # bus leaves, the wait approaches it, so take the ceiling of the seconds' worst
    allowance = math.ceil(max(wait_at(t, ext) for t in moments()))
    add("F-W1.allowance", allowance)
    add("F-W1.allowance_clock", clock(allowance))

    # 04: one person in the middle of every minute of one twelve-minute stretch
    people = middle_of_minute_people(OFFSET, 0, PERIOD)
    in_short = [w for t, w in people if t < OFFSET]
    in_long_ = [w for t, w in people if t > OFFSET]
    add("F-W2.per_minute", 1)
    add("F-W2.people", len(people))
    add("F-W2.short_people", len(in_short))
    add("F-W2.long_people", len(in_long_))
    add("F-W2.short_waits", csv(in_short))
    add("F-W2.long_waits_range", csv([max(in_long_), min(in_long_)]))
    add("F-W2.short_total", fmt(sum(in_short)))
    add("F-W2.long_total", fmt(sum(in_long_)))
    add("F-W2.total", fmt(sum(w for _, w in people)))
    add("F-W2.mean_wait", fmt(Q(sum(w for _, w in people)) / len(people)))
    add("F-W2.people_ratio", fmt(Q(len(in_long_), len(in_short))))

    # 05: the triangles
    short_area = area_by_seconds(OFFSET, 0, OFFSET)
    long_area = area_by_seconds(OFFSET, OFFSET, PERIOD)
    add("F-A1.short_area", fmt(short_area))
    add("F-A1.long_area", fmt(long_area))
    add("F-A1.total_area", fmt(short_area + long_area))
    add("F-A1.bar_width", 1)
    bars = sum((w * 1 for _, w in people), Q(0))
    add("F-A1.bars_equal_triangles", "true" if bars == short_area + long_area else "false")
    add("F-A1.width_ratio", fmt(Q(long_, short)))
    tallest = lambda a, b: max(wait_at(t, ext) for t in moments() if a <= t < b)
    # the tallest wait in a gap approaches the gap itself (just after a bus)
    add("F-A1.height_ratio", fmt(Q(math.ceil(tallest(OFFSET, PERIOD)), math.ceil(tallest(0, OFFSET)))))
    add("F-A1.area_ratio", fmt(long_area / short_area))
    up, down = triangle_tiles(long_ // short)
    add("F-A1.tiles.upright", up)
    add("F-A1.tiles.inverted", down)
    add("F-A1.mean_wait_ratio", fmt(long_wait / short_wait))
    # doubling a gap: the long gap's last five minutes are themselves a five-minute
    # triangle (the wait there runs from 5 down to 0), so compare ten with five
    add("F-A1.double", 2)
    half_gap_start = PERIOD - long_ // 2
    add("F-A1.double_area_ratio", fmt(long_area / area_by_seconds(OFFSET, half_gap_start, PERIOD)))

    # 01: your two mornings, just after the red bus and just before the blue
    add("F-M1.unlucky_at", fmt(Q(2 * OFFSET + 1, 2)))
    add("F-M1.unlucky_wait_clock", clock(wait_at(Q(2 * OFFSET + 1, 2), ext)))
    add("F-M1.lucky_at", fmt(Q(2 * PERIOD - 1, 2)))
    add("F-M1.lucky_wait_clock", clock(wait_at(Q(2 * PERIOD - 1, 2), ext)))

    # 02, 06: averaged over buses (a tag on each bus), averaged over people
    # (a passenger at the middle of every minute of the hour)
    tags = gap_before_each_bus(OFFSET)
    hour_people = [Q(2 * k + 1, 2) for k in range(HOUR)]
    add("F-S1.tags_per_gap", fmt(Q(len(tags), len(gaps))))
    add("F-S1.passengers", len(hour_people))
    add("F-S1.passengers_in_long", sum(1 for t in hour_people if gap_around(t, ext) == long_))
    add("F-S1.passengers_in_short", sum(1 for t in hour_people if gap_around(t, ext) == short))
    add("F-S1.bus_gap_tags", csv(tags))
    add("F-S1.bus_short_share", fmt(Q(tags.count(short), len(tags))))
    add("F-S1.bus_mean_gap", fmt(Q(sum(tags), len(tags))))
    person_short = time_average(lambda t, e: 1 if gap_around(t, e) == short else 0, OFFSET)
    add("F-S1.person_short_share", fmt(person_short))
    add("F-S1.person_long_share", fmt(1 - person_short))
    person_gap = time_average(gap_around, OFFSET)
    add("F-S1.person_gap", fmt(person_gap))
    add("F-S1.person_gap_clock", clock(person_gap))
    add("F-S1.averages", 2)
    within = [wait_at(t, ext) / gap_around(t, ext) for t in moments()]
    add("F-S1.wait_share_of_gap", fmt(sum(within, Q(0)) / len(within)))
    add("F-S1.half_person_gap_clock", clock(person_gap / 2))
    age = time_average(since_last, OFFSET)
    add("F-S1.age", fmt(age))
    add("F-S1.age_clock", clock(age))
    add("F-S1.age_plus_wait_clock", clock(age + wait))

    # 03: stretch one gap. Slide the red bus from six minutes after the blue down
    # to none; "the long gap" is the one from each red bus to the next blue bus.
    # Buses: the share that end that gap (the blue buses), from the bus list.
    # Passengers: one at the middle of every minute of the hour, by position.
    def after_red(t, d):
        return (t % PERIOD) >= d

    shares, twelfths, bus_shares, bus_counts, bus_means = [], [], [], [], []
    for d in range(EVEN_OFFSET, -1, -1):
        in_gap = sum(1 for t in hour_people if after_red(t, d))
        shares.append(Q(in_gap, len(hour_people)))
        twelfths.append(Q(in_gap * PERIOD, len(hour_people)))
        g = hour_gaps(d)
        bus_counts.append(len(bus_times(d)))
        bus_means.append(Q(sum(g), len(g)))
        if d > 0:  # with the buses together the blue and red bus share a moment
            bus_shares.append(Q(len(route(0)), len(bus_times(d))))
    add("F-G1.offsets", csv(range(EVEN_OFFSET, -1, -1)))
    add("F-G1.passenger_shares", csv(shares))
    add("F-G1.passengers_of_12", csv(twelfths))
    add("F-G1.spoken_of_12", csv(twelfths[1:5]))
    add("F-G1.share.d6", fmt(shares[0]))
    add("F-G1.share.d2", fmt(shares[4]))
    add("F-G1.share.d0", fmt(shares[-1]))
    add("F-G1.even_people_per_gap", fmt(Q(len(hour_people), len(hour_gaps(EVEN_OFFSET)))))
    add("F-G1.bus_share_every_offset", csv(sorted(set(bus_shares))))
    add("F-G1.buses_every_offset", csv(sorted(set(bus_counts))))
    add("F-G1.mean_gap_every_offset", csv(sorted(set(bus_means))))

    # 03, 07: even the gaps out; slide the red bus
    even_gaps = hour_gaps(EVEN_OFFSET)
    add("F-O1.even_offset", EVEN_OFFSET)
    add("F-O1.even_gap", csv(sorted(set(even_gaps))))
    even_wait = time_average(wait_at, EVEN_OFFSET)
    add("F-O1.even_wait", fmt(even_wait))
    add("F-O1.even_wait_clock", clock(even_wait))
    by_offset = [time_average(wait_at, d) for d in range(PERIOD + 1)]
    add("F-O1.wait_clocks", ",".join(clock(w) for w in by_offset))
    add("F-O1.wait.d0", clock(by_offset[0]))
    add("F-O1.wait.d2", clock(by_offset[2]))
    add("F-O1.wait.d6", clock(by_offset[6]))
    together = [b - a for a, b in zip(sorted(set(bus_times(0))), sorted(set(bus_times(0)))[1:])]
    add("F-O1.together_period", csv(sorted(set(together))))
    add("F-O1.min_wait", fmt(min(by_offset)))
    add("F-O1.max_wait", fmt(max(by_offset)))
    add("F-O1.best_offset", by_offset.index(min(by_offset)))
    # 07_5: the slider between whole minutes, every tenth of a minute
    tenths = [slider_wait(k) for k in range(10 * PERIOD + 1)]
    add("F-O1.tenth_offsets", len(tenths))
    add("F-O1.whole_second_tenths", sum(1 for w in tenths if (w * 60).denominator == 1))
    add("F-O1.between_offset", fmt(Q(25, 10)))
    add("F-O1.between_wait", fmt(tenths[25]))
    add("F-O1.between_readout", readout(tenths[25]))

    # 08: the formula, with the example's numbers
    squares = [g * g for g in gaps[:2]]
    mean_square = Q(sum(g * g for g in gaps), len(gaps))
    add("F-F1.area_factor", fmt(short_area / (short * short)))
    add("F-F1.squares", csv(squares))
    add("F-F1.mean_square", fmt(mean_square))
    add("F-F1.mean_gap_squared", fmt(mean_gap * mean_gap))
    spread = sum(((g - mean_gap) ** 2 for g in gaps), Q(0)) / len(gaps)
    add("F-F1.spread", fmt(spread))
    add("F-F1.distance", csv(sorted({abs(g - mean_gap) for g in gaps})))
    add("F-F1.penalty", fmt(spread / (2 * mean_gap)))
    add("F-F1.penalty_clock", clock(spread / (2 * mean_gap)))
    add("F-F1.cv2", fmt(spread / mean_gap**2))
    add("F-F1.wait_by_formula", fmt(mean_gap / 2 + spread / (2 * mean_gap)))

    # 08: never less than half the average gap; equal only when even
    cases, violations, equalities, iff = sweep()
    add("F-X1.max_gap", SWEEP_MAX_GAP)
    add("F-X1.max_count", SWEEP_MAX_COUNT)
    add("F-X1.cases", cases)
    add("F-X1.violations", violations)
    add("F-X1.equalities", equalities)
    add("F-X1.equal_iff_even", "true" if iff else "false")
    add("F-X1.bound_factor", "1/2")
    # 04's people: the middle of each minute gives the exact average for every gap length
    ok = all(sum(Q(2 * k + 1, 2) for k in range(g)) == Q(g * g, 2) for g in range(1, SWEEP_MAX_GAP + 1))
    add("F-X2.midpoint_exact", "true" if ok else "false")
    add("F-X2.checked", SWEEP_MAX_GAP)

    # 09: completely random buses (exponential gaps), exact moments k! mu^k
    mu = RANDOM_MEAN
    m1, m2 = math.factorial(1) * mu, math.factorial(2) * mu**2
    r_spread = m2 - m1**2
    add("F-R1.mean_gap", fmt(m1))
    add("F-R1.spread", fmt(r_spread))
    add("F-R1.half_mean", fmt(m1 / 2))
    add("F-R1.twice_mean", fmt(2 * m1))
    add("F-R1.mean_wait", fmt(m2 / (2 * m1)))
    add("F-R1.mean_age", fmt(m2 / (2 * m1)))
    add("F-R1.person_gap", fmt(m2 / m1))
    add("F-R1.person_gap_ratio", fmt(m2 / m1 / m1))
    add("F-R1.wait_over_mean", fmt(m2 / (2 * m1) / m1))
    # 09: more spread than random buses: three buses at once every eighteen minutes
    clumped = CLUMPED_GAPS
    add("F-R2.gaps", csv(clumped))
    add("F-R2.mean_gap", fmt(Q(sum(clumped), len(clumped))))
    clumped_wait = cycle_average_wait(clumped)
    add("F-R2.mean_wait", fmt(clumped_wait))
    add("F-R2.exceeds_random", "true" if clumped_wait > m2 / (2 * m1) else "false")

    # 10: two classes, by student; the profiler; Terada
    students = [size for size in CLASS_SIZES for _ in range(size)]
    add("F-C1.classes", len(CLASS_SIZES))
    add("F-C1.sizes", csv(CLASS_SIZES))
    add("F-C1.school_mean", fmt(Q(sum(CLASS_SIZES), len(CLASS_SIZES))))
    add("F-C1.students", len(students))
    add("F-C1.big_class_students", students.count(max(CLASS_SIZES)))
    add("F-C1.student_mean", fmt(Q(sum(students), len(students))))
    add("F-C1.year", FELD_GROFMAN_YEAR)
    calls = [LONG_CALL_MS] + [SHORT_CALL_MS] * SHORT_CALLS
    # sample the middle of every tenth of a millisecond of the calls laid end to end
    owner = [i for i, c in enumerate(calls) for _ in range(c * 10)]
    add("F-P1.long_ms", LONG_CALL_MS)
    add("F-P1.short_ms", SHORT_CALL_MS)
    add("F-P1.short_calls", SHORT_CALLS)
    add("F-P1.long_sample_share", fmt(Q(owner.count(0), len(owner))))
    add("F-P1.long_call_share", fmt(Q(1, len(calls))))
    add("F-H1.year", TERADA_YEAR)
    add("F-H1.over_a_century", "true" if THIS_YEAR - TERADA_YEAR > 100 else "false")

    # 11: someone who reads the timetable and turns up EARLY before each bus
    reader = {wait_at(Q(b - EARLY), ext) for b in bus_times(OFFSET)}
    add("F-K1.early", EARLY)
    add("F-K1.wait", csv(sorted(reader)))
    # 11: every passenger plans: ready at their dot, each stands EARLY before
    # the first bus they can make with EARLY to spare (nobody is shared out)
    plan = planners(OFFSET)
    at_stop = [arrive for _, arrive, _ in plan]
    stacks = Counter(bus % HOUR for _, _, bus in plan)
    add("F-K2.passengers", len(plan))
    add("F-K2.per_blue", csv(sorted({stacks[b] for b in blue})))
    add("F-K2.per_red", csv(sorted({stacks[b] for b in red})))
    add("F-K2.never_early", "true" if all(arrive >= ready for ready, arrive, _ in plan) else "false")
    add("F-K2.waits", csv(sorted({wait_at(t, ext) for t in at_stop})))
    add("F-K2.mean_wait_clock", clock(sum((wait_at(t, ext) for t in at_stop), Q(0)) / len(at_stop)))
    add("F-K2.in_long_share", fmt(Q(sum(1 for t in at_stop if gap_around(t, ext) == long_), len(at_stop))))
    add("F-K2.in_short_share", fmt(Q(sum(1 for t in at_stop if gap_around(t, ext) == short), len(at_stop))))
    add("F-K2.stand_at", csv(sorted({t % PERIOD for t in at_stop})))
    add("F-K2.wait_share_of_gap", fmt(sum((Q(wait_at(t, ext), gap_around(t, ext)) for t in at_stop), Q(0)) / len(at_stop)))

    # 12: the transfer puzzle, red bus four minutes after the blue
    p_gaps = hour_gaps(PUZZLE_OFFSET)
    p_short = area_by_seconds(PUZZLE_OFFSET, 0, PUZZLE_OFFSET)
    p_long = area_by_seconds(PUZZLE_OFFSET, PUZZLE_OFFSET, PERIOD)
    p_wait = time_average(wait_at, PUZZLE_OFFSET)
    add("F-Q1.offset", PUZZLE_OFFSET)
    add("F-Q1.gaps", csv(p_gaps[:2]))
    add("F-Q1.short", min(p_gaps))
    add("F-Q1.long", max(p_gaps))
    add("F-Q1.short_area", fmt(p_short))
    add("F-Q1.long_area", fmt(p_long))
    add("F-Q1.total_area", fmt(p_short + p_long))
    add("F-Q1.mean_wait", fmt(p_wait))
    add("F-Q1.mean_wait_clock", clock(p_wait))
    add("F-Q1.passenger_long_share", fmt(Q(sum(1 for t in hour_people if after_red(t, PUZZLE_OFFSET)), len(hour_people))))
    return out


# ------------------------------------------------------------- self-test

def self_test():
    failures = []

    def check(name, cond):
        print(("ok    " if cond else "FAIL  ") + name)
        if not cond:
            failures.append(name)

    f = dict(facts())
    check("keys are unique", len(f) == len(facts()))

    # the example, against the closed form
    gaps = hour_gaps(OFFSET)
    check("time average equals sum(g^2)/(2 sum g) at offset 2", time_average(wait_at, OFFSET) == closed_form_wait(gaps))
    check("the example is 4:20", f["F-W1.mean_wait_clock"] == "4:20")
    check("people see an 8:40 gap", f["F-S1.person_gap_clock"] == "8:40")
    check("age and wait average the same", f["F-S1.age"] == f["F-W1.mean_wait"])

    # every offset: brute force against the closed form (d^2 + (12-d)^2)/24
    for d in range(PERIOD + 1):
        want = Q(d * d + (PERIOD - d) ** 2, 2 * PERIOD)
        check(f"offset {d}: brute {time_average(wait_at, d)} == closed {want}", time_average(wait_at, d) == want)

    # negative control 1: even gaps, the paradox vanishes
    check("even gaps: wait is exactly half the gap", time_average(wait_at, EVEN_OFFSET) == Q(EVEN_OFFSET, 2))
    check("even gaps: people's gap equals the buses' gap", time_average(gap_around, EVEN_OFFSET) == EVEN_OFFSET)

    # edge: both routes at once, the zero gaps count for buses but hold no waiting
    g0 = hour_gaps(0)
    check("together: buses' average gap is still 6", Q(sum(g0), len(g0)) == 6 and 0 in g0)
    check("together: the wait is 6, a bus every 12", time_average(wait_at, 0) == 6)

    # negative control 2: a timetable reader is not a random arrival
    ext = extended(OFFSET)
    reader = [wait_at(Q(b - EARLY), ext) for b in bus_times(OFFSET) if b - EARLY >= 0]
    check("arriving one minute early waits one minute, whatever the gaps", set(reader) == {EARLY})

    # negative control 3: the people's grid matters; at the start of each minute,
    # two of them stand at a bus as it leaves, and the average is not the random one
    starts = [wait_at(Q(k), ext) for k in range(PERIOD)]
    check("people at the start of each minute do not average 13/3", Q(sum(starts), PERIOD) != Q(13, 3))

    # the sweep's claim, and that it is not vacuous
    cases, violations, equalities, iff = sweep()
    check(f"sweep: {cases} cases, no violations", violations == 0 and cases > 0)
    check("sweep: equality exactly when every gap is the same", iff and equalities == SWEEP_MAX_GAP * SWEEP_MAX_COUNT)
    check("sweep: 2,375 counts multisets, not sets (1,091) or ordered lists (30,936)",
          (cases, sum(1 for n in range(1, SWEEP_MAX_COUNT + 1) for c in itertools.combinations(range(SWEEP_MAX_GAP + 1), n) if sum(c)),
           sum(1 for n in range(1, SWEEP_MAX_COUNT + 1) for c in itertools.product(range(SWEEP_MAX_GAP + 1), repeat=n) if sum(c)))
          == (2375, 1091, 30936))
    check("a strictly uneven choice exceeds half", closed_form_wait([1, 2]) > Q(3, 4))

    # random integer timetables: brute force over the timeline vs closed form
    rng = random.Random(20261024)
    for trial in range(25):
        n = rng.randint(1, 6)
        gs = [rng.randint(1, 15) for _ in range(n)]
        check(f"random timetable {trial} {gs}: brute == closed", cycle_average_wait(gs) == closed_form_wait(gs))
    check("the cycle average reproduces the hour's 13/3", cycle_average_wait([OFFSET, PERIOD - OFFSET]) == Q(13, 3))
    check("clumped 0,0,18: brute == closed == 9", cycle_average_wait(CLUMPED_GAPS) == closed_form_wait(CLUMPED_GAPS) == 9)

    # completely random buses: the exact moments, and a seeded simulation of
    # random arrivals against exponential gaps (a measurement, not a fact)
    lam = 1 / float(RANDOM_MEAN)
    step = 0.001
    second_moment = sum((k * step) ** 2 * lam * math.exp(-lam * k * step) * step for k in range(1, 200_000))
    check(f"exponential E[G^2] by quadrature {second_moment:.4f} ~ 72", abs(second_moment - 72) < 0.05)
    sim = random.Random(7)
    t, buses = 0.0, [0.0]
    while t < 600_000:
        t += sim.expovariate(lam)
        buses.append(t)
    waits = []
    for _ in range(200_000):
        a = sim.uniform(0, buses[-1] - 100)
        waits.append(buses[bisect_right(buses, a)] - a)
    est = sum(waits) / len(waits)
    check(f"simulated random buses: mean wait {est:.3f} ~ 6 (200,000 arrivals)", abs(est - 6) < 0.1)

    # stretch one gap: the passengers' share is the gap's length over the period,
    # while the buses' share and the sign's average gap never move
    check("stretch: passenger share is (12 - d)/12", f["F-G1.passenger_shares"] == csv(Q(PERIOD - d, PERIOD) for d in range(EVEN_OFFSET, -1, -1)))
    check("stretch: buses' share 1/2, ten buses, mean gap 6 at every offset",
          (f["F-G1.bus_share_every_offset"], f["F-G1.buses_every_offset"], f["F-G1.mean_gap_every_offset"]) == ("1/2", "10", "6"))
    check("stretch: at offset 2 the dots' share is the 5/6 of the hour", f["F-G1.share.d2"] == f["F-T1.long_time_share"])

    # negative control 4: when every passenger plans, five in six still stand
    # in a long gap, but nobody is spread through a gap, so the triangles no
    # longer give their wait
    check("planners: every wait is one minute", f["F-K2.waits"] == "1")
    check("planners: ten before each blue bus, two before each red", (f["F-K2.per_blue"], f["F-K2.per_red"]) == ("10", "2"))
    check("planners: nobody turns up before they're ready", f["F-K2.never_early"] == "true")
    plan = planners(OFFSET)
    check("planners: ready at 1:30 and 11:30, too late to be a minute early, they stand at :11 and :13",
          (plan[1][1], plan[11][1]) == (11, 13))
    check("planners: still 5/6 in long gaps, the time share", f["F-K2.in_long_share"] == f["F-T1.long_time_share"] == "5/6")
    check("planners: the gap still ahead averages 1/6 of it, not the random 1/2",
          (f["F-K2.wait_share_of_gap"], f["F-S1.wait_share_of_gap"]) == ("1/6", "1/2"))
    check("planners: the same long-gap share, yet 1:00 instead of 4:20", f["F-K2.mean_wait_clock"] == "1:00" != f["F-W1.mean_wait_clock"])

    # the train: the longest wait is the long gap, well above the average
    check("train: allow 10:00, the long gap; the average is 4:20",
          (f["F-W1.allowance_clock"], f["F-W1.allowance"]) == ("10:00", f["F-T1.long"]) and Q(f["F-W1.mean_wait"]) < 10)

    # the slider between whole minutes: the half-second count agrees with the
    # closed form at every tenth; the readout rounds halves up, only there
    off = [k for k in range(10 * PERIOD + 1)
           if slider_wait(k) != (Q(k, 10) ** 2 + (PERIOD - Q(k, 10)) ** 2) / (2 * PERIOD)]
    check(f"slider: every tenth agrees with (d^2 + (12 - d)^2)/24 (disagreeing: {off})", off == [])
    check("slider: whole minutes agree with the timetable sweep", [slider_wait(10 * d) for d in range(PERIOD + 1)] == [time_average(wait_at, d) for d in range(PERIOD + 1)])
    check("slider: 13 of 121 tenths are whole seconds; 2.5 gives 193/48 = 4:01.25 -> 4:01",
          (f["F-O1.whole_second_tenths"], f["F-O1.between_wait"], f["F-O1.between_readout"]) == (13, "193/48", "4:01"))
    check("readout: 241.5 s rounds up to 4:02, 59.5 s to 1:00, whole seconds unchanged",
          (readout(Q(483, 120)), readout(Q(119, 120)), readout(Q(13, 3))) == ("4:02", "1:00", "4:20"))

    # the transfer puzzle against the closed form and the offset sweep
    check("puzzle: 8 + 32 = 40 per 12 minutes, 3:20", (f["F-Q1.total_area"], f["F-Q1.mean_wait_clock"]) == ("40", "3:20"))
    check("puzzle: agrees with the offset sweep at d = 4", f["F-Q1.mean_wait_clock"] == f["F-O1.wait_clocks"].split(",")[PUZZLE_OFFSET])
    check("puzzle: the gaps are the shortest 4 and the longest 8, the stretch's two",
          (f["F-Q1.short"], f["F-Q1.long"]) == (4, 8) and f["F-Q1.gaps"] == f"{f['F-Q1.short']},{f['F-Q1.long']}")

    # the profiler: as many samples in one 10 ms call as in five 2 ms calls
    check("profiler shares", f["F-P1.long_sample_share"] == "1/2" and f["F-P1.long_call_share"] == "1/6")

    # clock refuses a time that is not whole seconds
    try:
        clock(Q(1, 7))
        check("clock refuses 1/7 min", False)
    except ValueError:
        check("clock refuses 1/7 min", True)

    print(f"{len(failures)} failure(s)")
    return 1 if failures else 0


def main(argv):
    if "--self-test" in argv:
        return self_test()
    for key, value in facts():
        print(f"{key}\t{value}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
