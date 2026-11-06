#!/usr/bin/env python3
"""How much of four on-screen labels sits under the measured caption block. Run from the repository root.

    python3 -I -B <bundle>/check_label_shares.py MAPS

MAPS as in check_review.py. Each label's rows and column limits are set by hand from the cue tiles; its extent is then
measured from the map. Column share under a line box, controls visible, Arial plus 0.5 em. Writes outputs/label-shares.txt.
"""
import json
from pathlib import Path
import sys

import numpy as np
from PIL import Image, ImageFont

HERE = Path(__file__).resolve().parent
P = HERE.parent
maps = Path(sys.argv[1])
font = ImageFont.truetype('/System/Library/Fonts/Supplemental/Arial.ttf', 96)
PITCH = 54 * 2160 / 1035
BOTTOM = 944.3020839691162 * 2160 / 1035
# name, cues, map rows, map columns
LABELS = (('its logarithm', range(215, 221), (480, 496), (515, 610)),
          ('bottom edge: left out', [288], (452, 466), (355, 495)),
          ('infinitely many real logarithms', [357, 358], (468, 484), (540, 960)),
          ('many flows, one place: all home at t = 1', range(446, 450), (432, 448), (508, 960)))

cues = {}
for block in (P / 'logarithm-caption-timing-repair01/sidecars-timing01/captions.en.srt').read_text().strip().split('\n\n'):
    parts = block.split('\n')
    cues[int(parts[0])] = parts[2:]
out = []
for name, ids, (top, low), (left, right) in LABELS:
    out.append(name)
    for cue in ids:
        with Image.open(maps / 'cue_max' / f'{cue:03d}.png') as image:
            mask = np.asarray(image.convert('L')) >= 34
        columns = np.nonzero(mask[top:low, left:right].any(axis=0))[0] + left
        lines = cues[cue]
        n = len(lines)
        covered = np.zeros(960, bool)
        for i, line in enumerate(lines):
            y1 = BOTTOM - (n - 1 - i) * PITCH
            y0 = y1 - PITCH
            if y0 / 4 < low and y1 / 4 > top:
                half = (font.getlength(line) + 48) / 2
                covered[int((1920 - half) / 4):int(np.ceil((1920 + half) / 4))] = True
        share = covered[columns].sum() / max(len(columns), 1)
        out.append(f'  cue {cue}: label columns {columns.min()} to {columns.max()}, under a line box {share:.0%}')
text = '\n'.join(out) + '\n'
print(text, end='')
(HERE / 'outputs/label-shares.txt').write_text(text)
