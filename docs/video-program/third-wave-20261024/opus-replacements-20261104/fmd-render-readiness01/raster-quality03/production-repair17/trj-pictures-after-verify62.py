"""Extract final review images only after successful full technical verification."""
from pathlib import Path
import hashlib
import json
import subprocess
import time
R=Path('/data/projects/jmev-render-worker-20261107');Q=R/'qualification01/production19'
M=R/'jobs/trj-production19-master01';V=R/'jobs/trj-production19-verify01'
JOB='trj-production19-pictures01';HELPER=Q/'trj-final-pictures61.py';PLAN=Q/'final-picture-plan61.json'
def sha(p):
 with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest()
def read(p):return json.loads(Path(p).read_text())
def success(p):
 s=read(p);assert s.get('finished') and s.get('exit')==0 and s['inputs_sha256']==s['final_inputs_sha256']
 assert not any(s.get(k) for k in ('error','resource_stop','stopped_for_resource_limit'))
 return s
def main():
 assert not (R/'jobs'/JOB).exists(),'Existing extraction job; do not duplicate'
 fixed=[Path(__file__).resolve(),HELPER,PLAN,R/'guard_trj01.py']
 plan=read(PLAN);assert plan['frames']==133662 and len(plan['chapters'])==15
 offset=0
 for c in plan['chapters']:
  assert c['start_frame']==offset and c['frames']>0;offset+=c['frames']
 assert offset==133662
 assert len({x['frame'] for x in plan['samples']})==len(plan['samples'])
 for x in plan['samples']:
  c=next(c for c in plan['chapters'] if c['chapter']==x['chapter'])
  assert 0<=x['local_frame']<c['frames'] and x['frame']==c['start_frame']+x['local_frame']
 for x in plan['sources']:
  assert sha(x['path'])==x['sha256'];fixed.append(Path(x['path']))
 pins={str(p):sha(p) for p in fixed};deadline=time.monotonic()+48*3600
 while True:
  ready=True
  for j in [M,V]:
   receipt=j/'resource-receipt.json'
   if not receipt.exists() or not read(receipt).get('finished'):ready=False;continue
   success(receipt)
  if ready:break
  if time.monotonic()>=deadline:raise RuntimeError('Waiting deadline; no pictures launched')
  time.sleep(30)
 for p,h in pins.items():assert sha(p)==h,'Extraction preparation changed'
 files=set(fixed)
 for j,name in [(M,'master01-inputs.json'),(V,'verify01-inputs.json')]:
  receipt=j/'resource-receipt.json';state=success(receipt);manifest=Q/name
  assert sha(manifest)==state['inputs_sha256'];files.update([receipt,manifest])
  for x in read(manifest)['files']:
   assert sha(x['path'])==x['sha256'];files.add(Path(x['path']))
 files.update([M/'master-summary.json',V/'verification.json'])
 summary=read(M/'master-summary.json');assert summary['frames']==133662
 assert [x['chapter'] for x in plan['chapters']]==summary['chapters']
 for x in summary['outputs'].values():assert sha(x['path'])==x['sha256']
 files.update(p for p in (M/'final').rglob('*') if p.is_file())
 manifest=Q/'pictures01-inputs.json'
 with manifest.open('x') as f:json.dump({'files':[{'path':str(p),'sha256':sha(p)} for p in sorted(files)]},f,indent=2)
 subprocess.run(['python3','-I','-B',str(R/'guard_trj01.py'),'--phase','native','--job',JOB,'--inputs',str(manifest),'--','python3','-I','-B',str(HELPER)],check=True)
 success(R/'jobs'/JOB/'resource-receipt.json')
if __name__=='__main__':main()
