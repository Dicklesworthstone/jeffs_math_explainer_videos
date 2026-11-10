import json,hashlib,subprocess,time,sys,os
from pathlib import Path
Q=Path(__file__).resolve().parent
R=Q.parents[6]
W=R/'narration_lab/third-wave-20261024/root/five-film-release-20261027'
M=Path('/Users/jemanuel/scratch/jmev-complex-native-20261109')
def sha(p):
 with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest()
def read(p):return json.loads(Path(p).read_text())
def success(p):
 d=read(p);assert d.get('admitted') and d.get('finished') and d.get('exit')==0 and not any(d.get(k) for k in ('stopped_for_resource_limit','error','preflight_only'))
 assert d['inputs_sha256']==d['final_inputs_sha256'];return d
receipt=W/'complex-local-delivery818/resource-receipt.json'
for _ in range(360):
 if receipt.exists() and read(receipt).get('finished'):break
 time.sleep(10)
else:raise RuntimeError('818 observation deadline; no duplicate or render launched')
d=success(receipt)
manifest=Q/'local-delivery818-inputs.json';assert sha(manifest)==d['inputs_sha256']
rows=read(manifest)['files'];bound={r['path']:r['sha256'] for r in rows};assert len(bound)==len(rows)
for p,h in bound.items():assert sha(p)==h,p
assert M.is_symlink() and M.resolve()==W/'complex-mini-transfer812/tree'
review=M/'qualification/root-boundary775-pictures-reviewed.json';ind=M/'qualification/root-boundary787-independent-review.json'
a=read(review);b=read(ind)
assert a['complete'] and a['concrete_defects']==[]
assert b['safe_to_advance_to_final4k'] and b['blockers']==[] and b['authored_repair'] is False
assert b['source_sha256']=='cd1900aadd46090083729d9aa768e59e1551c42ab94e5bd055f520cb4eff5e6e'
binary=M/'jobs/native-boundary761/complex_analysis_visual-761'
assert sha(binary)=='71736c26d9953f08af8f4ce4f6fc0f13aeb98e7135b3dd415fb3ebf00e161c26'
assert sha('/opt/homebrew/bin/ffmpeg')=='1332dc2de372bade9a8a63da0d6cdfab9de97fcefbae707bcc0b0506e1203327'
nar=W/'complex-local-delivery818/narration'
assert read(nar.parent/'rebind818.json')['wavs_byte_identical']==105
for p in list(nar.iterdir())+[nar.parent/'rebind818.json',receipt,Path(__file__),Q/'local-guard814.py',Q/'local-render819-exec.py',review,ind]:bound[str(p)]=sha(p)
inputs=Q/'local-final819-inputs.json'
with inputs.open('x') as f:json.dump({'files':[{'path':p,'sha256':h} for p,h in sorted(bound.items())]},f,indent=2)
job=W/'complex-local-final819';assert not job.exists()
cmd=[sys.executable,'-I','-B',str(Q/'local-guard814.py'),'--phase','native','--job',job.name,'--inputs',str(inputs),'--',sys.executable,'-I','-B',str(Q/'local-render819-exec.py'),str(binary),'render','09_boundary','--res','3840x2160','--fps','60','--threads','4','--narration',str(nar),'--score',str(M/'score586'),'--out',str(job/'render'),'--lint','--crf','16','--preset','veryfast']
env=dict(os.environ,PATH='/opt/homebrew/bin:/Users/jemanuel/.local/bin:/usr/bin:/bin')
raise SystemExit(subprocess.call(cmd,env=env))
