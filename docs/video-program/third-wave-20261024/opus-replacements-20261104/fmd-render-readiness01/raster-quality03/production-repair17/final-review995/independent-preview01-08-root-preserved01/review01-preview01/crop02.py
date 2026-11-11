"""Make one labelled composite of enlarged crops from a copied chapter's pictures (960x540), by frame number.
Usage: crop02.py CHAPTER_COPY_DIR OUT.png x0 y0 x1 y1 scale frame [frame ...]. Refuses an existing OUT."""
import json, sys
from pathlib import Path
from PIL import Image, ImageDraw
copy, out = Path(sys.argv[1]), Path(sys.argv[2])
x0, y0, x1, y1, scale = map(int, sys.argv[3:8])
frames = [int(f) for f in sys.argv[8:]]
rec = json.loads((copy/'pictures.json').read_text())
index = {f: i for i, f in enumerate(rec['selected_frames'])}
tiles = []
for f in frames:
    path = copy/'pictures'/Path(rec['pictures'][index[f]]['path']).name
    with Image.open(path) as im:
        assert im.size == (960, 540)
        tile = im.crop((x0, y0, x1, y1)).resize(((x1-x0)*scale, (y1-y0)*scale), Image.Resampling.LANCZOS)
    tiles.append((f, tile))
w, h = tiles[0][1].size
cols = min(4, len(tiles)); rows = (len(tiles)+cols-1)//cols
sheet = Image.new('RGB', (cols*w, rows*(h+20)), '#222222'); draw = ImageDraw.Draw(sheet)
for n, (f, tile) in enumerate(tiles):
    x, y = (n % cols)*w, (n//cols)*(h+20)
    sheet.paste(tile, (x, y+20)); draw.text((x+4, y+4), f'frame {f} | {f/60:.3f}s', fill='white')
with out.open('xb') as fh:
    sheet.save(fh, format='PNG')
print(out, sheet.size)
