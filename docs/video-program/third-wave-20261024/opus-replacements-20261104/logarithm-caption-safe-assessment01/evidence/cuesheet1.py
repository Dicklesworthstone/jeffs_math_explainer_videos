"""Per-cue reading tiles, so a run composite does not blame a cue for ink from its neighbours.

Each tile is that cue's own 2 fps brightest-pixel map (plus a neighbour's if the new interval reaches it), red = measured
box with controls visible, yellow = the assumed 120 px lower box. Reading aid only.
"""
import json
import re
import sys
from pathlib import Path

from PIL import Image, ImageChops, ImageDraw

OUT = Path(sys.argv[1])
SRT = Path(sys.argv[2])
H = 2160
REF = {'player_h': 1035, 'player_w': 1840, 'top': 836.302, 'bottom': 944.302, 'line': 54, 'width': 788, 'chars': 42}
BOTTOM = REF['bottom'] / REF['player_h'] * H
LINE = REF['line'] / REF['player_h'] * H
RATE = REF['width'] / REF['player_w'] * 3840 / REF['chars'] * 1.05
old = {c['id']: c for c in json.loads((OUT / 'cues.json').read_text())}


def stamp(t):
    h, m, s = t.replace(',', '.').split(':')
    return int(h) * 3600 + int(m) * 60 + float(s)


cues = {}
for block in re.split(r'\n\s*\n', SRT.read_text().strip()):
    lines = block.splitlines()
    a, b = lines[1].split(' --> ')
    cues[int(lines[0])] = {'id': int(lines[0]), 'a': stamp(a), 'b': stamp(b), 'lines': lines[2:]}


def picture(c):
    base = Image.open(OUT / 'cue_max' / f"{c['id']:03d}.png")
    for other in (c['id'] - 1, c['id'] + 1):
        if other in old and c['a'] < old[other]['b'] and old[other]['a'] < c['b']:
            base = ImageChops.lighter(base, Image.open(OUT / 'cue_max' / f'{other:03d}.png'))
    return base


def box_of(c, shift):
    half = max(len(line) for line in c['lines']) * RATE / 2
    bottom = BOTTOM + shift
    return (1920 - half, bottom - len(c['lines']) * LINE, 1920 + half, bottom)


def clock(t):
    return f'{int(t // 60):02}:{t % 60:04.1f}'


zero = {r['id']: r['count'] for r in json.loads((OUT / 'measured-0.json').read_text()) if r['count'] >= 130}
lower = {r['id']: r['count'] for r in json.loads((OUT / 'measured-120.json').read_text()) if r['count'] >= 130}
ids = sorted(set(zero) | set(lower))
print('cues', len(ids), 'measured', len(zero), 'assumed lower', len(lower))
X0, X1, Y0, Y1 = 200, 760, 428, 534
WIDE, TALL = X1 - X0, Y1 - Y0
COLS, ROWS = 3, 14
tiles = []
for k in ids:
    c = cues[k]
    frame = picture(c).convert('RGB')
    draw = ImageDraw.Draw(frame)
    for shift, colour in ((0, (255, 70, 70)), (120, (230, 200, 40))):
        x0, y0, x1, y1 = box_of(c, shift)
        draw.rectangle((x0 / 4, y0 / 4, x1 / 4, y1 / 4), outline=colour)
    tile = frame.crop((X0, Y0, X1, Y1))
    ImageDraw.Draw(tile).text((2, 0), f"{k} {clock(c['a'])}-{clock(c['b'])} {zero.get(k, 0)}/{lower.get(k, 0)}", fill=(120, 255, 120))
    tiles.append(tile)
parts = (len(tiles) + COLS * ROWS - 1) // (COLS * ROWS)
for part in range(parts):
    group = tiles[part * COLS * ROWS:(part + 1) * COLS * ROWS]
    sheet = Image.new('RGB', (COLS * WIDE + (COLS - 1) * 6, ((len(group) + COLS - 1) // COLS) * (TALL + 4)), (70, 70, 70))
    for n, tile in enumerate(group):
        sheet.paste(tile, ((n % COLS) * (WIDE + 6), (n // COLS) * (TALL + 4)))
    sheet.save(OUT / f'c{part + 1}.jpg', quality=90)
print('sheets', parts)
