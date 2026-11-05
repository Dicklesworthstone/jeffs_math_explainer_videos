#!/usr/bin/env python3
"""What the lab's refusal and probe jobs should do, from reading the pinned engine. Not a measurement.

Frames: fmn-anim clock.rs FrameSample is 1-based with alpha = frame / n ("no zero-alpha emission
frame"); frame.rs complete_play_frame runs updaters after the animation and before the capture.
Paint: draw.rs Painter::build_styled rounds the blend to twelfths and writes only when it changes.
Rate: AnimateArgs leaves the animation's default, fmn-core rate.rs smooth.
Admission: the frozen estimator's repaint(), loaded from the sealed budget01 folder.

    python3 -I -B probe_desk.py
"""
import importlib.util
import math
from pathlib import Path

HERE = Path(__file__).resolve().parent
SPEC = importlib.util.spec_from_file_location("demand", HERE.parents[1] / "engine-geometry-budget01/checks/demand.py")
demand = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(demand)

ROWS, PER_ROW = 731, 1471
LIVE = ROWS * PER_ROW
CONTROL, BUDGET = 1 << 21, 1 << 22


def smooth(t):
    s = 1.0 - t
    return t**3 * (10.0 * s * s + 5.0 * s * t + t * t)


def rust_round(x):
    return math.floor(x + 0.5)


def twelfths(frames):
    return [rust_round(smooth(k / frames) * 12) for k in range(1, frames + 1)]


def distinct(paints):
    out = []
    for p in paints:
        if not out or out[-1] != p:
            out.append(p)
    return out


def outcome(frames, limit):
    paints = distinct(twelfths(frames))
    return paints, demand.repaint(ROWS, PER_ROW, limit, frames=len(paints))


demand.controls()
print()
assert LIVE == 1_075_301
assert LIVE <= CONTROL < 2 * LIVE and 3 * LIVE <= BUDGET < 4 * LIVE
print(f"z2 cells4: {LIVE} live knots. 1<<21 holds one set and not two; 1<<22 holds three and not four.")
print()
print("frames  paints in twelfths     distinct  under 1<<22")
for frames in (3, 4, 5, 6):
    paints, run = outcome(frames, BUDGET)
    first = [f for f, _ in run["compactions"]]
    print(f"{frames:>6}  {str(twelfths(frames)):<21}  {len(paints):>8}  peak {run['peak']}, compactions at capture index {first or 'none'}")
print()

_, short = outcome(3, BUDGET)
assert short["compactions"] == [], short
print("The existing 3-frame length never compacts at 1<<22, so it cannot test the crossing.")

paints, control = outcome(6, CONTROL)
assert paints == [0, 3, 6, 9, 12] and twelfths(6) == [0, 3, 6, 9, 12, 12]
assert control["refused_frame"] == 1 and control["requested"] == 2_097_646, control
print(f"Control, 6 frames at 1<<21: refused on the second paint, capture index {control['refused_frame']}, requested {control['requested']}, limit {CONTROL}.")
print(f"  crossing row: ({CONTROL} - {LIVE}) // {PER_ROW} + 1 = {(CONTROL - LIVE) // PER_ROW + 1} of the second set")
print("  exact only if no two rows in those two paints share a style; a smaller number would fault this estimate, not the budget.")

_, probe = outcome(6, BUDGET)
assert probe["refused_frame"] is None and [f for f, _ in probe["compactions"]] == [3], probe
assert probe["peak"] == 3 * LIVE == 3_225_903
print(f"Probe, 6 frames at 1<<22: admitted, first compaction at capture index 3 ({probe['compactions'][0][1]}), peak {probe['peak']} knots retained.")
print(f"  without that compaction the fourth paint would need {4 * LIVE}, over {BUDGET}: six frames written means relief ran, if all rows differ.")
print("  the last two paints are both 12/12 with no write between them, so frames 5 and 6 should be byte-identical.")
print("Control cost: one capture admitted, refused during the second, so about one frame of raster. Probe cost: six frames.")
