import json,hashlib,subprocess,time
from pathlib import Path
R=Path('/data/projects/jmev-stroke-probe-20261106');Q=R/'qualification01/continuous-painter15'
def sha(p):
 with p.open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest()
for job in ['csd-smooth-contours-build14','csd-smooth-detail14-04_reading_color','csd-smooth-detail14-05_gallery']:
 for _ in range(4320):
  try:d=json.loads((R/'jobs'/job/'resource-receipt.json').read_text())
  except (FileNotFoundError,json.JSONDecodeError):d={}
  if 'exit' in d:break
  time.sleep(10)
 else:raise TimeoutError(job)
 assert d['exit']==0 and d['inputs_sha256']==d['final_inputs_sha256'] and not d.get('resource_stop') and not d.get('stopped_for_resource_limit'),job
paths=set(Q.iterdir())
for row in json.loads((R/'qualification01/smooth-contours14/inputs14.json').read_text())['files']:
 p=Path(row['path']);assert sha(p)==row['sha256'];paths.add(p)
paths.update(p for p in (R/'work/smooth-contours14/workspace').rglob('*') if p.is_file())
paths.update((R/'jobs/csd-smooth-contours-build14').glob('*.json'))
m=Q/'inputs15.json'
with m.open('x') as f:json.dump({'files':[{'path':str(p),'sha256':sha(p)} for p in sorted(paths)]},f,indent=2)
subprocess.run(['/usr/bin/python3','-I','-B',str(R/'guard_csd02.py'),'--phase','native','--job','csd-continuous-painter-build15','--inputs',str(m),'--','/usr/bin/timeout','10800','/usr/bin/python3','-I','-B',str(Q/'build15.py')],check=True)
print('Continuous painter build returned; inspect terminal receipt and native motion before acceptance.')
