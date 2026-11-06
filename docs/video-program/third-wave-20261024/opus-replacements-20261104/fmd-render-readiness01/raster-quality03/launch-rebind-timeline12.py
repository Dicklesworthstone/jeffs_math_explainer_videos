import json,hashlib,subprocess
from pathlib import Path
R=Path('/data/projects/jmev-fmd-render-qualification-20261106-1111')
Q=R/'qualification01/raster-quality03'
T=R/'work/raster-quality-fmd04/workspace'
def sha(p):
 with p.open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest()
def success(job):
 p=R/'jobs'/job/'resource-receipt.json';d=json.loads(p.read_text())
 assert d['exit']==0 and d['inputs_sha256']==d['final_inputs_sha256'] and not d.get('resource_stop'),job
 return p
paths={Path(__file__),R/'guard_fmd01.py',Q/'native-detail11-reviewed.json',R/'work/bin/production09i'}
for j in ['fmd-raster-quality-build05','fmd-raster-native-detail11','fmd-raster-encoder-stop11','fmd-picture-narration01']:
 paths.add(success(j));paths.update((R/'jobs'/j).glob('*.json'))
for tree in [T,R/'work/picture-repair03/workspace',R/'jobs/fmd-picture-narration01/narration',R/'production-inputs01/timelines']:
 paths.update(p for p in tree.rglob('*') if p.is_file())
paths.add(R/'qualification01/raster-quality01/original-timeline-binding09.json')
paths.update(p for p in (R/'jobs/fmd-raster-rebind11').rglob('*') if p.is_file())
for job,helper,seconds in [('fmd-raster-rebind12','rebind-after-review12.py',1800),('fmd-raster-timeline12','timeline-after-review12.py',172900)]:
 paths.add(Q/helper)
 manifest=R/(job+'-inputs.json')
 with manifest.open('x') as f:json.dump({'files':[{'path':str(p),'sha256':sha(p)} for p in sorted(paths)]},f,indent=2)
 assert not (R/'jobs'/job).exists(),job
 subprocess.run(['/usr/bin/python3','-I','-B',str(R/'guard_fmd01.py'),'--phase','native','--job',job,'--inputs',str(manifest),'--','/usr/bin/timeout',str(seconds),'/usr/bin/python3','-I','-B',str(Q/helper),'--run'],check=True)
 paths.add(success(job))
 paths.update(p for p in (R/'jobs'/job).rglob('*') if p.is_file())
 paths.update(p for p in T.rglob('*') if p.is_file())
print('Rebind and full schedule comparison complete; new full picture review remains required.')
