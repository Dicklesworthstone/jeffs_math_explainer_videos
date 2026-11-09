"""Root-only launcher for the targeted 09 preview313. No waiting, no automatic review approval.

Adapted from launch-reviewed-preview296.py by cc0 (not run by cc0): 313 paths, the 09 tables
only, and the preview274 base movie frozen as the zoom-probe control.
"""
from pathlib import Path
import argparse
import hashlib
import json
import subprocess

R = Path('/data/projects/jmev-stroke-probe-20261106')
Q = R / 'qualification01/production-repair17'
J = R / 'jobs/csd-review-repair-preview313'
N = R / 'jobs/csd-review-repair-rebind313/narration'
N269 = R / 'jobs/csd-review-repair-rebind269/narration'
CONTROL = R / 'jobs/csd-review-repair-preview274/preview/09_boundary.mp4'
CONTROL_SHA256 = 'dba964d2b0730b0e327eb32b9b9bb81496ddb4ec3422f043d356a390f1de3fb4'
CHAPTER = '09_boundary'

def sha(p):
    with p.open('rb') as f:
        return hashlib.file_digest(f, 'sha256').hexdigest()

def success(job):
    p = R / 'jobs' / job / 'resource-receipt.json'
    r = json.loads(p.read_text())
    assert r.get('finished') and r.get('exit') == 0 and r.get('admitted')
    assert r['inputs_sha256'] == r['final_inputs_sha256']
    assert not any(r.get(k) for k in ('error', 'resource_stop', 'stopped_for_resource_limit'))
    return p, r

def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('--worker-sha256', required=True)
    ap.add_argument('--review', type=Path, required=True)
    ap.add_argument('--review-sha256', required=True)
    a = ap.parse_args()
    worker = Q / 'review-repair-preview313.py'
    review = a.review.resolve(strict=True)
    assert sha(worker) == a.worker_sha256
    assert sha(review) == a.review_sha256
    assert not J.exists(), 'Never duplicate or overwrite preview313'
    manifest = Q / 'review-repair-preview313-inputs.json'
    assert not manifest.exists(), 'Preserve any earlier attempt'
    assert sha(CONTROL) == CONTROL_SHA256
    files = {Path(__file__).resolve(), worker, review, R / 'guard_csd02.py', R / 'work/bin/review-repair313', CONTROL}
    for job, name in [('csd-review-repair-timeline313', 'review-repair-timeline313-inputs.json')]:
        receipt, state = success(job)
        inputs = Q / name
        assert sha(inputs) == state['inputs_sha256']
        files.update((receipt, inputs))
        for row in json.loads(inputs.read_text())['files']:
            p = Path(row['path'])
            assert sha(p) == row['sha256'], str(p)
            files.add(p)
        files.update(p for p in (R / 'jobs' / job).rglob('*') if p.is_file())
    for job in ('csd-review-repair-build313', 'csd-review-repair-rebind313', 'csd-review-repair-preview274'):
        receipt, _ = success(job)
        files.add(receipt)
    # the control movie's own input closure: preview274's bound manifest, every row re-verified
    _, state = success('csd-review-repair-preview274')
    inputs = Q / 'review-repair-preview274-inputs.json'
    assert sha(inputs) == state['inputs_sha256']
    files.add(inputs)
    for row in json.loads(inputs.read_text())['files']:
        p = Path(row['path'])
        assert sha(p) == row['sha256'], ('bound input changed', str(inputs), str(p))
        files.add(p)
    for job in ('csd-review-repair-build313', 'csd-review-repair-rebind313'):
        files.update(p for p in (R / 'jobs' / job).rglob('*') if p.is_file())
    t = R / 'jobs/csd-review-repair-timeline313'
    comparison = json.loads((t / 'comparison.json').read_text())
    chapters = json.loads((R / 'jobs/csd-review-repair-build313/qualification.json').read_text())['chapters']
    assert CHAPTER in chapters and comparison['frames'] == {CHAPTER: 16504}
    assert comparison['named_path_mapping'] == {str(N269): str(N)}
    rows = {(r['chapter'], r['kind']): r for r in comparison['tables']}
    assert len(rows) == len(comparison['tables']) == 3
    for kind in ('timeline', 'cues', 'cuelog'):
        row = rows[(CHAPTER, kind)]
        new = t / 'timeline-output' / (CHAPTER + '.' + kind + '.tsv')
        old = R / 'jobs/csd-review-repair-timeline269/timeline-output' / new.name
        assert sha(new) == row['new_sha256'] and sha(old) == row['timeline269_sha256']
        assert row['equal_to_timeline269_after_named_narration_path_mapping']
        assert old.read_bytes().replace(str(N269).encode(), str(N).encode()) == new.read_bytes()
        files.update((old, new))
    files.update(p for p in N.rglob('*') if p.is_file())
    with manifest.open('x') as f:
        json.dump({'files': [dict(path=str(p), sha256=sha(p)) for p in sorted(files)]}, f, indent=2)
    subprocess.run(['/usr/bin/python3', '-I', '-B', str(R / 'guard_csd02.py'), '--phase', 'native',
                    '--job', J.name, '--inputs', str(manifest), '--', '/usr/bin/timeout', '172900',
                    '/usr/bin/python3', '-I', '-B', str(worker)], check=True)
    success(J.name)

if __name__ == '__main__':
    main()
