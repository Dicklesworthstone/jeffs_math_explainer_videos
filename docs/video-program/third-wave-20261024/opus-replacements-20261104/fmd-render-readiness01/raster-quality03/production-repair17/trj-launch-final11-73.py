from pathlib import Path
import json, subprocess, importlib.util, time
R=Path('/data/projects/jmev-render-worker-20261107'); Q=R/'qualification01/production19'; D=Path('/data/projects/jmev-stroke-probe-20261106')
helper=Q/'trj-final-chapter19-02.py'; permit=Q/'review11-73/final-render-permit73.json'
spec=importlib.util.spec_from_file_location('worker',helper); m=importlib.util.module_from_spec(spec); spec.loader.exec_module(m)
prior=R/'jobs/trj-production19-final-10-rigidity36/resource-receipt.json'
for _ in range(1440):
 if prior.exists() and m.read(prior).get('finished'):
  m.success(prior); break
 time.sleep(60)
else: raise RuntimeError('Predecessor deadline; no chapter11 render launched')
review,_=m.check('11_counting',permit)
files={prior,helper,permit,Path(__file__).resolve(),R/'guard_trj01.py',R/'jobs/trj-production19-delivery02/resource-receipt.json',D/'jobs/csd-production-timeline21/resource-receipt.json',D/'jobs/csd-production-score21/resource-receipt.json',D/'jobs/csd-production-repair-build19/qualification.json',D/'jobs/csd-production-score21/score21-summary.json'}
inherited=Q/'delivery02-inputs.json'; files.add(inherited)
files.update(Path(x['path']) for x in m.read(inherited)['files'])
files.update(p for p in m.NARRATION.rglob('*') if p.is_file())
files.update(D/'jobs/csd-production-score21/score'/p for p in m.read(D/'jobs/csd-production-score21/score21-summary.json')['files'])
files.update(Path(x['path']) for x in review['evidence']); files.add(m.BIN)
for suffix in ['timeline.tsv','cues.tsv','cuelog.tsv']: files.add(m.TIMELINES/('11_counting.'+suffix))
inputs=Q/'final11-73-inputs.json'
with inputs.open('x') as f: json.dump({'files':[dict(path=str(p),sha256=m.sha(p)) for p in sorted(files)]},f,indent=2)
job='trj-production19-final-11-counting73'; assert not (R/'jobs'/job).exists()
subprocess.run(['python3','-I','-B',str(R/'guard_trj01.py'),'--phase','native','--job',job,'--inputs',str(inputs),'--','python3','-I','-B',str(helper),'--chapter','11_counting','--job',job,'--inputs',str(inputs),'--permit',str(permit),'--threads','16'],check=True)
