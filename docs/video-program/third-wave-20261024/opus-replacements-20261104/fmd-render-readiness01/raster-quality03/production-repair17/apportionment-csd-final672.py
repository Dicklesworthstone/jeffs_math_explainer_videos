"""One guarded CSD 4K chapter; exact schedules, score and quality retained."""
from pathlib import Path
import json,hashlib,subprocess,os,sys,signal,importlib.util,re
D=Path('/data/projects/jmev-stroke-probe-20261106');Q=D/'qualification01/production-repair17'
A=D/'jobs/csd-apportionment-assembly664';B=D/'jobs/csd-apportionment-build656';T=D/'jobs/csd-apportionment-timeline666'
REL='narration_lab/third-wave-20261024/root/five-film-release-20261027'
assert len(sys.argv)==2
c=sys.argv[1];assert all(x.isalnum()or x=='_'for x in c)
J=D/('jobs/csd-apportionment-final672-'+c)
def sha(p):
 with Path(p).open('rb')as f:return hashlib.file_digest(f,'sha256').hexdigest()
def read(p):return json.loads(Path(p).read_text())
spec=importlib.util.spec_from_file_location('owned671',Q/'apportionment-csd-support671.py');o=importlib.util.module_from_spec(spec);spec.loader.exec_module(o)
assert sha(Q/'apportionment-csd-support671.py')=='244fe1a47d5a40f71594031bafc09fefb45558c38ed72adbfb6aa1b1f269c40a'
o.own_group();o.STATE['job']=J
for sig in o.STOP_SIGNALS:signal.signal(sig,o.on_stop)
r=read(J/'resource-receipt.json');assert r.get('admitted')and not r.get('finished')and str(Path(__file__).resolve())in r['command']
assert r['policy']==dict(admission_gib=10,stop_free_gib=8,phase_growth_gib=4,work_cap_gib=56,target_growth_cap_gib=20,sample_seconds=10)
assert sorted(p.name for p in J.iterdir())==['command.log','resource-receipt.json']
for x in read(Q/('csd-apportionment-final672-'+c+'-inputs.json'))['files']:assert sha(x['path'])==x['sha256'],x['path']
for p in [A,B,T]:o.success(p/'resource-receipt.json')
comp=read(T/'comparison666.json');qual=read(B/'qualification.json');binary=B/'apportionment-native656'
assert comp['full_normalized_tables_equal']and comp['total_frames']==100886 and comp['score_qa_passed']==15
assert sha(binary)==qual['binary_sha256']
chapters=subprocess.check_output([str(binary),'chapters'],text=True,timeout=120).splitlines()
assert chapters==qual['chapters']and set(chapters)==set(comp['chapters'])and len(chapters)==15
assert c in chapters[2:],'Completed local chapters01/02 must not be rerendered'
review=A/'repo/docs/video-program/third-wave-20261024/opus-replacements-20261104/fmd-render-readiness01/raster-quality03/production-repair17/root-apportionment-picture-repair562/pictures-viewed610.json'
assert sha(review)=='a76504654c821ab8bd7de38e0a0748d3dee9e68e2a55483fdd2de46916f58398'
v=read(review);assert v['total_selected_pictures_viewed']==379 and v['total_sheets_viewed']==49 and v['concrete_sampled_layout_defects_found']==[]
o.R=D;o.REF=T/'timeline';o.STEMS=A/'repo'/REL/'apportionment-frederick-score445/score/stems';o.FFMPEG='/usr/bin/ffmpeg';o.FFPROBE='/usr/bin/ffprobe'
assert {p.stem for p in (o.STEMS/'qa').glob('*.json')}==set(chapters)
assert all(read(o.STEMS/'qa'/(ch+'.json'))['passed']for ch in chapters)
version=subprocess.check_output([o.FFMPEG,'-version'],text=True,timeout=30)
def lib(name):
 m=re.search(r'^'+name+r'\s+(\d+)\.\s*(\d+)\.\s*(\d+)\s*/',version,re.M);assert m,name
 return '.'.join(m.groups())
o.ENCODERS=('Lavc'+lib('libavcodec')+' libx264','Lavf'+lib('libavformat'))
os.sched_setaffinity(0,{0,1,2,3});os.environ['RAYON_NUM_THREADS']='4'
cmd=[str(binary),'render',c,'--res','3840x2160','--fps','60','--lint','--narration',str(A/'repo'/REL/'apportionment-picture-assembly566/narration'),'--score',str(o.STEMS),'--out',str(J/'render'),'--crf','16','--preset','slow','--threads','4']
assert not (J/'render').exists();print('FINAL672',c,flush=True)
code,tracked=o.run_owned(c,cmd,14400);assert code==0,(c,code)
result=o.chapter(J/'render',c)
assert result['frames']==comp['chapters'][c]['frames']
with (J/'final672-summary.json').open('x')as f:json.dump(dict(chapter=c,result=result,tracking=tracked,ffmpeg_version=version,encoder_tags=o.ENCODERS,scope='Final chapter technical checks only. Final pictures, motion, listening, fine sync, cross-host master compatibility and independent AV remain open.'),f,indent=2)
print('TECHNICALLY_CHECKED',c,flush=True)
