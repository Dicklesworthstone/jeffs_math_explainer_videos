"""Launch only the explicitly reviewed combined visual repair under the existing guard."""
from pathlib import Path
import hashlib
import json
import subprocess

R = Path('/data/projects/jmev-stroke-probe-20261106')
Q = R / 'qualification01/production-repair17'
JOB = 'csd-review-repair-build268'


def sha(path):
    with path.open('rb') as stream:
        return hashlib.file_digest(stream, 'sha256').hexdigest()


def success(path):
    receipt = json.loads(path.read_text())
    assert receipt.get('finished') and receipt.get('exit') == 0, path
    assert receipt['inputs_sha256'] == receipt['final_inputs_sha256'], path
    assert not any(receipt.get(k) for k in ('error', 'resource_stop', 'stopped_for_resource_limit')), path
    return receipt


assert not (R / 'jobs' / JOB).exists()
recipe_path = Q / 'review-repair-recipe268.json'
recipe = json.loads(recipe_path.read_text())
assert recipe['reviewed_by_root'] is True
assert recipe['scope'] == 'complex visual repairs'
assert set(recipe['files']) == {
    'boundary.rs', 'loop_sums.rs', 'one_derivative.rs', 'mod.rs',
    'rigidity.rs', 'gallery.rs', 'draw.rs', 'plate_fade_probe.rs',
}
paths = {Path(__file__).resolve(), recipe_path, Q / 'review-repair-build268.py', R / 'guard_csd02.py'}
for item in recipe['files'].values():
    candidate = Q / item['candidate']
    assert candidate.parent == Q and candidate.is_file()
    assert sha(candidate) == item['sha256'], candidate
    paths.add(candidate)
for name in ('csd-review-repair-build238', 'csd-review-repair-timeline242'):
    receipt_path = R / 'jobs' / name / 'resource-receipt.json'
    success(receipt_path)
    paths.add(receipt_path)
timeline_receipt = R / 'jobs/csd-production-timeline21/resource-receipt.json'
receipt = success(timeline_receipt)
manifest = Q / 'timeline21-inputs.json'
assert sha(manifest) == receipt['inputs_sha256']
paths.update((manifest, timeline_receipt))
for item in json.loads(manifest.read_text())['files']:
    path = Path(item['path'])
    assert sha(path) == item['sha256'], path
    paths.add(path)
paths.update(p for p in (R / 'work/production-repair19/workspace').rglob('*') if p.is_file())
manifest = Q / 'review-repair-build268-inputs.json'
with manifest.open('x') as stream:
    json.dump({'files': [{'path': str(p), 'sha256': sha(p)} for p in sorted(paths)]}, stream, indent=2)
subprocess.run([
    '/usr/bin/python3', '-I', '-B', str(R / 'guard_csd02.py'),
    '--phase', 'native', '--job', JOB, '--inputs', str(manifest), '--',
    '/usr/bin/timeout', '22000', '/usr/bin/python3', '-I', '-B',
    str(Q / 'review-repair-build268.py'),
], check=True)
