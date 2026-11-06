#!/usr/bin/env python3
"""Write both proxy candidates, the positioned VTT and the sample plan from one SRT and VTT pair. Run from the repository root.

    python3 -I -B <bundle>/checks/make_candidate02.py SRT VTT INK_JSON

SRT and VTT are repository-relative and are only read. INK_JSON is the per-cue ink list that picks each window's busiest
cue. A change in cue count, text or the SRT/VTT pairing stops this before anything is written.

Candidate A: one top position over the chapter title, for every cue in positions02.json.
Candidate B: the per-segment rectangles in placements03.json, which avoid the title and the math on the maps where they can.
"""
import hashlib
import json
from pathlib import Path
import re
import sys
import textwrap

HERE = Path(__file__).resolve().parents[1]
P = HERE.parent
FIRST_SRT = 'narration_lab/third-wave-20261024/root/elevenlabs-logarithm-rebuild-20261104/remote-mac-mini-max01/jobs/logarithm-remote-render-work01/logarithm_mysteries.en.srt'
MASTER = 'narration_lab/third-wave-20261024/root/elevenlabs-logarithm-rebuild-20261104/remote-mac-mini-max01/jobs/logarithm-remote-render-work01/logarithm_mysteries_master.mp4'
MASTER_SHA256 = '8534ec211790fb8400b809de19a6d1cb75af6150a5e90566f6a93bb90e7efd69'
FPS = 60
RATE = 50.9  # 4K px per character, the widest of the 23 real proxy boxes
EXTRA = {36: 'window', 109: 'window, no clean region', 177: 'window', 184: 'window', 247: 'window, no clean region', 271: 'window', 324: 'window', 329: 'window',
         345: 'window', 357: 'window', 25: 'no clean region', 111: 'no clean region', 245: 'no clean region'}
HEADER = """[Script Info]
ScriptType: v4.00+
PlayResX: 384
PlayResY: 288
ScaledBorderAndShadow: yes
YCbCr Matrix: None

[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
Style: Default,Arial,16,&Hffffff,&Hffffff,&H0,&H0,0,0,0,0,100,100,0,0,1,1,0,2,10,10,10,1
Style: Top,Arial,16,&Hffffff,&Hffffff,&H0,&H0,0,0,0,0,100,100,0,0,1,1,0,8,10,10,2,1

[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
"""


