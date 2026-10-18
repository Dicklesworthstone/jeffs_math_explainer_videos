#!/usr/bin/env python3
"""Independent values for every fact of the five-tile video (./x square_packing facts-check).

    python3 -I oracle.py            key<TAB>value lines, the format `square facts` prints

Rebuilt by a different route from src/model.rs, in exact arithmetic on numbers
a + b*sqrt(2) (Fractions a, b):
- the box side is solved from a different condition: the turned tile is the
  set |x - c| + |y - c| <= R (R its half-diagonal, c the box's centre), and
  the squeeze stops when a corner tile's inner corner lies on that boundary;
  the video derives it from the gap's diagonal instead;
- the turned tile is built directly as a diamond from its four vertices, not
  as a pose with a cosine and a sine;
- two tiles are compared by clipping one polygon against the other
  (Sutherland-Hodgman with closed half-planes): an empty result is apart, a
  result of zero area is a touch, a positive area an overlap. The video uses
  the separating-axis test;
- a tile is inside the box when every vertex is, and touches a wall when its
  nearest vertex is on it.

Every planted failure must come out as a failure, or this script stops.
"""

import math
import sys
from decimal import ROUND_HALF_EVEN, Decimal, getcontext
from fractions import Fraction

getcontext().prec = 60


class Q2:
    """a + b*sqrt(2), exactly."""

    __slots__ = ("a", "b")

    def __init__(self, a=0, b=0):
        self.a = Fraction(a)
        self.b = Fraction(b)

    def __add__(self, o):
        o = q2(o)
        return Q2(self.a + o.a, self.b + o.b)

    __radd__ = __add__

    def __neg__(self):
        return Q2(-self.a, -self.b)

    def __sub__(self, o):
        return self + (-q2(o))

    def __rsub__(self, o):
        return q2(o) - self

    def __mul__(self, o):
        o = q2(o)
        return Q2(self.a * o.a + 2 * self.b * o.b, self.a * o.b + self.b * o.a)

    __rmul__ = __mul__

    def __truediv__(self, o):
        o = q2(o)
        norm = o.a * o.a - 2 * o.b * o.b  # nonzero unless o is 0 (sqrt(2) is irrational)
        if norm == 0:
            raise ZeroDivisionError("division by zero in Q(sqrt 2)")
        conj = Q2(o.a / norm, -o.b / norm)
        return self * conj

    def sign(self):
        a, b = self.a, self.b
        if a >= 0 and b >= 0:
            return 1 if (a > 0 or b > 0) else 0
        if a <= 0 and b <= 0:
            return -1
        # opposite signs: compare a^2 with 2 b^2
        return (1 if a > 0 else -1) if a * a > 2 * b * b else (1 if b > 0 else -1)

    def __eq__(self, o):
        o = q2(o)
        return self.a == o.a and self.b == o.b

    def __hash__(self):
        return hash((self.a, self.b))

    def __lt__(self, o):
        return (self - o).sign() < 0

    def __le__(self, o):
        return (self - o).sign() <= 0

    def __gt__(self, o):
        return (self - o).sign() > 0

    def __ge__(self, o):
        return (self - o).sign() >= 0

    def decimal(self):
        root2 = Decimal(2).sqrt()
        frac = lambda f: Decimal(f.numerator) / Decimal(f.denominator)
        return frac(self.a) + frac(self.b) * root2

    def rounded(self, places):
        return str(self.decimal().quantize(Decimal(1).scaleb(-places), rounding=ROUND_HALF_EVEN))

    def __str__(self):
        def root(b):
            return "sqrt(2)" if b == 1 else f"{b}*sqrt(2)"

        if self.b == 0:
            return str(self.a)
        if self.a == 0:
            return root(self.b) if self.b > 0 else "-" + root(-self.b)
        return f"{self.a} + {root(self.b)}" if self.b > 0 else f"{self.a} - {root(-self.b)}"


def q2(x):
    return x if isinstance(x, Q2) else Q2(x)


SQRT2 = Q2(0, 1)
HALF = Fraction(1, 2)
HUNDREDTH = Fraction(1, 100)

# ---------------------------------------------------------------- polygons


def square(x0, y0):
    """The straight unit tile with lower-left corner (x0, y0), counter-clockwise."""
    x0, y0 = q2(x0), q2(y0)
    return [(x0, y0), (x0 + 1, y0), (x0 + 1, y0 + 1), (x0, y0 + 1)]


def diamond(cx, cy, r):
    """The tile turned an eighth: vertices r from (cx, cy) along the axes, counter-clockwise."""
    cx, cy, r = q2(cx), q2(cy), q2(r)
    return [(cx + r, cy), (cx, cy + r), (cx - r, cy), (cx, cy - r)]


