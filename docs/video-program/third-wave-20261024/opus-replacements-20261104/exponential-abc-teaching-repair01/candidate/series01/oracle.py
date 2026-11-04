#!/usr/bin/env python3
"""Exact polynomial arithmetic; no recurrence or native-model import.

The bars use the exact running product of ratios R/i, not the native power over factorial.
The equations are rebuilt from the column they come from, and each is checked by solving it.
"""
from fractions import Fraction
from math import ceil, factorial

RADIUS = Fraction(3, 2)
SEGMENTS = 60
LETTERS = {1: "a", 2: "b", 3: "c", 4: "d"}


def show(q):
    return str(q.numerator) if q.denominator == 1 else f"{q.numerator}/{q.denominator}"


def five(q):
    """Five places, refusing an exact tie at the sixth, where float rounding could differ."""
    scaled = q * 200000
    assert scaled.denominator != 1 or scaled.numerator % 2 == 0, "an exact tie at the fifth place"
    return f"{float(q):.5f}"


rows = [
    ("factorials.plot.radius", "1.500000"),
    ("factorials.plot.top", "5.000000"),
    ("factorials.plot.segments", SEGMENTS),
]
for n in range(5):
    denominator = factorial(n)
    rows.append((f"factorials.coefficient.{n}", "1" if denominator == 1 else f"1/{denominator}"))
    rows.append((f"factorials.denominator.{n}", denominator))
    rows.append((f"factorials.running.{n}", "1" if n == 0 else r"\cdot ".join(str(i) for i in range(1, n + 1))))
    for j in range(SEGMENTS + 1):
        x = RADIUS * j / SEGMENTS
        y = sum((x**k / factorial(k) for k in range(n + 1)), Fraction())
        rows.append((f"factorials.curve.{n}.{j}.x", f"{float(x):.6f}"))
        rows.append((f"factorials.curve.{n}.{j}.y", five(y)))
        assert 0 <= x <= RADIUS and 0 <= y <= 5

# Each equation is what its column says: n times the new coefficient equals the coefficient above it.
coefficient = {0: Fraction(1)}
for n in range(1, 5):
    above = coefficient[n - 1]
    solved = above / n
    coefficient[n] = solved
    left = LETTERS[n] if n == 1 else f"{n}{LETTERS[n]}"
    rows.append((f"factorials.derivative.{n}", left))
    rows.append((f"factorials.match.{n}.equation", f"{left} = {show(above)}"))
    rows.append((f"factorials.match.{n}.value", f"{LETTERS[n]} = {show(solved)}"))
    assert n * solved == above and solved == Fraction(1, factorial(n))

# The first index with R/index <= 1/2, found by walking the ratios.
index = 1
while RADIUS / index > Fraction(1, 2):
    index += 1
rows.append(("factorials.halving.from", index))
assert RADIUS / index == Fraction(1, 2), "the printed sign is non-strict because equality is reached"

for i in range(1, 9):
    rows.append((f"factorials.ratio.{i}", show(RADIUS / i)))

# Bars: the running product of the ratios themselves.
bar = {0: Fraction(1)}
for k in range(1, 10):
    bar[k] = bar[k - 1] * RADIUS / k
    assert bar[k] == RADIUS**k / factorial(k)
for k in range(10):
    rows.append((f"factorials.bar.{k}", five(bar[k])))

# The ceiling over the zoomed tail, and every bar under it, past the drawn range.
ceiling = {5: bar[5]}
for k in range(6, 30):
    ceiling[k] = ceiling[k - 1] / 2
    assert RADIUS**k / factorial(k) <= ceiling[k]
for k in range(5, 10):
    rows.append((f"factorials.envelope.{k}", five(ceiling[k])))
rows.append(("factorials.zoom", 16))

for n in (4, 8):
    assert n + 2 >= 2 * RADIUS
    bound = 2 * RADIUS ** (n + 1) / factorial(n + 1)
    partial = sum((RADIUS**k / factorial(k) for k in range(n + 1, 60)), Fraction())
    assert partial < bound
    rounded_up = Fraction(ceil(bound * 100000), 100000)
    assert rounded_up >= bound
    rows.append((f"factorials.tail.{n}.bound", f"{float(rounded_up):.5f}"))

assert len(rows) == len(dict(rows)) == 667
for key, value in rows:
    print(f"{key}\t{value}")
