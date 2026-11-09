from pathlib import Path
import json,time,subprocess,hashlib
R=Path('/Users/jemanuel/projects/jeffs_math_explainer_videos');B=R/'narration_lab/third-wave-20261024/root/five-film-release-20261027';Q=R/'docs/video-program/third-wave-20261024/opus-replacements-20261104/fmd-render-readiness01/raster-quality03/production-repair17';A=Q/'algebra-fixes';cs=(B/'algebra-fixes-inputs02/native-chapters01.txt').read_text().splitlines();deadline=time.monotonic()+86400
while True:
 complete=True
 for c in cs:
  p=B/('algebra-final12-'+c)/'resource-receipt.json'
  if not p.exists():complete=False;continue
  d=json.loads(p.read_text())
  if 'exit' not in d:complete=False;continue
  assert d['exit']==0 and d['inputs_sha256']==d['final_inputs_sha256'] and not d.get('resource_stop'),c
 if complete:break
 assert time.monotonic()<deadline,'waiting deadline expired, no master launched'
 time.sleep(30)
rows=json.loads((A/'final12-inputs.json').read_text())['files'];seen={x['path']:x for x in rows}
extra=[Path(__file__),A/'final-master12.py']
for c in cs:
 j=B/('algebra-final12-'+c);extra.extend([j/'resource-receipt.json',j/'technical-check.json']);extra.extend(p for p in (j/'render').rglob('*') if p.is_file())
for p in extra:seen[str(p)]={'path':str(p),'sha256':hashlib.sha256(p.read_bytes()).hexdigest()}
for x in seen.values():assert hashlib.sha256(Path(x['path']).read_bytes()).hexdigest()==x['sha256'],x['path']
p=A/'final-master12-inputs.json';assert not p.exists();p.write_text(json.dumps({'files':list(seen.values())},indent=2))
assert not (B/'algebra-final-master12').exists()
with (A/'final-master12-guard.log').open('x') as log:subprocess.run(['python3','-I','-B',str(Q/'guard_owner80g_target20g.py'),'--phase','native','--job','algebra-final-master12','--inputs',str(p),'--','python3','-I','-B',str(A/'final-master12.py')],cwd=R,stdout=log,stderr=subprocess.STDOUT,check=True)
