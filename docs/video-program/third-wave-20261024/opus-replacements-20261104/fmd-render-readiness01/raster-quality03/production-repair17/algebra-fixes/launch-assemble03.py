from pathlib import Path
import json,hashlib,subprocess,importlib.util,sys
sys.dont_write_bytecode=True
R=Path.cwd();Q=Path(__file__).resolve().parent;B=R/'narration_lab/third-wave-20261024/root/five-film-release-20261027';I=B/'algebra-fixes-inputs02';G=R/'docs/video-program/third-wave-20261024/opus-replacements-20261104/engine-secondary-validation01/guard_owner56g_target20g.py'
def sha(p):
 with p.open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest()
for name in ['algebra-fixes-build02','algebra-repair17-decode01','algebra-repair17-recognition01']:
 d=json.loads((B/name/'resource-receipt.json').read_text());assert d['exit']==0 and d['inputs_sha256']==d['final_inputs_sha256'] and not d.get('stopped_for_resource_limit')
paths={Path(__file__),Q/'assemble03.py',Q/'11_17-frederick.txt',G}
m=I/'build02-inputs.json';paths.add(m)
for x in json.loads(m.read_text())['files']:
 p=Path(x['path']);assert sha(p)==x['sha256'];paths.add(p)
for name in ['algebra-fixes-inputs02','algebra-fixes-prepare02','algebra-fixes-build02','algebra-repair17-decode01','algebra-repair17-recognition01']:
 paths.update(p for p in (B/name).rglob('*') if p.is_file())
old=B/'algebra-combined-repair-timeline01/narration';paths.update(p for p in old.rglob('*') if p.is_file())
for name in ['algebra-combined-repair-timeline01','algebra-corner-contrast-timeline01']:paths.add(B/name/'resource-receipt.json')
a=json.loads((old/'assembly.json').read_text())
for row in a['manifest']['units']:
 for k in ['wav','generation_receipt']:
  p=Path(row[k]);p=p if p.is_absolute() else R/p;paths.add(p)
 receipt=json.loads(p.read_text())
 for rec in receipt.get('records',[]):
  for k in ['wav','source','request']:
   if rec.get(k):
    p=Path(rec[k]);p=p if p.is_absolute() else R/p
    if p.is_file():paths.add(p)
for rec in json.loads((B/'algebra-repair17-decode01/decode-receipt.json').read_text())['records']:
 paths.update(Path(rec[k]) for k in ['wav','source','request'])
bridge=R/'docs/video-program/third-wave-20261024/opus-replacements-20261104/complex-frederick-stage07-reuse01/frederick_timing_bridge_stage07.py';paths.add(bridge)
s=importlib.util.spec_from_file_location('bridge',bridge);mod=importlib.util.module_from_spec(s);s.loader.exec_module(mod)
for rel,h in mod.DEPENDENCIES.values():
 p=R/rel;assert sha(p)==h;paths.add(p)
f=Q/'assemble03-inputs.json';assert not(B/'algebra-fixes-assemble03').exists()
with f.open('x') as o:json.dump({'files':[dict(path=str(p),sha256=sha(p)) for p in sorted(paths)]},o,indent=2)
subprocess.run([sys.executable,'-I','-B',str(G),'--phase','native','--job','algebra-fixes-assemble03','--inputs',str(f),'--',sys.executable,'-I','-B',str(Q/'assemble03.py'),'run'],check=True)
