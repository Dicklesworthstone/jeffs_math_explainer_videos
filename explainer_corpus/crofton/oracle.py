#!/usr/bin/env python3
"""Facts oracle for "Measure a Curve Without Following It".

Run from this folder (`./x crofton facts-check` does, with numpy via uv):

    python3 -I -B oracle.py

stdout: one `KEY<TAB>VALUE` line per fact, the same keys `crofton facts`
prints (src/facts.rs). stderr: cross-checks (lines starting with "#") and a
summary; exit 1 if any cross-check failed.

The coast, the coin, the sticks and the card are data shared with the Rust
model (they are the video's inputs). How each value is obtained here,
independently of the Rust model (which intersects every ruled line with
every piece, solving for the crossing angles of an arc, and integrates
|sin| in closed form):

  * crossings: each arc is split where it turns back across the lines (its
    extreme points in the lines' normal direction), so every sub-piece runs
    one way across the lines; a one-way piece from height a to height b
    crosses floor((b - s)/d) - floor((a - s)/d) lines, counted with floor
    arithmetic and never by solving for an intersection;
  * a direction's average over every slide of the sheet: the total of
    |b - a| over those one-way pieces, divided by the gap;
  * the coast's length: exactly, as a rational number plus a rational
    multiple of pi (straight runs and r times the turn), and again by summing
    16,384 tiny chords per arc (a cross-check);
  * Crofton's identity: the average over every direction is checked by a
    composite Simpson rule over 3,600 directions, against 2L/(pi d);
  * the worst a fixed set of m directions can do for a straight stick: by a
    dense search over the stick's direction, then for six directions against
    the closed forms t cot t and t / sin t (t = pi/12);
  * how often each count turns up over every placement, and how much of the
    chart of lines each single line's count fills: integrated in closed form
    between the directions where two of the coast's change heights meet (see
    "count distributions" below), then checked against direction grids of
    1,800 to 28,800 directions that count by meeting each line with each
    piece;
  * random drops and random lines: numpy's Generator(PCG64DXSM(seed)), the
    generator the Rust stats kit matches bit for bit; lines through two
    random pins are checked against Crofton's chord formula, whose weight
    is the chord length cubed, by quadrature. A spread said of one drop, ten
    or a thousand is the spread over every placement (from the count
    distribution); a seeded sample's own spread is only checked against it.

Assumptions, stated once: lengths in centimetres; the sheet's lines are
infinite, straight and exactly the gap apart; a "random" drop turns the sheet
to a direction uniform over half a turn and slides it uniformly over one gap;
a line that only touches the coast, or passes exactly through a joint, is
never one of the lines counted on screen (such lines have chance zero in a
random drop). Each placement shown on screen is chosen by one rule: for its
direction, the slide on a 0.05 cm grid over one gap that keeps the ruled
lines farthest from every joint, end and turning point of the coast (ties go
to the smaller slide); every such placement clears them by at least MARGIN.
"""

import math
import sys
from fractions import Fraction

import numpy as np

GAP = 2.0
CARD_W, CARD_H = 16.0, 9.0
MARGIN = 0.1
SLIDE_STEP = Fraction(1, 20)
# chapter 02 slides the first sheet up this far, live (it stops short of a
# whole gap: at 1.568 to 1.573 cm past the start a tangent point flickers the
# count for 0.005 cm, too briefly to read)
SLIDE_SHOWN = Fraction(3, 2)
SLIDE_SHOWN_STEPS = 15000

# The coast: start, heading in degrees, then pieces. ("S", length) is a
# straight run; ("A", radius, turn) an arc turning `turn` degrees, left
# positive. Every turn is a whole number of degrees, so every arc's length is
# a rational multiple of pi.
COAST_START = (Fraction(-73, 10), Fraction(-1, 2))
COAST_HEADING = -75
COAST = [
    ("A", Fraction(2), 75),
    ("S", Fraction(7, 2)),
    ("A", Fraction(5, 4), 105),
    ("S", Fraction(3, 2)),
    ("A", Fraction(3, 4), -195),
    ("S", Fraction(3, 2)),
    ("A", Fraction(7, 4), 165),
    ("A", Fraction(1), -120),
    ("S", Fraction(1)),
    ("A", Fraction(3, 5), 180),
    ("A", Fraction(1), -105),
]
# Which piece is the long beach (the run that later ripples), and the round
# bay. The names are the ones the narration and labels use.
BEACH = 1
BAY = 6
PIECE_NAMES = ["cove", "beach", "climb", "rise", "headland", "fall", "bay", "point", "run", "hook", "end"]

STICK = 6.0
COIN = 2.0
OTHER_COINS = [1.5, 3.0]
NINE_BY = 0.5
STICK_HOME = (-4.5, 1.2)
STICK_TURN = 90
STICK_TILT = 30
TEST_SHADOW = 4.5
SHOWN_DIRECTIONS = [0, 45, 90, 135]
# Chapters 10 and 11: the longest stick the coast is cut into (cm), and the
# sets of evenly spread directions from 0 degrees averaged (the last is the
# computer's)
STICK_LONGEST = 0.45
DIRECTION_GRID = [4, 8, 16, 32, 3600]
# Chapters 12 and 13: where the stick is moved to (cm right of home, and
# its direction, degrees), the halves crossed, the nested sticks along the
# coast's ends, the sheet turned for them, the wire bent into a self-crossing
# "4" (its pieces, cm, and the turn at each joint, degrees), and the year
# of Barbier's note (J. Math. Pures Appl. (2) 5, 1860) and Ramaley's
# "Buffon's noodle problem" (Amer. Math. Monthly 76, 1969)
MOVE_RIGHT = 1.5
MOVE_TO = 45
CROSS_AT = 2.7
CROSS_TO = 135
NESTED = [14.0, 15.0]
NESTED_SLIDES = [0.0, 1.0]
FOUR_PIECES = [3.0, 1.5, 1.5]
FOUR_TURN = 135
FOUR_SHEET = 45
BARBIER_YEAR = 1860
NOODLE_YEAR = 1969
STEINHAUS_DIRECTIONS = 6
DIRECTION_LADDER = [1, 2, 4, 6]
SEED_DROPS = 20261024
FEW_DROPS = 10
MANY_DROPS = 1000
SEED_LINES = 20261025
LINES = 10000
# Two equal sticks for chapter 06: one at the card's centre, one near a corner.
MIDDLE_STICK = ((0.0, -1.0), (0.0, 1.0))
CORNER_STICK = ((-7.0, 2.0), (-7.0, 4.0))
WIDE_GAPS = [2.0, 4.0, 8.0]
# Chapter 09's finer drawing: the beach redrawn as a row of half-circle
# ripples of this count, each a corner, a half turn and a corner
# (("T", degrees) turns on the spot), so the beach keeps its ends.
RIPPLES = 7
# Richardson's dividers: each step jumps to the first point further along
# the coast that is exactly this far away in a straight line.
DIVIDER_STEPS = [Fraction(4), Fraction(2), Fraction(1), Fraction(1, 2)]
# Chapter 01: the step the dividers open to, and the ruler laid under the
# beach and then on the headland's top (tangent at the middle of its arc)
OPENING_STEP = Fraction(2)
RULER = 5.0
HEADLAND = 4


# Richardson's exponent for the west coast of Britain, as reported by
# Mandelbrot (Science 156, 1967, p. 636): length ~ step^(1 - D)
RICHARDSON_D = 1.25


def rippled_coast(count=RIPPLES):
    r = COAST[BEACH][1] / (2 * count)
    ripple = [("T", 90), ("A", r, -180), ("T", 90)]
    return COAST[:BEACH] + ripple * count + COAST[BEACH + 1:]

failures = []


def check(ok, what):
    print(("# ok   " if ok else "# FAIL ") + what, file=sys.stderr)
    if not ok:
        failures.append(what)


def fixed(x, places):
    """x rounded half-away-from-zero to `places` decimals, as text."""
    q = Fraction(x).limit_denominator(10**12) if isinstance(x, float) else Fraction(x)
    scale = 10**places
    n = abs(q) * scale
    whole = int(n)
    if n - whole >= Fraction(1, 2):
        whole += 1
    sign = "-" if q < 0 and whole else ""
    text = str(whole).rjust(places + 1, "0")
    return sign + (text[:-places] + "." + text[-places:] if places else text)


# --------------------------------------------------------------- the coast


def build(start, heading, pieces):
    """Each piece as ("S", P, Q) or ("A", centre, r, a0, a1), in floats."""
    x, y = float(start[0]), float(start[1])
    h = math.radians(heading)
    out = []
    for p in pieces:
        if p[0] == "S":
            length = float(p[1])
            nx, ny = x + length * math.cos(h), y + length * math.sin(h)
            out.append(("S", (x, y), (nx, ny)))
            x, y = nx, ny
        elif p[0] == "T":
            # a corner: turn on the spot, drawing nothing
            h += math.radians(p[1])
        else:
            r, turn = float(p[1]), math.radians(p[2])
            side = 1.0 if turn > 0 else -1.0
            cx, cy = x - side * r * math.sin(h), y + side * r * math.cos(h)
            a0 = math.atan2(y - cy, x - cx)
            a1 = a0 + turn
            out.append(("A", (cx, cy), r, a0, a1))
            x, y = cx + r * math.cos(a1), cy + r * math.sin(a1)
            h += turn
    return out


def exact_length(pieces):
    """(rational part, coefficient of pi)."""
    straight = sum((p[1] for p in pieces if p[0] == "S"), Fraction(0))
    arcs = sum((p[1] * Fraction(abs(p[2]), 180) for p in pieces if p[0] == "A"), Fraction(0))
    return straight, arcs


def chord_length(geom, per_arc=16384):
    total = 0.0
    for g in geom:
        if g[0] == "S":
            total += math.dist(g[1], g[2])
        else:
            (cx, cy), r, a0, a1 = g[1], g[2], g[3], g[4]
            pts = [(cx + r * math.cos(a0 + (a1 - a0) * i / per_arc), cy + r * math.sin(a0 + (a1 - a0) * i / per_arc)) for i in range(per_arc + 1)]
            total += sum(math.dist(a, b) for a, b in zip(pts, pts[1:]))
    return total


def normal(beta_deg):
    """Unit normal of lines running in direction beta (degrees)."""
    b = math.radians(beta_deg)
    return (-math.sin(b), math.cos(b))


def one_way_pieces(geom, n):
    """Heights (a, b) across lines with normal n, for sub-pieces that run one
    way: each arc is cut at the angles where it is tangent to the lines."""
    psi = math.atan2(n[1], n[0])
    out = []
    for g in geom:
        if g[0] == "S":
            out.append((g[1][0] * n[0] + g[1][1] * n[1], g[2][0] * n[0] + g[2][1] * n[1]))
            continue
        (cx, cy), r, a0, a1 = g[1], g[2], g[3], g[4]
        lo, hi = min(a0, a1), max(a0, a1)
        cuts = []
        k = math.ceil((lo - psi) / math.pi)
        while psi + k * math.pi < hi:
            if psi + k * math.pi > lo:
                cuts.append(psi + k * math.pi)
            k += 1
        angles = [lo] + cuts + [hi]
        c = cx * n[0] + cy * n[1]
        hs = [c + r * math.cos(a - psi) for a in angles]
        out.extend(zip(hs, hs[1:]))
    return out


def turning_heights(geom, n):
    """Every height where a crossing count can change: joints, ends and
    tangent points."""
    hs = []
    for a, b in one_way_pieces(geom, n):
        hs.extend([a, b])
    return hs


def sheet_count(pieces, slide, gap=GAP):
    """Crossings of one-way pieces (heights) with the lines at slide + k gap."""
    return sum(abs(math.floor((b - slide) / gap) - math.floor((a - slide) / gap)) for a, b in pieces)


def line_count(pieces, p):
    """Crossings of one-way pieces (heights) with the single line at height p."""
    return sum(1 for a, b in pieces if min(a, b) < p < max(a, b))


def coast_chart_grid(geom, columns=360, rows=184):
    """Finite midpoint image, counted by monotone projection intervals.

    The image is not the closed-form chart measure computed below. Outside
    card midpoints are excluded from the conditional frequency denominator.
    """
    reach = math.hypot(CARD_W / 2, CARD_H / 2)
    bins = {}
    for i in range(columns):
        theta = math.pi * (i + 0.5) / columns
        n = (math.cos(theta), math.sin(theta))
        half = CARD_W / 2 * abs(n[0]) + CARD_H / 2 * abs(n[1])
        pieces = one_way_pieces(geom, n)
        for j in range(rows):
            p = reach * (2 * (j + 0.5) / rows - 1)
            if abs(p) < half:
                count = line_count(pieces, p)
                bins[count] = bins.get(count, 0) + 1
    area = math.pi / columns * 2 * reach / rows
    return [bins.get(i, 0) for i in range(max(bins) + 1)], area


def crossings(geom, beta, slide, gap=GAP):
    return sheet_count(one_way_pieces(geom, normal(beta)), slide, gap)


def clearance(geom, beta, slide, gap=GAP):
    """How close any joint, end or tangent point comes to a ruled line."""
    n = normal(beta)
    worst = math.inf
    for h in turning_heights(geom, n):
        t = (h - slide) / gap
        worst = min(worst, abs(t - round(t)) * gap)
    return worst


def shown_slide(geom, beta, gap=GAP):
    """The placement rule for every sheet shown on screen (see the top)."""
    steps = int(Fraction(gap).limit_denominator(1000) / SLIDE_STEP)
    best = max(range(steps), key=lambda k: (round(clearance(geom, beta, float(k * SLIDE_STEP), gap), 9), -k))
    return float(best * SLIDE_STEP)


def slide_counts(geom, beta, gap=GAP, slides=4000):
    """Every count seen as the sheet slides through one gap."""
    return sorted({crossings(geom, beta, gap * (i + 0.5) / slides, gap) for i in range(slides)})


def travel(geom, beta):
    """Total distance the curve moves across lines of direction beta."""
    return sum(abs(b - a) for a, b in one_way_pieces(geom, normal(beta)))


def worth(gap=GAP):
    """The length one crossing stands for on lines `gap` apart: a coin as
    wide as the gap always crosses twice, and its rim is pi x gap. Every
    estimate is a count times this, so the gap cancels."""
    return math.pi * gap / 2


def slid_mean(geom, beta, gap=GAP, slides=4000):
    """A direction's average count, by sliding the sheet through one gap and
    counting at each slide (no travel, no formula)."""
    return sum(crossings(geom, beta, gap * (i + 0.5) / slides, gap) for i in range(slides)) / slides


def scaled(geom, k):
    """The same drawing enlarged k times about the card's centre."""
    out = []
    for g in geom:
        if g[0] == "S":
            out.append(("S", (k * g[1][0], k * g[1][1]), (k * g[2][0], k * g[2][1])))
        else:
            out.append(("A", (k * g[1][0], k * g[1][1]), k * g[2], g[3], g[4]))
    return out


def sample_points(geom, per_arc=64):
    """The coast as a polyline: straight runs as they are, arcs as chords."""
    pts = [geom[0][1] if geom[0][0] == "S" else None]
    for g in geom:
        if g[0] == "S":
            pts.append(g[2])
        else:
            (cx, cy), r, a0, a1 = g[1], g[2], g[3], g[4]
            arc = [(cx + r * math.cos(a0 + (a1 - a0) * i / per_arc), cy + r * math.sin(a0 + (a1 - a0) * i / per_arc)) for i in range(per_arc + 1)]
            if pts[0] is None:
                pts[0] = arc[0]
            pts.extend(arc[1:])
    return pts


