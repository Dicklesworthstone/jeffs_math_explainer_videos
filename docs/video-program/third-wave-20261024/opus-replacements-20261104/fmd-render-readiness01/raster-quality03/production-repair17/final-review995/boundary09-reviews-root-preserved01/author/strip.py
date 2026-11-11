"""Crop the same region from several extracted 960x540 pictures and lay the enlargements side by side.

usage: python3 -I -B strip.py PICTURES_DIR OUT.png X0 Y0 X1 Y1 SCALE PIC [PIC ...]
PIC is a picture number (frame-NNNNNN.jpg). Reads the pictures only; writes a new OUT.png (refuses an existing
one). Nearest-neighbour enlargement; each panel has a 14 px header with its picture number and a 4 px gap.
"""
import sys
from pathlib import Path

from PIL import Image, ImageDraw

pics_dir, out = Path(sys.argv[1]), Path(sys.argv[2])
x0, y0, x1, y1, scale = (int(v) for v in sys.argv[3:8])
pics = [int(v) for v in sys.argv[8:]]
w, h = (x1 - x0) * scale, (y1 - y0) * scale
sheet = Image.new('RGB', (len(pics) * (w + 4), h + 14), (40, 40, 40))
draw = ImageDraw.Draw(sheet)
for i, n in enumerate(pics):
    im = Image.open(pics_dir / f'frame-{n:06d}.jpg').convert('RGB').crop((x0, y0, x1, y1))
    sheet.paste(im.resize((w, h), Image.NEAREST), (i * (w + 4), 14))
    draw.text((i * (w + 4) + 2, 1), str(n), fill=(230, 230, 230))
with out.open('xb') as f:
    sheet.save(f, format='PNG')
print(out, sheet.size)
