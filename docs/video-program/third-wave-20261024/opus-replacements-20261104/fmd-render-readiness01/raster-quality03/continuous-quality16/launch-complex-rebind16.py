from pathlib import Path
import json,hashlib,subprocess
R=Path('/data/projects/jmev-stroke-probe-20261106');Q=R/'qualification01/continuous-quality16';TREE=R/'work/continuous-quality16/workspace'
def sha(p):
 with p.open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest()
assert not (R/'jobs/csd-continuous-rebind16').exists()
paths=set()
canonical=TREE/'explainer_corpus/complex_analysis_visual/narration_units.json'
for row in json.loads((Q/'inputs16.json').read_text())['files']:
 p=Path(row['path']);assert sha(p)==row['sha256'],str(p)
 if p!=canonical:paths.add(p)
assert canonical.read_bytes()==(Q/'before-rebind-units16.json').read_bytes()
for name in ['native-detail16-reviewed.json','rebind16.py','marker-relocation-audit01.json','before-rebind-map16.json','before-rebind-units16.json','narration-direction-map-frederick-quality16.json','narration-units-quality16.json']:paths.add(Q/name)
for name in ['csd-continuous-quality-build16','csd-continuous-encoder-stop16','csd-continuous-detail16-04_reading_color','csd-continuous-detail16-05_gallery','csd-continuous-sheets16']:
 d=json.loads((R/'jobs'/name/'resource-receipt.json').read_text())
 assert d['exit']==0 and d['inputs_sha256']==d['final_inputs_sha256'] and not d.get('resource_stop') and not d.get('stopped_for_resource_limit')
 paths.update((R/'jobs'/name).glob('*.json'))
paths.add(R/'work/bin/production16')
paths.update(p for p in (R/'jobs/csd-raster-media15/narration').rglob('*') if p.is_file())
paths.update(p for p in (TREE/'tools').rglob('*.py') if p.is_file())
paths.add(Path(__file__))
m=Q/'rebind16-inputs.json'
with m.open('x') as f:json.dump({'files':[{'path':str(p),'sha256':sha(p)} for p in sorted(paths)],'intended_outputs':['jobs/csd-continuous-rebind16',str(canonical),'quality16 direction map and canonical backup']},f,indent=2)
subprocess.run(['/usr/bin/python3','-I','-B',str(R/'guard_csd02.py'),'--phase','native','--job','csd-continuous-rebind16','--inputs',str(m),'--','/usr/bin/timeout','3600','/usr/bin/python3','-I','-B',str(Q/'rebind16.py'),'--run'],check=True)
