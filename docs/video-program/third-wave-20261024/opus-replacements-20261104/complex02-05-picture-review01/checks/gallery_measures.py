"""04-A stripes, 05-A comb and 05-F bars on the chapter 05 pictures of preview02, with controls.

    python3 -B docs/video-program/third-wave-20261024/opus-replacements-20261104/complex02-05-picture-review01/checks/gallery_measures.py
    (from the repo root; PIL needs a normal python, not -I)

Reads extracted frame PNGs only and prints to stdout. Same row measure as stripes02.py: average each row's luma over a
patch, remove a 9-row running mean, report rms, rms % and the strongest period.

Controls:
  - plain frame background beside the picture must read near 0.
  - sin and tan are drawn in the same plane geometry. Their row residuals are correlated unshifted, and shifted by one
    row as the negative control. Rows that stay dark across different functions are tied to the geometry.
  - preview01 frames are the positive control for the comb and the bars: its frame-0092 has the comb and its frame-0200
    has the bars, by the preview01 review. A measure that cannot see them there proves nothing here.

Inputs were decoded from H.264 at CRF 26, ultrafast, so a pattern can come from the picture or the encoder.
"""
import math
import pathlib

from PIL import Image

BASE = pathlib.Path("narration_lab/third-wave-20261024/root/five-film-release-20261027")
NOW = BASE / "complex-combined-picture-frames02/05_gallery"
THEN = BASE / "complex-opus-rebuild-frames01/05_gallery"


def luma(p):
    return 0.2126 * p[0] + 0.7152 * p[1] + 0.0722 * p[2]


def load(path):
    return Image.open(path).convert("RGB")


def rows_of(im, x0, x1, y0, y1):
    out = []
    for y in range(y0, y1):
        vals = [luma(im.getpixel((x, y))) for x in range(x0, x1)]
        out.append(sum(vals) / len(vals))
    return out


def cols_of(im, x0, x1, y0, y1):
    out = []
    for x in range(x0, x1):
        vals = [luma(im.getpixel((x, y))) for y in range(y0, y1)]
        out.append(sum(vals) / len(vals))
    return out


def residual(series):
    return [series[i] - sum(series[i - 4 : i + 5]) / 9 for i in range(4, len(series) - 4)]


