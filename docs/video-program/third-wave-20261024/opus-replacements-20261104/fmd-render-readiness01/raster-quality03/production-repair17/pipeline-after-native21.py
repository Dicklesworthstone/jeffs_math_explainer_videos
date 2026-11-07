from pathlib import Path
import json,hashlib,time,subprocess
R=Path('/data/projects/jmev-stroke-probe-20261106');Q=R/'qualification01/production-repair17';OLD=R/'jobs/csd-production-timeline21'
def sha(p):
 with p.open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest()
def guard(job,manifest,helper):
 subprocess.run(['/usr/bin/python3','-I','-B',str(R/'guard_csd02.py'),'--phase','native','--job',job,'--inputs',str(manifest),'--','/usr/bin/timeout','7300','/usr/bin/python3','-I','-B',str(helper)],check=True)
def freeze(out,manifest,folders,extra):
 paths=set(extra)|{manifest};d=json.loads(manifest.read_text())
 for x in d['files']:
  p=Path(x['path']);assert sha(p)==x['sha256'];paths.add(p)
 for f in folders:paths.update(p for p in f.rglob('*') if p.is_file())
 with out.open('x') as o:json.dump({'files':[dict(path=str(p),sha256=sha(p)) for p in sorted(paths)]},o,indent=2)
for _ in range(2160):
 p=OLD/'resource-receipt.json'
 if p.exists() and 'finished' in json.loads(p.read_text()):break
 time.sleep(20)
else:raise RuntimeError('Native timeline wait deadline')
d=json.loads(p.read_text());assert d['exit']==1 and d['inputs_sha256']==d['final_inputs_sha256'] and not d.get('stopped_for_resource_limit')
f=Q/'timeline22-inputs.json';freeze(f,Q/'timeline21-inputs.json',[OLD],[Path(__file__),Q/'qualify-timeline22.py',R/'guard_csd02.py']);guard('csd-production-timeline22',f,Q/'qualify-timeline22.py')
s=Q/'score22-inputs.json';freeze(s,f,[R/'jobs/csd-production-timeline22'],[Q/'score22.py']);guard('csd-production-score22',s,Q/'score22.py')
subprocess.run(['/usr/bin/python3','-I','-B',str(Q/'launch-preview22.py'),'--phase','preview'],check=True)
