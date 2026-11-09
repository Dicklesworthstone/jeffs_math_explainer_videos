"""Launch the previously reviewed sixth chapter only after TS2 qualification."""
from pathlib import Path
import importlib.util
import json
import subprocess

R = Path('/data/projects/jmev-ts2-render-worker-20261107')
D = Path('/data/projects/jmev-stroke-probe-20261106')
T = Path('/data/projects/jmev-render-worker-20261107')
Q = R / 'qualification01'
helper = Q / 'ts2-final-chapter121.py'
permit = T / 'qualification01/production19/review06-34/final-render-permit34.json'
spec = importlib.util.spec_from_file_location('chapter_worker', helper)
m = importlib.util.module_from_spec(spec)
spec.loader.exec_module(m)
review, _ = m.check('06_one_derivative', permit)
assert review['frames'] == 10030
handoff = Q / 'chapter06-handoff123.json'
assert m.read(handoff)['old_controller_stopped_before_job'] is True
files = {helper, permit, handoff, Path(__file__).resolve(), R/'guard_ts2118.py', m.BIN}
for job in ('ts2-runtime-transfer118', 'ts2-production19-delivery119', 'ts2-encoder-stop122'):
    receipt = R/'jobs'/job/'resource-receipt.json'
    m.success(receipt)
    files.add(receipt)
for path in (D/'jobs/csd-production-timeline21/resource-receipt.json',
             D/'jobs/csd-production-score21/resource-receipt.json',
             D/'jobs/csd-production-repair-build19/qualification.json',
             D/'jobs/csd-production-score21/score21-summary.json'):
    files.add(path)
inherited = Q/'delivery119-inputs.json'
files.add(inherited)
files.update(Path(x['path']) for x in m.read(inherited)['files'])
files.update(p for p in m.NARRATION.rglob('*') if p.is_file())
files.update(D/'jobs/csd-production-score21/score'/p for p in m.read(D/'jobs/csd-production-score21/score21-summary.json')['files'])
files.update(Path(x['path']) for x in review['evidence'])
for suffix in ('timeline.tsv', 'cues.tsv', 'cuelog.tsv'):
    files.add(m.TIMELINES/('06_one_derivative.'+suffix))
job = 'ts2-production19-final-06-one-derivative123'
assert not (R/'jobs'/job).exists()
inputs = Q/'final06-123-inputs.json'
with inputs.open('x') as stream:
    json.dump({'files': [{'path': str(p), 'sha256': m.sha(p)} for p in sorted(files)]}, stream, indent=2)
subprocess.run(['python3', '-I', '-B', str(R/'guard_ts2118.py'), '--phase', 'native',
                '--job', job, '--inputs', str(inputs), '--', 'python3', '-I', '-B',
                str(helper), '--chapter', '06_one_derivative', '--job', job,
                '--inputs', str(inputs), '--permit', str(permit), '--threads', '16'], check=True)
