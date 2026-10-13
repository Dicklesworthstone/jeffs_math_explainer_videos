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
    third = max(range(4), key=lambda i: (find3[i], -i))
    emit("F-M.third.cell", CELLS[third])

    # the second choice in dots of the first hundred: what a search of each
    # cell would catch now, counted on the dots still lit after B's search
    catch = [d * s for d, s in zip(after, SEES)]
    emit("F-M.second.find.dots", join(frac(x) for x in catch))
    emit("F-M.second.find.dots.rounded", join(fixed(x, 0) for x in catch))
    check("in dots, nothing has changed for A or C, and only B has fallen",
          catch[0] == dots[0] * SEES[0] and catch[2] == dots[2] * SEES[2] and catch[1] < dots[1] * SEES[1])

    # "more dots, same map": split every dot in three and count whole dots
    split = 3
    lit = [d * split for d in after]
    check("after the split every dot count is whole", all(x.denominator == 1 for x in lit))
    emit("F-M.split.factor", split)
    emit("F-M.split.lit", frac(sum(lit)))
    emit("F-M.split.lit.cells", join(frac(x) for x in lit))
    out2, lit2 = search(lit, second)
    check("the second search puts out whole dots", out2.denominator == 1)
    emit("F-M.split.second.dots_out", frac(out2))
    emit("F-M.split.second.lit.cells", join(frac(x) for x in lit2))
    check("the split dots give 60, 15, 10, 15", join(percent(s, 0) for s in shares(lit2)) == "60,15,10,15")
    out3, lit3 = search(lit2, third)
    check("the third search puts out whole dots", out3.denominator == 1)
    emit("F-M.third.dots_out", frac(out3))
    emit("F-M.split.third.lit.cells", join(frac(x) for x in lit3))
    find4 = finds(lit3)
    emit("F-M.fourth.cell", CELLS[max(range(4), key=lambda i: (find4[i], -i))])

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

    # The proof in slices, by counting dots. Search one cell over and over,
    # alone: the dots each search puts out are that cell's slices.
    slices = []
    for cell in range(4):
        dots, mine = lit_dots(), []
        for _ in range(8):
            out, dots = search(dots, cell)
            mine.append(out)
        check(f"cell {CELLS[cell]}'s slices only shrink", all(a > b for a, b in zip(mine, mine[1:])))
        slices.append(mine)
        emit(f"F-U.slices.{CELLS[cell]}", join(fixed(x, 1) for x in mine[:4]))
    # What the rule's searches put out, by running it on the dots.
    dots, searched, outs = lit_dots(), [0] * 4, []
    for step in range(8):
        cell = choose("best", dots, step, searched)
        out, dots = search(dots, cell)
        check(f"search {step + 1} puts out that cell's next slice, whatever came between",
              out == slices[cell][searched[cell]])
        searched[cell] += 1
        outs.append(out)
    emit("F-U.best.dots_out", join(frac(x) for x in outs))
    emit("F-U.best.dots_out.rounded", join(fixed(x, 1) for x in outs))
    biggest = sorted((x for mine in slices for x in mine), reverse=True)[:8]
    emit("F-U.slices.rule_takes_biggest", outs == biggest)

    # a cell with no dots never gains one: search the two cells that have
    # dots, thirty times, and count the empty cells' dots
    dots = [F(50), F(50), F(0), F(0)]
    for step in range(30):
        cell = step % 2
        dots[cell] -= dots[cell] * F(2, 3)
    emit("F-U.zero_stays_zero", dots[2] == 0 and dots[3] == 0)


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
    # every plan searches the lettered grid only: its 180 cells, in reading order
    sweep = [(r, c) for r in GRID_ROWS for c in GRID_COLS]
    reading = {k: i for i, k in enumerate(sweep)}
    left = {k: F(MAP.get(k, 0), 10000) for k in sweep}
    # the map, never updated: likeliest first by the printed numbers
    once = sorted(sweep, key=lambda k: (-MAP.get(k, 0), reading[k]))
    found, out = F(0), []
    for look in range(looks):
        if rule == "sweep":
            where = sweep[look % len(sweep)]
        elif rule == "once":
            where = once[look % len(once)]
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
    emit("F-R.E5.one_in", frac(F(total, e5)))
    check("E5 holds exactly one dot in eight", e5 * 8 == total)
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
    emit("F-R.grid.miles", f"{len(GRID_ROWS)}x{fixed(len(GRID_COLS) * F(84, 100), 1)}")
    inside = sum(v for (r, c), v in MAP.items() if r in GRID_ROWS and c in GRID_COLS)
    emit("F-R.grid.points", inside)
    emit("F-R.outside.points", total - inside)

    sees = F(2, 3)
    emit("F-R.model.seeing", frac(sees))
    plans = {name: model_search(name, sees, 81) for name in ("best", "once", "sweep")}
    emit("F-R.model.best.first_cells", join(c + r for r, c, _ in plans["best"][:6]))
    for name in ("best", "once", "sweep"):
        for mark in (20, 81):
            emit(f"F-R.model.{name}.found_by.{mark}", percent(plans[name][mark - 1][2], 1))
    check("the rule is never behind the map never updated, nor that behind row by row",
          all(a[2] >= b[2] >= c[2] for a, b, c in zip(plans["best"], plans["once"], plans["sweep"])))
    at20 = [plans[name][19][2] for name in ("best", "once", "sweep")]
    check("at twenty searches most of the rule's lead is the map itself",
          at20[1] - at20[2] > 5 * (at20[0] - at20[1]))
    f6_sweep = next(i for i, (r, c, _) in enumerate(plans["sweep"], 1) if (r, c) == ("6", "F"))
    check("row by row reaches F6 at its place in reading order", f6_sweep == 5 * 15 + 6)
    # one model search in sea miles of towing: one cell's share of a pass
    # over the grid with a 200-foot strip, at the coverage that sees 2 in 3
    per_cell = F(84, 100) / (200 / FEET_PER_SEA_MILE)
    emit("F-R.model.search_miles", f"{float(per_cell) * math.log(3):.0f}")


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
    emit("F-H.once.miles.believed", fixed(area / (800 / FEET_PER_SEA_MILE), 0))
    # one cell: 0.84 sea miles wide, so that many strips side by side, each
    # one sea mile long
    passes = F(84, 100) * FEET_PER_SEA_MILE / strip_feet
    emit("F-H.once.per_cell", fixed(passes * 1, 0))
    emit("F-H.once.passes_per_cell", fixed(passes, 0))
    check("180 cells of that much track are the whole pass", passes * 180 == once)
    emit("F-H.track.cells", fixed(TRACK_MILES / passes, 0))
    emit("F-H.track.share.percent", percent(TRACK_MILES / once, 0))


