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

    # cited: F. Goebel, "Geometrical Packing and Covering Problems", in A.
    # Schrijver (ed.), Packing and Covering in Combinatorics, Math Centrum
    # Tracts 106 (1979) 179-199 (Friedman, DS7, reference [7])
    put("source.gobel", 1979)

    print("\n".join(out))


if __name__ == "__main__":
    main()
