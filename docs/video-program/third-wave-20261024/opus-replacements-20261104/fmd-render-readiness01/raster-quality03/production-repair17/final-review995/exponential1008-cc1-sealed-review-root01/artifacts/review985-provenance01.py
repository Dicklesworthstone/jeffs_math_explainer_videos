"""Local, read-only: a markdown provenance table of the 985 seeds and the root and independent final985 picture
reviews, per chapter: repo-relative path, sha256, reviewer identity, review time, extraction sha the review binds
(and whether it equals the seed file's sha), safe_to_master, concrete defect count, and frames/sheets viewed.
usage: python3 -I -B review985-provenance01.py P808"""
import hashlib
import json
import sys
from pathlib import Path

P = Path(sys.argv[1])
sha = lambda p: hashlib.sha256(p.read_bytes()).hexdigest()
print('| Chapter | File | sha256 | Reviewer | UTC | Binds seed | safe_to_master | Defects | Frames viewed | Sheets viewed |')
print('|---|---|---|---|---|---|---|---|---|---|')
for n in range(1, 16):
    sp = P / f'completed-pictures985-{n:02}.json'
    s_sha = sha(sp)
    seed = json.loads(sp.read_text())['chapters'][0]
    key, frames, nsheets = seed['chapter'], {x['frame'] for x in seed['pictures']}, len(seed['sheets'])
    print(f'| {key} | `{sp}` | `{s_sha}` | seed | | | | | {len(frames)} pictures | {nsheets} sheets |')
    for who in ('root', 'independent'):
        rp = P / f'{who}-final985-{n:02}-review.json'
        r = json.loads(rp.read_text())
        c = [x for x in r.get('chapters', []) if x.get('chapter') == key]
        c = c[0] if c else {}
        ext = r.get('extraction_sha256') or c.get('extraction_sha256')
        vf = set(c.get('viewed_frame_indices') or [])
        vs = set(c.get('viewed_sheets') or [])
        when = r.get('reviewed_utc') or r.get('utc') or ''
        print(f"| {key} | `{rp}` | `{sha(rp)}` | {r.get('reviewer_identity', '')} | {when[:19]} | {ext == s_sha} | "
              f"{r.get('safe_to_master')} | {len(r.get('concrete_defects') or [])} | {len(vf & frames)}/{len(frames)} | {len(vs)}/{nsheets} |")
