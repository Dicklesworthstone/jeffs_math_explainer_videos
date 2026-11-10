"""Guarded path-only relocation; retained recordings and timing remain exact."""
from pathlib import Path
import json,hashlib,shutil,subprocess,copy
D=Path('/data/projects/jmev-stroke-probe-20261106');Q=D/'qualification01/production-repair17';J=D/'jobs/csd-apportionment-assembly660';BUILD=D/'jobs/csd-apportionment-build656';R=J/'repo';OLD='/Users/jemanuel/projects/jeffs_math_explainer_videos/'
REL='narration_lab/third-wave-20261024/root/five-film-release-20261027/apportionment-picture-assembly566/narration'
N=R/REL

def read(p):return json.loads(Path(p).read_text())
def sha(p):
 with Path(p).open('rb')as f:return hashlib.file_digest(f,'sha256').hexdigest()
def success(p):
 d=read(p);assert d.get('admitted')and d.get('finished')and d.get('exit')==0 and d['inputs_sha256']==d['final_inputs_sha256'];assert not any(d.get(k)for k in ['error','resource_stop','stopped_for_resource_limit'])
def write(p,d):
 with p.open('x')as f:json.dump(d,f,indent=2)
a=read(J/'resource-receipt.json');assert a.get('admitted')and not a.get('finished')
for r in read(Q/'csd-apportionment-assembly660-inputs.json')['files']:assert sha(r['path'])==r['sha256']
success(BUILD/'resource-receipt.json');qual=read(BUILD/'qualification.json');assert sha(BUILD/'apportionment-native656')==qual['binary_sha256']
for n,h in qual['source'].items():assert sha(BUILD/n)==h
assert not R.exists();shutil.copytree(BUILD/'workspace',R)
rows={}
for v in ['652','655']:
 p=D/('jobs/csd-apportionment-import'+v+'-a1');success(p/'resource-receipt.json')
 for row in read(p/'import-result.json')['files']:
  assert sha(row['physical'])==row['sha256']
  if row['path']in rows:assert rows[row['path']]['sha256']==row['sha256']
  rows[row['path']]=row
for name,row in rows.items():
 if not name.startswith(OLD):continue
 rel=name.removeprefix(OLD);out=R/rel
 if out.exists():
  if rel in ['Cargo.toml','explainer_corpus/apportionment/Cargo.lock']:continue
  assert sha(out)==row['sha256'];continue
 out.parent.mkdir(parents=True,exist_ok=True)
 with Path(row['physical']).open('rb')as src,out.open('xb')as dst:shutil.copyfileobj(src,dst)
old_a=read(N/'assembly.json');old_r=read(N/'report.json');assert old_a['report_sha256']==sha(N/'report.json');assert len(old_a['outputs'])==93
receipt_paths=sorted(set(u['generation_receipt']for u in old_a['manifest']['units']))
receipt_map={p:str(R/p.removeprefix(OLD))+'.relocated660.json'for p in receipt_paths}
changes=[]
def relocate(x):
 if isinstance(x,dict):return {k:relocate(v)for k,v in x.items()}
 if isinstance(x,list):return [relocate(v)for v in x]
 if isinstance(x,str)and x.startswith(OLD):
  out=receipt_map.get(x,str(R/x.removeprefix(OLD)));changes.append([x,out]);return out
 return x
for p in receipt_paths:
 assert p in rows
 original=read(rows[p]['physical']);derived=relocate(original)
 write(Path(receipt_map[p]),derived)
for name in ['report.json','assembly.json']:
 p=N/name;backup=N/(name+'.before-path660');assert not backup.exists();p.rename(backup)
r=relocate(old_r);write(N/'report.json',r)
a=relocate(old_a);a['report_sha256']=sha(N/'report.json');write(N/'assembly.json',a)
for cue,row in old_a['outputs'].items():assert sha(N/(cue+'.wav'))==row['sha256']
assert sha(N/'phrase-timings.json')==rows[OLD+REL+'/phrase-timings.json']['sha256']
cmd=['/usr/bin/python3','-I','-B',str(R/'tools/coherent_narration.py'),'verify',str(N),'--script',str(R/'explainer_corpus/apportionment/script.tsv'),'--units',str(R/'explainer_corpus/apportionment/narration_units.json'),'--require-directions']
with (J/'verify.stdout').open('xb')as o,(J/'verify.stderr').open('xb')as e:p=subprocess.run(cmd,cwd=R,stdout=o,stderr=e,timeout=1800)
assert p.returncode==0,'Full source/direction/sample verification failed; preserve attempted relocation'
write(J/'relocation660.json',dict(path_changes=changes,cues=93,wavs_byte_identical=True,phrase_timings_byte_identical=True,full_coherent_verification=True,binary_sha256=qual['binary_sha256'],scope='Source-bound path-only relocation and sample reconstruction. No listening, timing acceptance, render or release claim.'))
print('ASSEMBLY660 verified93 unchanged recordings and timings',flush=True)
