"""Verify the copied 01 extraction against its own record: every picture and sheet hash, the movie and table
fields, the selection's runs and the sheet layout. Local, read-only. Usage: verify_actual01.py COPY_DIR"""
import hashlib, json, sys
from pathlib import Path
copy = Path(sys.argv[1])/'01_the_bump'
CSD = '/data/projects/jmev-stroke-probe-20261106/jobs/csd-native-combined-pictures01/01_the_bump/'
def sha(p): return hashlib.sha256(Path(p).read_bytes()).hexdigest()
rec = json.loads((copy/'pictures.json').read_text())
out = dict(pictures_json_sha256=sha(copy/'pictures.json'), keys=sorted(rec))
for k in sorted(rec):
    if k not in ('pictures', 'sheets', 'selected_frames'):
        out[k] = rec[k]
frames = rec['selected_frames']
out['selected'] = len(frames)
out['sorted_unique'] = frames == sorted(set(frames))
runs, start = [], frames[0]
for a, b in zip(frames, frames[1:] + [None]):
    if b != a + 1:
        runs.append([start, a]); start = b
out['runs'] = runs
out['sheets'] = len(rec['sheets'])
bad = []
for item in rec['pictures'] + rec['sheets']:
    assert item['path'].startswith(CSD), item['path']
    if sha(copy/item['path'][len(CSD):]) != item['sha256']: bad.append(item['path'])
out['hash_mismatches'] = bad
out['pictures_listed'] = len(rec['pictures'])
out['local_picture_files'] = len(list((copy/'pictures').glob('*.jpg')))
out['local_sheet_files'] = len(list((copy/'sheets').glob('*.jpg')))
out['sheet_ranges_ok'] = all(s['first_picture'] == 8*i+1 and s['last_picture'] == min(8*i+8, len(frames))
                             for i, s in enumerate(rec['sheets']))
out['picture_frames_match'] = [p.get('frame') for p in rec['pictures']] == frames if 'frame' in rec['pictures'][0] else 'no frame key'
print(json.dumps(out, indent=1))
