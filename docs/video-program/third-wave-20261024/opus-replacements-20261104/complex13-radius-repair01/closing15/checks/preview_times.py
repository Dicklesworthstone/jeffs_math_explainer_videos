"""Measure the OLD preview at exact times around 15_4's sweep. It says nothing about the repair.

The 0.5 s frames02 grid catches only one of the four crossings, so this decodes between its samples.
Three tables:
  labels   how far each label's box and each pole's disc differ from the moment before the circle starts
  ground   whether the backings and the moved names would lie on bare ground, on the old frames
  poles    the marks against the moment they are alone, before the radius 1 ring comes in
Runs ffmpeg once per time on a short seek and writes PNGs only under OUT_DIR.

Usage: python3 -B preview_times.py PREVIEW_MP4 OUT_DIR     (PIL needs site-packages, so not -I)
"""

import math
import subprocess
import sys
from pathlib import Path

from PIL import Image

MP4, OUT = sys.argv[1], Path(sys.argv[2])
OUT.mkdir(parents=True, exist_ok=True)
A, R_END, R_START, GROW_S, G0 = 3.0, 3.162, 0.02, 1.4, 64.665
PX = 0.95 * 90
BEFORE_RING, MARK_ALONE = 64.66, 47.63
SIZES = {"sqrt10": (1.6743, 1.0217, 0.64, 0.12), "x=3": (3.0, -0.45, 0.32, 0.10), "i": (0.42, 1.28, 0.05, 0.12), "-i": (0.5, -1.28, 0.13, 0.12)}
MOVED = {"i moved": (-0.42, 1.28, 0.05, 0.12), "-i moved": (-0.5, -1.28, 0.13, 0.12)}
POLES = {"pole i": (0.0, 1.0), "pole -i": (0.0, -1.0)}
PAD = 0.05 / 0.95
GROUND_TOL = 14


def to_px(re, im):
    return 640 + (-1.6 + (re - 2.8) * 0.95) * 90, 360 + (0.2 - im * 0.95) * 90


def grab(t):
    p = OUT / f"t{t:07.3f}.png"
    if not p.exists():
        subprocess.run(["ffmpeg", "-v", "error", "-ss", f"{t:.3f}", "-i", MP4, "-frames:v", "1", "-s", "1280x720", str(p)], check=True)
    return Image.open(p).convert("RGB").load()


def box(re, im, w, h):
    x0, y0 = to_px(re - w, im + h)
    x1, y1 = to_px(re + w, im - h)
    return [(x, y) for y in range(int(y0), int(y1) + 1) for x in range(int(x0), int(x1) + 1)]


def disc(re, im, radius=0.118):
    cx, cy = to_px(re, im)
    n = int(radius * 90) + 2
    return [(x, y) for y in range(int(cy) - n, int(cy) + n + 1) for x in range(int(cx) - n, int(cx) + n + 1) if math.hypot(x + 0.5 - cx, y + 0.5 - cy) / 90 <= radius]


def diff(at, ref, pixels):
    return sum(sum(abs(a - b) for a, b in zip(at[p], ref[p])) / 3 for p in pixels) / len(pixels)


ref = grab(BEFORE_RING)
names = list(SIZES)
print(f"LABELS: mean difference (0-255) in each label's box against {BEFORE_RING} s; r is the circle's radius by the source's rate, with G0 {G0}")
print("time_s\tr\t" + "\t".join(names))
for t in [BEFORE_RING, 64.75, 64.82, 64.88, 64.95] + [65.0 + 0.1 * k for k in range(8)] + [65.55, 65.8, 65.87, 65.92, 65.96, 66.0, 66.1, 66.3, 66.5, 67.5]:
    r = R_START + (R_END - R_START) * max(0.0, min(1.0, (t - G0) / GROW_S))
    at = grab(t)
    print(f"{t:.3f}\t{r:.3f}\t" + "\t".join(f"{diff(at, ref, box(*SIZES[n])):.1f}" for n in names))

print("GROUND: pixels off the ground colour, in the band a backing adds round its label, and in the boxes of the moved names")
print("(tolerance " + str(GROUND_TOL) + " per channel; time, place, count of pixels in it, count off ground, worst)")
for t in (BEFORE_RING, 67.5):
    at = grab(t)
    ground = at[int(to_px(-0.9, 2.4)[0]), int(to_px(-0.9, 2.4)[1])]
    rows = []
    for n in ("sqrt10", "x=3"):
        re, im, w, h = SIZES[n]
        inner = set(box(re, im, w, h))
        rows.append((f"band of {n}", [p for p in box(re, im, w + PAD, h + PAD) if p not in inner]))
    rows += [(n, box(*MOVED[n])) for n in MOVED]
    for name, pixels in rows:
        off = [max(abs(a - b) for a, b in zip(at[p], ground)) for p in pixels]
        print(f"{t:.1f}\t{name}\t{len(pixels)}\t{sum(o > GROUND_TOL for o in off)}\t{max(off)}")

clean = grab(MARK_ALONE)
print(f"POLES: mean difference in each mark's disc against {MARK_ALONE} s, when the marks are full and the radius 1 ring has not started")
print("time_s\t" + "\t".join(POLES))
for t in (MARK_ALONE, 47.8, 48.1, 53.0, 60.0, 61.5, 62.5, 64.4, 66.1, 67.5, 70.0):
    at = grab(t)
    print(f"{t:.3f}\t" + "\t".join(f"{diff(at, clean, disc(*POLES[n])):.1f}" for n in POLES))
