from pathlib import Path
import json,hashlib,subprocess,time
R=Path('/data/projects/jmev-stroke-probe-20261106');Q=R/'qualification01/production-repair17';JOB='csd-review-repair-build238'
def sha(p):
 with p.open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest()
def success(p):
 r=json.loads(p.read_text());assert r.get('finished') and r.get('exit')==0 and r['inputs_sha256']==r['final_inputs_sha256'];assert not any(r.get(k) for k in ('error','resource_stop','stopped_for_resource_limit'));return r
assert not (R/'jobs'/JOB).exists()
p=R/'jobs/csd-two-label-build164/resource-receipt.json';success(p)
paths={Path(__file__).resolve(),Q/'review-repair-build238.py',Q/'boundary-label-repair52.rs',Q/'loop-sums-review-repair238.rs',Q/'one-derivative-review-repair238.rs',Q/'mod-review-repair236.rs',R/'guard_csd02.py',p}
r=success(R/'jobs/csd-production-timeline21/resource-receipt.json');m=Q/'timeline21-inputs.json';assert sha(m)==r['inputs_sha256'];paths.update([m,R/'jobs/csd-production-timeline21/resource-receipt.json'])
for x in json.loads(m.read_text())['files']:
 p=Path(x['path']);assert sha(p)==x['sha256'],str(p);paths.add(p)

for n,h in {'one-derivative-review-repair238.rs': '7a4cc01acb497ad881c0d8478c696ab74db025494ab9381d0b414355fe3fd30d', 'loop-sums-review-repair238.rs': '8a13e8a190cd502e7b58efbcb7c40872cdb0e6e702feb69e1b789e451a7a42b5'}.items():assert sha(Q/n)==h,n
paths.update(p for p in (R/'work/production-repair19/workspace').rglob('*') if p.is_file())
m=Q/'review-repair-build238-inputs.json'
with m.open('x') as f:json.dump({'files':[dict(path=str(p),sha256=sha(p)) for p in sorted(paths)]},f,indent=2)
subprocess.run(['/usr/bin/python3','-I','-B',str(R/'guard_csd02.py'),'--phase','native','--job',JOB,'--inputs',str(m),'--','/usr/bin/timeout','22000','/usr/bin/python3','-I','-B',str(Q/'review-repair-build238.py')],check=True)