def divider_walk(geom, step, per_arc=4096):
    """Richardson's walk: the landings, each (point, distance along the
    coast), the first being the coast's start; and the last straight bit to
    the end. Each step lands where the coast, followed onward from the last
    landing, first reaches `step` in a straight line. A step that lands on
    the far end to within 1e-9 cm counts as a step."""
    pts = sample_points(geom, per_arc)
    along = [0.0]
    for a, b in zip(pts, pts[1:]):
        along.append(along[-1] + math.dist(a, b))
    here, i = pts[0], 0
    landings = [(here, 0.0)]
    while True:
        j = i + 1
        while j < len(pts) and math.dist(here, pts[j]) < step - 1e-9:
            j += 1
        if j == len(pts):
            return landings, math.dist(here, pts[-1])
        # on the chord pts[j-1] -> pts[j], solve |a + t (b - a) - here| = step
        a, b = pts[j - 1], pts[j]
        dx, dy = b[0] - a[0], b[1] - a[1]
        ax, ay = a[0] - here[0], a[1] - here[1]
        qa, qb, qc = dx * dx + dy * dy, 2 * (ax * dx + ay * dy), ax * ax + ay * ay - step * step
        t = (-qb + math.sqrt(qb * qb - 4 * qa * qc)) / (2 * qa)
        here = (a[0] + t * dx, a[1] + t * dy)
        s = along[j - 1] + t * math.dist(a, b)
        # the landing replaces the chord's start, so the next step's search
        # (and its distance along) begins there
        pts[j - 1], along[j - 1] = here, s
        landings.append((here, s))
        i = j - 1


def dividers(geom, step, per_arc=4096):
    """(number of whole steps, steps x step + the last straight bit to the
    end), from `divider_walk`."""
    landings, last = divider_walk(geom, step, per_arc)
    steps = len(landings) - 1
    return steps, steps * step + last


def segments_cross(p, q, r, s):
    """Whether segments pq and rs cross or touch."""
    d1, d2 = side(p, q, r), side(p, q, s)
    d3, d4 = side(r, s, p), side(r, s, q)
    # Collinear disjoint segments also have four zero side values. Require
    # overlap of both coordinate intervals before accepting a meeting.
    boxes_overlap = all(max(min(p[k], q[k]), min(r[k], s[k])) <= min(max(p[k], q[k]), max(r[k], s[k])) for k in (0, 1))
    return boxes_overlap and d1 * d2 <= 0 and d3 * d4 <= 0


def simpson_mean(f, a, b, n):
    h = (b - a) / n
    s = f(a) + f(b) + sum((4 if i % 2 else 2) * f(a + i * h) for i in range(1, n))
    return s * h / 3 / (b - a)


# ---------------------------------------------------------------- sticks


def stick_geom(length, angle_deg, centre=(0.0, 0.0)):
    a = math.radians(angle_deg)
    dx, dy = length / 2 * math.cos(a), length / 2 * math.sin(a)
    return [("S", (centre[0] - dx, centre[1] - dy), (centre[0] + dx, centre[1] + dy))]


def worst_case(m):
    """Lowest and highest (count x worth) / length for a straight stick, over
    its direction, when the sheet is laid at m equally spaced directions and
    each count is averaged over every slide."""
    lo, hi = math.inf, -math.inf
    steps = 180 * 240
    for i in range(steps + 1):
        alpha = math.pi * i / steps
        mean_abs = sum(abs(math.sin(alpha - k * math.pi / m)) for k in range(m)) / m
        ratio = mean_abs * math.pi / 2
        lo, hi = min(lo, ratio), max(hi, ratio)
    return lo, hi


# --------------------------------------------------------- random lines


def pin_line(rng):
    p = (-CARD_W / 2 + CARD_W * rng.random(), -CARD_H / 2 + CARD_H * rng.random())
    q = (-CARD_W / 2 + CARD_W * rng.random(), -CARD_H / 2 + CARD_H * rng.random())
    return p, q


def even_line(rng):
    """Uniform joint normal-angle/signed-distance measure, conditioned on
    meeting the card. Propose both coordinates uniformly in a fixed box and
    redraw both after a miss. Accepted angles alone are not uniform: their
    marginal density is proportional to the card's projected width."""
    reach = math.hypot(CARD_W / 2, CARD_H / 2)
    while True:
        theta = math.pi * rng.random()
        p = -reach + 2 * reach * rng.random()
        n = (math.cos(theta), math.sin(theta))
        if abs(p) <= CARD_W / 2 * abs(n[0]) + CARD_H / 2 * abs(n[1]):
            # a point on the line, and its direction
            return (p * n[0], p * n[1]), (p * n[0] - n[1], p * n[1] + n[0])


def side(p, q, x):
    return (q[0] - p[0]) * (x[1] - p[1]) - (q[1] - p[1]) * (x[0] - p[0])


def crosses(line, stick):
    p, q = line
    return side(p, q, stick[0]) * side(p, q, stick[1]) < 0


def chord(theta, p):
    """Length of the card's chord on the line x cos t + y sin t = p."""
    c, s = math.cos(theta), math.sin(theta)
    # clip the line's points p n + t (-s, c) to the rectangle
    lo, hi = -math.inf, math.inf
    for o, d, half in ((p * c, -s, CARD_W / 2), (p * s, c, CARD_H / 2)):
        if abs(d) < 1e-15:
            if abs(o) > half:
                return 0.0
            continue
        t1, t2 = (-half - o) / d, (half - o) / d
        lo, hi = max(lo, min(t1, t2)), min(hi, max(t1, t2))
    return max(0.0, hi - lo)


def pin_rate_exact(stick, steps=1440):
    """Chance that the line through two uniform pins in the card crosses the
    stick: the lines crossing it, weighted by chord length cubed, over three
    times the card's area squared (Crofton's chord formula)."""
    (x0, y0), (x1, y1) = stick
    total = 0.0
    for i in range(steps):
        theta = math.pi * (i + 0.5) / steps
        c, s = math.cos(theta), math.sin(theta)
        a, b = sorted((x0 * c + y0 * s, x1 * c + y1 * s))
        sub = 64
        h = (b - a) / sub
        total += sum(chord(theta, a + (j + 0.5) * h) ** 3 for j in range(sub)) * h
    total *= math.pi / steps
    return total / (3 * (CARD_W * CARD_H) ** 2)


# ---------------------------------------------------- count distributions
#
# For one direction beta the count changes only at the heights (along the
# lines' normal (-sin beta, cos beta)) of the curve's ends and joints and of
# its arcs' tangent points. Each such height is a cos(beta) + b sin(beta) + c.
# Between two directions where two heights meet (for a sheet: differ by a
# whole number of gaps, or one reaches a line) the heights keep their order,
# so the stretch between two neighbouring heights keeps its count, and its
# width, again a cos + b sin + c, integrates in closed form over that range
# of directions. Counts come from the floor arithmetic of `sheet_count` at the
# middle of each stretch. `grid_shares` is the cross-check: a midpoint rule
# over directions, each stretch weighted by its width, counting by meeting
# each line with each piece directly.


def height_functions(geom):
    """(a, b, c) for every height above: each distinct end or joint, and each
    arc's two tangent heights. A tangent height whose point is off its arc
    for some direction only splits a stretch there, which changes nothing."""
    ends = []
    for g in geom:
        if g[0] == "S":
            ends += [g[1], g[2]]
        else:
            (cx, cy), r, a0, a1 = g[1], g[2], g[3], g[4]
            ends += [(cx + r * math.cos(a), cy + r * math.sin(a)) for a in (a0, a1)]
    points = []
    for p in ends:
        if all(math.dist(p, q) > 1e-9 for q in points):
            points.append(p)
    out = [(y, -x, 0.0) for x, y in points]
    for g in geom:
        if g[0] == "A":
            (cx, cy), r = g[1], g[2]
            out += [(cy, -cx, r), (cy, -cx, -r)]
    return out


def event_directions(fs, gap=None):
    """Every direction in [0, pi), radians, where two heights meet or, on a
    sheet `gap` apart, differ by a whole number of gaps or reach a line; with
    0 and pi."""
    reach = max(math.hypot(a, b) + abs(c) for a, b, c in fs)
    found = {0.0, math.pi}

    def meet(da, db, rhs):
        # da cos(beta) + db sin(beta) = rhs
        r = math.hypot(da, db)
        if r < 1e-12 or abs(rhs) > r:
            return
        phi, w = math.atan2(db, da), math.acos(rhs / r)
        for t in (phi + w, phi - w):
            t %= 2 * math.pi
            if t < math.pi:
                found.add(t)

    wraps = range(-int(2 * reach / gap) - 1, int(2 * reach / gap) + 2) if gap else [0]
    for i, (ai, bi, ci) in enumerate(fs):
        for aj, bj, cj in fs[i + 1:]:
            for k in wraps:
                meet(ai - aj, bi - bj, cj - ci + k * (gap or 0))
        if gap:
            for k in wraps:
                meet(ai, bi, k * gap - ci)
    return sorted(found)


def stretch_integral(lo, mid, hi, h, d):
    """The integral over [m - h, m + h] of w = A cos + B sin + C, from w at
    m - d, m and m + d. Exact for every such w: w(m + d) + w(m - d) - 2 w(m)
    is (2 cos d - 2)(w(m) - C), and the wave part integrates to 2 sin h
    times its value at m. Below h = 1e-4 rad the cancellation would cost more
    than it saves, and the midpoint value is used (off by under h^3 x reach)."""
    if h < 1e-4:
        return 2 * h * mid
    wave = (hi + lo - 2 * mid) / (2 * math.cos(d) - 2)
    return 2 * h * (mid - wave) + 2 * math.sin(h) * wave


def count_shares(geom, gap=None):
    """Over a direction uniform on half a turn and, for a sheet `gap` apart, a
    slide uniform over one gap: each count's chance. For a single line (gap
    None): each count's area on the chart of lines, cm x radians (count 0
    within the partition envelope, including unused full-circle extrema).
    The zero bin is not the card's miss area; use the card measure minus the
    positive-count area for that. Raises if two
    heights pass each other between found directions (a missing event)."""
    fs = height_functions(geom)
    found = event_directions(fs, gap)
    out = {}
    for u, v in zip(found, found[1:]):
        if v - u < 1e-13:
            continue
        m, h = (u + v) / 2, (v - u) / 2
        d = h / 2
        rows = [[a * math.cos(t) + b * math.sin(t) + c for a, b, c in fs] for t in (m - d, m, m + d)]
        if gap:
            shift = [gap * math.floor(x / gap) for x in rows[1]]
            rows = [[x - s for x, s in zip(row, shift)] for row in rows]
        order = sorted(range(len(fs)), key=lambda i: rows[1][i])
        lo, mid, hi = ([row[i] for i in order] for row in rows)
        for side in (lo, hi):
            if any(y < x - 1e-9 for x, y in zip(side, side[1:])):
                raise ValueError(f"two heights pass each other between {u:.9f} and {v:.9f} rad")
        stretches = [(i, i + 1, 0.0) for i in range(len(fs) - 1)]
        if gap:
            stretches.append((len(fs) - 1, 0, gap))
        pieces = one_way_pieces(geom, normal(math.degrees(m)))
        for i, j, wrap in stretches:
            width = [side[j] + wrap - side[i] for side in (lo, mid, hi)]
            if width[1] <= 1e-12:
                continue
            at = (mid[i] + mid[j] + wrap) / 2
            k = sheet_count(pieces, at, gap) if gap else line_count(pieces, at)
            out[k] = out.get(k, 0.0) + stretch_integral(*width, h, d)
    if gap:
        return {k: w / (math.pi * gap) for k, w in sorted(out.items())}
    return dict(sorted(out.items()))


def grid_shares(geom, directions, gap=None):
    """As `count_shares`, by a midpoint rule over `directions` directions, each
    stretch between sorted heights weighted by its width, and each count found
    by meeting the line (or every line of the sheet) with every piece: a
    straight run where its ends' heights straddle it, an arc where the
    circle's two meeting angles fall inside the arc."""
    fs = np.array(height_functions(geom))
    beta = math.pi * (np.arange(directions) + 0.5) / directions
    nx, ny = -np.sin(beta)[:, None], np.cos(beta)[:, None]
    psi = np.arctan2(ny, nx)
    hs = fs[None, :, 0] * ny + fs[None, :, 1] * -nx + fs[None, :, 2]
    if gap:
        hs = hs - gap * np.floor(hs / gap)
    hs = np.sort(hs, axis=1)
    lo, hi = (hs[:, :-1], hs[:, 1:]) if not gap else (hs, np.concatenate([hs[:, 1:], hs[:, :1] + gap], axis=1))
    width, at = hi - lo, (lo + hi) / 2

    def meetings(p):
        total = np.zeros(p.shape, dtype=np.int64)
        for g in geom:
            if g[0] == "S":
                total += ((g[1][0] * nx + g[1][1] * ny - p) * (g[2][0] * nx + g[2][1] * ny - p)) < 0
            else:
                (cx, cy), r, a0, a1 = g[1], g[2], g[3], g[4]
                q = (p - (cx * nx + cy * ny)) / r
                w = np.arccos(np.clip(q, -1.0, 1.0))
                for t in (psi + w, psi - w):
                    u = np.mod(t - min(a0, a1), 2 * math.pi)
                    total += (np.abs(q) < 1) & (u > 0) & (u < abs(a1 - a0))
        return total

    if gap:
        lines = int(max(math.hypot(a, b) + abs(c) for a, b, c in fs) / gap) + 2
        counts = sum(meetings(at + j * gap) for j in range(-lines, lines + 1))
    else:
        counts = meetings(at)
    weights = width * math.pi / directions
    out = {int(k): float(weights[(counts == k) & (width > 1e-12)].sum()) for k in np.unique(counts)}
    out = {k: w for k, w in out.items() if w > 0}
    if gap:
        return {k: w / (math.pi * gap) for k, w in sorted(out.items())}
    return dict(sorted(out.items()))


def moments(shares):
    """Mean and standard deviation of a count with these chances."""
    mean = sum(k * p for k, p in shares.items())
    return mean, math.sqrt(sum((k - mean) ** 2 * p for k, p in shares.items()))


def sum_shares(shares, n):
    """The chances of each total of n independent counts (convolution)."""
    out = {0: 1.0}
    for _ in range(n):
        nxt = {}
        for s, p in out.items():
            for k, q in shares.items():
                nxt[s + k] = nxt.get(s + k, 0.0) + p * q
        out = nxt
    return out


def hull_perimeter(points):
    """The perimeter of the convex hull of `points` (monotone chain)."""
    pts = sorted(set(points))

    def half(seq):
        out = []
        for p in seq:
            while len(out) >= 2 and side(out[-2], out[-1], p) <= 0:
                out.pop()
            out.append(p)
        return out

    hull = half(pts)[:-1] + half(pts[::-1])[:-1]
    return sum(math.dist(a, b) for a, b in zip(hull, hull[1:] + hull[:1]))


