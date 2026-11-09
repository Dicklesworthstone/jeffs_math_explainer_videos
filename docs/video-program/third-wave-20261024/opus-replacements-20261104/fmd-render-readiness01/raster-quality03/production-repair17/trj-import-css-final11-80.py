"""Guarded exact import; only creates the new foreign CSS root."""
from pathlib import Path
import json,hashlib,sys
R=Path('/data/projects/jmev-render-worker-20261107');Q=R/'qualification01/production19';J=R/'jobs/trj-import-css-final11-80'
H=Path('/data/projects/jmev-css-render-worker-20261107');DEST=R/'work/imported-css-root80';D=Path('/data/projects/jmev-stroke-probe-20261106')
def sha(p):
 with p.open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest()
r=json.loads((J/'resource-receipt.json').read_text());assert r['admitted'] and 'exit' not in r
assert r['policy']['stop_free_gib']==8 and r['policy']['phase_growth_gib']==4
assert not H.exists() and not H.is_symlink() and not DEST.exists()
m=json.loads((Q/'final11-export80.json').read_text());assert len(m['files'])==len({x['path'] for x in m['files']})
for x in m['files']:
 p=Path(x['path']);assert '..' not in p.parts and any(p.is_relative_to(root) for root in (H,D,R))
 if not p.is_relative_to(H):assert p.is_file() and sha(p)==x['sha256'] and p.stat().st_size==x['bytes'],str(p)
DEST.mkdir(mode=0o700);H.symlink_to(DEST,target_is_directory=True)
rows=[]
for x in m['files']:
 p=Path(x['path']);existing=p.exists();remaining=x['bytes'];h=hashlib.sha256();f=None
 if existing:assert sha(p)==x['sha256'],str(p)
 else:
  assert p.is_relative_to(H);p.parent.mkdir(mode=0o700,parents=True,exist_ok=True);f=p.open('xb')
 try:
  while remaining:
   b=sys.stdin.buffer.read(min(remaining,1024*1024));assert b,'Truncated stream';h.update(b);remaining-=len(b)
   if f:f.write(b)
 finally:
  if f:f.close()
 assert h.hexdigest()==x['sha256'] and sha(p)==x['sha256'],str(p)
 if not existing:p.chmod(x['mode'])
 rows.append(dict(path=str(p),sha256=x['sha256'],bytes=x['bytes'],existing_identical=existing))
assert not sys.stdin.buffer.read(1),'Trailing data'
for x in rows:assert sha(Path(x['path']))==x['sha256']
with (J/'import-result.json').open('x') as f:json.dump(dict(files=rows,source_job=m['source_job'],foreign_root=str(H),physical_root=str(DEST),scope='Exact import only; master replay and final AV still required.'),f,indent=2)
