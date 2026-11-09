"""Wait for build313, then run rebind313 and the targeted 09 timeline313, each once under the guard.

Adapted from review-repair-after-build269.py by cc0 (not run by cc0). It launches nothing if
build313 did not succeed, and it pins both workers it launches.
"""
from pathlib import Path
import json,hashlib,subprocess,time
R=Path('/data/projects/jmev-stroke-probe-20261106');Q=R/'qualification01/production-repair17';TREE=R/'work/review-repair313/workspace'
WORKERS={'review-repair-rebind313.py':'0e04aa9eaa4f824f385459f24c7ca682f4df497d3fd424e756d34332486c2f8d','review-repair-timeline313.py':'1034931710af7c46c79b09c391122bd22a38471d1d1f73fd501f09ec672d474f'}
def sha(p):
 with p.open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest()
def success(job):
 r=json.loads((R/'jobs'/job/'resource-receipt.json').read_text());assert r.get('finished') and r.get('exit')==0 and r['inputs_sha256']==r['final_inputs_sha256'];assert not any(r.get(k) for k in ('error','resource_stop','stopped_for_resource_limit'));return r
def freeze_launch(job,manifest,worker,prior,extra=(),exclude=()):
 assert not (R/'jobs'/job).exists()
 assert sha(Q/worker)==WORKERS[worker],worker
 paths={Path(__file__).resolve(),Q/worker,R/'guard_csd02.py',R/'work/bin/review-repair313',Q/'prepare-assembly19.py'}
 for name,m in prior:
  r=success(name);m=Q/m;assert sha(m)==r['inputs_sha256'];paths.add(m)
  for row in json.loads(m.read_text())['files']:
   p=Path(row['path']);assert sha(p)==row['sha256'],str(p);paths.add(p)
  paths.update(p for p in (R/'jobs'/name).rglob('*') if p.is_file())
 paths.update(p for p in TREE.rglob('*') if p.is_file() and str(p.relative_to(TREE)) not in exclude)
 paths.update(p for p in (R/'jobs/csd-production-assembly19/narration').rglob('*') if p.is_file())
 for d in extra:paths.update(p for p in (R/'jobs'/d).rglob('*') if p.is_file())
 manifest=Q/manifest
 with manifest.open('x') as f:json.dump({'files':[dict(path=str(p),sha256=sha(p)) for p in sorted(paths)]},f,indent=2)
 command=['/usr/bin/python3','-I','-B',str(R/'guard_csd02.py'),'--phase','native','--job',job,'--inputs',str(manifest),'--','/usr/bin/timeout','172900','/usr/bin/python3','-I','-B',str(Q/worker)]
 if worker=='review-repair-rebind313.py':command.append('--run')
 subprocess.run(command,check=True);success(job)
for worker,digest in WORKERS.items():assert sha(Q/worker)==digest,worker
for _ in range(2160):
 p=R/'jobs/csd-review-repair-build313/resource-receipt.json'
 if p.exists() and json.loads(p.read_text()).get('finished'):break
 time.sleep(20)
else:raise RuntimeError('Build wait deadline; no dependent job launched')
success('csd-review-repair-build313')
# rebind313 compares with rebind269's narration and summary (its job folder, frozen here) and with
# the review269 map in the 268 workspace (already a row of build313's inputs manifest)
freeze_launch('csd-review-repair-rebind313','review-repair-rebind313-inputs.json','review-repair-rebind313.py',[('csd-review-repair-build313','review-repair-build313-inputs.json')],('csd-review-repair-rebind269',),('explainer_corpus/complex_analysis_visual/narration_units.json',))
freeze_launch('csd-review-repair-timeline313','review-repair-timeline313-inputs.json','review-repair-timeline313.py',[('csd-review-repair-rebind313','review-repair-rebind313-inputs.json'),('csd-review-repair-timeline269','review-repair-timeline269-inputs.json')])
print('Rebound105 unchanged WAVs and checked the 09 tables against timeline269; actual repaired pictures still required.')
