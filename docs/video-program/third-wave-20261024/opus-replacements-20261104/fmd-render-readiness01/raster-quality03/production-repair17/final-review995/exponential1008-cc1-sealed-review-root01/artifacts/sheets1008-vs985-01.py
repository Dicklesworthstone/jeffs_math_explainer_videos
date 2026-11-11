"""Local, read-only: for each finished pictures1008 chapter in a records dump, compare its contact sheets with the
985 seed's sheets: same count, same chapter-local frame list per sheet, and whether the sheet JPEG bytes are equal.
Sheets are composed images (tiles plus frame labels), so equal pictures do not imply equal sheet bytes.
usage: python3 -I -B sheets1008-vs985-01.py P808 records1008.json"""
import json
import sys
from pathlib import Path

P, dump = Path(sys.argv[1]), json.loads(Path(sys.argv[2]).read_text())
for n in range(1, 16):
    seed = json.loads((P / f'completed-pictures985-{n:02}.json').read_text())['chapters'][0]
    key = seed['chapter']
    rec = (dump['chapters'].get(key) or {}).get('record')
    if not rec:
        print(f'{key:22} not finished in this dump')
        continue
    off = rec['extract_source']['offset_frames']
    a, b = seed['sheets'], rec['sheets']
    same_frames = sum(1 for x, y in zip(a, b) if x['frames'] == [f - off for f in y['frames']])
    same_bytes = sum(1 for x, y in zip(a, b) if x['sha256'] == y['sha256'])
    print(f'{key:22} sheets 985 {len(a)} 1008 {len(b)} same local frame lists {same_frames}/{len(a)} same sheet bytes {same_bytes}/{len(a)}')