def sha(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def seconds(text):
    hours, minutes, rest = text.replace(',', '.').split(':')
    return int(hours) * 3600 + int(minutes) * 60 + float(rest)


def read_srt(path):
    cues = []
    for block in re.split(r'\n\s*\n', Path(path).read_text().strip()):
        lines = block.splitlines()
        start, end = lines[1].split(' --> ')
        cues.append({'id': int(lines[0]), 'start': start, 'end': end, 'a': seconds(start), 'b': seconds(end), 'lines': lines[2:]})
    return cues


def read_vtt(path):
    blocks = re.split(r'\n\s*\n', Path(path).read_text().strip())
    assert blocks[0] == 'WEBVTT', 'Expected a plain WEBVTT header'
    return [block.splitlines() for block in blocks[1:]]


def ass_time(value):
    centi = round(value * 100)
    return f'{centi // 360000}:{centi // 6000 % 60:02}:{centi // 100 % 60:02}.{centi % 100:02}'


def rewrap(lines, label):
    if label == 'full':
        return lines
    return textwrap.wrap(' '.join(lines), width=int((int(label[1:]) - 60) / RATE))


def main():
    assert len(sys.argv) == 4, __doc__
    srt, vtt, ink = sys.argv[1:]
    cues, blocks = read_srt(srt), read_vtt(vtt)
    first = read_srt(FIRST_SRT)
    assert len(cues) == len(first) == len(blocks) == 475 and all(c['lines'] == f['lines'] and c['id'] == f['id'] for c, f in zip(cues, first)), 'Cue count or text changed'
    for c, block in zip(cues, blocks):
        assert block[0] == f"{c['start'].replace(',', '.')} --> {c['end'].replace(',', '.')}" and block[1:] == c['lines'], f"SRT and VTT differ at cue {c['id']}"
        assert not re.search(r'[\\{}]', ' '.join(c['lines'])), f"Cue {c['id']} needs ASS escaping"
    positions = json.loads((HERE / 'positions02.json').read_text())
    placements = json.loads((HERE / 'placements03.json').read_text())
    top = {cue for row in positions['windows'] for cue in range(row['first'], row['last'] + 1)}
    placed = {cue: row for row in placements for cue in range(row['first'], row['last'] + 1)}
    assert set(placed) == top, 'Candidate B does not cover the same cues as candidate A'
    busy = {row['id']: row['count'] for row in json.loads(Path(ink).read_text())}
    events_a, events_b, out = HEADER, HEADER, 'WEBVTT\n'
    for c, block in zip(cues, blocks):
        place = 'Top' if c['id'] in top else 'Default'
        times = f"{ass_time(c['a'])},{ass_time(c['b'])}"
        text = '\\N'.join(c['lines'])
        events_a += f'Dialogue: 0,{times},{place},,0,0,0,,{text}\n'
        if c['id'] in placed:
            row = placed[c['id']]
            spot = row.get('none', row)
            x0, y0, x1, _ = spot['rect']
            wrapped = '\\N'.join(rewrap(c['lines'], spot['wrap']))
            events_b += f"Dialogue: 0,{times},Default,,0,0,0,,{{\\an8\\pos({(x0 + x1) / 20:.1f},{y0 / 7.5:.1f})}}{wrapped}\n"
        else:
            events_b += f'Dialogue: 0,{times},Default,,0,0,0,,{text}\n'
        timing = block[0] + (' ' + positions['vtt_setting'] if place == 'Top' else '')
        out += '\n' + '\n'.join([timing, *block[1:]]) + '\n'
    (HERE / 'candidate02.ass').write_text(events_a)
    (HERE / 'candidate03.ass').write_text(events_b)
    (HERE / 'candidate02.en.vtt').write_text(out)
    table = {c['id']: c for c in cues}

    def at(frame):
        return next(c for c in cues if c['a'] <= frame / FPS < c['b'])

    samples = []
    for row in json.loads((P / 'logarithm-caption-overlay01/plan01.json').read_text())['samples']:
        c = at(row['frame'])
        assert c['id'] == row['cue'] and c['lines'] == row['lines'], row['id']
        samples.append({'origin': row['id'], 'frame': row['frame'], 'cue': c['id'], 'state': row['state']})
    for cue, why in EXTRA.items():
        c = table[cue]
        samples.append({'origin': 'added', 'frame': round((c['a'] + c['b']) / 2 * FPS), 'cue': cue, 'state': why})
    for row in positions['bottom_controls']:
        c = table[row['cue']]
        samples.append({'origin': 'control', 'frame': round((c['a'] + c['b']) / 2 * FPS), 'cue': row['cue'], 'state': row['kept']})
    samples.sort(key=lambda row: row['frame'])
    tiers = {cue: row['tier'] for row in positions['windows'] for cue in range(row['first'], row['last'] + 1)}
    for n, row in enumerate(samples, 1):
        c = table[row['cue']]
        row.update(id=f'F{n:02}', seconds=round(row['frame'] / FPS, 3), place='top' if c['id'] in top else 'bottom', tier=tiers.get(c['id'], 'control'), lines=c['lines'], cue_start=c['a'], cue_end=c['b'])
    assert all(s['cue_start'] <= s['seconds'] < s['cue_end'] for s in samples)
    bundle = HERE.relative_to(Path.cwd())
    plan = {'schema': 'logarithm-caption-safe-plan-v2', 'fps': FPS, 'master': MASTER, 'master_sha256': MASTER_SHA256, 'base_srt': srt, 'base_srt_sha256': sha(srt), 'base_vtt': vtt, 'base_vtt_sha256': sha(vtt),
            'captions': {key: {'path': str(bundle / name), 'sha256': sha(HERE / name)} for key, name in (('A', 'candidate02.ass'), ('B', 'candidate03.ass'))}, 'top_cues': len(top), 'samples': samples}
    (HERE / 'plan02.json').write_text(json.dumps(plan, indent=1, ensure_ascii=False) + '\n')
    print(json.dumps({'top_cues': len(top), 'windows': len(positions['windows']), 'segments_b': len(placements), 'samples': len(samples), 'a_sha256': plan['captions']['A']['sha256'], 'b_sha256': plan['captions']['B']['sha256'],
                      'vtt_sha256': sha(HERE / 'candidate02.en.vtt')}, indent=1))


if __name__ == '__main__':
    main()
