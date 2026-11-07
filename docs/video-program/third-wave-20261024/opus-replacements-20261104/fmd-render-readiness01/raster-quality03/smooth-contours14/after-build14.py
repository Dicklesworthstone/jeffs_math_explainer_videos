import json,hashlib,subprocess,time,os
from pathlib import Path
R=Path('/data/projects/jmev-stroke-probe-20261106');Q=R/'qualification01/smooth-contours14'
os.sched_setaffinity(0,{0,1,2,3})
def sha(p):
 with p.open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest()
def success(j):
 d=json.loads((R/'jobs'/j/'resource-receipt.json').read_text());assert d['exit']==0 and d['inputs_sha256']==d['final_inputs_sha256'] and not d.get('stopped_for_resource_limit') and not d.get('resource_stop');return d
p=R/'jobs/csd-smooth-contours-build14/resource-receipt.json'
for _ in range(4320):
 try:d=json.loads(p.read_text())
 except (FileNotFoundError,json.JSONDecodeError):d={}
 if 'exit' in d:break
 time.sleep(10)
else:raise TimeoutError('Build prerequisite not terminal after12h')
success('csd-smooth-contours-build14')
paths={Path(__file__),Q/'stop14.py',Q/'detail14.py',R/'work/bin/production14'}
for row in json.loads((Q/'inputs14.json').read_text())['files']:
 p=Path(row['path']);assert sha(p)==row['sha256'];paths.add(p)
paths.update((R/'jobs/csd-smooth-contours-build14').glob('*.json'))
for job,script,args in [('csd-smooth-encoder-stop14','stop14.py',['--run']),('csd-smooth-detail14-04_reading_color','detail14.py',['--chapter','04_reading_color']),('csd-smooth-detail14-05_gallery','detail14.py',['--chapter','05_gallery'])]:
 assert not (R/'jobs'/job).exists()
 m=Q/(job+'-inputs.json')
 with m.open('x') as f:json.dump({'files':[{'path':str(p),'sha256':sha(p)} for p in sorted(paths)]},f,indent=2)
 subprocess.run(['/usr/bin/python3','-I','-B',str(R/'guard_csd02.py'),'--phase','native','--job',job,'--inputs',str(m),'--','/usr/bin/timeout','9000','/usr/bin/python3','-I','-B',str(Q/script),*args],check=True)
 success(job);paths.update((R/'jobs'/job).glob('*.json'))
print('Two native diagnostic chapters extracted. Actual picture and motion review remain open.')
