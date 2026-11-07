from pathlib import Path
import json,hashlib,subprocess,time
R=Path('/data/projects/jmev-stroke-probe-20261106');Q=R/'qualification01/production-repair17';T=R/'work/production-repair17/workspace';B=R/'jobs/csd-production-repair-build17';J=R/'jobs/csd-production-assembly17'
def sha(p):
 with p.open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest()
assert not J.exists(),'Do not duplicate assembly'
for _ in range(2160):
 p=B/'resource-receipt.json'
 if p.exists():
  d=json.loads(p.read_text())
  if 'finished' in d:
   assert d.get('exit')==0 and d['inputs_sha256']==d['final_inputs_sha256'] and not d.get('stopped_for_resource_limit'),d
   break
 time.sleep(20)
else:raise RuntimeError('Build wait deadline; no assembly launched')
paths={Path(__file__),Q/'install-and-assemble17.py',Q/'prepare-assembly17.py',Q/'coherent_narration.py',Q/'timing-selected17.json',R/'guard_csd02.py',B/'resource-receipt.json',B/'qualification.json',B/'source-copy.json',R/'work/bin/production17'}
for m in [Q/'build17-inputs.json',R/'qualification01/continuous-quality16/rebind16-inputs.json']:
 paths.add(m)
 for x in json.loads(m.read_text())['files']:
  p=Path(x['path']);assert sha(p)==x['sha256'],str(p);paths.add(p)
for folder in [R/'jobs/csd-continuous-rebind16/narration',Q/'audio17']:
 paths.update(p for p in folder.rglob('*') if p.is_file())
mutable={'tools/coherent_narration.py','explainer_corpus/complex_analysis_visual/narration_units.json'}
for n,h in json.loads((B/'source-copy.json').read_text())['files'].items():
 p=T/n;assert sha(p)==h,str(p)
 if n not in mutable:paths.add(p)
m=Q/'assembly17-inputs.json'
with m.open('x') as f:json.dump({'files':[{'path':str(p),'sha256':sha(p)} for p in sorted(paths)]},f,indent=2)
subprocess.run(['/usr/bin/python3','-I','-B',str(R/'guard_csd02.py'),'--phase','native','--job',J.name,'--inputs',str(m),'--','/usr/bin/timeout','7200','/usr/bin/python3','-I','-B',str(Q/'install-and-assemble17.py')],check=True)
