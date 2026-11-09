from pathlib import Path
import json,hashlib
R=Path('/data/projects/jmev-render-worker-20261107');Q=R/'qualification01/production19';H=Path('/data/projects/jmev-hz2-render-worker-20261107');D=Path('/data/projects/jmev-stroke-probe-20261106')
def sha(p):
 with p.open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest()
assert not (R/'jobs/trj-import-hz2-final15-56').exists()
assert not H.exists() and not H.is_symlink()
m=Q/'final15-export56.json';data=json.loads(m.read_text());paths={m,Q/'trj-import-hz2-final15-56.py',R/'guard_trj01.py',Path(__file__).resolve()}
for x in data['files']:
 p=Path(x['path']);assert p.is_relative_to(H) or p.is_relative_to(D)
 if p.is_relative_to(D):assert sha(p)==x['sha256'];paths.add(p)
with (Q/'import-hz2-final15-56-inputs.json').open('x') as f:json.dump({'files':[dict(path=str(p),sha256=sha(p)) for p in sorted(paths)]},f,indent=2)
print('Import manifest frozen; no receiver launched by this helper.')
