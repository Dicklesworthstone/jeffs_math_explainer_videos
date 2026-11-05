#!/usr/bin/env python3
"""Arithmetic behind REVIEW.md. Python only: python3 -I -B budget_arithmetic.py

Every number is a source conclusion: charging rules read from the patched review copies, row sizes
from struct_sizes_mirror.out (a standalone mirror, not the crate) cross-checked against hand layouts
below. Nothing here compiles, runs or measures the engine.
"""
from pathlib import Path
import math

HERE = Path(__file__).resolve().parent
MIB, GIB = 2**20, 2**30
failures = []


def check(ok, what):
    print(("  PASS " if ok else "  FAIL ") + what)
    if not ok:
        failures.append(what)


def pow2_at_least(n):
    """Capacity a Vec reaches from empty by push alone: 4, 8, 16, ... (rows of at most 1 KiB)."""
    return max(4, 1 << (n - 1).bit_length())


print("-- row sizes: rustc mirror against hand layouts")
size = {}
for line in (HERE / "struct_sizes_mirror.out").read_text().splitlines():
    if line and not line.startswith("#"):
        name, value = line.split()
        size[name] = int(value)
check(len(size) == 23, f"{len(size)} mirror rows read")
# Hand layouts: field bytes summed, then rounded up to the type's alignment. Option<T> without a
# niche adds one tag padded to T's alignment.
hand = {
    "Segment": 11 * 8,
    "MonoPiece": 6 * 8,
    "JoinWedge": 13 * 8,
    "Placement": 12 * 8,
    "Mob": 8 + 4 + 4,
    "PreparedSegment": math.ceil((32 + 8 + 16 + 48 + 1 + 1) / 8) * 8,
    "Instance": math.ceil((4 + 4 + 16 + 96 + 4 + 8 + 1 + 1) / 8) * 8,
    "Style": math.ceil((4 * 16 + 2 * 4 + 2 * 8 + 2 * 4 + 1 + 8 + 24 + 128 + 4) / 8) * 8,
    "Hint": 48 + 8,
    "FillKernel": 40 + 8,
    "Shape": 32 + 4 + 4 + 72 + 56 + 48 + 24,
    # first_segment, segment_count, shape; translate; 2 x Option<PoolRange>; style; kernel; joins;
    # Option<StrokeRef>; Option<FieldRef>; Option<[f32; 4]>; fill_slab; two bools.
    "Draw": math.ceil((12 + 16 + 2 * 12 + 264 + 48 + 8 + 48 + 36 + 20 + 32 + 2) / 8) * 8,
    "ProjectedCurvePiece": 88 + 72,
}
for name, value in hand.items():
    check(size[name] == value, f"{name}: mirror {size[name]}, hand {value}")
check(size["OptionDraw"] == size["Draw"], "Option<Draw> uses a niche: no extra tag")
# Agreement with the handoffs' own figures (two readers, not proof):
check(size["Segment"] == 88 and size["MonoPiece"] == 48 and size["ProjectedCurvePiece"] == 160,
      "Segment 88, MonoPiece 48 and ProjectedCurvePiece 160 equal the handoffs' SEGMENT, PIECE and CURVE_PIECE")
# Negative control: a hand layout that forgets alignment does not match.
check(32 + 8 + 16 + 48 + 1 + 1 != size["PreparedSegment"], "unpadded PreparedSegment (106) is rejected")

print("\n-- binning working bytes: bin.rs add_table_bytes, term by term")


def grid(w, h, tile):
    return math.ceil(w / tile), math.ceil(h / tile)


def working(refs, instances, macro_cells, macro_refs, tiles, partitions=1):
    return (
        4 * refs + 4 * refs + 4 * refs      # fine refs, draws, flags
        + 40 * instances                   # bounds
        + 8 * instances + 24 * instances   # instance counts, instance rows
        + 8 * macro_cells + 24 * macro_cells
        + 4 * macro_refs
        + 16 + 24 * partitions + 24 * partitions  # ranges, count rows, cursor rows
        + 4 * partitions * tiles + 4 * partitions * tiles  # count cells, cursor cells
        + 4 * (tiles + 1)                  # offsets
    )


def prune(refs, tiles):
    """prune_occluded: offsets, draws and flags copied beside the originals."""
    return 2 * (4 * (tiles + 1) + 8 * refs)


