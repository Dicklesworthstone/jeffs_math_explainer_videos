"""Arithmetic for engine-secondary-camera-budget01. Read from the engine source, not measured.

Nothing here ran the renderer. Struct sizes are added up by hand from their fields, because no
size_of has been run: f64 fields leave no padding, and a size marked "at least" leaves out an enum
tag or a neighbouring variant. The charges are logical bytes in the order compile_vector,
compile_surface and compile_dot charge them, not allocator capacity. Part 1 mirrors what the unrun
tests assert, and a mirror can share a misreading with them. Part 2 estimates scenes. Time per
frame is estimated only as a count of operations, never as seconds.
"""
import math

MIB = 2**20
MAX_DRAWS = 1 << 20
MAX_CURVES = 1 << 22
MAX_PIECES = 1 << 22
MAX_TRIANGLES = 1 << 22
MAX_WORKING = 512 * MIB
MAX_ENTRIES = 1 << 24
FRAME_HEIGHT = 8.0
FRAME_WIDTH = FRAME_HEIGHT * 16.0 / 9.0

VEC3 = 24
SEGMENT = 3 * VEC3 + 2 * 8
CLIPPED_QUADRATIC = 3 * VEC3 + 3 * 32 + 2 * 8
CLIPPED_FILL = 3 * VEC3 + 3 * 32
CURVE_PIECE = SEGMENT + 3 * VEC3
MONO_PIECE = 48
ATTRIBUTES = VEC3 + VEC3 + 4 * 8 + 2 * 8 + 8
RASTER_VERTEX = ATTRIBUTES + 2 * 8 + 8 + 8
RASTER_TRIANGLE = 3 * RASTER_VERTEX + 4 * 8
SURFACE_VERTEX = VEC3 + VEC3 + 4 * 8 + 2 * 8 + 8
TRUE_DOT = VEC3 + 8 + 4 * 8 + 8 + 8 + VEC3 + 8 + 16 * 8 + 8
# Shader::Dot's fields and its tag, the triangle Vec, depth_test, and Option<[f64; 4]> bounds.
COMPILED_DRAW_AT_LEAST = (VEC3 + 8 + 4 * 8 + 8 + VEC3 + VEC3 + 8 + VEC3 + VEC3 + 8) + 24 + 8 + 40

print("# Sizes, added by hand from the fields; no size_of was run")
for name, value in (
    ("Segment", SEGMENT),
    ("ClippedQuadratic", CLIPPED_QUADRATIC),
    ("ClippedFillQuadratic", CLIPPED_FILL),
    ("ProjectedCurvePiece", CURVE_PIECE),
    ("MonoPiece", MONO_PIECE),
    ("RasterTriangle", RASTER_TRIANGLE),
    ("SurfaceVertex", SURFACE_VERTEX),
    ("TrueDotDraw", TRUE_DOT),
    ("CompiledDraw, at least", COMPILED_DRAW_AT_LEAST),
):
    print(f"{name}: {value}")
assert (SEGMENT, CLIPPED_QUADRATIC, CLIPPED_FILL, CURVE_PIECE) == (88, 184, 168, 160)
assert (RASTER_TRIANGLE, SURFACE_VERTEX, TRUE_DOT) == (440, 104, 248)

STROKE_CURVE = SEGMENT + 2 * CLIPPED_QUADRATIC + CURVE_PIECE + SEGMENT
print(f"a stroked curve nothing clips or splits: {SEGMENT} + 2 x {CLIPPED_QUADRATIC} + {CURVE_PIECE} + {SEGMENT} = {STROKE_CURVE} bytes")
assert STROKE_CURVE == 704


def stroke_segment(pieces):
    """world segment, one clipped piece charged twice, then per curve its piece and screen segment."""
    return SEGMENT + 2 * CLIPPED_QUADRATIC + pieces * (CURVE_PIECE + SEGMENT)


def bins(width, height, curves, radius, box_px):
    """build_curve_bins: cell, grid, and memberships when every curve has the same control box."""
    extent = (width + 2 * radius, height + 2 * radius)
    cell = max(radius, math.sqrt(extent[0] * extent[1] / (2 * curves)))
    columns = max(math.ceil(extent[0] / cell), 1)
    rows = max(math.ceil(extent[1] / cell), 1)

    def bin_of(value, count):
        at = math.floor((value + radius) / cell)
        return 0 if at <= 0 else (count - 1 if at >= count - 1 else at)

    across = bin_of(box_px[2], columns) - bin_of(box_px[0], columns) + 1
    down = bin_of(box_px[3], rows) - bin_of(box_px[1], rows) + 1
    entries = curves * across * down
    bytes_needed = (columns * rows + 1) * 4 + entries * 4 + curves * 16
    return cell, columns, rows, entries, bytes_needed


print()
print("# Part 1: numbers the unrun tests assert")
print("-- a_bin_budget_above_the_default_bins_what_the_default_scans: 3,200 diagonals at 640x360")
to_px_x = lambda x: (x / (FRAME_WIDTH / 2) + 1) * 320
to_px_y = lambda y: (y / (FRAME_HEIGHT / 2) + 1) * 180
box = (to_px_x(-7.0), to_px_y(-3.9), to_px_x(7.0), to_px_y(3.9))
print(f"control box {box[0]:.2f}..{box[2]:.2f} by {box[1]:.2f}..{box[3]:.2f} px")
for radius in (0.5, 1.0, 2.0, 4.0):
    cell, columns, rows, entries, needed = bins(640, 360, 3200, radius, box)
    print(f"radius {radius}: cell {cell:.3f}, {columns}x{rows} = {columns * rows} bins, {entries} entries, {needed / MIB:.0f} MiB")
    assert MAX_ENTRIES < entries < 2**32 - 1
    assert needed < MAX_WORKING

