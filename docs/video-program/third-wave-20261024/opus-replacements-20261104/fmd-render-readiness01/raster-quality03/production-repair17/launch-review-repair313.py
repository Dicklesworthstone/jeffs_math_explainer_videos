"""Root-only launcher: build313 (build268 plus boundary-guide-repair310) under the existing guard.

Adapted from launch-review-repair268.py. It needs root's own review-repair-recipe313.json
(cc0 does not write it), freezes every input the worker reads, and launches once.
"""
from pathlib import Path
import hashlib
import json
import subprocess

R = Path('/data/projects/jmev-stroke-probe-20261106')
W = R / 'work'
Q = R / 'qualification01/production-repair17'
JOB = 'csd-review-repair-build313'
WORKER = 'review-repair-build313.py'
WORKER_SHA256 = 'ff1ff0b8619cce99d87f2a5c5d44da1ac106a5cb70f698af2e93b64f3248b86a'
CANDIDATE = 'boundary-guide-repair310.rs'
CANDIDATE_SHA256 = 'bda7d931498b23bd3203cd0c6106de69dc0a8c5e160e738d698376322ff39e53'
SOURCE_COPY268_SHA256 = '81d28272a7a23b63199e79e6aa1086f01f131f0ddd685d4ae859d04b1de1934c'


def sha(path):
    with path.open('rb') as stream:
        return hashlib.file_digest(stream, 'sha256').hexdigest()


def success(path):
    receipt = json.loads(path.read_text())
    assert receipt.get('finished') and receipt.get('exit') == 0 and receipt.get('admitted'), path
    assert receipt['inputs_sha256'] == receipt['final_inputs_sha256'], path
    assert not any(receipt.get(k) for k in ('error', 'resource_stop', 'stopped_for_resource_limit')), path
    return receipt


def closure(manifest, receipt):
    """A finished job's own guard inputs: the manifest is the one its receipt bound, and every row
    in it still holds the bytes it bound. All are carried into the new manifest. There are no
    exceptions: a mutated row stops the launch and names the path."""
    assert sha(manifest) == receipt['inputs_sha256'], manifest
    rows = json.loads(manifest.read_text())['files']
    assert rows, manifest
    for row in rows:
        path = Path(row['path'])
        assert path.is_file() and sha(path) == row['sha256'], ('bound input changed', str(manifest), str(path))
    return {manifest} | {Path(row['path']) for row in rows}


def main():
    assert not (R / 'jobs' / JOB).exists(), 'Never duplicate or overwrite build313'
    assert not (W / 'review-repair313').exists() and not (W / 'bin/review-repair313').exists()
    manifest = Q / 'review-repair-build313-inputs.json'
    assert not manifest.exists(), 'Preserve any earlier attempt'
    worker = Q / WORKER
    assert sha(worker) == WORKER_SHA256
    recipe_path = Q / 'review-repair-recipe313.json'
    recipe = json.loads(recipe_path.read_text())
    assert recipe['reviewed_by_root'] is True and recipe['scope'] == 'chapter 09 zoom-line repair'
    assert recipe['files'] == {'boundary.rs': {'candidate': CANDIDATE, 'sha256': CANDIDATE_SHA256}}
    paths = {Path(__file__).resolve(), recipe_path, worker, R / 'guard_csd02.py', Q / CANDIDATE}
    assert sha(Q / CANDIDATE) == CANDIDATE_SHA256
    for key in ('source_review', 'helper_review'):
        review = Q / recipe[key]
        assert review.parent == Q and review.is_file() and sha(review) == recipe[key + '_sha256'], key
        paths.add(review)
    # the base: build268's job folder (source-copy, qualification, logs, receipt), rebind269's
    # receipt and summary, binary 268, and the 268 workspace as it is now; and the full input
    # closure of both jobs: each one's bound inputs manifest with every row re-verified (for
    # build268 that includes launch268's timeline21 rows and the production-repair19 workspace)
    build268 = R / 'jobs/csd-review-repair-build268'
    receipt = success(build268 / 'resource-receipt.json')
    assert sha(build268 / 'source-copy.json') == SOURCE_COPY268_SHA256
    paths |= closure(Q / 'review-repair-build268-inputs.json', receipt)
    paths.update(p for p in build268.rglob('*') if p.is_file())
    rebind269 = R / 'jobs/csd-review-repair-rebind269'
    receipt = success(rebind269 / 'resource-receipt.json')
    paths |= closure(Q / 'review-repair-rebind269-inputs.json', receipt)
    paths.update((rebind269 / 'resource-receipt.json', rebind269 / 'rebind-summary.json'))
    paths.add(W / 'bin/review-repair268')
    paths.update(p for p in (W / 'review-repair268/workspace').rglob('*') if p.is_file())
    # toolchain configuration the worker passes to rustfmt and cargo
    paths.update((W / 'tree/repo-view/rustfmt.toml', W / 'tree/engine-budget.toml'))
    paths.add(R / 'jobs/fmd-raster-quality-build05/qualification.json')
    with manifest.open('x') as stream:
        json.dump({'files': [{'path': str(p), 'sha256': sha(p)} for p in sorted(paths)]}, stream, indent=2)
    subprocess.run([
        '/usr/bin/python3', '-I', '-B', str(R / 'guard_csd02.py'),
        '--phase', 'native', '--job', JOB, '--inputs', str(manifest), '--',
        '/usr/bin/timeout', '22000', '/usr/bin/python3', '-I', '-B', str(worker),
    ], check=True)
    success(R / 'jobs' / JOB / 'resource-receipt.json')


if __name__ == '__main__':
    main()
