from pathlib import Path
import json,hashlib,ast,subprocess,time
D=Path('/data/projects/jmev-stroke-probe-20261106');Q=D/'qualification01/production-repair17';B=D/'jobs/csd-apportionment-build656';J=D/'jobs/csd-apportionment-assembly664';inputs=Q/'csd-apportionment-assembly664-inputs.json'
def sha(p):
 with Path(p).open('rb')as f:return hashlib.file_digest(f,'sha256').hexdigest()
assert not J.exists()and not inputs.exists(),'No duplicate assembly'
for _ in range(2160):
 p=B/'resource-receipt.json';d=json.loads(p.read_text())if p.exists()else{}
 if d.get('finished'):
  assert d.get('exit')==0 and d['inputs_sha256']==d['final_inputs_sha256']and not d.get('stopped_for_resource_limit'), 'Build failed; no assembly launched'
  break
 time.sleep(20)
else:raise RuntimeError('Build wait expired; no assembly launched')
helper=Q/'apportionment-assembly664.py';guard=D/'guard_csd02.py';files={Path(__file__).resolve(),helper,guard,B/'resource-receipt.json',B/'qualification.json',B/'apportionment-native656',Path('/usr/bin/python3').resolve(),D/'qualification01/owned-process-stop02.py'}
qual=json.loads((B/'qualification.json').read_text())
for n,h in qual['source'].items():
 p=B/n;assert sha(p)==h;files.add(p)
for v in ['652','655']:
 j=D/('jobs/csd-apportionment-import'+v+'-a1');files.update([j/'resource-receipt.json',j/'import-result.json'])
 for r in json.loads((j/'import-result.json').read_text())['files']:
  p=Path(r['physical']);assert sha(p)==r['sha256'];files.add(p)
for node in ast.parse(guard.read_text()).body:
 if isinstance(node,ast.Assign)and any(isinstance(t,ast.Name)and t.id=='PINS'for t in node.targets):
  for n,h in ast.literal_eval(node.value).items():
   p=D/n;assert sha(p)==h;files.add(p)
with inputs.open('x')as f:json.dump(dict(files=[dict(path=str(p),sha256=sha(p))for p in sorted(files)]),f,indent=2)
print('BUILD passed; launching guarded assembly664',flush=True)
raise SystemExit(subprocess.call(['/usr/bin/python3','-I','-B',str(guard),'--phase','native','--job',J.name,'--inputs',str(inputs),'--','/usr/bin/python3','-I','-B',str(helper)]))
