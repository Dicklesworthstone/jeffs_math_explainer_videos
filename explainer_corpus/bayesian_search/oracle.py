#!/usr/bin/env python3
"""Facts oracle for Video H, "How to Find a Lost Submarine".

Run from this folder (`./x bayesian_search facts-check` does):

    uv run --no-project --with numpy python3 -I -B oracle.py

stdout: one `KEY<TAB>VALUE` line per fact, the same keys `search facts`
prints (src/facts.rs), in the rows of docs/fact-sheet.md: F-M (the four-cell
sea), F-U (the rules for where to search next), F-R (the 1968 map), F-H (the
hook), F-K (Koopman's formula and the lanes), F-D (the drifting cloud), F-S,
F-P, F-A, F-C and F-N (cited), F-Q (the quizzes).

stderr: cross-checks (lines starting with "#") and a summary; exit 1 if any
cross-check failed.

How the values are obtained, independently of the Rust model:
  * the four-cell sea by counting dots, never by the update formula: the
    stories put dots in cells, a search puts out the dots it would have
    found, and a cell's chance is its share of the dots still lit;
  * "no order of searches does better" by walking every order of up to 8
    searches with that dot counting;
  * the average number of searches in exact fractions (the video sums floats);
  * the 1968 map from a second transcription of Richardson and Stone's
    Figure 2, typed cell by cell from the page image and written here as
    sparse rows; its check sum is the paper's 10,000 replications;
  * the model search over that map with a sorted list of (chance left,
    place) pairs, not the video's scan for the largest;
  * the lanes and the drifting cloud regenerated from the same seeds with
    NumPy's PCG64DXSM, the lanes counted on a full grid of swept points;
  * dates with the standard library's calendar;
  * CITED literals typed a second time from the sources listed in
    docs/fact-sheet.md.
"""
import datetime
import itertools
import math
import sys
from fractions import Fraction as F

import numpy as np

_EMITTED = {}
_FAILED = []
_CHECKS = [0]


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


def frac(x):
    """A fraction as the video prints it: n/d, or n when whole."""
    x = F(x)
    return str(x.numerator) if x.denominator == 1 else f"{x.numerator}/{x.denominator}"


def fixed(x, places):
    """x >= 0 with exactly `places` decimals, rounded half up."""
    x = F(x)
    assert x >= 0
    scale = 10**places
    scaled = (2 * x.numerator * scale + x.denominator) // (2 * x.denominator)
    if places == 0:
        return str(scaled)
    return f"{scaled // scale}.{scaled % scale:0{places}d}"


def percent(x, places):
    return fixed(F(x) * 100, places)


def join(items):
    return ",".join(str(x) for x in items)


# --------------------------------------------------------------------------
# F-M: the four-cell sea, by counting dots
# --------------------------------------------------------------------------
CELLS = "ABCD"
DOTS = 100
# each story: its weight in percent, and how its dots split over A, B, C, D
STORIES = [(60, [F(1, 2), F(1, 3), F(1, 6), F(0)]), (40, [F(1, 4)] * 4)]
# how many of the dots in a cell one search of it puts out: canyon, then flat
SEES = [F(1, 4), F(2, 3), F(2, 3), F(2, 3)]


def story_dots(weight, split):
    return [F(weight, 100) * DOTS * s for s in split]


def lit_dots():
    """Dots per cell before any search: every story's dots, added up."""
    dots = [F(0)] * 4
    for weight, split in STORIES:
        dots = [d + s for d, s in zip(dots, story_dots(weight, split))]
    return dots


def search(dots, cell):
    """One search of `cell` that comes back empty: (dots put out, dots after)."""
    out = dots[cell] * SEES[cell]
    after = list(dots)
    after[cell] -= out
    return out, after


def shares(dots):
    total = sum(dots)
    return [d / total for d in dots]


def finds(dots):
    """The chance a search of each cell finds the wreck: dots it would put
    out, as a share of the dots lit."""
    total = sum(dots)
    return [d * s / total for d, s in zip(dots, SEES)]


def choose(rule, dots, step, searched):
    if rule == "sweep":
        return step % 4
    if rule == "best":
        key = lambda i: (finds(dots)[i], -i)
    elif rule == "likeliest":
        key = lambda i: (dots[i], -i)
    else:  # likeliest_fresh
        key = lambda i: (dots[i], -searched[i], -i)
    return max(range(4), key=key)


