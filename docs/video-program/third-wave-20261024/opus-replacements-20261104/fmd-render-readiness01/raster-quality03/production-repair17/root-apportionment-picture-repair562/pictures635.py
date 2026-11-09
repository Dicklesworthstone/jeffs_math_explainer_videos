"""Wait for each existing614 final and extract retained/native4K samples under the exclusive guard. No acceptance."""
import hashlib,json,subprocess,sys,time
from pathlib import Path
P=Path(__file__).resolve().parent
R=Path('/Users/jemanuel/projects/jeffs_math_explainer_videos')
B=R/'narration_lab/third-wave-20261024/root/five-film-release-20261027'
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def main():
 assert sha(P/'pictures634.py')=='6b68fe65467272c40660182c73a23dab01d1cc0dfa4d524c38b370efa1a91998'
 assert sha(P.parent/'root-apportionment-picture-support616.py')=='4deb8cf2b846607c1a4733494c502c2d1ae0834ddfb59e46361f2bce69d9d112'
 chapters=list(json.loads((B/'apportionment-picture-timeline567/comparison567.json').read_text())['chapters'])
 assert len(chapters)==15
 for ch in chapters:
  assert not (B/('apportionment-final-pictures634-'+ch)).exists()
  assert not (P/('pictures634-'+ch+'-inputs.json')).exists()
 for ch in chapters:
  rec=B/('apportionment-final632-'+ch)/'resource-receipt.json'
  deadline=time.monotonic()+86400
  while True:
   try:d=json.loads(rec.read_text())
   except (FileNotFoundError,json.JSONDecodeError):d={}
   if d.get('finished'):
    assert d.get('admitted') and d.get('exit')==0 and d['inputs_sha256']==d['final_inputs_sha256'],ch
    assert not any(d.get(k) for k in ('error','resource_stop','stopped_for_resource_limit')),ch
    break
   assert time.monotonic()<deadline,('wait expired; no restart',ch)
   time.sleep(15)
  print('FINAL_RECEIPT_READY',ch,flush=True)
  subprocess.run([sys.executable,'-I','-B',str(P/'pictures634.py'),'inputs',ch],check=True,cwd=R)
  command=[sys.executable,'-I','-B',str(P.parent/'guard_owner80g_target20g.py'),'--phase','native','--job','apportionment-final-pictures634-'+ch,'--inputs',str(P/('pictures634-'+ch+'-inputs.json')),'--',sys.executable,'-I','-B',str(P/'pictures634.py'),'extract',ch]
  subprocess.run(command,check=True,cwd=R)
  final=json.loads((B/('apportionment-final-pictures634-'+ch)/'resource-receipt.json').read_text())
  assert final.get('finished') and final.get('exit')==0 and final['inputs_sha256']==final['final_inputs_sha256']
  print('PICTURES_READY_FOR_ACTUAL_VIEWING',ch,flush=True)
if __name__=='__main__':main()
