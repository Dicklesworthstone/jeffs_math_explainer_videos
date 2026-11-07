from pathlib import Path
import json,hashlib,subprocess,time
R=Path('/data/projects/jmev-stroke-probe-20261106');Q=R/'qualification01/production-repair17'
def sha(p):
 with p.open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest()
def success(p):
 d=json.loads(p.read_text());assert d.get('exit')==0 and d['inputs_sha256']==d['final_inputs_sha256'] and not d.get('error') and not d.get('stopped_for_resource_limit'),str(p);return d
for _ in range(2160):
 p=R/'jobs/csd-production-assembly18/resource-receipt.json'
 if p.exists() and 'finished' in json.loads(p.read_text()):success(p);break
 bp=R/'jobs/csd-production-repair-build18/resource-receipt.json'
 if bp.exists() and 'finished' in json.loads(bp.read_text()):success(bp)
 time.sleep(20)
else:raise RuntimeError('Assembly wait deadline')
subprocess.run(['/usr/bin/python3','-I','-B',str(Q/'launch-preview18.py'),'--phase','timeline'],check=True)
d=success(R/'jobs/csd-production-timeline18/resource-receipt.json');paths={Path(__file__),Q/'score18.py',R/'guard_csd02.py'}
m=Q/'timeline18-inputs.json';assert sha(m)==d['inputs_sha256'];paths.add(m)
for x in json.loads(m.read_text())['files']:
 p=Path(x['path']);assert sha(p)==x['sha256'];paths.add(p)
paths.update(p for p in (R/'jobs/csd-production-timeline18').rglob('*') if p.is_file())
f=Q/'score18-inputs.json'
with f.open('x') as s:json.dump({'files':[dict(path=str(p),sha256=sha(p)) for p in sorted(paths)]},s,indent=2)
subprocess.run(['/usr/bin/python3','-I','-B',str(R/'guard_csd02.py'),'--phase','native','--job','csd-production-score18','--inputs',str(f),'--','/usr/bin/timeout','7300','/usr/bin/python3','-I','-B',str(Q/'score18.py')],check=True)
success(R/'jobs/csd-production-score18/resource-receipt.json')
subprocess.run(['/usr/bin/python3','-I','-B',str(Q/'launch-preview18.py'),'--phase','preview'],check=True)
print('Native preview completed; actual picture review required before any upload')