def run(rule, steps):
    """[(cell, chance this search finds, chance found by now, shares after)]."""
    dots, searched, out = lit_dots(), [0] * 4, []
    for step in range(steps):
        cell = choose(rule, dots, step, searched)
        find = finds(dots)[cell]
        _, dots = search(dots, cell)
        searched[cell] += 1
        out.append((cell, find, 1 - sum(dots) / DOTS, shares(dots)))
    return out


def found_by(order):
    dots = lit_dots()
    for cell in order:
        _, dots = search(dots, cell)
    return 1 - sum(dots) / DOTS


def expected_searches(rule):
    """Sum over n >= 0 of the chance the wreck is unfound after n searches,
    with every choice made in exact fractions."""
    dots, searched, total, step = lit_dots(), [0] * 4, F(0), 0
    while sum(dots) / DOTS >= F(1, 10**13):
        total += sum(dots) / DOTS
        cell = choose(rule, dots, step, searched)
        _, dots = search(dots, cell)
        searched[cell] += 1
        step += 1
    return total


def small_sea():
    emit("F-M.dots", DOTS)
    for i, (weight, split) in enumerate(STORIES, 1):
        emit(f"F-M.story{i}.weight", weight)
        emit(f"F-M.story{i}.share", join(frac(s) for s in split))
        emit(f"F-M.story{i}.dots", join(frac(d) for d in story_dots(weight, split)))
    dots = lit_dots()
    check("the stories' dots are whole and add to 100", all(d.denominator == 1 for d in dots) and sum(dots) == DOTS)
    emit("F-M.prior.dots", join(frac(d) for d in dots))
    emit("F-M.prior.percent", join(percent(s, 0) for s in shares(dots)))
    emit("F-M.seeing", join(frac(s) for s in SEES))
    find = finds(dots)
    emit("F-M.find.fraction", join(frac(x) for x in find))
    emit("F-M.find.percent", join(percent(x, 1) for x in find))
    likeliest = max(range(4), key=lambda i: (dots[i], -i))
    first = max(range(4), key=lambda i: (find[i], -i))
    emit("F-M.likeliest.cell", CELLS[likeliest])
    emit("F-M.likeliest.find.percent", percent(find[likeliest], 0))
    emit("F-M.first.cell", CELLS[first])
    emit("F-M.first.find.percent", percent(find[first], 0))
    emit("F-M.first.over_likeliest", frac(find[first] / find[likeliest]))
    check("the likeliest cell is not the best first search", likeliest != first)

    out, after = search(dots, first)
    emit("F-M.empty1.dots_out", frac(out))
    emit("F-M.empty1.dots_left", frac(sum(after)))
    emit("F-M.empty1.cell_dots_left", frac(after[first]))
    emit("F-M.empty1.after.fraction", join(frac(s) for s in shares(after)))
    emit("F-M.empty1.after.percent", join(percent(s, 1) for s in shares(after)))
    rise = shares(after)[0] / shares(dots)[0]
    check("every unsearched cell rises by the same factor",
          all(shares(after)[i] / shares(dots)[i] == rise for i in range(4) if i != first))
    check("only the searched cell lost dots", all(after[i] == dots[i] for i in range(4) if i != first))
    emit("F-M.empty1.rise", frac(rise))
    emit("F-M.empty1.rise.decimal", fixed(rise, 2))

    find2 = finds(after)
    second = max(range(4), key=lambda i: (find2[i], -i))
    emit("F-M.second.find.fraction", join(frac(x) for x in find2))
    emit("F-M.second.find.percent", join(percent(x, 1) for x in find2))
    emit("F-M.second.cell", CELLS[second])
    check("the second search is not the cell with half the chance", shares(after)[second] < F(1, 2) == shares(after)[0])
    _, after2 = search(after, second)
    emit("F-M.empty2.after.percent", join(percent(s, 0) for s in shares(after2)))
    emit("F-M.empty2.found_by", frac(1 - sum(after2) / DOTS))
    find3 = finds(after2)
    emit("F-M.third.find.percent", join(percent(x, 1) for x in find3))
    emit("F-M.third.cell", CELLS[max(range(4), key=lambda i: (find3[i], -i))])

    # n searches of one flat cell: the dots left of 27 after each
    for n in (1, 2, 3):
        left = F(27)
        for _ in range(n):
            left -= left * SEES[1]
        seen = 1 - left / 27
        emit(f"F-M.seen_within.{n}", frac(seen))
        emit(f"F-M.seen_within.{n}.percent", percent(seen, 1))


