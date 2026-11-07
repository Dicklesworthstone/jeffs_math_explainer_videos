"""Freeze reviewed production17 source inputs and enter the existing CSD guard."""
from pathlib import Path
import hashlib
import json
import subprocess

ROOT = Path('/data/projects/jmev-stroke-probe-20261106')
Q = ROOT / 'qualification01/production-repair17'
JOB = 'csd-production-repair-build17'


def sha(path):
    with path.open('rb') as stream:
        return hashlib.file_digest(stream, 'sha256').hexdigest()


assert not (ROOT / 'jobs' / JOB).exists(), 'Never duplicate an admitted job'
prior = json.loads((ROOT / 'jobs/csd-continuous-quality-build16/resource-receipt.json').read_text())
assert prior['exit'] == 0 and prior['inputs_sha256'] == prior['final_inputs_sha256']
assert not prior.get('resource_stop') and not prior.get('stopped_for_resource_limit')
paths = {Path(__file__), Q / 'build17.py', Q / 'candidate-receipt01.json', ROOT / 'guard_csd02.py'}
for row in json.loads((ROOT / 'qualification01/continuous-quality16/inputs16.json').read_text())['files']:
    path = Path(row['path'])
    assert sha(path) == row['sha256'], str(path)
    paths.add(path)
source = ROOT / 'work/continuous-quality16/workspace'
paths.update(p for p in source.rglob('*') if p.is_file())
for row in json.loads((Q / 'candidate-receipt01.json').read_text())['files']:
    path = Q / row['file']
    assert sha(path) == row['sha256']
    assert sha(source / 'explainer_corpus/complex_analysis_visual' / row['relative']) == row['before_sha256']
    paths.add(path)
manifest = Q / 'build17-inputs.json'
with manifest.open('x') as stream:
    json.dump({'files': [{'path': str(p), 'sha256': sha(p)} for p in sorted(paths)]}, stream, indent=2)
subprocess.run([
    '/usr/bin/python3', '-I', '-B', str(ROOT / 'guard_csd02.py'),
    '--phase', 'native', '--job', JOB, '--inputs', str(manifest), '--',
    '/usr/bin/timeout', '22000', '/usr/bin/python3', '-I', '-B', str(Q / 'build17.py'),
], check=True)
