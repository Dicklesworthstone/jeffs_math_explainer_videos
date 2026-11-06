"""Scratch only. An independent mirror of the x index's selection, written from stroke.rs, not from Ruby's mirror.

Mirrors logic only: is the index's run exactly what the scan's slab test admits, in order, including
ties, signed zeros, one-ulp neighbours and non-finite queries? Says nothing about Rust or the solver.
"""
import itertools
import math
import random
import sys


def partition_point(rows, predicate):
    lo, hi = 0, len(rows)
    while lo < hi:
        mid = (lo + hi) // 2
        if predicate(rows[mid]):
            lo = mid + 1
        else:
            hi = mid
    return lo


def scan(rows, p):
    return [i for i, s in enumerate(rows) if not (p[0] < s[0] or p[0] > s[2] or p[1] < s[1] or p[1] > s[3])]


def sorted_ok(rows, slab):
    finite = lambda edges: all(math.isfinite(edge) for edge in edges)
    return (finite(slab) and all(finite(s) for s in rows)
            and all(a[0] <= b[0] and a[2] <= b[2] for a, b in zip(rows, rows[1:])))


def candidates(rows, slab, flag, p):
    if not (flag and math.isfinite(p[0]) and math.isfinite(p[1])):
        return 0, len(rows)
    if p[1] < slab[1] or p[1] > slab[3]:
        return 0, 0
    first = partition_point(rows, lambda s: s[2] < p[0])
    count = partition_point(rows[first:], lambda s: s[0] <= p[0])
    return first, first + count


def index(rows, slab, flag, p):
    first, last = candidates(rows, slab, flag, p)
    return [i for i in range(first, last) if not (p[0] < rows[i][0] or p[0] > rows[i][2] or p[1] < rows[i][1] or p[1] > rows[i][3])]


def aggregate(rows):
    slab = [math.inf, math.inf, -math.inf, -math.inf]
    for s in rows:
        slab = [min(slab[0], s[0]), min(slab[1], s[1]), max(slab[2], s[2]), max(slab[3], s[3])]
    return slab


rng = random.Random(20261106)
EDGES = [-1.0, -0.0, 0.0, 0.5, 1.0]
trials = accepted = queries = admitted = 0
for _ in range(int(sys.argv[1])):
    n = rng.randint(1, 9)
    lefts = sorted(rng.choice(EDGES) for _ in range(n))
    rights = sorted(rng.choice(EDGES) for _ in range(n))
    rows = []
    for a, b in zip(lefts, rights):
        y0, y1 = sorted((rng.choice(EDGES), rng.choice(EDGES)))
        rows.append([a, y0, b, y1])  # left may exceed right: the proof must not need a <= b
    slab = aggregate(rows)
    flag = sorted_ok(rows, slab)
    trials += 1
    accepted += flag
    assert flag, 'every sorted, finite fixture must be accepted'
    xs = sorted({e for s in rows for e in (s[0], s[2])}) + [-0.0, 0.0]
    probes = [v for x in xs for v in (x, math.nextafter(x, -math.inf), math.nextafter(x, math.inf))]
    ys = [v for s in rows for e in (s[1], s[3]) for v in (e, math.nextafter(e, math.inf), math.nextafter(e, -math.inf))]
    for p in itertools.product(probes + [math.nan, math.inf, -math.inf], (ys + [math.nan, math.inf, -math.inf])[:9]):
        assert index(rows, slab, flag, p) == scan(rows, p), (rows, p)
        queries += 1
        admitted += bool(scan(rows, p))

# Refusals: a NaN or infinite slab, or a reversal, must never set the flag.
good = [[float(k), 0.0, k + 1.0, 1.0] for k in range(4)]
for label, rows in {
    'nan edge': good[:1] + [[math.nan, 0.0, 2.0, 1.0]] + good[2:],
    'inf edge': good[:1] + [[1.0, 0.0, math.inf, 1.0]] + good[2:],
    'y nan': good[:1] + [[1.0, math.nan, 2.0, 1.0]] + good[2:],
    'reversed': good[::-1],
}.items():
    assert not sorted_ok(rows, aggregate(rows)), label
print(f'PASS {trials} random sorted strokes, {accepted} accepted, {queries} queries ({admitted} admitted by the scan), 4 refusals')
