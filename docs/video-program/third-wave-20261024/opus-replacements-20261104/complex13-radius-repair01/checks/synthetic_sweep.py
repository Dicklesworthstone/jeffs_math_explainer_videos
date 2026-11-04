"""SYNTHETIC before-and-after of the walk's six worst poses. Not a render of either source.

Lines, rings and backings are drawn here from the geometry and z-order, at 720p x 4 and then scaled
down; only the two letters come from real frames. Its one use is judging the look of the backings
before a render exists. Line weights and antialiasing are this script's, not the engine's. It
proves nothing about the native picture.

Usage: python3 -B synthetic_sweep.py FRAMES_DIR OUT.png     (PIL needs site-packages, so not -I)
"""

import math
import sys
from pathlib import Path

from PIL import Image, ImageDraw

FRAMES, OUT = Path(sys.argv[1]), Path(sys.argv[2])
S = 4
PX = 90.0 * S
UNIT = 5.0 / 4.8
BG, GOLD, PINK, INK, DIM = (11, 16, 20), (255, 214, 102), (236, 72, 153), (235, 238, 242), (120, 125, 135)
W, H = 420 * S, 330 * S
OX, OY = W / 2, H - 100 * S
POSES = [(10, "right"), (20, "right"), (90, "pole"), (98.75, "pole"), (101, "pole"), (105.6, "pole")]
CROPS = {"right": ((0.7, 0.8), (2.2, -0.5)), "pole": ((-0.75, 1.9), (0.75, 0.6))}


def P(re, im):
    return OX + re * UNIT * PX, OY - im * UNIT * PX


def letter(frame, box):
    im = Image.open(FRAMES / f"frame-{frame:04}.png").convert("RGB").crop(box)
    im = im.resize((im.width * S, im.height * S), Image.LANCZOS)
    return im, im.split()[0].point(lambda v: max(0, min(255, int((v - 25) * 255 / 170))))


R_GLYPH = letter(100, (226, 477, 244, 497))
I_GLYPH = letter(88, (339, 407, 349, 427))


def scene(th, repaired):
    im = Image.new("RGB", (W, H), BG)
    d = ImageDraw.Draw(im)
    t = math.radians(th)
    tip = (2 * math.cos(t), 2 * math.sin(t))

    def line(a, b, width, color):
        d.line([P(*a), P(*b)], fill=color, width=max(1, round(width * 0.01 * PX)))

    def disk(c, r, color):
        x, y = P(*c)
        d.ellipse((x - r * PX, y - r * PX, x + r * PX, y + r * PX), fill=color)

    def stamp(glyph, c, alpha=1.0):
        x, y = P(*c)
        mask = glyph[1] if alpha >= 1 else glyph[1].point(lambda v: int(v * alpha))
        im.paste(glyph[0], (round(x - glyph[0].width / 2), round(y - glyph[0].height / 2)), mask)

    def arrow():
        d.polygon([P(0.55, 0.145), P(0.8, 0), P(0.55, -0.145)], fill=GOLD)
        line((0.2, 0), (0.55, 0), 6, GOLD)

    def pole_and_name():
        if repaired:
            disk((-0.28, 1), 0.1, BG)
        disk((0, 1), 0.13, BG)
        x, y = P(0, 1)
        r, w = 0.085 * PX, round(0.05 * PX)
        d.ellipse((x - r - w / 2, y - r - w / 2, x + r + w / 2, y + r + w / 2), outline=PINK, width=w)
        stamp(I_GLYPH, (-0.28, 1))

    line((-2.4, 0), (2.4, 0), 2, DIM)
    line((0, -1.25), (0, 2.25), 2, DIM)
    d.line([P(2 * math.cos(math.pi * k / 120), 2 * math.sin(math.pi * k / 120)) for k in range(121)], fill=GOLD, width=round(0.05 * PX))
    line((-2, 0), (2, 0), 5, GOLD)
    if repaired:
        line((0, 0), tip, 2.5, INK)
        arrow()
        pole_and_name()
    else:
        arrow()
        pole_and_name()
        line((0, 0), tip, 2.5, INK)
    disk(tip, 0.08, INK)
    z = (1.44 * math.cos(t) + 0.25 * math.sin(t), 1.44 * math.sin(t) - 0.25 * math.cos(t))
    if not repaired:
        stamp(R_GLYPH, z)
    else:
        shown = max(0.0, min(1.0, (t - 0.28) / 0.14))
        if shown >= 1:
            disk(z, 0.135, BG)
        if shown > 0:
            stamp(R_GLYPH, z, shown)
    return im


cells = []
for repaired in (False, True):
    for th, where in POSES:
        (a, b) = CROPS[where]
        x0, y0 = P(*a)
        x1, y1 = P(*b)
        cell = scene(th, repaired).crop((int(x0), int(y0), int(x1), int(y1))).resize((280, 240), Image.LANCZOS)
        ImageDraw.Draw(cell).text((5, 3), f"{'repair' if repaired else 'canonical'} {th} deg", fill=(255, 255, 0))
        cells.append(cell)
sheet = Image.new("RGB", (6 * 284, 2 * 244 + 20), (45, 25, 45))
ImageDraw.Draw(sheet).text((6, 4), "SYNTHETIC: drawn from geometry and z-order, letters lifted from frames02. Not a render, not native proof.", fill=(255, 255, 0))
for i, cell in enumerate(cells):
    sheet.paste(cell, ((i % 6) * 284, 20 + (i // 6) * 244))
sheet.save(OUT)
print(OUT.name, sheet.size)
