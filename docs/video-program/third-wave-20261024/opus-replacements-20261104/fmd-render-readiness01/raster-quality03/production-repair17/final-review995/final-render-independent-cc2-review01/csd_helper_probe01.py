# Read-only on CSD: facts the repaired-final helpers rely on but CC3's local checker cannot see. Writes nothing.
import hashlib,json,os,re,sys
from pathlib import Path
D=Path('/data/projects/jmev-stroke-probe-20261106');Q=D/'qualification01/production-repair17'
def sha(p):
 with open(p,'rb') as f:return hashlib.file_digest(f,'sha256').hexdigest()
def j(p):
 try:return json.loads(Path(p).read_text())
 except Exception as e:return {'_error':repr(e)[:120]}
out={}
runner=Q/'csd-native-repair-final01-run.py'
out['q_resolves_to_itself']=str(Q.resolve())==str(Q);out['D_resolved']=str(D.resolve());out['sys_executable']=sys.executable
out['usr_bin_python3_is_symlink']=os.path.islink('/usr/bin/python3')
out['runner_or_launch_staged']=[p.name for p in Q.glob('csd-native-repair-final01*')]
out['jobs_named_repair_final']=sorted(x for x in os.listdir(D/'jobs') if 'repair-final' in x)
g=(D/'guard_csd02.py').read_text();out['guard_sha256']=sha(D/'guard_csd02.py')
out['guard_lock_lines']=[l.strip()[:160] for l in g.splitlines() if re.search(r'flock|lockf|LOCK_|exclusive|O_EXCL|lock',l)][:12]
out['guard_command_lines']=[l.strip()[:160] for l in g.splitlines() if "'command'" in l or '"command"' in l][:6]
b=D/'jobs/csd-native-combined-build01';q=j(b/'qualification.json')
out['build01_qualification_keys']=sorted(q)[:40];out['build01']={k:q.get(k) for k in ('binary','binary_sha256','tests_passed','clippy_passed','explicit_native_plate_fade_probe_passed')}
out['build01_binary_sha_matches']=(sha(b/'complex_analysis_visual-combined01')==q.get('binary_sha256')) if (b/'complex_analysis_visual-combined01').is_file() else 'missing'
d=j(D/'jobs/csd-native-combined-delivery01/delivery-combined01.json')
out['delivery01']={'wavs_byte_identical':d.get('wavs_byte_identical'),'repaired_selected':(d.get('delivery') or {}).get('repaired',{}).get('selected_cues') if isinstance(d.get('delivery'),dict) else None,'delivery_keys':sorted((d.get('delivery') or {}).keys()) if isinstance(d.get('delivery'),dict) else None}
out['delivery01_wavs']=len(list((D/'jobs/csd-native-combined-delivery01/narration').glob('*.wav')))
for name,p in [('timeline01',D/'jobs/csd-native-combined-timeline01/comparison-combined01.json'),('preview01',D/'jobs/csd-native-combined-preview01/comparison-combined01.json')]:
 c=j(p);ch=c.get('chapters') if isinstance(c,dict) else None
 out[name+'_comparison']={k:{kk:(v.get(kk) if isinstance(v,dict) else None) for kk in ('frames','normalized_sha256','actual_sha256')} for k,v in ch.items()} if isinstance(ch,dict) else c
for ch in ['01_the_bump','08_loop_sums','09_boundary']:
 t=D/'jobs/csd-native-combined-timeline01/timeline-output'/(ch+'.timeline.tsv');out['timeline01_table_'+ch]=sha(t) if t.is_file() else None
 p=D/'jobs/csd-native-combined-preview01/preview/soundtrack'/ch/'soundtrack_0.wav';out['preview01_pcm_'+ch]=sha(p) if p.is_file() else None
 pt=D/'jobs/csd-native-combined-preview01/preview'/(ch+'.timeline.tsv');out['preview01_table_'+ch]=sha(pt) if pt.is_file() else None
src=D/'work/native-combined01/workspace/explainer_corpus/complex_analysis_visual/src/chapters'
out['scenes_combined01']={n:sha(src/n) for n in ('the_bump.rs','loop_sums.rs','boundary.rs','counting.rs') if (src/n).is_file()}
st=D/'jobs/csd-production-score21/score/stems'
out['stems']={}
for ch in ['01_the_bump','08_loop_sums','09_boundary','11_counting']:
 out['stems'][ch]={p.name:sha(p) for p in sorted(st.glob(ch+'.*'))};qa=st/'qa'/(ch+'.json');out['stems'][ch]['qa/'+ch+'.json']=sha(qa) if qa.is_file() else None
out['sampling']={}
for f in ['combined-pictures01/frames-01_the_bump-all.txt','combined-pictures01/frames-08_loop_sums-all.txt','combined-pictures01/frames-09_boundary-all.txt','counting-pictures02/frames-11_counting-all.txt']:
 p=Q/f;out['sampling'][f]=(sha(p),len(p.read_text().split())) if p.is_file() else None
out['pictures_inputs_present']={n:(Q/n).is_file() for n in ('csd-combined01-pictures-inputs.json','csd-counting02-pictures-inputs.json')}
out['prior775']=sha(Q/'csd-native855-prior775.json') if (Q/'csd-native855-prior775.json').is_file() else None
out['counting_jobs']=sorted(x for x in os.listdir(D/'jobs') if 'counting' in x)
print(json.dumps(out))
