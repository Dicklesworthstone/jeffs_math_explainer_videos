"""Measure stroke thickness on the probe pages, so "heavier" has a number.

python3 -B stroke_widths.py   (from the repo root; PIL needs site packages)

Each entry names one symbol by its registry row and a short scan line crossing it. A run is a
stretch of pixels on that line brighter than halfway between the page background and the line's
own peak, so a run's length is that stroke's thickness to within a pixel.
"""
from pathlib import Path

from PIL import Image

PAGES = Path("narration_lab/third-wave-20261024/root/five-film-release-20261027/complex-combined-picture-glyphs02/glyphs/pages")

# label, page, axis, fixed coordinate, start, end. "v" scans y at fixed x; "h" scans x at fixed y.
PROBES = [
    ("row 549 TeX, plus left arm (CM Roman)", 45, "v", 190, 680, 720),
    ("row 549 TeX, minus (Noto Sans Math)", 45, "v", 100, 680, 720),
    ("row 567 TeX, equals (CM Roman)", 47, "v", 540, 545, 580),
    ("row 567 TeX, fraction rule", 47, "v", 437, 555, 575),
    ("row 567 TeX, big left bar (drawn)", 47, "h", 560, 50, 65),
    ("row 567 TeX, integral stem (Noto Sans Math)", 47, "h", 560, 90, 135),
    ("row 503 Text, plus left arm (CM Roman)", 41, "v", 215, 820, 855),
    ("row 503 Text, minus (Noto Sans Math)", 41, "v", 118, 820, 855),
    ("row 508 Text, equals (CM Roman)", 42, "v", 528, 365, 400),
    ("row 508 Text, times, centre line (Noto Sans Math)", 42, "h", 384, 125, 155),
]


def runs(im, axis, fixed, start, end):
    px = im.load()
    w, h = im.size
    bg = px[w - 2, h - 2]
    line = [px[fixed, y] if axis == "v" else px[y, fixed] for y in range(start, end)]
    peak = max(line)
    cut = bg + 0.5 * (peak - bg)
    found, count = [], 0
    for v in line:
        if v >= cut:
            count += 1
        elif count:
            found.append(count)
            count = 0
    if count:
        found.append(count)
    return found, peak


def main():
    for label, page, axis, fixed, start, end in PROBES:
        im = Image.open(PAGES / f"frame_{page:06d}.png").convert("L")
        found, peak = runs(im, axis, fixed, start, end)
        print(f"{label}: runs {found} px, peak {peak} (page {page}, {axis} line at {fixed}, span {start}-{end})")


if __name__ == "__main__":
    main()
