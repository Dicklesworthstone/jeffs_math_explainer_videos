"""Arithmetic for engine-secondary-budget01. Read from the engine source, not measured.

Part 1 mirrors the piece count (fill.rs monotone_split_times, layout_for_segments) and the tile
grid (bin.rs Grid::new) for the numbers the unrun tests assert. Part 2 estimates what real scenes
would ask of each budget. Nothing here ran the renderer: a mirror can share a misreading with the
tests it checks. Time per frame is not estimated at all.
"""
import math
import struct

MAX_PIECES = 1 << 22
MAX_TABLE_BYTES = 1 << 28
MAX_TILE_REFS = 1 << 24
MAX_WORKING = 256 * 1024 * 1024
PIECE = 48  # MonoPiece: three [f64; 2]
RANGE = 8  # (u32, u32)
SPLIT_EPS = 1e-9


def extremum(v0, v1, v2):
    b = 2.0 * (v1 - v0)
    c = (v2 - v1) - (v1 - v0)
    scale = max(abs(v0), abs(v1), abs(v2), 1.0)
    if abs(c) <= 1e-14 * scale:
        return None
    t = -b / (2.0 * c)
    return t if SPLIT_EPS < t < 1.0 - SPLIT_EPS else None


def piece_count(p0, p1, p2):
    candidates = []
    for axis in (1, 0):
        t = extremum(p0[axis], p1[axis], p2[axis])
        if t is not None:
            candidates.append(t)
    candidates.sort()
    accepted, base = 0, 0.0
    for t in candidates:
        local = (t - base) / (1.0 - base)
        if SPLIT_EPS < local < 1.0 - SPLIT_EPS:
            accepted += 1
            base = t
    return accepted + 1


def screen(p, scale=4.5, origin=(32.0, 18.0)):
    return (origin[0] + p[0] * scale, origin[1] + p[1] * scale)


def subpath_pieces(points, linear=None):
    def place(p):
        if linear is None:
            return p
        return (linear[0][0] * p[0] + linear[0][1] * p[1], linear[1][0] * p[0] + linear[1][1] * p[1])

    placed = [screen(place(p)) for p in points]
    total = 0
    for i in range(0, len(placed) - 2, 2):
        total += piece_count(placed[i], placed[i + 1], placed[i + 2])
    return total + (1 if placed[0] != placed[-1] else 0)


def grid(width, height, tile):
    return math.ceil(width / tile), math.ceil(height / tile)


def working_bytes(fine_refs, instances, macro_cells, macro_refs, tiles, partitions=1):
    """bin.rs add_table_bytes in order, for the terms that can bind."""
    return (
        12 * fine_refs
        + 72 * instances
        + 32 * macro_cells
        + 4 * macro_refs
        + 64 * partitions
        + 8 * partitions * tiles
        + 4 * (tiles + 1)
    )


print("# Part 1: numbers the unrun tests assert")

print("-- frame_table_limits.rs, 64x36 with 16/8 tiles")
fine = grid(64, 36, 8)
coarse = grid(64, 36, 16)
print(f"fine tiles {fine[0]}x{fine[1]} = {fine[0] * fine[1]}; macro tiles {coarse[0]}x{coarse[1]} = {coarse[0] * coarse[1]}")
assert fine[0] * fine[1] == 40 and coarse[0] * coarse[1] == 12

curve = [(0.0, 0.0), (1.0, 0.5), (1.5, 1.5)]
turned = [[0.7071, -0.7071], [0.7071, 0.7071]]
table = subpath_pieces(curve)
local = subpath_pieces(curve, turned)
print(f"open curve: {table} pieces in the table, {local} in the frame")
assert (table, local) == (2, 3)
print(f"bytes: table {table * PIECE + RANGE}, frame {local * PIECE}")
assert table * PIECE + RANGE == 104 and local * PIECE == 144

rectangle = [(-8, -5), (0, -5), (8, -5), (8, 0), (8, 5), (0, 5), (-8, 5), (-8, 0), (-8, -5)]
left, top = screen(rectangle[0])
right, bottom = screen(rectangle[4])
print(f"rectangle on screen: x {left}..{right}, y {top}..{bottom}; {subpath_pieces(rectangle)} pieces")
assert left <= 0 and top <= 0 and right >= 64 and bottom >= 36

print("-- fine references walked macro tile by macro tile, limit 39 and 12")
running, trail = 0, []
for row, rows in enumerate((2, 2, 1)):
    for _ in range(4):
        running += 2 * rows
        trail.append(running)
print(trail)
assert running == 40 and next(v for v in trail if v > 39) == 40

print("-- ignored monotone control, f32 records as the test writes them")
SHAPES, CURVES = 1365, 1024


def f32(x):
    return struct.unpack("f", struct.pack("f", x))[0]


