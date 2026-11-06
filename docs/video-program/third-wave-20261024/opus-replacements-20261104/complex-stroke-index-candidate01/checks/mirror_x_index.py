"""Logic mirror of the x index in stroke-index.patch. Not Rust, not a speed measurement.

The patch leaves the per-segment body of `nearest` alone, so the output is unchanged if the
index visits exactly the segments the scan admits, in the same order. This file checks that on
the film's real strip geometry (blend_points, f32 records) and on random strokes, then plants
seven mistakes that the same comparison has to catch. A plant that survives fails the run.

The last section counts slab comparisons per plate frame from the engine's adaptive rule. That
is a model of operation counts. It is not a time and must not be quoted as one.

Standard library only. Run from anywhere: python3 -I -B mirror_x_index.py
"""
import math
import random
import struct
import sys

MIN_SEGMENTS = 32
ROW_GUARD = 1e-4
CELL = 0.035
TILE = 16
RES = {'64x36': (4.5, (32.0, 18.0)), '3840x2160': (270.0, (1920.0, 1080.0))}
# center x, center y, width, height: chapter 04's plane and a small plate from continuing.rs.
PLANES = {'reading_color': (-2.6, -0.25, 6.4, 6.4), 'continuing_small': (5.2, 1.65, 1.9, 1.9)}
failures = []


def check(name, ok, detail=''):
    print(('ok   ' if ok else 'FAIL ') + name + (f': {detail}' if detail else ''))
    if not ok:
        failures.append(name)


def f32(value):
    return struct.unpack('f', struct.pack('f', value))[0]


def rmin(a, b):
    """Rust f64::min, which ignores a NaN."""
    return b if a != a else a if b != b else min(a, b)


def rmax(a, b):
    return b if a != a else a if b != b else max(a, b)


def blend_points(left, y, cell, thick, n):
    """Port of plate_layout.rs::blend_points, x only: every record of a strip has the one y."""
    q = thick / 4.0
    right = left + cell * n
    centre = lambda k: left + cell * (k + 0.5)
    p = [left - 2.0 * q, left - q, left, left, left, (left + centre(0)) / 2.0, centre(0)]
    for k in range(1, n):
        p += [(centre(k - 1) + centre(k)) / 2.0, centre(k)]
    p += [(centre(n - 1) + right) / 2.0, right, right, right, right + q, right + 2.0 * q]
    return p


def plate(plane, row_units=CELL / 4.0):
    cx, cy, width, height = PLANES[plane]
    cols = max(round(width / row_units), 8)
    rows = max(round(height / row_units), 8)
    cell_w, cell_h = width / cols, height / rows
    left, top = cx - width / 2.0, cy + height / 2.0
    return cols, rows, left, top, cell_w, cell_h


def strip_slabs(plane, res, row):
    """Prepared slabs of one strip: records through f32, then segment_slab's hull and pad."""
    cols, rows, left, top, cell_w, cell_h = plate(plane)
    scale, origin = RES[res]
    y = f32(top - cell_h * (row + 0.5))
    xs = [f32(x) for x in blend_points(left, top, cell_w, cell_h, cols)]
    width_units = f32((cell_h + ROW_GUARD) / 0.01)
    pad = 0.5 * (width_units * 0.01 * abs(scale))  # aa_px is 0 on the film's plates
    py = origin[1] - y * scale
    slabs = []
    for i in range(0, len(xs) - 2, 2):
        px = [origin[0] + x * scale for x in xs[i:i + 3]]
        slabs.append([min(px) - pad, py - pad, max(px) + pad, py + pad])
    return slabs, cols


def aggregate(slabs):
    out = [math.inf, math.inf, -math.inf, -math.inf]
    for s in slabs:
        out = [rmin(out[0], s[0]), rmin(out[1], s[1]), rmax(out[2], s[2]), rmax(out[3], s[3])]
    return out


