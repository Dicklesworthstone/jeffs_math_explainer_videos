"""Cross-review of engine-geometry-budget01's geometry table, from the sources, without Ruby's estimator.

Standard library only. Nothing here runs the engine: it mirrors, by hand, what was read.

1. Shape sharing. `table.rs` shape_digest hashes each point minus the first point, so two rows
   with the same x values and one constant y share a shape. The mirror keeps the equality classes
   of the real digest and not its bytes: same relative f64 values, same class.
2. The corrected demand table, with its memory and ceiling consequences.

    python3 -I -B geometry.py
"""
import hashlib
import math
import struct


def f32(x):
    return struct.unpack("<f", struct.pack("<f", x))[0]


def digest(points):
    """Equality classes of table.rs shape_digest: point count, then relative f64 bits."""
    origin = points[0]
    h = hashlib.sha256(struct.pack("<Q", len(points)))
    for p in points:
        for c in range(3):
            r = p[c] - origin[c]
            h.update(struct.pack("<d", 0.0 if r == 0 else r))
    return h.hexdigest()


def curves(points):
    """The shared-anchor quads of a point run: 0,1,2 then 2,3,4 and so on."""
    return [points[i : i + 3] for i in range(0, len(points) - 2, 2)]


def positive(points):
    return sum(1 for c in curves(points) if not (c[0] == c[1] == c[2]))


# ---------------------------------------------------------------- the patch's test fixture
def fixture_row(left, y, cell, n):
    """row_points in engine-plan-limits.patch, every operation rounded to f32 as Rust does."""
    at = lambda x: (x, y, 0.0)
    q = f32(cell / 4.0)
    right = f32(left + f32(cell * f32(n)))
    centre = lambda k: f32(left + f32(cell * f32(f32(k) + 0.5)))
    mid = lambda a, b: f32(f32(a + b) / 2.0)
    p = [at(f32(left - f32(2.0 * q))), at(f32(left - q)), at(left), at(left), at(left)]
    p += [at(mid(left, centre(0))), at(centre(0))]
    for k in range(1, n):
        p += [at(mid(centre(k - 1), centre(k))), at(centre(k))]
    p += [at(mid(centre(n - 1), right)), at(right), at(right), at(right)]
    p += [at(f32(right + q)), at(f32(right + f32(2.0 * q)))]
    return p


def fixture_plate(rows, n, cell):
    cell = f32(cell)
    top = f32(f32(cell * f32(rows)) / 2.0)
    left = f32(f32(-cell * f32(n)) / 2.0)
    return [fixture_row(left, f32(top - f32(cell * f32(f32(row) + 0.5))), cell, n) for row in range(rows)]


def fixture_row_rgba(row, n, frame):
    """row_rgba in the patch: the test's 1_075_301 needs 731 different styles."""
    paint = lambda k, a: (f32(f32(row + 1.0) / 4096.0), f32(f32(k + 1.0) / 4096.0), f32(f32(frame + 1.0) / 64.0), a)
    c = [paint(0, 0.0)] * 3 + [paint(0, 1.0)] * 4
    for k in range(1, n):
        c += [paint(2 * k - 1, 1.0), paint(2 * k, 1.0)]
    c += [paint(2 * n, 1.0)] * 2 + [paint(2 * n, 0.0)] * 4
    return c


# ---------------------------------------------------------------- the film's plates
def blend_points(left, y, cell, thick, n):
    """plate_layout.rs blend_points in f64, then the record buffer's f32."""
    at = lambda x: (x, y, 0.0)
    q = thick / 4.0
    right = left + cell * n
    centre = lambda k: left + cell * (k + 0.5)
    p = [at(left - 2.0 * q), at(left - q), at(left), at(left), at(left)]
    p += [at((left + centre(0)) / 2.0), at(centre(0))]
    for k in range(1, n):
        p += [at((centre(k - 1) + centre(k)) / 2.0), at(centre(k))]
    p += [at((centre(n - 1) + right) / 2.0), at(right), at(right), at(right)]
    p += [at(right + q), at(right + 2.0 * q)]
    return [tuple(f32(v) for v in point) for point in p]


def film_plate(cx, cy, width, x, y, cell):
    """Plane::new, picture_cells and blended_rows, as the source has them."""
    height = width * (y[1] - y[0]) / (x[1] - x[0])
    cols = max(round(width / cell), 8)
    rows = max(round(height / cell), 8)
    cell_w, cell_h = width / cols, height / rows
    left, top = cx - width / 2.0, cy + height / 2.0
    return rows, cols, [blend_points(left, top - cell_h * (i + 0.5), cell_w, cell_h, cols) for i in range(rows)]


