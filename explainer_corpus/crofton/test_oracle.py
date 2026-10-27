#!/usr/bin/env python3
"""oracle.py's counting and averaging rules, at their edges and on things
that must fail: each negative control changes one assumption the video
relies on and checks that the claimed identity breaks for that reason.

    uv run --no-project --quiet --with numpy python3 -I -B test_oracle.py
"""
import math
import re
import sys
import unittest
from fractions import Fraction
from pathlib import Path

sys.dont_write_bytecode = True
HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
import numpy as np  # noqa: E402

import oracle as o  # noqa: E402

GEOM = o.build(o.COAST_START, o.COAST_HEADING, o.COAST)
STRAIGHT, PI_PART = o.exact_length(o.COAST)
LENGTH = float(STRAIGHT) + float(PI_PART) * math.pi


def circle(r, centre=(0.0, 0.0)):
    """A full circle as one arc piece, starting at its lowest point."""
    return [("A", centre, r, -math.pi / 2, 3 * math.pi / 2)]


SQUARE = [("S", (-1.0, -1.0), (1.0, -1.0)), ("S", (1.0, -1.0), (1.0, 1.0)), ("S", (1.0, 1.0), (-1.0, 1.0)), ("S", (-1.0, 1.0), (-1.0, -1.0))]


def mean_over_slides(geom, beta, gap=o.GAP, slides=2000):
    return sum(o.crossings(geom, beta, gap * (i + 0.5) / slides, gap) for i in range(slides)) / slides


class Counting(unittest.TestCase):
    def test_a_stick_along_the_lines_never_crosses(self):
        for y in (0.3, 0.7, 1.9):
            self.assertEqual(o.crossings(o.stick_geom(6.0, 0, (0.0, y)), 0, 0.0), 0)

    def test_a_stick_across_crosses_its_length_over_the_gap_give_or_take_one(self):
        stick = o.stick_geom(6.0, 90)
        seen = {o.crossings(stick, 0, 2 * (i + 0.5) / 400) for i in range(400)}
        self.assertEqual(seen, {3})  # exactly 3 gaps long: always 3, never 2 or 4
        longer = o.stick_geom(7.0, 90)
        self.assertEqual({o.crossings(longer, 0, 2 * (i + 0.5) / 400) for i in range(400)}, {3, 4})

    def test_crossings_do_not_depend_on_which_way_a_piece_is_walked(self):
        p, q = (-1.0, -2.3), (2.0, 3.1)
        for beta in (0, 30, 90, 150):
            self.assertEqual(o.crossings([("S", p, q)], beta, 0.4), o.crossings([("S", q, p)], beta, 0.4))

    def test_an_arc_that_turns_back_is_counted_on_both_sides(self):
        # a half circle of radius 3 bulging up from y = 0 crosses each line
        # below its top twice
        half = [("A", (0.0, 0.0), 3.0, 0.0, math.pi)]
        self.assertEqual(o.crossings(half, 0, 0.5), 4)  # lines at y = 0.5 and 2.5
        self.assertEqual(o.crossings(half, 0, 1.5), 2)  # y = 1.5 only; 3.5 is above
        # ...and the floor-only count agrees with solving the circle directly
        for s in (0.1, 0.5, 1.3, 1.9):
            direct = sum(2 for k in range(-3, 4) if 0 < s + 2 * k < 3.0)
            self.assertEqual(o.crossings(half, 0, s), direct)

    def test_a_coin_of_the_gap_always_crosses_twice_and_others_do_not(self):
        for d in (0.0, 0.3, 0.9, 1.7):  # 1.0 would touch two lines at once
            self.assertEqual(o.crossings(circle(1.0, (0.2, d)), 0, 0.0), 2)
            self.assertEqual(o.crossings(circle(1.0, (0.2, d)), 63, 0.0), 2)
        small = {o.crossings(circle(0.75, (0.0, 2 * (i + 0.5) / 200)), 0, 0.0) for i in range(200)}
        big = {o.crossings(circle(1.5, (0.0, 2 * (i + 0.5) / 200)), 0, 0.0) for i in range(200)}
        self.assertEqual(small, {0, 2})
        self.assertEqual(big, {2, 4})

    def test_the_coin_edge_is_where_the_count_changes(self):
        # just inside and just outside the gap
        self.assertEqual({o.crossings(circle(0.999), 0, s) for s in (0.0005, 1.0)}, {0, 2})
        self.assertEqual({o.crossings(circle(1.001), 0, s) for s in (0.0005, 1.0)}, {2, 4})

    def test_a_coin_as_wide_as_the_gap_meets_the_lines_in_two_points_even_when_it_grazes(self):
        # exact arithmetic: lines at heights 2k, a coin of radius 1 centred at
        # height c meets a line in 2 points if it is closer than 1, in 1 point
        # if exactly 1 away; the "always two" holds at the grazing placements
        for c in [Fraction(k, 64) for k in range(0, 129)]:
            points = 0
            for k in range(-3, 4):
                gap = abs(c - 2 * k)
                points += 2 if gap < 1 else (1 if gap == 1 else 0)
            self.assertEqual(points, 2, c)
        # a coin a hair wider meets them in four points when it grazes
        wide = Fraction(1001, 1000)
        self.assertEqual(sum(2 if abs(Fraction(1) - 2 * k) < wide else 0 for k in range(-3, 4)), 4)

    def test_clearance_is_zero_on_a_tangent_line(self):
        self.assertAlmostEqual(o.clearance(circle(1.0), 0, 1.0), 0.0, places=12)
        self.assertAlmostEqual(o.clearance(circle(1.0), 0, 0.0), 1.0, places=12)


