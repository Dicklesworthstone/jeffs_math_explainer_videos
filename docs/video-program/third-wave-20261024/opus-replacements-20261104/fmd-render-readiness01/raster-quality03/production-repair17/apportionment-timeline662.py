"""Guarded full native schedule comparison before any CSD final rendering."""
from pathlib import Path
import json,hashlib,subprocess,os,time
D=Path('/data/projects/jmev-stroke-probe-20261106');Q=D/'qualification01/production-repair17';J=D/'jobs/csd-apportionment-timeline662';A=D/'jobs/csd-apportionment-assembly660';B=D/'jobs/csd-apportionment-build656';R=A/'repo';C=B/'workspace/explainer_corpus/apportionment'
REL='narration_lab/third-wave-20261024/root/five-film-release-20261027';N=R/REL/'apportionment-picture-assembly566/narration';REF=R/REL/'apportionment-picture-timeline567/timeline';OLD='/Users/jemanuel/projects/jeffs_math_explainer_videos/'+REL+'/apportionment-picture-assembly566/narration'
def sha(p):
 with Path(p).open('rb')as f:return hashlib.file_digest(f,'sha256').hexdigest()
def read(p):return json.loads(Path(p).read_text())
def success(p):
 d=read(p);assert d.get('admitted')and d.get('finished')and d.get('exit')==0 and d['inputs_sha256']==d['final_inputs_sha256'];assert not any(d.get(k)for k in ['error','resource_stop','stopped_for_resource_limit'])
a=read(J/'resource-receipt.json');assert a.get('admitted')and not a.get('finished')
for r in read(Q/'csd-apportionment-timeline662-inputs.json')['files']:assert sha(r['path'])==r['sha256']
for p in [A,B]:success(p/'resource-receipt.json')
q=read(B/'qualification.json');binary=B/'apportionment-native656';assert sha(binary)==q['binary_sha256'];rel=read(A/'relocation660.json');assert rel['full_coherent_verification']and rel['cues']==93
os.sched_setaffinity(0,{0,1,2,3});os.environ['RAYON_NUM_THREADS']='4'
chapters=subprocess.check_output([str(binary),'chapters'],cwd=C,text=True,timeout=120).splitlines();assert chapters==q['chapters']and len(chapters)==15
qa=R/REL/'apportionment-frederick-score445/score/stems/qa';assert {p.stem for p in qa.glob('*.json')}==set(chapters)
assert all(read(qa/(c+'.json'))['passed']for c in chapters)
assert not (J/'timeline').exists();deadline=time.monotonic()+14400;records={}
for ch in chapters:
 cmd=[str(binary),'render',ch,'--res','64x36','--fps','60','--lint','--narration',str(N),'--out',str(J/'timeline'),'--crf','16','--preset','slow','--threads','4']
 print('TIMELINE662',ch,flush=True)
 with (J/(ch+'.stdout')).open('xb')as o,(J/(ch+'.stderr')).open('xb')as e:p=subprocess.run(cmd,cwd=C,stdout=o,stderr=e,timeout=max(1,deadline-time.monotonic()))
 assert p.returncode==0,(ch,p.returncode)
 tables={}
 for ext in ['timeline.tsv','cues.tsv','cuelog.tsv']:
  path=J/'timeline'/(ch+'.'+ext);old=REF/(ch+'.'+ext)
  assert path.read_text().replace(str(N),OLD)==old.read_text(),(ch,ext,'schedule mismatch')
  tables[ext]=sha(path)
 h=dict(x.split('\t',1)for x in (J/'timeline'/(ch+'.timeline.tsv')).read_text().splitlines()if x.startswith(('chapter\t','frames\t','fps\t')));assert h['chapter']==ch and h['fps']=='60'
 records[ch]=dict(frames=int(h['frames']),tables=tables,layout_sha256=sha(J/'timeline'/(ch+'.layout.tsv')))
 print('EXACT_SCHEDULE',ch,h['frames'],flush=True)
assert sum(x['frames']for x in records.values())==100886
with (J/'comparison662.json').open('x')as f:json.dump(dict(chapters=records,total_frames=100886,full_normalized_tables_equal=True,score_qa_passed=15,scope='Exact schedules after path-only relocation; no final picture/motion/listening or release acceptance.'),f,indent=2)
