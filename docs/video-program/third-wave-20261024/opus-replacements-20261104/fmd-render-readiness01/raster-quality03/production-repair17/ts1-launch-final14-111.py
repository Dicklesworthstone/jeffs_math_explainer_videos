"""Queue reviewed continuation behind TS1 gallery, retaining resource guards."""
from pathlib import Path
import importlib.util
import json
import subprocess
import time

R = Path('/data/projects/jmev-ts1-render-worker-20261107')
Q = R / 'qualification01'
D = Path('/data/projects/jmev-stroke-probe-20261106')
helper = Q / 'ts1-final-chapter103.py'
permit = Q / 'review14-111/final-render-permit111.json'
spec = importlib.util.spec_from_file_location('worker', helper)
m = importlib.util.module_from_spec(spec)
spec.loader.exec_module(m)
prior = R / 'jobs/ts1-production19-final-05-gallery103/resource-receipt.json'
deadline = time.monotonic() + 43200
while not (prior.exists() and m.read(prior).get('finished')):
    assert time.monotonic() < deadline, 'predecessor wait timed out'
    time.sleep(15)
m.success(prior)
review, _ = m.check('14_continuation', permit)
files = {helper, permit, Path(__file__).resolve(), prior, R/'guard_ts1100.py'}
for path in ['jobs/ts1-production19-delivery101/resource-receipt.json',
             'jobs/ts1-runtime-transfer100/resource-receipt.json',
             'jobs/ts1-encoder-stop102/resource-receipt.json']:
    files.add(R/path)
for path in ['jobs/csd-production-timeline21/resource-receipt.json',
             'jobs/csd-production-score21/resource-receipt.json',
             'jobs/csd-production-repair-build19/qualification.json',
             'jobs/csd-production-score21/score21-summary.json']:
    files.add(D/path)
inherited = Q/'delivery101-inputs.json'
files.add(inherited)
files.update(Path(x['path']) for x in m.read(inherited)['files'])
files.update(p for p in m.NARRATION.rglob('*') if p.is_file())
files.update(D/'jobs/csd-production-score21/score'/p for p in m.read(D/'jobs/csd-production-score21/score21-summary.json')['files'])
files.update(Path(x['path']) for x in review['evidence'])
files.add(m.BIN)
for suffix in ['timeline.tsv', 'cues.tsv', 'cuelog.tsv']:
    files.add(m.TIMELINES/('14_continuation.'+suffix))
inputs = Q/'final14-111-inputs.json'
with inputs.open('x') as f:
    json.dump({'files':[dict(path=str(p),sha256=m.sha(p)) for p in sorted(files)]},f,indent=2)
job = 'ts1-production19-final-14-continuation111'
assert not (R/'jobs'/job).exists()
subprocess.run(['python3','-I','-B',str(R/'guard_ts1100.py'),'--phase','native','--job',job,'--inputs',str(inputs),'--','python3','-I','-B',str(helper),'--chapter','14_continuation','--job',job,'--inputs',str(inputs),'--permit',str(permit),'--threads','16'],check=True)