class Averages(unittest.TestCase):
    def test_sliding_a_gap_averages_travel_over_gap_for_any_curve(self):
        for geom in (GEOM, o.stick_geom(6.0, 30), circle(1.4, (0.3, 0.1))):
            for beta in (0, 20, 111):
                self.assertAlmostEqual(mean_over_slides(geom, beta), o.travel(geom, beta) / o.GAP, delta=4e-3)

    def test_a_circle_travels_twice_its_width_in_every_direction(self):
        for beta in (0, 17, 45, 90, 170):
            self.assertAlmostEqual(o.travel(circle(1.3), beta), 4 * 1.3, places=12)

    def test_crofton_holds_for_sticks_circles_and_the_coast(self):
        for geom, length in ((o.stick_geom(5.0, 12), 5.0), (circle(0.9), 2 * math.pi * 0.9), (GEOM, LENGTH)):
            mean = o.simpson_mean(lambda b: o.travel(geom, b) / o.GAP, 0.0, 180.0, 3600)
            self.assertAlmostEqual(mean, 2 * length / (math.pi * o.GAP), places=6)

    def test_crofton_does_not_care_where_the_curve_sits(self):
        moved = o.build((Fraction(-5), Fraction(1)), 10, o.COAST)  # moved and turned
        a = o.simpson_mean(lambda b: o.travel(GEOM, b), 0.0, 180.0, 3600)
        b = o.simpson_mean(lambda b: o.travel(moved, b), 0.0, 180.0, 3600)
        self.assertAlmostEqual(a, b, places=6)

    def test_crossings_add_over_pieces(self):
        for beta in (0, 45, 90, 135):
            whole = o.travel(GEOM, beta)
            parts = sum(o.travel([g], beta) for g in GEOM)
            self.assertAlmostEqual(whole, parts, places=12)


