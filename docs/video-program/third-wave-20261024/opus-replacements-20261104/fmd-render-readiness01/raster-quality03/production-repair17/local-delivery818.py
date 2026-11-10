import json,hashlib,shutil,subprocess,sys
from pathlib import Path
Q=Path(__file__).resolve().parent;R=Q.parents[6];W=R/'narration_lab/third-wave-20261024/root/five-film-release-20261027';M=Path('/Users/jemanuel/scratch/jmev-complex-native-20261109')
old=M/'jobs/mini-boundary-delivery764/narration';out=W/'complex-local-delivery818/narration'
def sha(p):
 with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest()
def read(p):return json.loads(Path(p).read_text())
prior=read(W/'complex-local-check813/resource-receipt.json');assert prior.get('finished') and prior['exit']==1 and prior['inputs_sha256']==prior['final_inputs_sha256']
assert M.resolve()==W/'complex-mini-transfer812/tree'
out.mkdir()
for p in old.iterdir():
 assert p.is_file() and not p.is_symlink()
 if p.name not in ('report.json','assembly.json'):shutil.copy2(p,out/p.name);assert sha(p)==sha(out/p.name)
report=read(old/'report.json');assembly=read(old/'assembly.json');changes=[]
for cue,rows in report['lines'].items():
 for row in rows:
  before=row['source'];after=str(Path(before).resolve(strict=True));assert Path(before).is_relative_to(M)
  assert sha(before)==sha(after)==row['source_sha256'];row['source']=after;changes.append({'cue':cue,'before':before,'after':after})
# Only the source path may differ; every sentence, sample boundary and gap stays exact.
original=read(old/'report.json')
for cue,rows in report['lines'].items():
 for a,b in zip(rows,original['lines'][cue]):assert {k:v for k,v in a.items() if k!='source'}=={k:v for k,v in b.items() if k!='source'}
with (out/'report.json').open('x') as f:json.dump(report,f,indent=2)
assembly['report_sha256']=sha(out/'report.json')
with (out/'assembly.json').open('x') as f:json.dump(assembly,f,indent=2)
subprocess.run([sys.executable,'-I','-B',str(M/'source760/workspace/tools/narration_delivery.py'),'--topic',str(M/'source760/workspace/explainer_corpus/complex_analysis_visual'),'--narration',str(out),'--consumer','render','--gain-db','0','--chapter-key','09_boundary'],check=True,cwd=M/'repo')
wavs=list(old.glob('*.wav'));assert len(wavs)==105
for p in wavs:assert sha(p)==sha(out/p.name)
with (out.parent/'rebind818.json').open('x') as f:json.dump({'wavs_byte_identical':len(wavs),'source_path_changes':changes,'full_source_reconstruction_and_contract_pass':True,'listening_accepted':False},f,indent=2)
