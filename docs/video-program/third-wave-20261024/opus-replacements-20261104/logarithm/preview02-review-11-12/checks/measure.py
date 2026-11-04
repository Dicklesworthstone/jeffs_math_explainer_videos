"""Read-only measurements on logarithm frames02 chapters 11 and 12. Run from the repo root with
python3 -I -B. Boxes are frame pixels (x0, y0, x1, y1). Scene: x = (px - 640) / 90, y = (360 - py) / 90."""
import json
from pathlib import Path

import numpy as np
from PIL import Image

F = Path("narration_lab/third-wave-20261024/root/five-film-release-20261027/logarithm-opus-rebuild-frames02")
CH = {"11": F / "11_unreachable", "12": F / "12_what_it_forgets"}
TIMES = {k: json.load(open(d / "sampling.json"))["times"] for k, d in CH.items()}


def load(ch, i):
    return np.asarray(Image.open(CH[ch] / f"frame-{i:04d}.png").convert("RGB")).astype(int)


def lin(c):
    c = c / 255.0
    return c / 12.92 if c <= 0.03928 else ((c + 0.055) / 1.055) ** 2.4


def lum(rgb):
    return 0.2126 * lin(rgb[0]) + 0.7152 * lin(rgb[1]) + 0.0722 * lin(rgb[2])


def ratio(a, b):
    la, lb = lum(a), lum(b)
    return (max(la, lb) + 0.05) / (min(la, lb) + 0.05)


def contrast(ch, i, box, label):
    a = load(ch, i)[box[1] : box[3], box[0] : box[2]].reshape(-1, 3)
    values, counts = np.unique(a, axis=0, return_counts=True)
    bg = values[counts.argmax()]
    dist = np.abs(a - bg).sum(axis=1)
    cut = np.percentile(dist, 98.5)
    fg = a[dist >= cut].mean(axis=0)
    print(f"  {label}: ch{ch} f{i} t={TIMES[ch][i]:.3f} box {box} bg {bg.tolist()} fg {[round(float(x)) for x in fg]} ratio {ratio(fg, bg):.2f}")


def bbox(ch, i, box, label, thresh=60, bg=(12, 17, 20)):
    a = load(ch, i)
    sub = a[box[1] : box[3], box[0] : box[2]]
    mask = np.abs(sub - np.array(bg)).sum(axis=2) > thresh
    if not mask.any():
        print(f"  {label}: ch{ch} f{i} nothing above {thresh}")
        return None
    ys, xs = np.nonzero(mask)
    x0, x1, y0, y1 = xs.min() + box[0], xs.max() + box[0], ys.min() + box[1], ys.max() + box[1]
    print(
        f"  {label}: ch{ch} f{i} t={TIMES[ch][i]:.3f} px x {x0}..{x1} y {y0}..{y1}; scene x {(x0 - 640) / 90:.3f}..{(x1 - 640) / 90:.3f} y {(360 - y1) / 90:.3f}..{(360 - y0) / 90:.3f}"
    )
    return x0, y0, x1, y1


def crop(ch, i, box, name, scale=3):
    im = Image.open(CH[ch] / f"frame-{i:04d}.png").convert("RGB").crop(box)
    im = im.resize((im.width * scale, im.height * scale), Image.NEAREST)
    OUT.mkdir(parents=True, exist_ok=True)
    im.save(OUT / name)
    print(f"  crop {name}: ch{ch} f{i} t={TIMES[ch][i]:.3f} box {box} x{scale}")


def spans(ch, floor):
    n = len(TIMES[ch])
    prev = load(ch, 0)
    start = 0
    out = []
    for i in range(1, n):
        cur = load(ch, i)
        moved = int((np.abs(cur - prev).sum(axis=2) > 36).sum())
        if moved >= 30:
            if TIMES[ch][i - 1] - TIMES[ch][start] >= floor:
                out.append((start, i - 1))
            start = i
        prev = cur
    if TIMES[ch][n - 1] - TIMES[ch][start] >= floor:
        out.append((start, n - 1))
    return out


def nearest(ch, t):
    a = np.array(TIMES[ch])
    i = int(np.abs(a - t).argmin())
    return i, TIMES[ch][i], i // 4


OUT = Path("docs/video-program/third-wave-20261024/opus-replacements-20261104/logarithm/preview02-review-11-12/crops")
