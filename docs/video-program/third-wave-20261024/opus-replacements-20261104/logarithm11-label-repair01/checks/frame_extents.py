"""Measure the drawn extent of the lines this patch changes, on the frames of the rendered baseline.

python3 -B frame_extents.py FRAMES_DIR   (PIL needs site packages, so no -I)

Scene x = px / W * 14.22 - 7.11 and scene y = (H / 2 - py) / H * 8. A row band that holds one line of text is scanned for
pixels that differ from the panel background. The estimate from the chapter's width table is printed beside each, so
the table's margin on these lines is measured rather than assumed. Nothing is written.
"""
import sys
from pathlib import Path

from PIL import Image

FW = 14.22
FH = 8.0

# (what, frame file, burned-in label, time, scene y centre, half band height, min scene x, max scene x, estimate from em())
LINES = [
    ("teal positive line", "frame-0400.png", 400, 183.5, -0.05, 0.13, 2.0, 7.11, 4.175),
    ("teal axes line", "frame-0400.png", 400, 183.5, 0.40, 0.14, 2.0, 7.11, 3.595),
    ("purple stage line", "frame-0400.png", 400, 183.5, -0.85, 0.12, 2.0, 7.11, 4.268),
    ("closing singular line", "frame-0598.png", 598, 273.783, -1.85, 0.20, -7.11, 7.11, 4.247),
    ("closing like_zero line", "frame-0598.png", 598, 273.783, -2.5, 0.12, -7.11, 7.11, 4.579),
]


def main():
    top = Path(sys.argv[1])
    for what, name, label, t, y, half, x0, x1, est in LINES:
        image = Image.open(top / name).convert("RGB")
        w, h = image.size
        px = image.load()
        back = px[w - 3, h - 3]
        rows = range(int((FH / 2 - (y + half)) / FH * h), int((FH / 2 - (y - half)) / FH * h) + 1)
        cols = range(int((x0 + FW / 2) / FW * w), min(w, int((x1 + FW / 2) / FW * w) + 1))
        hit = [
            c for c in cols
            if any(sum(abs(px[c, r][k] - back[k]) for k in range(3)) > 60 for r in rows)
        ]
        left, right = hit[0] / w * FW - FW / 2, (hit[-1] + 1) / w * FW - FW / 2
        drawn = right - left
        print(f"{what:<24} {name} label {label} t {t:<8} {w}x{h}: x {left:6.3f} .. {right:6.3f}, drawn {drawn:5.3f}, "
              f"estimate {est:5.3f}, estimate/drawn {est / drawn:5.3f}")


if __name__ == "__main__":
    main()
