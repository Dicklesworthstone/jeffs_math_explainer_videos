"""Local, read-only: around one chapter-local frame, list the runs of decoded-frame change and hold in verify1005's
framehash of that chapter (a hash differing from the previous frame's = the decoded picture changed at that frame).
Also prints the selected 985 and 1008 pictures near it and the cue markers from the 985 reasons.
Hash change says pixels changed; it says nothing about how the motion looks.
usage: python3 -I -B frame-runs02.py FRAMEHASH SEED985_JSON RECORDS1008_JSON KEY LOCAL_FRAME HALF_WINDOW"""
import json
import sys
from pathlib import Path

fh, seed, recs, key, local, half = sys.argv[1], sys.argv[2], sys.argv[3], sys.argv[4], int(sys.argv[5]), int(sys.argv[6])
hashes = []
for line in Path(fh).read_text().splitlines():
    if line.startswith('#'):
        continue
    parts = [x.strip() for x in line.split(',')]
    assert int(parts[2]) == len(hashes), ('pts is not the frame index', line)
    hashes.append(parts[5])
s = json.loads(Path(seed).read_text())['chapters'][0]
assert s['chapter'] == key and s['frames'] == len(hashes), (s['frames'], len(hashes))
r = json.loads(Path(recs).read_text())['chapters'][key]['record']
off = r['extract_source']['offset_frames']
print(f'{key}: {len(hashes)} decoded frames in the framehash; offset {off}; local {local} = master {local + off} = {(local + off) / 60:.3f} s master, {local / 60:.3f} s chapter')

# Runs of change / hold over the window.
lo, hi = max(1, local - half), min(len(hashes) - 1, local + half)
runs, start, kind = [], lo, hashes[lo] != hashes[lo - 1]
for i in range(lo + 1, hi + 1):
    k = hashes[i] != hashes[i - 1]
    if k != kind:
        runs.append((kind, start, i - 1))
        start, kind = i, k
runs.append((kind, start, hi))
for kind, a, b in runs:
    mark = '  <== contains the frame' if a <= local <= b else ''
    print(f"  {'CHANGE' if kind else 'hold  '} local {a}-{b} ({b - a + 1} frames, {(b - a + 1) / 60:.3f} s), master {a + off}-{b + off}, "
          f"{(a + off) / 60:.3f}-{(b + off + 1) / 60:.3f} s{mark}")
# The full change run containing the frame (searching beyond the window if needed).
if hashes[local] != hashes[local - 1] or (local + 1 < len(hashes) and hashes[local + 1] != hashes[local]):
    a = local
    while a > 1 and hashes[a] != hashes[a - 1]:
        a -= 1
    b = local
    while b + 1 < len(hashes) and hashes[b + 1] != hashes[b]:
        b += 1
    print(f'  full change run around it: local {a}-{b} = {b - a + 1} changing frames ({(b - a + 1) / 60:.3f} s), master {a + off}-{b + off}')
    distinct = len(set(hashes[a - 1:b + 1]))
    print(f'  distinct decoded pictures from the hold before through the run: {distinct}')
else:
    print('  the frame sits inside a hold (identical to both neighbours)')

# Selected pictures and cue markers nearby.
for i, (p, q) in enumerate(zip(s['pictures'], r['pictures'])):
    if abs(p['frame'] - local) <= half:
        print(f"  picture {i + 1:3} local {p['frame']:5} master {q['frame']:6} sheet {i // 8 + 1:03} 985 {p['sha256'][:8]} 1008 {q['sha256'][:8]} "
              f"{'same' if p['sha256'] == q['sha256'] else 'DIFF'} reasons {p['reasons']}")
