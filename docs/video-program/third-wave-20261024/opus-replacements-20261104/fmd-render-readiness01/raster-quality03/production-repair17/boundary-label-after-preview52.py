from pathlib import Path
import json,hashlib,subprocess,time
R=Path('/data/projects/jmev-stroke-probe-20261106');Q=R/'qualification01/production-repair17';JOB='csd-boundary-label-build52'
def sha(p):
 with p.open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest()
def success(p):
 r=json.loads(p.read_text());assert r.get('finished') and r.get('exit')==0 and r['inputs_sha256']==r['final_inputs_sha256'];assert not any(r.get(k) for k in ('error','resource_stop','stopped_for_resource_limit'));return r
assert not (R/'jobs'/JOB).exists()
# Wait for existing preview consumer to leave the exclusive lane. A later
# picture-extraction failure does not invalidate completed native files;
# build is still guarded and does not consume preview as a qualification pass.
for _ in range(2160):
 p=R/'jobs/csd-production-preview21/resource-receipt.json'
 if p.exists() and json.loads(p.read_text()).get('finished'):break
 time.sleep(20)
else:raise RuntimeError('Existing preview wait deadline; no build launched')
paths={Path(__file__).resolve(),Q/'boundary-label-build52.py',Q/'boundary-label-repair52.rs',R/'guard_csd02.py'}
r=success(R/'jobs/csd-production-timeline21/resource-receipt.json');m=Q/'timeline21-inputs.json';assert sha(m)==r['inputs_sha256'];paths.update([m,R/'jobs/csd-production-timeline21/resource-receipt.json'])
for x in json.loads(m.read_text())['files']:
 p=Path(x['path']);assert sha(p)==x['sha256'],str(p);paths.add(p)
paths.update(p for p in (R/'work/production-repair19/workspace').rglob('*') if p.is_file())
m=Q/'boundary-label-build52-inputs.json'
with m.open('x') as f:json.dump({'files':[dict(path=str(p),sha256=sha(p)) for p in sorted(paths)]},f,indent=2)
subprocess.run(['/usr/bin/python3','-I','-B',str(R/'guard_csd02.py'),'--phase','native','--job',JOB,'--inputs',str(m),'--','/usr/bin/timeout','22000','/usr/bin/python3','-I','-B',str(Q/'boundary-label-build52.py')],check=True)
