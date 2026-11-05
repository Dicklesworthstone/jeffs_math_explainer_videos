"""Measure the pole mark of 14_3 on extracted frames, against a clean frame of the same render.

For each frame in the window, compares the disc of the mark (the ground-coloured backing, radius 0.13
scene units) with the same disc on REF, a frame where the mark is on top of everything. It also says
how much of the pink ring survives. Works on any frames folder laid out like frames02 (frame-NNNN.png
plus sampling.json), so it can be run unchanged on the next preview of the repair, as long as chain_plane()
has not moved. Take REF from the first second of the scene, where one mark sits on top. Frames from a lossy
video do not reach zero: on preview02 the mark's clean frames read up to about 3.5 (worst 12), and the old
doubled mark 4.3. It measures sampled frames only: nothing between them, nothing about motion.

Usage: python3 -B pole_frames.py FRAMES_DIR REF_FRAME [FIRST_S LAST_S]     (PIL needs site-packages, so not -I)
"""

import json
import math
import sys
from pathlib import Path

from PIL import Image

FRAMES = Path(sys.argv[1])
REF = int(sys.argv[2])
FIRST, LAST = (float(sys.argv[3]), float(sys.argv[4])) if len(sys.argv) > 4 else (43.0, 86.0)

# chain_plane() is Plane::new(v(-3.2, -0.2), 5.4, (-2.2, 3.2), (-2.4, 2.4)): one plane unit is one scene
# unit, and its window's middle (0.5, 0) sits at scene (-3.2, -0.2). So z = 1 is at scene (-2.7, -0.2).
POLE_SCENE = (-2.7, -0.2)
BACK, RING, RING_W = 0.13, 0.085, 0.05


def load(n):
    return Image.open(FRAMES / f"frame-{n:04}.png").convert("RGB")


def discs(size):
    px = size[0] / (64.0 / 4.5)  # 14.22 scene units across
    cx, cy = size[0] / 2 + POLE_SCENE[0] * px, size[1] / 2 - POLE_SCENE[1] * px
    inside, band, core = [], [], []
    r = int(BACK * px) + 2
    for y in range(int(cy) - r, int(cy) + r + 2):
        for x in range(int(cx) - r, int(cx) + r + 2):
            d = math.hypot(x + 0.5 - cx, y + 0.5 - cy) / px
            if d <= BACK - 0.012:  # one 720p pixel inside the backing's antialiased edge
                inside.append((x, y))
            if abs(d - RING) <= RING_W / 2 - 0.008:
                band.append((x, y))
            if d <= RING - RING_W / 2 - 0.012:
                core.append((x, y))
    return inside, band, core


ref = load(REF)
inside, band, core = discs(ref.size)
ref_at = ref.load()
sampling = json.loads((FRAMES / "sampling.json").read_text(encoding="utf8"))
times = sampling["times"]
print(f"reference frame {REF} at {times[REF]} s; {len(inside)} pixels in the disc, {len(band)} on the ring, {len(core)} in its hole")
print("frame\ttime_s\tdisc_mean_abs_diff\tdisc_worst\tring_pixels_within_24\thole_pixels_within_12")
rows = []
for n, t in enumerate(times):
    if not FIRST <= t <= LAST or not (FRAMES / f"frame-{n:04}.png").exists():
        continue
    at = load(n).load()
    diffs = [sum(abs(a - b) for a, b in zip(at[p], ref_at[p])) / 3 for p in inside]
    ring = sum(1 for p in band if max(abs(a - b) for a, b in zip(at[p], ref_at[p])) <= 24)
    hole = sum(1 for p in core if max(abs(a - b) for a, b in zip(at[p], ref_at[p])) <= 12)
    rows.append((n, t, sum(diffs) / len(diffs), max(diffs), ring / len(band), hole / len(core)))
    print(f"{n}\t{t:.3f}\t{rows[-1][2]:.1f}\t{rows[-1][3]:.0f}\t{rows[-1][4]:.0%}\t{rows[-1][5]:.0%}")
worst = max(rows, key=lambda r: r[2])
print(f"worst mean difference {worst[2]:.1f} at frame {worst[0]} ({worst[1]} s); smallest ring survival {min(r[4] for r in rows):.0%}; smallest hole survival {min(r[5] for r in rows):.0%}")

# The name "1" sits up and to the right of the mark until sentence 4. The mark now stays over it in z-order,
# which matters only if they touch, so measure the gap on REF: the pink pixels beyond the ring's reach.
px = ref.size[0] / (64.0 / 4.5)
cx, cy = ref.size[0] / 2 + POLE_SCENE[0] * px, ref.size[1] / 2 - POLE_SCENE[1] * px
gaps = []
for y in range(int(cy - 0.5 * px), int(cy)):
    for x in range(int(cx), int(cx + 0.5 * px)):
        r, g, b = ref_at[x, y]
        d = math.hypot(x + 0.5 - cx, y + 0.5 - cy) / px
        if r > 110 and r > 1.6 * g and d > BACK + 0.02:
            gaps.append(d - BACK)
print(f"name '1' on frame {REF}: {len(gaps)} ink pixels, nearest {min(gaps):.3f} scene units ({min(gaps) * px:.1f} px) from the backing's edge" if gaps else f"name '1' not on frame {REF}")
