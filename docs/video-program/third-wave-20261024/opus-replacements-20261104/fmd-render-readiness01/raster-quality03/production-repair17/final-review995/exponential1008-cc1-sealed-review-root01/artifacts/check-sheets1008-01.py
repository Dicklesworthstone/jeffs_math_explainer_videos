"""Local, read-only: check copied pictures1008 sheets against the dumped records (path name, sha256) and print each
sheet's master frames with their chapter-local frames and seconds, for the viewing log.
usage: python3 -I -B check-sheets1008-01.py records1008.json SHEETDIR KEY [KEY ...]"""
import hashlib
import json
import sys
from pathlib import Path

dump = json.loads(Path(sys.argv[1]).read_text())
root = Path(sys.argv[2])
bad = 0
for key in sys.argv[3:]:
    rec = dump['chapters'][key]['record']
    off = rec['extract_source']['offset_frames']
    local = sorted(p.name for p in (root / key).iterdir())
    want = [Path(s['path']).name for s in rec['sheets']]
    if local != want:
        bad += 1
        print('FILES DIFFER', key, local, want)
    for s in rec['sheets']:
        p = root / key / Path(s['path']).name
        ok = p.is_file() and hashlib.sha256(p.read_bytes()).hexdigest() == s['sha256']
        bad += not ok
        print(f"{key} {p.name} sha {'ok' if ok else 'MISMATCH'} {s['sha256'][:8]} master frames {s['frames'][0]}..{s['frames'][-1]} "
              f"(local {s['frames'][0] - off}..{s['frames'][-1] - off}; {s['frames'][0] / 60:.3f}-{s['frames'][-1] / 60:.3f} s)")
print('mismatches', bad)
raise SystemExit(1 if bad else 0)
