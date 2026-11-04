#!/usr/bin/env python3
"""Exact rational arithmetic for the small-multiplier chapter; imports no native model.

Coefficients come from Pascal's rule, not from the native chain of factors. Endpoints are exact powers of exact
fractions. E at an input is an exact partial sum bracketed by its tail bound, and each printed value is accepted only
when both ends of the bracket print the same digits.
"""
from fractions import Fraction as Q
from math import ceil, factorial

R = Q(3, 2)
TABLE = (4, 8, 16, 64, 1024)
SWEEP = 16
FILL = (4, 16, 64, 256, 1024)
FILL_TOP = 100
FILL_COLUMNS = (3, 100)
WARNING = (4, 16, 40, 100)
WARNING_COLUMN = 3
BARS = (4, 16, 64, 1024)
BAR_TOP = 9
EARLY, CUT = 4, 8
AIM = Q(1, 100)
HEADS = (64, 256, 1024)
EDGE = Q(-3)
EDGE_COUNTS = (1, 2, 3, 4, 16, 64, 1024)
SAMPLES = 90
WALKS = (4, 16)


def places(q, d):
    """Rounded at d places from the exact value. A true tie goes to the even digit, as Rust prints an exact float."""
    scaled = abs(q) * 10**d
    whole = scaled.numerator // scaled.denominator
    rest = scaled - whole
    if rest > Q(1, 2) or (rest == Q(1, 2) and whole % 2 == 1):
        whole += 1
    text = f"{whole:0{d + 1}d}"
    body = text[:-d] + "." + text[-d:]
    return ("-" if q < 0 and whole != 0 else "") + body


def bracket(x, degree=48):
    """E(x) between exact bounds: the partial sum and the first omitted term, doubled."""
    assert degree + 2 >= 2 * abs(x)
    s = sum((x**k / factorial(k) for k in range(degree + 1)), Q(0))
    w = 2 * abs(x) ** (degree + 1) / factorial(degree + 1)
    return s - w, s + w


def e_places(x, d):
    low, high = bracket(x)
    a, b = places(low, d), places(high, d)
    assert a == b, f"E({x}) is too close to a rounding edge at {d} places"
    return a


def size(x, k):
    return abs(x) ** k / factorial(k)


def pascal(n):
    row = [1]
    for _ in range(n):
        row = [1] + [row[i] + row[i + 1] for i in range(len(row) - 1)] + [1]
    return row


def chain_small(n, k):
    """C(n,k) k! / n^k from Pascal's row: nothing here multiplies correction factors."""
    return Q(pascal(n)[k] * factorial(k), n**k) if k <= n else Q(0)


def chain(n, k):
    """The falling factorial over n^k, in integers, still with no 1 - j/n factor. Zero once k passes n."""
    if k > n:
        return Q(0)
    top = 1
    for i in range(k):
        top *= n - i
    return Q(top, n**k)


assert all(chain(n, k) == chain_small(n, k) for n in (1, 2, 3, 4, 16) for k in range(20)), "the two counts must agree"


def up(q):
    """Round up at the fifth place, the rule the printed tail bounds use."""
    return places(Q(ceil(q * 10**5), 10**5), 5)


def tail(cut):
    return up(2 * size(R, cut + 1))


def head(n, cut=CUT):
    return sum((size(R, k) * (1 - chain(n, k)) for k in range(cut + 1)), Q(0))


def distance(x, n):
    """|E(x) - (1+x/n)^n|, bracketed, printed only when both ends agree."""
    power = (1 + x / n) ** n
    low, high = bracket(x)
    a, b = abs(low - power), abs(high - power)
    p, q = places(min(a, b), 5), places(max(a, b), 5)
    assert p == q, f"distance at {x}, {n} is too close to a rounding edge"
    return p


rows = [("tiny.step.multiplier", "5/4")]
for n in TABLE:
    rows.append((f"tiny.one.{n}", places((1 + Q(1, n)) ** n, 3)))
rows.append(("tiny.sweep.cells", SWEEP))
for x in (R, -R):
    rows.append((f"tiny.sweep.{float(x)}.endpoint", places((1 + x / SWEEP) ** SWEEP, 3)))
    rows.append((f"tiny.sweep.{float(x)}.series", e_places(x, 3)))
for k in range(5):
    rows.append((f"tiny.ways.{k}", pascal(4)[k]))
    rows.append((f"tiny.strip.{k}", f"{pascal(4)[k]}/{4**k}"))
    rows.append((f"tiny.chain.4.{k}", places(chain(4, k), 5)))
for n in FILL:
    rows.append((f"tiny.fill.{n}", " ".join(places(chain(n, k), 5) for k in range(FILL_TOP + 1))))
    for k in FILL_COLUMNS:
        rows.append((f"tiny.column.{n}.{k}", places(chain(n, k), 3)))
rows.append(("tiny.warning.column", WARNING_COLUMN))
for n in WARNING:
    rows.append((f"tiny.warning.{n}", f"1/{n}"))
rows.append(("tiny.warning.total", 1))
for n in BARS:
    rows.append((f"tiny.bars.{n}", " ".join(places(size(R, k) * chain(n, k), 5) for k in range(BAR_TOP + 1))))
rows.append(("tiny.aim", "0.01"))
for cut in (EARLY, CUT):
    rows.append((f"tiny.cut.{cut}.tail", tail(cut)))
for n in HEADS:
    printed_head = up(head(n))
    rows.append((f"tiny.head.{n}", printed_head))
    rows.append((f"tiny.total.{n}", places(Q(printed_head) + Q(tail(CUT)), 5)))
    rows.append((f"tiny.distance.{n}", distance(R, n)))
    rows.append((f"tiny.negative.{n}", distance(-R, n)))
for n in EDGE_COUNTS:
    base = 1 + EDGE / n
    value = base**n
    base_text = str(base.numerator) if base.denominator == 1 else f"{base.numerator}/{base.denominator}"
    # A reduced fraction is what the table prints; n = 4 reduces to itself, the others are odd over a power of two.
    rows.append((f"tiny.edge.{n}.base", base_text))
    if n <= 3:
        value_text = str(value.numerator) if value.denominator == 1 else f"{value.numerator}/{value.denominator}"
    else:
        value_text = places(value, 4)
    rows.append((f"tiny.edge.{n}.value", value_text))
rows.append(("tiny.edge.series", e_places(EDGE, 4)))
for j in range(SAMPLES + 1):
    x = EDGE + Q(j, 20)
    rows.append((f"tiny.curve.{j}", f"{places(x, 2)} {e_places(x, 5)}"))
for n in WALKS:
    for side, x in (("right", R), ("left", EDGE)):
        step = 1 + x / n
        rows.append((f"tiny.walk.{n}.{side}", " ".join(places(step**j, 5) for j in range(n + 1))))

assert len(rows) == 1 + 5 + 1 + 4 + 15 + 5 + 10 + 1 + 4 + 1 + 4 + 1 + 2 + 12 + 14 + 1 + 91 + 4, len(rows)
for key, value in rows:
    print(f"{key}\t{value}")
