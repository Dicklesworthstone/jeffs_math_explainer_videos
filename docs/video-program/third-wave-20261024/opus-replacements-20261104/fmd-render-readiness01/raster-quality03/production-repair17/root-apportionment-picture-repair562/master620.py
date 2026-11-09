"""Wait for all existing final614 chapters; freeze exact closure then guarded master619 once."""
import hashlib,json,subprocess,sys,time
from pathlib import Path
P=Path(__file__).resolve().parent
R=Path('/Users/jemanuel/projects/jeffs_math_explainer_videos')
B=R/'narration_lab/third-wave-20261024/root/five-film-release-20261027'
JOB=B/'apportionment-master619'
INPUTS=P/'master619-inputs.json'
def sha(p):
 with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest()
def main():
 assert not JOB.exists() and not INPUTS.exists()
 assert sha(P/'master619.py')=='bda28acef3134b9630ef5b36c9fd795ab1c9744d78f49bacb1462435c81639b1'
 comparison=B/'apportionment-picture-timeline567/comparison567.json'
 registry=list(json.loads(comparison.read_text())['chapters']);assert len(registry)==15 and len(set(registry))==15
 deadline=time.monotonic()+86400
 while True:
  waiting=[]
  for ch in registry:
   path=B/('apportionment-final614-'+ch)/'resource-receipt.json'
   try:d=json.loads(path.read_text())
   except (FileNotFoundError,json.JSONDecodeError):d={}
   if not d.get('finished'):waiting.append(ch);continue
   assert d.get('admitted') and d.get('exit')==0 and d['inputs_sha256']==d['final_inputs_sha256'],ch
   assert not any(d.get(k) for k in ('error','resource_stop','stopped_for_resource_limit')),ch
  if not waiting:break
  assert time.monotonic()<deadline,('wait expired, no master launched',waiting)
  time.sleep(30)
 rows={};total=0;encoder=None
 def add(path,digest=None):
  path=Path(path);actual=sha(path)
  if digest is not None:assert actual==digest,path
  assert str(path) not in rows or rows[str(path)]==actual,path
  rows[str(path)]=actual
 for ch in registry:
  job=B/('apportionment-final614-'+ch);receipt=job/'resource-receipt.json';d=json.loads(receipt.read_text());add(receipt)
  inputs=P/('final614-'+ch+'-inputs.json');add(inputs,d['inputs_sha256'])
  for row in json.loads(inputs.read_text())['files']:add(row['path'],row['sha256'])
  summary=job/'final614-summary.json';add(summary);s=json.loads(summary.read_text());assert set(s['chapters'])=={ch};row=s['chapters'][ch]
  assert row['folder']==str((job/'render').relative_to(R));total+=row['frames']
  if encoder is None:encoder=row['encoder_settings']
  assert row['encoder_settings']==encoder,ch
  for name,digest in row['files'].items():
   assert not Path(name).is_absolute() and '..' not in Path(name).parts
   add(job/'render'/name,digest)
 assert total==100886
 for path in [P/'master619.py',Path(__file__).resolve(),Path(sys.executable).resolve(),comparison]:add(path)
 assert not JOB.exists()
 with INPUTS.open('x') as f:json.dump({'files':[{'path':k,'sha256':v} for k,v in sorted(rows.items())]},f,indent=2)
 command=[sys.executable,'-I','-B',str(P.parent/'guard_owner80g_target20g.py'),'--phase','native','--job',JOB.name,'--inputs',str(INPUTS),'--',sys.executable,'-I','-B',str(P/'master619.py')]
 print('LAUNCH_MASTER619',len(rows),'exact bindings',flush=True)
 subprocess.run(command,cwd=R,check=True)
 final=json.loads((JOB/'resource-receipt.json').read_text())
 assert final.get('finished') and final.get('exit')==0 and final['inputs_sha256']==final['final_inputs_sha256']
 print('MASTER619_COMPLETED; final verification and actual AV review pending',flush=True)
if __name__=='__main__':main()
