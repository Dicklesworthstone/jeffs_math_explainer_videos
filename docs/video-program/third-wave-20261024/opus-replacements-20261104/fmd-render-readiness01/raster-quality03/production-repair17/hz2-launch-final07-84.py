from pathlib import Path
import json,importlib.util,subprocess,time
R=Path('/data/projects/jmev-hz2-render-worker-20261107');Q=R/'qualification01/production19';D=Path('/data/projects/jmev-stroke-probe-20261106');helper=Q/'hz2-final-chapter53.py';permit=Q/'review07-84/final-render-permit84.json';JOB='hz2-production19-final-07-one-point84'
spec=importlib.util.spec_from_file_location('worker',helper);m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m)
# Wait for the same host's existing final15; no concurrent native jobs.
for _ in range(2160):
 prior=R/'jobs/hz2-production19-final-12-residues59/resource-receipt.json'
 if prior.exists() and m.read(prior).get('finished'):break
 time.sleep(20)
else:raise RuntimeError('Prior final15 deadline; no chapter12 render launched')
m.success(prior)
review,_=m.check('07_one_point',permit)
assert not (R/'jobs'/JOB).exists()
# Written only after the old waiting controller is confirmed stopped and
# its old final job never admitted, so this chapter has one producer.
ownership=Q/'chapter07-handoff84.json';h=m.read(ownership);assert h['prior_controller_stopped'] is True and h['prior_job_absent'] is True and h['chapter']=='07_one_point'
files={prior,helper,permit,Path(__file__).resolve(),ownership,R/'guard_hz201.py',R/'jobs/hz2-production19-delivery49/resource-receipt.json',R/'jobs/hz2-runtime-transfer47/resource-receipt.json',R/'jobs/hz2-encoder-stop03/resource-receipt.json',D/'jobs/csd-production-timeline21/resource-receipt.json',D/'jobs/csd-production-score21/resource-receipt.json',D/'jobs/csd-production-repair-build19/qualification.json',D/'jobs/csd-production-score21/score21-summary.json'}
inherited=Q/'delivery49-inputs.json';files.add(inherited);files.update(Path(x['path']) for x in m.read(inherited)['files']);files.update(p for p in m.NARRATION.rglob('*') if p.is_file());files.update(D/'jobs/csd-production-score21/score'/p for p in m.read(D/'jobs/csd-production-score21/score21-summary.json')['files']);files.update(Path(x['path']) for x in review['evidence']);files.add(m.BIN)
for suffix in ['timeline.tsv','cues.tsv','cuelog.tsv']:files.add(m.TIMELINES/('12_residues.'+suffix))
inputs=Q/'final07-84-inputs.json'
with inputs.open('x') as f:json.dump({'files':[dict(path=str(p),sha256=m.sha(p)) for p in sorted(files)]},f,indent=2)
subprocess.run(['/usr/bin/python3','-I','-B',str(R/'guard_hz201.py'),'--phase','native','--job',JOB,'--inputs',str(inputs),'--','/usr/bin/python3','-I','-B',str(helper),'--chapter','07_one_point','--job',JOB,'--inputs',str(inputs),'--permit',str(permit),'--threads','12'],check=True)
