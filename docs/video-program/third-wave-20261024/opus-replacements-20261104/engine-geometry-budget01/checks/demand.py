#!/usr/bin/env python3
"""What a colour plate asks of the pinned renderer's retained plan (franken_manim a399d626).

Pure arithmetic with no renderer, so nothing here is a measurement. It mirrors, from reading:
  - fmn-render/src/stroke_profile.rs record_stations + from_records (knots of one row)
  - fmn-render/src/plan.rs sync_with_limits + Reachability (when a repaint is refused or compacts)
  - fmn-render/src/bin.rs fine-tile spans (binning load)
The positive controls below tie the mirror to the logged failure and to an existing engine test.

    python3 -I -B demand.py
"""
import math
import struct

CELL = 0.035
LIMIT_NOW = 1 << 20
LIMIT_UPSTREAM = 1 << 22


def f32(x):
    return struct.unpack("f", struct.pack("f", x))[0]


def rust_round(x):
    return math.floor(x + 0.5)


# The two record layouts ------------------------------------------------------------------------


def blend_points(left, y, cell, thick, n):
    """explainer_corpus plate_layout::blend_points."""
    q = thick / 4
    right = left + cell * n
    centre = lambda k: left + cell * (k + 0.5)
    xs = [left - 2 * q, left - q, left, left, left, (left + centre(0)) / 2, centre(0)]
    for k in range(1, n):
        xs += [(centre(k - 1) + centre(k)) / 2, centre(k)]
    xs += [(centre(n - 1) + right) / 2, right, right, right, right + q, right + 2 * q]
    return [(f32(x), f32(y)) for x in xs]


def blend_rgba(colours):
    n = len(colours)
    out = [colours[0] + (0.0,)] * 3 + [colours[0] + (1.0,)] * 4
    for k in range(1, n):
        a, b = colours[k - 1], colours[k]
        out += [tuple((a[i] + b[i]) / 2 for i in range(3)) + (1.0,), b + (1.0,)]
    out += [colours[-1] + (1.0,)] * 2 + [colours[-1] + (0.0,)] * 4
    return out


def steps_points(left, y, cell, thick, n):
    """explainer_kit props::strip_points, PLATE_NOW's layout."""
    q = thick / 4
    xs = [left - 2 * q, left - q, left]
    for k in range(n):
        a0, a1 = left + cell * k, left + cell * (k + 1)
        xs += [a0, a0, (a0 + a1) / 2, a1]
    end = left + cell * n
    xs += [end, end, end + q, end + 2 * q]
    return [(f32(x), f32(y)) for x in xs]


def steps_rgba(colours):
    out = [colours[0] + (0.0,)] * 3
    for c in colours:
        out += [c + (1.0,)] * 4
    out += [colours[-1] + (0.0,)] * 4
    return out


