from pathlib import Path
import json,hashlib,subprocess,time
R=Path('/data/projects/jmev-stroke-probe-20261106');Q=R/'qualification01/production-repair17';TREE=R/'work/review-repair238/workspace'
def sha(p):
 with p.open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest()
def success(job):
 r=json.loads((R/'jobs'/job/'resource-receipt.json').read_text());assert r.get('finished') and r.get('exit')==0 and r['inputs_sha256']==r['final_inputs_sha256'];assert not any(r.get(k) for k in ('error','resource_stop','stopped_for_resource_limit'));return r
def freeze_launch(job,manifest,worker,prior,exclude=()):
 assert not (R/'jobs'/job).exists()
 paths={Path(__file__).resolve(),Q/worker,R/'guard_csd02.py',R/'work/bin/review-repair238',Q/'prepare-assembly19.py'}
 for name,m in prior:
  r=success(name);m=Q/m;assert sha(m)==r['inputs_sha256'];paths.add(m)
  for row in json.loads(m.read_text())['files']:
   p=Path(row['path']);assert sha(p)==row['sha256'],str(p);paths.add(p)
  paths.update(p for p in (R/'jobs'/name).rglob('*') if p.is_file())
 paths.update(p for p in TREE.rglob('*') if p.is_file() and str(p.relative_to(TREE)) not in exclude)
 paths.update(p for p in (R/'jobs/csd-production-assembly19/narration').rglob('*') if p.is_file())
 manifest=Q/manifest
 with manifest.open('x') as f:json.dump({'files':[dict(path=str(p),sha256=sha(p)) for p in sorted(paths)]},f,indent=2)
 command=['/usr/bin/python3','-I','-B',str(R/'guard_csd02.py'),'--phase','native','--job',job,'--inputs',str(manifest),'--','/usr/bin/timeout','172900','/usr/bin/python3','-I','-B',str(Q/worker)]
 if worker=='review-repair-rebind242.py':command.append('--run')
 subprocess.run(command,check=True);success(job)
for _ in range(2160):
 p=R/'jobs/csd-review-repair-build238/resource-receipt.json'
 if p.exists() and json.loads(p.read_text()).get('finished'):break
 time.sleep(20)
else:raise RuntimeError('Build wait deadline; no dependent job launched')
success('csd-review-repair-build238')
freeze_launch('csd-review-repair-rebind242','review-repair-rebind242-inputs.json','review-repair-rebind242.py',[('csd-review-repair-build238','review-repair-build238-inputs.json')],('explainer_corpus/complex_analysis_visual/narration_units.json',))
freeze_launch('csd-review-repair-timeline242','review-repair-timeline242-inputs.json','review-repair-timeline242.py',[('csd-review-repair-rebind242','review-repair-rebind242-inputs.json'),('csd-production-timeline21','timeline21-inputs.json')])
print('Rebound105 full takes and exact chapters06/08 schedules checked; actual repaired review still required.')
