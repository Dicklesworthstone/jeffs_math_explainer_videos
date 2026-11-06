#!/usr/bin/env python3
"""Write plan01.json from the real SRT and the bounds measured on the master (see measurements/).

Run from the repository root on a host that holds the delivery copy. It reads only the SRT.
Each sample is one cue beside one footer state, on a frame inside that overlap where the footer is
fully shown. Stretches are (first change, fully shown, fade starts, last visible), in seconds.
"""
import hashlib
import json
from pathlib import Path
import re

HERE = Path(__file__).resolve().parent
O = Path('narration_lab/third-wave-20261024/root/elevenlabs-logarithm-rebuild-20261104/remote-mac-mini-max01')
MASTER = O / 'jobs/logarithm-remote-render-work01/logarithm_mysteries_master.mp4'
SRT = O / 'jobs/logarithm-remote-render-work01/logarithm_mysteries.en.srt'
GUARD = Path('docs/video-program/third-wave-20261024/opus-replacements-20261104/engine-secondary-validation01/guard_owner56g_target20g.py')
GUARD_SOURCE = Path('narration_lab/production-20261020/second-wave-central/duration-and-storage-20261020/root/run_bounded_phase_oliver_owner48g_target14g_20261102.py')
MASTER_SHA256 = '8534ec211790fb8400b809de19a6d1cb75af6150a5e90566f6a93bb90e7efd69'
SRT_SHA256 = 'c9321cd7a2912d4457f2dc617735fd06f0b2fc7d684442815fa46b989bc31b7c'
FPS, FRAMES = 60, 116665
ROWS = {-3.62: 2057, -3.40: 1998, -2.93: 1871}

# Measured to the frame in measurements/rows01.json. A cut has no fade, so its last two values match.
FOOTERS = [
    {'name': 'F1', 'what': 'chapter 10 textbook and source rows', 'row': -3.40, 'mail_window': [1484.42, 1497.62],
     'span': [1483.950, 1484.300, 1497.617, 1497.617], 'states': [[1484.300, 1497.617, 'both rows']]},
    {'name': 'F2', 'what': 'chapter 11 Liouville footer', 'row': -3.40, 'mail_window': [1547.63, 1566.10],
     'span': [1547.367, 1547.700, 1565.700, 1566.067], 'states': [[1547.700, 1565.700, 'footer']]},
    {'name': 'F3', 'what': 'chapter 11 Culver footer', 'row': -3.40, 'mail_window': [1720, 1751],
     'span': [1719.567, 1719.850, 1751.533, 1751.883], 'states': [[1719.850, 1751.533, 'footer']]},
    {'name': 'F4', 'what': 'chapter 11 Gallier footer', 'row': -3.40, 'mail_window': [1766, 1773],
     'span': [1762.267, 1762.600, 1773.450, 1773.450],
     'states': [[1762.600, 1766.483, 'footer'], [1766.800, 1773.450, 'footer, red lines above it']]},
    {'name': 'F5', 'what': 'chapter 12 closing footer', 'row': -3.40, 'mail_window': [1930, 1944],
     'span': [1929.667, 1929.933, 1943.800, 1944.167],
     'states': [[1929.933, 1931.267, 'footer'], [1931.533, 1934.217, 'footer, convention line'],
                [1934.483, 1943.800, 'footer, convention line, family label']]},
]
# Same fine print one row lower, in five other chapters, and in no window of Mail 5673. Found in the source and
# the keyframe map, so these are coarser: one sample each, on the widest caption inside the seen stretch.
ADDED = [('F6', 'chapter 00 fine print', [91.667, 100.0], 23), ('F7', 'chapter 02 fine print', [347.533, 351.7], 90),
         ('F8', 'chapter 04 fine print', [605.933, 622.6], 160), ('F9', 'chapter 07 fine print', [1141.05, 1149.383], 287),
         ('F10', 'chapter 08 fine print', [1223.25, 1235.75], 306)]


