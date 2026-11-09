from pathlib import Path
import json,hashlib,subprocess,time
Q=Path(__file__).resolve().parent;R=Path('/Users/jemanuel/projects/jeffs_math_explainer_videos');B=R/'narration_lab/third-wave-20261024/root/five-film-release-20261027';TQ='/data/projects/jmev-render-worker-20261107/qualification01/production19'
receipt='/data/projects/jmev-render-worker-20261107/jobs/trj-production19-join-extract83/resource-receipt.json'
def sha(p):
 with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest()
for _ in range(2160):
 p=subprocess.run(['ssh','-o','BatchMode=yes','trj','cat '+receipt],capture_output=True)
 if p.returncode==0:
  r=json.loads(p.stdout)
  if r.get('finished'):
   assert r.get('exit')==0 and r['inputs_sha256']==r['final_inputs_sha256'] and not any(r.get(k) for k in ('error','resource_stop','stopped_for_resource_limit'));break
 time.sleep(60)
else:raise RuntimeError('Join extraction wait deadline; no local job launched')
assert subprocess.check_output(['ssh','trj','sha256sum '+TQ+'/trj-join-export88.py'],text=True).split()[0]==sha(Q/'trj-join-export88.py')
export=Q/'join-export88.json'
with export.open('xb') as f:subprocess.run(['ssh','trj','python3 -I -B '+TQ+'/trj-join-export88.py'],stdout=f,check=True)
assert len(json.loads(export.read_text())['files'])==33
subprocess.run(['ssh','trj','test ! -e '+TQ+'/join-export88.json'],check=True)
subprocess.run(['scp',str(export),'trj:'+TQ+'/join-export88.json'],check=True)
assert subprocess.check_output(['ssh','trj','sha256sum '+TQ+'/join-export88.json'],text=True).split()[0]==sha(export)
files={Path(__file__).resolve(),Q/'complex-join-recognition88.py',export,Q/'trj-join-export88.py',Q/'guard_owner80g_target20g.py',Path('/opt/homebrew/bin/fw').resolve(),Path('/Users/jemanuel/.cache/franken_whisper/models/whisper/whisper-large-v3-turbo-f16-v1/ggml-large-v3-turbo.bin')}
inputs=Q/'join-recognition88-inputs.json';assert not (B/'complex-final-join-recognition88').exists()
with inputs.open('x') as f:json.dump(dict(files=[dict(path=str(p),sha256=sha(p)) for p in sorted(files)]),f,indent=2)
with (Q/'join-recognition88-guard.log').open('x') as f:subprocess.run(['python3','-I','-B',str(Q/'guard_owner80g_target20g.py'),'--phase','native','--job','complex-final-join-recognition88','--inputs',str(inputs),'--','python3','-I','-B',str(Q/'complex-join-recognition88.py')],cwd=R,stdout=f,stderr=subprocess.STDOUT,check=True)