def cross(a, b, p):
    """(b - a) x (p - a): positive when p is left of the directed edge a -> b."""
    return (b[0] - a[0]) * (p[1] - a[1]) - (b[1] - a[1]) * (p[0] - a[0])


def clip(subject, clipper):
    """The closed intersection of two convex counter-clockwise polygons, as a
    (possibly degenerate) vertex list; empty when they are apart."""
    out = list(subject)
    n = len(clipper)
    for i in range(n):
        a, b = clipper[i], clipper[(i + 1) % n]
        pts, out = out, []
        for j, p in enumerate(pts):
            q = pts[(j + 1) % len(pts)]
            sp, sq = cross(a, b, p), cross(a, b, q)
            if sp.sign() >= 0:
                out.append(p)
            if sp.sign() * sq.sign() < 0:
                t = sp / (sp - sq)
                out.append((p[0] + t * (q[0] - p[0]), p[1] + t * (q[1] - p[1])))
        if not out:
            return []
    return out


def area(poly):
    total = Q2(0)
    for i, p in enumerate(poly):
        q = poly[(i + 1) % len(poly)]
        total = total + (p[0] * q[1] - q[0] * p[1])
    return total * HALF


def relation(p, q):
    """'apart', 'touch' or 'overlap' for two tiles."""
    inter = clip(p, q)
    if not inter:
        return "apart"
    s = area(inter).sign()
    if s < 0:
        sys.exit("oracle: a clipped polygon came out clockwise")
    return "overlap" if s > 0 else "touch"


def walls_touched(poly, side):
    out = []
    for name, depth in (("left", lambda v: v[0]), ("bottom", lambda v: v[1]),
                        ("right", lambda v: side - v[0]), ("top", lambda v: side - v[1])):
        if min(depth(v) for v in poly) == 0:
            out.append(name)
    return out


def escaped(poly, side):
    return any(v[0] < 0 or v[1] < 0 or v[0] > side or v[1] > side for v in poly)


def verdict(side, tiles):
    """'fits', or what broke, in the video's format."""
    overlaps = [f"{i}-{j}" for i in range(len(tiles)) for j in range(i + 1, len(tiles))
                if relation(tiles[i], tiles[j]) == "overlap"]
    out = [i for i, t in enumerate(tiles) if escaped(t, side)]
    parts = []
    if overlaps:
        parts.append("overlap:" + ",".join(overlaps))
    if out:
        parts.append("escape:" + ",".join(map(str, out)))
    return ";".join(parts) or "fits"


def touching(tiles):
    return ",".join(f"{i}-{j}" for i in range(len(tiles)) for j in range(i + 1, len(tiles))
                    if relation(tiles[i], tiles[j]) == "touch")


def corner_tiles(side):
    """Lower left, lower right, upper left, upper right: the video's order."""
    s = q2(side)
    return [square(0, 0), square(s - 1, 0), square(0, s - 1), square(s - 1, s - 1)]


def rotate(poly, centre, cos, sin):
    cx, cy = centre
    return [(cx + cos * (x - cx) - sin * (y - cy), cy + sin * (x - cx) + cos * (y - cy)) for x, y in poly]


# ------------------------------------------------------------------- facts


