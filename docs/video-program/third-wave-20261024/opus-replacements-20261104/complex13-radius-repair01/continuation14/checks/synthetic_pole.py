"""SYNTHETIC look at the repair: the clean mark's pixels pasted over old frames. Not a render.

For each pose it shows the old frame and, below it, the same frame with the disc of the mark replaced
by that disc from REF (a frame where one mark is on top). This is what the repair should draw inside
the disc, since the backing is opaque, but the disc's edge is this script's feathering and not the
engine's. Everything outside the disc is the old frame. It proves nothing about the native picture.

Usage: python3 -B synthetic_pole.py FRAMES_DIR REF_FRAME OUT.png     (PIL needs site-packages, so not -I)
"""

import json
import math
import sys
from pathlib import Path

from PIL import Image, ImageDraw

FRAMES, REF, OUT = Path(sys.argv[1]), int(sys.argv[2]), Path(sys.argv[3])
times = json.loads((FRAMES / "sampling.json").read_text(encoding="utf8"))["times"]
ZOOMS = [116, 134, 146, 148, 150, 151, 155, 161, 163]
WHOLE = [146, 155, 163]
CX, CY, BACK_PX, ZOOM_R, Z, SS = 397.0, 378.0, 0.13 * 90, 26, 5, 4
WHOLE_BOX = (60, 110, 660, 660)


def load(n):
    return Image.open(FRAMES / f"frame-{n:04}.png").convert("RGB")


ref = load(REF)
size = int(2 * (BACK_PX + 3))
x0, y0 = int(CX - size / 2), int(CY - size / 2)
disc = ref.crop((x0, y0, x0 + size, y0 + size))
mask = Image.new("L", (size * SS, size * SS), 0)
ImageDraw.Draw(mask).ellipse(
    [((CX - x0) - BACK_PX) * SS, ((CY - y0) - BACK_PX) * SS, ((CX - x0) + BACK_PX) * SS, ((CY - y0) + BACK_PX) * SS],
    fill=255,
)
mask = mask.resize((size, size), Image.LANCZOS)


def repaired(n):
    im = load(n)
    im.paste(disc, (x0, y0), mask)
    return im


def zoom(im):
    box = (int(CX) - ZOOM_R, int(CY) - ZOOM_R, int(CX) + ZOOM_R, int(CY) + ZOOM_R)
    return im.crop(box).resize((2 * ZOOM_R * Z, 2 * ZOOM_R * Z), Image.NEAREST)


cell = 2 * ZOOM_R * Z + 4
wide_w, wide_h = (WHOLE_BOX[2] - WHOLE_BOX[0]), (WHOLE_BOX[3] - WHOLE_BOX[1])
width = max(len(ZOOMS) * cell, len(WHOLE) * (wide_w + 4))
sheet = Image.new("RGB", (width, 24 + 2 * cell + 2 * (wide_h + 4)), (45, 25, 45))
draw = ImageDraw.Draw(sheet)
draw.text((6, 6), f"SYNTHETIC: the mark's disc from frame {REF} pasted on old frames, lower rows. Not a render, not native proof.", fill=(255, 255, 0))
for i, n in enumerate(ZOOMS):
    for row, im in enumerate((load(n), repaired(n))):
        z = zoom(im)
        ImageDraw.Draw(z).text((4, 3), f"{'old' if row == 0 else 'pasted'} {n}: {times[n]:.1f}s", fill=(255, 255, 0))
        sheet.paste(z, (i * cell, 24 + row * cell))
for i, n in enumerate(WHOLE):
    for row, im in enumerate((load(n), repaired(n))):
        w = im.crop(WHOLE_BOX)
        ImageDraw.Draw(w).text((4, 3), f"{'old' if row == 0 else 'pasted'} {n}: {times[n]:.1f}s", fill=(255, 255, 0))
        sheet.paste(w, (i * (wide_w + 4), 24 + 2 * cell + row * (wide_h + 4)))
sheet.save(OUT)
print(OUT.name, sheet.size)
