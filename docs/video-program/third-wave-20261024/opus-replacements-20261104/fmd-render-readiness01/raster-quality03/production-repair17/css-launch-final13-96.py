from pathlib import Path
import json,subprocess,importlib.util,time
R=Path('/data/projects/jmev-css-render-worker-20261107');Q=R/'qualification01';D=Path('/data/projects/jmev-stroke-probe-20261106')
helper=Q/'css-final-chapter78.py'
permit=Q/'review13-96/final-render-permit96.json'
spec=importlib.util.spec_from_file_location('worker',helper);m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m)
prerequisite=R/'jobs/css-production19-final-11-counting78/resource-receipt.json'
deadline=time.monotonic()+43200
while True:
 if prerequisite.exists() and m.read(prerequisite).get('finished'):
  m.success(prerequisite);break
 assert time.monotonic()<deadline,'predecessor wait timed out'
 time.sleep(15)
review,_=m.check('13_real_integral',permit)
files={helper,permit,Path(__file__).resolve(),prerequisite,R/'guard_css74.py',R/'jobs/css-production19-delivery76/resource-receipt.json',R/'jobs/css-runtime-transfer75/resource-receipt.json',R/'jobs/css-encoder-stop79/resource-receipt.json',D/'jobs/csd-production-timeline21/resource-receipt.json',D/'jobs/csd-production-score21/resource-receipt.json',D/'jobs/csd-production-repair-build19/qualification.json',D/'jobs/csd-production-score21/score21-summary.json'}
inherited=Q/'delivery76-inputs.json';files.add(inherited)
files.update(Path(x['path']) for x in m.read(inherited)['files'])
files.update(p for p in m.NARRATION.rglob('*') if p.is_file())
files.update(D/'jobs/csd-production-score21/score'/p for p in m.read(D/'jobs/csd-production-score21/score21-summary.json')['files'])
files.update(Path(x['path']) for x in review['evidence']);files.add(m.BIN)
for suffix in ['timeline.tsv','cues.tsv','cuelog.tsv']:files.add(m.TIMELINES/('13_real_integral.'+suffix))
inputs=Q/'final13-96-inputs.json'
with inputs.open('x') as f:json.dump({'files':[dict(path=str(p),sha256=m.sha(p)) for p in sorted(files)]},f,indent=2)
job='css-production19-final-13-real-integral96';assert not (R/'jobs'/job).exists()
subprocess.run(['python3','-I','-B',str(R/'guard_css74.py'),'--phase','native','--job',job,'--inputs',str(inputs),'--','python3','-I','-B',str(helper),'--chapter','13_real_integral','--job',job,'--inputs',str(inputs),'--permit',str(permit),'--threads','16'],check=True)
