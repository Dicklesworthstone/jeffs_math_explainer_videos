"""Does the sweep check's model match real pixels? Read frames02, compare with the canonical formula.

These are the frames root inspected: preview02, SCRATCH voice, canonical source, 720p, one picture
per 0.5 s. They show the old picture only and say nothing about the repair. For each frame in the
walk, it measures the walker's angle on the half circle, then finds the moving "R" where the
formula puts it and reports the error. It also measures the name and ring half sizes that
sweep_check.py and the Rust tests use.

Usage: python3 -B frame_calibration.py FRAMES_DIR     (PIL needs site-packages, so not -I)
"""

import math
import sys
from pathlib import Path

from PIL import Image

FRAMES = Path(sys.argv[1])
PX = 90.0  # 1280 px over 14.2222 scene units
UNIT = 5.0 / 4.8
OX, OY = 640 + (-3.0) * PX, 360 - (-1.15 + (0 - 0.5) * UNIT) * PX


def px(re, im):
    return OX + re * UNIT * PX, OY - im * UNIT * PX


def plane(x, y):
    return (x - OX) / (UNIT * PX), (OY - y) / (UNIT * PX)


def bbox(im, box, test):
    hits = [(x, y) for y in range(box[1], box[3]) for x in range(box[0], box[2]) if test(im.getpixel((x, y)))]
    if not hits:
        return None
    return min(h[0] for h in hits), min(h[1] for h in hits), max(h[0] for h in hits) + 1, max(h[1] for h in hits) + 1


def open_frame(n):
    return Image.open(FRAMES / f"frame-{n:04}.png").convert("RGB")


def half_sizes(name, n, box, test):
    b = bbox(open_frame(n), box, test)
    x0, y1 = plane(b[0], b[1])
    x1, y0 = plane(b[2], b[3])
    print(f"  {name:18} frame {n}: {b[2] - b[0]} x {b[3] - b[1]} px; centre {(x0 + x1) / 2:+.3f}, {(y0 + y1) / 2:+.3f}; half {(x1 - x0) / 2:.3f} x {(y1 - y0) / 2:.3f}")


def pink(c):
    return c[0] > 130 and c[1] < 110 and c[2] > 90


def dim_pink(c):
    return c[0] > 110 and c[1] < 100 and c[2] > 60


def white(c):
    return min(c) > 150


print("half sizes, plane units (one px = 0.011)")
ix, iy = px(0, 1)
half_sizes("i", 88, (int(ix) - 45, int(iy) - 20, int(ix) - 14, int(iy) + 20), pink)
half_sizes("ring", 88, (int(ix) - 13, int(iy) - 18, int(ix) + 14, int(iy) + 18), pink)
half_sizes("R at the walk's end", 100, (int(px(-1.44, 0)[0]) - 22, int(px(0, 0.25)[1]) - 20, int(px(-1.44, 0)[0]) + 22, int(px(0, 0.25)[1]) + 15), white)
half_sizes("i at R = 10", 130, (int(px(-0.6, 0)[0]), int(px(0, 0.45)[1]), int(px(-0.14, 0)[0]), int(px(0, 0.03)[1])), pink)
half_sizes("-i at R = 10", 130, (int(px(0.14, 0)[0]) + 4, int(px(0, -0.03)[1]), int(px(0.8, 0)[0]), int(px(0, -0.5)[1])), dim_pink)

print("the walk: measured walker angle and R centre against r_name_z(2, th)")
print("  frame   time  walker   R measured        R formula       error")
worst = 0.0
angles = []
for n in range(90, 96):
    im = open_frame(n)
    rr = 2 * UNIT * PX
    on = [a / 10 for a in range(0, 1801) if min(im.getpixel((round(OX + rr * math.cos(math.radians(a / 10))), round(OY - rr * math.sin(math.radians(a / 10)))))) > 215]
    th = (min(on) + max(on)) / 2
    angles.append((n * 0.5 - 7.5, th))
    t = math.radians(th)
    want = (1.44 * math.cos(t) + 0.25 * math.sin(t), 1.44 * math.sin(t) - 0.25 * math.cos(t))
    cx, cy = px(*want)
    b = bbox(im, (int(cx) - 11, int(cy) - 12, int(cx) + 12, int(cy) + 12), white)
    got = plane((b[0] + b[2]) / 2, (b[1] + b[3]) / 2)
    err = math.hypot(got[0] - want[0], got[1] - want[1])
    worst = max(worst, err)
    print(f"  {n:5} {n * 0.5 - 7.5:6.1f} {th:7.2f}  {got[0]:+.3f}, {got[1]:+.3f}   {want[0]:+.3f}, {want[1]:+.3f}   {err:.3f}")
rate = (angles[-1][1] - angles[0][1]) / (angles[-1][0] - angles[0][0])
start = angles[0][0] - angles[0][1] / rate
print(f"worst R error {worst:.3f} plane units ({worst * UNIT * PX:.1f} px)")
print(f"walker rate {rate:.1f} deg/s; walker at 0 deg at {start:.2f} s, at 180 deg at {start + 180 / rate:.2f} s (frame time, scratch voice)")
print("no frame falls inside the ring band 83.3 to 96.7 deg:", all(not 83.3 <= a <= 96.7 for _, a in angles))
print("no frame falls inside the letter band 103.9 to 107.3 deg:", all(not 103.9 <= a <= 107.3 for _, a in angles))
print("no frame falls inside the segment band 5.6 to 14.1 deg:", all(not 5.6 <= a <= 14.1 for _, a in angles))
print("frames inside the R on axis band 96.4 to 103.3 deg:", [a for _, a in angles if 96.4 <= a <= 103.3])
