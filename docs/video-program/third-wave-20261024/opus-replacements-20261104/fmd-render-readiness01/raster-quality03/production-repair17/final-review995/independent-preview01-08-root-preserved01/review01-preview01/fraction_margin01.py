"""Measure the 01 header ink against the frame edges on the extracted 960x540 pictures: the bounding box of the
fraction 1/(1+x^2) (search box x 380..520, y 0..100) and of the title (x 250..380, y 0..100), the topmost ink row
of the whole picture, and the leftmost and rightmost ink columns. Ink = more than THRESH levels from the
background colour in any channel. Frame height is 8 scene units, so 67.5 px per unit at 540p.
Local, read-only. Usage: fraction_margin01.py CHAPTER_COPY_DIR THRESH frame [frame ...]"""
import json, sys
from pathlib import Path
from PIL import Image

copy, thresh = Path(sys.argv[1]), int(sys.argv[2])
rec = json.loads((copy/'pictures.json').read_text())
index = {f: i for i, f in enumerate(rec['selected_frames'])}
PX_PER_UNIT = 540/8
def bbox(px, bg, x0, y0, x1, y1):
    pts = [(x, y) for x in range(x0, x1) for y in range(y0, y1)
           if max(abs(a - b) for a, b in zip(px[x, y], bg)) > thresh]
    if not pts: return None
    xs, ys = [p[0] for p in pts], [p[1] for p in pts]
    return [min(xs), min(ys), max(xs), max(ys)]
print('frame  fraction_bbox          top_px top_units top_%   title_bbox            frame_top_ink  left  right  right_margin_%')
for f in map(int, sys.argv[3:]):
    im = Image.open(copy/'pictures'/Path(rec['pictures'][index[f]]['path']).name).convert('RGB')
    px = im.load(); W, H = im.size
    bg = px[3, H - 3]
    frac = bbox(px, bg, 380, 0, 520, 100)
    title = bbox(px, bg, 250, 0, 380, 100)
    whole = bbox(px, bg, 0, 0, W, H)
    if frac:
        top = frac[1]
        print('%5d  %-22s %4d   %6.3f   %5.2f   %-21s %5d         %4d  %5d  %6.2f' % (
            f, frac, top, top/PX_PER_UNIT, 100*top/H, title, whole[1], whole[0], whole[2], 100*(W - 1 - whole[2])/W))
    else:
        print('%5d  no fraction ink; whole %s' % (f, whole))
