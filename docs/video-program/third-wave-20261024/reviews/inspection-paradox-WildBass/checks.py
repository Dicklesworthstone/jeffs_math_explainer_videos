#!/usr/bin/env python3
"""Independent checks for the inspection_paradox review (WildBass).

Written for the review of Bead mx-video-program-program-87do.13.2. Nothing
here imports the author's oracle or model: every number is recomputed from
the timetable with exact fractions. The oracle is only run as a separate
process at the end, and its printed facts are compared with these.

Usage (from the repo root):

    python3 -I -B docs/video-program/third-wave-20261024/reviews/inspection-paradox-WildBass/checks.py \
        narration_lab/third-wave-20261024/root/intake-0550

Exit status 0 means every check printed "ok". A "finding" line is a
reviewed claim that this script shows to depend on an unstated assumption;
it does not fail the run.
"""

import hashlib
import json
import math
import pathlib
import random
import subprocess
import sys
from fractions import Fraction as F
from itertools import combinations, combinations_with_replacement

PERIOD = 12
HOUR = 60
OFFSET = 2
failures = 0


def check(name, ok, detail=""):
    global failures
    if not ok:
        failures += 1
    print(f"{'ok  ' if ok else 'FAIL'} {name}{': ' + detail if detail else ''}")


def finding(name, detail):
    print(f"finding {name}: {detail}")


def clock(minutes):
    s = minutes * 60
    if s.denominator != 1:
        return None
    s = int(s)
    return f"{s // 60}:{s % 60:02d}"


# ------------------------------------------------------------- the timetable