print("-- a_working_byte_budget_above_the_default...: 800 shapes of 1,000 straight stroked curves")
SHAPES, CURVES = 800, 1000
curves = SHAPES * CURVES
charged = curves * STROKE_CURVE
print(f"{curves} curves x {STROKE_CURVE} = {charged} against {MAX_WORKING}; the plan admits {1 << 20} curves")
assert charged > MAX_WORKING and curves < 1 << 20
records = SHAPES * (2 * CURVES + 1)
held = records * 68 + 2 * curves * SEGMENT + curves * CURVE_PIECE
print(f"held: records {records} x 68, two plans at {SEGMENT} a segment, curves {curves} x {CURVE_PIECE}: {held / 1e9:.2f} GB before bins")

print("-- both_routes_refuse_each_camera_budget...: square and three dots")
print("draw rows 1 + 3 = 4; projected curves 4; raster triangles 3 x 2 = 6")

print()
print("# Part 2: which camera budget a scene reaches first, 3840x2160")
ROWS, ROW_SEGMENTS, SEGMENTS = 731, 734, 536554
assert ROWS * ROW_SEGMENTS == SEGMENTS
print(f"-- z^2 cells4 as stroked rows: {ROWS} vectors, {SEGMENTS} segments, by the budget01 demand")
area = 3842 * 2162
for pieces in (1, 2, 4):
    per = stroke_segment(pieces)
    total = SEGMENTS * per
    cell, columns, rows, _, _ = bins(3840, 2160, ROW_SEGMENTS * pieces, 1.0, (0, 0, 4, 4))
    bin_bytes = ROWS * ((columns * rows + 1) * 4 + ROW_SEGMENTS * pieces * (16 + 4 * 2))
    print(
        f"{pieces} curve(s) a segment: {per} bytes x {SEGMENTS} = {total / MIB:.0f} MiB, "
        f"about {bin_bytes / MIB:.0f} MiB of bins, {SEGMENTS * pieces} curves against {MAX_CURVES}"
    )
print("which row is real depends on perspective: a segment splits until it is within 1/256 px; that count is unmeasured")
print(f"fill, if any, adds at least 72 + 2 x {CLIPPED_FILL} + {MONO_PIECE} = {72 + 2 * CLIPPED_FILL + MONO_PIECE} bytes a segment")

print("-- plan clone, once per camera frame in flight, outside every budget")
for generations in (1, 2):
    print(f"{generations} generation(s) of segments: {SEGMENTS * generations * SEGMENT / MIB:.0f} MiB a frame before styles and shapes")

print("-- surface grids, G x G cells")
limit_g = math.floor(math.sqrt(MAX_WORKING / (2 * RASTER_TRIANGLE)))
triangle_g = math.floor(math.sqrt(MAX_TRIANGLES / 2))
print(f"working bytes bind near G = {limit_g} ({2 * limit_g**2} triangles); raster triangles alone would bind at G = {triangle_g}")
for grid in (500, 750, 1000, 1448):
    job = 2 * grid**2 * RASTER_TRIANGLE
    capture = (grid + 1) ** 2 * SURFACE_VERTEX + 6 * grid**2 * 4
    print(f"G = {grid}: {2 * grid**2} triangles, {job / MIB:.0f} MiB in the job, {capture / MIB:.0f} MiB of mesh a frame in flight")

print("-- dot clouds")
per_dot = COMPILED_DRAW_AT_LEAST + 2 * RASTER_TRIANGLE
dots = MAX_WORKING // per_dot
print(f"a visible dot charges at least {per_dot} bytes: working bytes bind by {dots} dots, not at {MAX_DRAWS} draws")
print(f"{MAX_DRAWS} visible dots need at least {MAX_DRAWS * per_dot / MIB:.0f} MiB in the job and {MAX_DRAWS * TRUE_DOT / MIB:.0f} MiB of commands a frame in flight")

print("-- stroke search: where one vector leaves its bins, with a default budget")
print("memberships are about 2 N^2 x (control box area / frame area), N curves in the one vector")
for fraction in (1.0, 0.1, 0.01, 0.0001):
    print(f"boxes covering {fraction:g} of the frame: scans above about {math.sqrt(MAX_ENTRIES / (2 * fraction)):.0f} curves")
print(f"{MAX_ENTRIES} memberships is {MAX_ENTRIES * 4 / MIB:.0f} MiB of entries")

print("-- operations a frame, not seconds")
tiles = math.ceil(3840 / 16) * math.ceil(2160 / 16)
print(f"{tiles} fine tiles; every tile tests every draw's bounds")
for name, draws in (("731 rows", ROWS), (f"{dots} dots", dots), (f"{MAX_DRAWS} dots", MAX_DRAWS)):
    print(f"{name}: {tiles * draws:.3e} bounds tests")

print("-- memory a camera frame can hold, by formula")
raw = 3840 * 2160 * 8
print(f"raw frame {raw / MIB:.0f} MiB")
for teams, working in ((2, 512), (2, 2048), (4, 2048)):
    print(f"{teams} teams at {working} MiB each: up to {teams * working + teams * raw / MIB:.0f} MiB, before plan clones and the scene")
