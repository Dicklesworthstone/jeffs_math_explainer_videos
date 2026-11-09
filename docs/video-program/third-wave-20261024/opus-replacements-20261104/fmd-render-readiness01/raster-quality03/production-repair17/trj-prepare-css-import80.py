from pathlib import Path
import json,hashlib
R=Path('/data/projects/jmev-render-worker-20261107');Q=R/'qualification01/production19';H=Path('/data/projects/jmev-css-render-worker-20261107');D=Path('/data/projects/jmev-stroke-probe-20261106')
def sha(p):
 with p.open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest()
assert not (R/'jobs/trj-import-css-final11-80').exists()
assert not H.exists() and not H.is_symlink()
m=Q/'final11-export80.json';data=json.loads(m.read_text());paths={m,Q/'trj-import-css-final11-80.py',R/'guard_trj01.py',Path(__file__).resolve()}
for x in data['files']:
 p=Path(x['path']);assert '..' not in p.parts and any(p.is_relative_to(root) for root in (H,D,R))
 if not p.is_relative_to(H):assert sha(p)==x['sha256'];paths.add(p)
with (Q/'import-css-final11-80-inputs.json').open('x') as f:json.dump({'files':[dict(path=str(p),sha256=sha(p)) for p in sorted(paths)]},f,indent=2)
