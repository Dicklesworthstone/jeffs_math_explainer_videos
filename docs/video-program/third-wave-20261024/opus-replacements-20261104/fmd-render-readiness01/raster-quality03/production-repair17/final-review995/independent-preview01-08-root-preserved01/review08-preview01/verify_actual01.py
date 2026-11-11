"""Verify the copied 08 extraction against its own record: every picture and sheet hash, the movie and
table fields, the selection against my reproduction (sheet_map02 rule), and the sheet layout. Local, read-only.
Usage: verify_actual01.py COPY_DIR (holding 08_loop_sums/) SHEET_MAP02_OUT"""
import hashlib, json, sys
from pathlib import Path
copy, mapped = Path(sys.argv[1])/'08_loop_sums', json.loads(Path(sys.argv[2]).read_text())
CSD = '/data/projects/jmev-stroke-probe-20261106/jobs/csd-native-combined-pictures01/08_loop_sums/'
def sha(p): return hashlib.sha256(Path(p).read_bytes()).hexdigest()
rec = json.loads((copy/'pictures.json').read_text())
out = dict(pictures_json_sha256=sha(copy/'pictures.json'), keys=sorted(rec))
out.update({k: rec.get(k) for k in ('movie', 'movie_sha256', 'frames', 'full_decode', 'layout_sha256',
                                     'retained_samples', 'sampling', 'tables')})
frames = rec['selected_frames']
out['selected'] = len(frames)
out['sorted_unique'] = frames == sorted(set(frames))
out['run_10200_10380_complete'] = set(range(10200, 10381)) <= set(frames)
out['anchors_in'] = [f in frames for f in (10788, 12336)]
out['layout_anchors'] = [a['frame'] for a in rec['layout_anchors']]
out['sheets'] = len(rec['sheets'])
out['matches_my_count'] = len(frames) == mapped['selected'] and len(rec['sheets']) == mapped['sheets']
bad = []
for item in rec['pictures'] + rec['sheets']:
    assert item['path'].startswith(CSD), item['path']
    local = copy/item['path'][len(CSD):]
    if sha(local) != item['sha256']: bad.append(item['path'])
out['hash_mismatches'] = bad
out['pictures_listed'] = len(rec['pictures'])
out['local_picture_files'] = len(list((copy/'pictures').glob('*.jpg')))
out['local_sheet_files'] = len(list((copy/'sheets').glob('*.jpg')))
out['sheet_ranges_ok'] = all(s['first_picture'] == 8*i+1 and s['last_picture'] == min(8*i+8, len(frames))
                             for i, s in enumerate(rec['sheets']))
print(json.dumps(out, indent=1))
