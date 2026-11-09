from pathlib import Path
import json,time,subprocess,hashlib,importlib.util
R=Path('/Users/jemanuel/projects/jeffs_math_explainer_videos');B=R/'narration_lab/third-wave-20261024/root/five-film-release-20261027';Q=R/'docs/video-program/third-wave-20261024/opus-replacements-20261104/fmd-render-readiness01/raster-quality03/production-repair17';A=Q/'algebra-fixes';M=B/'algebra-final-master11';deadline=time.monotonic()+90000
while True:
 p=M/'resource-receipt.json'
 if p.exists():
  r=json.loads(p.read_text())
  if 'exit' in r:
   assert r['exit']==0 and r['inputs_sha256']==r['final_inputs_sha256'] and not r.get('resource_stop');break
 assert time.monotonic()<deadline,'wait deadline; no verification launched'
 time.sleep(30)
rows=json.loads((A/'final-master11-inputs.json').read_text())['files'];seen={x['path']:x for x in rows}
shared=R/'docs/video-program/third-wave-20261024/opus-replacements-20261104/logarithm-stage09-final-verification02/verify_stage09_final.py'
spec=importlib.util.spec_from_file_location('shared_verify',shared);v=importlib.util.module_from_spec(spec);spec.loader.exec_module(v)
extra=[Path(__file__),A/'final-verify11.py',shared,R/v.STRICT_REL,R/v.SCREEN_REL]+[p for p in M.rglob('*') if p.is_file()]
for p in extra:seen[str(p)]={'path':str(p),'sha256':hashlib.sha256(p.read_bytes()).hexdigest()}
for x in seen.values():assert hashlib.sha256(Path(x['path']).read_bytes()).hexdigest()==x['sha256'],x['path']
p=A/'final-verify11-inputs.json';assert not p.exists();p.write_text(json.dumps({'files':list(seen.values())},indent=2));assert not (B/'algebra-final-verify11').exists()
with (A/'final-verify11-guard.log').open('x') as log:subprocess.run(['python3','-I','-B',str(Q/'guard_owner80g_target20g.py'),'--phase','native','--job','algebra-final-verify11','--inputs',str(p),'--','python3','-I','-B',str(A/'final-verify11.py')],cwd=R,stdout=log,stderr=subprocess.STDOUT,check=True)
