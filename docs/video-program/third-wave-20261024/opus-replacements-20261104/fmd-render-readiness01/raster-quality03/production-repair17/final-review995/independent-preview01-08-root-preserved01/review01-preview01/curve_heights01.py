"""Read the yellow and white curve heights of a 01 series picture at fixed x, and compare them with the bump
and the partial sums of degree 2..14. Calibration: x from the two dashed lines at x = -1 and +1 (or the
given column of x = +1 when there are none), y from the white curve at x = +-1 and +-2 (1/2 and 1/5).
Local, read-only. Usage: curve_heights01.py CHAPTER_COPY_DIR XONE_COLUMN frame [frame ...]"""
import json, sys
from pathlib import Path
from PIL import Image

copy, xone = Path(sys.argv[1]), int(sys.argv[2])
rec = json.loads((copy/'pictures.json').read_text())
index = {f: i for i, f in enumerate(rec['selected_frames'])}
def partial(x, d): return sum((-x*x)**k for k in range(d//2 + 1))
def is_white(p): return min(p) > 200 and max(p) - min(p) < 30
def is_yellow(p): return p[0] > 170 and p[1] > 140 and p[2] < 140 and p[0] - p[2] > 70
def is_gray(p): return 80 < p[0] < 190 and max(p) - min(p) < 25

for f in map(int, sys.argv[3:]):
    im = Image.open(copy/'pictures'/Path(rec['pictures'][index[f]]['path']).name).convert('RGB'); px = im.load()
    axis_y = max(range(300, 530), key=lambda y: sum(is_gray(px[x, y]) for x in range(110, 690)))
    axis_x = max(range(300, 500), key=lambda x: sum(is_gray(px[x, y]) for y in range(160, 520)))
    sx = xone - axis_x
    def mean_rows(x, test, lo=150, hi=None):
        ys = [y for y in range(lo, hi or axis_y - 2) if test(px[x, y])]
        return (sum(ys)/len(ys), max(ys) - min(ys)) if ys else (None, None)
    # y scale from the white curve at u = 2 (1/5) and u = 1 (1/2), both sides
    samples = []
    for u, v in ((2, 0.2), (-2, 0.2), (1.5, 1/3.25), (-1.5, 1/3.25)):
        y, _ = mean_rows(round(axis_x + u*sx), is_white)
        if y is not None: samples.append((axis_y - y)/v)
    sy = sum(samples)/len(samples)
    print('frame', f, 'axis', axis_x, axis_y, 'sx', sx, 'sy %.1f' % sy, 'sy samples', [round(s, 1) for s in samples])
    print('   u   yellow   spread   bump   d2     d6     d8     d10    d14')
    for u in (0.3, 0.5, 0.6, 0.7, 0.8, 0.85, 0.9, 0.95, 1.0, -0.8, -0.9):
        y, spread = mean_rows(round(axis_x + u*sx), is_yellow, 150, 535)
        val = None if y is None else (axis_y - y)/sy
        print('  %5.2f  %s  %s   %.3f  %s' % (u, 'none  ' if val is None else '%6.3f' % val,
              ' -  ' if spread is None else '%4d' % spread, 1/(1 + u*u),
              '  '.join('%6.3f' % partial(u, d) for d in (2, 6, 8, 10, 14))))
