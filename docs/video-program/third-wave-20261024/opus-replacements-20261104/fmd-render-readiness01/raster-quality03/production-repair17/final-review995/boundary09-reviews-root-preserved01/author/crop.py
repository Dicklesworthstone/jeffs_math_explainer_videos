"""Crop and enlarge a region of one extracted 960x540 picture, for reading small gaps by eye.

usage: python3 -I -B crop.py PICTURE OUT.png X0 Y0 X1 Y1 [SCALE]
Reads PICTURE only; writes a new OUT.png (refuses an existing one). Nearest-neighbour enlargement, so every
source pixel shows as a block and nothing is smoothed or invented.
"""
import sys
from pathlib import Path

from PIL import Image

src, out = Path(sys.argv[1]), Path(sys.argv[2])
x0, y0, x1, y1 = (int(v) for v in sys.argv[3:7])
scale = int(sys.argv[7]) if len(sys.argv) > 7 else 6
im = Image.open(src).convert('RGB').crop((x0, y0, x1, y1))
im = im.resize((im.width * scale, im.height * scale), Image.NEAREST)
with out.open('xb') as f:
    im.save(f, format='PNG')
print(out, im.size)
