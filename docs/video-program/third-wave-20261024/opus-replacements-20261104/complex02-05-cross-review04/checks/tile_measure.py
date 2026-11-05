"""Where do the portrait's steps actually fall, on existing frame 114 of 05, against exp_tile? And how well does gold read there?"""
import json, math
from pathlib import Path
from PIL import Image
ROOT = Path("narration_lab/third-wave-20261024/root/five-film-release-20261027/complex-combined-picture-frames02/05_gallery")
im = Image.open(ROOT / "frame-0114.png").convert("RGB")
W, H = im.size
print(W, H, json.loads((ROOT / "sampling.json").read_text())["times"][114])
sx = lambda x: (x + 64 / 9) / (128 / 9) * W
sy = lambda y: (4 - y) / 8 * H
unit = 3.2 / 6
px = lambda x: sx(-2.6 + unit * x)
py = lambda y: sy(-0.25 + unit * (y - 2))
s = math.tau / 12
xs = [s * (k - 0.5) for k in range(-5, 7)]
ys = [s * (k - 0.5) for k in range(-7, 16)]
lum = lambda c: 0.2126 * c[0] + 0.7152 * c[1] + 0.0722 * c[2]
# value steps along a row across several tiles, and a column
def steps(values):
    return [(i, values[i + 1] - values[i]) for i in range(len(values) - 1)]
y_row = int(py(1.57))
row = [lum(im.getpixel((x, y_row))) for x in range(int(px(-2.9)), int(px(2.9)))]
start = int(px(-2.9))
found = [i + start + 1 for i, d in steps(row) if abs(d) > 12]
print("row y", y_row, "big steps at x:", found)
print("computed x lines:", [round(px(v), 1) for v in xs])
x_col = int(px(0.52))
col = [lum(im.getpixel((x_col, y))) for y in range(int(py(7.9)), int(py(-3.9)))]
top = int(py(7.9))
print("col x", x_col, "luma steps at y:", [i + top + 1 for i, d in steps(col) if abs(d) > 12])
print("computed y lines:", [round(py(v), 1) for v in ys if -3.9 < v < 7.9])


def rl(c):
    def f(v):
        v /= 255
        return v / 12.92 if v <= 0.03928 else ((v + 0.055) / 1.055) ** 2.4
    return 0.2126 * f(c[0]) + 0.7152 * f(c[1]) + 0.0722 * f(c[2])


def contrast(a, b):
    a, b = rl(a), rl(b)
    return (max(a, b) + 0.05) / (min(a, b) + 0.05)


# the bracket is the same colour and weight on this frame
gold = None
for y in range(int(py(3)), int(py(3)) + 1):
    for x in range(int(px(3.0)) + 5, int(px(3.0)) + 40):
        c = im.getpixel((x, y))
        if gold is None or c[0] + c[1] > gold[0] + gold[1]:
            gold = c
print("gold on this frame", gold)
for band in (1,):
    for k in range(-5, 12):
        x0, y0 = s * (band - 0.5), s * (k - 0.5)
        box = [(int(px(x0 + s * i / 8)), int(py(y0 + s * j / 8))) for i in range(9) for j in (0, 8)] + [(int(px(x0 + s * i)), int(py(y0 + s * j / 8))) for i in (0, 1) for j in range(9)]
        vals = [contrast(gold, im.getpixel(p)) for p in box if 0 <= p[0] < W and 0 <= p[1] < H]
        print(f"tile y band {k}: y {y0:5.2f}..{y0 + s:5.2f}  contrast of gold to the pixels under its outline: min {min(vals):.2f} mean {sum(vals) / len(vals):.2f}" + ("  <= repair" if k == 3 else ""))
