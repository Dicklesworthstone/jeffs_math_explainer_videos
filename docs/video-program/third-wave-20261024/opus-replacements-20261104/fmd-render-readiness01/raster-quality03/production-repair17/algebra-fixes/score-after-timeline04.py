from pathlib import Path
import hashlib,json,subprocess,sys,time
R=Path.cwd();Q=Path(__file__).resolve().parent;B=R/'narration_lab/third-wave-20261024/root/five-film-release-20261027';G=R/'docs/video-program/third-wave-20261024/opus-replacements-20261104/engine-secondary-validation01/guard_owner56g_target20g.py';T=B/'algebra-fixes-timeline04'
def sha(p):
 with p.open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest()
for _ in range(2160):
 p=T/'resource-receipt.json'
 if p.exists() and 'finished' in json.loads(p.read_text()):break
 time.sleep(20)
else:raise RuntimeError('Timeline wait deadline')
d=json.loads(p.read_text());assert d['exit']==0 and d['inputs_sha256']==d['final_inputs_sha256'] and not d.get('stopped_for_resource_limit');m=Q/'timeline04-inputs.json';assert sha(m)==d['inputs_sha256'];paths={Path(__file__),Q/'score05.py',m,G}
for x in json.loads(m.read_text())['files']:
 p=Path(x['path']);assert sha(p)==x['sha256'];paths.add(p)
paths.update(p for p in T.rglob('*') if p.is_file());f=Q/'score05-inputs.json'
with f.open('x') as o:json.dump({'files':[dict(path=str(p),sha256=sha(p)) for p in sorted(paths)]},o,indent=2)
subprocess.run([sys.executable,'-I','-B',str(G),'--phase','native','--job','algebra-fixes-score05','--inputs',str(f),'--',sys.executable,'-I','-B',str(Q/'score05.py')],check=True)