def buses(d):
    """Both routes in one hour, blue at 0, 12, ... and red d minutes later."""
    blue = [k * PERIOD for k in range(HOUR // PERIOD)]
    red = [(b + d) % HOUR for b in blue]
    return sorted(blue + red)


def gaps(d):
    """The gap after each bus; the next hour's first bus closes the last."""
    t = buses(d)
    return [b - a for a, b in zip(t, t[1:] + [t[0] + HOUR])]


def next_bus(d, t):
    """The first bus strictly after moment t (any real t; the hour repeats)."""
    base = math.floor(t / HOUR) * HOUR
    cands = [base + b + k * HOUR for k in (0, 1) for b in buses(d)]
    return min(c for c in cands if c > t)


def last_bus(d, t):
    base = math.floor(t / HOUR) * HOUR
    cands = [base + b + k * HOUR for k in (-1, 0) for b in buses(d)]
    return max(c for c in cands if c <= t)


def wait(d, t):
    return next_bus(d, t) - t


def gap_of(d, t):
    return next_bus(d, t) - last_bus(d, t)


def time_average_wait(gs):
    return F(sum(g * g for g in gs), 2 * sum(gs))


# ---------------------------------------------------------------- 0. inputs

def manifest(intake):
    data = json.loads((intake / "inputs.json").read_text())
    topic = data["topics"]["inspection_paradox"]
    snap = intake / "inspection_paradox"
    bad = 0
    listed = set()
    for f in topic["files"]:
        p = snap / f["path"]
        listed.add(f["path"])
        b = p.read_bytes()
        if hashlib.sha256(b).hexdigest() != f["sha256"] or len(b) != f["bytes"]:
            bad += 1
            print(f"     mismatch {f['path']}")
    extra = sorted(str(p.relative_to(snap)) for p in snap.rglob("*")
                   if p.is_file() and str(p.relative_to(snap)) not in listed
                   and "__pycache__" not in p.parts)
    check(f"snapshot: {len(listed)} files match inputs.json (captured {data['captured_at']})",
          bad == 0, f"{bad} mismatched")
    check("snapshot: no unlisted files", not extra, ", ".join(extra))
    return snap


# ------------------------------------------- 1. counting buses vs moments

def sampling():
    t = buses(OFFSET)
    gs = gaps(OFFSET)
    check("timetable: 10 buses, gaps alternate 2 and 10", len(t) == 10 and sorted(set(gs)) == [2, 10])
    # tags: the gap before each bus (the bus at :00 follows :50 of the hour before)
    tags = [b - last_bus(OFFSET, b - F(1, 1000)) for b in t]
    check("tags (02_2): 10,2,10,2,... and average 6 over buses",
          tags == [10, 2] * 5 and F(sum(tags), len(tags)) == 6, ",".join(map(str, tags)))
    check("bus sampling: half the tags say ten", F(tags.count(10), len(tags)) == F(1, 2))
    # moments: the share of the hour inside long gaps
    long_time = sum(g for g in gs if g == 10)
    check("time sampling (02_4, 02_6): 50 of 60 minutes are in long gaps, 5/6",
          F(long_time, HOUR) == F(5, 6))
    dots = [F(2 * k + 1, 2) for k in range(HOUR)]
    in_long = sum(1 for x in dots if gap_of(OFFSET, x) == 10)
    check("midpoint dots (02_5): 50 of 60 land in long gaps", in_long == 50)
    person_gap = F(sum(gap_of(OFFSET, x) for x in dots), len(dots))
    check("passengers' gap (06_4): 26/3 = 8:40, length-biased mean sum g^2 / sum g",
          person_gap == F(26, 3) == F(sum(g * g for g in gs), sum(gs)), clock(person_gap))
    # a uniform moment inside a gap is uniform, so forward and backward halves agree
    ages = F(sum(x - last_bus(OFFSET, x) for x in dots), len(dots))
    check("since the last bus (06_7): also 4:20", ages == F(13, 3), clock(ages))


# --------------------------------- 2. midpoints vs the continuous integral

def integrals():
    gs = gaps(OFFSET)
    exact = F(sum(g * g for g in gs), 2)  # area of the triangles, min^2 per hour
    check("area (05_5): the hour's triangles hold 5 x (2 + 50) = 260 min^2", exact == 260)
    # numeric quadrature of the sawtooth, independent of the triangle picture
    n = 60_000
    riemann = sum(float(wait(OFFSET, F(k, 1000))) for k in range(n)) / 1000
    check("left Riemann sum, 1/1000 min steps, approaches 260", abs(riemann - 260) < 0.05,
          f"{riemann:.4f}")
    mid = sum(wait(OFFSET, F(2 * k + 1, 2)) for k in range(HOUR))
    start = sum(wait(OFFSET, F(k)) for k in range(HOUR))
    end = sum(wait(OFFSET, F(k) + 1 - F(1, 10**9)) for k in range(HOUR))
    check("midpoint passengers (04_6, K02): 260 passenger-minutes, exactly the area", mid == exact)
    check("negative control: start-of-minute passengers give 290, 29/6 = 4:50",
          start == 290 and clock(start / HOUR) == "4:50")
    check("negative control: end-of-minute passengers give about 230, 23/6 = 3:50",
          abs(float(end) - 230) < 1e-6)
    # finer midpoints are still exact because the wait is linear inside every gap
    for m in (2, 3, 7):
        s = sum(wait(OFFSET, F(2 * k + 1, 2 * m)) for k in range(HOUR * m)) / m
        check(f"midpoints of 1/{m} min also give exactly 260", s == exact)
    # units: 1 passenger per minute -> passenger-minutes / passengers = minutes
    check("units (08_2, 11_3): 260 passenger-min / 60 passengers = 52 min^2 / 12 min = 13/3 min",
          F(mid, HOUR) == F(52, 12) == F(13, 3))
    # 05_4: each minute's bar equals its trapezoid, with two equal corners of 1/8
    g = 10
    for k in range(g):
        bar = F(2 * (g - k) - 1, 2)
        trapezoid = F((g - k) + (g - k - 1), 2)
        assert bar == trapezoid
    check("05_4: each bar = its trapezoid; the two corners are 1/8 min^2 each", True)
    # 04 per-gap arithmetic
    short = [wait(OFFSET, F(1, 2)), wait(OFFSET, F(3, 2))]
    long_ = [wait(OFFSET, F(2 * k + 1, 2)) for k in range(2, 12)]
    check("04_3/04_6: short waits 3/2 and 1/2 sum 2; long 19/2 .. 1/2 sum 50; 52/12 = 4:20",
          sorted(short) == [F(1, 2), F(3, 2)] and sum(short) == 2 and sum(long_) == 50
          and long_[0] == F(19, 2) and long_[-1] == F(1, 2) and clock(F(52, 12)) == "4:20")
    # 05_7 tiling: a size-5 triangle holds 25 unit triangles, 15 up and 10 down
    for n in range(1, 13):
        up, down = n * (n + 1) // 2, n * (n - 1) // 2
        assert up + down == n * n
    check("05_7: tiling n(n+1)/2 + n(n-1)/2 = n^2 (n = 1..12); n = 5 gives 15 + 10", True)


# ------------------------------------------------------ 3. offsets and edges

def offsets():
    clocks = []
    for d in range(13):
        gs = gaps(d)
        w = time_average_wait(gs)
        direct = sum(wait(d, F(2 * k + 1, 2)) for k in range(HOUR)) / HOUR
        closed = F(d * d + (PERIOD - d) ** 2, 2 * PERIOD)
        assert w == direct == closed, d
        assert len(gs) == 10 and F(sum(gs), len(gs)) == 6, d
        clocks.append(clock(w))
    check("07_5/07_6: wait(d) = (d^2 + (12-d)^2)/24 for d = 0..12, by midpoints and by gaps",
          True, ",".join(clocks))
    check("07_6/07_7: 4:20 at d=2, 3:00 at d=6, 6:00 at d=0 and d=12",
          clocks[2] == "4:20" and clocks[6] == "3:00" and clocks[0] == clocks[12] == "6:00")
    gs0 = gaps(0)
    check("d = 0 (03_6, 07_6): zero gaps counted, average gap over buses still 6, wait 6",
          sorted(gs0) == [0] * 5 + [12] * 5 and time_average_wait(gs0) == 6)
    merged = [g for g in gs0 if g > 0]
    check("d = 0 with coincident buses merged: average gap 12, wait 6 = exactly half",
          F(sum(merged), len(merged)) == 12 and time_average_wait(merged) == 6)
    print("     note: Larson & Odoni's derivation assumes F_Y(0) = 0 (no zero gaps); the film's"
          " d = 0 edge counts them, which keeps the identity but makes 'the average gap' a counting choice")
    # 03: share of passengers in the red-to-blue gap, and of buses ending it
    for d in range(1, 7):
        dots = [F(2 * k + 1, 2) for k in range(HOUR)]
        share = F(sum(1 for x in dots if (x % PERIOD) >= d), HOUR)
        enders = sum(1 for b in buses(d) if b % PERIOD == 0)  # blue buses end red->blue gaps
        assert share == F(PERIOD - d, PERIOD) and F(enders, 10) == F(1, 2), d
    check("03_2..03_4: passengers' share (12-d)/12 (6,7,8,9,10 of 12), buses' share 1/2, d = 6..1", True)
    # the 07_5 slider is continuous: how many tenth-minute offsets give whole seconds?
    tenths = [F(k, 10) for k in range(0, 121)]
    whole = [d for d in tenths if clock(F(d * d + (PERIOD - d) ** 2, 2 * PERIOD)) is not None]
    finding("07_5 slider", f"of the 121 offsets d = 0.0, 0.1, ..., 12.0, only {len(whole)} give a"
            f" whole-second wait (e.g. d = 2.5 gives {float(F(25, 4) + F(361, 4)) / 24 * 60:.2f} s);"
            " model::offset_wait takes i64 and clock() refuses fractions, so a live sweep needs"
            " a rational offset and an explicit rounding path, or integer steps")


# ------------------------------------------------------ 4. the bound sweep

def bound_sweep():
    values = range(0, 13)
    cases = viol = eq = 0
    iff = True
    for size in range(1, 5):
        for gs in combinations_with_replacement(values, size):
            if sum(gs) == 0:
                continue
            cases += 1
            w = time_average_wait(gs)
            half = F(sum(gs), 2 * len(gs))
            viol += w < half
            equal = w == half
            eq += equal
            iff &= equal == (len(set(gs)) == 1)
    sets = sum(1 for size in range(1, 5) for c in combinations(values, size) if sum(c) > 0)
    tuples = sum(13 ** size for size in range(1, 5)) - 4
    check("08_8 sweep: 2,375 cases, 0 below half, 48 equalities, equal iff even",
          (cases, viol, eq, iff) == (2375, 0, 48, True), f"{cases}, {viol}, {eq}, {iff}")
    finding("08_8 card wording", f"2,375 counts multisets (repeats allowed, order ignored, not all"
            f" zero); true sets would be {sets} and ordered lists {tuples}")
    # Cauchy-Schwarz form: n * sum g^2 >= (sum g)^2, equality iff all equal
    check("bound identity: wait - half gap = Var/(2 E[G]) on the 2/10 timetable = 16/12 = 1:20",
          time_average_wait(gaps(OFFSET)) - 3 == F(16, 12) and clock(F(16, 12)) == "1:20")


# ------------------------------------------- 5. completely random buses

def poisson():
    mu = 6
    # exponential moments by numerical quadrature, not by k! mu^k
    def moment(k, steps=200_000, top=200.0):
        h = top / steps
        return sum(((i + 0.5) * h) ** k * math.exp(-(i + 0.5) * h / mu) / mu for i in range(steps)) * h
    m1, m2 = moment(1), moment(2)
    check("exponential, mean 6: E[G] = 6, E[G^2] = 72 (midpoint quadrature)",
          abs(m1 - 6) < 1e-6 and abs(m2 - 72) < 1e-4, f"{m1:.6f}, {m2:.4f}")
    check("09_4/09_6: wait E[G^2]/(2E[G]) = 6 = 3 + 36/12; your gap 12 (09_7)",
          abs(m2 / (2 * m1) - 6) < 1e-5 and abs(m2 / m1 - 12) < 1e-5)
    rng = random.Random(20261024)
    horizon = 2_000_000.0
    times, t = [], 0.0
    while t < horizon:
        t += rng.expovariate(1 / mu)
        times.append(t)
    import bisect
    waits, gaps_hit, cond = [], [], [0, 0, 0]
    for _ in range(200_000):
        x = rng.uniform(1000.0, horizon - 1000.0)
        i = bisect.bisect_right(times, x)
        w, a = times[i] - x, x - times[i - 1]
        waits.append(w)
        gaps_hit.append(w + a)
        if a > 6:
            cond[0] += 1
            cond[1] += w > 6
        cond[2] += w > 6
    mw, mg = sum(waits) / len(waits), sum(gaps_hit) / len(gaps_hit)
    p_all, p_old = cond[2] / len(waits), cond[1] / cond[0]
    check("seeded simulation (seed 20261024, 200,000 moments): mean wait near 6, gap near 12",
          abs(mw - 6) < 0.1 and abs(mg - 12) < 0.2, f"{mw:.3f}, {mg:.3f}")
    check("no memory (09_5): P(wait > 6 | last bus > 6 ago) matches P(wait > 6) = e^-1",
          abs(p_old - p_all) < 0.01 and abs(p_all - math.exp(-1)) < 0.01, f"{p_old:.4f} vs {p_all:.4f}")
    # 09_8: clumped 0, 0, 18 keeps the average gap 6 and waits 9
    check("09_8: gaps 0,0,18 -> average gap 6, wait 9 > 6",
          F(18, 3) == 6 and time_average_wait([0, 0, 18]) == 9)
    # K07 scope: finite mean square matters. Pareto, alpha 1.5, mean 6: E[G^2] infinite.
    alpha, xm = 1.5, 2.0
    est = []
    rng = random.Random(7)
    s1 = s2 = 0.0
    n = 0
    for target in (10**3, 10**4, 10**5, 10**6):
        while n < target:
            g = xm / (1 - rng.random()) ** (1 / alpha)
            s1 += g
            s2 += g * g
            n += 1
        est.append(s2 / (2 * s1))
    finding("K07 finite-moment control", "Pareto gaps (alpha 1.5, mean 6, infinite mean square):"
            " the running wait estimate at n = 1e3..1e6 is " + ", ".join(f"{e:.1f}" for e in est)
            + " min; it has no limit, so K07's finite-mean-square condition is needed (correctly stated)")


# ----------------------------------------- 6. who the formula describes

def scope():
    # a commuter who turns up at the same moment every day, independent of the
    # (fixed) timetable, does not average 4:20
    fixed = wait(OFFSET, F(5, 2))
    finding("K07 vs K05/K09 wording", f"turning up at :02:30 every day is independent of a fixed"
            f" timetable, yet waits {clock(fixed)} every day; uniform turning-up waits"
            f" {clock(time_average_wait(gaps(OFFSET)))}. K05 and K09 say 'uniformly / spread evenly"
            " in time'; K07 says only 'independently of the buses'")
    # 11: the planners. The author's model: 60 passengers shared evenly, 6 per bus.
    t = buses(OFFSET)
    even = [b - 1 for b in t for _ in range(6)]
    even_long = F(sum(1 for x in even if gap_of(OFFSET, x) == 10), len(even))
    check("11_6 as drafted (even split): 6 per bus, half in a short gap, every wait 1",
          even_long == F(1, 2) and all(wait(OFFSET, x) == 1 for x in even))
    # Ready-time model: each of the 60 dots (one per minute, as in 02) is when
    # that passenger is ready; they plan to be at the stop 1 minute before the
    # first bus they can still make, and wait 1 minute for it.
    dots = [F(2 * k + 1, 2) for k in range(HOUR)]
    target = [next_bus(OFFSET, x + 1 - F(1, 10**9)) for x in dots]  # first bus b with b - 1 >= x
    arrive = [b - 1 for b in target]
    per_bus = {}
    for b in target:
        per_bus[b % HOUR] = per_bus.get(b % HOUR, 0) + 1
    counts = [per_bus.get(b, 0) for b in t]
    ready_long = F(sum(1 for x in arrive if gap_of(OFFSET, x) == 10), len(arrive))
    check("ready-time planners: every wait 1 minute", all(wait(OFFSET, x) == 1 for x in arrive))
    check("ready-time planners: 2 before each red bus, 10 before each blue bus",
          counts == [10, 2] * 5, ",".join(map(str, counts)))
    finding("11_6 even split", f"under the ready-time model the share standing in long gaps stays"
            f" {ready_long} (not 1/2); the even split needs at least"
            f" {sum(max(0, 6 - c) for c in counts)} of 60 passengers to pass up a bus they could"
            " make, which the draft does not state")
    # 10: classes and the profiler
    check("10_4: (2*2 + 10*10)/12 = 104/12 = 26/3", F(2 * 2 + 10 * 10, 12) == F(26, 3))
    total = 10 + 5 * 2
    check("10_6: 10 ms call 1/2 of samples, 1/6 of calls; 2 ms calls 1/2 and 5/6",
          F(10, total) == F(1, 2) and F(1, 6) == F(1, 1 + 5) and F(5 * 2, total) == F(1, 2))
    # 12: the transfer puzzle
    p = gaps(4)
    check("12_3/12_4: gaps 4 and 8, triangles 8 and 32, 40/12 = 3:20, passengers' long share 2/3",
          sorted(set(p)) == [4, 8] and time_average_wait(p) == F(10, 3)
          and clock(F(10, 3)) == "3:20" and F(8, 12) == F(2, 3))


# ---------------------------------------------- 7. the opening's two bars

def opening_bars():
    line_x0, line_x1 = -6.0, 6.0          # draw.rs LINE_X0, LINE_X1
    x_of = lambda m: line_x0 + (line_x1 - line_x0) * m / HOUR
    unit = x_of(1) - x_of(0)
    lucky_at, unlucky_at = F(23, 2), F(5, 2)   # model::lucky_morning, unlucky_morning
    lw, uw = wait(OFFSET, lucky_at), wait(OFFSET, unlucky_at)
    check("01_4: lucky 0:30 and unlucky 9:30 both end at the :12 bus",
          clock(lw) == "0:30" and clock(uw) == "9:30"
          and lucky_at + lw == unlucky_at + uw == 12)
    lucky_rt, unlucky_rt = 1.0, 2.6        # the_stop.rs LUCKY_RT, UNLUCKY_RT
    r_l, r_u = float(lw) / lucky_rt, float(uw) / unlucky_rt
    finding("01_4 bar rates", f"bars are to scale ({unit:.2f} units/min: {float(lw) * unit:.2f} and"
            f" {float(uw) * unit:.2f} wide) but drain at {r_l:.2f} and {r_u:.2f} wait-minutes per"
            f" second, {r_u / r_l:.1f}x apart; the storyboard says 'shrinks the same way'")
    print(f"     labels: 'wait 0:30' centred at x = {x_of(float(lucky_at)):.2f}, 'wait 9:30' at"
          f" x = {x_of(float(unlucky_at)):.2f}; the unlucky bar spans x = {x_of(float(unlucky_at)):.2f}"
          f" .. {x_of(12):.2f}, so the lucky label (not faded until both mornings end) sits under it")


def oracle_agrees(snap):
    out = subprocess.run([sys.executable, "-I", "-B", str(snap / "oracle.py")],
                         capture_output=True, text=True)
    if out.stderr:
        print(out.stderr, end="")
    facts = dict(line.split("\t", 1) for line in out.stdout.splitlines() if "\t" in line)
    mine = {
        "F-W1.mean_wait": "13/3", "F-W1.extra_clock": "1:20", "F-S1.person_gap": "26/3",
        "F-T1.long_time_share": "5/6", "F-M1.lucky_wait_clock": "0:30",
        "F-M1.unlucky_wait_clock": "9:30",
        "F-O1.wait_clocks": ",".join(clock(time_average_wait(gaps(d))) for d in range(13)),
        "F-X1.cases": "2375", "F-X1.violations": "0", "F-X1.equalities": "48",
        "F-R1.mean_wait": "6", "F-R1.person_gap": "12", "F-R2.mean_wait": "9",
        "F-C1.student_mean": "26/3", "F-Q1.mean_wait_clock": "3:20",
        "F-K2.in_long_share": "1/2", "F-K2.per_bus": "6",
    }
    diff = [k for k, v in mine.items() if facts.get(k) != v]
    check(f"oracle (exit {out.returncode}, {len(facts)} facts) agrees on {len(mine)} reviewed keys",
          out.returncode == 0 and not diff, ", ".join(f"{k}={facts.get(k)}" for k in diff))


def main():
    intake = pathlib.Path(sys.argv[1])
    snap = manifest(intake)
    sampling()
    integrals()
    offsets()
    bound_sweep()
    poisson()
    scope()
    opening_bars()
    oracle_agrees(snap)
    print(f"{failures} failure(s)")
    return 1 if failures else 0


if __name__ == "__main__":
    sys.exit(main())
