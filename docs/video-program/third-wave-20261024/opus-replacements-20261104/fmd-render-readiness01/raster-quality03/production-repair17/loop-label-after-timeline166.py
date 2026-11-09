from pathlib import Path
import json,hashlib,subprocess,time
R=Path('/data/projects/jmev-stroke-probe-20261106');Q=R/'qualification01/production-repair17';PRE=R/'jobs/csd-two-label-timeline165'
def sha(p):
 with p.open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest()
for _ in range(2880):
 p=PRE/'resource-receipt.json'
 if p.exists() and json.loads(p.read_text()).get('finished'):break
 # Fail promptly on terminal failed prerequisites; never create a duplicate.
 for name in ('csd-two-label-build164','csd-two-label-rebind165'):
  x=R/'jobs'/name/'resource-receipt.json'
  if x.exists():
   a=json.loads(x.read_text())
   if a.get('finished'):assert a.get('exit')==0,(name,a.get('exit'))
 time.sleep(20)
else:raise RuntimeError('Timeline wait deadline')
a=json.loads(p.read_text());assert a.get('exit')==0 and a['inputs_sha256']==a['final_inputs_sha256'] and not any(a.get(k) for k in ('error','resource_stop','stopped_for_resource_limit'))
m=Q/'two-label-timeline165-inputs.json';assert sha(m)==a['inputs_sha256'];paths={m,p,Path(__file__).resolve(),Q/'loop-label-preview166.py',R/'guard_csd02.py'}
for x in json.loads(m.read_text())['files']:
 f=Path(x['path']);assert sha(f)==x['sha256'];paths.add(f)
paths.update(x for x in PRE.rglob('*') if x.is_file())
assert not (R/'jobs/csd-loop-label-preview166').exists()
m=Q/'loop-label-preview166-inputs.json'
with m.open('x') as f:json.dump({'files':[dict(path=str(p),sha256=sha(p)) for p in sorted(paths)]},f,indent=2)
subprocess.run(['/usr/bin/python3','-I','-B',str(R/'guard_csd02.py'),'--phase','native','--job','csd-loop-label-preview166','--inputs',str(m),'--','/usr/bin/timeout','173000','/usr/bin/python3','-I','-B',str(Q/'loop-label-preview166.py')],check=True)
print('Preview produced; actual review still required, no automatic acceptance.')