def rules():
    runs = {}
    for rule in ("best", "likeliest", "likeliest_fresh", "sweep"):
        r = runs[rule] = run(rule, 12)
        emit(f"F-U.{rule}.order", "".join(CELLS[c] for c, *_ in r))
        emit(f"F-U.{rule}.found_by.percent", join(percent(x[2], 1) for x in r))
        emit(f"F-U.{rule}.expected_searches", fixed(expected_searches(rule), 2))
    for rule in ("likeliest", "likeliest_fresh", "sweep"):
        check(f"the rule is never behind {rule} in 12 searches",
              all(a[2] >= b[2] for a, b in zip(runs["best"], runs[rule])))
    best = []
    for n in range(1, 9):
        best.append(max(found_by(order) for order in itertools.product(range(4), repeat=n)))
    emit("F-U.brute.searches", 8)
    emit("F-U.brute.orders", 4**8)
    emit("F-U.brute.best", join(frac(b) for b in best))
    emit("F-U.brute.rule_is_best", all(b == r[2] for b, r in zip(best, runs["best"])))


# --------------------------------------------------------------------------
# F-R: the 1968 map (Richardson and Stone 1971, Figure 2, p. 145)
# --------------------------------------------------------------------------
# A second transcription: per printed row, the column of its first number and
# the numbers from there eastward (a 0 is a blank cell inside the run).
# Columns: three unlettered ones west of A (w3 w2 w1), A to O, two east (e1 e2).
COLS = ["w3", "w2", "w1"] + list("ABCDEFGHIJKLMNO") + ["e1", "e2"]
MAP_ROWS = [
    ("n2", "N", [5, 7, 1]),
    ("n1", "L", [3, 11, 14, 24, 6]),
    ("1", "K", [5, 26, 35, 22, 26, 9, 1]),
    ("2", "J", [18, 46, 74, 42, 18, 10, 4, 2]),
    ("3", "H", [8, 60, 140, 99, 45, 20, 4, 2, 1, 1]),
    ("4", "A", [2, 21, 137, 16, 7, 1, 20, 215, 239, 105, 30, 5, 3, 1, 1, 1]),
    ("5", "w1", [18, 40, 46, 747, 30, 1250, 205, 571, 277, 38, 5, 2, 1, 1, 1]),
    ("6", "w2", [14, 326, 3, 1, 28, 31, 63, 85, 62, 1, 8, 7, 10, 7, 3, 4]),
    ("7", "w3", [1, 359, 175, 174, 1096, 282, 245, 82, 71, 65, 35, 27, 9, 12, 6, 5, 4]),
    ("8", "w2", [24, 25, 42, 82, 297, 230, 129, 115, 61, 33, 14, 14, 10, 6, 2, 5, 1]),
    ("9", "w2", [17, 25, 20, 20, 20, 19, 55, 99, 46, 30, 14, 15, 3, 5, 1, 6]),
    ("10", "w2", [2, 13, 14, 25, 20, 24, 45, 34, 27, 19, 15, 5, 7, 5, 5, 1]),
    ("11", "A", [7, 13, 12, 9, 1, 3, 3, 11, 14, 5, 4, 3, 2, 1]),
    ("12", "G", [1, 4, 4, 10, 5, 4, 1]),
    ("s1", "I", [1, 3, 2]),
    ("s2", "J", [3, 2]),
]
MAP = {}
for _row, _start, _numbers in MAP_ROWS:
    for _k, _n in enumerate(_numbers):
        if _n:
            MAP[(_row, COLS[COLS.index(_start) + _k])] = _n
ROW_NAMES = [r for r, _, _ in MAP_ROWS]
GRID_ROWS = [str(i) for i in range(1, 13)]
GRID_COLS = list("ABCDEFGHIJKLMNO")


def cell(name):
    col = name.rstrip("0123456789")
    return MAP.get((name[len(col):], col), 0)


def model_search(rule, sees, looks):
    """[(row, col, chance found by now)]; every search comes back empty."""
    left = {k: F(v, 10000) for k, v in MAP.items()}
    reading = {(r, c): i for i, (r, c) in enumerate((r, c) for r in ROW_NAMES for c in COLS)}
    sweep = [(r, c) for r in GRID_ROWS for c in GRID_COLS]
    found, out = F(0), []
    for look in range(looks):
        if rule == "sweep":
            where = sweep[look % len(sweep)]
        else:
            where = sorted(left, key=lambda k: (-left[k], reading[k]))[0]
        w = left.get(where, F(0))
        found += w * sees
        left[where] = w * (1 - sees)
        out.append((where[0], where[1], found))
    return out


