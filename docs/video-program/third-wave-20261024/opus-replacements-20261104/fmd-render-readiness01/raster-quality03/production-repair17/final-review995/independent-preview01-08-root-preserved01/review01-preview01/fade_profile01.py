"""The ending fade of 01, measured on the extracted pictures: per frame, the mean and the 99.9th percentile of
the per-pixel distance from the background colour, and the share of pixels more than 8 levels from it.
Also the same for a few probe regions (title, fraction, ring, thumbnails). Local, read-only.
Usage: fade_profile01.py CHAPTER_COPY_DIR first_frame last_frame"""
import json, sys
from pathlib import Path
from PIL import Image

copy, lo, hi = Path(sys.argv[1]), int(sys.argv[2]), int(sys.argv[3])
rec = json.loads((copy/'pictures.json').read_text())
index = {f: i for i, f in enumerate(rec['selected_frames'])}
REGIONS = {'title+fraction': (270, 30, 480, 110), 'curve+ring': (100, 160, 690, 520), 'thumbnails': (790, 80, 930, 480)}
print('frame   bg              mean   p99.9  >8lvl%   ' + '  '.join('%s(max)' % k for k in REGIONS))
for f in [x for x in rec['selected_frames'] if lo <= x <= hi]:
    im = Image.open(copy/'pictures'/Path(rec['pictures'][index[f]]['path']).name).convert('RGB')
    px = im.load(); W, H = im.size
    bg = px[3, H - 3]
    d = sorted(max(abs(a - b) for a, b in zip(px[x, y], bg)) for x in range(0, W, 2) for y in range(0, H, 2))
    mean = sum(d)/len(d); p999 = d[int(len(d)*0.999)]; share = 100*sum(1 for v in d if v > 8)/len(d)
    reg = []
    for name, (x0, y0, x1, y1) in REGIONS.items():
        reg.append(max(max(abs(a - b) for a, b in zip(px[x, y], bg)) for x in range(x0, x1) for y in range(y0, y1)))
    print('%5d  %-14s %5.2f  %5d  %6.2f   ' % (f, bg, mean, p999, share) + '  '.join('%8d' % r for r in reg))