def stats(series):
    resid = residual(series)
    mean = sum(series) / len(series)
    rms = math.sqrt(sum(r * r for r in resid) / len(resid))
    n = len(resid)
    best = max(
        (abs(sum(r * complex(math.cos(2 * math.pi * k * i / n), math.sin(2 * math.pi * k * i / n)) for i, r in enumerate(resid))), k)
        for k in range(2, n // 2)
    )
    return mean, rms, 100 * rms / mean, n / best[1]


def correlation(a, b):
    ma, mb = sum(a) / len(a), sum(b) / len(b)
    top = sum((x - ma) * (y - mb) for x, y in zip(a, b))
    bottom = math.sqrt(sum((x - ma) ** 2 for x in a) * sum((y - mb) ** 2 for y in b))
    return top / bottom


# name, frame, x0, x1, y0, y1. Patches keep clear of markers, cards, loops and titles on these frames.
PATCHES = [
    ("z^3 - 1, upper left", "frame-0008.png", 100, 220, 140, 260),
    ("e^z, upper left", "frame-0091.png", 270, 395, 110, 250),
    ("sin, top", "frame-0131.png", 60, 700, 185, 235),
    ("tan, top flat", "frame-0217.png", 60, 480, 185, 235),
    ("tan, bottom flat", "frame-0217.png", 60, 480, 505, 552),
    ("zeta, upper left", "frame-0321.png", 60, 300, 185, 300),
    ("zoom, lower left", "frame-0473.png", 660, 790, 470, 600),
    ("background (control)", "frame-0217.png", 860, 1240, 400, 600),
]

print("patch | frame | rows: mean luma, rms, rms %, period px | columns: rms %, period px")
for name, rel, x0, x1, y0, y1 in PATCHES:
    im = load(NOW / rel)
    r = stats(rows_of(im, x0, x1, y0, y1))
    c = stats(cols_of(im, x0, x1, y0, y1))
    print(f"{name} | {rel} | {r[0]:.1f}, {r[1]:.2f}, {r[2]:.2f}%, {r[3]:.2f} | {c[2]:.2f}%, {c[3]:.2f}")

print("\nsin against tan, same plane, same rows (unshifted, then shifted one row as the negative control)")
sin_im, tan_im = load(NOW / "frame-0131.png"), load(NOW / "frame-0217.png")
for label, x0, x1, y0, y1 in [("top", 60, 480, 185, 235), ("bottom", 60, 480, 505, 552)]:
    a = residual(rows_of(sin_im, x0, x1, y0, y1))
    b = residual(rows_of(tan_im, x0, x1, y0, y1))
    print(f"{label}: {correlation(a, b):+.3f}, shifted {correlation(a[:-1], b[1:]):+.3f}")

print("\n05-A comb on e^z. Plane x 262..550, 48 px a unit, so x = 0 is column 406.")
print("Measure: rows 110..650, centre column against the mean of its neighbours 4 px away; a comb flips sign from row to row")
for label, im in [("preview01 frame-0092 (positive control)", load(THEN / "frame-0092.png")), ("preview02 frame-0091", load(NOW / "frame-0091.png"))]:
    for column in (406, 440):
        diffs = [luma(im.getpixel((column, y))) - (luma(im.getpixel((column - 4, y))) + luma(im.getpixel((column + 4, y)))) / 2 for y in range(110, 650)]
        rough = sum(abs(p - q) for p, q in zip(diffs, diffs[1:])) / (len(diffs) - 1)
        print(f"{label}, column {column}: mean |row-to-row change| {rough:.2f}, mean |difference| {sum(abs(d) for d in diffs) / len(diffs):.2f}")
    profile = cols_of(im, 386, 428, 110, 650)
    print(f"{label}: column means 386..427:", " ".join(f"{v:.0f}" for v in profile))

print("\n05-F bars on tan. Plane x 46..802 cut into 16 bars of pi, the review's measure. Mean luma per bar, max/min.")
print("On preview02 frame-0217 the two cards start at x 530 and x 523, so its rows stop at bar 10, x 518.")
for label, im, rows, stop in [
    ("preview01 frame-0200 (positive control)", load(THEN / "frame-0200.png"), (200, 260), 16),
    ("preview02 frame-0217", load(NOW / "frame-0217.png"), (200, 260), 10),
    ("preview02 frame-0217, below the line", load(NOW / "frame-0217.png"), (530, 480), 10),
]:
    for yy in rows:
        means = []
        for b in range(stop):
            xs = range(int(46.0 + b * 47.25 + 8), int(46.0 + (b + 1) * 47.25 - 8))
            vals = [luma(im.getpixel((a, c))) for a in xs for c in range(yy - 8, yy + 9)]
            means.append(sum(vals) / len(vals))
        print(f"{label}, row {yy}: {[round(m) for m in means]}, max/min {max(means) / min(means):.2f}")

# Added after the sin/tan correlation came out weak (+0.14, +0.17): sin's own horizontal bands swamp its row residuals in
# these patches (9.38% rms, period 7), so that pair cannot test the geometry. This test needs no second function. Two flat
# patches of one picture, about 300 rows apart, are folded on a candidate period. A pattern locked to one row lattice from
# the picture's top keeps the same phase in both only at the lattice's period. 0.035 scene units is 3.15 px at 720p.
import cmath

print("\nRow lattice on tan, frame-0217: rows folded on a period. Coherence 1 is a perfect comb; phase is in degrees, from y = 179")
tan = load(NOW / "frame-0217.png")
for period in (3.0, 3.05, 3.1, 3.15, 3.2, 3.25, 3.3):
    cells = []
    for x0, x1, y0, y1 in [(60, 480, 185, 235), (60, 480, 505, 552)]:
        resid = residual(rows_of(tan, x0, x1, y0, y1))
        total = sum(r * cmath.exp(2j * math.pi * (y0 + 4 + i - 179) / period) for i, r in enumerate(resid))
        cells.append((abs(total) / sum(abs(r) for r in resid), math.degrees(cmath.phase(total))))
    gap = abs((cells[0][1] - cells[1][1] + 180) % 360 - 180)
    print(f"period {period:.2f}: upper {cells[0][0]:.2f} at {cells[0][1]:+.0f}, lower {cells[1][0]:.2f} at {cells[1][1]:+.0f}, phase gap {gap:.0f}")
