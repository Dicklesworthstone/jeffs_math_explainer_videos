"""Paint the label move onto copies of preview02 frames, as a desk check and a pass control.

Run from the repo root:
  python3 -I -B docs/video-program/third-wave-20261024/opus-replacements-20261104/complex11-counting-review-repair01/checks/desk_composites.py

These are NOT renders. They move the real label's pixels from frame 535 to the place computed
from the staged source, clear the old place, and redraw the part of the disk's circle that passed
through it with PIL, fitted to that frame's own circle. Each carries a stamp saying so. They show
where the label would land and let anchor_probe.py prove it can pass. They say nothing about how
franken_manim draws. Requires Pillow. Writes only under checks/desk-composites/.
"""
import math
import os

from PIL import Image, ImageDraw

HERE = "docs/video-program/third-wave-20261024/opus-replacements-20261104/complex11-counting-review-repair01/checks/desk-composites/"
FRAMES = "narration_lab/third-wave-20261024/root/five-film-release-20261027/complex-combined-picture-frames02/11_counting/"
BG = (12, 17, 20)
TEAL = (98, 204, 182)
OLD = (415, 376, 526, 396)
# staged place out.p(0, 2.2) is native (410.5, 171.9); the old out.p(0.95, -1.2) is (470.35, 386.1)
SHIFT = (-60, -214)
WINDOW = (288, 188, 534, 434)
RADIUS, HALF_LINE = 49.5, 1.6


def frame(n):
    return Image.open(f"{FRAMES}frame-{n:04d}.png").convert("RGB")


def teal(p):
    return p[1] > p[0] + 40 and p[2] > p[0] + 30


def ring_centre(im):
    """Least squares for the disk's circle on its teal pixels, leaving out the old label's box."""
    px = im.load()
    points = [
        (x, y)
        for y in range(WINDOW[1], WINDOW[3])
        for x in range(WINDOW[0], WINDOW[2])
        if teal(px[x, y]) and not (OLD[0] <= x < OLD[2] and OLD[1] <= y < OLD[3])
    ]
    n = len(points)
    sx, sy = sum(p[0] for p in points) / n, sum(p[1] for p in points) / n
    suu = sum((p[0] - sx) ** 2 for p in points)
    svv = sum((p[1] - sy) ** 2 for p in points)
    suv = sum((p[0] - sx) * (p[1] - sy) for p in points)
    b1 = sum(((p[0] - sx) ** 3 + (p[0] - sx) * (p[1] - sy) ** 2) for p in points) / 2
    b2 = sum(((p[1] - sy) ** 3 + (p[1] - sy) * (p[0] - sx) ** 2) for p in points) / 2
    det = suu * svv - suv * suv
    return sx + (b1 * svv - b2 * suv) / det, sy + (b2 * suu - b1 * suv) / det


def compose(n, label):
    im = frame(n)
    cx, cy = ring_centre(im)
    im.paste(BG, OLD)
    px = im.load()
    for y in range(OLD[1], OLD[3]):
        for x in range(OLD[0], OLD[2]):
            alpha = max(0.0, min(1.0, HALF_LINE + 0.5 - abs(math.hypot(x - cx, y - cy) - RADIUS)))
            if alpha > 0:
                px[x, y] = tuple(round(BG[i] + (TEAL[i] - BG[i]) * alpha) for i in range(3))
    im.paste(label, (OLD[0] + SHIFT[0], OLD[1] + SHIFT[1]))
    ImageDraw.Draw(im).text((8, 702), f"DESK COMPOSITE of preview02 f{n}, NOT A RENDER", fill=(255, 120, 120))
    im.save(f"{HERE}reach-from-frame-{n:04d}.png")
    return round(cx, 1), round(cy, 1)


def main():
    os.makedirs(HERE, exist_ok=True)
    label = frame(535).crop(OLD)
    for n in (535, 544, 547, 549, 550, 551):
        print(n, "disk centre", compose(n, label))


main()