def x_sorted(slabs, slab, plant=None):
    """segments_are_x_sorted. The two plants remove one clause each."""
    finite = lambda edges: all(math.isfinite(e) for e in edges)
    if plant == 'accept_nan':
        finite = lambda edges: True
    if len(slabs) < MIN_SEGMENTS or not finite(slab) or not all(finite(s) for s in slabs):
        return False
    if plant == 'accept_unsorted':
        return True
    return all(a[0] <= b[0] and a[2] <= b[2] for a, b in zip(slabs, slabs[1:]))


def partition_point(items, start, predicate):
    lo, hi = start, len(items)
    while lo < hi:
        mid = (lo + hi) // 2
        if predicate(items[mid]):
            lo = mid + 1
        else:
            hi = mid
    return lo


def x_candidates(slabs, slab, flag, p, plant=None):
    """PreparedStroke::x_candidates, with one way to get each line wrong."""
    finite = math.isfinite(p[0]) and math.isfinite(p[1])
    if plant == 'no_nan_guard':
        finite = True
    if not (flag and finite):
        return range(0, len(slabs))
    if plant == 'y_edge_inclusive':
        if p[1] <= slab[1] or p[1] >= slab[3]:
            return range(0, 0)
    elif p[1] < slab[1] or p[1] > slab[3]:
        return range(0, 0)
    if plant == 'right_edge_inclusive':
        first = partition_point(slabs, 0, lambda s: s[2] <= p[0])
    else:
        first = partition_point(slabs, 0, lambda s: s[2] < p[0])
    if plant == 'left_edge_strict':
        last = partition_point(slabs, first, lambda s: s[0] < p[0])
    else:
        last = partition_point(slabs, first, lambda s: s[0] <= p[0])
    return range(first, last)


def admits(s, p):
    return not (p[0] < s[0] or p[0] > s[2] or p[1] < s[1] or p[1] > s[3])


def scan_visits(slabs, p):
    return [i for i, s in enumerate(slabs) if admits(s, p)]


def index_visits(slabs, slab, flag, p, plant=None):
    found = x_candidates(slabs, slab, flag, p, plant)
    order = reversed(found) if plant == 'reverse_order' else found
    return [i for i in order if admits(slabs[i], p)]


def edge_points(slabs, slab):
    mid_x, mid_y = 0.5 * (slab[0] + slab[2]), 0.5 * (slab[1] + slab[3])
    around = lambda e: (math.nextafter(e, -math.inf), e, math.nextafter(e, math.inf))
    points = [(x, mid_y) for s in slabs for e in (s[0], s[2]) for x in around(e)]
    points += [(mid_x, y) for e in (slab[1], slab[3]) for y in around(e)]
    points += [(math.nan, mid_y), (mid_x, math.nan), (math.nan, math.nan), (math.inf, mid_y), (-math.inf, mid_y),
               (mid_x, math.inf), (mid_x, -math.inf)]
    return points


def compare(cases, plant=None):
    """Mismatches between the index and the scan. `cases` is [(slabs, points)]."""
    bad = 0
    for slabs, points in cases:
        slab = aggregate(slabs)
        flag = x_sorted(slabs, slab, plant)
        for p in points:
            if index_visits(slabs, slab, flag, p, plant) != scan_visits(slabs, p):
                bad += 1
    return bad


