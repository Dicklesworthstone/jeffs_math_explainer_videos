"""Clock times and seconds for the spots classified by reading c1-c5.jpg. Cue times are Bronze's final SRT."""
import json
import re
import sys
from pathlib import Path

OUT = Path(sys.argv[1])
SRT = Path(sys.argv[2])


def stamp(t):
    h, m, s = t.replace(',', '.').split(':')
    return int(h) * 3600 + int(m) * 60 + float(s)


cues = {}
for block in re.split(r'\n\s*\n', SRT.read_text().strip()):
    lines = block.splitlines()
    a, b = lines[1].split(' --> ')
    cues[int(lines[0])] = (stamp(a), stamp(b))
zero = {r['id']: r['count'] for r in json.loads((OUT / 'measured-0.json').read_text())}
lower = {r['id']: r['count'] for r in json.loads((OUT / 'measured-120.json').read_text())}


def clock(t):
    return f'{int(t // 60):02}:{t % 60:04.1f}'


GROUPS = {
    'both': [(23, 25), (102, 103), (107, 111), (205, 214), (237, 240), (243, 243), (245, 245), (247, 248), (329, 331), (345, 347), (372, 373), (474, 475)],
    'measured': [(13, 14), (20, 22), (35, 36), (65, 65), (78, 78), (104, 106), (147, 149), (177, 178), (221, 229), (241, 242), (244, 244), (246, 246), (269, 271), (276, 279), (436, 437)],
    'lower': [(89, 90), (157, 162), (286, 288), (305, 310), (387, 391), (430, 433), (435, 437)],
    'diagram': [(51, 58), (95, 96), (150, 156), (295, 304), (333, 340), (439, 449)],
}
for name, spans in GROUPS.items():
    total = 0
    print('---', name)
    for first, last in spans:
        ids = [k for k in range(first, last + 1) if k in cues]
        seconds = sum(cues[k][1] - cues[k][0] for k in ids)
        total += seconds
        print(f'{clock(cues[first][0])}-{clock(cues[last][1])}\tcues {first}-{last}\t{seconds:.1f} s\tmeasured {max(zero.get(k, 0) for k in ids)}\tlower {max(lower.get(k, 0) for k in ids)}')
    print('total', round(total, 1))
done = {k for spans in GROUPS.values() for first, last in spans for k in range(first, last + 1)}
wide = {k for k, v in zero.items() if v >= 130} | {k for k, v in lower.items() if v >= 130}
print('130+ cues outside the named groups:', sorted(wide - done))
