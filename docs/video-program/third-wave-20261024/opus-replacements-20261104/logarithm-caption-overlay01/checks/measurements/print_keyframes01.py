"""Group the keyframe map into stretches with ink in the footer rows, with the cue text beside them."""
import json
import re
import sys

rows = json.load(open(sys.argv[1]))
srt = open('narration_lab/third-wave-20261024/root/elevenlabs-logarithm-rebuild-20261104/remote-mac-mini-max01/jobs/logarithm-remote-render-work01/logarithm_mysteries.en.srt').read()
print('gaps max', max(b['t'] - a['t'] for a, b in zip(rows, rows[1:])), 'count', len(rows))
TOP, SCALE = 1830, 4


def band(row, low, high):
    return sum(row['rows'][(low - TOP) // SCALE:(high - TOP) // SCALE])


def stretches(low, high, floor):
    hits = [r['t'] for r in rows if band(r, low, high) >= floor]
    groups = []
    for t in hits:
        if groups and t - groups[-1][1] <= 9:
            groups[-1][1] = t
        else:
            groups.append([t, t])
    return groups


for name, low, high, floor in (('-3.62 row (px 2030-2085)', 2030, 2085, 25), ('-3.40 row (px 1972-2026)', 1972, 2026, 25),
                               ('-3.05..-3.25 rows (px 1900-1970)', 1900, 1970, 25)):
    print(name)
    for a, b in stretches(low, high, floor):
        print(f'   {a:9.3f} .. {b:9.3f}')