# ------------------------------------------------------------------ facts


def main():
    geom = build(COAST_START, COAST_HEADING, COAST)
    straight, pi_part = exact_length(COAST)
    length = float(straight) + float(pi_part) * math.pi
    f = {}

    # the card, the sheet, the coast
    f["card.width"] = fixed(CARD_W, 0)
    f["card.height"] = fixed(CARD_H, 0)
    f["sheet.gap"] = fixed(GAP, 0)
    f["coast.pieces"] = str(len(COAST))
    f["coast.arcs"] = str(sum(p[0] == "A" for p in COAST))
    f["coast.straight_runs"] = str(sum(p[0] == "S" for p in COAST))
    f["coast.length_exact"] = f"{straight} + {pi_part}pi"
    f["coast.length"] = fixed(length, 1)
    f["coast.length_2dp"] = fixed(length, 2)
    check(abs(chord_length(geom) - length) < 1e-6, f"chords sum to the exact length {length:.9f}")
    pts = sample_points(geom)
    reach_x = max(abs(p[0]) for p in pts)
    reach_y = max(abs(p[1]) for p in pts)
    check(reach_x < CARD_W / 2 - 0.25 and reach_y < CARD_H / 2 - 0.25, f"the coast keeps a quarter centimetre inside the card (reaches |x| {reach_x:.2f}, |y| {reach_y:.2f})")
    path = list(zip(pts, pts[1:]))
    touching = [(i, j) for i in range(len(path)) for j in range(i + 2, len(path)) if segments_cross(*path[i], *path[j])]
    check(not touching, f"the coast never crosses itself ({len(path)} chords, {len(touching)} meetings)")
    ends = math.dist(pts[0], pts[-1])
    f["coast.ends_apart"] = fixed(ends, 1)
    check(ends > 1.0, f"its two ends are {ends:.2f} cm apart (an open coast, not a loop)")
    bay = geom[BAY]
    bay_chord = math.dist(
        (bay[1][0] + bay[2] * math.cos(bay[3]), bay[1][1] + bay[2] * math.sin(bay[3])),
        (bay[1][0] + bay[2] * math.cos(bay[4]), bay[1][1] + bay[2] * math.sin(bay[4])),
    )
    bay_arc = float(COAST[BAY][1]) * math.radians(abs(COAST[BAY][2]))
    f["bay.arc"] = fixed(bay_arc, 1)
    f["bay.chord"] = fixed(bay_chord, 1)
    check(bay_chord < bay_arc, "the bay's chord is shorter than the bay")

    # the first placement: lines running across the card, slid by the rule
    first_slide = shown_slide(geom, 0)
    first = crossings(geom, 0, first_slide)
    f["open.slide"] = fixed(first_slide, 2)
    f["open.crossings"] = str(first)
    f["open.estimate"] = fixed(first * worth(), 1)
    f["open.error_percent"] = fixed((first * worth() - length) / length * 100, 0)
    f["open.short_by"] = fixed(length - first * worth(), 1)
    # Chapter 06: slide the same sheet into the next run of nine.
    nine_slide = first_slide + NINE_BY
    nine = crossings(geom, 0, nine_slide)
    f["bet.nine_slide"] = fixed(nine_slide, 2)
    f["bet.nine_crossings"] = str(nine)
    f["bet.nine_estimate"] = fixed(nine * worth(), 1)
    f["bet.nine_long_by"] = fixed(nine * worth() - length, 1)
    passed = []
    for i in range(5001):
        n = crossings(geom, 0, first_slide + NINE_BY * i / 5000)
        if not passed or passed[-1] != n:
            passed.append(n)
    check(passed == [first, nine], f"chapter 06's slide up {NINE_BY} cm passes the counts {passed}")
    check(first * worth() < length < nine * worth(), f"the bet is short at {first} crossings and long at {nine}")
    per_line = []
    for k in range(-4, 5):
        height = first_slide + k * GAP
        n = sum(1 for a, b in one_way_pieces(geom, normal(0)) if min(a, b) < height < max(a, b))
        if n:
            per_line.append(n)
    f["open.lines_crossed"] = str(len(per_line))
    f["open.per_line"] = ",".join(map(str, per_line))
    check(sum(per_line) == first, f"the first placement's lines, bottom to top, cross {per_line} times")
    # which piece holds each crossing, walking the coast from its start, and
    # which line (0 the lowest crossed) it is on
    walk = []
    lowest = None
    for i, piece in enumerate(geom):
        for a, b in one_way_pieces([piece], normal(0)):
            lo_k = math.floor((min(a, b) - first_slide) / GAP) + 1
            hi_k = math.floor((max(a, b) - first_slide) / GAP)
            ks = list(range(lo_k, hi_k + 1))
            walk.extend((i, k) for k in (ks if b > a else reversed(ks)))
    lowest = min(k for _, k in walk)
    f["open.crossing_pieces"] = ",".join(PIECE_NAMES[i] for i, _ in walk)
    f["open.crossing_lines"] = ",".join(str(k - lowest) for _, k in walk)
    check(len(walk) == first, f"walking the coast meets its crossings in order {f['open.crossing_pieces']}")
    # the tempting exchange rate, one gap per crossing, is impossible: no
    # curve is shorter than the straight distance between its ends
    f["open.gap_guess"] = fixed(first * GAP, 0)
    check(first * GAP < ends, f"seven gaps, {first * GAP:g} cm, fall short of the {ends:.2f} cm between the ends")
    # Chapters 02/03: per-piece and per-line counts, and equal added arc lengths.
    f["open.lower_crossings"] = str(per_line[0])
    f["open.upper_crossings"] = str(per_line[1])
    f["open.bay_crossings"] = str(sum(PIECE_NAMES[i] == "bay" for i, _ in walk))
    f["open.end_crossings"] = str(sum(PIECE_NAMES[i] == "end" for i, _ in walk))
    for key, n in [("big", 1), ("small", 7)]:
        recipe = rippled_coast(n)
        redrawn = build(COAST_START, COAST_HEADING, recipe)
        amount = crossings(redrawn, 0, first_slide)
        f[f"swap.{key}_crossings"] = str(amount)
        f[f"swap.{key}_more"] = str(amount - first)
        straight_part, pi_part_new = exact_length(recipe)
        extra = float(straight_part) + float(pi_part_new) * math.pi - length
        if n == 1:
            f["swap.extra"] = fixed(extra, 0)
            f["swap.extra_3dp"] = fixed(extra, 3)
    # chapter 02's slide: the sheet moves up SLIDE_SHOWN cm from the first
    # placement; the counts in the order they appear, and each one's run
    runs = []
    for i in range(SLIDE_SHOWN_STEPS + 1):
        n = crossings(geom, 0, first_slide + float(SLIDE_SHOWN) * i / SLIDE_SHOWN_STEPS)
        if runs and runs[-1][0] == n:
            runs[-1][1] += 1
        else:
            runs.append([n, 1])
    f["slide.distance"] = fixed(SLIDE_SHOWN, 1)
    f["slide.sequence"] = ",".join(str(n) for n, _ in runs)
    shortest = min(steps for _, steps in runs) * float(SLIDE_SHOWN) / SLIDE_SHOWN_STEPS
    f["slide.shortest_run"] = fixed(shortest, 2)
    # where the shortest run lies, cm past the first placement, so the live
    # slide can slow through it
    brief = min(range(len(runs)), key=lambda i: runs[i][1])
    brief_from = sum(steps for _, steps in runs[:brief]) * float(SLIDE_SHOWN) / SLIDE_SHOWN_STEPS
    f["slide.shortest_count"] = str(runs[brief][0])
    f["slide.shortest_from"] = fixed(brief_from, 2)
    f["slide.shortest_to"] = fixed(brief_from + shortest, 2)
    check(shortest >= 0.05, f"every count in the shown slide lasts at least {shortest:.3f} cm of sliding")
    check(clearance(geom, 0, first_slide) >= MARGIN, f"first placement clears every turning point by {clearance(geom, 0, first_slide):.3f}")

    # the coin: two crossings at every placement
    rim = math.pi * COIN
    f["coin.width"] = fixed(COIN, 0)
    f["coin.rim"] = fixed(rim, 2)
    f["coin.crossings"] = "2"
    f["coin.worth"] = fixed(rim / 2, 2)
    check(COIN == GAP and abs(rim / 2 - worth()) < 1e-12, f"the coin is as wide as the gap, so its rim over its two crossings, {rim / 2:.6f}, is the worth of one crossing")
    # chapter 05, exactly: a line d from a coin's centre meets its rim at 2
    # points if d < r, at 1 if d = r (x^2 = r^2 - d^2 = 0), else at none;
    # the coin set down at every 1/360 cm across one gap
    def coin_kinds(r):
        def roots(d):
            return 2 if d < r else 1 if d == r else 0
        kinds = set()
        for c in (Fraction(k, 360) for k in range(721)):
            per_line = [roots(abs(c - Fraction(GAP) * k)) for k in range(-3, 4)]
            kinds.add((per_line.count(2), per_line.count(1), sum(per_line)))
        return kinds

    seen = coin_kinds(Fraction(COIN) / 2)
    check(seen == {(1, 0, 2), (0, 2, 2)}, f"a coin as wide as the gap has one line through it and two points, or touches two lines at a point each: {sorted(seen)}")
    f["coin.lines_through"] = "1"
    f["coin.touching_lines"] = "2"
    f["coin.touch_points"] = "1"
    def coin_counts(width, draws=20000, seed=7):
        """Crossings of a coin `width` across at uniform random heights
        (a circle looks the same turned any way, so one direction will do)."""
        rng = np.random.Generator(np.random.PCG64DXSM(seed))
        out = []
        for _ in range(draws):
            centre = rng.random() * GAP
            out.append(2 * sum(1 for k in range(-4, 5) if centre - width / 2 < k * GAP < centre + width / 2))
        return out

    check(set(coin_counts(COIN)) == {2}, f"a {COIN:g} cm coin crosses {sorted(set(coin_counts(COIN)))} times")
    for w in OTHER_COINS:
        key = fixed(w, 1).replace(".", "_")
        seen = coin_counts(w)
        if w < GAP:
            share, rare, expected = w / GAP, 2, {0, 2}
            f[f"coin.{key}.counts"] = "0,2"
            f[f"coin.{key}.two_share"] = fixed(share, 2)
        else:
            share, rare, expected = w / GAP - 1, 4, {2, 4}
            f[f"coin.{key}.counts"] = "2,4"
            f[f"coin.{key}.four_share"] = fixed(share, 2)
        f[f"coin.{key}.mean"] = fixed(2 * w / GAP, 2)
        got = sum(x == rare for x in seen) / len(seen)
        se = math.sqrt(share * (1 - share) / len(seen))
        check(set(seen) == expected, f"a {w:g} cm coin crosses {sorted(set(seen))} times")
        check(abs(got - share) < 4 * se, f"a {w:g} cm coin shows {rare} crossings {got:.4f} of the time, against {share:.4f}")
        check(abs(sum(seen) / len(seen) - 2 * w / GAP) < 4 * 2 * se, f"a {w:g} cm coin averages 2 x rim / (pi x gap)")

    # the stick
    f["stick.length"] = fixed(STICK, 0)
    # 07_1 lays the stick across the lines, its centre 0.3 cm above one;
    # 07_3 lays it along them, midway between two
    across = crossings(stick_geom(STICK, 90, (0.0, 0.3)), 0, 0.0)
    along = crossings(stick_geom(STICK, 0, (0.0, GAP / 2)), 0, 0.0)
    f["stick.across"] = str(across)
    f["stick.along"] = str(along)
    f["stick.across_estimate"] = fixed(across * worth(), 1)
    f["stick.along_estimate"] = fixed(along * worth(), 0)
    # Chapter 07's on-screen stick, independently sampled through its turn.
    def home(angle):
        return stick_geom(STICK, angle, STICK_HOME)

    f["stick.home_across"] = str(crossings(home(90), 0, first_slide))
    f["stick.home_along"] = str(crossings(home(0), 0, first_slide))
    turn_seen = []
    for i in range(100 * STICK_TURN + 1):
        n = crossings(home(i / 100), 0, first_slide)
        if not turn_seen or turn_seen[-1] != n:
            turn_seen.append(n)
    f["stick.turn_counts"] = ",".join(map(str, turn_seen))
    f["stick.shadow_across"] = fixed(travel(home(90), 0), 0)
    f["stick.shadow_along"] = fixed(travel(home(0), 0), 0)
    y0 = Fraction(STICK_HOME[1]).limit_denominator(1000)
    s0, gap = Fraction(first_slide).limit_denominator(1000), Fraction(GAP)
    off = min(min((e - s0) % gap, gap - (e - s0) % gap) for e in (y0 - Fraction(STICK) / 2, y0 + Fraction(STICK) / 2))
    check(off >= Fraction(1, 5), f"lying across, the stick's ends keep {off} cm from the lines, at least a fifth")
    check(f["stick.home_across"] == f["stick.across"] and f["stick.home_along"] == f["stick.along"],
          "the stick on screen crosses as many lines as the oracle's own placements")
    check(turn_seen == sorted(set(turn_seen)), f"turned from along to {STICK_TURN} degrees, the count only climbs: {turn_seen}")
    slides = 4000
    at30 = sum(crossings(stick_geom(STICK, 30), 0, GAP * (i + 0.5) / slides) for i in range(slides)) / slides
    f["stick.at30_mean"] = fixed(at30, 1)
    f["stick.at30_counts"] = ",".join(map(str, sorted({crossings(stick_geom(STICK, 30), 0, GAP * (i + 0.5) / 97) for i in range(97)})))
    f["stick.reach30"] = fixed(STICK * math.sin(math.radians(30)), 0)
    turned = simpson_mean(lambda b: travel(stick_geom(STICK, 90), b) / GAP, 0.0, 180.0, 3600)
    f["stick.turned_mean"] = fixed(turned, 2)
    f["stick.turned_estimate"] = fixed(turned * worth(), 1)
    mean_reach = simpson_mean(lambda b: travel(stick_geom(STICK, 90), b), 0.0, 180.0, 3600)
    f["stick.mean_reach"] = fixed(mean_reach, 2)
    f["stick.mean_reach_share"] = fixed(mean_reach / STICK, 2)
    check(abs(turned - 2 * STICK / (math.pi * GAP)) < 1e-9, "turned stick averages 2 x length / (pi x gap)")
    check(abs(at30 - STICK * 0.5 / GAP) < 1e-3, "a stick at 30 degrees averages its reach over the gap")
    # Chapters08/09: independent midpoint enumeration, not native event runs.
    slide_steps = 20000

    def slid(angle):
        seen, shares, total = [], {}, 0
        for i in range(slide_steps):
            n = crossings(home(angle), 0, first_slide + GAP * (i + 0.5) / slide_steps)
            if not seen or seen[-1] != n:
                seen.append(n)
            shares[n] = shares.get(n, 0) + 1
            total += n
        return seen, {n: k / slide_steps for n, k in shares.items()}, total / slide_steps

    def shadow_of(angle):
        [(_, (_, y0), (_, y1))] = home(angle)
        return min(y0, y1), max(y0, y1)

    def off_lines(h):
        t = (h - first_slide) / GAP
        return abs(t - round(t)) * GAP

    tilted = 180 - STICK_TILT
    tested = 180 - math.degrees(math.asin(TEST_SHADOW / STICK))
    seen, shares, mean = slid(tilted)
    lo, hi = shadow_of(tilted)
    first_count = crossings(home(tilted), 0, first_slide)
    f["stick.tilt"] = fixed(STICK_TILT, 0)
    f["stick.tilted_first"] = str(first_count)
    f["stick.tilted_counts"] = ",".join(map(str, seen))
    f["stick.tilted_one_share"] = fixed(shares.get(1, 0), 2)
    f["stick.tilted_two_share"] = fixed(shares.get(2, 0), 2)
    f["stick.tilted_leftover"] = fixed(hi - lo - GAP, 0)
    whole, held = set(), 0
    for i in range(slide_steps):
        s = first_slide + GAP * (i + 0.5) / slide_steps
        whole.add(math.floor((lo + GAP - s) / GAP) - math.floor((lo - s) / GAP))
        held += math.floor((hi - s) / GAP) - math.floor((lo + GAP - s) / GAP)
    f["stick.gap_piece_counts"] = ",".join(map(str, sorted(whole)))
    f["stick.leftover_share"] = fixed(held / slide_steps, 2)
    check(seen == [first_count, first_count - 1, first_count],
          f"tilted {STICK_TILT} degrees, the slide passes {seen}: two, one, two")
    check(fixed(mean, 1) == f["stick.at30_mean"] and abs(mean - (hi - lo) / GAP) < 1e-3,
          f"the slide on screen averages {mean:.4f}, its shadow over the gap")
    check(abs(held / slide_steps - (hi - lo - GAP) / GAP) < 1e-3, "the part left over holds a line for its share of the slide")
    t_seen, t_shares, t_mean = slid(tested)
    t_lo, t_hi = shadow_of(tested)
    t_first = crossings(home(tested), 0, first_slide)
    f["stick.test_shadow"] = fixed(TEST_SHADOW, 1)
    f["stick.test_first"] = str(t_first)
    f["stick.test_counts"] = ",".join(map(str, t_seen))
    f["stick.test_three_share"] = fixed(t_shares.get(3, 0), 2)
    f["stick.test_mean"] = fixed(t_mean, 2)
    check(abs(t_hi - t_lo - TEST_SHADOW) < 1e-9, f"tilted to {tested:.2f} degrees, the shadow is {t_hi - t_lo:.6f} cm")
    check(t_seen == [t_first, t_first + 1, t_first], f"the test slide passes {t_seen}: two, three, two")
    check(abs(t_mean - TEST_SHADOW / GAP) < 1e-3, f"the test slide averages {t_mean:.4f}, its shadow over the gap")
    near = min(off_lines(h) for h in (lo, hi, t_lo, t_hi))
    check(near >= 0.2, f"tilted and tested, the stick's ends keep {near:.3f} cm from the lines")
    f["stick.across_rule"] = fixed(travel(home(90), 0) / GAP, 0)
    f["stick.along_rule"] = fixed(travel(home(0), 0) / GAP, 0)
    turned_shadows = [travel(home(180 * (i + 0.5) / 3600), 0) for i in range(3600)]
    turned_shadow = sum(turned_shadows) / 3600
    f["stick.mean_reach_percent"] = fixed(turned_shadow / STICK * 100, 0)
    check(abs(turned_shadow - mean_reach) < 1e-6, "the stick turned half a turn averages the shadow the sheet turned does")
    check(sum(s > STICK / 2 for s in turned_shadows) / 3600 > 0.5, "the arch stays above half the stick for most directions")
    # Buffon's noodle: the same stick bent into a hook, a 3 cm arm then a
    # 3 cm arm at a right angle, averages the same
    hook = [("S", (-1.5, -1.5), (1.5, -1.5)), ("S", (1.5, -1.5), (1.5, 1.5))]
    hook_mean = simpson_mean(lambda b: travel(hook, b) / GAP, 0.0, 180.0, 3600)
    f["hook.mean"] = fixed(hook_mean, 2)
    f["hook.counts"] = ",".join(map(str, sorted({crossings(hook, 180 * i / 61, GAP * (j + 0.5) / 53) for i in range(61) for j in range(53)})))
    check(abs(hook_mean - turned) < 1e-9, "the bent stick averages what the straight one does")
    f["stick.all_counts"] = ",".join(map(str, sorted({crossings(stick_geom(STICK, 90), 180 * i / 61, GAP * (j + 0.5) / 53) for i in range(61) for j in range(53)})))
    check(f["hook.counts"] != f["stick.all_counts"], f"bent and straight sticks give different single counts ({f['hook.counts']} against {f['stick.all_counts']}) and the same average")

    # four turns of the sheet, each slid by the rule
    counts = []
    for beta in SHOWN_DIRECTIONS:
        s = shown_slide(geom, beta)
        n = crossings(geom, beta, s)
        counts.append(n)
        seen = slide_counts(geom, beta)
        f[f"turn.{beta}.slide"] = fixed(s, 2)
        f[f"turn.{beta}.crossings"] = str(n)
        f[f"turn.{beta}.slide_counts"] = f"{seen[0]}..{seen[-1]}"
        f[f"turn.{beta}.slide_mean"] = fixed(travel(geom, beta) / GAP, 2)
        f[f"turn.{beta}.reach"] = fixed(travel(geom, beta), 2)
        # 11_1's table shows the travel to three places, so that halving the
        # shown travel gives the shown average
        f[f"turn.{beta}.reach_3dp"] = fixed(travel(geom, beta), 3)
        check(fixed(float(f[f"turn.{beta}.reach_3dp"]) / GAP, 2) == f[f"turn.{beta}.slide_mean"], f"direction {beta}: the shown {f[f'turn.{beta}.reach_3dp']} / {GAP:g} rounds to the shown {f[f'turn.{beta}.slide_mean']}")
        check(clearance(geom, beta, s) >= MARGIN, f"direction {beta} at slide {s:.2f} clears every turning point by {clearance(geom, beta, s):.3f}")
        sweep = sum(crossings(geom, beta, GAP * (i + 0.5) / 4000) for i in range(4000)) / 4000
        check(abs(sweep - travel(geom, beta) / GAP) < 2e-3, f"direction {beta}: sliding through a gap averages {sweep:.4f}, travel over gap {travel(geom, beta) / GAP:.4f}")
    check(first == counts[SHOWN_DIRECTIONS.index(0)], "the first placement is the turn chapter's first direction")
    four = sum(counts) / len(counts)
    f["turn.crossings_total"] = str(sum(counts))
    f["turn.crossings_mean"] = fixed(four, 2)
    f["turn.estimate"] = fixed(four * worth(), 1)
    slid_four = sum(travel(geom, b) for b in SHOWN_DIRECTIONS) / len(SHOWN_DIRECTIONS) / GAP
    f["turn.slide_mean"] = fixed(slid_four, 2)
    f["turn.slide_estimate"] = fixed(slid_four * worth(), 2)
    # 11_4 multiplies the shown mean by pi, so it shows three places
    f["turn.slide_mean_3dp"] = fixed(slid_four, 3)

    # Crofton: averaged over every turn and slide
    mean_all = simpson_mean(lambda b: travel(geom, b) / GAP, 0.0, 180.0, 3600)
    f["crofton.mean_crossings"] = fixed(length * 2 / (math.pi * GAP), 2)
    f["crofton.mean_reach"] = fixed(simpson_mean(lambda b: travel(geom, b), 0.0, 180.0, 3600), 2)
    f["crofton.twice_length"] = fixed(2 * length, 2)
    # the places 11_5 and 19_4 show, so that each shown result follows from
    # its shown operands: 16.433 / 2 = 8.2165, 8.2165 x pi = 25.81, and
    # 2 x 25.813 = 51.63
    f["crofton.mean_reach_3dp"] = fixed(2 * length / math.pi, 3)
    f["crofton.mean_crossings_4dp"] = fixed(length * 2 / (math.pi * GAP), 4)
    f["coast.length_3dp"] = fixed(length, 3)
    for shown, result in (
        (float(f["turn.slide_mean_3dp"]) * math.pi, f["turn.slide_estimate"]),
        (float(f["crofton.mean_reach_3dp"]) / GAP, f["crofton.mean_crossings_4dp"]),
        (float(f["crofton.mean_crossings_4dp"]) * math.pi, f["coast.length_2dp"]),
        (2 * float(f["coast.length_3dp"]), f["crofton.twice_length"]),
    ):
        places = len(result.split(".")[1]) if "." in result else 0
        check(fixed(shown, places) == result, f"a shown product {shown:.6f} rounds to its shown result {result}")
    check(fixed(float(f["crofton.mean_crossings"]) * math.pi, 2) != f["coast.length_2dp"], f"negative control: the two-place mean {f['crofton.mean_crossings']} x pi gives {fixed(float(f['crofton.mean_crossings']) * math.pi, 2)}, not {f['coast.length_2dp']}, so 11_5 must show more places")
    # the chart of lines: lines meeting the 6 cm stick, the 2 cm stick and the
    # card, as areas (distance in cm times direction in radians)
    f["chart.stick_band"] = fixed(simpson_mean(lambda b: travel(stick_geom(STICK, 90), b), 0.0, 180.0, 3600) * math.pi, 2)
    f["chart.stick_poses"] = ",".join(fixed(simpson_mean(lambda b: travel(stick_geom(STICK, angle, center), b), 0.0, 180.0, 3600) * math.pi, 2) for center, angle in [((0., 0.), 90), ((3., 0.), 90), ((0., 0.), 0)])
    f["chart.stick_quadrature"] = f["chart.stick_band"]
    f["chart.card"] = fixed(2 * (CARD_W + CARD_H), 0)
    card = [("S", (-8.0, -4.5), (8.0, -4.5)), ("S", (8.0, -4.5), (8.0, 4.5)), ("S", (8.0, 4.5), (-8.0, 4.5)), ("S", (-8.0, 4.5), (-8.0, -4.5))]
    card_band = simpson_mean(lambda b: travel(card, b) / 2, 0.0, 180.0, 3600) * math.pi
    check(abs(card_band - 2 * (CARD_W + CARD_H)) < 1e-6, f"lines meeting the card fill {card_band:.6f} of the chart, its perimeter")
    # the coin's chart: lines within 1 cm of its centre, a flat band 2 cm
    # tall across the whole chart, each crossing it twice
    coin_geom = [("A", (0.0, 0.0), COIN / 2, -math.pi / 2, 3 * math.pi / 2)]
    coin_total = simpson_mean(lambda b: travel(coin_geom, b) / 2, 0.0, 180.0, 3600) * math.pi * 2
    f["chart.coin_band"] = fixed(COIN * math.pi, 2)
    f["chart.coin_total"] = fixed(coin_total, 2)
    f["coin.radius"] = fixed(COIN / 2, 0)
    # Roots of t^2 + p^2 = r^2, including the distinct tangent root.
    discriminants = [(COIN / 2)**2 - p**2 for p in [0., COIN / 2, 0.75 * COIN]]
    f["formula.coin_counts"] = ",".join(str(2 if d > 0 else 1 if d == 0 else 0) for d in discriminants)
    f["cauchy.reported_year"] = "1832"  # Barbier1860, printed p277; original memoir not read.
    check(abs(coin_total - 2 * rim) < 1e-9, f"the coin's chart totals {coin_total:.6f}, twice its rim")
    f["chart.coast_total"] = fixed(mean_all * GAP * math.pi, 2)
    check(abs(mean_all * GAP * math.pi - 2 * length) < 1e-5, "the coast's chart, counted with crossings, totals twice its length")
    check(abs(mean_all - 2 * length / (math.pi * GAP)) < 1e-6, f"Simpson over directions {mean_all:.9f} = 2L/(pi gap)")
    # 19_1's shading: how much of the chart each single line's count fills
    # (a single line, not a sheet: a sheet's count runs higher)
    lines = count_shares(geom)
    meet = sum(w for k, w in lines.items() if k > 0)
    line_max = max(k for k, w in lines.items() if w > 1e-9)
    card_lines = 2 * (CARD_W + CARD_H)
    f["chart.line_max"] = str(line_max)
    f["chart.coast_lines"] = fixed(meet, 2)
    f["chart.meet_percent"] = fixed(meet / card_lines * 100, 0)
    f["chart.miss_percent"] = fixed((1 - meet / card_lines) * 100, 0)
    f["chart.once_percent"] = fixed(lines[1] / meet * 100, 0)
    grid_bins, cell_area = coast_chart_grid(geom)
    grid_total = sum(k * n * cell_area for k, n in enumerate(grid_bins))
    f["chart.grid_columns"], f["chart.grid_rows"] = "360", "184"
    f["chart.grid_bins"] = ",".join(map(str, grid_bins))
    f["chart.grid_total"] = fixed(grid_total, 5)
    f["chart.grid_total_2dp"] = fixed(grid_total, 2)
    f["chart.grid_miss_percent"] = fixed(100 * grid_bins[0] / sum(grid_bins), 0)
    f["chart.grid_once_percent"] = fixed(100 * grid_bins[1] / sum(grid_bins[1:]), 0)
    f["chart.grid_max"] = str(len(grid_bins) - 1)
    f["chart.selected_counts"] = f"0,1,{len(grid_bins)-1}"
    f["chart.exact_total_5dp"] = fixed(2 * length, 5)
    f["crofton.year"] = "1868"
    f["chart.line_areas"] = ",".join(f"{k}:{fixed(w, 2)}" for k, w in lines.items() if k > 0)
    hull = hull_perimeter(sample_points(geom, 4096))
    check(abs(meet - hull) < 1e-6, f"lines meeting the coast fill {meet:.7f} of the chart, the perimeter of its convex hull {hull:.7f} (Cauchy)")
    check(abs(sum(k * w for k, w in lines.items()) - 2 * length) < 1e-9, "single lines' counts times their areas total twice the length")
    check(all(w < 1e-12 for k, w in lines.items() if k > line_max), f"no single line crosses the coast more than {line_max} times, except on lines of no area")
    check(lines[1] / meet > 0.5 and meet / card_lines > 0.5, f"most lines across the card meet the coast ({meet / card_lines:.4f}), and most that meet it cross once ({lines[1] / meet:.4f})")

    # fixed sets of directions, for a straight stick
    bands = {}
    for m in DIRECTION_LADDER:
        lo, hi = worst_case(m)
        bands[m] = (lo, hi)
        f[f"ladder.{m}.low_percent"] = fixed((lo - 1) * 100, 1 if m > 1 else 0)
        f[f"ladder.{m}.high_percent"] = fixed((hi - 1) * 100, 1 if m > 1 else 0)
        # the closed forms, which the search above does not use
        t = math.pi / (2 * m)
        check(abs(lo - t / math.tan(t)) < 1e-6 and abs(hi - t / math.sin(t)) < 1e-6, f"{m} directions: search {lo:.6f}..{hi:.6f} matches t cot t..t / sin t")
    # six directions in closed form: cot(pi/12) = 2 + sqrt 3, so the band
    # runs from (2 + sqrt 3) pi / 12 - 1 = -2.2951% to (pi/12) / sin(pi/12)
    # - 1 = +1.1515%. Steinerberger (2026, section 1.4) prints -2.26%; on the
    # band that is a stick about 0.08 degrees off one family, not its low end
    lo6, hi6 = bands[6]
    check(abs(lo6 - (2 + math.sqrt(3)) * math.pi / 12) < 1e-12 and abs(hi6 - (math.pi / 12) / math.sin(math.pi / 12)) < 1e-12, f"six directions: {(lo6 - 1) * 100:.4f}% to {(hi6 - 1) * 100:+.4f}%, the closed forms")

    def six_ratio(alpha):
        return sum(abs(math.sin(alpha - k * math.pi / 6)) for k in range(6)) / 6 * math.pi / 2

    a, b = 0.0, math.pi / 12
    for _ in range(60):
        a, b = (a, (a + b) / 2) if six_ratio((a + b) / 2) > 1 - 0.0226 else ((a + b) / 2, b)
    check(math.degrees(a) < 0.1, f"the printed -2.26% is the six-direction estimate of a stick {math.degrees(a):.4f} degrees off a family")
    # a curve is a crowd of tiny sticks, so its error under a fixed set of
    # directions (slid through a gap) lies inside the stick's band
    for m in (4, 6):
        est = sum(travel(geom, 180 * k / m) / GAP for k in range(m)) / m * worth()
        ratio = est / length
        f[f"ladder.{m}.coast_estimate"] = fixed(est, 2)
        f[f"ladder.{m}.coast_error_percent"] = fixed((ratio - 1) * 100, 1)
        check(bands[m][0] - 1e-9 <= ratio <= bands[m][1] + 1e-9, f"{m} slid directions give the coast {est:.3f}, inside the stick's band")
    # Chapter 10: the coast as a crowd of sticks. Each piece cut where it
    # lies level with the lines at 0 degrees (an arc at pi/2 + k pi) runs one
    # way across them; each such stretch split into the fewest equal sticks
    # no longer than STICK_LONGEST. A one-way stick's average count over
    # every slide is its shadow over the gap (08), so the coast's is all
    # the shadows added up, over the gap
    n0 = normal(0)
    stretches = []
    for g in geom:
        if g[0] == "S":
            stretches.append((g, math.dist(g[1], g[2])))
            continue
        (cx, cy), r, a0, a1 = g[1], g[2], g[3], g[4]
        lo, hi = min(a0, a1), max(a0, a1)
        k = math.ceil((lo - math.pi / 2) / math.pi)
        level = []
        while math.pi / 2 + k * math.pi < hi:
            if math.pi / 2 + k * math.pi > lo:
                level.append(math.pi / 2 + k * math.pi)
            k += 1
        angles = [lo] + level + [hi]
        stretches.extend((("A", (cx, cy), r, a, b), r * (b - a)) for a, b in zip(angles, angles[1:]))
    check(all(len(one_way_pieces([g], n0)) == 1 for g, _ in stretches), f"cut where it lies level with the lines at 0 degrees, the coast runs one way in each of its {len(stretches)} stretches")
    check(any(len(one_way_pieces([g], n0)) > 1 for g in geom), "negative control: cut only at its joints, some piece of the coast turns back across the lines")
    check(abs(sum(w for _, w in stretches) - length) < 1e-9, "the stretches add up to the coast's length")
    sticks = []
    for g, w in stretches:
        k = math.ceil(w / STICK_LONGEST - 1e-9)
        if g[0] == "S":
            p, q = g[1], g[2]
            pts = [(p[0] + (q[0] - p[0]) * i / k, p[1] + (q[1] - p[1]) * i / k) for i in range(k + 1)]
            sticks += [("S", a, b) for a, b in zip(pts, pts[1:])]
        else:
            c, r, a, b = g[1], g[2], g[3], g[4]
            sticks += [("A", c, r, a + (b - a) * i / k, a + (b - a) * (i + 1) / k) for i in range(k)]
    f["sticks.count"] = str(len(sticks))
    check(all(len(one_way_pieces([s], n0)) == 1 for s in sticks), f"each of the {len(sticks)} sticks runs one way across the lines at 0 degrees")
    for beta in SHOWN_DIRECTIONS:
        laid = sum(travel([s], beta) for s in sticks)
        check(abs(laid - travel(geom, beta)) < 1e-9, f"at {beta} degrees the sticks' shadows add up to the coast's travel {laid:.6f}")
    zero = travel(geom, 0)
    check(fixed(float(f["turn.0.reach"]) / GAP, 2) == f["turn.0.slide_mean"], f"10_8: the shown {f['turn.0.reach']} / {GAP:g} rounds to the shown {f['turn.0.slide_mean']}")
    check(zero / GAP > first == min(slide_counts(geom, 0)), f"10_6 to 10_9: the average {zero / GAP:.4f} is above the first count {first}, the slide's lowest")
    # 10_5: the bay at 0 degrees is two one-way pieces, down then back up;
    # the line through it crosses each once
    bay = one_way_pieces([geom[BAY]], n0)
    check(len(bay) == 2 and bay[0][1] < bay[0][0] and bay[1][1] > bay[1][0], "10_5: at 0 degrees the bay goes down and comes back up, two shadows")
    check(line_count(bay, first_slide - GAP) == 2 == int(f["open.bay_crossings"]), "10_5: the line through the bay crosses it twice, once on each way")
    check(abs(sum(abs(b - a) for a, b in bay) - travel([geom[BAY]], 0)) < 1e-12 and abs(abs(bay[0][1] - bay[0][0]) - geom[BAY][2]) < 1e-12, "10_5: both of the bay's shadows go in, the way down its radius")
    # 10_4 and 11_2: the beach along the lines, then across them
    beach = [geom[BEACH]]
    f["beach.shadow_0"] = fixed(travel(beach, 0), 0)
    f["beach.shadow_90"] = fixed(travel(beach, 90), 1)
    check(travel(beach, 0) < 1e-12 and abs(travel(beach, 90) - math.dist(geom[BEACH][1], geom[BEACH][2])) < 1e-12, "10_4 and 11_2: the beach casts no shadow at 0 degrees and its whole length at 90")
    
    # Chapter 11: one placement, the four slid directions, and grids of
    # evenly spread directions (numerical averages, not the identity)
    check(fixed(first * math.pi, 1) == f["open.estimate"], f"11_4: the shown {first} x pi rounds to the shown {f['open.estimate']}")
    shown_four = sum(float(f[f"turn.{b}.slide_mean"]) for b in SHOWN_DIRECTIONS) / len(SHOWN_DIRECTIONS)
    check(fixed(shown_four * math.pi, 2) == f["turn.slide_estimate"], f"11_5: the four shown averages, averaged and times pi, round to the shown {f['turn.slide_estimate']}")
    check(fixed(float(fixed(shown_four, 2)) * math.pi, 2) != f["turn.slide_estimate"], f"negative control: a shown mean {fixed(shown_four, 2)} x pi would give {fixed(float(fixed(shown_four, 2)) * math.pi, 2)}, so 11_5 shows none")
    four_est = slid_four * worth()
    f["turn.slide_short_percent"] = fixed((length - four_est) / length * 100, 0)
    check(four_est < length and abs(length - first * worth()) > abs(length - four_est), f"11_5: four directions give {four_est:.4f}, short of {length:.4f} and nearer it than one placement's {first * worth():.4f}")
    grid = {}
    for m in DIRECTION_GRID:
        grid[m] = sum(travel(geom, 180 * k / m) for k in range(m)) / m
        f[f"grid.{m}.estimate"] = fixed(grid[m] / GAP * worth(), 2)
    est = [grid[m] / GAP * worth() for m in DIRECTION_GRID]
    check(all(a < b for a, b in zip(est, est[1:])) and est[-1] < length, "11_7: more directions come nearer the length, from below: " + ", ".join(f"{e:.4f}" for e in est))
    check(f[f"grid.{DIRECTION_GRID[0]}.estimate"] == f["turn.slide_estimate"] == f["ladder.4.coast_estimate"], "11_7's four directions are 11_5's")
    check(f[f"grid.{DIRECTION_GRID[3]}.estimate"] == f["coast.length_2dp"] != f[f"grid.{DIRECTION_GRID[2]}.estimate"], "11_7: thirty-two directions match the length to two places, sixteen do not")
    most = DIRECTION_GRID[-1]
    f["grid.most"] = str(most)
    f[f"grid.{most}.mean_reach"] = fixed(grid[most], 3)
    f[f"grid.{most}.mean_crossings"] = fixed(grid[most] / GAP, 4)
    gap_to_identity = abs(grid[most] - 2 * length / math.pi)
    check(f[f"grid.{most}.mean_reach"] == f["crofton.mean_reach_3dp"] and 0 < gap_to_identity < 1e-5, f"11_8: {most} directions average {grid[most]:.9f}, {gap_to_identity:.1e} from 2L/pi: the same to three places, computed and not exact")
    check(fixed(float(f[f"grid.{most}.mean_reach"]) / GAP, 4) == f[f"grid.{most}.mean_crossings"], f"11_8: the shown {f[f'grid.{most}.mean_reach']} / {GAP:g} gives the shown {f[f'grid.{most}.mean_crossings']}")
    check(fixed(float(f[f"grid.{most}.mean_crossings"]) * math.pi, 2) == f[f"grid.{most}.estimate"] == f["coast.length_2dp"], f"11_8: the shown {f[f'grid.{most}.mean_crossings']} x pi gives {f['coast.length_2dp']}, the true length to two places")
    shadows = [travel(geom, 180 * k / most) for k in range(most)]
    check(14.5 <= min(shadows) and max(shadows) <= 17.7, f"11_6: the total shadow runs {min(shadows):.4f} to {max(shadows):.4f} cm, inside the plot's 14 to 18")

    # Chapters 12 and 13 (a function of its own, so its names never shadow
    # the ones main() uses after it)
    def chapters_12_13():
        # Chapter 12: why every curve, by Barbier's route (no appeal to the
        # identity: each step below is checked on its own; the identity only
        # appears at the end as a cross-check). Counts are piece by piece, as
        # one_way_pieces counts them: a line through two pieces at one point
        # counts for both
        f["barbier.year"] = str(BARBIER_YEAR)
        by_piece = [crossings([g], 0, first_slide) for g in geom]
        f["open.piece_counts"] = ",".join(map(str, by_piece))
        check(sum(by_piece) == first, f"12_3: the pieces' counts {by_piece} add up to the coast's {first}")
        check(by_piece[BAY] == 2 == int(f["open.bay_crossings"]) and by_piece[BEACH] == 0 and by_piece[HEADLAND] == 0, "12_3: the bay is crossed twice, the beach and the headland not at all")
        # crossings add, against a different count: the coast as a fine
        # polyline, each chord tested against each line directly
        fine = sample_points(geom, 512)
        worst_sum = 0
        for i in range(36):
            beta = 180 * (i + 0.5) / 36
            n = normal(beta)
            for j in range(25):
                slide = GAP * (j + 0.5) / 25
                hs = [p[0] * n[0] + p[1] * n[1] for p in fine]
                direct = sum(abs(math.floor((b - slide) / GAP) - math.floor((a - slide) / GAP)) for a, b in zip(hs, hs[1:]))
                pieces_sum = sum(crossings([g], beta, slide) for g in geom)
                worst_sum = max(worst_sum, abs(direct - pieces_sum))
        check(worst_sum == 0, f"12_4: at 900 placements the pieces' counts add up to a fine polyline's direct count (worst difference {worst_sum})")

        # 12_5: the stick moved right and turned, against the stick at home
        # with the sheet moved the other way (turned about home)
        def home_stick(angle, centre=STICK_HOME):
            return stick_geom(STICK, angle, centre)

        moved = home_stick(MOVE_TO, (STICK_HOME[0] + MOVE_RIGHT, STICK_HOME[1]))
        f["move.count"] = str(crossings(moved, 0, first_slide))
        n0, n1 = normal(0), normal(90 - MOVE_TO)
        back = first_slide - (n0[0] * MOVE_RIGHT) + (n1[0] - n0[0]) * STICK_HOME[0] + (n1[1] - n0[1]) * STICK_HOME[1]
        f["move.sheet_count"] = str(crossings(home_stick(90), 90 - MOVE_TO, back))
        check(f["move.count"] == f["move.sheet_count"] != f["stick.home_across"], f"12_5: moved, the stick crosses {f['move.count']}, not {f['stick.home_across']}; put back with the sheet moved the other way, {f['move.sheet_count']}")
        rng = np.random.default_rng(1213)
        worst_move = 0
        for _ in range(400):
            dx, dy, turn = rng.uniform(-1.5, 1.5), rng.uniform(-1.5, 1.5), rng.uniform(-90, 90)
            beta, slide = rng.uniform(0, 180), rng.uniform(0, GAP)
            stick_now = home_stick(90 + turn, (STICK_HOME[0] + dx, STICK_HOME[1] + dy))
            nb, nt = normal(beta), normal(beta - turn)
            sheet_back = slide - (nb[0] * dx + nb[1] * dy) + (nt[0] - nb[0]) * STICK_HOME[0] + (nt[1] - nb[1]) * STICK_HOME[1]
            worst_move = max(worst_move, abs(crossings(stick_now, beta, slide) - crossings(home_stick(90), beta - turn, sheet_back)))
        check(worst_move == 0, "12_5: at 400 random moves, moving the stick and moving the sheet the other way give the same count")

        # 12_7: the halves at home, across: their counts, and through a slide
        # of one gap they always add up to the whole stick's (except where a
        # line passes through the cut, a single slide)
        low = [("S", (STICK_HOME[0], STICK_HOME[1] - STICK / 2), STICK_HOME)]
        high = [("S", STICK_HOME, (STICK_HOME[0], STICK_HOME[1] + STICK / 2))]
        f["halves.counts"] = f"{crossings(low, 0, first_slide)},{crossings(high, 0, first_slide)}"
        halves_ok = all(crossings(low, 0, s) + crossings(high, 0, s) == crossings(home_stick(90), 0, s) for s in (first_slide + GAP * (i + 0.5) / 400 for i in range(400)))
        check(halves_ok and sum(map(int, f["halves.counts"].split(","))) == int(f["stick.home_across"]), f"12_7: the halves cross {f['halves.counts']}, and through a slide they add up to the whole's")
        # 12_8: the low half laid across the high one's middle; a line through
        # their meeting point counts once for each
        a = math.radians(CROSS_TO)
        crossed = [("S", (STICK_HOME[0] - STICK / 4 * math.cos(a), CROSS_AT - STICK / 4 * math.sin(a)), (STICK_HOME[0] + STICK / 4 * math.cos(a), CROSS_AT + STICK / 4 * math.sin(a)))]
        meet = (STICK_HOME[0], CROSS_AT)
        f["cross.count"] = str(crossings(high + crossed, 0, first_slide))
        through = CROSS_AT % GAP
        f["cross.through"] = str(crossings(high + crossed, 0, through))
        check(segments_cross(high[0][1], high[0][2], crossed[0][1], crossed[0][2]), "12_8: the two halves cross")
        check(f["cross.through"] == "2" and crossings(high, 0, through) == 1 == crossings(crossed, 0, through), "12_8: the line through their meeting point counts once for each half, two")
        points = {(round(x, 9), round(y, 9)) for g in high + crossed for x, y in [(g[1][0] + (CROSS_AT - g[1][1]) / (g[2][1] - g[1][1]) * (g[2][0] - g[1][0]), CROSS_AT)]}
        check(len(points) == 1 != int(f["cross.through"]), "negative control: counted as points of the card, the line through the meeting point would count one, not two")
        check(abs(sum(math.dist(g[1], g[2]) for g in high + crossed) - STICK) < 1e-12 and math.dist(meet, ((crossed[0][1][0] + crossed[0][2][0]) / 2, (crossed[0][1][1] + crossed[0][2][1]) / 2)) < 1e-12, "12_8: still 6 cm in all, the meeting point at the crossed half's middle")

        # 12_9: equal pieces have equal averages (by moving them), so a third
        # of the stick has a third of its average; three quarters, three
        # quarters. Checked numerically, each average over every slide and
        # direction
        def stick_mean(l):
            return simpson_mean(lambda b: travel(stick_geom(l, 90), b) / GAP, 0.0, 180.0, 3600)

        f["pieces.third"] = fixed(STICK / 3, 0)
        f["pieces.quarter"] = fixed(STICK / 4, 1)
        f["pieces.three_quarters"] = fixed(3 * STICK / 4, 1)
        m6, m2, m15, m45 = stick_mean(STICK), stick_mean(STICK / 3), stick_mean(STICK / 4), stick_mean(3 * STICK / 4)
        check(abs(3 * m2 - m6) < 1e-9 and abs(4 * m15 - m6) < 1e-9 and abs(m45 - 3 * m15) < 1e-9 and abs(m45 - 0.75 * m6) < 1e-9, f"12_9: 3 x {m2:.6f} = 4 x {m15:.6f} = {m6:.6f}, and 4.5 cm averages {m45:.6f}, three quarters of it")

        # 12_10 and 12_11: the straight line between the coast's ends, and the
        # 14 and 15 cm sticks along it from the coast's start, the sheet turned
        # a quarter; a longer stick is crossed at least as often as a shorter
        # one inside it, so the average is squeezed between tenths, hundredths
        start, end = fine[0], fine[-1]
        apart = math.dist(start, end)
        f["ends.apart_2dp"] = fixed(apart, 2)
        u = ((end[0] - start[0]) / apart, (end[1] - start[1]) / apart)

        def along(l):
            return [("S", start, (start[0] + l * u[0], start[1] + l * u[1]))]

        nested = [along(NESTED[0]), along(apart), along(NESTED[1])]
        for k in range(3):
            lo = Fraction(math.floor(apart * 10**k), 10**k)
            f[f"ends.bracket.{k}"] = f"{fixed(lo, k)},{fixed(lo + Fraction(1, 10**k), k)}"
            check(lo < apart < lo + Fraction(1, 10**k), f"12_11: {apart:.6f} lies between {f[f'ends.bracket.{k}']}")
        check(f["ends.bracket.1"].split(",")[0] == f["coast.ends_apart"], "12_10: the bracket's lower tenth is 03's 14.7")
        for i, s in enumerate(NESTED_SLIDES):
            f[f"ends.nested.{i}"] = ",".join(str(crossings(g, 90, s)) for g in nested)
            worst_end = min(clearance(g, 90, s) for g in nested)
            check(worst_end > 0.25, f"12_10: at 90 degrees, slide {s}: counts {f[f'ends.nested.{i}']}, every end {worst_end:.3f} cm from a line")
        check(f["ends.nested.0"] == "7,7,7" and len(set(f["ends.nested.1"].split(","))) > 1, "12_10: first all three alike, then the longer ones crossed more")
        order_bad = sum(not (crossings(nested[0], 180 * (i + 0.5) / 120, GAP * (j + 0.5) / 30) <= crossings(nested[1], 180 * (i + 0.5) / 120, GAP * (j + 0.5) / 30) <= crossings(nested[2], 180 * (i + 0.5) / 120, GAP * (j + 0.5) / 30)) for i in range(120) for j in range(30))
        check(order_bad == 0, "12_10: at 3,600 placements the 14 cm stick is never crossed more than the line between the ends, nor that more than the 15 cm stick")

        # 12_12 and 12_13: chains with their corners on the coast, at its
        # joints, at chapter 10's cuts, at 10's sticks' ends. Each chord is
        # crossed only by a line that crosses the coast between its ends, so a
        # chain is never crossed more than the coast
        def point_at(s):
            for g in geom:
                if g[0] == "S":
                    w = math.dist(g[1], g[2])
                    if s <= w + 1e-12:
                        return (g[1][0] + s / w * (g[2][0] - g[1][0]), g[1][1] + s / w * (g[2][1] - g[1][1]))
                else:
                    (cx, cy), r, a0, a1 = g[1], g[2], g[3], g[4]
                    w = r * abs(a1 - a0)
                    if s <= w + 1e-12:
                        t = a0 + (a1 - a0) * s / w
                        return (cx + r * math.cos(t), cy + r * math.sin(t))
                s -= w
            return end

        widths = [math.dist(g[1], g[2]) if g[0] == "S" else g[2] * abs(g[4] - g[3]) for g in geom]
        joints = [sum(widths[:i]) for i in range(len(geom) + 1)]
        cut_at = list(joints)
        for i, g in enumerate(geom):
            if g[0] == "A":
                lo_a, hi_a = sorted((g[3], g[4]))
                k = math.ceil((lo_a - math.pi / 2) / math.pi)
                while math.pi / 2 + k * math.pi < hi_a:
                    lev = math.pi / 2 + k * math.pi
                    if lev - lo_a > 1e-9 and hi_a - lev > 1e-9:
                        cut_at.append(joints[i] + g[2] * abs(lev - g[3]))
                    k += 1
        cut_at.sort()
        stick_at = []
        for a, b in zip(cut_at, cut_at[1:]):
            k = max(1, math.ceil((b - a) / STICK_LONGEST - 1e-9))
            stick_at += [a + (b - a) * i / k for i in range(k)]
        stick_at.append(cut_at[-1])
        chains = {}
        for name, at in (("joints", joints), ("cuts", cut_at), ("sticks", stick_at)):
            pts = [point_at(s) for s in at]
            chain = [("S", p, q) for p, q in zip(pts, pts[1:])]
            chains[name] = chain
            f[f"chain.{name}.chords"] = str(len(chain))
            f[f"chain.{name}.length"] = fixed(sum(math.dist(p, q) for p, q in zip(pts, pts[1:])), 2)
            f[f"chain.{name}.crossings"] = str(crossings(chain, 0, first_slide))
        check(f["chain.sticks.chords"] == f["sticks.count"], f"12_13: the finest chain has a corner at each of 10's {f['sticks.count']} sticks' ends")
        lengths = [float(f[f"chain.{n}.length"]) for n in ("joints", "cuts", "sticks")]
        check(lengths == sorted(lengths) and lengths[-1] < length, f"12_13: the chains' lengths {lengths} rise towards the coast's {length:.4f} from below")
        check(crossings([chains["joints"][BAY]], 0, first_slide) == 0 and f["chain.joints.crossings"] == str(first - by_piece[BAY]), "12_12: the joints chain's chord across the bay misses both of the bay's crossings, and the chain counts the rest")
        check(f["chain.cuts.crossings"] == f["chain.sticks.crossings"] == str(first), "12_13: the finer chains catch all seven")
        more, caught = 0, {n: 0 for n in chains}
        for i in range(90):
            beta = 180 * (i + 0.5) / 90
            for j in range(40):
                slide = GAP * (j + 0.5) / 40
                c = crossings(geom, beta, slide)
                for n, chain in chains.items():
                    k = crossings(chain, beta, slide)
                    more += k > c
                    caught[n] += k == c
        check(more == 0 and caught["joints"] < caught["cuts"] < caught["sticks"] < 3600, f"12_13: at 3,600 placements no chain is crossed more than the coast, and finer chains match it more often {caught}")
        # each chain is a polygon: its average is exactly its length x 1/pi on
        # this sheet (by 12_9's argument); checked numerically for the coarsest
        joints_mean = simpson_mean(lambda b: travel(chains["joints"], b) / GAP, 0.0, 180.0, 3600)
        check(abs(joints_mean - float(sum(math.dist(g[1], g[2]) for g in chains["joints"])) * 2 / (math.pi * GAP)) < 1e-6, f"12_12: the joints chain averages {joints_mean:.6f}, its length over pi")

        # 12_15 and 12_16: the coin calibrates: two crossings at every
        # placement, a rim of 2 pi; so 1/pi a centimetre, and the coast's
        # shown length over pi gives the shown average
        check(fixed(float(f["coast.length_2dp"]) / math.pi, 2) == f["crofton.mean_crossings"], f"12_16: the shown {f['coast.length_2dp']} / pi rounds to the shown {f['crofton.mean_crossings']}")
        check(abs(int(f["coin.crossings"]) / rim - 1 / math.pi) < 1e-15, "12_15: two crossings for a rim of 2 pi is 1/pi a centimetre")

        # Chapter 13: the same 6 cm wire bent at its middle into a right angle
        # at home, its right arm along the lines and its left arm turned up
        # (swung clockwise about the middle, through the open upper left)
        corner = STICK_HOME
        bent = [("S", (corner[0] + STICK / 2, corner[1]), corner), ("S", corner, (corner[0], corner[1] + STICK / 2))]
        f["hook.home_count"] = str(crossings(bent, 0, first_slide))
        check(f["stick.home_along"] == "0" != f["hook.home_count"], f"13_2: lying along, the straight stick crosses 0; bent up, {f['hook.home_count']}")
        pin = (corner[0], first_slide)
        sweep = []
        for i in range(3601):
            beta = 180 * i / 3600
            nb = normal(beta)
            k = crossings(bent, beta, first_slide + (nb[0] - n0[0]) * pin[0] + (nb[1] - n0[1]) * pin[1])
            if not sweep or sweep[-1] != k:
                sweep.append(k)
        f["hook.sweep"] = ",".join(map(str, sweep))
        check(sweep[0] == sweep[-1] == int(f["hook.home_count"]), f"13_3: half a turn of the sheet about the corner's line, counts {f['hook.sweep']}, back where it began")
        straight_sh, bent_sh = count_shares(stick_geom(STICK, 90), GAP), count_shares(bent, GAP)
        for name, sh in (("stick", straight_sh), ("hook", bent_sh)):
            f[f"{name}.shares_percent"] = ",".join(fixed(sh.get(k, 0.0) * 100, 1) for k in range(min(sh), max(sh) + 1))
            check(abs(moments(sh)[0] - STICK * 2 / (math.pi * GAP)) < 1e-9, f"13_4: the {name}'s closed-form chances average {moments(sh)[0]:.9f}, 6 / pi")
            grid = grid_shares(stick_geom(STICK, 90) if name == "stick" else bent, 3600, GAP)
            coarse = grid_shares(stick_geom(STICK, 90) if name == "stick" else bent, 36, GAP)
            gap_fine = max(abs(grid.get(k, 0) - sh.get(k, 0)) for k in set(grid) | set(sh))
            gap_coarse = max(abs(coarse.get(k, 0) - sh.get(k, 0)) for k in set(coarse) | set(sh))
            check(gap_fine < 1e-6 < 1e-4 < gap_coarse, f"13_4: the {name}'s chances over 3,600 directions agree with the closed form within {gap_fine:.1e}; over 36 they miss by {gap_coarse:.1e} (a direction grid is an estimate, not exact)")
        check(min(bent_sh) == 1 and 0 not in bent_sh and STICK / 2 / math.sqrt(2) > GAP, f"13_4: bent, it never misses: one arm always reaches {STICK / 2 / math.sqrt(2):.3f} cm across, more than a gap")
        check(f["hook.counts"] == ",".join(map(str, sorted(bent_sh))) and f["stick.all_counts"] == ",".join(map(str, sorted(straight_sh))), "13_4: the closed form's counts are the sampled ones")
        # 13_5: bent again into a "4" that crosses itself, flat on the card
        pts = [(corner[0] + FOUR_PIECES[0], corner[1]), corner]
        heading = 180.0
        for w in FOUR_PIECES[1:]:
            heading -= FOUR_TURN
            h = math.radians(heading)
            pts.append((pts[-1][0] + w * math.cos(h), pts[-1][1] + w * math.sin(h)))
        four = [("S", p, q) for p, q in zip(pts, pts[1:])]
        check(abs(sum(math.dist(g[1], g[2]) for g in four) - STICK) < 1e-12 and segments_cross(four[0][1], four[0][2], four[2][1], four[2][2]), "13_5: the 4 is still 6 cm and its last piece crosses its first")
        x_meet = (pts[2][0], corner[1])
        nf = normal(FOUR_SHEET)
        turned = first_slide + (nf[0] - n0[0]) * x_meet[0] + (nf[1] - n0[1]) * x_meet[1]
        shift = (x_meet[0] * nf[0] + x_meet[1] * nf[1] - turned) % GAP
        f["four.count"] = str(crossings(four, FOUR_SHEET, turned + shift))
        f["four.through"] = str(crossings([four[0]], FOUR_SHEET, turned + shift) + crossings([four[2]], FOUR_SHEET, turned + shift))
        four_sh = count_shares(four, GAP)
        f["four.mean"] = fixed(moments(four_sh)[0], 2)
        check(f["four.mean"] == f["hook.mean"] == f["stick.turned_mean"] and abs(moments(four_sh)[0] - STICK / math.pi) < 1e-9, f"13_5: the 4 averages {moments(four_sh)[0]:.9f}, its length over pi")
        check(f["four.through"] == "2", f"13_5: the line through the 4's crossing counts both pieces there ({f['four.count']} in all)")
        f["noodle.year"] = str(NOODLE_YEAR)
        # negative controls for 13
        folded = [("S", (-1.5, 0.0), (1.5, 0.0)), ("S", (1.5, 0.0), (-1.5, 0.0))]
        check(abs(moments(count_shares(folded, GAP))[0] - STICK / math.pi) < 1e-9 and abs(moments(count_shares(folded[:1], GAP))[0] - STICK / 2 / math.pi) < 1e-9, "negative control: folded double, counted per layer it averages 6/pi; counted per point of the card, 3/pi, as if 3 cm long")
        short = [("S", (-2.9, 0.0), (0.0, 0.0)), ("S", (0.0, 0.0), (0.0, 3.0))]
        check(fixed(moments(count_shares(short, GAP))[0], 2) != f["hook.mean"], f"negative control: a bend that loses 1 mm averages {fixed(moments(count_shares(short, GAP))[0], 2)}, not {f['hook.mean']}")
        for g in (1.0, 4.0):
            mg = moments(count_shares(bent, g))[0]
            check(abs(mg * worth(g) - STICK) < 1e-9 and fixed(mg, 2) != f["hook.mean"] and abs(mg * worth() - STICK) > 1.0, f"spacing control: lines {g:g} cm apart, the bent wire averages {mg:.4f}; times that sheet's worth {worth(g):.4f} it is 6 cm, times this sheet's pi it is not")
        four_dirs = sum(travel(bent, b) for b in (0, 45, 90, 135)) / 4 / GAP
        check(fixed(four_dirs, 2) != f["hook.mean"] and abs(four_dirs * math.pi - STICK) > 0.1, f"negative control: four directions estimate {four_dirs:.4f}, times pi {four_dirs * math.pi:.3f}, not 6: an estimate, not exact")
        # every wire shown keeps clear of the coast on the card
        coast_pts = sample_points(geom, 256)

        def clear_of_coast(wire):
            return min(min(math.dist(p, (g[1][0] + t / 40 * (g[2][0] - g[1][0]), g[1][1] + t / 40 * (g[2][1] - g[1][1]))) for t in range(41)) for g in wire for p in coast_pts)

        worst_wire = min(clear_of_coast(w) for w in (moved, low + high, high + crossed, bent, four))
        check(worst_wire > 0.3, f"12 and 13: every wire shown keeps {worst_wire:.3f} cm from the coast")

    chapters_12_13()
    f["steinhaus.directions"] = str(STEINHAUS_DIRECTIONS)
    f["steinhaus.angle"] = fixed(180 / STEINHAUS_DIRECTIONS, 0)
    f["steinhaus.gap_mm"] = fixed(12 / math.pi, 2)
    # six families of lines, 12/pi mm apart: average crossings per millimetre
    per_mm = STEINHAUS_DIRECTIONS * 2 / (math.pi * (12 / math.pi))
    f["steinhaus.crossings_per_mm"] = fixed(per_mm, 0)

    # curves of constant width: the coin and the Reuleaux triangle
    reuleaux = 3 * COIN * math.pi / 3
    f["reuleaux.width"] = fixed(COIN, 0)
    f["reuleaux.arcs"] = "3"
    f["reuleaux.arc_degrees"] = "60"
    f["reuleaux.rim"] = fixed(reuleaux, 2)
    f["coin.width_2dp"] = fixed(COIN, 2)
    f["reuleaux.width_2dp"] = fixed(COIN, 2)
    f["triangle.width_flat"] = fixed(COIN * math.sqrt(3) / 2, 2)
    f["triangle.width_upright"] = fixed(COIN, 2)
    f["triangle.perimeter"] = fixed(3 * COIN, 0)
    f["reuleaux.chain_corners"] = "3,6,12"
    f["reuleaux.chain_lengths"] = ",".join(fixed(2 * COIN * n * math.sin(math.pi / (2 * n)), 2) for n in (3, 6, 12))
    # A convex body of width GAP has one interior grid line except at
    # the isolated tangent slide, hence two transverse rim crossings.
    # The independent chapter checker also checks the exact shown path.
    f["reuleaux.lines_between"] = "1"
    f["reuleaux.home_count"] = "2"
    f["reuleaux.wander_counts"] = "2"
    f["reuleaux.mean_crossings"] = fixed(2 * reuleaux / (math.pi * GAP), 0)
    check(abs(reuleaux - rim) < 1e-12, "the Reuleaux rim equals the coin's rim")
    # its width really is the same in every direction: support function check
    # its width is the same in every direction: sample its rim (three arcs,
    # each centred on a corner and running between the other two) and
    # measure the spread of the rim across 720 directions
    corners = [
        (COIN / math.sqrt(3) * math.cos(math.radians(90 + 120 * k)), COIN / math.sqrt(3) * math.sin(math.radians(90 + 120 * k)))
        for k in range(3)
    ]
    rim_points = []
    for k, c in enumerate(corners):
        p, q = (corners[j] for j in range(3) if j != k)
        a0 = math.atan2(p[1] - c[1], p[0] - c[0])
        sweep = (math.atan2(q[1] - c[1], q[0] - c[0]) - a0 + math.pi) % (2 * math.pi) - math.pi
        check(abs(abs(sweep) - math.pi / 3) < 1e-12, f"Reuleaux arc {k} turns 60 degrees")
        rim_points += [(c[0] + COIN * math.cos(a0 + sweep * j / 240), c[1] + COIN * math.sin(a0 + sweep * j / 240)) for j in range(241)]
    widths = []
    for i in range(720):
        u = (math.cos(math.pi * i / 720), math.sin(math.pi * i / 720))
        hs = [x * u[0] + y * u[1] for x, y in rim_points]
        widths.append(max(hs) - min(hs))
    check(max(widths) - min(widths) < 1e-3 and abs(min(widths) - COIN) < 1e-3, f"Reuleaux width spans {min(widths):.5f}..{max(widths):.5f}")
    # Random nonexceptional placements have two rim crossings, not two lines.
    reuleaux_counts = set()
    rng = np.random.Generator(np.random.PCG64DXSM(11))
    for _ in range(4000):
        t = math.pi * rng.random()
        slide = GAP * rng.random()
        u = (-math.sin(t), math.cos(t))
        hs = [x * u[0] + y * u[1] for x, y in rim_points]
        lo, hi = min(hs), max(hs)
        reuleaux_counts.add(2 * sum(1 for k in range(-4, 5) if lo < slide + k * GAP < hi))
    check(reuleaux_counts == {2}, f"the Reuleaux triangle crosses {sorted(reuleaux_counts)} times")

    # random drops of the card on the sheet
    rng = np.random.Generator(np.random.PCG64DXSM(SEED_DROPS))
    drops = []
    for _ in range(MANY_DROPS):
        beta = 180.0 * rng.random()
        slide = GAP * rng.random()
        drops.append(crossings(geom, beta, slide))
    few = drops[:FEW_DROPS]
    f["drops.seed"] = str(SEED_DROPS)
    f["drops.few"] = str(FEW_DROPS)
    f["drops.few.counts"] = ",".join(map(str, few))
    f["drops.few.mean"] = fixed(Fraction(sum(few), FEW_DROPS), 1)
    f["drops.few.estimate"] = fixed(sum(few) / FEW_DROPS * worth(), 1)
    f["drops.few.lowest"] = str(min(few))
    f["drops.few.highest"] = str(max(few))
    f["drops.many"] = str(MANY_DROPS)
    f["drops.sample_range"] = f"{min(drops)}..{max(drops)}"
    f["drops.error_cm"] = "1"
    f["drops.repeat_seed"] = str(SEED_DROPS + 2)
    many_mean = sum(drops) / MANY_DROPS
    sd = math.sqrt(sum((x - many_mean) ** 2 for x in drops) / (MANY_DROPS - 1))
    f["drops.many.mean"] = fixed(many_mean, 2)
    f["drops.many.estimate"] = fixed(many_mean * worth(), 1)
    # every placement at once: each count's chance, in closed form. The
    # spreads the film says belong to the method, so they come from these
    # chances; the seeded draws are pictures, checked against them.
    shares = {}
    for gap in WIDE_GAPS:
        shares[gap] = count_shares(geom, gap)
        mean_g, sd_g = moments(shares[gap])
        check(abs(sum(shares[gap].values()) - 1) < 1e-12 and abs(mean_g - 2 * length / (math.pi * gap)) < 1e-9, f"{gap:g} cm sheet: the chances add to 1 and average {mean_g:.10f} crossings, 2L/(pi gap)")
    mean_c, sd_c = moments(shares[GAP])
    one = sd_c * worth()
    f["drops.one.spread"] = fixed(one, 1)
    f["drops.many.spread"] = fixed(one / math.sqrt(MANY_DROPS), 2)
    f["drops.few.spread"] = fixed(one / math.sqrt(FEW_DROPS), 1)
    # drops for a spread of a tenth of a centimetre: the spread falls as one
    # over the square root of the number of drops
    for_tenth = math.ceil((one / 0.1) ** 2)
    f["drops.for_tenth"] = str(for_tenth)
    f["drops.for_tenth_about"] = str(round(for_tenth, -3))
    check(one / math.sqrt(for_tenth) <= 0.1 < one / math.sqrt(for_tenth - 1), f"{for_tenth} drops bring the spread {one:.6f} to a tenth, {for_tenth - 1} do not")
    f["drops.count_range"] = f"{min(shares[GAP])}..{max(shares[GAP])}"
    f["drops.top_count_percent"] = fixed(shares[GAP][max(shares[GAP])] * 100, 3)
    # the seeded thousand: their spread, against the closed form's (the
    # standard error of a sample spread is sqrt(mu4 - sd^4) / (2 sd sqrt n))
    mu4 = sum((k - mean_c) ** 4 * p for k, p in shares[GAP].items())
    se_sd = math.sqrt(mu4 - sd_c**4) / (2 * sd_c * math.sqrt(MANY_DROPS))
    check(abs(sd - sd_c) < 4 * se_sd, f"the seeded thousand spread {sd * worth():.4f} cm a drop, the closed form {one:.4f} (standard error {se_sd * worth():.4f})")
    check(set(drops) <= set(shares[GAP]), f"every seeded count, {min(drops)} to {max(drops)}, is one the closed form gives, {f['drops.count_range']}")
    # repeat the ten-drop experiment: how often is it more than 1 cm off?
    rng = np.random.Generator(np.random.PCG64DXSM(SEED_DROPS + 2))
    repeats, far = 1000, 0
    for _ in range(repeats):
        ten = [crossings(geom, 180.0 * rng.random(), GAP * rng.random()) for _ in range(FEW_DROPS)]
        far += abs(sum(ten) / FEW_DROPS * worth() - length) > 1.0
    f["drops.ten.repeats"] = str(repeats)
    f["drops.ten.far_percent"] = fixed(Fraction(far * 100, repeats), 0)
    # Numerically evaluate the ten-count convolution. The historical key name
    # "exact" distinguishes this calculation from the seeded sample, not exact
    # arithmetic: count_shares uses binary64 and a tiny-interval approximation.
    far_exact = sum(p for s, p in sum_shares(shares[GAP], FEW_DROPS).items() if abs(s / FEW_DROPS * worth() - length) > 1.0)
    f["drops.ten.far_exact_percent"] = fixed(far_exact * 100, 0)
    check(abs(far / repeats - far_exact) < 4 * math.sqrt(far_exact * (1 - far_exact) / repeats), f"the thousand seeded ten-drop runs miss by over 1 cm {far / repeats:.3f} of the time; every possible ten drops, {far_exact:.4f}")
    f["drops.few.within"] = fixed(abs(sum(few) / FEW_DROPS * worth() - length), 2)
    hist = {}
    for x in drops:
        hist[x] = hist.get(x, 0) + 1
    f["drops.many.histogram"] = ",".join(f"{k}:{hist[k]}" for k in sorted(hist))
    check(abs(many_mean - 2 * length / (math.pi * GAP)) < 4 * sd_c / math.sqrt(MANY_DROPS), "the 1000-drop mean is within four standard errors")

    # the direction grids, against the closed form: every shown spread,
    # range and share rounds the same at 1,800 directions (the Rust model's
    # grid) and on to 28,800
    for directions in (1800, 3600, 7200, 14400, 28800):
        worst = 0.0
        same = True
        for gap in WIDE_GAPS:
            grid = grid_shares(geom, directions, gap)
            worst = max(worst, abs(moments(grid)[1] - moments(shares[gap])[1]) * worth(gap), max(abs(grid.get(k, 0.0) - p) for k, p in shares[gap].items()))
            same &= fixed(moments(grid)[1] * worth(gap), 1) == fixed(moments(shares[gap])[1] * worth(gap), 1)
            same &= min(k for k, p in grid.items() if p > 0) == min(shares[gap]) and max(k for k, p in grid.items() if p > 0) == max(shares[gap])
        grid = grid_shares(geom, directions, GAP)
        g_one = moments(grid)[1] * worth()
        g_far = sum(p for s, p in sum_shares(grid, FEW_DROPS).items() if abs(s / FEW_DROPS * worth() - length) > 1.0)
        same &= math.ceil((g_one / 0.1) ** 2) == for_tenth and fixed(g_far * 100, 0) == f["drops.ten.far_exact_percent"]
        same &= fixed(grid[max(grid)] * 100, 3) == f["drops.top_count_percent"]
        g_lines = grid_shares(geom, directions)
        g_meet = sum(w for k, w in g_lines.items() if k > 0)
        worst = max(worst, abs(g_meet - meet), max(abs(g_lines.get(k, 0.0) - w) for k, w in lines.items()))
        same &= max(k for k, w in g_lines.items() if w > 1e-9) == line_max and fixed(g_lines[1] / g_meet * 100, 0) == f["chart.once_percent"]
        same &= fixed(g_meet / card_lines * 100, 0) == f["chart.meet_percent"] and fixed(g_meet, 2) == f["chart.coast_lines"]
        check(same and worst < 1e-4, f"{directions} directions: every shown spread, range and share as in closed form, largest difference {worst:.1e}")

    # wider gaps: the same average, rougher counts. The draws are the same
    # seed's, the slide scaled; the spreads are every placement's.
    for gap in WIDE_GAPS:
        rng = np.random.Generator(np.random.PCG64DXSM(SEED_DROPS))
        xs_ = []
        for _ in range(MANY_DROPS):
            beta = 180.0 * rng.random()
            slide = gap * rng.random()
            xs_.append(crossings(geom, beta, slide, gap))
        m_ = sum(xs_) / MANY_DROPS
        key = fixed(gap, 0)
        f[f"gap.{key}.mean"] = fixed(2 * length / (math.pi * gap), 2)
        f[f"gap.{key}.worth"] = fixed(worth(gap), 2)
        f[f"gap.{key}.drops_mean_estimate"] = fixed(m_ * worth(gap), 1)
        f[f"gap.{key}.one_drop_spread"] = fixed(moments(shares[gap])[1] * worth(gap), 1)
        f[f"gap.{key}.counts"] = f"{min(xs_)}..{max(xs_)}"
        f[f"gap.{key}.count_range"] = f"{min(shares[gap])}..{max(shares[gap])}"
    spreads = [moments(shares[gap])[1] * worth(gap) for gap in WIDE_GAPS]
    check(all(a < b for a, b in zip(spreads, spreads[1:])), f"a wider gap spreads one drop more: {[round(s, 4) for s in spreads]}")

    # spacing and scale controls (root's review, Mail 6787: an estimate
    # written as pi x mean travel / gap is right only at a 2 cm gap). Counts
    # here come from sliding the sheet and counting, not from travel.
    # 1. The gap cancels: a unit circle and the coast, measured on sheets
    #    1, 2 and 4 cm apart, give one length each.
    unit_circle = [("A", (0.0, 0.0), 1.0, 0.0, 2 * math.pi)]
    six = [180 * k / 6 for k in range(6)]
    for name, g, truth, directions, slides in (
        ("unit circle", unit_circle, 2 * math.pi, [0.0, 45.0], 4000),
        ("coast, six directions", geom, sum(travel(geom, b) for b in six) / 6 * math.pi / 2, six, 4000),
    ):
        for gap in (1.0, 2.0, 4.0):
            est = sum(slid_mean(g, b, gap, slides) for b in directions) / len(directions) * worth(gap)
            check(abs(est - truth) < 0.02, f"{name} on a {gap:g} cm sheet: {est:.4f} cm, against {truth:.4f}")
    # 2. Negative control: the expression the review flagged, evaluated on the
    #    same unit circle, gives 4 pi, 2 pi and pi, so check 1 would catch it
    flagged = {gap: math.pi * travel(unit_circle, 0.0) / gap for gap in (1.0, 2.0, 4.0)}
    check(max(flagged.values()) - min(flagged.values()) > 1.0, "pi x travel / gap moves with the gap: " + ", ".join(f"{gap:g} cm gives {v:.4f}" for gap, v in flagged.items()))
    # 3. Scale: enlarge the coast and the sheet together and every count is
    #    unchanged while the measured length grows by the same factor; enlarge
    #    the coast alone and the counts grow instead
    k = 1.5
    big = scaled(geom, k)
    check(abs(chord_length(big) - k * length) < 1e-5, f"the enlarged coast is {chord_length(big):.6f} cm, {k:g} times the coast")
    placements = [(b, s) for b in range(0, 180, 15) for s in (0.13, 0.71, 1.37)]
    same = sum(crossings(big, b, k * s, k * GAP) == crossings(geom, b, s) for b, s in placements)
    check(same == len(placements), f"enlarging coast and sheet together keeps {same} of {len(placements)} counts")
    small = sum(slid_mean(geom, b) for b in six) / 6
    together = sum(slid_mean(big, b, k * GAP) for b in six) / 6
    alone = sum(slid_mean(big, b) for b in six) / 6
    check(abs(together - small) < 0.005, f"six directions, coast and sheet enlarged together: mean count {together:.4f}, against {small:.4f}")
    check(abs(together * worth(k * GAP) - k * small * worth()) < 0.02, f"and the measured length grows {together * worth(k * GAP) / (small * worth()):.4f} times")
    check(abs(alone - k * small) < 0.01, f"the coast enlarged alone, on the 2 cm sheet: mean count {alone:.4f}, {alone / small:.4f} times as many")

    # two ways to draw random lines across the card
    rng = np.random.Generator(np.random.PCG64DXSM(SEED_LINES))
    pin_hits = [0, 0]
    for _ in range(LINES):
        line = pin_line(rng)
        pin_hits[0] += crosses(line, MIDDLE_STICK)
        pin_hits[1] += crosses(line, CORNER_STICK)
    even_hits = [0, 0]
    for _ in range(LINES):
        line = even_line(rng)
        even_hits[0] += crosses(line, MIDDLE_STICK)
        even_hits[1] += crosses(line, CORNER_STICK)
    f["lines.seed"] = str(SEED_LINES)
    f["lines.count"] = str(LINES)
    f["lines.stick"] = fixed(math.dist(*MIDDLE_STICK), 0)
    f["lines.pins.middle"] = str(pin_hits[0])
    f["lines.pins.corner"] = str(pin_hits[1])
    f["lines.even.middle"] = str(even_hits[0])
    f["lines.even.corner"] = str(even_hits[1])
    f["lines.chart_shown"] = str(LINES // 10)
    f["lines.half_turn"] = fixed(math.degrees(math.pi), 0)
    f["lines.reach"] = fixed(math.hypot(CARD_W / 2, CARD_H / 2), 1)
    even_rate = 2 * math.dist(*MIDDLE_STICK) / (2 * (CARD_W + CARD_H))
    f["lines.even.rate_percent"] = fixed(even_rate * 100, 0)
    for name, stick, hits in (("middle", MIDDLE_STICK, pin_hits[0]), ("corner", CORNER_STICK, pin_hits[1])):
        exact = pin_rate_exact(stick)
        f[f"lines.pins.{name}.rate_percent"] = fixed(exact * 100, 1)
        se = math.sqrt(exact * (1 - exact) / LINES)
        check(abs(hits / LINES - exact) < 4 * se, f"pins/{name}: {hits}/{LINES} against Crofton's chord formula {exact:.5f}")
    for name, hits in (("middle", even_hits[0]), ("corner", even_hits[1])):
        se = math.sqrt(even_rate * (1 - even_rate) / LINES)
        check(abs(hits / LINES - even_rate) < 4 * se, f"even/{name}: {hits}/{LINES} against 2 x stick / perimeter {even_rate:.4f}")

    # a finer drawing: the beach as a row of half-circle ripples
    beach = float(COAST[BEACH][1])
    rippled_beach = RIPPLES * math.pi * (beach / RIPPLES) / 2
    f["scale.ripples"] = str(RIPPLES)
    f["scale.beach"] = fixed(beach, 1)
    f["scale.rippled_beach"] = fixed(rippled_beach, 1)
    f["scale.rippled_length"] = fixed(length - beach + rippled_beach, 1)
    f["scale.factor"] = fixed(math.pi / 2, 2)
    check(abs(rippled_beach / beach - math.pi / 2) < 1e-12, "ripples of any size make the beach pi/2 times longer")
    rippled = rippled_coast()
    rgeom = build(COAST_START, COAST_HEADING, rippled)
    r_straight, r_pi = exact_length(rippled)
    r_length = float(r_straight) + float(r_pi) * math.pi
    f["scale.rippled_length_exact"] = f"{r_straight} + {r_pi}pi"
    check(abs(r_length - (length - beach + rippled_beach)) < 1e-12, f"the rippled drawing is {r_length:.4f} cm long")
    check(math.dist(sample_points(geom)[-1], sample_points(rgeom)[-1]) < 1e-9, "the ripples leave the coast's far end where it was")
    rpts = sample_points(rgeom)
    rpath = list(zip(rpts, rpts[1:]))
    meets = sum(1 for i in range(len(rpath)) for j in range(i + 2, len(rpath)) if segments_cross(*rpath[i], *rpath[j]))
    # each corner joins two chords that only touch, which is not a crossing
    check(meets == 0, f"the rippled coast never crosses itself ({meets} meetings)")
    r_mean = simpson_mean(lambda b: travel(rgeom, b) / GAP, 0.0, 180.0, 3600)
    f["scale.rippled_mean_crossings"] = fixed(r_mean, 2)
    check(abs(r_mean - 2 * r_length / (math.pi * GAP)) < 1e-6, "the sheet's average on the rippled drawing is its length over pi")
    for step in DIVIDER_STEPS:
        key = fixed(step, 1).replace(".", "_")
        n, walked = dividers(geom, float(step))
        rn, rwalked = dividers(rgeom, float(step))
        f[f"dividers.{key}.steps"] = str(n)
        f[f"dividers.{key}.length"] = fixed(walked, 1)
        f[f"dividers.{key}.rippled_length"] = fixed(rwalked, 1)
        check(walked < length and rwalked < r_length, f"dividers of {float(step):g} cm: {walked:.3f} and {rwalked:.3f}, each short of its drawing")
    walks = [dividers(geom, float(s))[1] for s in DIVIDER_STEPS]
    check(all(a <= b + 1e-9 for a, b in zip(walks, walks[1:])), f"the film's four steps walk further as they shrink: {[round(w, 3) for w in walks]} (true of these four, not a rule)")
    # not a rule: 1.9 cm dividers take as many steps as 2 cm ones, each a
    # little shorter, and walk less (25_6's counterexample). A square of side
    # 2 does the same with no arcs at all.
    n19, w19 = dividers(geom, 1.9)
    f["dividers.1_9.steps"] = str(n19)
    f["dividers.1_9.length"] = fixed(w19, 1)
    check(w19 < walks[1] and str(n19) == f["dividers.2_0.steps"], f"1.9 cm dividers: {n19} steps, {w19:.3f} cm, less than the 2 cm walk's {walks[1]:.3f}")
    sweep = [dividers(geom, 0.25 + 0.05 * i, per_arc=1024)[1] for i in range(76)]
    rises = sum(b > a for a, b in zip(sweep, sweep[1:]))
    check(0 < rises < len(sweep) - 1, f"from 0.25 to 4 cm in 0.05 cm steps, a longer step walks further at {rises} of {len(sweep) - 1} increments")
    square_outline = [("S", (0.0, 0.0), (2.0, 0.0)), ("S", (2.0, 0.0), (2.0, 2.0)), ("S", (2.0, 2.0), (0.0, 2.0)), ("S", (0.0, 2.0), (0.0, 0.0))]
    on_square = {s: dividers(square_outline, s)[1] for s in (1.9, 2.0)}
    check(on_square[1.9] < on_square[2.0] and abs(on_square[2.0] - 8.0) < 1e-9, f"a square of side 2: 1.9 cm dividers walk {on_square[1.9]:.4f}, 2 cm ones {on_square[2.0]:.4f}")
    # chapter 01's ruler, checked on 4096 chords per arc: under the beach
    # it touches along the beach and nowhere else in its span; on the
    # headland's top (the tangent at the middle of its arc) the whole coast
    # lies on one side and the ruler touches one run of points
    dense = sample_points(geom, 4096)
    beach_a, beach_b = geom[BEACH][1], geom[BEACH][2]
    mid_x = (beach_a[0] + beach_b[0]) / 2
    span = [p for p in dense if abs(p[0] - mid_x) <= RULER / 2]
    on = [p for p in span if abs(p[1] - beach_a[1]) < 1e-9]
    f["ruler.length"] = fixed(RULER, 0)
    f["ruler.beach_contact"] = fixed(max(p[0] for p in on) - min(p[0] for p in on), 1)
    check(abs(beach_a[1] - beach_b[1]) < 1e-12 and min(p[1] for p in span) >= beach_a[1] - 1e-9, f"the ruler under the beach fits along {max(p[0] for p in on) - min(p[0] for p in on):.6f} cm and the coast stays above it")
    (hx, hy), hr, ha0, ha1 = geom[HEADLAND][1:]
    mid = (ha0 + ha1) / 2
    u = (math.cos(mid), math.sin(mid))
    apex = (hx + hr * u[0], hy + hr * u[1])
    heights = [(p[0] - apex[0]) * u[0] + (p[1] - apex[1]) * u[1] for p in dense]
    along = [-(p[0] - apex[0]) * u[1] + (p[1] - apex[1]) * u[0] for p in dense]
    touching = [i for i, (h, a) in enumerate(zip(heights, along)) if h > -1e-6 and abs(a) <= RULER / 2]
    runs = 1 + sum(1 for i, j in zip(touching, touching[1:]) if j != i + 1) if touching else 0
    f["ruler.headland_touches"] = str(runs)
    check(max(heights) < 1e-12, f"the whole coast lies on one side of the headland's top tangent (highest {max(heights):.2e})")
    check(runs == 1 and max(math.dist(dense[i], apex) for i in touching) < 0.01, f"the ruler on the headland touches one run of {len(touching)} points, all within 0.01 cm of the top")
    clear = min(-h for h, a in zip(heights, along) if 0.5 <= abs(a) <= RULER / 2)
    check(clear > 0.1, f"half a centimetre along the ruler either way the coast is already {clear:.3f} cm below it")
    f["dividers.opening_step"] = fixed(OPENING_STEP, 0)

    # chapter 01's walk at 2 cm: the short last bit, and the step that
    # leaves the headland (where the ruler fitted at one point) with the
    # coast it skips
    landings, last = divider_walk(geom, 2.0)
    f["dividers.2_0.last"] = fixed(last, 2)
    check(abs((len(landings) - 1) * 2.0 + last - dividers(geom, 2.0)[1]) < 1e-12, f"{len(landings) - 1} steps of 2 cm and {last:.4f} make the walk")
    ends_at, acc = [], 0.0
    for p in COAST:
        acc += float(p[1]) if p[0] == "S" else float(p[1]) * math.radians(abs(p[2]))
        ends_at.append(acc)
    head = PIECE_NAMES.index("headland")
    head_from, head_to = ends_at[head - 1], ends_at[head]
    k = next(k for k in range(1, len(landings)) if head_from < landings[k - 1][1] < head_to)
    skipped = landings[k][1] - landings[k - 1][1]
    f["dividers.2_0.headland_step"] = str(k)
    f["dividers.2_0.headland_coast"] = fixed(skipped, 2)
    check(abs(math.dist(landings[k - 1][0], landings[k][0]) - 2.0) < 1e-9 and landings[k][1] > head_to, f"step {k} leaves the headland: 2 cm straight across {skipped:.4f} cm of coast, landing past it")
    check(all(b[1] - a[1] >= 2.0 - 1e-9 for a, b in zip(landings, landings[1:])), "every step skips at least its own 2 cm of coast")
    check(length - landings[-1][1] >= last - 1e-9, f"the last straight bit, {last:.4f}, is no longer than the {length - landings[-1][1]:.4f} cm of coast it cuts across")
    # halving the step from 1 cm to 0.5 cm on the drawn coast
    f["dividers.halving_percent"] = fixed((walks[3] / walks[2] - 1) * 100, 0)
    # Richardson's law for the west coast of Britain, as Mandelbrot 1967
    # reports it: length proportional to step^(1 - D), D = 1.25 (a sourced
    # constant, not computed here); halving the step multiplies the length by
    # 2^(D - 1)
    f["richardson.west_coast_d"] = fixed(RICHARDSON_D, 2)
    f["richardson.halving_percent"] = fixed((2 ** (RICHARDSON_D - 1) - 1) * 100, 0)
    # a finer rippling: twice as many ripples, half the size, the same length
    fine = rippled_coast(2 * RIPPLES)
    fine_beach = sum(p[1] * Fraction(abs(p[2]), 180) for p in fine[BEACH:BEACH + 3 * 2 * RIPPLES] if p[0] == "A")
    f["scale.ripples_fine"] = str(2 * RIPPLES)
    f["scale.rippled_beach_fine"] = fixed(float(fine_beach) * math.pi, 1)
    check(fine_beach * 2 == COAST[BEACH][1], f"{2 * RIPPLES} ripples of half the size: the beach is {fine_beach}pi, exactly pi/2 times {COAST[BEACH][1]}")
    fgeom = build(COAST_START, COAST_HEADING, fine)
    check(math.dist(sample_points(fgeom)[-1], sample_points(geom)[-1]) < 1e-9, "the finer ripples also leave the far end in place")

    # the closing question: a 2 cm square dropped on the sheet. Its width runs
    # from 2 to 2 sqrt 2, under two gaps, so it always crosses 2 or 4 times,
    # and its average 2 x 8 / (pi x 2) says how often it crosses 4
    square = [("S", (-1.0, -1.0), (1.0, -1.0)), ("S", (1.0, -1.0), (1.0, 1.0)), ("S", (1.0, 1.0), (-1.0, 1.0)), ("S", (-1.0, 1.0), (-1.0, -1.0))]
    sq_mean = 2 * 8.0 / (math.pi * GAP)
    four = (sq_mean - 2) / 2
    f["square.side"] = "2"
    f["square.rim"] = "8"
    f["square.mean"] = fixed(sq_mean, 2)
    f["square.four_share_percent"] = fixed(four * 100, 0)
    f["square.four_share_exact"] = "4/pi - 1"
    check(abs(four - (4 / math.pi - 1)) < 1e-12, f"on a {GAP:g} cm sheet the square's share of fours, {four:.6f}, is the printed 4/pi - 1")
    rng = np.random.Generator(np.random.PCG64DXSM(13))
    sq_counts = [crossings(square, 180.0 * rng.random(), GAP * rng.random()) for _ in range(20000)]
    check(set(sq_counts) == {2, 4}, f"the square crosses {sorted(set(sq_counts))} times")
    got = sq_counts.count(4) / len(sq_counts)
    check(abs(got - four) < 4 * math.sqrt(four * (1 - four) / len(sq_counts)), f"the square crosses 4 lines {got:.4f} of the time, against 4/pi - 1 = {four:.4f}")
    check(abs(simpson_mean(lambda b: travel(square, b) / GAP, 0.0, 180.0, 3600) - sq_mean) < 1e-9, "the square's average is 8/pi")

    for k, v in f.items():
        print(f"{k}\t{v}")
    print(f"# {len(f)} facts, {len(failures)} failed cross-checks", file=sys.stderr)
    return 1 if failures else 0


if __name__ == "__main__":
    sys.exit(main())
