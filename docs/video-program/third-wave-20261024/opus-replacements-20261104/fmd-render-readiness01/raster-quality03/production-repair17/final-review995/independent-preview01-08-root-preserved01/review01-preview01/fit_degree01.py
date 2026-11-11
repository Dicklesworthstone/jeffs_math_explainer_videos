"""Which partial sum of 1 - x^2 + x^4 - ... is the yellow curve in a 01 picture? Calibrates the plot from the
white bump 1/(1+x^2) and the axes, reads the yellow curve's height per column, and reports the RMS error
against each even degree 0..60. Local, read-only. Usage: fit_degree01.py CHAPTER_COPY_DIR frame [frame ...]"""
import json, math, sys
from pathlib import Path
from PIL import Image

copy = Path(sys.argv[1])
rec = json.loads((copy/'pictures.json').read_text())
index = {f: i for i, f in enumerate(rec['selected_frames'])}

def partial(x, d):          # terms up to degree d (d even): sum_{k<=d/2} (-x^2)^k
    return sum((-x*x)**k for k in range(d//2 + 1))

for f in map(int, sys.argv[2:]):
    im = Image.open(copy/'pictures'/Path(rec['pictures'][index[f]]['path']).name).convert('RGB')
    W, H = im.size
    px = im.load()
    # the plot lives in x 100..700, y 150..530 (960x540 picture)
    def is_white(p): return min(p) > 200 and max(p) - min(p) < 30
    def is_yellow(p): return p[0] > 170 and p[1] > 140 and p[2] < 140 and p[0] - p[2] > 70
    def is_gray(p): return 80 < p[0] < 190 and max(p) - min(p) < 25
    # horizontal axis row: the row in y 300..520 with the most gray pixels over x 110..690
    axis_y = max(range(300, 530), key=lambda y: sum(is_gray(px[x, y]) for x in range(110, 690)))
    # vertical axis column: the column in x 300..500 with the most gray pixels over y 160..520
    axis_x = max(range(300, 500), key=lambda x: sum(is_gray(px[x, y]) for y in range(160, 520)))
    # white curve: per column, the mean row of white pixels above the axis
    white = {}
    for x in range(110, 690):
        ys = [y for y in range(150, axis_y - 2) if is_white(px[x, y])]
        if ys: white[x] = sum(ys)/len(ys)
    # calibrate sx and sy by least squares of the white points on sy/(1+u^2), u = (x-axis_x)/sx
    # (the yellow curve covers the white one near the peak, so the peak itself is not used)
    best = None
    for sx10 in range(600, 1400):
        sx = sx10/10
        g = {x: 1/(1 + ((x - axis_x)/sx)**2) for x in white}
        sy = sum((axis_y - white[x])*g[x] for x in white)/sum(g[x]**2 for x in white)
        err = sum((axis_y - white[x] - sy*g[x])**2 for x in white)
        if best is None or err < best[0]: best = (err, sx, sy)
    _, sx, sy = best
    yellow = {}
    for x in range(110, 690):
        ys = [y for y in range(150, 532) if is_yellow(px[x, y])]
        if ys and max(ys) - min(ys) < 12: yellow[x] = sum(ys)/len(ys)
    pts = [((x - axis_x)/sx, (axis_y - y)/sy) for x, y in yellow.items()]
    pts = [(u, v) for u, v in pts if abs(u) < 1.25]
    fits = sorted((math.sqrt(sum((v - partial(u, d))**2 for u, v in pts)/len(pts)), d) for d in range(0, 62, 2))
    xs = [u for u, _ in pts]
    print(json.dumps(dict(frame=f, axis_x=axis_x, axis_y=axis_y, sx=sx, sy=sy, white_columns=len(white),
                          white_rms=round(math.sqrt(best[0]/len(white))/sy, 4), yellow_points=len(pts),
                          yellow_x_range=[round(min(xs), 3), round(max(xs), 3)] if xs else None,
                          best_degrees=[(d, round(r, 4)) for r, d in fits[:4]])))