fx, fy = grid(3840, 2160, 16)
mx, my = grid(3840, 2160, 128)
T, M = fx * fy, mx * my
check((T, M) == (32_400, 510), f"4K certified grid: {T} fine, {M} macro")
per_instance = working(T, 1, M, M, T) - working(0, 0, M, 0, T)
constant = working(0, 0, M, 0, T)
check((per_instance, constant) == (390_912, 405_188), f"full-frame working bytes = I x {per_instance} + {constant}")
ruby = {517: (16_750_800, 193, 256), 518: (16_783_200, 193, 256), 1024: (33_177_600, 382, 506), 4096: (132_710_400, 1527, 2025)}
for n, (refs, w, p) in ruby.items():
    got = (n * T, round(working(n * T, n, M, n * M, T) / MIB), round(prune(n * T, T) / MIB))
    check(got == (refs, w, p), f"{n} full-frame instances: refs {got[0]}, working {got[1]} MiB, prune {got[2]} MiB, as the frame-table handoff")
# Negative control: dropping the flags column cannot reproduce the 4,096 row.
wrong = working(4096 * T, 4096, M, 4096 * M, T) - 4 * 4096 * T
check(round(wrong / MIB) != 1527, f"without flags the 4,096 row would read {round(wrong / MIB)} MiB, so the flags term is load-bearing")
for n in ruby:
    check(prune(n * T, T) > working(n * T, n, M, n * M, T),
          f"{n}: prune peak exceeds the build peak, so the binning peak is the prune copy, not their sum")

print("\n-- the facade binning control at 1920x1080, frame_height 4")
fx, fy = grid(1920, 1080, 16)
mx, my = grid(1920, 1080, 128)
t, m = fx * fy, mx * my
px_per_unit = 1080 / 4.0
check(8 * px_per_unit >= 1920 and 6 * px_per_unit >= 1080, f"the 8x6 polygon spans {8 * px_per_unit:.0f}x{6 * px_per_unit:.0f} px, the whole frame")
check(2057 * t > 2**24 >= 2056 * t, f"2,057 x {t} = {2057 * t} is the first layer count over 2^24")
w1080 = working(2057 * t, 2057, m, 2057 * m, t)
check(w1080 == 202_782_632 and w1080 < 256 * MIB, f"working bytes {w1080}, under the default 256 MiB, so the fine-reference refusal is the one named")
check(2057 * t == 16_785_120, "covers_tile runs once per reference: 16,785,120 convex tests, each building a Vec of 8 points, per admitted frame")

print("\n-- the frame-table hook example: refs 2^25, working 2^29, max_pieces 2^23, default table bytes 2^28")
table_bytes = 2**28
pieces_cap = (table_bytes - 8) // size["MonoPiece"]
check(pieces_cap == 5_592_405 and 2**23 > pieces_cap,
      f"one shape range leaves {pieces_cap} pieces under max_table_bytes, so max_pieces 2^23 = {2**23} is inert above it")
n_full = 2**25 // T
w_hook = working(n_full * T, n_full, M, n_full * M, T)
check(n_full == 1035 and w_hook < 2**29, f"{n_full} full-frame instances reach 2^25 refs first; working {w_hook / MIB:.1f} MiB < 512 MiB")
p_hook = prune(2**25, T)
check(p_hook > 512 * MIB, f"prune at 2^25 refs: {p_hook} B = {p_hook / MIB:.2f} MiB, just above the declared working budget and outside it")
mono_shared, mono_local = table_bytes, table_bytes  # checked separately (handoff: "up to twice it")
cache = 3840 * 2160 * 8  # PixelTileCache: RGBA16F payload per tile, a full frame when every tile is held
team = mono_shared + mono_local + p_hook + cache
check(cache == 66_355_200, f"tile cache, every tile held: {cache / MIB:.1f} MiB")
check(1.0 * GIB < team < 1.1 * GIB, f"budgeted ceiling per team: 2 x 256 MiB mono + {p_hook / MIB:.0f} MiB binning + {cache / MIB:.0f} MiB cache = {team / GIB:.3f} GiB")
check(abs(2 * team / GIB - 2.124) < 0.001, f"two teams (frames_in_flight default 2): {2 * team / GIB:.3f} GiB before arena rows, plan generations and raw frames")

print("\n-- unbudgeted affine arena rows, z^2 cells4 rows (731 rows x 734 segments a generation)")
ROWS, SEGS = 731, 734
seg = ROWS * SEGS
check(seg == 536_554, f"{seg} segments a generation, as both handoffs")
for gens in (1, 2):
    n = gens * seg
    rows_len = n * size["Segment"] + n * size["PreparedSegment"] + gens * ROWS * size["OptionDraw"]
    rows_cap = (pow2_at_least(n) * size["Segment"] + pow2_at_least(n) * size["PreparedSegment"]
                + pow2_at_least(gens * ROWS) * size["OptionDraw"])
    print(f"  {gens} generation(s), rotating and stroked: transformed segments {size['Segment']} + prepared stroke {size['PreparedSegment']} a segment, draws {size['OptionDraw']} a row")
    print(f"    rows {rows_len} B = {rows_len / MIB:.1f} MiB; push-grown capacity {rows_cap} B = {rows_cap / MIB:.1f} MiB, kept for the team's life")
    if gens == 1:
        check(abs(rows_len / MIB - 102.7) < 0.05 and abs(rows_cap / MIB - 200.5) < 0.05, "one generation: 102.7 MiB of rows, 200.5 MiB held")
    else:
        check(abs(rows_cap / MIB - 401.0) < 0.05, "two generations: 401.0 MiB held (the draw pool rounds to 2,048 rows)")
