"""SYNTHETIC look at the repair: old preview frames with pixels pasted where the repair should change them. Not a render.

Each pose appears twice: the old frame, and below it the same frame with
  - each label's backing rectangle replaced by that rectangle from the frame just before the circle starts,
  - the old pole names' boxes painted to ground except the ring's gold, and the names' pink pixels drawn at the moved place,
  - each pole's backing disc replaced by that disc from the moment the marks are alone.
The edges of the pasted shapes are this script's, the names' new places are rounded to whole pixels, and the
old frames are 720p with the scratch voice's times. It is for judging the look, chiefly the bite the backing
takes from the ring as it passes. It proves nothing about the native picture.

Usage: python3 -B synthetic_sweep.py PREVIEW_MP4 WORK_DIR OUT.png     (PIL needs site-packages, so not -I)
"""

import subprocess
import sys
from pathlib import Path

from PIL import Image, ImageDraw

MP4, WORK, OUT = sys.argv[1], Path(sys.argv[2]), Path(sys.argv[3])
WORK.mkdir(parents=True, exist_ok=True)
PX = 0.95 * 90
BEFORE_RING, MARK_ALONE = 64.66, 47.63
POSES = [62.5, 64.88, 65.2, 65.4, 65.55, 65.87, 65.92, 66.1, 67.5, 70.0]
BOX = (170, 160, 690, 550)
Z, ZOOM_R = 6, 22
BACK_R, PAD = 0.13 * 90, 0.05 / 0.95
LABELS = [(1.6743, 1.0217, 0.634, 0.113), (3.0, -0.45, 0.317, 0.099)]
NAMES = [(0.42, 1.28, 0.06, 0.13, -0.84), (0.5, -1.28, 0.14, 0.13, -1.0)]
POLES = [(0.0, 1.0), (0.0, -1.0)]
GROUND = (12, 17, 20)


def to_px(re, im):
    return 640 + (-1.6 + (re - 2.8) * 0.95) * 90, 360 + (0.2 - im * 0.95) * 90


def frame(t):
    p = WORK / f"t{t:07.3f}.png"
    if not p.exists():
        subprocess.run(["ffmpeg", "-v", "error", "-ss", f"{t:.3f}", "-i", MP4, "-frames:v", "1", "-s", "1280x720", str(p)], check=True)
    return Image.open(p).convert("RGB")


def pink(c):
    return c[0] > 45 and c[0] > 1.35 * c[1] and c[2] > 0.6 * c[1]


def gold(c):
    return c[0] > 120 and c[1] > 0.7 * c[0] and c[2] < 0.75 * c[0]


before, alone = frame(BEFORE_RING), frame(MARK_ALONE)


def repaired(t):
    im = frame(t).copy()
    px = im.load()
    if t >= BEFORE_RING:
        for re, zim, w, h in LABELS:
            x0, y0 = to_px(re - w - PAD, zim + h + PAD)
            x1, y1 = to_px(re + w + PAD, zim - h - PAD)
            box = (int(x0), int(y0), int(x1) + 1, int(y1) + 1)
            im.paste(before.crop(box), box[:2])
    source = before.load()
    for re, zim, w, h, dx in NAMES:
        x0, y0 = to_px(re - w, zim + h)
        x1, y1 = to_px(re + w, zim - h)
        shift = round(dx * PX)
        for y in range(int(y0), int(y1) + 1):
            for x in range(int(x0), int(x1) + 1):
                if not gold(px[x, y]):
                    px[x, y] = GROUND
        for y in range(int(y0), int(y1) + 1):
            for x in range(int(x0), int(x1) + 1):
                if pink(source[x, y]):
                    px[x + shift, y] = source[x, y]
    for re, zim in POLES:
        cx, cy = to_px(re, zim)
        size = int(2 * (BACK_R + 3))
        x0, y0 = int(cx - size / 2), int(cy - size / 2)
        mask = Image.new("L", (size * 4, size * 4), 0)
        ImageDraw.Draw(mask).ellipse([((cx - x0) - BACK_R) * 4, ((cy - y0) - BACK_R) * 4, ((cx - x0) + BACK_R) * 4, ((cy - y0) + BACK_R) * 4], fill=255)
        im.paste(alone.crop((x0, y0, x0 + size, y0 + size)), (x0, y0), mask.resize((size, size), Image.LANCZOS))
    return im


w, h = BOX[2] - BOX[0], BOX[3] - BOX[1]
cols = 5
rows = (len(POSES) + cols - 1) // cols
zoom_cell = 2 * ZOOM_R * Z + 4
sheet = Image.new("RGB", (cols * (w + 4), 24 + 2 * rows * (h + 4) + 2 * zoom_cell), (45, 25, 45))
draw = ImageDraw.Draw(sheet)
draw.text((6, 6), "SYNTHETIC: pixels pasted on old frames, the 'pasted' rows. Not a render, not native proof. Times on the scratch voice.", fill=(255, 255, 0))
for i, t in enumerate(POSES):
    for row, im in enumerate((frame(t), repaired(t))):
        c = im.crop(BOX)
        ImageDraw.Draw(c).text((4, 3), f"{'old' if row == 0 else 'pasted'} {t:.2f}s", fill=(255, 255, 0))
        sheet.paste(c, ((i % cols) * (w + 4), 24 + (2 * (i // cols) + row) * (h + 4)))
for i, t in enumerate((62.5, 66.1, 67.5)):
    for j, (re, zim) in enumerate(POLES):
        cx, cy = to_px(re, zim)
        for row, im in enumerate((frame(t), repaired(t))):
            c = im.crop((int(cx) - ZOOM_R, int(cy) - ZOOM_R, int(cx) + ZOOM_R, int(cy) + ZOOM_R)).resize((2 * ZOOM_R * Z, 2 * ZOOM_R * Z), Image.NEAREST)
            ImageDraw.Draw(c).text((4, 3), f"{'old' if row == 0 else 'pasted'} {t:.1f}s", fill=(255, 255, 0))
            sheet.paste(c, ((2 * i + row) * zoom_cell, 24 + 2 * rows * (h + 4) + j * zoom_cell))
sheet.save(OUT)
print(OUT.name, sheet.size)
