"""Wait for exact selected chapter imports, then invoke full guarded master checks.
Does not issue publication or visual acceptance. No retry after a failed receipt.
"""
from pathlib import Path
import hashlib,json,subprocess,time
R=Path('/data/projects/jmev-render-worker-20261107');Q=R/'qualification01/production19'
PLAN=Q/'chapter-plan183.json';CONTROLLER=Q/'trj-master-after-chapters184.py'
def sha(p):
 with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest()
def read(p):return json.loads(Path(p).read_text())
def success(r):
 assert r.get('finished') and r.get('exit')==0 and r['inputs_sha256']==r['final_inputs_sha256']
 assert not any(r.get(k) for k in ['error','resource_stop','stopped_for_resource_limit'])
def main():
 assert not (R/'jobs/trj-production19-master01').exists()
 files=[Path(__file__).resolve(),PLAN,CONTROLLER,Q/'trj-final-master184.py',Q/'trj-final-chapter19-02.py',R/'guard_trj01.py']
 pins={p:sha(p) for p in files};plan=read(PLAN);rows=plan['chapters']
 assert len(rows)==15 and len({x['chapter'] for x in rows})==15
 assert [x['chapter'] for x in rows]==sorted(x['chapter'] for x in rows)
 receipts=[]
 for x in rows:
  assert Path(x['job']).name==x['job']
  if 'import_receipt' in x:receipts.append(Path(x['import_receipt']))
  else:
   assert x['job'].startswith('trj-production19-final-')
   receipts.append(R/'jobs'/x['job']/'resource-receipt.json')
 assert len(set(receipts))==15
 deadline=time.monotonic()+48*3600
 while True:
  pending=[]
  for p in receipts:
   if not p.exists():pending.append(str(p));continue
   r=read(p)
   if not r.get('finished'):pending.append(str(p));continue
   success(r)
  if not pending:break
  if time.monotonic()>deadline:raise RuntimeError('Prerequisite wait expired; no master launched')
  time.sleep(30)
 for p,h in pins.items():assert sha(p)==h,str(p)
 assert not (R/'jobs/trj-production19-master01').exists()
 subprocess.run(['python3','-I','-B',str(CONTROLLER),'--plan',str(PLAN)],check=True)
 success(read(R/'jobs/trj-production19-master01/resource-receipt.json'))
if __name__=='__main__':main()
