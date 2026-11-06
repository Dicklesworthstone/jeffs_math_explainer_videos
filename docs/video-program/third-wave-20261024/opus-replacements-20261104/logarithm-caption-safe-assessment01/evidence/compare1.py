"""Minimal comparison on the 23 real clean 4K frames of the first css run: no decode, no caption drawn, outlines only.

Blue = the libass proxy's real changed-pixel box from that run. Red = root's measured box, controls visible.
Yellow = the same box 120 px lower, an assumption. Boxes use the cue active under Bronze's final SRT at that frame.
Prints bright pixels of the clean frame inside each box.
"""
import json
import re
import sys
from pathlib import Path

from PIL import Image, ImageDraw

OUT = Path(sys.argv[1])
SRT = Path(sys.argv[2])
JOB = Path(sys.argv[3])
TILE_OUT = sys.argv[4]
INK = 34
H = 2160
REF = {'player_h': 1035, 'player_w': 1840, 'bottom': 944.302, 'line': 54, 'width': 788, 'chars': 42}
BOTTOM = REF['bottom'] / REF['player_h'] * H
LINE = REF['line'] / REF['player_h'] * H
RATE = REF['width'] / REF['player_w'] * 3840 / REF['chars'] * 1.05


def stamp(t):
    h, m, s = t.replace(',', '.').split(':')
    return int(h) * 3600 + int(m) * 60 + float(s)


cues = []
for block in re.split(r'\n\s*\n', SRT.read_text().strip()):
    lines = block.splitlines()
    a, b = lines[1].split(' --> ')
    cues.append({'id': int(lines[0]), 'a': stamp(a), 'b': stamp(b), 'lines': lines[2:]})


def box_of(c, shift):
    half = max(len(line) for line in c['lines']) * RATE / 2
    bottom = BOTTOM + shift
    return (int(1920 - half), int(bottom - len(c['lines']) * LINE), int(1920 + half), int(bottom))


def clock(t):
    return f'{int(t // 60):02}:{t % 60:04.1f}'


def ink(gray, box):
    return sum(gray.crop(box).point(lambda v: 255 if v >= INK else 0).histogram()[255:])


summary = json.loads((JOB / 'overlay-summary.json').read_text())
rows, tiles = [], []
for s in summary['samples']:
    active = [c for c in cues if c['a'] <= s['seconds'] < c['b']]
    with Image.open(JOB / 'clean' / (s['id'] + '.png')) as frame:
        rgb = frame.convert('RGB')
    gray = rgb.convert('L')
    row = {'id': s['id'], 'seconds': s['seconds'], 'proxy_box': s['caption_box'], 'proxy_ink': ink(gray, tuple(s['caption_box'])), 'row': s['row']}
    draw = ImageDraw.Draw(rgb)
    draw.rectangle(s['caption_box'], outline=(90, 150, 255), width=5)
    if active:
        c = active[0]
        row.update(cue=c['id'], lines=c['lines'], measured_ink=ink(gray, box_of(c, 0)), lower_ink=ink(gray, box_of(c, 120)))
        draw.rectangle(box_of(c, 0), outline=(255, 70, 70), width=5)
        draw.rectangle(box_of(c, 120), outline=(230, 200, 40), width=5)
    rows.append(row)
    tile = rgb.crop((0, 1640, 3840, 2160)).resize((960, 130), Image.LANCZOS)
    ImageDraw.Draw(tile).text((3, 1), f"{s['id']} {clock(s['seconds'])} cue {row.get('cue', 'none')}", fill=(120, 255, 120))
    tiles.append(tile)
    print(s['id'], clock(s['seconds']), 'cue', row.get('cue'), 'proxy', row['proxy_ink'], 'measured', row.get('measured_ink'), 'lower', row.get('lower_ink'), '|', ' / '.join(row.get('lines', [])), flush=True)
(OUT / 'cmp.json').write_text(json.dumps(rows, indent=1))
sheet = Image.new('RGB', (1926, 12 * 134), (70, 70, 70))
for n, tile in enumerate(tiles):
    sheet.paste(tile, ((n % 2) * 966, (n // 2) * 134))
sheet.save(TILE_OUT, quality=90)
print('frames with ink', {key: sum(1 for r in rows if r.get(key, 0) >= 300) for key in ('proxy_ink', 'measured_ink', 'lower_ink')}, 'of', len(rows))
