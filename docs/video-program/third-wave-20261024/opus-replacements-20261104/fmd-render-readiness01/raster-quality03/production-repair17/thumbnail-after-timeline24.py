from pathlib import Path
import json,hashlib,subprocess,time
R=Path('/data/projects/jmev-stroke-probe-20261106');Q=R/'qualification01/production-repair17'
def sha(p):
 with p.open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest()
for _ in range(2160):
 p=R/'jobs/csd-production-timeline21/resource-receipt.json'
 if p.exists():
  r=json.loads(p.read_text())
  if 'finished' in r:
   assert r['exit']==0 and r['inputs_sha256']==r['final_inputs_sha256'] and not r.get('resource_stop') and not r.get('stopped_for_resource_limit');break
 time.sleep(20)
else:raise RuntimeError('Timeline wait deadline; thumbnail not launched')
rows=json.loads((Q/'build19-inputs.json').read_text())['files'];seen={x['path']:x for x in rows}
for x in seen.values():assert sha(Path(x['path']))==x['sha256'],x['path']
extra=[Path(__file__),Q/'thumbnail24.py',Q/'thumbnail24.rs',Q/'before-thumbnail24.rs',R/'guard_csd02.py',R/'work/bin/production19',R/'jobs/csd-production-repair-build19/resource-receipt.json',p]+[p for p in (R/'work/production-repair19/workspace').rglob('*') if p.is_file()]
for p in extra:seen[str(p)]=dict(path=str(p),sha256=sha(p))
m=Q/'thumbnail24-inputs.json';assert not m.exists();assert not (R/'jobs/csd-production-thumbnail24').exists();m.write_text(json.dumps(dict(files=list(seen.values())),indent=2))
with (Q/'thumbnail24-guard.log').open('x') as out:subprocess.run(['/usr/bin/python3','-I','-B',str(R/'guard_csd02.py'),'--phase','native','--job','csd-production-thumbnail24','--inputs',str(m),'--','/usr/bin/timeout','7300','/usr/bin/python3','-I','-B',str(Q/'thumbnail24.py')],stdout=out,stderr=subprocess.STDOUT,check=True)