# --------------------------------------------------------------------------
# F-K: Koopman's random search formula, and the lanes
# --------------------------------------------------------------------------
LANE_SEED, SIDE, WIDTH, RUNS = 1946, 200, 2, 1000
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
    # Tables 1 and 3 again: the chance the whole search so far would have
    # found her, and F6's chance on the map on 9 October by the paper's own
    # update, L' = L (1 − E) ÷ (1 − SEP)
    sep_at_sea, sep_after = [40, 73, 75], [29, 45, 48]
    emit("F-S.SEP.at_sea", join(sep_at_sea))
    emit("F-S.SEP.after", join(sep_after))
    prior = F(cell("F6"), 10000)
    by_ship = prior * F(100 - at_sea[1], 100) / F(100 - sep_at_sea[1], 100)
    by_measure = prior * F(100 - after[1], 100) / F(100 - sep_after[1], 100)
    emit("F-S.F6.chance.at_sea", percent(by_ship, 2))
    emit("F-S.F6.chance.after", percent(by_measure, 2))
    check("even by the measured numbers F6 had fallen, and by the ship's it had fallen further",
          by_ship < by_measure < prior)
    # the same two figures as coverage on Koopman's curve
    emit("F-S.F6.coverage.at_sea", f"{-math.log(1 - at_sea[1] / 100):.2f}")
    emit("F-S.F6.coverage.after", f"{-math.log(1 - after[1] / 100):.2f}")


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
    emit("F-P.years_before_scorpion", 1968 - crash.year)
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
    # of the paper's four steps, the first is the surface search
    emit("F-A.underwater_searches", 4 - 1)
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
    # the first guess, the evening before: 9:30 to 11:30 p.m.
    guess = (datetime.datetime(2013, 7, 23, 21, 30), datetime.datetime(2013, 7, 23, 23, 30))
    middle = guess[0] + (guess[1] - guess[0]) / 2
    emit("F-C.aldridge.guess", f"{guess[0]:%H:%M}-{guess[1]:%H:%M}")
    emit("F-C.aldridge.guess.middle", f"{middle:%H:%M}")
    emit("F-C.aldridge.hours_off", frac(F(int((fell - middle).total_seconds()), 3600)))
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
    ("stories", "03_5", 6),
    ("first_search", "05_4", 6),
    ("after_empty", "06_5", 6),
    ("next_search", "07_3", 6),
    ("two_searches", "09_5", 6),
]