step = f32(1.0 / CURVES)
for shape in (0, SHAPES // 2, SHAPES - 1):
    shift = f32(shape * f32(0.001))
    points = []
    for curve_index in range(CURVES + 1):
        at = f32(f32(curve_index * step) + shift)
        points.append((at, at))
        if curve_index < CURVES:
            points.append((f32(at + f32(2.0 * step)), f32(at + f32(3.0 * step))))
    pieces = subpath_pieces(points)
    print(f"shape {shape}: {pieces} pieces")
    assert pieces == 3 * CURVES + 1
demand = SHAPES * (3 * CURVES + 1)
print(f"{SHAPES} x {3 * CURVES + 1} = {demand} pieces against {MAX_PIECES}")
print(f"{SHAPES - 1} shapes would need {(SHAPES - 1) * (3 * CURVES + 1)}")
assert demand > MAX_PIECES > (SHAPES - 1) * (3 * CURVES + 1)
table_bytes = demand * PIECE + SHAPES * RANGE
print(f"table bytes {table_bytes} against {MAX_TABLE_BYTES}")
assert table_bytes < MAX_TABLE_BYTES
curves = SHAPES * CURVES
print(f"curves {curves}: over the plan's 1<<20, under the test's 1<<21")
assert 1 << 20 < curves < 1 << 21
records = SHAPES * (2 * CURVES + 1)
memory = records * 68 + curves * 88 + demand * PIECE
print(f"records {records} x 68 + segments {curves} x 88 + pieces {demand} x 48 = {memory / 1e9:.2f} GB, by arithmetic")

print("-- facade binning control, 1920x1080 with certified 16/128 tiles")
fine = grid(1920, 1080, 16)
coarse = grid(1920, 1080, 128)
tiles, macro_cells = fine[0] * fine[1], coarse[0] * coarse[1]
print(f"fine {fine[0]}x{fine[1]} = {tiles}; macro {coarse[0]}x{coarse[1]} = {macro_cells}")
LAYERS = 2057
refs = LAYERS * tiles
print(f"{LAYERS} x {tiles} = {refs} against {MAX_TILE_REFS}; {LAYERS - 1} layers need {(LAYERS - 1) * tiles}")
assert refs > MAX_TILE_REFS >= (LAYERS - 1) * tiles
macro_refs = LAYERS * macro_cells
bytes_needed = working_bytes(refs, LAYERS, macro_cells, macro_refs, tiles)
print(f"working bytes {bytes_needed} against {MAX_WORKING}")
assert bytes_needed < MAX_WORKING

print()
print("# Part 2: which budget a real scene reaches first, 3840x2160 with 16/128 tiles")
fine = grid(3840, 2160, 16)
coarse = grid(3840, 2160, 128)
T, M = fine[0] * fine[1], coarse[0] * coarse[1]
print(f"fine tiles {T}, macro tiles {M}")

print("-- full-frame instances: a hatch of diagonals references its whole bounding box")
full = MAX_TILE_REFS // T
print(f"{full} fit max_tile_references, {full + 1} refuse")
for instances in (full, full + 1, 1024, 4096):
    refs = instances * T
    # prune_occluded copies offsets, draws and flags beside the originals, outside the budget.
    prune = 2 * (4 * (T + 1) + 8 * refs)
    print(
        f"{instances} full-frame instances: {refs} references, "
        f"{working_bytes(refs, instances, M, instances * M, T) / 2**20:.0f} MiB working bytes, "
        f"{prune / 2**20:.0f} MiB while pruning"
    )
binding = (MAX_WORKING - 72 * 518 - 32 * M - 4 * 518 * M - 64 - 8 * T - 4 * (T + 1)) // 12
print(f"with only max_tile_references raised, working bytes bind near {binding} references, about {binding // T} instances")

print("-- rotating z^2 rows on the cells4 grid, by the validation02 demand")
ROWS, ROW_SEGMENTS, ROW_PX = 731, 734, 1512
pieces = ROWS * (ROW_SEGMENTS + 1)  # a straight row: one piece a segment and its chord
print(f"pieces a generation {pieces}; two generations {2 * pieces} against {MAX_PIECES}")
aabb = ROW_PX * math.cos(math.radians(45))
across = math.ceil(aabb / 16) + 1
refs = ROWS * across * across
print(f"a row at 45 degrees spans about {aabb:.0f} px, {across}x{across} tiles; about {refs} references against {MAX_TILE_REFS}")
print("the 4,292,432 in validation02 was a bound of 4 pieces a segment over two generations; a straight segment makes 1")
print("exception: rounding can make a nearly constant axis look curved at a turn near 0 or 90 degrees; the bound still holds")

print("-- where the two monotone budgets cross")
print(f"max_table_bytes alone admits {(MAX_TABLE_BYTES) // PIECE} pieces; raising max_pieces past that needs max_table_bytes too")