class NegativeControls(unittest.TestCase):
    def test_one_direction_alone_is_not_crofton(self):
        # one direction, even slid through a whole gap, misses the length
        one = o.travel(GEOM, 0) / o.GAP * o.worth()
        self.assertGreater(abs(one - LENGTH), 1.0)

    def test_weighting_directions_unevenly_breaks_the_identity(self):
        # directions drawn with density proportional to 1 + cos(2 beta),
        # crowded near lines running left to right, give another average:
        # for a 6 cm stick lying left to right, 8/(pi x gap) instead of
        # 12/(pi x gap), two thirds of the fair count
        def skewed_mean(geom):
            def weighted(b):
                return o.travel(geom, b) / o.GAP * (1 + math.cos(math.radians(2 * b)))
            return o.simpson_mean(weighted, 0.0, 180.0, 3600)
        stick = o.stick_geom(6.0, 0)
        self.assertAlmostEqual(skewed_mean(stick), 8 / (math.pi * o.GAP), places=6)
        self.assertAlmostEqual(skewed_mean(stick) / (2 * 6.0 / (math.pi * o.GAP)), 2 / 3, places=6)
        # the coast wanders in every direction, so the skew shows far less
        # on it (about 1 percent) but does not vanish
        fair = 2 * LENGTH / (math.pi * o.GAP)
        self.assertGreater(abs(skewed_mean(GEOM) - fair), 0.05)
        self.assertLess(abs(skewed_mean(GEOM) - fair), 0.2)

    def test_two_pins_favour_the_middle_and_even_lines_do_not(self):
        middle = o.pin_rate_exact(o.MIDDLE_STICK)
        corner = o.pin_rate_exact(o.CORNER_STICK)
        self.assertGreater(middle / corner, 1.5)
        rng = np.random.Generator(np.random.PCG64DXSM(99))
        hits = [0, 0]
        for _ in range(20000):
            line = o.even_line(rng)
            hits[0] += o.crosses(line, o.MIDDLE_STICK)
            hits[1] += o.crosses(line, o.CORNER_STICK)
        rate = 2 * 2.0 / (2 * (o.CARD_W + o.CARD_H))
        se = math.sqrt(rate * (1 - rate) / 20000)
        for h in hits:
            self.assertLess(abs(h / 20000 - rate), 4 * se)

    def test_a_curve_that_is_not_of_constant_width_does_not_always_cross_twice(self):
        # a 2 cm square: its width runs from 2 to 2 x sqrt 2
        sq = [("S", (-1.0, -1.0), (1.0, -1.0)), ("S", (1.0, -1.0), (1.0, 1.0)), ("S", (1.0, 1.0), (-1.0, 1.0)), ("S", (-1.0, 1.0), (-1.0, -1.0))]
        seen = {o.crossings(sq, 45, 2 * (i + 0.5) / 200) for i in range(200)}
        self.assertEqual(seen, {2, 4})
        # yet its average still obeys Crofton: rim 8, so 2 x 8 / (pi x gap)
        mean = o.simpson_mean(lambda b: o.travel(sq, b) / o.GAP, 0.0, 180.0, 3600)
        self.assertAlmostEqual(mean, 2 * 8 / (math.pi * o.GAP), places=6)

    def test_a_finite_estimate_is_not_the_identity(self):
        # ten drops land near the length for this seed, but other seeds do not
        far = 0
        for seed in range(200):
            rng = np.random.Generator(np.random.PCG64DXSM(seed))
            ten = [o.crossings(GEOM, 180 * rng.random(), o.GAP * rng.random()) for _ in range(10)]
            if abs(sum(ten) / 10 * o.worth() - LENGTH) > 1.0:
                far += 1
        self.assertGreater(far, 40)
        self.assertLess(far, 160)


