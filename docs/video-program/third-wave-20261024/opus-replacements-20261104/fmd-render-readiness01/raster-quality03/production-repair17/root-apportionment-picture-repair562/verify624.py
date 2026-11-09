"""Wait for existing master619; freeze full evidence and launch guarded verifier623 once."""
import hashlib,json,subprocess,sys,time
from pathlib import Path
P=Path(__file__).resolve().parent
R=Path('/Users/jemanuel/projects/jeffs_math_explainer_videos')
B=R/'narration_lab/third-wave-20261024/root/five-film-release-20261027'
MASTER=B/'apportionment-master619'
JOB=B/'apportionment-verify623'
INPUTS=P/'verify623-inputs.json'
def sha(p):
 with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest()
def main():
 assert not JOB.exists() and not INPUTS.exists()
 helper=P/'verify623.py';assert sha(helper)=='a01c84457dc5084502df2cee55c9a0594a3ea3d2272470b11ce0f1bd84629e4b'
 receipt=MASTER/'resource-receipt.json';deadline=time.monotonic()+86400
 while True:
  try:d=json.loads(receipt.read_text())
  except (FileNotFoundError,json.JSONDecodeError):d={}
  if d.get('finished'):break
  assert time.monotonic()<deadline,'master wait expired; no verifier launched'
  time.sleep(30)
 assert d.get('admitted') and d.get('exit')==0 and d['inputs_sha256']==d['final_inputs_sha256']
 assert not any(d.get(k) for k in ('error','resource_stop','stopped_for_resource_limit'))
 rows={}
 def add(p,digest=None):
  p=Path(p);actual=sha(p)
  if digest is not None:assert actual==digest,p
  assert str(p) not in rows or rows[str(p)]==actual,p
  rows[str(p)]=actual
 original=P/'master619-inputs.json';add(original,d['inputs_sha256'])
 for row in json.loads(original.read_text())['files']:add(row['path'],row['sha256'])
 for file in MASTER.rglob('*'):
  assert not file.is_symlink(),file
  if file.is_file():add(file)
 for file in (helper,Path(__file__).resolve(),Path(sys.executable).resolve(),R/'explainer_corpus/apportionment/script.tsv',Path('/opt/homebrew/bin/ffmpeg').resolve(),Path('/opt/homebrew/bin/ffprobe').resolve()):add(file)
 with INPUTS.open('x') as f:json.dump({'files':[{'path':k,'sha256':v} for k,v in sorted(rows.items())]},f,indent=2)
 command=[sys.executable,'-I','-B',str(P.parent/'guard_owner80g_target20g.py'),'--phase','native','--job',JOB.name,'--inputs',str(INPUTS),'--',sys.executable,'-I','-B',str(helper)]
 print('LAUNCH_VERIFIER623',len(rows),'bindings',flush=True)
 subprocess.run(command,cwd=R,check=True)
 result=json.loads((JOB/'resource-receipt.json').read_text())
 assert result.get('finished') and result.get('exit')==0 and result['inputs_sha256']==result['final_inputs_sha256']
 print('VERIFIER623_COMPLETED; actual visual/audio and independent acceptance remain',flush=True)
if __name__=='__main__':main()