def map_1968():
    total = sum(MAP.values())
    check("Figure 2's numbers add to the paper's 10,000 replications", total == 10000)
    emit("F-R.replications", 10000)
    emit("F-R.total", total)
    emit("F-R.printed_cells", len(MAP))
    emit("F-R.rows", join(sum(v for (r, _), v in MAP.items() if r == row) for row in ROW_NAMES))
    emit("F-R.columns", join(sum(v for (_, c), v in MAP.items() if c == col) for col in COLS))
    shading = [0, 0, 0, 0]
    for v in MAP.values():
        shading[0 if v <= 10 else 1 if v <= 100 else 2 if v <= 1000 else 3] += 1
    emit("F-R.shading", join(shading))
    e5, b7, f6 = cell("E5"), cell("B7"), cell("F6")
    check("E5 is the only cell with more points than B7", sorted(MAP.values())[-2:] == [b7, e5])
    emit("F-R.E5", e5)
    emit("F-R.E5.percent", percent(F(e5, total), 1))
    emit("F-R.B7", b7)
    emit("F-R.top_two.percent", percent(F(e5 + b7, total), 0))
    check("the paper's '23 percent' for E5 and B7", percent(F(e5 + b7, total), 0) == "23")
    emit("F-R.F6", f6)
    emit("F-R.F6.percent", percent(F(f6, total), 2))
    emit("F-R.F6.rank", 1 + sum(1 for v in MAP.values() if v > f6))
    emit("F-R.E5.rank", 1 + sum(1 for v in MAP.values() if v > e5))
    emit("F-R.E5_over_F6", fixed(F(e5, f6), 1))
    emit("F-R.grid.cells", len(GRID_ROWS) * len(GRID_COLS))
    emit("F-R.cell.miles", "1x0.84")
    emit("F-R.grid.area", fixed(len(GRID_ROWS) * len(GRID_COLS) * F(84, 100), 1))
    inside = sum(v for (r, c), v in MAP.items() if r in GRID_ROWS and c in GRID_COLS)
    emit("F-R.grid.points", inside)
    emit("F-R.outside.points", total - inside)

    sees = F(2, 3)
    emit("F-R.model.seeing", frac(sees))
    best, sweep = model_search("best", sees, 400), model_search("sweep", sees, 400)
    emit("F-R.model.best.first_cells", join(c + r for r, c, _ in best[:6]))
    f6_best = next(i for i, (r, c, _) in enumerate(best, 1) if (r, c) == ("6", "F"))
    f6_sweep = next(i for i, (r, c, _) in enumerate(sweep, 1) if (r, c) == ("6", "F"))
    emit("F-R.model.best.F6_search", f6_best)
    emit("F-R.model.sweep.F6_search", f6_sweep)
    check("the sweep reaches F6 at its place in reading order", f6_sweep == 5 * 15 + 6)
    for mark in (10, 20, 35, 81):
        emit(f"F-R.model.best.found_by.{mark}", percent(best[mark - 1][2], 1))
        emit(f"F-R.model.sweep.found_by.{mark}", percent(sweep[mark - 1][2], 1))
    check("the model search is never behind the sweep",
          all(a[2] >= b[2] for a, b in zip(best, sweep)))


# --------------------------------------------------------------------------
# F-H: "search everywhere"
# --------------------------------------------------------------------------
FEET_PER_SEA_MILE = F(1852) / F(3048, 10000)
TRACK_MILES = 1026  # Richardson and Stone (1971), p. 149


def hook():
    emit("F-H.track.miles", TRACK_MILES)
    emit("F-H.track.days_at_one_knot", fixed(F(TRACK_MILES, 24), 0))
    check("1,026 miles at one knot are the paper's 'about 43 days'", fixed(F(TRACK_MILES, 24), 0) == "43")
    check("153 miles at 24 a day are the paper's 6.4 days", fixed(F(153, 24), 1) == "6.4")
    strip_feet = 200
    emit("F-H.strip.feet", strip_feet)
    emit("F-H.feet_per_sea_mile", fixed(FEET_PER_SEA_MILE, 0))
    area = 180 * F(84, 100)
    once = area / (strip_feet / FEET_PER_SEA_MILE)
    emit("F-H.once.miles", fixed(once, 0))
    emit("F-H.once.hundreds", fixed(once / 100, 0))
    emit("F-H.once.days_at_one_knot", fixed(once / 24, 0))
    emit("F-H.once.over_track", fixed(once / TRACK_MILES, 1))
    emit("F-H.once.miles.camera", fixed(area / (185 / FEET_PER_SEA_MILE), 0))
    emit("F-H.once.miles.magnetometer", fixed(area / (220 / FEET_PER_SEA_MILE), 0))


