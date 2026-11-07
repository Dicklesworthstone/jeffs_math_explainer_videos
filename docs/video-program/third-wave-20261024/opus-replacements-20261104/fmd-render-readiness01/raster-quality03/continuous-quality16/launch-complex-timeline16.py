from pathlib import Path
import json,hashlib,subprocess
R=Path('/data/projects/jmev-stroke-probe-20261106');Q=R/'qualification01/continuous-quality16';T=R/'work/continuous-quality16/workspace';J='csd-continuous-timeline16'
def sha(p):
 with p.open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest()
assert not (R/'jobs'/J).exists()
paths={Path(__file__),Q/'timeline16.py',Q/'original-timeline-binding09.json',R/'guard_csd02.py'}
for row in json.loads((Q/'rebind16-inputs.json').read_text())['files']:
 p=Path(row['path']);assert sha(p)==row['sha256'],str(p);paths.add(p)
r=json.loads((R/'jobs/csd-continuous-rebind16/resource-receipt.json').read_text())
assert r['exit']==0 and r['inputs_sha256']==r['final_inputs_sha256'] and not r.get('resource_stop') and not r.get('stopped_for_resource_limit')
for folder in [T,R/'jobs/csd-continuous-rebind16',Q/'original-timelines']:
 paths.update(p for p in folder.rglob('*') if p.is_file())
m=Q/'timeline16-inputs.json'
with m.open('x') as f:json.dump({'files':[{'path':str(p),'sha256':sha(p)} for p in sorted(paths)]},f,indent=2)
subprocess.run(['/usr/bin/python3','-I','-B',str(R/'guard_csd02.py'),'--phase','native','--job',J,'--inputs',str(m),'--','/usr/bin/timeout','172900','/usr/bin/python3','-I','-B',str(Q/'timeline16.py'),'--run'],check=True)
