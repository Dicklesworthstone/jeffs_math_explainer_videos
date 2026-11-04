"""Paint each fix onto a copy of a preview02 frame, as a desk check and a pass control.

Run from the repo root:
  python3 -I -B docs/video-program/third-wave-20261024/opus-replacements-20261104/complex06-09-repair03/checks/desk_composites.py

These are NOT renders. They move real pixels from preview02 (A06, A07, A08N1, A09) or draw with
PIL (A08N2), at positions computed from the staged source. They show where things would land
and let anchor_probe.py prove it can pass. They say nothing about how franken_manim draws.
Requires Pillow. Writes only under checks/desk-composites/.
"""
import math
import os

from PIL import Image, ImageDraw

HERE = "docs/video-program/third-wave-20261024/opus-replacements-20261104/complex06-09-repair03/checks/desk-composites/"
FRAMES = "narration_lab/third-wave-20261024/root/five-film-release-20261027/complex-combined-picture-frames02/"
BG = (12, 17, 20)


def frame(chapter, n):
    return Image.open(f"{FRAMES}{chapter}/frame-{n:04d}.png").convert("RGB")


def clear(im, box):
    im.paste(BG, box)


def a06():
    """f450: reflection's rates gone, doubling's rates and cross moved up to their new places."""
    im = frame("06_one_derivative", 450)
    rates = im.crop((930, 462, 1110, 491))
    clear(im, (930, 200, 1110, 232))
    clear(im, (930, 256, 1110, 284))
    clear(im, (930, 462, 1110, 491))
    clear(im, (985, 535, 1055, 600))
    # staged rates at scene y 1.3, so native y 243; on the frame they are centred at y 476.5
    im.paste(rates, (930, 462 - round(476.5 - 243)))
    return im


def a07():
    """f10: tail moved to scene x -0.42, the right plane's Re copied over the left one's, name recentred."""
    im = frame("07_one_point", 10)
    re_clean = im.crop((1178, 362, 1206, 384))
    name = im.crop((600, 312, 680, 345))
    clear(im, (566, 362, 602, 384))
    im.paste(re_clean, (567, 362))
    clear(im, (600, 312, 680, 345))
    im.paste(name, (608, 312))
    return im


def a08n1():
    """f92: 08_1's green dot and z_k name, copied from f20, where the same step is marked."""
    im, source = frame("08_loop_sums", 92), frame("08_loop_sums", 20)
    to, fro = im.load(), source.load()
    for y in range(285, 345):
        for x in range(345, 410):
            r, g, b = fro[x, y]
            if g > 110 and g > r + 25 and g > b + 25:
                to[x, y] = (r, g, b)
    return im


def a08n2():
    """f186: every circulation mark redrawn with PIL from the staged circulation_head."""
    im = frame("08_loop_sums", 186)
    scale = 8
    length, width, r = 0.16, 0.11, 0.26
    color = (86, 192, 220)
    back = 2.0 * math.asin(length / (2.0 * r))
    for row in range(4):
        for col in range(4):
            cx, cy = 640 + 90 * (-2.6 - 1.5 + col), 360 - 90 * (-0.1 - 1.5 + row)
            box = (round(cx - 36), round(cy - 36), round(cx + 36), round(cy + 36))
            tile = Image.new("RGB", (72 * scale, 72 * scale), BG)
            pen = ImageDraw.Draw(tile)

            def at(x, y):
                return ((36 + x * 90) * scale, (36 - y * 90) * scale)

            def point(a):
                return (r * math.cos(a), r * math.sin(a))

            arc = [at(*point(0.4 + 5.0 * j / 60)) for j in range(61)]
            pen.line(arc, fill=color, width=round(2.7 * scale), joint="curve")
            tip, base = point(5.4), point(5.4 - back)
            ux, uy = (tip[0] - base[0]) / length, (tip[1] - base[1]) / length
            half = width / 2
            pen.polygon([at(*tip), at(base[0] - uy * half, base[1] + ux * half), at(base[0] + uy * half, base[1] - ux * half)], fill=color)
            im.paste(tile.resize((72, 72), Image.LANCZOS), box[:2])
    return im


def a09():
    """f240: the a = 0 card from f200 moved to p.p(1.05, -0.4); the plate it hid comes from f360."""
    im = frame("09_boundary", 240)
    card = frame("09_boundary", 200).crop((384, 428, 472, 473))
    plate = frame("09_boundary", 360)
    for box in [(382, 426, 475, 455), (458, 455, 475, 476)]:
        im.paste(plate.crop(box), box[:2])
    # p.p(1.05, -0.4) is native (448.8, 418.3); the card on f200 is centred at (428, 450.5)
    im.paste(card, (384 + 21, 428 - 32))
    return im


os.makedirs(HERE, exist_ok=True)
for name, n, make in [("A06", 450, a06), ("A07", 10, a07), ("A08N1", 92, a08n1), ("A08N2", 186, a08n2), ("A09", 240, a09)]:
    make().save(f"{HERE}{name}-from-frame-{n:04d}.png")
    print("wrote", name, n)
