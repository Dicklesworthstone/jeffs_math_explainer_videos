from pathlib import Path
import json,hashlib,subprocess,time,re
R=Path('/Users/jemanuel/projects/jeffs_math_explainer_videos');B=R/'narration_lab/third-wave-20261024/root/five-film-release-20261027';Q=R/'docs/video-program/third-wave-20261024/opus-replacements-20261104/fmd-render-readiness01/raster-quality03/production-repair17';A=Q/'algebra-fixes';J=B/'algebra-stream-cap14-build';I=B/'algebra-stream-cap14-inputs';deadline=time.monotonic()+14400
while True:
 p=J/'resource-receipt.json'
 if p.exists():
  r=json.loads(p.read_text())
  if 'exit' in r:
   assert r['exit']==0 and r['inputs_sha256']==r['final_inputs_sha256'] and not r.get('resource_stop');break
 assert time.monotonic()<deadline;time.sleep(20)
assert re.search(r'test result: ok\. 123 passed; 0 failed; 0 ignored;', (I/'test.log').read_text())
assert (I/'facts.tsv').read_bytes()==(B/'algebra-fixes-inputs02/native-facts01.tsv').read_bytes()
assert (I/'chapters.txt').read_bytes()==(B/'algebra-fixes-inputs02/native-chapters01.txt').read_bytes()
assert str(B/'algebra-stream-cap12-inputs/workspace/explainer_corpus/abstract_algebra_expanded').encode() in (I/'binary').read_bytes()
rows=json.loads((A/'build-stream-cap14-inputs.json').read_text())['files'];seen={x['path']:x for x in rows}
extra=[Path(__file__),A/'final-chapter14.py',A/'final-controller14.py',J/'resource-receipt.json',J/'command.log']+[I/n for n in ['binary','test.log','build.log','clippy.log','facts.tsv','chapters.txt']]
for p in extra:seen[str(p)]={'path':str(p),'sha256':hashlib.sha256(p.read_bytes()).hexdigest()}
for x in seen.values():assert hashlib.sha256(Path(x['path']).read_bytes()).hexdigest()==x['sha256'],x['path']
p=A/'final14-inputs.json';assert not p.exists();p.write_text(json.dumps({'files':list(seen.values())},indent=2))
subprocess.run(['python3','-I','-B',str(A/'final-controller14.py')],cwd=R,check=True)