# The code panel, typed from the storyboard's "The code panel" section.
CODE = [
    "p[i] *= 1 - d[i]                       # the search of cell i came back empty",
    "t = sum(p); p = [x / t for x in p]     # the map adds up to one again",
    "find = [x * y for x, y in zip(p, d)]   # chance of a find, cell by cell",
    "i = find.index(max(find))              # search there next",
]


def code_panel():
    """Emit the panel's lines, check they are the storyboard's, and run them."""
    import pathlib
    board = pathlib.Path(__file__).with_name("docs").joinpath("storyboard.md").read_text()
    check("the code panel is the storyboard's, line for line", "\n".join(CODE) in board)
    for n, line in enumerate(CODE, 1):
        emit(f"F-M.code.{n}", line)
    # run it on the four-cell sea, in exact fractions: choose (lines 3, 4),
    # then update (lines 1, 2), eight times
    env = {"p": [F(4, 10), F(3, 10), F(2, 10), F(1, 10)], "d": list(SEES), "i": None}
    order, maps = "", []
    for _ in range(8):
        exec("\n".join(CODE[2:]), {}, env)
        exec("\n".join(CODE[:2]), {}, env)
        order += CELLS[env["i"]]
        maps.append(list(env["p"]))
    check("the panel's code searches in the rule's order", order == "BCAABDAC")
    check("the panel's code gives 50, 12.5, 25, 12.5 and then 60, 15, 10, 15",
          join(percent(x, 1) for x in maps[0]) == "50.0,12.5,25.0,12.5"
          and join(percent(x, 0) for x in maps[1]) == "60,15,10,15")


def storyboard_countdowns():
    """(line, seconds) of every countdown beat in docs/storyboard.md."""
    import pathlib
    import re
    out = []
    for row in pathlib.Path(__file__).with_name("docs").joinpath("storyboard.md").read_text().splitlines():
        cells = [c.strip() for c in row.split("|")]
        if len(cells) > 5 and re.fullmatch(r"\d\d_\d+", cells[1]) and "countdown" in cells[5]:
            out.append((cells[1], int(re.search(r"listen (\d+) s", cells[5]).group(1))))
    return out


def quizzes():
    emit("F-Q.count", len(QUIZZES))
    emit("F-Q.names", join(name for name, _, _ in QUIZZES))
    check("the storyboard's countdowns are exactly the quizzes",
          storyboard_countdowns() == [(line, seconds) for _, line, seconds in QUIZZES])
    dots = lit_dots()
    # A's chance from the two stories, as the card works it: 60% of a half plus 40% of a quarter
    stories_answer = F(60, 100) * F(1, 2) + F(40, 100) * F(1, 4)
    check("the stories quiz is the map's A", stories_answer == shares(dots)[0])
    first = max(range(4), key=lambda i: (finds(dots)[i], -i))
    _, after = search(dots, first)
    second = max(range(4), key=lambda i: (finds(after)[i], -i))
    # two searches of one cell, on nine dots: each pass catches two in
    # three of what is still lit
    nine = F(9)
    for _ in range(2):
        nine -= nine * SEES[1]
    answers = {
        "stories": percent(stories_answer, 0),
        "first_search": CELLS[first],
        "after_empty": percent(shares(after)[first], 1),
        "next_search": CELLS[second],
        "two_searches": frac(9 - nine),
    }
    for name, line, seconds in QUIZZES:
        emit(f"F-Q.{name}.line", line)
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
    code_panel()
    quizzes()
    print(f"# {len(_EMITTED)} facts, {_CHECKS[0]} cross-checks, {len(_FAILED)} failed", file=sys.stderr)
    if _FAILED:
        for name in _FAILED:
            print(f"# FAILED: {name}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
