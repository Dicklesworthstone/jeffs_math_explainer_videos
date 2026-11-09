from pathlib import Path
import json,subprocess,importlib.util
R=Path('/data/projects/jmev-css-render-worker-20261107');Q=R/'qualification01';D=Path('/data/projects/jmev-stroke-probe-20261106')
helper=Q/'css-final-chapter78.py'
permit=Path('/data/projects/jmev-render-worker-20261107/qualification01/production19/review11-73/final-render-permit73.json')
spec=importlib.util.spec_from_file_location('worker',helper);m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m)
h=m.read(Q/'chapter11-handoff78.json');assert h['prior_controller_stopped'] and h['prior_job_absent'] and h['chapter']=='11_counting'
review,_=m.check('11_counting',permit)
files={helper,permit,Path(__file__).resolve(),Q/'chapter11-handoff78.json',R/'guard_css74.py',R/'jobs/css-production19-delivery76/resource-receipt.json',R/'jobs/css-runtime-transfer75/resource-receipt.json',R/'jobs/css-encoder-stop79/resource-receipt.json',D/'jobs/csd-production-timeline21/resource-receipt.json',D/'jobs/csd-production-score21/resource-receipt.json',D/'jobs/csd-production-repair-build19/qualification.json',D/'jobs/csd-production-score21/score21-summary.json'}
inherited=Q/'delivery76-inputs.json';files.add(inherited)
files.update(Path(x['path']) for x in m.read(inherited)['files'])
files.update(p for p in m.NARRATION.rglob('*') if p.is_file())
files.update(D/'jobs/csd-production-score21/score'/p for p in m.read(D/'jobs/csd-production-score21/score21-summary.json')['files'])
files.update(Path(x['path']) for x in review['evidence']);files.add(m.BIN)
for suffix in ['timeline.tsv','cues.tsv','cuelog.tsv']:files.add(m.TIMELINES/('11_counting.'+suffix))
inputs=Q/'final11-78-inputs.json'
with inputs.open('x') as f:json.dump({'files':[dict(path=str(p),sha256=m.sha(p)) for p in sorted(files)]},f,indent=2)
job='css-production19-final-11-counting78';assert not (R/'jobs'/job).exists()
subprocess.run(['python3','-I','-B',str(R/'guard_css74.py'),'--phase','native','--job',job,'--inputs',str(inputs),'--','python3','-I','-B',str(helper),'--chapter','11_counting','--job',job,'--inputs',str(inputs),'--permit',str(permit),'--threads','16'],check=True)
