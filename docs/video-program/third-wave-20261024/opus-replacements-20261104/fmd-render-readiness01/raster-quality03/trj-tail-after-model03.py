import json,time,hashlib,subprocess
from pathlib import Path
T=Path('/home/ubuntu/jmev-algebra-asr-20261106');S=T/'work/tail03-inputs';M=T/'jobs/model-stage01';deadline=time.monotonic()+3600
def sha(p):
 with p.open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest()
while True:
 r=json.loads((M/'resource-receipt.json').read_text())
 if 'finished' in r:break
 if time.monotonic()>deadline:raise SystemExit('model-stage wait expired; inspect, never duplicate')
 time.sleep(10)
assert r.get('exit')==0 and not r.get('stopped_for_resource_limit') and r['inputs_sha256']==r['final_inputs_sha256'],r
model=M/'ggml-large-v3-turbo.bin';assert sha(model)=='1fc70f774d38eb169993ac391eea357ef47c88757ef72ee5943879b7e8e2bc69'
assert (S/'preserve-tail03.py').exists()
for row in json.loads((S/'source-downloads.json').read_text()):assert sha(Path(row['source']))==row['sha256']
paths=list(S.iterdir())+[model,M/'resource-receipt.json',M/'model-verified.json',T/'inputs/guard-trj01.py',T/'inputs/original-guard.py',Path('/home/ubuntu/.local/bin/fw').resolve()]
with (S/'inputs.json').open('x') as f:json.dump({'files':[{'path':str(p),'sha256':sha(p)} for p in paths]},f,indent=2)
cmd=['python3',str(T/'inputs/guard-trj01.py'),'--phase','asr','--job','algebra-frederick-chapter12-tail03','--inputs',str(S/'inputs.json'),'--','python3','-I','-B',str(S/'preserve-tail03.py'),'--run']
raise SystemExit(subprocess.run(cmd).returncode)
