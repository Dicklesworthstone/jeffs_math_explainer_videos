"""05-A comb on e^z and 05-F bars on tan, from the film's shade() rule and from the frames.

    python3 -I -B comb_and_bars.py      (from the repo root)

Mirrors model.rs shade() and cell_input() and draw.rs picture_cells(). Uses libm's hypot through ctypes,
because Rust's f64::hypot calls it and Python's math.hypot rounds differently. Reads frames with PIL.
Not a render: no image is written.
"""
import cmath
import ctypes
import math
import pathlib

from PIL import Image

libm = ctypes.CDLL(None)
libm.hypot.restype = ctypes.c_double
libm.hypot.argtypes = [ctypes.c_double, ctypes.c_double]
STEP = math.tau / 12
CELL = 0.035
ROOT = pathlib.Path("narration_lab/third-wave-20261024/root/five-film-release-20261027/complex-opus-rebuild-frames01")


def rem1(v):
    r = math.fmod(v, 1.0)
    return r + 1.0 if r < 0 else r


def bands(re, im, shift=0.0):
    size = float(libm.hypot(re, im))
    arg = math.atan2(im, re)
    a = math.fmod(arg, math.tau)
    a = a + math.tau if a < 0 else a
    return rem1(math.log(size) / STEP + shift), rem1(a / STEP + shift)


def shade(re, im, shift=0.0):
    s, p = bands(re, im, shift)
    return (0.58 + 0.42 * s) * (0.80 + 0.20 * p)


def luma(px):
    return 0.2126 * px[0] + 0.7152 * px[1] + 0.0722 * px[2]


print("step", STEP, "band ratio e^step", math.exp(STEP))

# --- comb on e^z: width 3.2, x -3..3, y -4..8, height 6.4
W, H = 3.2, 6.4
cols, rows = round(W / CELL), round(H / CELL)
mid = (cols - 1) / 2
print(f"e^z picture: {cols} columns, {rows} rows; centre column index {mid}")
u = (mid + 0.5) / cols
x = -3.0 + 6.0 * u
print("centre column samples x =", x)
pred = []
for r in range(rows):
    y = 8.0 - 12.0 * ((r + 0.5) / rows)
    re, im = math.exp(x) * math.cos(y), math.exp(x) * math.sin(y)
    s, _ = bands(re, im)
    pred.append(s > 0.5)
print("rows where centre cell lands on the band's top (size just under 1):", sum(pred), "of", rows)
flips = sum(1 for a, b in zip(pred, pred[1:]) if a != b)
print("changes of side between neighbouring rows:", flips)

# frame 92: plane x 262..550, y 94..670 native; centre column x about 406
im = Image.open(ROOT / "05_gallery/frame-0092.png").convert("RGB")
top, height = 94.5, 576.0
agree = 0
tested = 0
obs = []
for r in range(rows):
    yy = int(round(top + (r + 0.5) * height / rows))
    if not (0 <= yy < 720):
        continue
    c = luma(im.getpixel((406, yy)))
    left = luma(im.getpixel((402, yy)))
    right = luma(im.getpixel((410, yy)))
    # bright side of the edge is the left; a tooth is the centre cell matching the left, not the right
    tooth = abs(c - left) < abs(c - right)
    obs.append(tooth)
    tested += 1
    agree += tooth == pred[r]
print(f"frame 92, teeth seen at x=406 by nearest neighbour: {sum(obs)} of {tested}; agreement with predicted rows: {agree} of {tested}")

# --- bars on tan
print("tan z above the line: shade across one period, unshifted and with bands moved half a step")
for y in (1.0, 2.0, 3.0):
    vals, shifted = [], []
    for k in range(64):
        xx = -3.0 + k * math.pi / 64 + 1e-3
        w = cmath.tan(complex(xx, y))
        vals.append(shade(w.real, w.imag))
        shifted.append(shade(w.real, w.imag, 0.5))
    print(f"  y={y}: min {min(vals):.3f} max {max(vals):.3f} ratio {max(vals) / min(vals):.2f}; half-step shift: min {min(shifted):.3f} max {max(shifted):.3f} ratio {max(shifted) / min(shifted):.2f}")
print("tan z above the line with only the size bands moved half a step (phase lines stay on the wheel's spokes)")
for y in (1.0, 2.0, 3.0):
    vals = []
    for k in range(64):
        w = cmath.tan(complex(-3.0 + k * math.pi / 64 + 1e-3, y))
        s, ph = bands(w.real, w.imag)
        s = rem1(s + 0.5)
        vals.append((0.58 + 0.42 * s) * (0.80 + 0.20 * ph))
    print(f"  y={y}: min {min(vals):.3f} max {max(vals):.3f} ratio {max(vals) / min(vals):.2f}")
print("e^z at the x=0 band edge with bands moved half a step: left and right of 0 give", round(shade(math.exp(-0.05), 0.1, 0.5), 3), round(shade(math.exp(0.05), 0.1, 0.5), 3))

# frame 200 of 05: tan picture x 46..802; bar means along two rows above the line
im = Image.open(ROOT / "05_gallery/frame-0200.png").convert("RGB")
for yy in (200, 260):
    means = []
    x0 = 46.0
    wbar = 756.0 / 16
    for b in range(16):
        xs = range(int(x0 + b * wbar + 8), int(x0 + (b + 1) * wbar - 8))
        rows_ = range(yy - 8, yy + 9)
        vals = [luma(im.getpixel((a, c))) for a in xs for c in rows_]
        means.append(sum(vals) / len(vals))
    print(f"frame 200 row {yy}: mean luma of the 16 bars:", [round(m) for m in means], f"max/min {max(means) / min(means):.2f}")
