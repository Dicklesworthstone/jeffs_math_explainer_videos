from pathlib import Path
import json,hashlib,subprocess,time,shlex
R=Path('/data/projects/jmev-render-worker-20261107');Q=R/'qualification01/production19'
def sha(p):
 with p.open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest()
for _ in range(720):
 result=subprocess.run(['ssh','-o','BatchMode=yes','csd','python3','-I','-B','/data/projects/jmev-stroke-probe-20261106/qualification01/production-repair17/csd-score-transfer-inventory21.py'],capture_output=True,text=True,timeout=180)
 if result.returncode:raise RuntimeError('CSD prerequisite or transport failed: '+result.stderr[-2000:])
 d=json.loads(result.stdout)
 if d['state']=='ready':break
 time.sleep(60)
else:raise RuntimeError('Waiting deadline; no transfer launched')
m=Q/'score-transfer21-files.json'
with m.open('x') as f:json.dump(d,f,indent=2)
files=[m,Path(__file__).resolve(),Q/'trj-score-transfer21.py',R/'guard_trj01.py',R/'jobs/trj-production19-delivery02/resource-receipt.json']
inputs=Q/'score-transfer21-inputs.json'
with inputs.open('x') as f:json.dump({'files':[dict(path=str(p),sha256=sha(p)) for p in files]},f,indent=2)
subprocess.run(['python3','-I','-B',str(R/'guard_trj01.py'),'--phase','native','--job','trj-production21-score-transfer01','--inputs',str(inputs),'--','python3','-I','-B',str(Q/'trj-score-transfer21.py')],check=True)
print('Score and schedules staged; final render still requires actual review authorization')
