import json,hashlib,subprocess,time
from pathlib import Path
R=Path('/data/projects/jmev-fmd-render-qualification-20261106-1111')
def sha(p):
 with p.open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest()
def success(job):
 p=R/'jobs'/job/'resource-receipt.json'
 for _ in range(7200):
  try:d=json.loads(p.read_text())
  except (FileNotFoundError,json.JSONDecodeError):d={}
  if 'exit' in d:break
  time.sleep(2)
 else:raise RuntimeError('Waiting deadline '+job)
 assert d['exit']==0 and d['inputs_sha256']==d['final_inputs_sha256'] and not d.get('resource_stop'),d
 return p
prior=success('fmd-raster-quality-build03')
base=json.loads((R/'fmd-raster-quality-build03-inputs.json').read_text())
for row in base['files']:assert sha(Path(row['path']))==row['sha256'],row['path']
paths={Path(row['path']) for row in base['files']}
for root in (R/'work/raster-quality-fmd02/workspace',R/'jobs/fmd-raster-quality-build03'):
 paths.update(p for p in root.rglob('*') if p.is_file())
paths.add(R/'work/bin/production09g')
for job,helper in [('fmd-raster-encoder-stop09','encoder-stop09.py'),('fmd-raster-native-detail09','native-detail09.py')]:
 p=R/'qualification01/raster-quality01'/helper; paths.add(p)
 manifest=R/(job+'-inputs.json')
 with manifest.open('x') as f:json.dump({'files':[{'path':str(x),'sha256':sha(x)} for x in sorted(paths)]},f,indent=2)
 assert not (R/'jobs'/job).exists()
 subprocess.run(['/usr/bin/python3','-I','-B',str(R/'guard_fmd01.py'),'--phase','native','--job',job,'--inputs',str(manifest),'--','/usr/bin/timeout','9000','/usr/bin/python3','-I','-B',str(p),'--run'],check=True)
 paths.add(success(job));paths.update(x for x in (R/'jobs'/job).glob('*.json'))
print('Native diagnostic completed; every image remains unviewed until actual review. No production permit.')
