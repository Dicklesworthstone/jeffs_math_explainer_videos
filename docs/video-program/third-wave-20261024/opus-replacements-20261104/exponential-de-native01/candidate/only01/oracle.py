#!/usr/bin/env python3
"""Exact rational arithmetic for the uniqueness chapter; imports no native model.

E at an input is an exact partial sum bracketed by its tail bound. A printed value is accepted only when both ends
of its bracket print the same digits, so a value that could round either way stops this oracle.
"""
from fractions import Fraction as Q
from math import factorial

PANEL = Q(3, 2)
SAMPLES = 30
DEGREES = (2, 4)
FAR = (Q(-2), Q(-1, 2))
NEAR = (Q(-1, 4), Q(1))
MULTIPLE = 3
PIECE_SAMPLES = 15
RATES = ((Q(1, 2), "1/2"), (Q(0), "0"), (Q(-1, 2), "-1/2"))
RATE_START = 2
RATE_END = 2
RATE_SAMPLES = 20
AXIS_TOPS = (5, 3, 3, 6)


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
    assert degree + 2 >= 2 * abs(x)
    s = sum((x**k / factorial(k) for k in range(degree + 1)), Q(0))
    w = 2 * abs(x) ** (degree + 1) / factorial(degree + 1)
    return s - w, s + w


def scaled_places(low, high, d):
    a, b = places(low, d), places(high, d)
    assert a == b, f"too close to a rounding edge: {a} {b}"
    return a


def e5(x, multiplier=Q(1), d=5):
    low, high = bracket(x)
    return scaled_places(min(multiplier * low, multiplier * high), max(multiplier * low, multiplier * high), d)


def partial(x, degree):
    return sum((x**k / factorial(k) for k in range(degree + 1)), Q(0))


def product5(x, degree):
    """The polynomial times E(-x), both ends of E(-x)'s bracket."""
    p = partial(x, degree)
    low, high = bracket(-x)
    return scaled_places(min(p * low, p * high), max(p * low, p * high), 5)


def imagined5(x):
    factor = 1 - x * x / 6
    low, high = bracket(x)
    return scaled_places(min(factor * low, factor * high), max(factor * low, factor * high), 5)


inputs = [-PANEL + 2 * PANEL * Q(j, SAMPLES) for j in range(SAMPLES + 1)]
rows = [
    ("only.e", " ".join(e5(x) for x in inputs)),
    ("only.imagined", " ".join(imagined5(x) for x in inputs)),
]
rows.append(("only.axes.tops", " ".join(str(top) for top in AXIS_TOPS)))


def defect_denominator(d):
    """From the polynomial's coefficients: y' - y keeps only minus its top term, and this is that term's denominator."""
    y = [Q(1, factorial(k)) for k in range(d + 1)]
    slope = [(k + 1) * y[k + 1] for k in range(d)] + [Q(0)]
    defect = [slope[k] - y[k] for k in range(d + 1)]
    assert all(c == 0 for c in defect[:-1]), defect
    assert defect[-1].numerator == -1, defect
    return defect[-1].denominator


for d in DEGREES:
    rows.append((f"only.polynomial.{d}", " ".join(places(partial(x, d), 5) for x in inputs)))
    rows.append((f"only.product.{d}", " ".join(product5(x, d) for x in inputs)))
    rows.append((f"only.defect.{d}", defect_denominator(d)))
rows.append(("only.far.multiple", MULTIPLE))
far = [FAR[0] + (FAR[1] - FAR[0]) * Q(j, PIECE_SAMPLES) for j in range(PIECE_SAMPLES + 1)]
near = [NEAR[0] + (NEAR[1] - NEAR[0]) * Q(j, PIECE_SAMPLES) for j in range(PIECE_SAMPLES + 1)]
rows.append(("only.far", " ".join(e5(x, Q(MULTIPLE)) for x in far)))
rows.append(("only.near", " ".join(e5(x) for x in near)))
rows.append(("only.rate.start", RATE_START))
rows.append(("only.rate.end", RATE_END))
for rate, name in RATES:
    times = [Q(RATE_END * j, RATE_SAMPLES) for j in range(RATE_SAMPLES + 1)]
    rows.append((f"only.rate.{name}", " ".join(e5(rate * t, Q(RATE_START)) for t in times)))
    rows.append((f"only.rate.{name}.end", e5(rate * RATE_END, Q(RATE_START), 3)))

assert len(rows) == 2 + 1 + 6 + 3 + 2 + 6, len(rows)
for key, value in rows:
    print(f"{key}\t{value}")
