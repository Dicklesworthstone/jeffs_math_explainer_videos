#!/usr/bin/env python3
"""Facts oracle for Video I, "Proofs You Can See".

Run from this folder (`./x visual_proofs facts-check` does):

    python3 -I -B oracle.py

stdout: one `KEY<TAB>VALUE` line per fact, the same keys `proofs facts`
prints (src/facts.rs), in the rows of docs/fact-sheet.md: F-C (the pattern and
the circle), F-O (L-shaped layers), F-T (two staircases), F-K (the cubes'
slabs), F-S (three stacks and a box), F-H (halves), F-R (quarters), F-B (the
cut square, cube and fourth power), F-P (four triangles), F-L (the picture
that lies), F-Q (the quizzes) and F-X (cited).

stderr: cross-checks (lines starting with "#") and a summary; exit 1 if any
cross-check failed. A key that ends in `.through` is the reach of a slow
check: the check is run here for every size up to the reach, and the first
size that fails stops the oracle with a message naming it.

This file was written from docs/fact-sheet.md alone, without reading the Rust
model, and every picture is rebuilt by a route of its own:
  * the circle's regions by drawing the lines one at a time: a new line adds
    one region, and one more for each distinct point where it crosses the
    lines already there (exact fractions, crossings that coincide counted
    once). Corners and edges (the video's route) are only a cross-check;
  * an L-shaped layer as a corner tile and two arms, a staircase by rows and
    its copy as the column heights reversed and hung from the ceiling; every
    fit by marking unit cells on a grid, each exactly once;
  * the cubes' layers by laying slabs along each arm from its far end: the
    arm's length in half slabs decides whether a slab must be cut, never the
    cube being odd or even; then every unit cell of the square is marked;
  * Siu's three stacks from the fact sheet's inequalities, the box filled
    in half-height cells; the turn that carries one stack onto each of the
    three found by trying all 48 rigid motions of the grid; the next height
    by taking the lifted smaller solid out of the bigger one and looking at
    what is left;
  * the halved and quartered squares as exact rectangles, never as powers;
  * the cut cube by building every block as a product of intervals and
    marking the cells of the cube;
  * the triangles as polygons: the three copies by quarter turns about the
    big square's centre, areas by the shoelace formula, every fit by the
    exact area two convex polygons share, each slide worked out from where
    the piece has to land;
  * the lying picture cell by cell: what each unit square of the grid shares
    with each piece; the second arrangement found, not typed (the triangles
    change ends, and a search of every slide of the two L-shapes into the
    space left finds one way only);
  * the whole-number sums by running totals in exact integers;
  * CITED literals typed from the Sources table of docs/fact-sheet.md.
"""
import itertools
import math
import pathlib
import re
import sys
from collections import Counter
from fractions import Fraction as F

# --- What the video draws -------------------------------------------------------
# The only typed sizes in this file (docs/fact-sheet.md, "Facts"). Every value
# printed below is computed from these; the only other literals are the cited
# years and pages in cited().

SIZES = 5  # five cubes, five L-shaped layers, five columns, five running totals
HUNDRED = 100  # "after a hundred layers"; the hundred-step staircase

# The circle: seven points, each from a parameter p/d, in the order the
# chapter adds them; the video draws the first six.
CIRCLE_PARAMETERS = ((1, 19), (-6, 1), (7, 3), (-1, 2), (4, 9), (-7, 6), (1, 1))
CIRCLE_SHOWN = 6
# Three lines through one point: these three points and their opposites.
THREE_THROUGH_ONE = ((F(3, 5), F(4, 5)), (F(5, 13), F(12, 13)), (F(15, 17), F(8, 17)))

STACK_LAYERS = 4  # Siu's stack as drawn; chapter 09 grows it by one
HALVED_PIECES = 5  # pieces of the halved square that carry a readout
SMALL_GAPS = (("thousandth", 1000), ("millionth", 1000000))
QUARTERED_LEVELS = 3  # levels of the quartered square that carry a readout
PARTS = (3, 2)  # the long part and the short part of the cut side
CUT_POWERS = (("square", 2), ("cube", 3), ("fourth", 4))
CUT_SWEEP = (4, 5)  # cuts rebuilt: dimensions 1 to 4, each part from 1 to 5
LEGS = (3, 4)  # the right triangle's two shorter sides
OTHER_LEGS = (2, 5)  # the second pair, shown at 13_6

# The lying picture's pieces in their first arrangement: two triangles by
# their corners, two L-shapes by rows of unit squares (y, first x, last x).
LIE_BIG = ((8, 0), (8, 3), (0, 0))
LIE_SMALL = ((13, 3), (13, 5), (8, 3))
LIE_L7 = ((0, 8, 12), (1, 8, 9))
LIE_L8 = ((1, 10, 12), (2, 8, 12))

# How far each slow check is asked to reach. A reach is printed only when
# every size from 1 up to it has passed here.
REACH = {
    "F-O.tiled.through": 60,
    "F-O.sum.through": 1000,
    "F-T.tiled.through": 60,
    "F-T.sum.through": 1000,
    "F-K.tiled.through": 40,
    "F-K.sum.through": 1000,
    "F-S.filled.through": 24,
    "F-S.grows.through": 24,
    "F-S.sum.through": 1000,
    "F-H.checked.through": 40,
    "F-R.checked.through": 20,
    "F-P.checked.through": 60,
}

# The quizzes: name, the storyboard line that asks, seconds on the ring.
QUIZZES = (
    ("regions", "02_5", 4),
    ("next_layer", "03_3", 4),
    ("rectangle", "04_3", 5),
    ("cut_slabs", "05_4", 8),
    ("stacks", "07_2", 7),
    ("share", "11_3", 6),
    ("kinds", "12_4", 8),
    ("tilted", "13_4", 6),
    ("slopes", "14_4", 8),
)

# --- Printing and checking ------------------------------------------------------

_EMITTED = {}
_FAILED = []
_CHECKS = [0]
ANSWERS = {}  # quiz name -> answer, filled in by the section that owns the picture


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


def note(text):
    print(f"# note {text}", file=sys.stderr)


def frac(x):
    """A fraction as the video prints it: n/d, or n when whole."""
    x = F(x)
    return str(x.numerator) if x.denominator == 1 else f"{x.numerator}/{x.denominator}"


def join(items, sep=","):
    return sep.join(str(x) for x in items)


def by(*sizes):
    return join((frac(s) for s in sizes), "x")


def reach(key, passes):
    """Run passes(n) for n = 1 to the reach asked for and print how far it got.

    passes(n) returns None when size n is fine and a sentence when it is not;
    the first failure stops the oracle, so a reach is never printed on trust.
    """
    done = 0
    for n in range(1, REACH[key] + 1):
        why = passes(n)
        if why is not None:
            sys.exit(f"oracle: {key} asked for {REACH[key]} but fails at {n}: {why}")
        done = n
    emit(key, done)


def misfit(cells, wide, tall):
    """Why these unit cells do not tile a wide-by-tall grid; None when they do."""
    grid = [[0] * wide for _ in range(tall)]
    for x, y in cells:
        if not (0 <= x < wide and 0 <= y < tall):
            return f"cell ({x}, {y}) lies outside {wide} by {tall}"
        grid[y][x] += 1
    for y, row in enumerate(grid):
        for x, times in enumerate(row):
            if times != 1:
                return f"cell ({x}, {y}) is covered {times} times"
    return None


def extent(cells):
    """Wide and tall of the smallest grid, cornered at the origin, that holds the cells."""
    return 1 + max(x for x, _ in cells), 1 + max(y for _, y in cells)


# --- Polygons, exactly -----------------------------------------------------------


def shoelace(poly):
    """Signed area: positive when the corners run counterclockwise."""
    return F(sum(p[0] * q[1] - q[0] * p[1] for p, q in zip(poly, poly[1:] + poly[:1])), 2)


def area(poly):
    return abs(shoelace(poly))


def turn_of(a, b, c):
    """Twice the signed area of a, b, c: positive when c is left of a to b."""
    return (b[0] - a[0]) * (c[1] - a[1]) - (b[1] - a[1]) * (c[0] - a[0])


def shared(p, q):
    """The area two convex polygons have in common (clipping p by each side of q)."""
    p, q = list(p), list(q)
    if shoelace(q) < 0:
        q.reverse()
    for a, b in zip(q, q[1:] + q[:1]):
        kept = []
        for c, d in zip(p[-1:] + p[:-1], p):
            sc, sd = turn_of(a, b, c), turn_of(a, b, d)
            if (sc < 0) != (sd < 0):
                t = F(sc, sc - sd)
                kept.append((c[0] + t * (d[0] - c[0]), c[1] + t * (d[1] - c[1])))
            if sd >= 0:
                kept.append(d)
        p = kept
        if len(p) < 3:
            return F(0)
    return area(p)