# A still plate: the rows share one 734-segment outline through placements (the geometry
# cross-review's "segments, by reading: 734"). Stroke rows are per draw, so they do not shrink.
still_rows = seg * size["PreparedSegment"] + ROWS * size["OptionDraw"]
still_cap = pow2_at_least(seg) * size["PreparedSegment"] + pow2_at_least(ROWS) * size["OptionDraw"]
shared_plan = SEGS * size["Segment"]
check(abs(still_rows / MIB - 57.67) < 0.01 and abs(still_cap / MIB - 112.5) < 0.01 and shared_plan < 64 * 1024,
      f"still plate, one shared outline: plan segments {shared_plan} B, yet stroke and draw rows {still_rows / MIB:.2f} MiB, held {still_cap / MIB:.1f} MiB a team")
mono2 = 2 * ROWS * (SEGS + 1) * size["MonoPiece"]
bins2 = 16 * 3_380_000
check(mono2 + bins2 < 110 * MIB, f"the governed tables for two generations: mono {mono2 / MIB:.1f} MiB + binning about {bins2 / MIB:.1f} MiB")

print("\n-- plan generations (affine make_mut and camera clone), z^2 cells4 rows")
gen_segments = seg * size["Segment"]
arc = seg * 16 + ROWS * 8  # curve_lengths 8 a curve, cumulative 8 a curve plus one
gen = gen_segments + arc + ROWS * size["Shape"] + ROWS * 4 + ROWS * size["Instance"]
check(round(gen_segments / MIB) == 45, f"segments alone {gen_segments / MIB:.2f} MiB: the camera handoff's 45 MiB")
check(abs(gen / MIB - 53.49) < 0.01, f"with arc-length tables, shape rows, one subpath start and instance rows: {gen} B = {gen / MIB:.2f} MiB, still before styles and maps")
check(abs(3 * gen / MIB - 160.5) < 0.1, f"live plan plus 2 frames in flight: {3 * gen / MIB:.1f} MiB; with two row generations in the plan: {6 * gen / MIB:.1f} MiB")

print("\n-- the ignored monotone control (1,365 shifted outlines of 1,024 quadratics)")
SHAPES, CURVES = 1_365, 1_024
records, curves, pieces = SHAPES * (2 * CURVES + 1), SHAPES * CURVES, SHAPES * (3 * CURVES + 1)
ruby_sum = records * 68 + curves * size["Segment"] + pieces * size["MonoPiece"]
check(ruby_sum == 514_534_020 and round(ruby_sum / 1e9, 2) == 0.51 and round(ruby_sum / GIB, 2) == 0.48,
      f"the handoff's sum: {ruby_sum} B = 0.51 GB = {ruby_sum / GIB:.2f} GiB, so BUILD.md's 0.51 GB is decimal and the #[ignore] text's 0.55 GiB matches neither")
arc_c = curves * 16 + SHAPES * 8
copy = curves * size["Segment"] + arc_c + SHAPES * size["Shape"]
check(abs(copy / 1e6 - 145.7) < 0.1, f"arc-length tables add {arc_c / 1e6:.1f} MB, and each plan copy made while a frame is in flight about {copy / 1e6:.1f} MB")

print("\n-- camera projected curves: one try_reserve_exact(1) per curve on a fresh Vec")
for per in (1, 2, 4):
    n = SEGS * per
    copied = n * (n - 1) // 2 * size["ProjectedCurvePiece"]
    print(f"  {per} curve(s) a segment: {n} curves a row, {n} reallocation calls a row, {ROWS * n} a frame;"
          f" if every one moves, {copied / 1e6:.1f} MB copied a row, {ROWS * copied / 1e9:.1f} GB a frame")
check(ROWS * SEGS == 536_554 and SEGS * (SEGS - 1) // 2 * 160 == 43_041_760, "one curve a segment: 536,554 calls and at most 43.0 MB copied a row")

print()
print("RESULT " + ("PASS" if not failures else f"FAIL {len(failures)}"))