# --------------------------------------------------------------------------
# F-K: Koopman's random search formula, and the lanes
# --------------------------------------------------------------------------
LANE_SEED, SIDE, WIDTH, RUNS = 1946, 200, 4, 1000
PER_COVERAGE = SIDE // WIDTH


def lanes_of(draws):
    """Lanes from pairs of uniform draws: (vertical?, first row or column)."""
    return [(u < 0.5, min(int(v * SIDE), SIDE - 1)) for u, v in zip(draws[0::2], draws[1::2])]


def swept_after(lanes):
    """Points swept after each lane, counted on the full grid."""
    grid = np.zeros((SIDE, SIDE), dtype=bool)
    counts = []
    for vertical, at in lanes:
        index = [(at + k) % SIDE for k in range(WIDTH)]
        if vertical:
            grid[:, index] = True
        else:
            grid[index, :] = True
        counts.append(int(grid.sum()))
    return counts


def formula():
    for name, coverage in (("half", 0.5), ("1", 1.0), ("2", 2.0), ("3", 3.0)):
        emit(f"F-K.formula.{name}", f"{100 * (1 - math.exp(-coverage)):.1f}")
        emit(f"F-K.perfect.{name}", f"{100 * min(1.0, coverage):.0f}")
    emit("F-K.formula.1.unseen", f"{100 * math.exp(-1.0):.1f}")
    emit("F-K.coverage.flat", f"{-math.log(1 - 2 / 3):.2f}")
    emit("F-K.coverage.canyon", f"{-math.log(1 - 1 / 4):.2f}")
    check("two in three is the formula at coverage ln 3", abs((1 - math.exp(-math.log(3))) - 2 / 3) < 1e-12)

    emit("F-K.lanes.seed", LANE_SEED)
    emit("F-K.lanes.side", SIDE)
    emit("F-K.lanes.share.percent", percent(F(WIDTH, SIDE), 0))
    emit("F-K.lanes.per_coverage", PER_COVERAGE)
    emit("F-K.lanes.runs", RUNS)
    n = 3 * PER_COVERAGE
    rng = np.random.Generator(np.random.PCG64DXSM(LANE_SEED))
    draws = rng.random(2 * n * (RUNS + 1)).tolist()
    runs = [lanes_of(draws[2 * n * r: 2 * n * (r + 1)]) for r in range(RUNS + 1)]
    one = swept_after(runs[0])
    sums = [0] * n
    for lanes in runs[1:]:
        for i, x in enumerate(swept_after(lanes)):
            sums[i] += x
    points = SIDE * SIDE
    for coverage in (1, 2, 3):
        k = coverage * PER_COVERAGE
        emit(f"F-K.lanes.one_run.{coverage}", percent(F(one[k - 1], points), 1))
        emit(f"F-K.lanes.mean.{coverage}", percent(F(sums[k - 1], points * RUNS), 1))
        exact = 1 - (1 - WIDTH / SIDE) ** k
        emit(f"F-K.lanes.exact.{coverage}", f"{100 * exact:.1f}")
        check(f"the mean of {RUNS} runs is within one point of the exact average at coverage {coverage}",
              abs(sums[k - 1] / (points * RUNS) - exact) < 0.01)
        check(f"the formula is within half a point of thin lanes at coverage {coverage}",
              abs(exact - (1 - math.exp(-coverage))) < 0.005)
    emit("F-K.lanes.first", join(("v" if v else "h") + str(at) for v, at in runs[0][:4]))


# --------------------------------------------------------------------------
# F-D: the drifting cloud (model)
# --------------------------------------------------------------------------
DRIFT_SEED, CLOUD = 2013, 2000
START_RADIUS, DRIFT, ERROR_SHARE = F(1), F(3, 2), F(3, 10)
TRACK, STRIP = F(180), F(1, 10)


def radius(hours):
    return START_RADIUS + ERROR_SHARE * DRIFT * hours


