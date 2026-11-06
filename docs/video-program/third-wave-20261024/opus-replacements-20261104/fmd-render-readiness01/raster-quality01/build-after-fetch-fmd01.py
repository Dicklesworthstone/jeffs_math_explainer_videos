import json,hashlib,subprocess,time
from pathlib import Path
R=Path('/data/projects/jmev-fmd-render-qualification-20261106-1111')
p=R/'jobs/fmd-raster-fetch01/resource-receipt.json'
for attempt in range(1800):
 try:d=json.loads(p.read_text())
 except (FileNotFoundError,json.JSONDecodeError):d={}
 if 'exit' in d:break
 time.sleep(2)
else:raise RuntimeError('fetch waiting deadline')
assert d['exit']==0 and d['inputs_sha256']==d['final_inputs_sha256'] and not d.get('resource_stop'),d
base=json.loads((R/'fmd-raster-fetch01-inputs.json').read_text())
for row in base['files']:
 with Path(row['path']).open('rb') as f:assert hashlib.file_digest(f,'sha256').hexdigest()==row['sha256'],row['path']
helper=R/'qualification01/raster-quality01/build-fmd03.py'
for f in (helper,p):base['files'].append({'path':str(f),'sha256':hashlib.sha256(f.read_bytes()).hexdigest()})
manifest=R/'fmd-raster-quality-build03-inputs.json'
with manifest.open('x') as f:json.dump(base,f,indent=2)
assert not (R/'jobs/fmd-raster-quality-build03').exists()
subprocess.run(['env','CARGO_BUILD_JOBS=2','/usr/bin/python3','-I','-B',str(R/'guard_fmd01.py'),'--phase','native','--job','fmd-raster-quality-build03','--inputs',str(manifest),'--','/usr/bin/timeout','7200','/usr/bin/python3','-I','-B',str(helper),'--run'],check=True)