def panel(cx, cy, x, yc):
    h = (x[1] - x[0]) * 2.6 / 4.2
    return (cx, cy, 4.2, x, (yc - h / 2.0, yc + h / 2.0))


CELL4 = 0.035 / 4.0
SQUARE = (-2.0, 2.0)
PLATES = {
    "z2 cells4 (reading plane)": (-2.6, -0.25, 6.4, SQUARE, SQUARE),
    "sin(1/z) cells4 (square plane)": (-3.4, -0.2, 5.6, SQUARE, SQUARE),
}
PANELS = [
    panel(-4.5, 1.55, (-2.7, 2.7), 0.3),
    panel(0.0, 1.55, (-2.7, 2.7), 0.0),
    panel(4.5, 1.55, (-3.5, 3.5), 1.2),
    panel(-4.5, -1.35, (-6.6, 6.6), 0.0),
    panel(0.0, -1.35, (-4.5, 3.5), 0.0),
    panel(4.5, -1.35, (-7.0, 3.0), 0.0),
]


def main():
    print("1. SHAPE SHARING")
    print("(the mirror keeps the real digest's equality classes, not its bytes)")
    rows = fixture_plate(731, 731, 0.00875)
    classes = {digest(r) for r in rows}
    seg = positive(rows[0])
    print(f"patch fixture plate(731, 731, 0.00875): {len(rows)} rows, {len(rows[0])} records a row")
    print(f"  distinct outlines {len(classes)}; positive curves a row {seg}; input curves a row {len(curves(rows[0]))}")
    print(f"  so plan.segments().len() should be {len(classes) * seg}, and the patch asserts 731 * 734 = {731 * 734}")
    styles = {tuple(fixture_row_rgba(r, 731, 0)) for r in range(731)}
    print(f"  distinct row paints {len(styles)} of 731, so the knot total holds: 731 x 1471 = {731 * 1471}")
    nudged = [list(p) for p in rows[1]]
    nudged[5][0] = nudged[5][0] + 2.0**-20
    print(f"  control, one x moved 2^-20 in row 1: distinct outlines {len({digest(rows[0]), digest([tuple(p) for p in nudged])})}")

    for name, (cx, cy, w, x, y) in PLATES.items():
        r, c, pts = film_plate(cx, cy, w, x, y, CELL4)
        d = {digest(p) for p in pts}
        print(f"film {name}: {r} x {c}, distinct outlines {len(d)}, segments {len(d) * positive(pts[0])}")

    all_d = {}
    print("film 05_9, six panels at cells4:")
    for i, (cx, cy, w, x, y) in enumerate(PANELS):
        r, c, pts = film_plate(cx, cy, w, x, y, CELL4)
        d = {digest(p) for p in pts}
        print(f"  panel {i}: {r} x {c}, distinct outlines in the panel {len(d)}, positive curves a row {positive(pts[0])}")
        for one in d:
            all_d.setdefault(one, positive(pts[0]))
    six_segments = sum(all_d.values())
    print(f"  distinct outlines over the six {len(all_d)}, segments {six_segments}")

    print()
    print("2. CORRECTED TABLE (Ruby's knots and input curves stand)")
    limit = 1 << 20
    print(f"{'plate':<20}{'knots':>10}{'2x knots':>10}{'curves':>9}{'segments, handoff':>19}{'segments':>10}{'room, handoff':>15}{'room':>10}")
    table = [
        ("z2 cells4", 731, 731, 1, 1),
        ("sin(1/z) cells4", 640, 640, 1, 1),
        ("six panels cells4", 1782, 480, len(all_d), 1),
    ]
    for name, r, n, outlines, _ in table:
        knots = r * (2 * n + 9)
        input_curves = r * (n + 5)
        old = r * (n + 3)
        new = outlines * (n + 3)
        print(f"{name:<20}{knots:>10}{2 * knots:>10}{input_curves:>9}{old:>19}{new:>10}{limit - old:>15}{limit - new:>10}")

    print()
    print("Moving a plate. A translation is a placement and reuses the shape (plan.rs test at 2296-2312).")
    print("A rewrite to a new outline adds that outline once: z2 cells4 goes from 734 to 1468 segments, not 1,073,108.")
    print(f"Only if every row had its own outline would a plate reach the handoff's figure: 731 x 734 = {731 * 734}.")

    print()
    print("Point motion on z2 cells4, baked into f32 records as a point write would be (a mirror, on one frame)")
    r, c, pts = film_plate(-2.6, -0.25, 6.4, SQUARE, SQUARE, CELL4)
    bake = lambda f: {digest([tuple(f32(v) for v in f(p)) for p in row]) for row in pts}
    cos, sin = math.cos(0.1), math.sin(0.1)
    cases = [
        ("translation by (0.37, -0.11)", lambda p: (p[0] + 0.37, p[1] - 0.11, 0.0)),
        ("uniform scale by 0.93 about the origin", lambda p: (p[0] * 0.93, p[1] * 0.93, 0.0)),
        ("rotation by 0.1 rad", lambda p: (p[0] * cos - p[1] * sin, p[0] * sin + p[1] * cos, 0.0)),
        ("a warp, y + 0.01 sin(3x + 2y)", lambda p: (p[0], p[1] + 0.01 * math.sin(3 * p[0] + 2 * p[1]), 0.0)),
    ]
    for label, f in cases:
        n = len(bake(f))
        print(f"  {label}: {n} distinct outlines of {r}; about {n * 734} new segments a frame")
    print(f"  all rows distinct: first motion frame {731 * 734} + {731 * 734} = {2 * 731 * 734} against 1<<20 = {limit}: refused by name, nothing unreachable yet")
    print(f"  with max_retained_segments raised: {731 * 734} unreachable > max_unreachable_segments {1 << 16}, so a rebuild on every later frame")
    print(f"  and at the 4-a-segment bound {4 * 2 * 731 * 734} pieces against MonoTable's fixed 1<<22 = {1 << 22}, which RenderOptions cannot change")

    print()
    print("Monotone table. fill.rs build_with_limits walks plan.shapes().shapes(), once an outline (fill.rs:594-603).")
    b = 98 * 1024 * 1024 / (536554 * 4)
    print(f"z2 cells4: at most 4 x 734 = {4 * 734} pieces, not 2,146,216; about {4 * 734 * b / 1024:.0f} KiB at the handoff's {b:.0f} B a piece, not 98 MiB")
    print("Sharing shrinks the table only. Raster still walks each instance, so it does not lower the raster estimate.")

    print()
    print("z2 cells4 memory, arithmetic only")
    records = 731 * 1473
    knots = 731 * 1471
    mib = lambda v: v / (1024 * 1024)
    r_mib = mib(records * 68)
    k_mib = mib(knots * 80)
    s_mib = mib(734 * 88)
    keys = mib(knots * 48)
    print(f"  stage records      {records} x 68 B = {r_mib:.0f} MiB")
    print(f"  live knots + keys  {knots} x 80 B = {k_mib:.0f} MiB (of which keys {keys:.0f} MiB)")
    print(f"  segments           734 x 88 B = {s_mib * 1024:.0f} KiB (handoff 45 MiB)")
    print(f"  still, geometry    {r_mib + k_mib + s_mib:.0f} MiB (handoff about 200)")
    print(f"  a fade adds two family copies (fade_in: target + start): {2 * r_mib:.0f} MiB more, {3 * r_mib:.0f} MiB of records (handoff has none of it)")
    peak = 3 * k_mib
    rebuild = 4 * k_mib + keys
    print(f"  repaint peak, 3L knots + keys {peak:.0f} MiB; during a rebuild {rebuild:.0f} MiB (handoff 328, without the pending index's {keys:.0f} MiB)")
    inflight = 2 * 3 * keys
    print(f"  two in-flight frames at 3L keys: up to {inflight:.0f} MiB")
    print(f"  upper bound in a fade, before frame buffers and caches: {3 * r_mib + rebuild + inflight:.0f} MiB")

    print()
    print("Serial work on the scene thread, bytes only (no timing was measured)")
    for name, k in (("cells2", 366 * 741), ("cells4", 731 * 1471)):
        print(f"  z2 {name}: identity hash each capture {k} x 28 B = {k * 28 / 1e6:.1f} MB; key copy per in-flight frame {mib(k * 48):.0f} MiB at L, {mib(3 * k * 48):.0f} MiB at 3L")

    print()
    print("z2 at cells8 (handoff: 'the monotone limit would then need its own setting')")
    r8 = round(6.4 / (CELL4 / 2.0))
    knots8 = r8 * (2 * r8 + 9)
    curves8 = r8 * (r8 + 5)
    first = (limit // (r8 + 5)) + 1
    print(f"  {r8} x {r8}: knots {knots8}, 2x {2 * knots8}, segments {r8 + 3} (one outline), pieces at most {4 * (r8 + 3)}")
    print(f"  the unmentioned bind: input curves {curves8} against {limit}; crosses at row {first}, requested {first * (r8 + 5)}")
    print("  so cells8 needs max_input_curves raised as well as knots, and not the monotone limit")


if __name__ == "__main__":
    main()