def real_cases(rng):
    cases = []
    narrowed = []
    for plane in PLANES:
        rows = plate(plane)[1]
        for res in RES:
            sorted_rows = sum(x_sorted(*(lambda s: (s, aggregate(s)))(strip_slabs(plane, res, row)[0])) for row in range(rows))
            check(f'{plane} {res}: every one of {rows} strips is accepted', sorted_rows == rows, f'{sorted_rows}/{rows}')
            for row in (0, rows // 2, rows - 1):
                slabs, cols = strip_slabs(plane, res, row)
                slab = aggregate(slabs)
                points = edge_points(slabs, slab)
                for _ in range(1500):
                    points.append((rng.uniform(slab[0] - 2, slab[2] + 2), rng.uniform(slab[1] - 1, slab[3] + 1)))
                if res == '64x36':
                    # Every pixel centre, 2x and 4x subcell near the strip's own row.
                    for py in range(int(slab[1]) - 1, int(slab[3]) + 2):
                        for px in range(int(slab[0]) - 1, int(slab[2]) + 2):
                            for samples in (1, 2, 4):
                                for sy in range(samples):
                                    for sx in range(samples):
                                        points.append((px + (sx + 0.5) / samples, py + (sy + 0.5) / samples))
                cases.append((slabs, points))
                mid = (0.5 * (slab[0] + slab[2]), 0.5 * (slab[1] + slab[3]))
                narrowed.append(len(x_candidates(slabs, slab, True, mid)))
    return cases, narrowed


def random_cases(rng):
    """Strokes the real film does not draw: overlaps, repeats, thin and wide reach, some unsorted."""
    cases = []
    for k in range(240):
        n = rng.randint(MIN_SEGMENTS - 2, 90)
        x = rng.uniform(-20, 20)
        reach = rng.choice((0.0, 0.02, 1.5, 12.0))
        slabs = []
        right = -math.inf
        for i in range(n):
            step = rng.choice((0.0, 0.0, 1e-9, 0.25, 1.0, 3.0))
            width = rng.choice((0.0, 0.5, 2.0))
            y = rng.uniform(-3, 3)
            right = x + width + reach if k % 4 == 3 else max(right, x + width + reach)
            slabs.append([x - reach, y - reach - rng.random(), right, y + reach + rng.random()])
            x += step
        if k % 3 == 0:
            a, b = rng.randrange(n), rng.randrange(n)
            slabs[a], slabs[b] = slabs[b], slabs[a]
        slab = aggregate(slabs)
        points = edge_points(slabs, slab)
        for _ in range(120):
            points.append((rng.uniform(slab[0] - 1, slab[2] + 1), rng.uniform(slab[1] - 1, slab[3] + 1)))
        cases.append((slabs, points))
    return cases


def tied_case():
    """Each cell twice: both copies admit the same points, and order is the only difference."""
    slabs = []
    for k in range(20):
        slabs += [[k - 1.0, -2.0, k + 2.0, 2.0], [k - 1.0, -2.0, k + 2.0, 2.0]]
    return slabs, [(k + 0.5, 0.25) for k in range(20)]


def nan_slab_case():
    """A NaN y edge passes the scan's y test, and `min`/`max` drop it from the aggregate.

    A NaN x edge already fails the sort comparison, so only a y edge reaches the finite clause.
    With that clause gone, the aggregate's y early return would skip a segment the scan admits.
    """
    slabs = [[float(k), -1.0, k + 1.0, 1.0] for k in range(40)]
    slabs[20] = [20.0, math.nan, 21.0, math.nan]
    return slabs, [(20.5, 5.0), (20.5, -5.0), (1.5, 0.0), (38.5, 0.0)]


def model(plane, res):
    """Slab comparisons per plate frame under the adaptive rule, with the engine's own counts."""
    cols, rows, left, top, cell_w, cell_h = plate(plane)
    scale, origin = RES[res]
    half = 0.5 * (f32((cell_h + ROW_GUARD) / 0.01) * 0.01 * scale)
    centres = sorted(origin[1] - (top - cell_h * (i + 0.5)) * scale for i in range(rows))
    segments = cols + 5
    plate_px = cols * cell_w * scale
    x_tiles = math.ceil(plate_px / TILE)
    log_cost = 2 * math.ceil(math.log2(segments))
    y_lo, y_hi = int(centres[0] - half) - 1, int(centres[-1] + half) + 2

    def near(lo, hi):
        a = partition_point(centres, 0, lambda c: c + half <= lo)
        b = partition_point(centres, 0, lambda c: c - half < hi)
        return centres[a:b]

    def inked(c, y):
        return abs(y - c) - half < 0

    old_row = old_ssaa = old_span = new_row = new_ssaa = 0.0
    cells = {2: 0.0, 4: 0.0}
    for py in range(y_lo, y_hi):
        row_strips = near(py, py + 1)
        band = py - py % TILE
        tile_strips = len(near(band, band + TILE))
        crossings = shades = hits = 0
        for c in row_strips:
            centre = inked(c, py + 0.5)
            shades += 1
            hits += centre
            edge = False
            for y in (py + 0.25, py + 0.25, py + 0.75, py + 0.75):
                shades += 1
                hit = inked(c, y)
                hits += hit
                if hit != centre:
                    edge = True
                    break
            crossings += edge
        samples = 4 if crossings >= 4 else 2 if crossings >= 2 else 0
        old_span += len(row_strips) * x_tiles * segments
        old_row += plate_px * shades * segments
        new_row += plate_px * (hits * log_cost + (shades - hits) * 2)
        if samples:
            cells[samples] += plate_px
            old_ssaa += plate_px * samples ** 2 * tile_strips * segments
            ys = [py + (j + 0.5) / samples for j in range(samples)]
            sample_hits = samples * sum(sum(inked(c, y) for c in near(py, py + 1)) for y in ys)
            total = samples ** 2 * tile_strips
            new_ssaa += plate_px * (sample_hits * log_cost + (total - sample_hits) * 2)
    old = old_row + old_ssaa + old_span
    new = new_row + new_ssaa + old_span
    print(f'{res} {plane}: {cols}x{rows} strips, tile {TILE}, {tile_strips_at(centres, half, TILE)} strips at most per tile')
    print(f'  cells 2x {cells[2]:.3g}  4x {cells[4]:.3g}  of {plate_px ** 2:.3g} plate cells')
    print(f'  today: row pass {old_row:.3g}  ssaa {old_ssaa:.3g}  row_span {old_span:.3g}  total {old:.3g}')
    print(f'  index: row pass {new_row:.3g}  ssaa {new_ssaa:.3g}  row_span {old_span:.3g} (unchanged)  total {new:.3g}')
    print(f'  ratio of comparisons {old / new:.3g}, and row_span is then {old_span / new:.0%} of what is left')
    return old_ssaa / old, old / new


def tile_strips_at(centres, half, tile):
    best = 0
    band = int(centres[0] - half) - int(centres[0] - half) % tile
    while band < centres[-1] + half:
        a = partition_point(centres, 0, lambda c: c + half <= band)
        b = partition_point(centres, 0, lambda c: c - half < band + tile)
        best = max(best, b - a)
        band += tile
    return best


def main():
    rng = random.Random(20261105)
    print('--- segments visited, index against scan')
    real, narrowed = real_cases(rng)
    check('real strips', compare(real) == 0, f'{sum(len(p) for _, p in real)} points on {len(real)} strips')
    print('candidates at a strip middle, 2 planes x 2 resolutions x 3 rows:', sorted(set(narrowed)))
    check('the index narrows every real strip', max(narrowed) <= 8, f'at most {max(narrowed)} of about 200 to 736')
    scattered = random_cases(rng)
    flagged = sum(x_sorted(s, aggregate(s)) for s, _ in scattered)
    check('random strokes', compare(scattered) == 0, f'{flagged} accepted and {len(scattered) - flagged} refused of {len(scattered)}')
    check('both verdicts occur in the random set', 0 < flagged < len(scattered))
    tied = [tied_case()]
    check('tied copies', compare(tied) == 0)
    nan = [nan_slab_case()]
    check('NaN slab is refused', compare(nan) == 0 and not x_sorted(nan[0][0], aggregate(nan[0][0])))
    print('--- plants: the same comparison must catch each')
    corpus = real[:3] + scattered[:60]
    for plant, cases in (('reverse_order', tied), ('left_edge_strict', corpus), ('right_edge_inclusive', corpus),
                         ('y_edge_inclusive', corpus), ('no_nan_guard', corpus), ('accept_unsorted', scattered),
                         ('accept_nan', nan)):
        bad = compare(cases, plant)
        check(f'plant {plant} is caught', bad > 0, f'{bad} mismatches')
    print('--- model: slab comparisons per plate frame, not a time')
    low, _ = model('reading_color', '64x36')
    high, _ = model('reading_color', '3840x2160')
    print(f'supersampled share of today\'s comparisons: 64x36 {low:.0%}, 4K {high:.0%}')
    print('RESULT', 'FAIL ' + ', '.join(failures) if failures else 'PASS')
    return 1 if failures else 0


if __name__ == '__main__':
    sys.exit(main())
