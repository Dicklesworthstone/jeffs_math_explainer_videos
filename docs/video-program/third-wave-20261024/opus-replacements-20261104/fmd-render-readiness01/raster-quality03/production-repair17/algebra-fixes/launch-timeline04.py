from pathlib import Path
import json,hashlib,subprocess,sys
R=Path.cwd();Q=Path(__file__).resolve().parent;B=R/'narration_lab/third-wave-20261024/root/five-film-release-20261027';G=R/'docs/video-program/third-wave-20261024/opus-replacements-20261104/engine-secondary-validation01/guard_owner56g_target20g.py'
def sha(p):
 with p.open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest()
a=json.loads((B/'algebra-fixes-assemble03/resource-receipt.json').read_text());assert a['exit']==0 and a['inputs_sha256']==a['final_inputs_sha256'] and not a.get('stopped_for_resource_limit')
m=Q/'assemble03-inputs.json';assert sha(m)==a['inputs_sha256'];paths={Path(__file__),Q/'timeline04.py',m,G}
for x in json.loads(m.read_text())['files']:
 p=Path(x['path']);assert sha(p)==x['sha256'];paths.add(p)
paths.update(p for p in (B/'algebra-fixes-assemble03').rglob('*') if p.is_file());f=Q/'timeline04-inputs.json'
with f.open('x') as o:json.dump({'files':[dict(path=str(p),sha256=sha(p)) for p in sorted(paths)]},o,indent=2)
subprocess.run([sys.executable,'-I','-B',str(G),'--phase','native','--job','algebra-fixes-timeline04','--inputs',str(f),'--',sys.executable,'-I','-B',str(Q/'timeline04.py')],check=True)
