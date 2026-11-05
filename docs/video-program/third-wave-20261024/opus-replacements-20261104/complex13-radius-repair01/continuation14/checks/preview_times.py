"""Decode the OLD preview at 0.1 s steps around 14_3's fades, and measure the mark's disc against 44.0 s.

The anchor formula's sentence times are read from this on the scratch voice. Its "clean" rows also give the
floor of the measure: H.264 moves these dark pixels a little even where the mark is truly on top. It says
nothing about the repair. It runs ffmpeg once per row on a short seek and writes PNGs only under OUT_DIR.

Usage: python3 -B preview_times.py PREVIEW_MP4 OUT_DIR     (PIL needs site-packages, so not -I)
"""

import math
import subprocess
import sys
from pathlib import Path

from PIL import Image

MP4, OUT = sys.argv[1], Path(sys.argv[2])
OUT.mkdir(parents=True, exist_ok=True)
CX, CY, PX = 397.0, 378.0, 90.0
TIMES = (
    [44.0]
    + [49.9 + 0.1 * k for k in range(8)]
    + [58.6 + 0.1 * k for k in range(12)]
    + [65.3 + 0.1 * k for k in range(16)]
    + [71.2 + 0.1 * k for k in range(6)]
    + [73.9 + 0.1 * k for k in range(8)]
)


def grab(t):
    p = OUT / f"t{t:07.3f}.png"
    if not p.exists():
        subprocess.run(["ffmpeg", "-v", "error", "-ss", f"{t:.3f}", "-i", MP4, "-frames:v", "1", "-s", "1280x720", str(p)], check=True)
    return Image.open(p).convert("RGB")


ref = grab(44.0).load()
disc = [(x, y) for y in range(360, 396) for x in range(379, 415) if math.hypot(x + 0.5 - CX, y + 0.5 - CY) / PX <= 0.118]
print("time_s\tdisc_mean_abs_diff\tdisc_worst")
for t in TIMES:
    at = grab(t).load()
    diffs = [sum(abs(a - b) for a, b in zip(at[p], ref[p])) / 3 for p in disc]
    print(f"{t:.3f}\t{sum(diffs) / len(diffs):.1f}\t{max(diffs):.0f}")