def unit(x, y):
    return [(x, y), (x + 1, y), (x + 1, y + 1), (x, y + 1)]


def box(x0, y0, x1, y1):
    return [(x0, y0), (x1, y0), (x1, y1), (x0, y1)]


def slid(poly, by_x, by_y):
    return [(x + by_x, y + by_y) for x, y in poly]


def bounds(poly):
    xs, ys = [p[0] for p in poly], [p[1] for p in poly]
    return min(xs), min(ys), max(xs), max(ys)


def length_squared(side):
    (x0, y0), (x1, y1) = side
    return (x1 - x0) ** 2 + (y1 - y0) ** 2


def long_side(triangle):
    """The longest side of a triangle, as its two ends."""
    return max(itertools.combinations(triangle, 2), key=length_squared)


# --- F-C: the pattern, and the circle that breaks one ---------------------------


def crossing(one, two):
    """The point where two segments cross strictly inside both, or None."""
    (a, b), (c, d) = one, two
    r, s = (b[0] - a[0], b[1] - a[1]), (d[0] - c[0], d[1] - c[1])
    lean = r[0] * s[1] - r[1] * s[0]
    gap = (c[0] - a[0], c[1] - a[1])
    if lean == 0:
        assert gap[0] * r[1] - gap[1] * r[0] != 0, "two lines lie along each other"
        return None
    t = F(gap[0] * s[1] - gap[1] * s[0]) / lean
    u = F(gap[0] * r[1] - gap[1] * r[0]) / lean
    if 0 < t < 1 and 0 < u < 1:
        return (a[0] + t * r[0], a[1] + t * r[1])
    return None


def draw(points):
    """Join the points pair by pair, one line at a time, counting regions as they split.

    A new line is cut by the lines already drawn into one more stretch than
    it has distinct crossing points, and each stretch splits one region in
    two. Returns the regions after each point, the lines, the distinct
    crossings, and how many pairs of lines cross.
    """
    regions, lines, crossings, pairs, after = 1, [], set(), 0, []
    for k, p in enumerate(points):
        for q in points[:k]:
            new = (q, p)
            hits = [x for x in (crossing(old, new) for old in lines) if x is not None]
            pairs += len(hits)
            regions += 1 + len(set(hits))
            crossings |= set(hits)
            lines.append(new)
        after.append(regions)
    return after, lines, crossings, pairs


def by_corners_and_edges(points, lines, crossings):
    """The fact sheet's own count: edges - corners + 1."""
    corners = len(points) + len(crossings)
    edges = len(points)  # the arcs
    for a, b in lines:
        on_it = sum(
            1
            for x in crossings
            if turn_of(a, b, x) == 0 and min(a[0], b[0]) <= x[0] <= max(a[0], b[0]) and min(a[1], b[1]) <= x[1] <= max(a[1], b[1])
        )
        edges += 1 + on_it
    return edges - corners + 1


def circle():
    totals, total = [], 0
    for k in range(1, SIZES + 1):
        total += len(list(itertools.product(range(k), repeat=3)))  # the unit cubes of cube k
        totals.append(total)
    roots = [math.isqrt(t) for t in totals]
    check("every running total of cubes is a perfect square", all(r * r == t for r, t in zip(roots, totals)))
    emit("F-C.cube_totals", join(totals))
    emit("F-C.cube_roots", join(roots))

    points = []
    for p, d in CIRCLE_PARAMETERS:
        t = F(p, d)
        points.append(((1 - t * t) / (1 + t * t), 2 * t / (1 + t * t)))
    check("the drawn points are distinct points of the unit circle", len(set(points)) == len(points) and all(x * x + y * y == 1 for x, y in points))
    shown = points[:CIRCLE_SHOWN]
    after, lines, crossings, pairs = draw(shown)
    doubling, guess = [], 1
    for _ in shown:
        doubling.append(guess)
        guess += guess
    emit("F-C.regions", join(after))
    emit("F-C.doubling", join(doubling))
    after_all, lines_all, crossings_all, pairs_all = draw(points)
    emit("F-C.regions.seven", after_all[-1])
    emit("F-C.lines", len(lines))
    emit("F-C.crossings", len(crossings))
    fours = len(list(itertools.combinations(shown, 4)))
    emit("F-C.fours", fours)
    closed = [1 + math.comb(n, 2) + math.comb(n, 4) for n in range(1, len(points) + 1)]
    emit("F-C.closed_form", join(closed))
    check("the first six of seven give the same counts as six alone", after_all[:CIRCLE_SHOWN] == after)
    check("every four points give one pair of lines that cross", pairs == fours and pairs_all == math.comb(len(points), 4))
    check("no three lines of the drawn points meet at one inside point", len(crossings) == pairs and len(crossings_all) == pairs_all)
    check("regions by corners and edges agree, six points", by_corners_and_edges(shown, lines, crossings) == after[-1])
    check("regions by corners and edges agree, seven points", by_corners_and_edges(points, lines_all, crossings_all) == after_all[-1])
    check("the closed form agrees with the drawing at every count", closed == after_all)
    held = [n for n, (r, g) in enumerate(zip(after_all, doubling + [guess]), 1) if r == g]
    check("doubling holds for 1 to 5 points and fails at 6 and 7", held == [1, 2, 3, 4, 5])
    ANSWERS["regions"] = after[-1]

    through = [p for x, y in THREE_THROUGH_ONE for p in ((x, y), (-x, -y))]
    check("the three-through-one points are distinct points of the unit circle", len(set(through)) == 6 and all(x * x + y * y == 1 for x, y in through))
    t_after, t_lines, t_crossings, t_pairs = draw(through)
    emit("F-C.three_through_one.crossings", len(t_crossings))
    emit("F-C.three_through_one.regions", t_after[-1])
    centre = [line for line in t_lines if turn_of(line[0], line[1], (0, 0)) == 0]
    check("exactly three of those lines pass through the centre", len(centre) == 3 and (F(0), F(0)) in t_crossings)
    check("the centre is the only point where lines coincide", t_pairs - len(t_crossings) == 2)
    check("regions by corners and edges agree, three through one", by_corners_and_edges(through, t_lines, t_crossings) == t_after[-1])
    check("three through one point costs exactly one region", after[-1] - t_after[-1] == 1)
    # 02_6 says "six evenly spaced points give 30". Evenly spaced points have
    # irrational coordinates, but a slanted stretch of the plane keeps every
    # line straight and every crossing where it was along its line, and takes
    # the regular hexagon to this one with whole coordinates.
    hexagon = [(1, 0), (0, 1), (-1, 1), (-1, 0), (0, -1), (1, -1)]
    h_after, _, h_crossings, _ = draw(hexagon)
    emit("F-C.evenly_spaced.regions", h_after[-1])
    check("six evenly spaced points give the three-through-one counts", (h_after[-1], len(h_crossings)) == (t_after[-1], len(t_crossings)))


# --- F-O: L-shaped layers --------------------------------------------------------


def l_layer(k):
    """Layer k as its corner tile and two arms of k - 1 tiles each."""
    corner = [(k - 1, k - 1)]
    upright = [(k - 1, y) for y in range(k - 1)]
    top = [(x, k - 1) for x in range(k - 1)]
    return corner + upright + top


def odd_numbers():
    return itertools.count(1, 2)


def layers():
    drawn = [l_layer(k) for k in range(1, SIZES + 1)]
    sheet = [[(x, y) for x in range(k) for y in range(k) if max(x, y) == k - 1] for k in range(1, SIZES + 2)]
    check("a corner and two arms are the fact sheet's layer", all(sorted(l_layer(k)) == sorted(sheet[k - 1]) for k in range(1, SIZES + 2)))
    sizes = [len(layer) for layer in drawn]
    totals = [sum(sizes[:k]) for k in range(1, SIZES + 1)]
    everything = [cell for layer in drawn for cell in layer]
    wide, tall = extent(everything)
    emit("F-O.layers", join(sizes))
    emit("F-O.totals", join(totals))
    emit("F-O.sum", len(everything))
    emit("F-O.square", by(wide, tall))
    check("the five layers tile that square", misfit(everything, wide, tall) is None and wide == tall)
    emit("F-O.next", len(l_layer(SIZES + 1)))
    ANSWERS["next_layer"] = len(l_layer(SIZES + 1))

    def tiled(n):
        cells = [cell for k in range(1, n + 1) for cell in l_layer(k)]
        why = misfit(cells, n, n)
        if why is None and len(l_layer(n)) != 2 * n - 1:
            why = f"layer {n} has {len(l_layer(n))} cells"
        if why is None and n > 1 and len(l_layer(n)) - len(l_layer(n - 1)) != 2:
            why = f"layer {n} is not two longer than the last"
        return why

    reach("F-O.tiled.through", tiled)

    odds, running = odd_numbers(), [0]

    def summed(n):
        running[0] += next(odds)
        return None if running[0] == n * n else f"the first {n} odd numbers add up to {running[0]}"

    reach("F-O.sum.through", summed)
    first_hundred = list(itertools.islice(odd_numbers(), HUNDRED))
    emit("F-O.hundredth", first_hundred[-1])
    emit("F-O.sum.hundred", sum(first_hundred))
    check("the first hundred odd numbers make a hundred times a hundred", sum(first_hundred) == HUNDRED * HUNDRED)


