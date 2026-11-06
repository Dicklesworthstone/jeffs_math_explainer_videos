"""Overlap of Bronze's final captions with the existing per-cue maps, at root's measured YouTube geometry.

Controls-visible state is measured once, on another film. The shifted state is an assumption for sizing the risk only.
A cue whose new interval reaches into a neighbour's old interval also reads that neighbour's map.
"""
import json
import re
import sys
from pathlib import Path

from PIL import Image, ImageChops, ImageDraw

OUT = Path(sys.argv[1])
SRT = Path(sys.argv[2])
INK = 34
H = 2160
REF = {'player_h': 1035, 'player_w': 1840, 'top': 836.302, 'bottom': 944.302, 'line': 54, 'width': 788, 'chars': 42}
BOTTOM = REF['bottom'] / REF['player_h'] * H
LINE = REF['line'] / REF['player_h'] * H
RATE = REF['width'] / REF['player_w'] * 3840 / REF['chars'] * 1.05   # 5% over the one measured line, since letter mix varies
old = {c['id']: c for c in json.loads((OUT / 'cues.json').read_text())}


def stamp(t):
    h, m, s = t.replace(',', '.').split(':')
    return int(h) * 3600 + int(m) * 60 + float(s)


cues = []
for block in re.split(r'\n\s*\n', SRT.read_text().strip()):
    lines = block.splitlines()
    a, b = lines[1].split(' --> ')
    cues.append({'id': int(lines[0]), 'a': stamp(a), 'b': stamp(b), 'lines': lines[2:]})


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


def study(shift):
    rows = []
    for c in cues:
        x0, y0, x1, y1 = box_of(c, shift)
        crop = (int(x0) // 4, int(y0) // 4, int(x1) // 4 + 1, int(y1) // 4 + 1)
        image = picture(c).point(lambda v: 255 if v >= INK else 0).crop(crop)
        ink = image.getbbox()
        if ink:
            rows.append({'id': c['id'], 'a': c['a'], 'b': c['b'], 'n': len(c['lines']), 'count': sum(image.histogram()[255:]),
                         'ink': [(ink[0] + crop[0]) * 4, (ink[1] + crop[1]) * 4, (ink[2] + crop[0]) * 4, (ink[3] + crop[1]) * 4], 'text': ' '.join(c['lines'])})
    return rows


def runs(rows):
    out = []
    for r in rows:
        if out and r['id'] == out[-1]['last'] + 1:
            out[-1].update(last=r['id'], b=r['b'], peak=max(out[-1]['peak'], r['count']))
        else:
            out.append({'first': r['id'], 'last': r['id'], 'a': r['a'], 'b': r['b'], 'peak': r['count']})
    return out


def clock(t):
    return f'{int(t // 60):02}:{t % 60:04.1f}'


print('measured box at 4K: two lines y', round(BOTTOM - 2 * LINE), '-', round(BOTTOM), '; one line y', round(BOTTOM - LINE), '-', round(BOTTOM), '; px per char', round(RATE, 1))
for name, shift in (('controls visible, measured', 0), ('controls hidden, ASSUMED 120 px lower', 120)):
    rows = study(shift)
    wide = [r for r in rows if r['count'] >= 130]
    print('---', name, ': cues with any ink', len(rows), 'seconds', round(sum(r['b'] - r['a'] for r in rows), 1), '; cues with 130+ ink', len(wide), 'seconds', round(sum(r['b'] - r['a'] for r in wide), 1))
    for run in runs(wide):
        print(f"  {clock(run['a'])}-{clock(run['b'])} cues {run['first']}-{run['last']} peak {run['peak']}")
    (OUT / f'measured-{shift}.json').write_text(json.dumps(rows, indent=1))
    # reading sheets, 960 wide, lower strip
    tiles = []
    for run in runs(wide):
        ids = [c for c in cues if run['first'] <= c['id'] <= run['last']]
        base = None
        for c in ids:
            image = picture(c)
            base = image if base is None else ImageChops.lighter(base, image)
        frame = base.convert('RGB')
        draw = ImageDraw.Draw(frame)
        widest = max(ids, key=lambda c: max(len(line) for line in c['lines']) + 100 * len(c['lines']))
        x0, y0, x1, y1 = box_of(widest, shift)
        draw.rectangle((x0 / 4, y0 / 4, x1 / 4, y1 / 4), outline=(255, 80, 80))
        draw.text((4, 408), f"{run['first']}-{run['last']}  {clock(run['a'])}-{clock(run['b'])}", fill=(255, 255, 0))
        tiles.append(frame.crop((0, 405, 960, 530)))
    for part in range((len(tiles) + 19) // 20):
        group = tiles[part * 20:part * 20 + 20]
        sheet = Image.new('RGB', (1926, ((len(group) + 1) // 2) * 129), (70, 70, 70))
        for n, tile in enumerate(group):
            sheet.paste(tile, ((n % 2) * 966, (n // 2) * 129))
        sheet.save(OUT / f'm{shift}-{part + 1}.jpg', quality=88)
    print('  sheets', (len(tiles) + 19) // 20)
