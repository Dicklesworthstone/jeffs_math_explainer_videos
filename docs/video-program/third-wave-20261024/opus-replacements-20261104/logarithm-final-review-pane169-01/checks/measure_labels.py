#!/usr/bin/env python3
"""Measure small label contrast on the verification PNGs. Read-only.

Each row names a verification frame, a label and a crop in 960x540 pixels. The
background is the median luminance of the crop. The ink is the 99.5th percentile,
so it is a conservative stand-in for the drawn colour: thin strokes are scaled
down in these PNGs and never reach it. Reported ratio is WCAG (L1+0.05)/(L2+0.05).

Run from the repo root:
  python3 -I -B docs/video-program/third-wave-20261024/opus-replacements-20261104/logarithm-final-review-pane169-01/checks/measure_labels.py
"""

from pathlib import Path

from PIL import Image

FRAMES = Path(
    "narration_lab/third-wave-20261024/root/five-film-release-20261027/"
    "logarithm-stage09-final-verify02"
)

ROWS = [
    ("frame-1168.png", "purple: a half turn, then a stretch", (628, 318, 925, 334)),
    ("frame-1199.png", "purple: only the two axes go straight back", (402, 269, 607, 283)),
    ("frame-1295.png", "purple: spin: 1/2 lap per unit of time", (520, 107, 705, 121)),
    ("frame-1295.png", "purple: laps value 0.50", (732, 139, 757, 152)),
    ("frame-1356.png", "purple: laps: 1/2, 1 1/2, ...", (520, 175, 732, 189)),
    ("frame-1125.png", "grey: (here X is the sample flow)", (628, 285, 840, 300)),
    ("frame-1199.png", "grey: dashes: straight back", (402, 195, 515, 208)),
    ("frame-1221.png", "grey: each row is one matrix", (127, 113, 460, 128)),
    ("frame-1289.png", "grey: each corner keeps to a circle", (520, 107, 775, 121)),
    ("frame-1295.png", "grey: time", (520, 139, 548, 152)),
    ("frame-1356.png", "control gold: so: infinitely many", (520, 275, 826, 292)),
    ("frame-1356.png", "control white: precisely footer", (54, 491, 907, 506)),
]


def channel(value: int) -> float:
    c = value / 255.0
    return c / 12.92 if c <= 0.03928 else ((c + 0.055) / 1.055) ** 2.4


def luminance(pixel: tuple[int, int, int]) -> float:
    r, g, b = pixel
    return 0.2126 * channel(r) + 0.7152 * channel(g) + 0.0722 * channel(b)


def measure(name: str, box: tuple[int, int, int, int]) -> tuple[float, tuple, tuple]:
    with Image.open(FRAMES / name) as image:
        crop = image.convert("RGB").crop(box)
    pixels = sorted(crop.getdata(), key=luminance)
    back = pixels[len(pixels) // 2]
    ink = pixels[min(len(pixels) - 1, int(len(pixels) * 0.995))]
    low, high = sorted((luminance(back), luminance(ink)))
    return (high + 0.05) / (low + 0.05), ink, back


def main() -> None:
    print("frame\tratio\tink\tback\tlabel")
    for name, label, box in ROWS:
        ratio, ink, back = measure(name, box)
        print(f"{name}\t{ratio:.2f}\t{ink}\t{back}\t{label}")


if __name__ == "__main__":
    main()
