"""Third pass: corrected boxes, column gaps, and decoded frames around two moments."""
import sys
from pathlib import Path

import numpy as np
from PIL import Image

sys.path.insert(0, str(Path(__file__).parent))
import measure as M


def runs(ch, i, box, label, thresh=60, min_gap=3):
    """Column runs of non-field pixels inside a box, and the gaps between them."""
    a = M.load(ch, i)[box[1] : box[3], box[0] : box[2]]
    cols = (np.abs(a - np.array([12, 17, 20])).sum(axis=2) > thresh).any(axis=0)
    out, start, off = [], None, 0
    for x, v in enumerate(cols):
        if v:
            if start is None:
                start = x
            off = 0
        elif start is not None:
            off += 1
            if off >= min_gap:
                out.append((start + box[0], x - off + box[0]))
                start, off = None, 0
    if start is not None:
        out.append((start + box[0], box[2] - 1))
    print(f"  {label}: ch{ch} f{i} t={M.TIMES[ch][i]:.3f} column runs {out}")


def fill(ch, i, box, label):
    a = M.load(ch, i)[box[1] : box[3], box[0] : box[2]].reshape(-1, 3)
    m = a.mean(axis=0)
    print(f"  {label}: ch{ch} f{i} mean {[round(float(x)) for x in m]} std {a.std(axis=0).round(1).tolist()} against field: {M.ratio(m, np.array([12, 17, 20])):.2f}:1")


print("fills and small text")
fill("11", 24, (327, 395, 340, 420), "before-F stem")
fill("11", 33, (327, 395, 340, 420), "before-F stem with after present")
fill("11", 33, (290, 395, 300, 420), "after-F stem")
M.contrast("12", 320, (805, 558, 1170, 576), "turning tick labels, dim 16 pt")
M.contrast("12", 300, (690, 183, 735, 203), "time word, dim")
M.contrast("12", 300, (972, 183, 1012, 203), "laps value, purple")
M.contrast("12", 62, (307, 565, 335, 585), "minus-3-pi label, purple 16 pt")

print("readout chapter 11")
M.bbox("11", 69, (690, 90, 1100, 132), "signed area word f69")
M.bbox("11", 69, (690, 134, 780, 162), "live value f69")
M.bbox("11", 137, (690, 90, 1100, 132), "det word f137")
M.bbox("11", 137, (690, 134, 780, 162), "live value f137")
runs("11", 375, (880, 392, 1090, 410), "key F then word, f375")
runs("11", 386, (880, 392, 1090, 410), "key F then word, f386")
runs("12", 62, (300, 508, 625, 530), "Log line then border, f62")

print("decoded frames near the end of the 1.5-lap run")
S = Path(__file__).parent / "dec"
R = "narration_lab/third-wave-20261024/root/five-film-release-20261027/logarithm-opus-rebuild-preview02/render/12_what_it_forgets.mp4"
for k in range(10):
    p = S / f"a110_{k:02d}.png"
    a = np.asarray(Image.open(p).convert("RGB")).astype(int)
    # where is the gold moving dot near the end ring? count gold pixels inside the end-ring box
    ring = a[120:165, 215:255]  # end ring at sheet (178,265) -> frame (237,313)
    print(f"  {p.name}: mean of end-ring box {ring.mean(axis=(0, 1)).round(1).tolist()}")
Image.open(S / "a110_00.png").crop((680, 170, 1100, 220)).save(S / "a110_00_read.png")
for k in range(10):
    Image.open(S / f"a110_{k:02d}.png").crop((680, 170, 1100, 220)).save(S / f"a110_{k:02d}_read.png")
strip = Image.new("RGB", (420 * 2, 50 * 5))
for k in range(10):
    strip.paste(Image.open(S / f"a110_{k:02d}_read.png"), ((k % 2) * 420, (k // 2) * 50))
strip.save(S / "a110_strip.png")
strip2 = Image.new("RGB", (300 * 2, 240 * 5))
for k in range(10):
    strip2.paste(Image.open(S / f"b126_{k:02d}.png").crop((170, 190, 470, 430)), ((k % 2) * 300, (k // 2) * 240))
strip2.save(S / "b126_strip.png")
strip3 = Image.new("RGB", (300 * 2, 240 * 5))
for k in range(10):
    strip3.paste(Image.open(S / f"a110_{k:02d}.png").crop((170, 190, 530, 430)).resize((300, 200)), ((k % 2) * 300, (k // 2) * 240))
strip3.save(S / "a110_cube_strip.png")
