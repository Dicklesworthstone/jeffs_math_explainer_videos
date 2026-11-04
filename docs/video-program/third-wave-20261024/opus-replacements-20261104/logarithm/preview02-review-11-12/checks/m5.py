"""Fifth pass: gold over the x2 arrow by frame, the chapter 11 plane's still range, the laps chip's fade."""
import sys
from pathlib import Path

import numpy as np
from PIL import Image

sys.path.insert(0, str(Path(__file__).parent))
import measure as M

gold = []
for i in range(169, 222):
    row = M.load("11", i)[387, 190:316]
    gold.append(int(((row[:, 0] > 190) & (row[:, 1] > 150) & (row[:, 2] < 150)).sum()))
print("gold f169-f221 min, max", min(gold), max(gold))

ref = M.load("11", 425)[170:600, 100:530]
for i in range(423, 516):
    moved = int((np.abs(M.load("11", i)[170:600, 100:530] - ref).sum(axis=2) > 36).sum())
    print(f"f{i} {M.TIMES['11'][i]:.3f} moved {moved}")

for k in range(24):
    a = np.asarray(Image.open(Path(__file__).parent / "dec" / f"d114_{k:02d}.png").convert("RGB")).astype(int)
    sub = a[228:258, 690:1200]
    print(k, round(114.40 + k / 30, 3), int(np.abs(sub - np.array([12, 17, 20])).sum(axis=2).max()))