def cloud_dots():
    rng = np.random.Generator(np.random.PCG64DXSM(DRIFT_SEED))

    def in_disc():
        while True:
            x = -1.0 + 2.0 * rng.random()
            y = -1.0 + 2.0 * rng.random()
            if x * x + y * y <= 1.0:
                return x, y

    r0, e = float(START_RADIUS), float(ERROR_SHARE * DRIFT)
    dots = []
    for _ in range(CLOUD):
        sx, sy = in_disc()
        ex, ey = in_disc()
        dots.append((r0 * sx, r0 * sy, e * ex, e * ey))
    return dots


def flight(dots, hours, side):
    half = side / 2.0
    inside = sum(1 for sx, sy, ex, ey in dots
                 if abs(sx + ex * hours) <= half and abs(sy + ey * hours) <= half)
    sees = 1.0 - math.exp(-float(TRACK * STRIP) / float(side * side))
    return inside, sees, inside / len(dots) * sees


def cloud():
    emit("F-D.seed", DRIFT_SEED)
    emit("F-D.dots", CLOUD)
    emit("F-D.start_radius", frac(START_RADIUS))
    emit("F-D.drift", fixed(DRIFT, 1))
    emit("F-D.error_share", fixed(ERROR_SHARE, 1))
    emit("F-D.track", frac(TRACK))
    emit("F-D.strip", fixed(STRIP, 1))
    emit("F-D.strip.metres", fixed(STRIP * 1852, 0))
    swept = TRACK * STRIP
    emit("F-D.swept", frac(swept))
    dots = cloud_dots()
    last = 1.0
    for hours in (1, 3, 6, 12):
        r = radius(hours)
        area = math.pi * float(r) ** 2
        coverage = float(swept) / area
        sees = 1.0 - math.exp(-coverage)
        farthest = max(math.hypot(sx + ex * hours, sy + ey * hours) for sx, sy, ex, ey in dots)
        check(f"every dot is inside the model's radius at hour {hours}", farthest <= float(r))
        check(f"the same flight sees less at hour {hours}", sees < last)
        last = sees
        emit(f"F-D.h{hours}.radius", fixed(r, 2))
        emit(f"F-D.h{hours}.area", f"{area:.1f}")
        emit(f"F-D.h{hours}.coverage", f"{coverage:.2f}")
        emit(f"F-D.h{hours}.seeing", f"{100 * sees:.0f}")
        emit(f"F-D.h{hours}.farthest", f"{farthest:.2f}")
    for hours in (3, 6):
        flights = [flight(dots, float(hours), side) for side in range(1, 9)]
        best = max(range(8), key=lambda i: (flights[i][2], -i))
        emit(f"F-D.flight.h{hours}.best_side", best + 1)
        emit(f"F-D.flight.h{hours}.inside", flights[best][0])
        emit(f"F-D.flight.h{hours}.seeing", f"{100 * flights[best][1]:.0f}")
        emit(f"F-D.flight.h{hours}.find", f"{100 * flights[best][2]:.0f}")
        emit(f"F-D.flight.h{hours}.find_by_side", join(f"{100 * x[2]:.0f}" for x in flights))
    r = radius(1)
    emit("F-D.double.area_factor", frac((2 * r) ** 2 / r**2))
    emit("F-D.double.coverage_factor", frac(r**2 / (2 * r) ** 2))


# --------------------------------------------------------------------------
# Cited (typed a second time from the sources; docs/fact-sheet.md §Sources)
# --------------------------------------------------------------------------
def iso(d):
    return d.isoformat()


def months_between(a, b):
    months = (b.year - a.year) * 12 + (b.month - a.month)
    return months - 1 if b.day < a.day else months


FOOT = F(3048, 10000)  # metres