# --- F-T: two staircases ---------------------------------------------------------


def staircase(n):
    """By rows: row y runs from x = y to the right-hand edge."""
    return [(x, y) for y in range(n) for x in range(y, n)]


def two_staircases(n):
    """The staircase, its turned copy, and the height the copy hangs from.

    Half a turn reverses the order of the columns and stands each on its
    head, so the copy is the column heights reversed, hung from a ceiling
    just high enough for the tallest pair of columns.
    """
    first = staircase(n)
    heights = Counter(x for x, _ in first)
    turned = [heights[n - 1 - x] for x in range(n)]
    ceiling = max(heights[x] + turned[x] for x in range(n))
    copy = [(x, ceiling - 1 - k) for x in range(n) for k in range(turned[x])]
    return first, copy, ceiling, [heights[x] + turned[x] for x in range(n)]


def staircases():
    first, copy, _, pairs = two_staircases(SIZES)
    columns = Counter(x for x, _ in first)
    emit("F-T.columns", join(columns[x] for x in range(SIZES)))
    emit("F-T.count", len(first))
    wide, tall = extent(first + copy)
    emit("F-T.rectangle", by(wide, tall))
    emit("F-T.rectangle.cells", len(first) + len(copy))
    emit("F-T.counts", join(len(staircase(n)) for n in range(1, SIZES + 1)))
    check("the two staircases tile that rectangle", misfit(first + copy, wide, tall) is None)
    check("every pair of columns comes to the same height", len(set(pairs)) == 1 and pairs[0] == tall)
    check("the rectangle is one taller than it is wide", tall == wide + 1)
    ANSWERS["rectangle"] = by(wide, tall)

    def tiled(n):
        one, two, top, _ = two_staircases(n)
        if sorted(two) != sorted((n - 1 - x, n - y) for x, y in one):
            return "the copy is not the fact sheet's half turn of the staircase"
        if sorted(one) != sorted((x, y) for x in range(n) for y in range(x + 1)):
            return "the rows are not the fact sheet's columns"
        return misfit(one + two, n, n + 1) or (None if top == n + 1 else f"the copy hangs from {top}")

    reach("F-T.tiled.through", tiled)

    running = [0]

    def summed(n):
        running[0] += n
        if n == HUNDRED:
            emit("F-T.count.hundred", running[0])
        return None if 2 * running[0] == n * (n + 1) else f"1 + ... + {n} is {running[0]}"

    reach("F-T.sum.through", summed)
    check("the hundred-step staircase was counted", "F-T.count.hundred" in _EMITTED)


# --- F-K: the cubes' slabs -------------------------------------------------------


