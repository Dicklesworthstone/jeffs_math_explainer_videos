"""Read-only: per chapter, what the root and independent final985 reviews covered (frames or sheets viewed versus
the seed's pictures and sheets), their safe_to_master value and concrete defects, and whether each bound the seed
record sha256 that verify1005's inputs bind.
usage: python3 -I -B review985-coverage01.py P808 VM_ROWS_TXT"""
import json
import sys
from pathlib import Path

P = Path(sys.argv[1])
bound = {}
for line in Path(sys.argv[2]).read_text().splitlines():
    s, p = line.split(' ', 1)
    if p.endswith('pictures985.json'):
        bound[p.split('css-exponential-pictures985-')[1].split('/')[0]] = s
for n in range(1, 16):
    seed = json.loads((P / f'completed-pictures985-{n:02}.json').read_text())['chapters'][0]
    key, frames, nsheets = seed['chapter'], {x['frame'] for x in seed['pictures']}, len(seed['sheets'])
    for who in ('root', 'independent'):
        r = json.loads((P / f'{who}-final985-{n:02}-review.json').read_text())
        c = [x for x in r.get('chapters', []) if x.get('chapter') == key]
        c = c[0] if c else {}
        vf = set(c.get('viewed_frame_indices') or [])
        vs = c.get('viewed_sheets') or []
        ext = r.get('extraction_sha256') or c.get('extraction_sha256')
        defects = r.get('concrete_defects') or []
        print(f"{key:22} {who:11} safe_to_master {str(r.get('safe_to_master')):5} seed-bound {ext == bound.get(key)} "
              f"frames {len(vf & frames)}/{len(frames)} sheets {len(set(vs))}/{nsheets} defects {len(defects)}"
              + ''.join(f"\n      - {json.dumps(d)[:260]}" for d in defects))
