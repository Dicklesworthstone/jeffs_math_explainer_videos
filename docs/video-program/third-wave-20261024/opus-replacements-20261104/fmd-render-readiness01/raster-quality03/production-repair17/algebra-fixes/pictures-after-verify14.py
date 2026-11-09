from pathlib import Path
import json,time,subprocess,hashlib,importlib.util
R=Path('/Users/jemanuel/projects/jeffs_math_explainer_videos');B=R/'narration_lab/third-wave-20261024/root/five-film-release-20261027';Q=R/'docs/video-program/third-wave-20261024/opus-replacements-20261104/fmd-render-readiness01/raster-quality03/production-repair17';A=Q/'algebra-fixes';M=B/'algebra-final-verify14';deadline=time.monotonic()+90000
while True:
 p=M/'resource-receipt.json'
 if p.exists():
  r=json.loads(p.read_text())
  if 'exit' in r:
   assert r['exit']==0 and r['inputs_sha256']==r['final_inputs_sha256'] and not r.get('resource_stop');break
 assert time.monotonic()<deadline,'wait deadline; no verification launched'
 time.sleep(30)
rows=json.loads((A/'final-verify14-inputs.json').read_text())['files'];seen={x['path']:x for x in rows}
extra=[Path(__file__),A/'final-pictures14.py',A/'final-picture-plan14.json']+[p for p in M.rglob('*') if p.is_file()]
extra += [Path(x['path']) for x in json.loads((A/'final-picture-plan14.json').read_text())['sources']]
for p in extra:seen[str(p)]={'path':str(p),'sha256':hashlib.sha256(p.read_bytes()).hexdigest()}
for x in seen.values():assert hashlib.sha256(Path(x['path']).read_bytes()).hexdigest()==x['sha256'],x['path']
p=A/'final-pictures14-inputs.json';assert not p.exists();p.write_text(json.dumps({'files':list(seen.values())},indent=2));assert not (B/'algebra-final-pictures14').exists()
with (A/'final-pictures14-guard.log').open('x') as log:subprocess.run(['python3','-I','-B',str(Q/'guard_owner80g_target20g.py'),'--phase','native','--job','algebra-final-pictures14','--inputs',str(p),'--','python3','-I','-B',str(A/'final-pictures14.py')],cwd=R,stdout=log,stderr=subprocess.STDOUT,check=True)