def scorpion():
    # NHHC, DANFS "Scorpion VI (SSN-589)"; NT-24-1 p. 7; Richardson and Stone (1971)
    last_message = datetime.date(1968, 5, 21)
    sank = datetime.date(1968, 5, 22)
    due = datetime.date(1968, 5, 27)
    presumed = datetime.date(1968, 6, 5)
    found = datetime.date(1968, 10, 28)
    began = datetime.date(1968, 6, 10)
    scenarios_set = datetime.date(1968, 7, 18)
    map_arrived = datetime.date(1968, 8, 12)
    emit("F-S.crew", 99)
    emit("F-S.last_message", iso(last_message))
    emit("F-S.sank", iso(sank))
    emit("F-S.due", iso(due))
    emit("F-S.presumed_lost", iso(presumed))
    emit("F-S.found", iso(found))
    emit("F-S.days.sank_to_due", (due - sank).days)
    emit("F-S.days.due_to_presumed", (presumed - due).days)
    emit("F-S.days.due_to_found", (found - due).days)
    emit("F-S.months.due_to_found", months_between(due, found))
    emit("F-S.search_began", iso(began))
    emit("F-S.scenarios_set", iso(scenarios_set))
    emit("F-S.scenarios", 9)
    emit("F-S.map_arrived", iso(map_arrived))
    emit("F-S.days.search_to_map", (map_arrived - began).days)
    emit("F-S.months.search_to_map", months_between(began, map_arrived))
    emit("F-S.days.map_to_found", (found - map_arrived).days)
    emit("F-S.cruises", 5)
    emit("F-S.yards", 260)
    emit("F-S.yards.metres", fixed(260 * 3 * FOOT, 0))
    emit("F-S.miles_from_azores", 400)
    emit("F-S.depth.feet", 10000)
    emit("F-S.depth.metres", fixed(10000 * FOOT, 0))
    emit("F-S.depth.miles", fixed(F(10000, 5280), 1))
    emit("F-S.forecast.days", "35-45")
    emit("F-S.bottom_days", 43)
    check("the 43 days fell inside the forecast of 1 August", 35 <= 43 <= 45)
    emit("F-S.miles_to_reach_F6", 153)
    # Table 2 and p. 151; Tables 1 and 3
    emit("F-S.sweep.at_sea", "800,400")
    emit("F-S.sweep.after", "185,220,0")
    emit("F-S.sweep.magnetometer.ratio", fixed(F(400, 220), 1))
    at_sea, after = [65, 94, 99], [47, 65, 74]
    emit("F-S.F6.searched.at_sea", join(at_sea))
    emit("F-S.F6.searched.after", join(after))
    check("at sea F6 always looked more searched than it had been", all(a > b for a, b in zip(at_sea, after)))
    emit("F-S.F6.left.at_sea.percent", 100 - at_sea[1])
    emit("F-S.F6.left.after.percent", 100 - after[1])
    emit("F-S.F6.left.ratio", fixed(F(100 - after[1], 100 - at_sea[1]), 1))


def palomares():
    # Defense Nuclear Agency, Palomares Summary Report (1975), pp. 15-17, 94
    crash = datetime.date(1966, 1, 17)
    found = datetime.date(1966, 3, 15)
    recovered = datetime.date(1966, 4, 7)
    emit("F-P.date", iso(crash))
    emit("F-P.bombs", 4)
    emit("F-P.on_land", 3)
    emit("F-P.in_sea", 4 - 3)
    emit("F-P.airmen", "7 of 11")
    emit("F-P.found", iso(found))
    emit("F-P.recovered", iso(recovered))
    emit("F-P.days.to_found", (found - crash).days)
    emit("F-P.days.to_recovered", (recovered - crash).days)
    emit("F-P.depth.feet", 2550)
    emit("F-P.depth.metres", fixed(2550 * FOOT, 0))


def air_france():
    # BEA final report (2012); Stone, Keller, Kratzke and Strumpfer (2014), pp. 69-77
    lost, found = datetime.date(2009, 6, 1), datetime.date(2011, 4, 2)
    emit("F-A.lost", iso(lost))
    emit("F-A.found", iso(found))
    emit("F-A.aboard", 216 + 12)
    emit("F-A.months", months_between(lost, found))
    emit("F-A.days", (found - lost).days)
    emit("F-A.circle.nm", 40)
    radius_km = F(40 * 1852, 1000)
    emit("F-A.circle.km", fixed(radius_km, 0))
    area = math.pi * float(radius_km) ** 2
    emit("F-A.circle.area_km2", f"{area:.0f}")
    check("the circle is the BEA's 'more than 17,000 km2'", 17000 < area < 17500)
    weights = [35, 35, 30]
    emit("F-A.weights", join(weights))
    emit("F-A.weights.sum", sum(weights))
    emit("F-A.points", 75000)
    emit("F-A.empty_searches", 4)
    # the paper's own sum, p. 76: (1 − 0.1²)(0.8)² + 0.9 · 2(0.8)(0.2)
    cap, survives = F(9, 10), F(8, 10)
    independent = (1 - (1 - cap) ** 2) * survives**2 + cap * 2 * survives * (1 - survives)
    dependent = cap * survives
    emit("F-A.beacon.independent", fixed(independent, 4))
    emit("F-A.beacon.independent.rounded", fixed(independent, 2))
    emit("F-A.beacon.dependent", fixed(dependent, 2))
    used = F(1, 4) * F(92, 100) + F(3, 4) * dependent
    emit("F-A.beacon.used.percent", percent(used, 0))
    emit("F-A.beacon.left.percent", percent(1 - used, 0))
    check("the paper's 0.77 and 0.23", percent(used, 0) == "77" and percent(1 - used, 0) == "23")
    emit("F-A.found.nm", "6.5")
    emit("F-A.found.km", fixed(F(65, 10) * F(1852, 1000), 0))
    emit("F-A.found.share_of_radius", percent(F(65, 10) / 40, 0))
    emit("F-A.depth.metres", 3900)


