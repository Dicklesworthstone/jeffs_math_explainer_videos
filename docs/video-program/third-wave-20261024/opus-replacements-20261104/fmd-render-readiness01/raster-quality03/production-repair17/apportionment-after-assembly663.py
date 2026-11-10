from pathlib import Path
import json,hashlib,ast,subprocess,time,shutil
D=Path('/data/projects/jmev-stroke-probe-20261106');Q=D/'qualification01/production-repair17';A=D/'jobs/csd-apportionment-assembly660';B=D/'jobs/csd-apportionment-build656';J=D/'jobs/csd-apportionment-timeline662';inputs=Q/'csd-apportionment-timeline662-inputs.json';guard=D/'guard_csd02.py'
def sha(p):
 with Path(p).open('rb')as f:return hashlib.file_digest(f,'sha256').hexdigest()
assert not J.exists()and not inputs.exists(),'No duplicate timeline'
for _ in range(2160):
 p=A/'resource-receipt.json';d=json.loads(p.read_text())if p.exists()else{}
 b=json.loads((B/'resource-receipt.json').read_text())
 if b.get('finished'):assert b.get('exit')==0,'Build failed; no timeline'
 if d.get('finished'):
  assert d.get('exit')==0 and d['inputs_sha256']==d['final_inputs_sha256']and not d.get('stopped_for_resource_limit'),'Assembly failed; no timeline'
  break
 time.sleep(20)
else:raise RuntimeError('Assembly wait deadline; no timeline launched')
# The native binary embeds its build topic path. Install identical retained runtime
# contract helpers beside that topic, without changing any build-frozen source.
installed=[]
for n in ['coherent_narration.py','narration_cache.py','narration_delivery.py','storyboard.py']:
 src=A/'repo/tools'/n;dst=B/'workspace/tools'/n
 assert src.is_file()and not dst.exists();dst.parent.mkdir(parents=True,exist_ok=True)
 with src.open('rb')as s,dst.open('xb')as t:shutil.copyfileobj(s,t)
 assert sha(src)==sha(dst);installed.append(dict(path=str(dst),sha256=sha(dst)))
with (Q/'apportionment-runtime663.json').open('x')as f:json.dump(installed,f,indent=2)
files={Path(__file__).resolve(),Q/'apportionment-timeline662.py',Q/'apportionment-runtime663.json',guard,A/'resource-receipt.json',A/'relocation660.json',B/'resource-receipt.json',B/'qualification.json',B/'apportionment-native656',Path('/usr/bin/python3').resolve(),D/'qualification01/owned-process-stop02.py',Path('/usr/bin/ffmpeg').resolve(),Path('/usr/bin/ffprobe').resolve()}
qual=json.loads((B/'qualification.json').read_text())
for n,h in qual['source'].items():
 p=B/n;assert sha(p)==h;files.add(p)
files.update(Path(x['path'])for x in installed)
files.update(p for p in (A/'repo').rglob('*')if p.is_file())
for node in ast.parse(guard.read_text()).body:
 if isinstance(node,ast.Assign)and any(isinstance(t,ast.Name)and t.id=='PINS'for t in node.targets):
  for n,h in ast.literal_eval(node.value).items():
   p=D/n;assert sha(p)==h;files.add(p)
with inputs.open('x')as f:json.dump(dict(files=[dict(path=str(p),sha256=sha(p))for p in sorted(files)]),f,indent=2)
print('ASSEMBLY passed; launching guarded timeline662',flush=True)
raise SystemExit(subprocess.call(['/usr/bin/python3','-I','-B',str(guard),'--phase','native','--job',J.name,'--inputs',str(inputs),'--','/usr/bin/python3','-I','-B',str(Q/'apportionment-timeline662.py')]))