class TheCoast(unittest.TestCase):
    def test_the_exact_length_is_the_sum_of_its_pieces(self):
        self.assertEqual(STRAIGHT, Fraction(15, 2))
        self.assertEqual(PI_PART, Fraction(1399, 240))
        self.assertAlmostEqual(LENGTH, 25.812867177, places=8)

    def test_pieces_join_without_corners(self):
        # every piece starts where the last ended, heading the same way
        for a, b in zip(GEOM, GEOM[1:]):
            end = a[2] if a[0] == "S" else (a[1][0] + a[2] * math.cos(a[4]), a[1][1] + a[2] * math.sin(a[4]))
            start = b[1] if b[0] == "S" else (b[1][0] + b[2] * math.cos(b[3]), b[1][1] + b[2] * math.sin(b[3]))
            self.assertLess(math.dist(end, start), 1e-12)

    def test_every_shown_placement_clears_the_coast(self):
        for beta in o.SHOWN_DIRECTIONS:
            s = o.shown_slide(GEOM, beta)
            self.assertGreaterEqual(o.clearance(GEOM, beta, s), o.MARGIN)
            # and the count is stable under a small nudge of the sheet
            for nudge in (-0.05, 0.05):
                self.assertEqual(o.crossings(GEOM, beta, s + nudge), o.crossings(GEOM, beta, s))

    def test_shown_slide_picks_the_clearest_grid_point(self):
        for beta in o.SHOWN_DIRECTIONS:
            s = o.shown_slide(GEOM, beta)
            best = o.clearance(GEOM, beta, s)
            for k in range(40):
                self.assertLessEqual(o.clearance(GEOM, beta, k * 0.05), best + 1e-9)