def coast_guard():
    # Kratzke, Stone and Frost (2010); USCG Addendum M16130.2F; Tough (2014)
    emit("F-C.casp", 1974)
    emit("F-C.sarops", 2007)
    emit("F-C.sarops.dots", "2500,5000,10000")
    emit("F-C.no_search.percent", 90)
    emit("F-C.aldridge.date", iso(datetime.date(2013, 7, 24)))
    fell = datetime.datetime(2013, 7, 24, 3, 30)
    seen = datetime.datetime(2013, 7, 24, 14, 46) + datetime.timedelta(minutes=12)
    rerun = datetime.datetime(2013, 7, 24, 8, 30)
    emit("F-C.aldridge.fell", fell.strftime("%H:%M"))
    emit("F-C.aldridge.seen", seen.strftime("%H:%M"))
    minutes = int((seen - fell).total_seconds()) // 60
    emit("F-C.aldridge.in_water", f"{minutes // 60}h{minutes % 60:02d}")
    emit("F-C.aldridge.in_water.hours", fixed(F(minutes, 60), 1))
    check("'almost twelve hours' in the water", 11 * 60 < minutes < 12 * 60)
    emit("F-C.aldridge.rerun", rerun.strftime("%H:%M"))
    emit("F-C.aldridge.rerun.hours_after_fall", fixed(F(int((rerun - fell).total_seconds()), 3600), 0))
    emit("F-C.aldridge.miles_south", 30)
    emit("F-C.aldridge.miles_offshore", "40-43")


def names():
    emit("F-N.koopman", 1946)
    emit("F-N.bayes.died", 1761)
    emit("F-N.bayes.read", 1763)
    emit("F-N.laplace", 1774)
    emit("F-N.chew", 1967)
    emit("F-N.paper", 1971)
    emit("F-N.paper.years_after", 1971 - 1968)


# --------------------------------------------------------------------------
# F-Q: the quizzes
# --------------------------------------------------------------------------
QUIZZES = [
    ("stories", 6),
    ("first_search", 6),
    ("after_empty", 6),
    ("next_search", 6),
    ("two_searches", 5),
    ("radius_doubles", 5),
]


def quizzes():
    emit("F-Q.count", len(QUIZZES))
    emit("F-Q.names", join(name for name, _ in QUIZZES))
    dots = lit_dots()
    # A's chance from the two stories, as the card works it: 60% of a half plus 40% of a quarter
    stories_answer = F(60, 100) * F(1, 2) + F(40, 100) * F(1, 4)
    check("the stories quiz is the map's A", stories_answer == shares(dots)[0])
    first = max(range(4), key=lambda i: (finds(dots)[i], -i))
    _, after = search(dots, first)
    second = max(range(4), key=lambda i: (finds(after)[i], -i))
    # two searches of one cell, by listing the four outcomes of seen/missed
    seen_twice = sum(
        (SEES[1] if a else 1 - SEES[1]) * (SEES[1] if b else 1 - SEES[1])
        for a in (True, False) for b in (True, False) if a or b
    )
    answers = {
        "stories": percent(stories_answer, 0),
        "first_search": CELLS[first],
        "after_empty": percent(shares(after)[first], 1),
        "next_search": CELLS[second],
        "two_searches": frac(seen_twice),
        "radius_doubles": frac(F(1, 2) ** 2),
    }
    for name, seconds in QUIZZES:
        emit(f"F-Q.{name}.seconds", seconds)
        emit(f"F-Q.{name}.answer", answers[name])


def main():
    small_sea()
    rules()
    map_1968()
    hook()
    formula()
    cloud()
    scorpion()
    palomares()
    air_france()
    coast_guard()
    names()
    quizzes()
    print(f"# {len(_EMITTED)} facts, {_CHECKS[0]} cross-checks, {len(_FAILED)} failed", file=sys.stderr)
    if _FAILED:
        for name in _FAILED:
            print(f"# FAILED: {name}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