def knots(points, rgba):
    """record_stations then from_records: three stations per curve of nonzero length, a shared
    anchor once, a knot dropped only when identical to the last. Assumes no subpath break, which
    the logged failure confirms for this layout (controls below)."""
    assert len(points) == len(rgba) and len(points) % 2 == 1
    curves = []
    total = 0.0
    for i in range((len(points) - 1) // 2):
        p0, p1, p2 = points[2 * i : 2 * i + 3]
        length = math.dist(p0, p1) + math.dist(p1, p2)
        if length > 0:
            curves.append((i, length))
            total += length
    stations, distance = [], 0.0
    for i, length in curves:
        for offset, fraction in ((0, 0.0), (1, 0.5), (2, 1.0)):
            station = (2 * i + offset, (distance + fraction * length) / total)
            if not stations or stations[-1] != station:
                stations.append(station)
        distance += length
    out = []
    for index, s in stations:
        knot = (s, rgba[index])
        if not out or out[-1] != knot:
            out.append(knot)
    return len(out), len(curves), (len(points) - 1) // 2


def colours(row, n):
    return [((row + 1) / 4096, (k + 1) / 4096, 0.5) for k in range(n)]


def blend_row(n, cell=0.00875, left=-3.2, y=0.25, row=0):
    return knots(blend_points(left, y, cell, cell, n), blend_rgba(colours(row, n)))


def steps_row(n, cell=0.035, left=-3.2, y=0.25, row=0):
    return knots(steps_points(left, y, cell, cell, n), steps_rgba(colours(row, n)))


def plate_row(n, blend):
    """(knots, positive curves, input curves) of one row; exact, cached by formula after checking."""
    if blend:
        return (2 * n + 9, n + 3, n + 5)
    return (3 * (n + 2), n + 2, 2 * n + 3)


# Admission over a repaint ----------------------------------------------------------------------


def repaint(rows, per_row, limit, frames=64, unreachable_rows=1 << 12):
    """Every row restyled every frame, shapes reused. Returns what happens, frame by frame."""
    live = rows * per_row
    if live > limit:
        crossing = limit // per_row + 1
        return {"refused_frame": 0, "requested": crossing * per_row, "peak": 0, "compactions": []}
    retained_rows, retained, peak, compactions = rows, live, live, []
    for frame in range(1, frames):
        unreachable = retained_rows - rows
        if unreachable > unreachable_rows:
            retained_rows, retained = rows, live
            compactions.append((frame, "UnreachableBudget"))
        elif retained + live > limit:
            if unreachable > 0:
                retained_rows, retained = rows, live
                compactions.append((frame, "AdmissionBoundary"))
            else:
                crossing = (limit - retained) // per_row + 1
                return {
                    "refused_frame": frame,
                    "requested": retained + crossing * per_row,
                    "peak": peak,
                    "compactions": compactions,
                }
        else:
            retained_rows += rows
            retained += live
        peak = max(peak, retained)
    return {"refused_frame": None, "requested": None, "peak": peak, "compactions": compactions}


# Controls --------------------------------------------------------------------------------------


def controls():
    for n in (1, 2, 3, 4, 8, 16, 183, 366, 640, 731):
        got = blend_row(n)
        assert got[0] == 2 * n + 9 and got[1] == n + 3 and got[2] == n + 5, (n, got)
    for n in (4, 16, 183, 366):
        got = steps_row(n)
        assert got[0] == 3 * (n + 2) and got[1] == n + 2 and got[2] == 2 * n + 3, (n, got)
    # the same row at the test fixture's scale and at another height
    assert blend_row(4, cell=1.0, left=-2.0, y=1.5, row=3)[0] == 17
    assert blend_row(731, cell=0.00875, left=-3.198, y=-3.19, row=730)[0] == 1471
    print("control: blended row knots = 2n + 9, positive curves n + 3, input curves n + 5, n up to 731")
    print("control: stepped row knots = 3(n + 2), positive curves n + 2, input curves 2n + 3 (fewer if neighbours match)")
    crossing = LIMIT_NOW // 1471 + 1
    assert (crossing, crossing * 1471) == (713, 1_048_823)
    print("control: logged failure reproduced: row", crossing, "of 731, requested", crossing * 1471, "limit", LIMIT_NOW)
    # engine test a_hard_boundary_compacts_when_rebuilding_the_live_state_fits, style axis:
    # one row, limit two: frame 1 admitted, frame 2 AdmissionBoundary
    run = repaint(1, 1, 2, frames=3)
    assert run["refused_frame"] is None and run["compactions"] == [(2, "AdmissionBoundary")], run
    print("control: engine test a_hard_boundary_compacts_...: frame 1 admitted, frame 2 compacts")
    # the patch's own test numbers
    assert repaint(4, 17, 135, frames=2)["requested"] == 136
    run = repaint(4, 17, 136, frames=6)
    assert run["refused_frame"] is None and [f for f, _ in run["compactions"]] == [2, 4], run
    assert (16 * 41, 3 * 41) == (656, 123)
    print("control: patch test numbers: 4x4 refused at 135 asking 136; 136 compacts on frames 2, 4; 16x16 total 656, third row 123")


# Rungs -----------------------------------------------------------------------------------------

RUNGS = {
    "now": (CELL, CELL, False),
    "blend": (CELL, CELL, True),
    "rows2": (CELL / 2, CELL, True),
    "cells2": (CELL / 2, CELL / 2, True),
    "cells4": (CELL / 4, CELL / 4, True),
}


def cells(width, height, rung):
    row, col, _ = RUNGS[rung]
    return max(rust_round(width / col), 8), max(rust_round(height / row), 8)


def plate(width, height, rung):
    cols, rows = cells(width, height, rung)
    k, seg, cur = plate_row(cols, RUNGS[rung][2])
    return {"cols": cols, "rows": rows, "per_row": k, "knots": rows * k, "segments": rows * seg, "curves": rows * cur}


def rungs():
    print("\nLab stills, geometry only (a lab still adds nothing but the plate; plate_lab.rs still())")
    print(f"{'plate':<26}{'cells':>9}{'k/row':>7}{'knots':>11}{'2x knots':>11}{'segments':>10}{'curves':>10}")
    for name, side in (("z2 6.4 wide", 6.4), ("sinrecip 5.6 wide", 5.6)):
        for rung in RUNGS:
            p = plate(side, side, rung)
            shape = f"{p['cols']}x{p['rows']}"
            print(f"{name + ' ' + rung:<26}{shape:>9}{p['per_row']:>7}{p['knots']:>11}{2 * p['knots']:>11}{p['segments']:>10}{p['curves']:>10}")


# The film, had PLATE been cells4 ---------------------------------------------------------------


def size(width, x, y):
    return width, width * (y[1] - y[0]) / (x[1] - x[0])


READING = size(6.4, (-2.0, 2.0), (-2.0, 2.0))
PANEL = (4.2, 2.6)
ZOOM = size(5.0, (0, 1.3), (0, 1.3))
SCENES = [
    ("04 reading plane", [READING]),
    ("05 square", [size(5.6, (-2, 2), (-2, 2))]),
    ("05 exp", [size(3.2, (-3, 3), (-4, 8))]),
    ("05 trig", [size(8.4, (-6.6, 6.6), (-3.3, 3.3))]),
    ("05 gamma", [size(8.4, (-4.5, 4.5), (-2.5, 2.5))]),
    ("05 zeta + strip", [size(6.4, (-7, 3), (-3, 3)), (2.0, 5.9)]),
    ("05_9 six panels", [PANEL] * 6),
    ("05_9 cubic panel + zoom", [PANEL, ZOOM]),
    ("08 bump (one small)", [size(1.5, (-1.2, 1.2), (-1.2, 1.2))]),
    ("rigidity sin(1/z)", [size(5.6, (-0.42, 0.42), (-0.42, 0.42))]),
    ("rigidity sine", [size(6.0, (-4, 4), (-3.5, 3.5))]),
    ("continuing bump", [size(5.6, (-2.5, 2.5), (-2.5, 2.5))]),
    ("continuing lacunary", [size(5.4, (-1.05, 1.05), (-1.05, 1.05))]),
    ("continuing gamma", [size(7.5, (-4.8, 3.0), (-2, 2))]),
    ("residues three small", [size(2.8, (-1.2, 1.2), (-1.2, 1.2))] * 3),
    ("residues bump", [size(5.0, (-2, 2), (-2, 2))]),
    ("one point painted", [size(5.4, (-2, 2), (-2, 2))]),
    ("counting input", [size(5.6, (-2, 2), (-2, 2))]),
    ("boundary picture", [size(5.6, (-2.5, 2.5), (-2.5, 2.5))]),
]


def film():
    for rung in ("now", "cells2", "cells4"):
        print(f"\nThe film's painted plates had PLATE been {rung}: geometry only, overlays not counted")
        print(f"{'scene':<26}{'knots':>11}{'2x knots':>11}{'segments':>10}{'curves':>10}  under 1<<20 still/repaint, 1<<21 repaint, 1<<22 repaint")
        worst = None
        for name, sizes in SCENES:
            ps = [plate(w, h, rung) for w, h in sizes]
            k, s, c = sum(p["knots"] for p in ps), sum(p["segments"] for p in ps), sum(p["curves"] for p in ps)
            verdict = lambda limit: "yes" if 2 * k <= limit else "NO"
            print(f"{name:<26}{k:>11}{2 * k:>11}{s:>10}{c:>10}  {'yes' if k <= LIMIT_NOW else 'NO'}/{verdict(LIMIT_NOW)}, {verdict(1 << 21)}, {verdict(1 << 22)}")
            if worst is None or k > worst[1]:
                worst = (name, k, s, c)
        print("worst:", worst)


# Cadence and memory ----------------------------------------------------------------------------

KNOT_BYTES = 32  # f64 s, f32 width, 4 f32 rgba, aligned
KEY_BYTES_PER_KNOT = 48  # Style::bits: six u64 per knot, the key StyleTable.index holds
SEGMENT_BYTES = 88
RECORD_BYTES = 68  # vmobject schema: seventeen f32 lanes
MIB = 1 << 20


def cadence():
    print("\nA repaint of every row, every frame (fade, recolour); shapes reused")
    print(f"{'plate':<18}{'limit':>10}  outcome")
    z2, sin = plate(6.4, 6.4, "cells4"), plate(5.6, 5.6, "cells4")
    panels = plate(*PANEL, "cells4")
    cases = [("z2 cells4", z2["rows"], z2["per_row"]), ("sinrecip cells4", sin["rows"], sin["per_row"]), ("six panels cells4", 6 * panels["rows"], panels["per_row"]), ("z2 cells2", plate(6.4, 6.4, "cells2")["rows"], plate(6.4, 6.4, "cells2")["per_row"])]
    for name, rows, per_row in cases:
        for limit in (1 << 20, 1 << 21, 1 << 22):
            run = repaint(rows, per_row, limit)
            if run["refused_frame"] is not None:
                text = f"REFUSED at frame {run['refused_frame']}, requested {run['requested']}"
            else:
                period = run["compactions"][1][0] - run["compactions"][0][0]
                reason = run["compactions"][0][1]
                text = f"admitted; compacts every {period} frames ({reason}), peak {run['peak']} knots, {run['peak'] * (KNOT_BYTES + KEY_BYTES_PER_KNOT) / MIB:.0f} MiB of knots and keys"
            print(f"{name:<18}{'1<<' + str(int(math.log2(limit))):>10}  {text}")
    print("\nz2 cells4, in memory, by reading (not measured)")
    records = z2["rows"] * (2 * z2["cols"] + 11)
    print(f"  stage records      {records} x {RECORD_BYTES} B = {records * RECORD_BYTES / MIB:.0f} MiB")
    print(f"  live knots + keys  {z2['knots']} x {KNOT_BYTES + KEY_BYTES_PER_KNOT} B = {z2['knots'] * (KNOT_BYTES + KEY_BYTES_PER_KNOT) / MIB:.0f} MiB")
    print(f"  segments           {z2['segments']} x {SEGMENT_BYTES} B = {z2['segments'] * SEGMENT_BYTES / MIB:.0f} MiB")
    run = repaint(z2["rows"], z2["per_row"], 1 << 22)
    print(f"  repaint peak at 1<<22: {run['peak']} knots = {run['peak'] * (KNOT_BYTES + KEY_BYTES_PER_KNOT) / MIB:.0f} MiB; a rebuild holds the old plan plus one live set: {(run['peak'] + z2['knots']) * (KNOT_BYTES + KEY_BYTES_PER_KNOT) / MIB:.0f} MiB")
    print(f"  each frame still in flight can pin one more copy of the keys: up to {run['peak'] * KEY_BYTES_PER_KNOT / MIB:.0f} MiB each (Arc::make_mut, frames_in_flight 2)")


# Binning ---------------------------------------------------------------------------------------


def binning():
    print("\nBinning, z2 cells4, one partition (Binning::build), stroke half height + 3 px margin; default fine 16, macro 128")
    print(f"{'output':<12}{'px/cell':>8}{'fine refs':>11}{'limit':>11}{'macro refs':>11}{'partition cells':>16}{'limit':>10}{'working MiB':>13}{'limit':>7}")
    p = plate(6.4, 6.4, "cells4")
    for w, h in ((1280, 720), (3840, 2160)):
        scale = h / 8
        cell = 6.4 / p["rows"]
        x0, x1 = w / 2 + (-2.6 - 3.2 - cell / 2) * scale, w / 2 + (-2.6 + 3.2 + cell / 2) * scale
        fine, coarse = 0, 0
        for r in range(p["rows"]):
            y = h / 2 - (-0.25 + 3.2 - cell * (r + 0.5)) * scale
            half = cell * scale / 2 + 3
            for tile, kind in ((16, "fine"), (128, "macro")):
                span = (math.floor((x1 + 3) / tile) - math.floor((x0 - 3) / tile) + 1) * (math.floor((y + half) / tile) - math.floor((y - half) / tile) + 1)
                if kind == "fine":
                    fine += span
                else:
                    coarse += span
        tiles = math.ceil(w / 16) * math.ceil(h / 16)
        working = fine * 12 + tiles * 12 + p["rows"] * 72 + math.ceil(w / 128) * math.ceil(h / 128) * 8
        print(f"{w}x{h:<7}{cell * scale:>8.2f}{fine:>11}{1 << 24:>11}{coarse:>11}{tiles:>16}{1 << 24:>10}{working / MIB:>13.1f}{256:>7}")
    print("  monotone table: at most 4 pieces a segment, the bound fill.rs's own test uses:")
    print(f"  {p['segments']} x 4 = {4 * p['segments']} pieces against {1 << 22}; {4 * p['segments'] * 48 / MIB:.0f} MiB against 256 MiB")


def main():
    controls()
    rungs()
    film()
    cadence()
    binning()
    # the logged times on z2, for the raster extrapolation in the handoff
    blend, cells2 = (183 * 183, 1.1), (366 * 366, 2.6)
    slope = (cells2[1] - blend[1]) / (cells2[0] - blend[0])
    print(f"\nRaster, extrapolated from the 720p lab log, z2 blend 1.1 s and cells2 2.6 s, linear in cells: cells4 about {blend[1] + (731 * 731 - blend[0]) * slope:.1f} s a frame. A guess, not a measurement.")
    n721 = 721 * (2 * 721 + 9)
    n722 = 722 * (2 * 722 + 9)
    print(f"Largest square blended plate under 1<<20: 721 x 721 = {n721} knots; 722 x 722 = {n722}.")


if __name__ == "__main__":
    main()
