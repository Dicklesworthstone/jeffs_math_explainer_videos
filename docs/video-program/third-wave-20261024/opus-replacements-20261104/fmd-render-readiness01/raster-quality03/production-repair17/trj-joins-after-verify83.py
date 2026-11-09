from pathlib import Path
import hashlib,json,subprocess,time,shutil
R=Path('/data/projects/jmev-render-worker-20261107');Q=R/'qualification01/production19'
def sha(p):
 with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest()
def read(p):return json.loads(Path(p).read_text())
p=R/'jobs/trj-production19-verify01/resource-receipt.json'
helper=Q/'trj-final-joins83.py';helper_hash=sha(helper)
for _ in range(4320):
 if p.exists() and read(p).get('finished'):break
 time.sleep(30)
else:raise RuntimeError('Verify wait deadline; no joins job launched')
r=read(p);assert r['exit']==0 and r['inputs_sha256']==r['final_inputs_sha256'] and not any(r.get(k) for k in ('error','resource_stop','stopped_for_resource_limit'))
assert sha(helper)==helper_hash
prior=Q/'verify01-inputs.json';assert sha(prior)==r['inputs_sha256']
files={x['path']:x['sha256'] for x in read(prior)['files']}
for path,digest in files.items():assert sha(path)==digest,path
for path in (Path(__file__).resolve(),helper,p,prior,R/'jobs/trj-production19-verify01/verification.json',Path(shutil.which('ffmpeg')).resolve(),R/'guard_trj01.py'):files[str(path)]=sha(path)
inputs=Q/'join-extract83-inputs.json'
assert not (R/'jobs/trj-production19-join-extract83').exists()
with inputs.open('x') as f:json.dump(dict(files=[dict(path=p,sha256=h) for p,h in sorted(files.items())]),f,indent=2)
subprocess.run(['python3','-I','-B',str(R/'guard_trj01.py'),'--phase','native','--job','trj-production19-join-extract83','--inputs',str(inputs),'--','python3','-I','-B',str(helper)],check=True)