class Scale(unittest.TestCase):
    def test_dividers_walk_a_straight_stick_exactly(self):
        for step in (0.5, 1.0, 2.5, 7.0):
            n, walked = o.dividers(o.stick_geom(6.0, 37), step, per_arc=8)
            self.assertEqual(n, int(6.0 // step))
            self.assertAlmostEqual(walked, 6.0, places=9)

    def test_dividers_on_a_circle_with_its_radius_walk_a_hexagon(self):
        # the walk follows chords of the circle 1/4096 of a turn long, so a
        # landing can sit 3e-7 cm inside the circle: the length is 6 to five
        # places, not 2 pi, and the step count is checked on a true hexagon
        _, walked = o.dividers(circle(1.0), 1.0)
        self.assertAlmostEqual(walked, 6.0, places=5)
        hexagon = [("S", (math.cos(math.pi * k / 3), math.sin(math.pi * k / 3)), (math.cos(math.pi * (k + 1) / 3), math.sin(math.pi * (k + 1) / 3))) for k in range(6)]
        n, walked = o.dividers(hexagon, 1.0)
        self.assertEqual(n, 6)  # the sixth step lands back at the start
        self.assertAlmostEqual(walked, 6.0, places=12)
        # a step a little longer than the radius leaves a last straight bit
        n, walked = o.dividers(circle(1.0), 1.1)
        self.assertEqual(n, 5)
        self.assertLess(walked, 2 * math.pi)

    def test_a_ripple_keeps_the_heading_and_the_ends(self):
        r = Fraction(1, 4)
        one = o.build((Fraction(0), Fraction(0)), 0, [("T", 90), ("A", r, -180), ("T", 90), ("S", Fraction(1))])
        # after the ripple the straight run still heads along x, from x = 1/2
        self.assertAlmostEqual(one[-1][1][0], 0.5, places=12)
        self.assertAlmostEqual(one[-1][1][1], 0.0, places=12)
        self.assertAlmostEqual(one[-1][2][1], 0.0, places=12)
        self.assertEqual(o.exact_length([("T", 90), ("A", r, -180)]), (0, Fraction(1, 4)))

    def test_the_sheet_sees_ripples_on_average_and_dividers_do_not(self):
        rgeom = o.build(o.COAST_START, o.COAST_HEADING, o.rippled_coast())
        s, p = o.exact_length(o.rippled_coast())
        r_length = float(s) + float(p) * math.pi
        mean = o.simpson_mean(lambda b: o.travel(rgeom, b) / o.GAP, 0.0, 180.0, 3600)
        self.assertAlmostEqual(mean * o.worth(), r_length, places=5)
        self.assertGreater(r_length - LENGTH, 1.9)
        _, smooth = o.dividers(GEOM, 2.0)
        _, rough = o.dividers(rgeom, 2.0)
        self.assertLess(abs(rough - smooth), 0.2)


class Spacing(unittest.TestCase):
    """The gap must cancel: a count goes as 1/gap and a crossing's worth as
    gap. Counts here come from sliding a sheet and counting."""

    GAPS = (0.5, 1.0, 2.0, 3.0, 4.0)

    def test_a_coin_as_wide_as_any_gap_calibrates_that_gap(self):
        for gap in self.GAPS:
            coin = circle(gap / 2)
            self.assertEqual({o.crossings(coin, 0, gap * (i + 0.5) / 97, gap) for i in range(97)}, {2})
            self.assertAlmostEqual(math.pi * gap / 2, o.worth(gap), places=12)

    def test_counts_times_worth_give_one_length_at_every_gap(self):
        stick = o.stick_geom(6.0, 20)
        directions = [180 * (j + 0.5) / 90 for j in range(90)]
        for gap in self.GAPS:
            ring = o.slid_mean(circle(1.0), 0, gap, 2000) * o.worth(gap)
            self.assertAlmostEqual(ring, 2 * math.pi, delta=0.01)
            turned = sum(o.slid_mean(stick, b, gap, 400) for b in directions) / len(directions) * o.worth(gap)
            self.assertAlmostEqual(turned, 6.0, delta=0.05)

    def test_pi_times_travel_over_gap_is_right_only_at_two(self):
        # negative control: the expression root's review found (Mail 6787)
        for gap in self.GAPS:
            flagged = math.pi * o.travel(circle(1.0), 0) / gap
            self.assertEqual(abs(flagged - 2 * math.pi) < 1e-9, gap == 2.0)

    def test_one_crossing_is_pi_cm_only_on_a_two_cm_sheet(self):
        # negative control: count x pi, the narration's shortcut for its own
        # 2 cm sheet, gives a unit circle 4 pi on a 1 cm sheet
        self.assertAlmostEqual(o.slid_mean(circle(1.0), 0, 1.0, 2000) * math.pi, 4 * math.pi, delta=0.02)
        self.assertAlmostEqual(o.slid_mean(circle(1.0), 0, 2.0, 2000) * math.pi, 2 * math.pi, delta=0.02)

    def test_enlarging_the_drawing_and_the_sheet_together_keeps_every_count(self):
        for k in (0.5, 3.0):
            big = o.scaled(GEOM, k)
            for beta in range(0, 180, 9):
                for s in (0.07, 0.61, 1.33, 1.91):
                    self.assertEqual(o.crossings(big, beta, k * s, k * o.GAP), o.crossings(GEOM, beta, s))
            self.assertAlmostEqual(o.chord_length(big, 4096), k * LENGTH, places=4)

    def test_enlarging_the_drawing_alone_multiplies_the_average_count(self):
        k = 3.0
        mean = o.simpson_mean(lambda b: o.travel(o.scaled(GEOM, k), b) / o.GAP, 0.0, 180.0, 3600)
        self.assertAlmostEqual(mean * o.worth(), k * LENGTH, places=5)
        self.assertAlmostEqual(mean, k * 2 * LENGTH / (math.pi * o.GAP), places=5)


class Distributions(unittest.TestCase):
    """`count_shares` (closed form between event directions) on shapes whose
    chances are known on paper, and the grids converging on it."""

    def assertShares(self, got, want, places=12):
        self.assertEqual(sorted(got), sorted(want), got)
        for k, p in want.items():
            self.assertAlmostEqual(got[k], p, places=places, msg=f"count {k}")

    def test_a_coin_as_wide_as_the_gap_always_crosses_twice(self):
        self.assertShares(o.count_shares(circle(1.0), 2.0), {2: 1.0})

    def test_a_short_stick_crosses_with_buffons_chance(self):
        # Buffon's needle: a stick shorter than the gap crosses one line with
        # chance 2 x length / (pi x gap), else none
        p = 2 * 1.5 / (math.pi * 2.0)
        self.assertShares(o.count_shares(o.stick_geom(1.5, 23), 2.0), {0: 1 - p, 1: p})

    def test_a_square_crosses_four_times_four_over_pi_minus_one_of_the_time(self):
        four = 4 / math.pi - 1
        self.assertShares(o.count_shares(SQUARE, 2.0), {2: 1 - four, 4: four})

    def test_single_lines_meet_a_circle_twice_over_its_rim(self):
        # every line meeting a circle crosses it twice; those lines fill the
        # chart over the circle's rim (Cauchy: a convex shape's perimeter)
        self.assertShares(o.count_shares(circle(1.3, (0.4, -0.2))), {2: 2 * math.pi * 1.3})

    def test_single_lines_on_the_coast_total_twice_its_length(self):
        lines = o.count_shares(GEOM)
        self.assertAlmostEqual(sum(k * w for k, w in lines.items()), 2 * LENGTH, places=9)
        self.assertEqual(max(lines), 6)
        meet = sum(w for k, w in lines.items() if k > 0)
        self.assertAlmostEqual(meet, o.hull_perimeter(o.sample_points(GEOM, 4096)), places=6)

    def test_the_grids_converge_on_the_closed_form(self):
        exact = o.count_shares(GEOM, 2.0)
        gaps = []
        for directions in (3600, 28800):
            grid = o.grid_shares(GEOM, directions, 2.0)
            self.assertEqual(sorted(grid), sorted(exact))
            gaps.append(max(abs(grid[k] - p) for k, p in exact.items()))
        self.assertLess(gaps[0], 1e-5)
        self.assertLess(gaps[1], gaps[0] / 4)

    def test_a_missing_event_is_refused_not_integrated(self):
        # negative control: with no event directions inside the half turn,
        # heights pass each other inside one range, and the guard says so
        real = o.event_directions
        try:
            o.event_directions = lambda fs, gap=None: [0.0, math.pi]
            with self.assertRaises(ValueError):
                o.count_shares(GEOM, 2.0)
        finally:
            o.event_directions = real

    def test_the_stretch_rule_is_exact_only_for_waves_of_one_turn(self):
        m, h = 0.7, 0.6

        def at(w):
            return [w(m - h / 2), w(m), w(m + h / 2)]
        wave = lambda t: 1.7 * math.cos(t) - 0.4 * math.sin(t) + 2.5  # noqa: E731
        exact = 1.7 * (math.sin(m + h) - math.sin(m - h)) + 0.4 * (math.cos(m + h) - math.cos(m - h)) + 2.5 * 2 * h
        self.assertAlmostEqual(o.stretch_integral(*at(wave), h, h / 2), exact, places=12)
        # negative control: a wave of half a turn is not in the family
        double = lambda t: math.cos(2 * t)  # noqa: E731
        self.assertGreater(abs(o.stretch_integral(*at(double), h, h / 2) - (math.sin(2 * (m + h)) - math.sin(2 * (m - h))) / 2), 1e-4)

    def test_a_seeded_sample_spread_is_not_the_methods_spread(self):
        # the thousand seeded drops spread 4.3 cm a drop; every placement
        # spreads 4.4: the film says the second (review F3)
        rng = np.random.Generator(np.random.PCG64DXSM(o.SEED_DROPS))
        drops = [o.crossings(GEOM, 180.0 * rng.random(), o.GAP * rng.random()) for _ in range(o.MANY_DROPS)]
        mean = sum(drops) / len(drops)
        sample = math.sqrt(sum((x - mean) ** 2 for x in drops) / (len(drops) - 1)) * o.worth()
        _, sd = o.moments(o.count_shares(GEOM, 2.0))
        self.assertEqual((o.fixed(sample, 1), o.fixed(sd * o.worth(), 1)), ("4.3", "4.4"))
        self.assertEqual(max(drops), 12)
        self.assertEqual(max(o.count_shares(GEOM, 2.0)), 13)


# What the storyboards show adds up as shown (AGENTS: what is said equals
# what is shown, rounded the same way). An equation is a run of numbers,
# pi and operators around = or ≈ in a picture column, after fact keys,
# quotes and units are taken out. "crossing" (singular) stays in, so a rate
# such as "1 crossing = π cm" is not read as arithmetic.
PLAIN = re.compile(r"\s*−?\d+(?:\.\d+)?%?\s*")
RUN = re.compile(r"[0-9.,π×÷+−/()⅙ =≈%]+")


def value(expr):
    py = expr.replace("×", "*").replace("÷", "/").replace("−", "-").replace("⅙", "(1/6)").replace("%", "")
    py = re.sub(r"(\d)\s*π", r"\1*pi", py).replace("π", "pi")
    if not re.fullmatch(r"[0-9.+\-*/() pi]*", py):
        raise ValueError(expr)
    return eval(py, {"__builtins__": {}}, {"pi": math.pi})  # only digits, operators and pi


def places(plain):
    s = plain.strip().rstrip("%")
    return len(s.split(".")[1]) if "." in s else 0


def rounds_to(x, plain):
    shown = plain.strip().replace("−", "-")
    return o.fixed(x * (100 if shown.endswith("%") else 1), places(plain)) == shown.rstrip("%")


def equations(picture):
    """Every (shown, follows) pair in a picture: an equation is wrong when its
    shown result is not what its shown operands give, rounded to the shown
    places, or when two exact sides differ."""
    text = re.sub(r"\(\s*`[^)]*\)", " ", picture)
    text = re.sub(r"`[^`]*`", " ", text).replace('"', " ")
    text = re.sub(r"\s*\b(?:cm|mm|crossings|steps|drops)\b", "", text)
    text = re.sub(r",\s*(?=[×÷+−])", " ", text)
    text = re.sub(r"(?<=\d),(?=\d{3})", "", text)
    out = []
    for run in RUN.findall(text):
        parts = re.split(r"[=≈]", run)
        for i, (left, right) in enumerate(zip(parts, parts[1:])):
            left, right = left.strip(" ,."), right.strip(" ,.")
            if not re.search(r"\d|π", left) or not re.search(r"\d|π", right):
                continue
            # a side cut off by a word ("gap × 6/π") is not a shown operand
            if any(re.match(r"[×÷+−/]", s) or re.search(r"[×÷+−/(]$", s) for s in (left, right)):
                continue
            # in a chain ("a ÷ 2 = b × π = c") a middle part's leading number
            # is the result so far, and the rest carries on to the next part
            lead = re.match(r"(−?\d+(?:\.\d+)?%?)\s+(?=[×÷+−])", right)
            if lead and i + 2 < len(parts):
                right = lead.group(1)
            if PLAIN.fullmatch(left) and PLAIN.fullmatch(right):
                continue
            if PLAIN.fullmatch(right):
                ok = rounds_to(value(left), right)
            elif PLAIN.fullmatch(left):
                ok = rounds_to(value(right), left)
            else:
                ok = abs(value(left) - value(right)) < 1e-9
            out.append((f"{left} = {right}", ok))
    return out


def picture_columns(path):
    for line in path.read_text().splitlines():
        cells = re.split(r"(?<!\\)\|", line)
        if len(cells) > 4 and re.fullmatch(r"\s*\d\d_\d+\s*", cells[1]):
            yield cells[1].strip(), cells[3]


class DisplayedArithmetic(unittest.TestCase):
    def test_every_shown_equation_follows_from_its_shown_operands(self):
        seen = 0
        for name in ("docs/storyboard.md", "docs/storyboard-draft.md"):
            for beat, picture in picture_columns(HERE / name):
                for eq, ok in equations(picture):
                    seen += 1
                    self.assertTrue(ok, f"{name} {beat}: {eq}")
        self.assertGreater(seen, 25)

    def test_the_review_s_mismatched_products_are_caught(self):
        # negative controls: the pictures before the review's F2 repair, and
        # the draft's old 01_5 sum
        for picture in (
            '"mean 8.13 × π = 25.53 cm" (`turn.slide_mean`, `turn.slide_estimate`)',
            '"16.43 ÷ 2 = 8.22 crossings, × π = 25.81 cm" (`crofton.mean_crossings`)',
            '"51.63" (`chart.coast_total`, `crofton.twice_length`) = 2 × 25.81 (`coast.length_2dp`)',
            "0° 15.26 ÷ 2 = 7.63; 45° 17.49 ÷ 2 = 8.74",
            '"11 steps + 0.64 cm = 22.6 cm" (`dividers.2_0.steps`)',
        ):
            found = equations(picture)
            self.assertTrue(found and not all(ok for _, ok in found), picture)

    def test_the_repaired_pictures_add_up(self):
        for picture in (
            '"8.127 × π = 25.53 cm"',
            '"16.433 ÷ 2 = 8.2165 crossings, × π = 25.81 cm"',
            '"51.63 = 2 × 25.813"',
            "0°: 15.260 ÷ 2 = 7.63; 45°: 17.485 ÷ 2 = 8.74",
            "average = 2 × 8 ÷ (π × 2) = 8/π ≈ 2.55; p = 4/π − 1 ≈ 27%",
        ):
            found = equations(picture)
            self.assertTrue(found and all(ok for _, ok in found), (picture, found))


class PinLines(unittest.TestCase):
    """18_4: the pins' crowd stays in the middle of the card. In every
    direction a chord's length across the card is even and concave in its
    distance from the centre (the card is convex and symmetric), so the chord
    weight, its cube, peaks through the centre; a stick centred there meets
    the most pin lines, and the same stick moved, without turning, fewer."""

    @staticmethod
    def stick(angle, centre, length=2.0):
        a = math.radians(angle)
        dx, dy = length / 2 * math.cos(a), length / 2 * math.sin(a)
        return ((centre[0] - dx, centre[1] - dy), (centre[0] + dx, centre[1] + dy))

    def test_moving_a_stick_off_the_middle_always_loses_pin_lines(self):
        for angle in (0, 30, 90):
            middle = o.pin_rate_exact(self.stick(angle, (0.0, 0.0)))
            for centre in ((1.0, 0.0), (0.0, 1.0), (3.0, 1.5), (-7.0, 3.0), (6.5, -3.4)):
                self.assertLess(o.pin_rate_exact(self.stick(angle, centre)), middle, (angle, centre))

    def test_moving_and_turning_together_can_gain(self):
        # why 18_4 says "without turning": lying across the middle, a stick
        # catches fewer pin lines than moved 3 cm right and stood upright
        lying = o.pin_rate_exact(self.stick(0, (0.0, 0.0)))
        moved_upright = o.pin_rate_exact(self.stick(90, (3.0, 0.0)))
        self.assertGreater(moved_upright, lying + 0.05)

    def test_shortchanged_means_against_the_middle_not_against_fair(self):
        # the film's corner stick still beats the fair even rate, 8%
        fair = 2 * 2.0 / (2 * (o.CARD_W + o.CARD_H))
        self.assertGreater(o.pin_rate_exact(o.CORNER_STICK), fair)


class Formatting(unittest.TestCase):
    def test_fixed_rounds_half_away_from_zero(self):
        self.assertEqual(o.fixed(2.25, 1), "2.3")
        self.assertEqual(o.fixed(-2.25, 1), "-2.3")
        self.assertEqual(o.fixed(0.04, 1), "0.0")
        self.assertEqual(o.fixed(-0.04, 1), "0.0")
        self.assertEqual(o.fixed(Fraction(41, 5), 1), "8.2")
        self.assertEqual(o.fixed(25.8128, 0), "26")


if __name__ == "__main__":
    unittest.main(verbosity=2)
