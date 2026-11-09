from pathlib import Path
import json,hashlib,subprocess,importlib.util
R=Path('/data/projects/jmev-render-worker-20261107');Q=R/'qualification01/production19';D=Path('/data/projects/jmev-stroke-probe-20261106')
helper=Q/'trj-final-chapter19-02.py';permit=Q/'review02-32/final-render-permit32.json'
spec=importlib.util.spec_from_file_location('worker',helper);m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m)
import time
prior=R/'jobs/trj-production19-final-01-the-bump30/resource-receipt.json'
for _ in range(1440):
 if prior.exists():
  state=m.read(prior)
  if state.get('finished'):
   m.success(prior);break
 time.sleep(60)
else:raise RuntimeError('Predecessor waiting deadline; no render launched')
review,_=m.check('02_turning_numbers',permit)
files=set([helper,permit,Path(__file__).resolve(),R/'guard_trj01.py',R/'jobs/trj-production19-delivery02/resource-receipt.json',D/'jobs/csd-production-timeline21/resource-receipt.json',D/'jobs/csd-production-score21/resource-receipt.json',D/'jobs/csd-production-repair-build19/qualification.json',D/'jobs/csd-production-score21/score21-summary.json'])
inherited=Q/'delivery02-inputs.json';files.add(inherited);files.update(Path(x['path']) for x in m.read(inherited)['files']);files.update(p for p in m.NARRATION.rglob('*') if p.is_file());files.update(D/'jobs/csd-production-score21/score'/p for p in m.read(D/'jobs/csd-production-score21/score21-summary.json')['files']);files.update(Path(x['path']) for x in review['evidence']);files.add(m.BIN)
for suffix in ['timeline.tsv','cues.tsv','cuelog.tsv']:files.add(m.TIMELINES/('02_turning_numbers.'+suffix))
inputs=Q/'final02-32-inputs.json'
with inputs.open('x') as f:json.dump({'files':[{'path':str(p),'sha256':m.sha(p)} for p in sorted(files)]},f,indent=2)
job='trj-production19-final-02-turning-numbers32';assert not (R/'jobs'/job).exists()
subprocess.run(['python3','-I','-B',str(R/'guard_trj01.py'),'--phase','native','--job',job,'--inputs',str(inputs),'--','python3','-I','-B',str(helper),'--chapter','02_turning_numbers','--job',job,'--inputs',str(inputs),'--permit',str(permit),'--threads','16'],check=True)
