import json,hashlib,subprocess
from pathlib import Path
R=Path('/data/projects/jmev-stroke-probe-20261106');Q=R/'qualification01/raster-quality12'
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
paths={Path(__file__),Q/'csd-detail13.py'}
for r in json.loads((R/'csd-raster-encoder-stop13-inputs.json').read_text())['files']:
 p=Path(r['path']);assert sha(p)==r['sha256'],p;paths.add(p)
for j in ['csd-raster-stage13','csd-raster-encoder-stop13']:
 p=R/'jobs'/j/'resource-receipt.json';d=json.loads(p.read_text());assert d['exit']==0 and d['inputs_sha256']==d['final_inputs_sha256'] and not d.get('resource_stop');paths.update((R/'jobs'/j).glob('*.json'))
for c in ['05_gallery','10_rigidity','14_continuation']:
 job='csd-raster-detail13-'+c;assert not (R/'jobs'/job).exists()
 m=R/(job+'-inputs.json')
 with m.open('x') as f:json.dump({'files':[{'path':str(p),'sha256':sha(p)} for p in sorted(paths)]},f,indent=2)
 subprocess.run(['/usr/bin/python3','-I','-B',str(R/'guard_csd02.py'),'--phase','native','--job',job,'--inputs',str(m),'--','/usr/bin/timeout','9000','/usr/bin/python3','-I','-B',str(Q/'csd-detail13.py'),'--chapter',c],check=True)
 p=R/'jobs'/job/'resource-receipt.json';d=json.loads(p.read_text());assert d['exit']==0 and d['inputs_sha256']==d['final_inputs_sha256'] and not d.get('resource_stop')
print('Three native4K diagnostic extracts complete; actual viewing remains mandatory.')
