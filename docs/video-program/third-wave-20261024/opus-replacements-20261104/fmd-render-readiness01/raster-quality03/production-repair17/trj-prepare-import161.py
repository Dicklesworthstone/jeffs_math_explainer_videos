from pathlib import Path
import json,hashlib
R=Path('/data/projects/jmev-render-worker-20261107');Q=R/'qualification01/production19';H=Path('/data/projects/jmev-hz2-render-worker-20261107');D=Path('/data/projects/jmev-stroke-probe-20261106')
def sha(p):
 with p.open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest()
assert not (R/'jobs/trj-import-hz2-final07-86').exists()
assert H.is_symlink() and H.resolve()==R/'work/imported-hz2-root56'
prior_path=R/'jobs/trj-import-hz2-final12-60/resource-receipt.json'
prior=json.loads(prior_path.read_text());assert prior.get('finished') and prior.get('exit')==0 and prior['inputs_sha256']==prior['final_inputs_sha256'] and not prior.get('resource_stop')
m=Q/'final07-export86.json';data=json.loads(m.read_text());paths={prior_path,m,Q/'trj-import-hz2-final07-161.py',R/'guard_trj01.py',Path(__file__).resolve()}
for x in data['files']:
 p=Path(x['path']);assert p.is_relative_to(H) or p.is_relative_to(D)
 if p.exists():assert sha(p)==x['sha256'];paths.add(p)
 elif p.is_relative_to(D):raise RuntimeError('Missing shared source: '+str(p))
with (Q/'import-hz2-final07-86-inputs.json').open('x') as f:json.dump({'files':[dict(path=str(p),sha256=sha(p)) for p in sorted(paths)]},f,indent=2)
print('Import manifest frozen; no receiver launched by this helper.')