def cube_layer(k, side):
    """Cube k's slabs laid round a square of the given side.

    Returns (pieces, None) or (None, why it cannot be done). A piece is
    (x, y, wide, tall, slab, where): slab is which of the cube's k slabs it
    was cut from, where is "corner", "upright" or "top". Each arm is `side`
    long; laid from its far end it takes half slabs, k wide and k/2 long, so
    it is the arm's length in half slabs that says whether one is cut.
    """
    pile = list(range(k))
    pieces = [(side, side, k, k, pile.pop(), "corner")]
    half_slabs = F(2 * side, k)
    if half_slabs.denominator != 1:
        return None, f"an arm {side} long is not a whole number of half slabs of side {k}"
    whole, odd = divmod(int(half_slabs), 2)
    if odd and k % 2:
        return None, f"a slab of side {k} cannot be halved on the grid"
    if len(pile) != 2 * whole + odd:
        return None, f"the arms take {2 * whole + odd} slabs and the cube has {len(pile)} left"
    cut = pile.pop() if odd else None
    reached = 0
    if odd:
        pieces.append((side, 0, k, k // 2, cut, "upright"))
        reached = k // 2
    for _ in range(whole):
        pieces.append((side, reached, k, k, pile.pop(), "upright"))
        reached += k
    if reached != side:
        return None, f"the upright arm ends at {reached}, not {side}"
    reached = 0
    if odd:
        pieces.append((0, side, k // 2, k, cut, "top"))
        reached = k // 2
    for _ in range(whole):
        pieces.append((reached, side, k, k, pile.pop(), "top"))
        reached += k
    if reached != side or pile:
        return None, f"the top arm ends at {reached} with {len(pile)} slabs left over"
    return pieces, None


def cubes():
    limit = REACH["F-K.tiled.through"]
    room = 0
    for k in range(1, limit + 1):
        room += k
    board = [bytearray(room) for _ in range(room)]
    state = {"side": 0, "covered": 0, "staircase": 0}
    record = {}

    def tiled(n):
        """Lay layer n on the board: every one of its cells must be new and inside."""
        pieces, why = cube_layer(n, state["side"])
        if why:
            return why
        side = state["side"] + n
        cells = 0
        for x0, y0, wide, tall, _, _ in pieces:
            for y in range(y0, y0 + tall):
                row = board[y] if y < side else None
                for x in range(x0, x0 + wide):
                    if row is None or x >= side:
                        return f"layer {n} sticks out of the square of side {side}"
                    if row[x]:
                        return f"cell ({x}, {y}) is covered twice"
                    row[x] = 1
                    cells += 1
        state["covered"] += cells
        state["staircase"] += n
        if cells != n * n * n:
            return f"layer {n} has {cells} unit squares"
        if state["covered"] != side * side:
            return f"{state['covered']} cells do not fill the square of side {side}"
        if side != state["staircase"]:
            return f"the side is {side}, not 1 + ... + {n}"
        upright = [p for p in pieces if p[5] == "upright"]
        top = [p for p in pieces if p[5] == "top"]
        if sorted((y, x, tall, wide) for x, y, wide, tall, _, _ in upright) != sorted(p[:4] for p in top):
            return "the top arm is not the upright arm's mirror image"
        record[n] = {"pieces": pieces, "before": state["side"], "side": side, "cells": cells, "covered": state["covered"]}
        state["side"] = side
        return None

    for n in range(1, limit + 1):  # run now, printed in the fact sheet's order below
        why = tiled(n)
        if why:
            sys.exit(f"oracle: F-K.tiled.through asked for {limit} but fails at {n}: {why}")
    drawn = [record[k] for k in range(1, SIZES + 1)]
    pieces = [(k, p) for k in range(1, SIZES + 1) for p in record[k]["pieces"]]
    slabs = Counter((k, p[4]) for k, p in pieces)
    emit("F-K.cubes", join(len(list(itertools.product(range(k), repeat=3))) for k in range(1, SIZES + 1)))
    emit("F-K.sides", join(r["side"] for r in drawn))
    emit("F-K.side", drawn[-1]["side"])
    emit("F-K.unit_squares", drawn[-1]["covered"])
    emit("F-K.slabs", len(slabs))
    emit("F-K.cut_slabs", sum(1 for parts in slabs.values() if parts > 1))
    emit("F-K.pieces", len(pieces))
    emit("F-K.layer_areas", join(r["cells"] for r in drawn))
    check("every cut slab is in two halves", all(parts in (1, 2) for parts in slabs.values()))
    check("the cut slabs are the two cube's and the four cube's", sorted(k for (k, _), parts in slabs.items() if parts > 1) == [2, 4])
    check("cube k gives exactly k slabs", all(sum(1 for (c, _) in slabs if c == k) == k for k in range(1, SIZES + 1)))
    check("the slabs hold as many unit squares as the cubes hold unit cubes", drawn[-1]["covered"] == sum(k * k * len(range(k)) for k in range(1, SIZES + 1)))

    def arm(k):
        upright = [p for p in record[k]["pieces"] if p[5] == "upright"]
        wide = max(p[0] + p[2] for p in upright) - min(p[0] for p in upright)
        long = max(p[1] + p[3] for p in upright) - min(p[1] for p in upright)
        fill = sum(F(p[2] * p[3], k * k) for p in upright)
        check(f"arm {k}: its slabs are its area in slabs, and one fewer than the cube in two arms", fill == F(wide * long, k * k) and 2 * fill == k - 1)
        return by(wide, long), frac(fill)

    for k in (SIZES - 1, SIZES):
        size, fill = arm(k)
        emit(f"F-K.arm.{k}", size)
        emit(f"F-K.arm.{k}.slabs", fill)
    sixth = record[SIZES + 1]
    emit("F-K.sixth.cube", sixth["cells"])
    emit("F-K.sixth.side", sixth["side"])
    emit("F-K.sixth.total", sixth["covered"])
    for k in (SIZES + 1, SIZES + 2, SIZES + 3):  # the sixth, and two more for 06_3's strips
        size, fill = arm(k)
        emit(f"F-K.arm.{k}", size)
        emit(f"F-K.arm.{k}.slabs", fill)
    ANSWERS["cut_slabs"] = sum(1 for (k, _), parts in slabs.items() if k == SIZES - 1 and parts > 1)

    # Nicomachus: deal the odd numbers out in runs of 1, 2, 3, ... (S2).
    odds = odd_numbers()
    runs = [list(itertools.islice(odds, k)) for k in range(1, 201)]
    emit("F-K.odd_runs", join((join(run) for run in runs[:SIZES]), "|"))
    check("every run of odd numbers adds up to its cube, to 200", all(sum(run) == k**3 and run[0] == k * (k - 1) + 1 for k, run in enumerate(runs, 1)))
    # 06_5 redraws the square as thin layers coloured in those runs: the thin
    # layers of each run must be exactly the cells of that cube's thick layer.
    thin, same = 0, True
    for k, run in enumerate(runs[:SIZES], 1):
        cells = set()
        for size in run:
            thin += 1
            same = same and len(l_layer(thin)) == size
            cells |= set(l_layer(thin))
        thick = {(x, y) for x0, y0, wide, tall, _, _ in record[k]["pieces"] for x in range(x0, x0 + wide) for y in range(y0, y0 + tall)}
        same = same and cells == thick
    check("each cube's thick layer is its run of thin layers, cell for cell", same)
    emit("F-K.tiled.through", limit)

    totals = {"cubes": 0, "staircase": 0}

    def summed(n):
        totals["cubes"] += n * n * n
        totals["staircase"] += n
        return None if totals["cubes"] == totals["staircase"] ** 2 else f"the cubes to {n} add up to {totals['cubes']}"

    reach("F-K.sum.through", summed)


# --- F-S: three stacks and a box -------------------------------------------------


def one_stack(n):
    return [(i, j, level) for level in range(n) for i in range(n - level) for j in range(n - level)]


def placed(n):
    """The three stacks in the box, straight from the fact sheet's inequalities."""
    stacks = ([], [], [])
    for x in range(n):
        for y in range(n + 1):
            for z in range(n + 2):
                if z < n and x < n - z and y < n - z:
                    stacks[0].append((x, y, z))
                if 1 <= y <= n and x < y and n - y <= z < n:
                    stacks[1].append((x, y, z))
                if y <= x < n and n - x <= z <= n:
                    stacks[2].append((x, y, z))
    return stacks


def rigid_motions():
    """The 48 ways to turn or mirror the grid onto itself, as matrices."""
    for order in itertools.permutations(range(3)):
        for signs in itertools.product((1, -1), repeat=3):
            yield tuple(tuple(signs[r] if c == order[r] else 0 for c in range(3)) for r in range(3))


def determinant(m):
    return (
        m[0][0] * (m[1][1] * m[2][2] - m[1][2] * m[2][1])
        - m[0][1] * (m[1][0] * m[2][2] - m[1][2] * m[2][0])
        + m[0][2] * (m[1][0] * m[2][1] - m[1][1] * m[2][0])
    )


def settled(cells, m=None):
    """The cells moved by m and slid so that their least corner is the origin."""
    if m is not None:
        cells = [tuple(sum(row[c] * cell[c] for c in range(3)) for row in m) for cell in cells]
    low = [min(cell[c] for cell in cells) for c in range(3)]
    return frozenset(tuple(cell[c] - low[c] for c in range(3)) for cell in cells)


def turns_onto(n, stacks):
    """For each placed stack, the rigid motions that carry one stack onto it."""
    images = [(m, settled(one_stack(n), m)) for m in rigid_motions()]
    return [[m for m, image in images if image == settled(stack)] for stack in stacks]


def block(n):
    """The locked block of three n-layer stacks, measured: why it fails, or its numbers."""
    stacks = placed(n)
    cubes = Counter(cell for stack in stacks for cell in stack)
    if any(times != 1 for times in cubes.values()):
        return "two stacks share a cube", None
    wide = 1 + max(x for x, _, _ in cubes)
    deep = 1 + max(y for _, y, _ in cubes)
    levels = Counter(z for _, _, z in cubes)
    full = 0
    while levels[full] == wide * deep:
        full += 1
    top = [cell for cell in cubes if cell[2] >= full]
    if any(z != full for _, _, z in top):
        return "the top step is more than one cube thick", None
    return None, {"stacks": stacks, "wide": wide, "deep": deep, "full": full, "top": top, "cubes": cubes}


def sliced(n):
    """Fill the box in half-height cells: why it fails, or the block and its halves."""
    why, b = block(n)
    if why:
        return why, None
    filled, top_halves = Counter(), []
    for x, y, z in b["cubes"]:
        if z < b["full"]:
            filled[(x, y, 2 * z)] += 1
            filled[(x, y, 2 * z + 1)] += 1
        else:  # cut at half height: the upper half is turned half round into the gap
            lower, upper = (x, y, 2 * z), (n - 1 - x, n - y, 2 * z)
            top_halves += [lower, upper]
            filled[lower] += 1
            filled[upper] += 1
    b["halves"] = top_halves
    b["tall"] = F(1 + max(h for _, _, h in filled), 2)
    for cell in itertools.product(range(b["wide"]), range(b["deep"]), range(2 * n + 1)):
        if filled[cell] != 1:
            return f"half cell {cell} is filled {filled[cell]} times", None
    if len(filled) != b["wide"] * b["deep"] * (2 * n + 1):
        return "a half cell lies outside the box", None
    if (b["wide"], b["deep"], b["tall"]) != (n, n + 1, n + F(1, 2)):
        return f"the box is {by(b['wide'], b['deep'], b['tall'])}", None
    b["volume"] = F(len(filled), 2)
    return None, b


def stacks():
    n = STACK_LAYERS
    stack = one_stack(n)
    per_level = Counter(level for _, _, level in stack)
    emit("F-S.layers", join(per_level[level] for level in range(n)))
    emit("F-S.stack", len(stack))
    why, b = sliced(n)
    if why:
        sys.exit(f"oracle: the {n}-layer stacks do not fill their box: {why}")
    emit("F-S.three_stacks", sum(len(s) for s in b["stacks"]))
    emit("F-S.block", by(b["wide"], b["deep"], b["full"]))
    emit("F-S.box", by(b["wide"], b["deep"], b["tall"]))
    emit("F-S.box.volume", frac(b["volume"]))
    emit("F-S.top", len(b["top"]))
    emit("F-S.top.halves", len(b["halves"]))
    emit("F-S.top.layer", len({(x, y) for x, y, _ in b["cubes"]}))
    emit("F-S.sums", join(len(one_stack(k)) for k in range(1, SIZES + 1)))
    check("the box's volume is wide by deep by tall, and three stacks", b["volume"] == b["wide"] * b["deep"] * b["tall"] == 3 * len(stack))
    check("the top step is a staircase from the third stack, half of the top face", sorted(b["top"]) == sorted(c for c in b["stacks"][2] if c[2] == n) and 2 * len(b["top"]) == b["wide"] * b["deep"])
    check("the top step sits where y <= x", sorted((x, y) for x, y, _ in b["top"]) == sorted((x, y) for x in range(n) for y in range(x + 1)))

    # Turned or mirrored? Every rigid motion of the grid that lands one stack
    # on each placed stack; the best determinant among them is 1 exactly when
    # turning alone will do.
    onto = turns_onto(n, b["stacks"])
    emit("F-S.turns", join(max(determinant(m) for m in motions) for motions in onto))
    check("the first stack stays put", any(all(m[r][c] == (r == c) for r in range(3) for c in range(3)) for m in onto[0]))
    check("each stack is reached by one turn and by one mirror image (a stack is its own mirror image)", all(sorted(determinant(m) for m in motions) == [-1, 1] for motions in onto))
    for which, motions in enumerate(onto):
        for m in motions:
            if determinant(m) == 1:
                power, times = m, 1
                while any(power[r][c] != (r == c) for r in range(3) for c in range(3)):
                    power = tuple(tuple(sum(power[r][k] * m[k][c] for k in range(3)) for c in range(3)) for r in range(3))
                    times += 1
                note(f"stack {which} is the first stack turned by {m}: back where it started after {times} such turn(s)")

    def filled(size):
        why, b = sliced(size)
        if why:
            return why
        if any(len(s) != len(one_stack(size)) for s in b["stacks"]):
            return "the three stacks are not the same size"
        if not all(any(determinant(m) == 1 for m in motions) for motions in turns_onto(size, b["stacks"])):
            return "a stack is not a turned copy of the first"
        if 3 * len(one_stack(size)) != b["volume"]:
            return "three stacks are not the box's volume"
        return None

    reach("F-S.filled.through", filled)

    why, grown = block(n + 1)
    if why:
        sys.exit(f"oracle: the {n + 1}-layer stacks do not lock: {why}")
    emit("F-S.grown.box", by(grown["wide"], grown["deep"], grown["full"]))
    emit("F-S.grown.top", len(grown["top"]))
    slabs_at = {}

    def grows(size):
        """Each stack at this height, less itself one lower and lifted, leaves one flat slab."""
        now, before = placed(size), placed(size - 1)
        slabs = []
        for which in range(3):
            lifted = {(x, y, z + 1) for x, y, z in before[which]}
            if not lifted <= set(now[which]):
                return f"stack {which} lifted by one sticks out of itself one layer taller"
            slab = set(now[which]) - lifted
            flat = [axis for axis in range(3) if len({cell[axis] for cell in slab}) == 1]
            spans = [len({cell[axis] for cell in slab}) for axis in range(3)]
            if len(slab) != size * size or sorted(spans) != [1, size, size] or (size > 1 and len(flat) != 1):
                return f"what stack {which} gains is not a flat slab of side {size}"
            slabs.append(slab)
        under, behind, beside = slabs
        if {z for _, _, z in under} != {0} or {y for _, y, _ in behind} != {size} or {x for x, _, _ in beside} != {size - 1}:
            return "the new slabs are not under, behind and beside"
        if len(under | behind | beside) != 3 * size * size:
            return "two new slabs overlap"
        slabs_at[size] = len(under)
        return None

    for size in range(1, REACH["F-S.grows.through"] + 1):
        why = grows(size)
        if why:
            sys.exit(f"oracle: F-S.grows.through asked for {REACH['F-S.grows.through']} but fails at {size}: {why}")
    emit("F-S.grown.slab", slabs_at[n + 1])
    emit("F-S.grows.through", max(slabs_at))
    check("the grown block is one bigger in every direction", (grown["wide"], grown["deep"], grown["full"]) == (b["wide"] + 1, b["deep"] + 1, b["full"] + 1))

    why, one = sliced(1)
    if why:
        sys.exit(f"oracle: three single cubes do not fill their box: {why}")
    emit("F-S.one.cubes", len(one["cubes"]))
    emit("F-S.one.box", by(one["wide"], one["deep"], one["tall"]))

    # 07_2 asks for the fewest stacks that make a box when only the cubes that
    # stick out on top may be sliced. A stack of n levels is n by n by n in every
    # direction, so such a box (width by depth by h whole layers and a half layer)
    # has both sides at least n and at least n - 1 whole layers under the half;
    # it holds width * depth * (2h + 1) / 2 cubes, and m stacks have m of them.
    def half_boxes(m):
        cubes = m * sum(len(level) for level in ([(i, j) for i in range(n - l) for j in range(n - l)] for l in range(n)))
        found = []
        for width in range(n, 2 * cubes + 1):
            for depth in range(width, 2 * cubes // width + 1):
                if (2 * cubes) % (width * depth) == 0:
                    odd = 2 * cubes // (width * depth)
                    if odd % 2 == 1 and (odd - 1) // 2 >= n - 1:
                        found.append((width, depth, (odd - 1) // 2))
        return found

    fewest = next(m for m in range(1, 13) if half_boxes(m))
    emit("F-S.fewest", fewest)
    emit("F-S.fewest.box", join(f"{w}x{d}x{frac(F(2 * h + 1, 2))}" for w, d, h in half_boxes(fewest)))
    check("one and two stacks make no such box, and three make exactly Siu's", fewest == 3 and half_boxes(3) == [(n, n + 1, n)])

    running = [0]

    def summed(size):
        running[0] += size * size
        return None if 6 * running[0] == size * (size + 1) * (2 * size + 1) else f"the squares to {size} add up to {running[0]}"

    reach("F-S.sum.through", summed)
    share = F(b["volume"], len(stack))
    check("the box holds a whole number of stacks, the number placed in it", share.denominator == 1 and share == len(b["stacks"]))
    ANSWERS["stacks"] = frac(share)


# --- F-H: a square halved again and again ----------------------------------------


def rect_area(r):
    return (r[2] - r[0]) * (r[3] - r[1])


def halved(pieces):
    """The pieces as exact rectangles (x0, y0, x1, y1), and what is left."""
    left, out = (F(0), F(0), F(1), F(1)), []
    for k in range(1, pieces + 1):
        x0, y0, x1, y1 = left
        if k % 2:  # the left half of what is left
            middle = (x0 + x1) / 2
            piece, left = (x0, y0, middle, y1), (middle, y0, x1, y1)
        else:  # the bottom half
            middle = (y0 + y1) / 2
            piece, left = (x0, y0, x1, middle), (x0, middle, x1, y1)
        out.append(piece)
    return out, left


def halves():
    pieces, left = halved(HALVED_PIECES)
    areas = [rect_area(p) for p in pieces]
    emit("F-H.pieces", join(frac(a) for a in areas))
    emit("F-H.filled", join(frac(sum(areas[:k])) for k in range(1, HALVED_PIECES + 1)))
    emit("F-H.gap", frac(rect_area(left)))
    for name, parts in SMALL_GAPS:
        count = 1
        while rect_area(halved(count)[1]) >= F(1, parts):
            count += 1
        emit(f"F-H.pieces_to.{name}", count)
        emit(f"F-H.gap_at.{name}", frac(rect_area(halved(count)[1])))
        check(f"one piece fewer leaves a gap that is not under a {name}", rect_area(halved(count - 1)[1]) >= F(1, parts))

    def checked(n):
        pieces, left = halved(n)
        filled, gap = sum(rect_area(p) for p in pieces), rect_area(left)
        if filled + gap != 1:
            return f"filled {filled} and gap {gap} do not make the square"
        if gap != rect_area(pieces[-1]):
            return "the gap is not the size of the last piece"
        if not filled < 1:
            return "the square is full"
        if left[2:] != (1, 1):
            return "what is left has moved out of the top right corner"
        for p, q in itertools.combinations(pieces + [left], 2):
            if min(p[2], q[2]) > max(p[0], q[0]) and min(p[3], q[3]) > max(p[1], q[1]):
                return "two pieces overlap"
        return None

    # 10_4's note for programmers: a 64-bit float's running sum of the halves
    total, piece, pieces = 0.0, 0.5, 0
    while total < 1.0:
        total += piece
        piece /= 2
        pieces += 1
    emit("F-H.float.reaches_one", pieces)
    reach("F-H.checked.through", checked)


# --- F-R: a square quartered again and again -------------------------------------


def quartered(levels):
    """Each level's three kept squares and the corner still uncut, as (x, y, side)."""
    x, y, side, out = F(0), F(0), F(1), []
    for _ in range(levels):
        s = side / 2
        out.append({"cut": (x, y, side), "counted": (x, y, s), "others": [(x, y + s, s), (x + s, y, s)], "uncut": (x + s, y + s, s)})
        x, y, side = x + s, y + s, s
    return out


def quarters():
    levels = quartered(QUARTERED_LEVELS)
    counted = [lv["counted"][2] ** 2 for lv in levels]
    emit("F-R.pieces", join(frac(a) for a in counted))
    emit("F-R.share", join(frac(sum(counted[:k])) for k in range(1, QUARTERED_LEVELS + 1)))
    emit("F-R.left", join(frac(lv["uncut"][2] ** 2) for lv in levels))
    kept = {1 + len(lv["others"]) for lv in levels}
    check("every level keeps the same number of squares", len(kept) == 1)
    colours = kept.pop()
    emit("F-R.shares", colours)
    # The limit, two ways: the picture is itself again at the scale of its
    # uncut corner, so a total L obeys L = first piece + (corner's share) x L;
    # and the colours share equally all but a corner that shrinks to nothing.
    scale = levels[0]["uncut"][2] ** 2
    limit = counted[0] / (1 - scale)
    emit("F-R.limit", frac(limit))
    check("the limit is one share among the colours", limit == F(1, colours))
    check("each counted square is the last one at the corner's scale", all(b == a * scale for a, b in itertools.pairwise(counted)))
    emit("F-R.short", frac(limit - sum(counted)))
    ANSWERS["share"] = frac(limit)

    def checked(n):
        levels = quartered(n)
        last = levels[-1]
        s = last["counted"][2]
        if last["cut"][:2] != (1 - 2 * s, 1 - 2 * s) or s != F(1, 2**n):
            return "the square being cut is not where the fact sheet puts it"
        if {sq[2] for sq in [last["counted"], *last["others"], last["uncut"]]} != {s} or 4 * s * s != last["cut"][2] ** 2:
            return "the four squares are not equal quarters"
        share = sum(lv["counted"][2] ** 2 for lv in levels)
        if any(sum(lv["others"][i][2] ** 2 for lv in levels) != share for i in range(2)):
            return "the three colours hold different amounts"
        corner = last["uncut"][2] ** 2
        if colours * share + corner != 1:
            return "three shares and the corner do not make the square"
        if limit - share != corner / 3:
            return "a share is not short of a third by a third of the corner"
        return None

    reach("F-R.checked.through", checked)


# --- F-B: a square, a cube and a fourth power, cut --------------------------------


def cut(dimensions, long, short):
    """The blocks of the cut cube, each as (short sides, cells), or None.

    Every block is built as a product of stretches, the long one or the
    short one along each side, and its cells are marked in the cube; None
    when some cell of the cube is not marked exactly once.
    """
    side = long + short
    marks = dict.fromkeys(itertools.product(range(side), repeat=dimensions), 0)
    blocks = []
    for shorts in itertools.product((False, True), repeat=dimensions):
        stretches = [range(long, side) if is_short else range(long) for is_short in shorts]
        cells = 0
        for cell in itertools.product(*stretches):
            marks[cell] += 1
            cells += 1
        blocks.append((sum(shorts), cells))
    return blocks if all(times == 1 for times in marks.values()) else None


def cut_cube():
    long, short = PARTS
    emit("F-B.parts", join(PARTS))
    kinds = {}
    for name, dimensions in CUT_POWERS:
        blocks = cut(dimensions, long, short)
        if blocks is None:
            sys.exit(f"oracle: the cut {name} does not fill its cube")
        count = Counter(kind for kind, _ in blocks)
        sizes = {kind: {cells for k, cells in blocks if k == kind} for kind in count}
        check(f"blocks of one kind of the cut {name} are the same size", all(len(s) == 1 for s in sizes.values()))
        power = 1
        for _ in range(dimensions):
            power *= long + short
        kinds[name] = [count[kind] for kind in range(dimensions + 1)]
        emit(f"F-B.{name}.blocks", len(blocks))
        emit(f"F-B.{name}.kinds", join(kinds[name]))
        emit(f"F-B.{name}.each", join(min(sizes[kind]) for kind in range(dimensions + 1)))
        emit(f"F-B.{name}.by_kind", join(sum(cells for k, cells in blocks if k == kind) for kind in range(dimensions + 1)))
        emit(f"F-B.{name}.total", sum(cells for _, cells in blocks))
        emit(f"F-B.{name}.power", power)
        if name == "fourth":
            cube = kinds["cube"]
            emit("F-B.fourth.from_cube", join(f"{cube[j - 1]}+{cube[j]}" for j in range(1, len(cube))))
            check("each middle count of the fourth power is the sum of two neighbours of the cube's", [cube[j - 1] + cube[j] for j in range(1, len(cube))] == kinds[name][1:-1])
            twice = sorted([(k, c * long) for k, c in cut(3, long, short)] + [(k + 1, c * short) for k, c in cut(3, long, short)])
            check("every block of the cube appears twice in the fourth power", twice == sorted(blocks))
    ANSWERS["kinds"] = join(kinds["cube"])
    dimensions, most = CUT_SWEEP
    good = 0
    for d in range(1, dimensions + 1):
        for a in range(1, most + 1):
            for b in range(1, most + 1):
                blocks = cut(d, a, b)
                if blocks is not None and len(blocks) == 2**d and sum(cells for _, cells in blocks) == (a + b) ** d:
                    good += 1
    emit("F-B.filled.cuts", good)
    check("every cut tried fills its cube", good == dimensions * most * most)


# --- F-P: four triangles, two ways -----------------------------------------------


def four_triangles(a, b):
    """Everything chapter 13 draws for shorter sides a and b, and why it fails (or None)."""
    s = a + b
    square = box(0, 0, s, s)

    def quarter_turn(p):
        """A quarter turn about the big square's centre."""
        return (s - p[1], p[0])

    triangles = [[(0, 0), (a, 0), (0, b)]]
    for _ in range(3):
        triangles.append([quarter_turn(p) for p in triangles[-1]])
    sheet = [[(0, 0), (a, 0), (0, b)], [(s, 0), (s, a), (a, 0)], [(s, s), (b, s), (s, a)], [(0, s), (0, b), (b, s)]]
    out = {"big": area(square), "triangle": area(triangles[0])}
    if triangles != sheet:
        return out, "quarter turns of the first triangle are not the fact sheet's four"
    for t in triangles:
        legs = [(t[1][0] - t[0][0], t[1][1] - t[0][1]), (t[2][0] - t[0][0], t[2][1] - t[0][1])]
        if t[0] not in square or legs[0][0] * legs[1][0] + legs[0][1] * legs[1][1] != 0:
            return out, "a triangle's right angle is not at a corner of the big square"
    # the tilted shape: the points where two triangles meet, in turn
    tilted = [(a, 0)]
    for _ in range(3):
        tilted.append(quarter_turn(tilted[-1]))
    met = Counter(p for t in triangles for p in t)
    if sorted(tilted) != sorted(p for p, times in met.items() if times == 2):
        return out, "the tilted shape's corners are not where the triangles meet"
    sides = [(q[0] - p[0], q[1] - p[1]) for p, q in zip(tilted, tilted[1:] + tilted[:1])]
    out["tilted"] = area(tilted)
    if {x * x + y * y for x, y in sides} != {a * a + b * b}:
        return out, "the tilted shape's sides are not equal"
    if any(u[0] * v[0] + u[1] * v[1] != 0 for u, v in zip(sides, sides[1:] + sides[:1])):
        return out, "the tilted shape has a corner that is not a right angle"
    if sorted(sorted(e) for e in zip(tilted, tilted[1:] + tilted[:1])) != sorted(sorted(long_side(t)) for t in triangles):
        return out, "the tilted shape's sides are not the triangles' long sides"
    if out["tilted"] != a * a + b * b or out["tilted"] != out["big"] - 4 * out["triangle"]:
        return out, f"the tilted shape's area is {out['tilted']}"
    first = triangles + [tilted]
    if sum(area(p) for p in first) != out["big"] or any(shared(p, q) != 0 for p, q in itertools.combinations(first, 2)):
        return out, "the first arrangement does not tile the big square"
    # the slides: opposite triangles close into a rectangle, the first pair in
    # the lower left corner (where the first already is), the other pair in
    # the upper right; each triangle goes where its own bounding box fits
    slides = []
    for which, t in enumerate(triangles):
        x0, y0, x1, y1 = bounds(t)
        target = (0, 0) if which % 2 == 0 else (s - (x1 - x0), s - (y1 - y0))
        slides.append((target[0] - x0, target[1] - y0))
    out["slides"] = slides
    if slides != [(0, 0), (0, b), (-b, -a), (a, 0)]:
        return out, "the slides are not the fact sheet's"
    moved = [slid(t, *by_) for t, by_ in zip(triangles, slides)]
    rectangles = [box(0, 0, a, b), box(a, b, s, s)]
    for rectangle, pair in zip(rectangles, ((moved[0], moved[2]), (moved[1], moved[3]))):
        inside = all(shared(t, rectangle) == area(t) for t in pair)
        if not inside or shared(*pair) != 0 or sum(area(t) for t in pair) != area(rectangle) or area(rectangle) != a * b:
            return out, "a pair of slid triangles does not close into an a by b rectangle"
    squares = [box(0, b, a, s), box(a, 0, s, b)]
    out["squares"] = squares
    for sq, leg in zip(squares, (a, b)):
        x0, y0, x1, y1 = bounds(sq)
        if (x1 - x0, y1 - y0) != (leg, leg):
            return out, "what is left is not a square on each shorter side"
    second = moved + squares
    if sum(area(p) for p in second) != out["big"] or any(shared(p, q) != 0 for p, q in itertools.combinations(second, 2)):
        return out, "the second arrangement does not tile the big square"
    if any(shared(p, square) != area(p) for p in first + second):
        return out, "a piece sticks out of the big square"
    if area(squares[0]) + area(squares[1]) != out["tilted"]:
        return out, "the two squares are not the tilted square"
    out["moved"] = moved
    return out, None


def left_over(a, b, moved):
    """The unit squares of the big square that no slid triangle touches, in connected lumps."""
    s = a + b
    free = {(x, y) for x in range(s) for y in range(s) if all(shared(unit(x, y), t) == 0 for t in moved)}
    lumps = []
    while free:
        lump, edge = set(), [free.pop()]
        while edge:
            x, y = edge.pop()
            lump.add((x, y))
            for near in ((x + 1, y), (x - 1, y), (x, y + 1), (x, y - 1)):
                if near in free:
                    free.remove(near)
                    edge.append(near)
        lumps.append(lump)
    return lumps


def pythagoras():
    a, b = LEGS
    drawn, why = four_triangles(a, b)
    if why:
        sys.exit(f"oracle: shorter sides {a} and {b}: {why}")
    emit("F-P.legs", join(LEGS))
    emit("F-P.big", frac(drawn["big"]))
    emit("F-P.triangle", frac(drawn["triangle"]))
    emit("F-P.triangles", frac(4 * drawn["triangle"]))
    emit("F-P.tilted", frac(drawn["tilted"]))
    lumps = left_over(a, b, drawn["moved"])
    whole = []
    for lump in lumps:
        xs, ys = {x for x, _ in lump}, {y for _, y in lump}
        whole.append(len(xs) == len(ys) and len(lump) == len(xs) * len(ys))
    check("what the slid triangles leave is two whole squares", len(lumps) == 2 and all(whole))
    sizes = sorted((len(lump) for lump in lumps), key=lambda cells: LEGS.index(math.isqrt(cells)))
    emit("F-P.squares", join(sizes))
    check("the two squares left are the fact sheet's", sorted(sizes) == sorted(area(sq) for sq in drawn["squares"]))
    side = math.isqrt(int(drawn["tilted"]))
    emit("F-P.long_side", side if side * side == drawn["tilted"] else "none")
    emit("F-P.slides", join(f"{x} {y}" for x, y in drawn["slides"]))
    ANSWERS["tilted"] = frac(drawn["tilted"])

    other, why = four_triangles(*OTHER_LEGS)
    if why:
        sys.exit(f"oracle: shorter sides {OTHER_LEGS}: {why}")
    emit("F-P.other.legs", join(OTHER_LEGS))
    emit("F-P.other.tilted", frac(other["tilted"]))
    side = math.isqrt(int(other["tilted"]))
    emit("F-P.other.whole_side", side if side * side == other["tilted"] else "none")

    def checked(n):
        for m in range(1, n + 1):
            for pair in {(m, n), (n, m)}:
                _, why = four_triangles(*pair)
                if why:
                    return f"shorter sides {pair}: {why}"
        return None

    reach("F-P.checked.through", checked)


# --- F-L: the picture that lies ---------------------------------------------------


def rows(spec):
    return {(x, y) for y, first, last in spec for x in range(first, last + 1)}


def arrangement(big, small, l7, l8, wide, tall):
    """One arrangement, measured cell by cell: its bend, outline, holes, and why it is unsound."""
    low, far, high = (0, 0), (wide, 0), (wide, tall)
    common = set(big) & set(small)
    if len(common) != 1:
        return None, "the two triangles do not meet at one corner"
    bend = common.pop()
    if {frozenset(long_side(big)), frozenset(long_side(small))} != {frozenset((low, bend)), frozenset((bend, high))}:
        return None, "the long sides do not run from corner to bend to corner"
    outline = [low, far, high, bend]
    parts = [[low, far, bend], [far, high, bend]]  # the outline in two triangles, split at the bend
    squares = [unit(x, y) for x, y in sorted(l7 | l8)]
    if l7 & l8 or shared(big, small) != 0 or any(shared(sq, t) != 0 for sq in squares for t in (big, small)):
        return None, "two pieces overlap"
    if any(sum(shared(piece, part) for part in parts) != area(piece) for piece in [big, small, *squares]):
        return None, "a piece sticks out of the outline"
    holes = [
        (x, y)
        for x in range(wide)
        for y in range(tall)
        if (x, y) not in l7 | l8
        and sum(shared(unit(x, y), part) for part in parts) == 1
        and shared(unit(x, y), big) == 0
        and shared(unit(x, y), small) == 0
    ]
    pieces = area(big) + area(small) + len(l7) + len(l8)
    if area(outline) - pieces != len(holes):
        return None, "the pieces and the empty squares do not fill the outline"
    return {"bend": bend, "outline": outline, "holes": holes, "parts": parts}, None


def height(bend, wide, tall, x):
    """How high the bent edge from (0, 0) through the bend to (wide, tall) is at x."""
    if x <= bend[0]:
        return F(bend[1], bend[0]) * x
    return bend[1] + F(tall - bend[1], wide - bend[0]) * (x - bend[0])


def lie():
    big, small, l7, l8 = list(LIE_BIG), list(LIE_SMALL), rows(LIE_L7), rows(LIE_L8)
    corners = big + small + [(x + dx, y + dy) for x, y in l7 | l8 for dx in (0, 1) for dy in (0, 1)]
    x0, y0, wide, tall = bounds(corners)
    check("the pieces sit in the corner of the grid", (x0, y0) == (0, 0))
    areas = [area(big), area(small), len(l8), len(l7)]
    emit("F-L.pieces", join(frac(a) for a in areas))
    emit("F-L.pieces.total", frac(sum(areas)))
    emit("F-L.frame", by(wide, tall))
    true_triangle = [(0, 0), (wide, 0), (wide, tall)]
    emit("F-L.true_triangle", frac(area(true_triangle)))
    slopes = []
    for triangle in (big, small):
        p, q = long_side(triangle)
        slopes.append(F(q[1] - p[1], q[0] - p[0]))
    straight = F(tall, wide)
    emit("F-L.slopes", join(frac(s) for s in slopes))
    emit("F-L.straight_slope", frac(straight))

    first, why = arrangement(big, small, l7, l8, wide, tall)
    if why:
        sys.exit(f"oracle: the lying picture's first arrangement: {why}")

    # The second arrangement, worked out. The triangles change ends along the
    # sloping edge: the small one's long side now starts at the low corner and
    # the big one's now ends at the high corner.
    slides = {}
    end = min(long_side(small))
    slides["small"] = (0 - end[0], 0 - end[1])
    end = max(long_side(big))
    slides["big"] = (wide - end[0], tall - end[1])
    big2, small2 = slid(big, *slides["big"]), slid(small, *slides["small"])
    common = set(big2) & set(small2)
    check("the slid triangles meet at one corner again", len(common) == 1)
    bend2 = next(iter(common))
    parts2 = [[(0, 0), (wide, 0), bend2], [(wide, 0), (wide, tall), bend2]]
    free = {
        (x, y)
        for x in range(wide)
        for y in range(tall)
        if sum(shared(unit(x, y), part) for part in parts2) == 1 and shared(unit(x, y), big2) == 0 and shared(unit(x, y), small2) == 0
    }
    # The L-shapes: every slide of each that lands it on free unit squares,
    # and every pair of those that do not collide.
    ways = []
    for d8 in itertools.product(range(-wide, wide + 1), range(-tall, tall + 1)):
        moved8 = {(x + d8[0], y + d8[1]) for x, y in l8}
        if not moved8 <= free:
            continue
        for d7 in itertools.product(range(-wide, wide + 1), range(-tall, tall + 1)):
            moved7 = {(x + d7[0], y + d7[1]) for x, y in l7}
            if moved7 <= free and not moved7 & moved8:
                ways.append((d8, d7, moved8, moved7))
    if len(ways) != 1:
        sys.exit(f"oracle: the two L-shapes slide into the space the triangles leave in {len(ways)} ways, not one")
    slides["l8"], slides["l7"], l8_2, l7_2 = ways[0]
    second, why = arrangement(big2, small2, l7_2, l8_2, wide, tall)
    if why:
        sys.exit(f"oracle: the lying picture's second arrangement: {why}")

    emit("F-L.outline.first", frac(area(first["outline"])))
    emit("F-L.outline.second", frac(area(second["outline"])))
    sliver = [(0, 0), second["bend"], (wide, tall), first["bend"]]
    emit("F-L.sliver", frac(area(sliver)))
    check("the sliver is the difference of the two outlines", area(sliver) == area(second["outline"]) - area(first["outline"]))

    def gap(x):
        """How far the second bent edge is above the first at x, measured straight up."""
        return height(second["bend"], wide, tall, x) - height(first["bend"], wide, tall, x)

    # the gap is straight between bends, so it is greatest at a bend
    bends = sorted({0, first["bend"][0], second["bend"][0], wide})
    thickest = max(gap(x) for x in bends)
    emit("F-L.sliver.thickest", frac(thickest))
    check("the gap is never greater at any eighth of a square along the edge", all(gap(F(k, 8)) <= thickest for k in range(8 * wide + 1)))
    check("the gap is at its thickest all the way between the two bends", all(gap(F(k, 8)) == thickest for k in range(8 * bends[1], 8 * bends[2] + 1)))
    check("the gap, added up along the edge, is the sliver", sum((gap(p) + gap(q)) / 2 * (q - p) for p, q in itertools.pairwise(bends)) == area(sliver))
    check("the second bent edge is never below the first", all(gap(F(k, 8)) >= 0 for k in range(8 * wide + 1)))
    emit("F-L.holes.first", len(first["holes"]))
    emit("F-L.holes.second", len(second["holes"]))
    emit("F-L.hole.at", join((f"{x} {y}" for x, y in second["holes"]), ";"))
    dip = straight * first["bend"][0] - first["bend"][1]
    bulge = second["bend"][1] - straight * second["bend"][0]
    emit("F-L.dip", frac(dip))
    emit("F-L.bulge", frac(bulge))
    emit("F-L.slides", join(f"{x} {y}" for x, y in (slides["big"], slides["small"], slides["l8"], slides["l7"])))

    # The sliver cut along the grid: its overlap with every unit square, exact,
    # and the whole-square slide that takes each piece into the one empty square.
    def clip(poly, ax, ay, c):
        """The part of a convex polygon with ax * x + ay * y <= c."""
        kept = []
        for k, p in enumerate(poly):
            r = poly[(k + 1) % len(poly)]
            fp, fr = ax * p[0] + ay * p[1] - c, ax * r[0] + ay * r[1] - c
            if fp <= 0:
                kept.append(p)
            if (fp < 0 < fr) or (fr < 0 < fp):
                t = F(fp) / (fp - fr)
                kept.append((p[0] + t * (r[0] - p[0]), p[1] + t * (r[1] - p[1])))
        return kept

    def overlap(a, b):
        """The area two convex polygons share."""
        sign = 1 if shoelace(b) > 0 else -1
        piece = list(a)
        for k, p in enumerate(b):
            r = b[(k + 1) % len(b)]
            # keep what is on the inside of the edge p to r
            ax, ay = sign * (r[1] - p[1]), -sign * (r[0] - p[0])
            piece = clip(piece, ax, ay, ax * p[0] + ay * p[1])
            if len(piece) < 3:
                return F(0)
        return area(piece)

    sliver_corners = [(F(x), F(y)) for x, y in sliver]
    cut = []
    for cy in range(tall):
        for cx in range(wide):
            piece = sliver_corners
            for ax, ay, c in ((-1, 0, -cx), (1, 0, cx + 1), (0, -1, -cy), (0, 1, cy + 1)):
                piece = clip(piece, ax, ay, c)
                if len(piece) < 3:
                    break
            if len(piece) >= 3 and area(piece) > 0:
                cut.append(((cx, cy), piece))
    hx, hy = second["holes"][0]
    fold = [(hx - cx, hy - cy) for (cx, cy), _ in cut]
    emit("F-L.sliver.pieces", len(cut))
    emit("F-L.sliver.fold", join(f"{dx} {dy}" for dx, dy in fold))
    moved = [[(x + dx, y + dy) for x, y in piece] for ((_, piece), (dx, dy)) in zip(cut, fold)]
    check("the sliver's pieces add up to the sliver", sum(area(piece) for _, piece in cut) == area(sliver))
    check("slid by whole squares, every piece lies inside the empty square", all(hx <= x <= hx + 1 and hy <= y <= hy + 1 for piece in moved for x, y in piece))
    check("slid into the empty square, no two pieces overlap", all(overlap(a, b) == 0 for a, b in itertools.combinations(moved, 2)))
    check("the overlap measure is not vacuous: a piece overlaps itself fully", overlap(moved[0], moved[0]) == area(moved[0]) > 0)
    emit("F-L.slopes.thousandths", join(frac(1000 * s) for s in slopes))
    emit("F-L.slopes.differ", frac(slopes[1] - slopes[0]))
    on_the_edge = sum(1 for bend in (first["bend"], second["bend"]) if turn_of((0, 0), (wide, tall), bend) == 0)
    emit("F-L.bends.on_the_edge", on_the_edge)
    check("the pieces are 32 and a half less a half: half a square short of a true triangle", area(true_triangle) - sum(areas) == F(1, 2))
    check("the true triangle lies half way between the two outlines", area(true_triangle) - area(first["outline"]) == area(second["outline"]) - area(true_triangle))
    check("the first bend is below the straight edge and the second above it", dip > 0 and bulge > 0)
    check("the straight edge's slope lies between the two triangles' slopes", min(slopes) < straight < max(slopes))
    check("the bends are the fact sheet's", (first["bend"], second["bend"]) == ((8, 3), (5, 2)))
    ANSWERS["slopes"] = ("big", "small")[slopes.index(max(slopes))]
    check("one triangle is steeper than the other", slopes[0] != slopes[1])


# --- F-Q: the quizzes --------------------------------------------------------------


def quizzes():
    emit("F-Q.count", len(QUIZZES))
    emit("F-Q.names", join(name for name, _, _ in QUIZZES))
    for name, line, seconds in QUIZZES:
        emit(f"F-Q.{name}.line", line)
        emit(f"F-Q.{name}.seconds", seconds)
        emit(f"F-Q.{name}.answer", ANSWERS[name])
    # Every countdown in the storyboard, when this file sits beside it.
    storyboard = pathlib.Path(__file__).resolve().parent / "docs" / "storyboard.md"
    if not storyboard.is_file():
        note("docs/storyboard.md is not here: the quiz lines and seconds were not compared with it")
        return
    found = []
    for row in storyboard.read_text(encoding="utf-8").splitlines():
        asked = re.match(r"\|\s*(\d\d_\d+)\s*\|", row)
        ring = re.search(r"listen (\d+) s \(countdown\)", row)
        if asked and ring:
            found.append((asked.group(1), int(ring.group(1))))
    check("the storyboard's countdowns are these quizzes, line for line and second for second", found == [(line, seconds) for _, line, seconds in QUIZZES])


# --- F-X: cited numbers -------------------------------------------------------------


def cited():
    """Years and pages as the Sources table of docs/fact-sheet.md gives them."""
    # S1: Leo Moser, "On the Danger of Induction", Mathematics Magazine 23(2), 1949, p. 109
    emit("F-X.moser", join((1949, 109)))
    # S2: Nicomachus, Introduction to Arithmetic (about 100 CE), II.20.5
    nicomachus = 100
    emit("F-X.nicomachus", join((nicomachus, "II.20.5")))
    # S4: Aryabhata, Aryabhatiya (499 CE), Ganitapada stanza 22
    emit("F-X.aryabhata", join((499, 22)))
    # S3: Heath on al-Karaji, "tenth-eleventh century": about the year 1000
    al_karaji = 1000
    emit("F-X.al_karaji", al_karaji)
    # S5: Alan L. Fry, Mathematics Magazine 58(1), 1985, p. 11
    emit("F-X.fry", join((1985, 11)))
    # S8: Man-Keung Siu, Mathematics Magazine 57(2), 1984, p. 92
    emit("F-X.siu", join((1984, 92)))
    # S9: Warren Page, Mathematics Magazine 54(4), 1981, p. 201
    emit("F-X.page", join((1981, 201)))
    # S10: Ajose and Nelsen, Mathematics Magazine 67(3), 1994, p. 230
    emit("F-X.ajose_nelsen", join((1994, 230)))
    # S12: Euclid, Elements, Book II, Proposition 4
    emit("F-X.euclid", "II.4")
    # S14: Gardner, Mathematics, Magic and Mystery (1956), pp. 139 and 145; Curry's idea, 1953
    curry, gardner = 1953, 1956
    emit("F-X.curry", curry)
    emit("F-X.gardner", join((gardner, 139, 145)))
    # S15: Nelsen, Proofs Without Words (1993) and Proofs Without Words II (2000)
    emit("F-X.nelsen", join((1993, 2000)))
    emit("F-X.al_karaji.after_nicomachus", al_karaji - nicomachus)
    emit("F-X.gardner.after_curry", gardner - curry)


def main():
    circle()
    layers()
    staircases()
    cubes()
    stacks()
    halves()
    quarters()
    cut_cube()
    pythagoras()
    lie()
    quizzes()
    cited()
    print(f"# {len(_EMITTED)} facts, {_CHECKS[0]} cross-checks, {len(_FAILED)} failed", file=sys.stderr)
    if _FAILED:
        for name in _FAILED:
            print(f"# FAILED: {name}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
