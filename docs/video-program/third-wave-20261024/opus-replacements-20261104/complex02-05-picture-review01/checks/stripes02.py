"""04-A on preview02: row-luma stripes in the colour pictures, with controls.

    python3 -B checks/stripes02.py      (from the repo root; PIL needs a normal python, not -I)

Same measure as complex-preview01-review-02-05/checks/stripes.py: for a patch, average each row's luma, remove a 9-row
running mean, report the rms left and the strongest period by a plain DFT. Also the same measure across columns, so a
stripe that is only horizontal shows as a row/column gap.

Inputs are the extracted frame PNGs, which were decoded from the preview's H.264 at CRF 26, ultrafast, so a pattern here
can come from the picture or from the encoder. Controls for that:
  - the plain frame background beside the picture must read near 0.
  - the row pattern at one place across several frames of one still picture. This control is WEAK: inter-frame coding can
    copy a still picture's noise forward, so a correlation near 1 here does not separate picture from encoder.
  - the row pattern on DIFFERENT functions drawn in the same plane geometry, chosen frames with no loop or marker in the
    patch. Rows that stay dark across different content are tied to the geometry. The same pair shifted by one row is the
    negative control, and should fall well below the unshifted value.
Reads only. Prints to stdout.
"""
import math
import pathlib

from PIL import Image

ROOT = pathlib.Path("narration_lab/third-wave-20261024/root/five-film-release-20261027/complex-combined-picture-frames02")
# name, frame, x0, x1, y0, y1
PATCHES = [
    ("04 f(z)=z, upper left, no needle", "04_reading_color/frame-0020.png", 130, 390, 120, 360),
    ("04 f(z)=z, lower left, no needle", "04_reading_color/frame-0020.png", 130, 390, 400, 650),
    ("04 f(z)=z, lower right, no needle", "04_reading_color/frame-0020.png", 420, 680, 400, 650),
    ("background beside the picture (control)", "04_reading_color/frame-0020.png", 720, 880, 300, 650),
]
REPEAT = [
    "04_reading_color/frame-0020.png",
    "04_reading_color/frame-0024.png",
    "04_reading_color/frame-0028.png",
]
REPEAT_PATCH = (130, 390, 120, 360)


def luma(p):
    return 0.2126 * p[0] + 0.7152 * p[1] + 0.0722 * p[2]


def rows_of(path, x0, x1, y0, y1):
    im = Image.open(path).convert("RGB")
    out = []
    for y in range(y0, y1):
        vals = [luma(im.getpixel((x, y))) for x in range(x0, x1)]
        out.append(sum(vals) / len(vals))
    return out


def cols_of(path, x0, x1, y0, y1):
    im = Image.open(path).convert("RGB")
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
    return mean, rms, 100 * rms / mean, max(resid) - min(resid), n / best[1]


def correlation(a, b):
    ma, mb = sum(a) / len(a), sum(b) / len(b)
    top = sum((x - ma) * (y - mb) for x, y in zip(a, b))
    bottom = math.sqrt(sum((x - ma) ** 2 for x in a) * sum((y - mb) ** 2 for y in b))
    return top / bottom


print("patch | frame | rows: mean luma, rms, rms %, peak-to-peak, period px | columns: rms %, period px")
for name, rel, x0, x1, y0, y1 in PATCHES:
    r = stats(rows_of(ROOT / rel, x0, x1, y0, y1))
    c = stats(cols_of(ROOT / rel, x0, x1, y0, y1))
    print(f"{name} | {rel} | {r[0]:.1f}, {r[1]:.2f}, {r[2]:.2f}%, {r[3]:.1f}, {r[4]:.2f} | {c[2]:.2f}%, {c[4]:.2f}")

print("\nsame still picture, different frames: correlation of the row residuals (near 1 means the picture draws it)")
series = [residual(rows_of(ROOT / rel, *REPEAT_PATCH)) for rel in REPEAT]
for i in range(len(REPEAT)):
    for j in range(i + 1, len(REPEAT)):
        print(f"{REPEAT[i]} vs {REPEAT[j]}: {correlation(series[i], series[j]):.3f}")

print("\nfirst 24 row residuals, frame-0020 upper left:")
print(" ".join(f"{v:+.1f}" for v in series[0][:24]))

# Same plane, same rows, different functions. Patches keep clear of the loop, markers, title and the two lower bands.
FUNCTIONS = [
    ("z", "04_reading_color/frame-0020.png"),
    ("z squared", "04_reading_color/frame-0143.png"),
    ("1/z", "04_reading_color/frame-0205.png"),
    ("bump", "04_reading_color/frame-0290.png"),
]
FUNCTION_PATCHES = [("upper left", 125, 265, 100, 240), ("lower left", 125, 265, 520, 660)]

print("\ndifferent functions, same plane geometry: rows rms % / period px")
for name, rel in FUNCTIONS:
    cells = []
    for _, x0, x1, y0, y1 in FUNCTION_PATCHES:
        r = stats(rows_of(ROOT / rel, x0, x1, y0, y1))
        cells.append(f"{r[2]:.2f}% / {r[4]:.2f}")
    print(f"{name} | {rel} | " + " | ".join(cells))

print("\nrow-residual correlation between functions (unshifted, then shifted one row as the negative control)")
for patch, x0, x1, y0, y1 in FUNCTION_PATCHES:
    series = {name: residual(rows_of(ROOT / rel, x0, x1, y0, y1)) for name, rel in FUNCTIONS}
    names = [name for name, _ in FUNCTIONS]
    for i in range(len(names)):
        for j in range(i + 1, len(names)):
            a, b = series[names[i]], series[names[j]]
            print(f"{patch}: {names[i]} vs {names[j]}: {correlation(a, b):+.3f}, shifted {correlation(a[:-1], b[1:]):+.3f}")