def stamp(text):
    hours, minutes, seconds = text.replace(',', '.').split(':')
    return int(hours) * 3600 + int(minutes) * 60 + float(seconds)


def cues():
    rows = {}
    for block in re.split(r'\n\s*\n', SRT.read_text().strip()):
        lines = block.splitlines()
        start, end = lines[1].split(' --> ')
        rows[int(lines[0])] = {'cue': int(lines[0]), 'start': stamp(start), 'end': stamp(end), 'lines': lines[2:]}
    return rows


def sample(group, cue, low, high, state, row):
    frame = round((low + high) / 2 * FPS)
    return {'group': group, 'frame': frame, 'seconds': round(frame / FPS, 3), 'cue': cue['cue'], 'lines': cue['lines'],
            'cue_start': cue['start'], 'cue_end': cue['end'], 'overlap': [round(low, 3), round(high, 3)], 'state': state, 'row': row,
            'row_px': ROWS[row]}


def main():
    assert hashlib.sha256(SRT.read_bytes()).hexdigest() == SRT_SHA256
    table = cues()
    assert len(table) == 475
    samples, coverage = [], []
    for footer in FOOTERS:
        first, _, _, last = footer['span']
        met = [cue for cue in table.values() if cue['start'] < last and cue['end'] > first]
        coverage.append({'footer': footer['name'], 'what': footer['what'], 'mail_5673_window': footer['mail_window'],
                         'measured': dict(zip(('first_change', 'fully_shown', 'fade_starts', 'last_visible'), footer['span'])),
                         'cues': [cue['cue'] for cue in met],
                         'extended_window': [round(min(first, met[0]['start']) - 0.5, 3), round(max(last, met[-1]['end']) + 0.5, 3)]})
        for cue in met:
            found = False
            for low, high, state in footer['states']:
                low, high = max(low, cue['start']), min(high, cue['end'])
                if high - low >= 1 / FPS:
                    samples.append(sample(footer['name'], cue, low, high, state, footer['row']))
                    found = True
            assert found, (footer['name'], cue['cue'])
    for name, what, seen, number in ADDED:
        cue = table[number]
        assert seen[0] <= cue['start'] and cue['end'] <= seen[1] + 4.2 or seen[0] - 0.7 <= cue['start']
        low, high = max(seen[0], cue['start']), min(seen[1], cue['end'])
        samples.append(sample(name, cue, low, high, 'fine print, widest caption seen beside it', -3.62))
        coverage.append({'footer': name, 'what': what, 'mail_5673_window': None, 'seen_on_keyframes': seen, 'cues': [number]})
    for number, row in enumerate(sorted(samples, key=lambda row: row['frame']), 1):
        row['id'] = f'S{number:02d}'
    samples.sort(key=lambda row: row['frame'])
    assert all(0 <= row['frame'] < FRAMES for row in samples)
    plan = {'schema': 'logarithm-caption-overlay-plan-v1', 'master': str(MASTER), 'master_sha256': MASTER_SHA256, 'srt': str(SRT),
            'srt_sha256': SRT_SHA256, 'fps': FPS, 'frames': FRAMES, 'coverage': coverage, 'samples': samples}
    (HERE.parent / 'plan01.json').write_text(json.dumps(plan, indent=1, ensure_ascii=False) + '\n')
    rows = []
    for path in (GUARD, GUARD_SOURCE, MASTER, SRT):
        data = path.read_bytes()
        rows.append({'path': str(path), 'sha256': hashlib.sha256(data).hexdigest(), 'bytes': len(data)})
    assert rows[2]['sha256'] == MASTER_SHA256
    transfer = {'note': 'Paths are the same under the repository root here and the css staging root. Send the whole bundle folder too.', 'files': rows}
    (HERE.parent / 'transfer-files01.json').write_text(json.dumps(transfer, indent=1) + '\n')
    print(len(samples), 'samples')
    for row in samples:
        print(row['id'], row['group'], row['frame'], row['seconds'], 'cue', row['cue'], row['state'], '/'.join(row['lines'])[:60])


if __name__ == '__main__':
    main()
