import json,hashlib,time,subprocess
from pathlib import Path
R=Path('/data/projects/jmev-stroke-probe-20261106');Q=R/'qualification01/continuous-quality16';paths={Path(__file__),Q/'sheets16.py'}
def sha(p):
 with p.open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest()
for job in ['csd-continuous-quality-build16','csd-continuous-encoder-stop16','csd-continuous-detail16-04_reading_color','csd-continuous-detail16-05_gallery']:
 for _ in range(4320):
  try:d=json.loads((R/'jobs'/job/'resource-receipt.json').read_text())
  except (FileNotFoundError,json.JSONDecodeError):d={}
  if 'exit' in d:break
  time.sleep(10)
 else:raise TimeoutError(job)
 assert d['exit']==0 and d['inputs_sha256']==d['final_inputs_sha256'] and not d.get('resource_stop') and not d.get('stopped_for_resource_limit'),job
 paths.update((R/'jobs'/job).glob('*.json'))
 if 'detail16' in job:
  summary=json.loads((R/'jobs'/job/'detail-summary.json').read_text());paths.update(Path(p['path']) for p in summary['pictures'])
for row in json.loads((Q/'inputs16.json').read_text())['files']:
 p=Path(row['path']);assert sha(p)==row['sha256'];paths.add(p)
m=Q/'sheets16-inputs.json'
with m.open('x') as f:json.dump({'files':[{'path':str(p),'sha256':sha(p)} for p in sorted(paths)]},f,indent=2)
subprocess.run(['/usr/bin/python3','-I','-B',str(R/'guard_csd02.py'),'--phase','native','--job','csd-continuous-sheets16','--inputs',str(m),'--','/usr/bin/timeout','1800','/usr/bin/python3','-I','-B',str(Q/'sheets16.py')],check=True)
