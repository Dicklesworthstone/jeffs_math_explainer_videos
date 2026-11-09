"""Read-only export inventory for terminal HZ2 chapter15; never exports a running job."""
from pathlib import Path
import json,hashlib,sys
H=Path('/data/projects/jmev-hz2-render-worker-20261107');Q=H/'qualification01/production19';J=H/'jobs/hz2-production19-final-10-rigidity85';M=Q/'final10-85-inputs.json'
def sha(p):
 with p.open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest()
r=json.loads((J/'resource-receipt.json').read_text());assert r.get('finished') and r.get('exit')==0 and r['inputs_sha256']==r['final_inputs_sha256']==sha(M);assert not any(r.get(k) for k in ('error','resource_stop','stopped_for_resource_limit'))
paths={M,J/'resource-receipt.json',J/'render-summary.json',Path(__file__).resolve()}
for x in json.loads(M.read_text())['files']:
 p=Path(x['path']);assert sha(p)==x['sha256'];paths.add(p)
s=json.loads((J/'render-summary.json').read_text());assert s['chapter']=='10_rigidity' and s['frames']==7442
for x in s['files']:
 p=Path(x['path']);assert p.is_relative_to(J/'render') and sha(p)==x['sha256'];paths.add(p)
for p in J.rglob('*'):
 if p.is_file():paths.add(p)
rows=[]
for p in sorted(paths):
 assert p.is_absolute() and '..' not in p.parts and str(p).startswith((str(H)+'/', '/data/projects/jmev-stroke-probe-20261106/'))
 rows.append(dict(path=str(p),sha256=sha(p),bytes=p.stat().st_size,mode=p.stat().st_mode & 0o777))
assert sha(M)==r['inputs_sha256']
print(json.dumps(dict(source_job=str(J),receipt_sha256=sha(J/'resource-receipt.json'),files=rows,bytes=sum(x['bytes'] for x in rows),scope='Terminal source inventory; transfer and master replay not yet verified.'),indent=2))
