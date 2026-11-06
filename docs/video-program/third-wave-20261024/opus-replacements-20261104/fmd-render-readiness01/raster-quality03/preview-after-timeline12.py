import json,hashlib,subprocess,time
from pathlib import Path
R=Path('/data/projects/jmev-fmd-render-qualification-20261106-1111');Q=R/'qualification01/raster-quality03'
def sha(p):
 with p.open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest()
p=R/'jobs/fmd-raster-timeline12/resource-receipt.json'
for _ in range(86400):
 try:d=json.loads(p.read_text())
 except (FileNotFoundError,json.JSONDecodeError):d={}
 if 'exit' in d:break
 time.sleep(2)
else:raise RuntimeError('Timeline waiting deadline')
assert d['exit']==0 and d['inputs_sha256']==d['final_inputs_sha256'] and not d.get('resource_stop')
base=json.loads((R/'fmd-raster-timeline12-inputs.json').read_text())
paths={Path(__file__),Q/'preview12.py',Q/'original-score-summary12.json',Q/'original-score-receipt12.json',Q/'original-score-files12.json'}
for row in base['files']:
 p=Path(row['path']);assert sha(p)==row['sha256'],p;paths.add(p)
for folder in [R/'jobs/fmd-raster-timeline12',R/'production-inputs01/scored']:
 paths.update(p for p in folder.rglob('*') if p.is_file())
manifest=R/'fmd-raster-preview12-inputs.json'
with manifest.open('x') as f:json.dump({'files':[{'path':str(p),'sha256':sha(p)} for p in sorted(paths)]},f,indent=2)
assert not (R/'jobs/fmd-raster-preview12').exists()
subprocess.run(['/usr/bin/python3','-I','-B',str(R/'guard_fmd01.py'),'--phase','native','--job','fmd-raster-preview12','--inputs',str(manifest),'--','/usr/bin/timeout','172900','/usr/bin/python3','-I','-B',str(Q/'preview12.py'),'--run'],check=True)
print('Narrated preview terminal; pictures remain unreviewed.')
