#!/usr/bin/env python3
"""Reviewer's own arithmetic for apportionment chapter 12.

Exact fractions from the three populations in the script, and nothing else: no
import from the crate, no read of oracle.py or facts.rs. Run with python3 -I -B.
This checks numbers shown on the frames. It is not a build, and it says nothing
about pictures, motion or sound.
"""
from fractions import Fraction as Fr
from math import ceil, floor

POP = {"Ash": 4000, "Birch": 9000, "Cedar": 26000}
TOTAL = sum(POP.values())
FRAC = {t: Fr(p, TOTAL) for t, p in POP.items()}


def start(t, k):
    return (k - 1) / FRAC[t]


def end(t, k):
    return k / FRAC[t]


def walk(n):
    """Jefferson's list from the top; skip a seat that would lift a town above
    its share rounded up at the new house size."""
    seats = {t: 0 for t in POP}
    out = []
    for h in range(1, n + 1):
        order = sorted(POP, key=lambda t: -Fr(POP[t], seats[t] + 1))
        skipped = []
        for t in order:
            if seats[t] + 1 <= ceil(h * FRAC[t]):
                seats[t] += 1
                out.append((h, t, seats[t], skipped, dict(seats)))
                break
            skipped.append(f"{t} {seats[t] + 1}")
    return out


print("populations", POP, "total", TOTAL)
print("tile lengths", {t: str(1 / f) for t, f in FRAC.items()})

ranked = sorted(
    ((Fr(p, k), t, k) for t, p in POP.items() for k in range(1, 14)),
    key=lambda e: (-e[0], e[1]),
)[:19]
print("Jefferson's list, top 19 (the film shows 18; ranks 18 and 19 tie):")
for r, (sign, t, k) in enumerate(ranked, 1):
    print(f"  {r:2d} {t} {k:2d} signpost {round(sign):6,d} tile {start(t, k)!s:>6} to {end(t, k)!s:>6}")

print("quota method walk:")
history = {0: {t: 0 for t in POP}}
for h, t, k, skipped, seats in walk(16):
    history[h] = seats
    note = f"  skipped first: {', '.join(skipped)}" if skipped else ""
    print(f"  {h:2d} {t} seat {k:2d} -> {seats['Ash']},{seats['Birch']},{seats['Cedar']}{note}")

for h in range(1, 17):
    for t in POP:
        q = h * FRAC[t]
        assert floor(q) <= history[h][t] <= ceil(q), (h, t)
print("within quota at every house 1 to 16: yes")

# entered (strictly past the start) is the same test as "not above the ceiling"
for t in POP:
    for h in range(1, 40):
        for k in range(1, 40):
            assert (start(t, k) < h) == (k <= ceil(h * FRAC[t])), (t, h, k)
print("tile k entered at h <=> k <= ceil(h f), all towns, h and k below 40: yes")

then, now, missing = 6, 13, ("Cedar", 5)
print(f"stretch {then} to {now}: seats {now - then}, need {now - then + 1}")
inside = [
    (t, k)
    for t in POP
    for k in range(1, 40)
    if start(t, k) >= then and end(t, k) <= now
]
print("whole tiles inside:", inside)
for t in POP:
    bound = (now - then) * FRAC[t]
    n = sum(1 for x in inside if x[0] == t)
    print(f"  {t}: {n} <= {now - then} x {POP[t] // 1000}/39 = {float(bound):.3f}")
    assert n <= bound
print("seats at then", history[then], "at now", history[now])
print("missing tile", missing, "from", start(*missing), "to", end(*missing))
print("Birch 2 ends", end("Birch", 2), "later than", end(*missing), ":", end("Birch", 2) > end(*missing))

# by the spoken definition of then, a tile handed out since then ends by the missing tile's end
late = [x for x in inside if end(*x) > end(*missing)]
print("whole tiles inside that end later than the missing tile:", late)

# ruled-out sets drawn as veils, against the frames
print("start before then, except Birch 2:", [(t, k) for t in POP for k in range(1, 14)
      if start(t, k) < then and start(t, k) < 19.5 and (t, k) != ("Birch", 2)])
print("start at or past then, end after now:", [(t, k) for t in POP for k in range(1, 14)
      if start(t, k) >= then and end(t, k) > now and start(t, k) < 19.5])
print("not entered at now:", [(t, k) for t in POP for k in range(1, 14) if now <= start(t, k) < 19.5])

for t in POP:
    q = 15 * FRAC[t]
    print(f"15 seats, {t}: share {float(q):.3f}, floor {floor(q)}, ceiling {ceil(q)}")
remainders = sorted(POP, key=lambda t: -(15 * FRAC[t] - floor(15 * FRAC[t])))
hamilton = {t: floor(15 * FRAC[t]) for t in POP}
for t in remainders[: 15 - sum(hamilton.values())]:
    hamilton[t] += 1
print("Hamilton at 15:", hamilton, " quota method at 15:", history[15])