def main():
    out = []
    put = lambda k, v: out.append(f"{k}\t{v}")

    # the straight grid
    n = 5
    full = math.isqrt(n)
    side3 = full if full * full == n else full + 1
    put("tiles", n)
    put("grid.full", full)
    put("grid.full.holds", full * full)
    put("grid.side", side3)
    put("grid.room", side3 * side3 - n)
    straight = corner_tiles(side3) + [square(Fraction(side3, 2) - HALF, Fraction(side3, 2) - HALF)]
    put("straight.check", verdict(q2(side3), straight))
    put("straight.touching", touching(straight))

    # the turn in the box of three: the turning tile's corners stay on the
    # circle of its half-diagonal, which passes through every inner corner
    put("turn.reach2", HALF * HALF + HALF * HALF)
    c3 = Fraction(side3, 2)
    put("turn.inner2", (1 - c3) ** 2 + (1 - c3) ** 2)

    # the turned tile: half-diagonal R; the squeeze stops when the inner
    # corner (1, 1) is on the boundary |x - c| + |y - c| = R, i.e. 2(c - 1) = R
    r = SQRT2 * HALF  # sqrt(1/2): (r * r == 1/2) is checked below
    assert r * r == Q2(HALF)
    c = 1 + r * HALF
    side = 2 * c
    tiles = corner_tiles(side) + [diamond(c, c, r)]
    edge = (tiles[4][1][0] - tiles[4][0][0], tiles[4][1][1] - tiles[4][0][1])
    put("turned.degrees", round(math.degrees(math.atan2(abs(float(edge[1].decimal())), abs(float(edge[0].decimal()))))))
    gap = side - 2
    put("turned.gap", gap)
    put("turned.gap.2dp", gap.rounded(2))
    put("turned.side", side)
    put("turned.side.2dp", side.rounded(2))
    put("turned.side.5dp", side.rounded(5))
    inner = [(q2(1), q2(1)), (side - 1, q2(1)), (q2(1), side - 1), (side - 1, side - 1)]
    a, b = inner[0], inner[3]
    put("turned.diagonal2", (b[0] - a[0]) * (b[0] - a[0]) + (b[1] - a[1]) * (b[1] - a[1]))
    put("turned.check", verdict(side, tiles))
    put("turned.touching", touching(tiles))
    put("turned.walls", sum(len(walls_touched(t, side)) for t in tiles))
    d = tiles[4]
    mids = [((d[k][0] + d[(k + 1) % 4][0]) * HALF, (d[k][1] + d[(k + 1) % 4][1]) * HALF) for k in range(4)]
    put("turned.midpoints", sum(1 for p in inner if p in mids))

    # the planted failures, each built here by its own route
    small = (Fraction(399, 401), Fraction(40, 401))  # tan(half angle) = 1/20
    assert small[0] ** 2 + small[1] ** 2 == 1
    narrow = side - HUNDREDTH
    c_narrow = narrow * HALF
    straight_side = side3 - HUNDREDTH
    planted = {
        "squeeze_further": (narrow, corner_tiles(narrow) + [diamond(c_narrow, c_narrow, r)]),
        "middle_turned_more": (side, corner_tiles(side) + [rotate(diamond(c, c, r), (c, c), q2(small[0]), q2(small[1]))]),
        "middle_shifted": (side, corner_tiles(side) + [diamond(c + HUNDREDTH, c, r)]),
        "box_smaller": (side - HUNDREDTH, tiles),
        "straight_squeezed": (q2(straight_side), corner_tiles(straight_side)
                              + [square(straight_side * HALF - HALF, straight_side * HALF - HALF)]),
    }
    for key, (s, ts) in planted.items():
        v = verdict(q2(s), ts)
        if v == "fits":
            sys.exit(f"oracle: the planted failure {key} was not caught")
        put(f"planted.{key}", v)

    # chapter 02, the four pins, by gaps rather than shadows: cut [0, L] at
    # L/3 and 2L/3; an open unit interval with no mark inside it exists
    # exactly when some piece between neighbouring cut points is at least one
    # tile long, and a closed one misses every mark exactly when some piece is
    # longer than one tile
    def pieces(length):
        cuts = sorted(q2(x) for x in (0, length * Fraction(1, 3), length * Fraction(2, 3), length))
        return [b - a for a, b in zip(cuts, cuts[1:])]

    def open_step(length):  # every shadow holds a mark strictly inside
        if q2(length) < 1:
            return "no room"
        return "yes" if all(p < 1 for p in pieces(length)) else "no"

    def closed_step(length):  # every closed shadow touches a mark
        if q2(length) < 1:
            return "no room"
        return "yes" if all(p <= 1 for p in pieces(length)) else "no"

    piece = max(pieces(side))
    put("pins.count", len([(x, y) for x in (1, 2) for y in (1, 2)]))
    put("pins.piece", piece)
    put("pins.piece.2dp", piece.rounded(2))
    # the threshold by a scan of widths in hundredths, exactly
    threshold = next(Fraction(k, 100) for k in range(100, 1001) if max(pieces(Fraction(k, 100))) >= 1)
    put("pins.threshold", threshold)
    put("pins.lemma", open_step(side))
    put("pins.lemma.at3", open_step(3))
    put("pins.lemma.half", open_step(Fraction(1, 2)))
    put("pins.closed.at3", closed_step(3))
    # straight tiles: the step on the floor and on the wall at once
    put("pins.straight_lemma", open_step(side))
    put("pins.straight_lemma.at3", open_step(3))

    def strictly_in(poly, p):  # strictly inside a counter-clockwise convex polygon
        return all(cross(poly[i], poly[(i + 1) % len(poly)], p).sign() > 0 for i in range(len(poly)))

    def owners(length, tiles):
        m1, m2 = q2(length) * Fraction(1, 3), q2(length) * Fraction(2, 3)
        pin_list = [(m1, m1), (m2, m1), (m1, m2), (m2, m2)]
        rows = []
        for t in tiles:
            held = [str(i) for i, p in enumerate(pin_list) if strictly_in(t, p)]
            rows.append(",".join(held) or "none")
        return "|".join(rows)

    straight_middle = corner_tiles(side) + [square(c - HALF, c - HALF)]
    put("pins.owners.straight", owners(side, straight_middle))
    put("pins.owners.turned", owners(side, tiles))
    put("pins.owners.at3", owners(side3, straight))

    # chapter 03, a turn by a hair, by distances to edges rather than tile
    # coordinates: the box of 2.71, the corner tiles and the middle tile's
    # centre held; the middle tile's diamond turned about that centre
    def turned(cd, sd, centre=c):
        return rotate(diamond(centre, centre, r), (centre, centre), q2(cd), q2(sd))

    def depth_in(poly, p):  # the least distance from p to an edge (edges are a tile long)
        return min(cross(poly[i], poly[(i + 1) % len(poly)], p) for i in range(len(poly)))

    def half_angle(n):  # the turn whose half-angle has tangent 1/n
        return Fraction(n * n - 1, n * n + 1), Fraction(2 * n, n * n + 1)

    centre = (c, c)
    held = [(q2(1), q2(1)), (side - 1, q2(1)), (q2(1), side - 1), (side - 1, side - 1)]
    put("hair.inner2", (held[0][0] - c) * (held[0][0] - c) + (held[0][1] - c) * (held[0][1] - c))
    flat_tile = tiles[4]
    edge_gap = cross(flat_tile[0], flat_tile[1], centre)
    put("hair.inradius2", edge_gap * edge_gap)
    cd, sd = half_angle(20)
    put("hair.turn", f"{cd},{sd}")
    cd2, sd2 = cd * cd - sd * sd, 2 * cd * sd
    put("hair.degrees.1dp", f"{math.degrees(math.atan2(sd, cd)):.1f}")
    put("hair.degrees2.1dp", f"{math.degrees(math.atan2(sd2, cd2)):.1f}")
    lens_corner = 1  # the lower right corner tile's inner corner is magnified
    one, two, minus = turned(cd, sd), turned(cd2, sd2), turned(cd, -sd)
    for poly in (one, two, minus):
        depths = {str(depth_in(poly, p)) for p in held}
        if len(depths) != 1:
            sys.exit("oracle: the held corners sink different depths")
    d1, d2 = depth_in(one, held[0]), depth_in(two, held[0])
    put("hair.depth", d1)
    put("hair.depth.4dp", d1.rounded(4))
    put("hair.depth2", d2)
    put("hair.depth2.4dp", d2.rounded(4))
    put("hair.ratio", d2 / d1)
    put("hair.ratio.2dp", (d2 / d1).rounded(2))
    corner = corner_tiles(side)[lens_corner]
    b1, b2, bm = clip(corner, one), clip(corner, two), clip(corner, minus)
    put("hair.vertices", len(b1))
    put("hair.area", area(b1))
    put("hair.area2", area(b2))
    put("hair.minus", f"{depth_in(minus, held[0])}|{area(bm)}")
    put("hair.check", verdict(side, corner_tiles(side) + [one]))
    put("hair.check.minus", verdict(side, corner_tiles(side) + [minus]))
    c3 = Fraction(side3, 2)
    roomy = []
    for s in (sd, -sd):
        ts = corner_tiles(side3) + [turned(cd, s, c3)]
        roomy.append("free" if verdict(q2(side3), ts) == "fits" and touching(ts) == "" else "blocked")
    put("hair.roomy", "|".join(roomy))
    put("hair.tiny", ",".join(str(depth_in(turned(*half_angle(n)), held[0])) for n in (100, 10_000)))
    lens_half, lens_bar = Fraction(1, 40), Fraction(1, 100)
    put("hair.lens.mag", 44)
    put("hair.lens.half", lens_half)
    put("hair.lens.bar", lens_bar)
    p_held = held[lens_corner]
    near = all(v[i] - p_held[i] <= lens_half and p_held[i] - v[i] <= lens_half for v in b2 for i in (0, 1))
    put("hair.lens.fits", "yes" if near else "no")
    quad = clip(corner_tiles(side)[0], turned(*half_angle(3)))
    put("hair.beyond", f"{len(quad)}|{area(quad)}")

    # chapter 04, the corner lemma (Friedman DS7, Lemma 1), by polygons: a
    # tile is a unit square turned about its centre; its shadow is the width
    # of its vertices on the floor; the point's depth is its least distance
    # to an edge (positive inside, zero on an edge)
    def tile_at(cx, cy, cd, sd):
        sq = [(q2(cx) - HALF, q2(cy) - HALF), (q2(cx) + HALF, q2(cy) - HALF),
              (q2(cx) + HALF, q2(cy) + HALF), (q2(cx) - HALF, q2(cy) + HALF)]
        return rotate(sq, (q2(cx), q2(cy)), q2(cd), q2(sd))

    def width(poly):
        xs = [v[0] for v in poly]
        return max(xs) - min(xs)

    def where(poly, p):
        d = depth_in(poly, p)
        return "inside" if d > 0 else ("edge" if d == 0 else "outside")

    def walls(poly):
        return "inside the walls" if all(v[0] >= 0 and v[1] >= 0 for v in poly) else "through a wall"

    corner_pt = (q2(1), q2(1))
    put("corner.point", "1,1")
    straight_t, eighth_t = (1, 0), (r, r)
    put("corner.shadow.min", width(tile_at(2, 2, *straight_t)))
    s_max = width(tile_at(2, 2, *eighth_t))
    put("corner.shadow.max", s_max)
    put("corner.shadow.max.2dp", s_max.rounded(2))
    put("corner.bound.max", 1 * (1 - HALF))
    put("corner.bound.eighth", s_max * (1 - s_max * HALF))
    checked = [straight_t, eighth_t]
    for n in range(2, 21):
        cd_n, sd_n = half_angle(n)
        checked += [(cd_n, sd_n), (sd_n, cd_n)]
    margins = []
    for cd_n, sd_n in checked:
        s_n = width(tile_at(2, 2, cd_n, sd_n))
        lo_n = s_n * HALF
        # the least depth over the four extreme centres of [s/2, 1]^2
        m = min(depth_in(tile_at(x, y, cd_n, sd_n), corner_pt) for x in (lo_n, q2(1)) for y in (lo_n, q2(1)))
        if not (m == HALF * (s_n - 1) * (s_n - 1)):
            sys.exit(f"oracle: corner margin {m} is not half the square of {s_n} - 1")
        margins.append(m)
    put("corner.lemma", "yes" if all(m >= 0 for m in margins) else "no")
    put("corner.turns", len(checked))
    put("corner.margin.eighth", margins[1])
    put("corner.edges", "|".join(where(tile_at(x, y, *straight_t), corner_pt) for x, y in
                                 ((HALF, HALF), (HALF, Fraction(4, 5)), (Fraction(4, 5), HALF), (Fraction(3, 4), Fraction(3, 4)))))
    turned_ok = all(where(tile_at(x, y, cd_n, sd_n), corner_pt) == "inside"
                    for cd_n, sd_n in checked if width(tile_at(2, 2, cd_n, sd_n)) > 1
                    for x in (width(tile_at(2, 2, cd_n, sd_n)) * HALF, q2(1))
                    for y in (width(tile_at(2, 2, cd_n, sd_n)) * HALF, q2(1)))
    put("corner.turned", "inside" if turned_ok else "not always inside")
    past = tile_at(Fraction(8, 5), HALF, *straight_t)
    put("corner.fail.centre", f"{where(past, corner_pt)}|{walls(past)}")
    crossing = tile_at(HALF, HALF, *eighth_t)
    put("corner.fail.wall", f"{where(crossing, corner_pt)}|{walls(crossing)}")
    far = side - 1
    put("corner.box.points", " ".join(f"({x}, {y})" for x, y in ((q2(1), q2(1)), (q2(1), far), (far, q2(1)), (far, far))))
    lo_c, hi_c = q2(HALF), side - HALF
    region = (hi_c - lo_c) * (hi_c - lo_c)
    block = 1 - lo_c
    put("corner.box.region", region)
    put("corner.box.block", block)
    put("corner.box.covered", 4 * block * block)
    put("corner.box.left", region - 4 * block * block)
    put("corner.box.middle", side - 2)

    # chapter 05, the side strip (Friedman DS7, Lemma 2), by polygons: a tile
    # rests on the floor when its polygon is shifted down onto it; a line
    # cuts it where it meets its edges; the slip height is read off the cap
    # over the tile's top vertex (its two edges' run per unit of rise), not
    # from the formula the video derives; the tile's centre is the mean of
    # its vertices
    def rest(a, cd, sd):
        poly = tile_at(a, 2, cd, sd)
        low = min(v[1] for v in poly)
        return [(v[0], v[1] - low) for v in poly]

    def cut_len(poly, yy):
        xs = []
        for i, p in enumerate(poly):
            q = poly[(i + 1) % len(poly)]
            lo, hi = (p, q) if p[1] <= q[1] else (q, p)
            if lo[1] <= yy <= hi[1]:
                if lo[1] == hi[1]:
                    xs += [lo[0], hi[0]]
                else:
                    xs.append(lo[0] + (yy - lo[1]) * (hi[0] - lo[0]) / (hi[1] - lo[1]))
        return max(xs) - min(xs) if xs else Q2(0)

    def size(d):
        return d if d.sign() >= 0 else -d

    def slip(poly, gap_x):
        """The line height above which the resting tile's cut is shorter than gap_x."""
        top = max(v[1] for v in poly)
        tops = [i for i, v in enumerate(poly) if v[1] == top]
        if len(tops) == 2:  # straight: above its top edge the line misses it
            return top
        t = poly[tops[0]]
        sides = [poly[(tops[0] - 1) % 4], poly[(tops[0] + 1) % 4]]
        run = sum((size(v[0] - t[0]) / (t[1] - v[1]) for v in sides), Q2(0))
        h = t[1] - gap_x / run
        if h < max(v[1] for v in sides):
            sys.exit("oracle: the slip height is below the cap")
        return h

    def centre_of(poly):
        return (sum((v[0] for v in poly), Q2(0)) * Fraction(1, 4), sum((v[1] for v in poly), Q2(0)) * Fraction(1, 4))

    def broken(poly, gap_x, y_s):
        cx, cy = centre_of(poly)
        kept = [("0<x<=1", 0 < gap_x <= 1), ("0<y<=1", 0 < y_s <= 1),
                ("x+2y<2sqrt2", gap_x + 2 * y_s < 2 * SQRT2), ("quadrant", walls(poly) == "inside the walls"),
                ("centre", 1 <= cx <= 1 + gap_x and 0 <= cy <= y_s)]
        return [name for name, ok in kept if not ok]

    def strip_words(poly, gap_x, y_s):
        return ",".join(where(poly, p) for p in ((q2(1), q2(y_s)), (1 + q2(gap_x), q2(y_s))))

    gap = side - 2
    one = q2(1)
    strip_pts = [(one, one), (one + gap, one)]
    if strip_pts != [(q2(1), q2(1)), (far, q2(1))]:
        sys.exit("oracle: the strip's points are not two of the four")
    put("strip.points", " ".join(f"({x}, {y})" for x, y in strip_pts))
    put("strip.sum", gap + 2)
    put("strip.assumed", "yes" if not broken(rest(1, 1, 0), gap, one) else "no")
    def admits(cd, sd, y_s):  # some centre at most y_s up keeps the tile above the floor
        return min(v[1] for v in tile_at(0, y_s, cd, sd)) >= 0

    if not all(admits(cd_n, sd_n, one) for cd_n, sd_n in checked):
        sys.exit("oracle: a checked turn admits no centre one tile up")
    cuts = [cut_len(rest(0, cd_n, sd_n), one) for cd_n, sd_n in checked]
    slips = [slip(rest(0, cd_n, sd_n), gap) for cd_n, sd_n in checked]
    if any((c >= gap) != (s_h >= 1) for c, s_h in zip(cuts, slips)):
        sys.exit("oracle: the floor cut and the slip height disagree")
    put("strip.lemma", "yes" if all(c >= gap for c in cuts) else "no")
    put("strip.turns", len(checked))
    h_straight, h_eighth = slips[0], slips[1]
    put("strip.height.straight", h_straight)
    put("strip.height.eighth", h_eighth)
    put("strip.height.eighth.2dp", h_eighth.rounded(2))
    put("strip.least", "straight" if h_straight < h_eighth else ("eighth" if h_eighth < h_straight else "both"))
    put("strip.cut.straight", cuts[0])
    put("strip.cut.eighth", cuts[1])
    put("strip.cut.eighth.2dp", cuts[1].rounded(2))
    put("strip.margin.eighth", cuts[1] - gap)
    quarter = Fraction(1, 4)
    put("strip.vacuous", f"{'a centre' if admits(r, r, quarter) else 'no centre'}|{cut_len(rest(0, r, r), quarter)}")
    put("strip.edges", "|".join(strip_words(rest(a, *straight_t), gap, one) for a in (one, one + gap * HALF, one + gap)))
    put("strip.alternatives", "|".join(strip_words(rest(a, *eighth_t), gap, one)
                                        for a in (Fraction(11, 10), one + gap * HALF, Fraction(9, 10) + gap)))
    turned_in = True
    for cd_n, sd_n in checked:
        w_n = width(tile_at(2, 2, cd_n, sd_n))
        if w_n > 1:
            for x_c in (one, one + gap):
                for y_c in (w_n * HALF, one):
                    words = strip_words(tile_at(x_c, y_c, cd_n, sd_n), gap, one).split(",")
                    turned_in = turned_in and "inside" in words
    put("strip.turned", "inside" if turned_in else "not always inside")
    put("strip.one", strip_words(tile_at(Fraction(8, 5), HALF, *straight_t), gap, one))
    dodges = [("floor", tile_at(one + gap * HALF, HALF, *eighth_t), gap, one),
              ("high", rest(one + gap * HALF, *straight_t), gap, Fraction(21, 20)),
              ("wide", rest(Fraction(3, 2), *eighth_t), one, one),
              ("past", rest(Fraction(8, 5), *straight_t), Fraction(6, 5), Fraction(3, 4))]
    for key, poly, gap_x, y_s in dodges:
        put(f"strip.fail.{key}", f"{strip_words(poly, gap_x, y_s)}|{walls(poly)}|{','.join(broken(poly, gap_x, y_s))}")
    strip_area = (far - 1) * (1 - lo_c)
    covered = 4 * (1 - lo_c) * (1 - lo_c) + 4 * strip_area
    put("strip.box.strip", strip_area)
    put("strip.box.covered", covered)
    put("strip.box.left", region - covered)

    # chapter 06, the middle (Friedman DS7, Lemma 3), by polygons: a corner is
    # past a side of the tile when it is strictly outside that edge's line;
    # a tile holds a corner when the corner's depth is not negative; the
    # lemma's examples are checked by those depths alone, not by the video's
    # case split; a triangle holds a point by the signs of its edge crosses,
    # and one of no area only on its segments
    def tri_holds(tri, p):
        area2 = cross(tri[0], tri[1], tri[2])
        if area2 == 0:
            for k in range(3):
                a, b = tri[k], tri[(k + 1) % 3]
                if cross(a, b, p) == 0 and (p[0] - a[0]) * (p[0] - b[0]) + (p[1] - a[1]) * (p[1] - b[1]) <= 0:
                    return "edge"
            return "outside"
        s = [cross(tri[k], tri[(k + 1) % 3], p) * area2.sign() for k in range(3)]
        if any(v < 0 for v in s):
            return "outside"
        return "inside" if all(v > 0 for v in s) else "edge"

    side_names = ["right", "top", "left", "bottom"]  # the tile's edges 1-2, 2-3, 3-0, 0-1

    def past(poly, p):
        return [cross(poly[(k + 1) % 4], poly[(k + 2) % 4], p) < 0 for k in range(4)]

    def how_missed(poly, tri):
        flags = [past(poly, p) for p in tri]
        if any(not any(f) for f in flags):
            return "holds"
        for i in range(3):
            for j in range(i + 1, 3):
                for k in range(4):
                    if flags[i][k] and flags[j][(k + 2) % 4]:
                        return f"apart:{side_names[k]},{side_names[(k + 2) % 4]}"
        passed = [any(f[k] for f in flags) for k in range(4)]
        for k in range(4):
            if all(not passed[m] or m in (k, (k + 1) % 4) for m in range(4)):
                return f"beside:{side_names[k]},{side_names[(k + 1) % 4]}"
        sys.exit("oracle: three outside corners pass opposite sides without an opposite pair")

    def squares(tri):
        return ",".join(str((tri[(k + 1) % 3][0] - tri[k][0]) * (tri[(k + 1) % 3][0] - tri[k][0])
                            + (tri[(k + 1) % 3][1] - tri[k][1]) * (tri[(k + 1) % 3][1] - tri[k][1])) for k in range(3))

    four = [(q2(1), q2(1)), (q2(1), far), (far, q2(1)), (far, far)]
    split = [[0, 2, 3], [0, 3, 1]]  # the diagonal from (1, 1) to (far, far)
    tris = [[four[i] for i in t] for t in split]
    if any(area(t).sign() <= 0 for t in tris) or area(tris[0]) + area(tris[1]) != gap * gap:
        sys.exit("oracle: the two triangles are not the middle square, counter-clockwise")
    put("middle.triangles", "|".join(",".join(map(str, t)) for t in split))
    put("middle.sides2", "|".join(squares(t) for t in tris))
    put("middle.diagonal2", (far - 1) * (far - 1) + (far - 1) * (far - 1))
    n_checks = 0
    for tri in tris:
        centres = list(tri) + [((tri[k][0] + tri[(k + 1) % 3][0]) * HALF, (tri[k][1] + tri[(k + 1) % 3][1]) * HALF)
                               for k in range(3)]
        centres.append(((tri[0][0] + tri[1][0] + tri[2][0]) * Fraction(1, 3), (tri[0][1] + tri[1][1] + tri[2][1]) * Fraction(1, 3)))
        for cx, cy in centres:
            if tri_holds(tri, (cx, cy)) == "outside":
                sys.exit("oracle: a checked centre is outside its triangle")
            for cd_n, sd_n in checked:
                poly = tile_at(cx, cy, cd_n, sd_n)
                if not any(depth_in(poly, p) >= 0 for p in tri):
                    sys.exit(f"oracle: the middle lemma fails at {cx}, {cy}")
                n_checks += 1
    put("middle.lemma", "yes")
    put("middle.checks", n_checks)
    put("middle.edge.holds", ",".join(where(tiles[4], p) for p in four))
    put("middle.straight.holds", ",".join(where(tile_at(c, c, 1, 0), p) for p in four))
    put("middle.centre.on", "|".join(tri_holds(t, (c, c)) for t in tris))
    one4 = Fraction(7, 4)
    big = [(q2(1), q2(1)), (q2(1), q2(one4)), (q2(one4), q2(1)), (q2(one4), q2(one4))]
    big_tri = [big[i] for i in split[0]]
    big_tile = tile_at(Fraction(11, 8), Fraction(11, 8), r, r)
    put("middle.fail.large", f"{how_missed(big_tile, big_tri)}|{squares(big_tri)}|{','.join(where(big_tile, p) for p in big)}")
    out_tile = tile_at(Fraction(11, 10), Fraction(9, 10) + gap, 1, 0)
    out_centre = centre_of(out_tile)
    put("middle.fail.out", f"{how_missed(out_tile, tris[0])}|{tri_holds(tris[0], out_centre)}|"
                           f"{','.join(where(out_tile, p) for p in tris[0])}|{where(out_tile, four[1])}")
    put("middle.packing.covers", "|".join(",".join(str(i) for i, t in enumerate(tiles) if depth_in(t, p) >= 0)
                                          for p in four))
    inside_pts = [str(k) for k, p in enumerate(four) if any(depth_in(t, p) > 0 for t in tiles)]
    put("middle.packing.inside", ",".join(inside_pts) or "none")
    put("middle.box.left", region - (4 * (1 - lo_c) * (1 - lo_c) + 4 * gap * (1 - lo_c) + gap * gap))

    # chapter 07, four points, five tiles, by polygons: a tile grown to lam
    # wide is its unit square scaled about its own centre; a point's depth in
    # it is its least distance to an edge (edges lam long); growing a whole
    # picture moves each centre to lam times itself; two grown tiles meet as
    # their clipped polygon says
    def grown_poly(cx, cy, cd, sd, lam):
        h = q2(lam) * HALF
        sq = [(q2(cx) - h, q2(cy) - h), (q2(cx) + h, q2(cy) - h), (q2(cx) + h, q2(cy) + h), (q2(cx) - h, q2(cy) + h)]
        return rotate(sq, (q2(cx), q2(cy)), q2(cd), q2(sd))

    def depth_grown(poly, p, lam):
        return min(cross(poly[i], poly[(i + 1) % 4], p) for i in range(4)) / q2(lam)

    def grade(d):
        return "inside" if d > 0 else ("edge" if d == 0 else "outside")

    def about(poly, lam):
        cx, cy = centre_of(poly)
        return [(cx + lam * (v[0] - cx), cy + lam * (v[1] - cy)) for v in poly]

    lam = Fraction(6, 5)
    put("scale.lambda", lam)
    small_box = side / q2(lam)
    put("scale.small", small_box)
    put("scale.small.2dp", small_box.rounded(2))
    core_c, core_t = (Fraction(7, 5), Fraction(13, 10)), (Fraction(4, 5), Fraction(3, 5))
    core = tile_at(*core_c, *core_t)
    put("scale.core.held", ",".join(where(core, p) for p in four))
    big_tile = grown_poly(*core_c, *core_t, lam)
    put("scale.grown.held", ",".join(grade(depth_grown(big_tile, p, lam)) for p in four))
    margin = depth_grown(big_tile, four[0], lam)
    if where(core, four[0]) != "edge" or margin != (lam - 1) * HALF:
        sys.exit("oracle: the grown depth of the core's edge point is not (lam - 1)/2")
    put("scale.margin", margin)
    put("scale.margin.2dp", margin.rounded(2))
    for key, l in (("scale.margin.at1", Fraction(1)), ("scale.margin.below", Fraction(9, 10))):
        d = depth_grown(grown_poly(*core_c, *core_t, l), four[0], l)
        put(key, f"{d}|{grade(d)}")
    put("scale.grown.walls", "through a wall" if escaped(big_tile, side) else "inside the walls")
    small_tile = tile_at(core_c[0] / lam, core_c[1] / lam, *core_t)
    put("scale.small.walls", "through a wall" if escaped(small_tile, small_box) else "inside the walls")
    put("scale.count", f"{len(tiles)}|{len(four)}")
    put("scale.shared", f"{relation(tiles[0], tiles[4])}|{relation(about(tiles[0], lam), about(tiles[4], lam))}")

    # cited: F. Goebel, "Geometrical Packing and Covering Problems", in A.
    # Schrijver (ed.), Packing and Covering in Combinatorics, Math Centrum
    # Tracts 106 (1979) 179-199 (Friedman, DS7, reference [7])
    put("source.gobel", 1979)

    print("\n".join(out))


if __name__ == "__main__":
    main()
