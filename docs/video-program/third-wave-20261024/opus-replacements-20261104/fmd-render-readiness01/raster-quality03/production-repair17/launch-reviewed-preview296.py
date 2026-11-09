"""Root-only launcher for reviewed preview274. No waiting, no automatic review approval."""
from pathlib import Path
import argparse
import hashlib
import json
import subprocess

R = Path('/data/projects/jmev-stroke-probe-20261106')
Q = R / 'qualification01/production-repair17'
J = R / 'jobs/csd-review-repair-preview274'
N = R / 'jobs/csd-review-repair-rebind269/narration'
OLD = R / 'jobs/csd-production-assembly19/narration'

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
    worker = Q / 'cc3-preview-fade274.py'
    review = a.review.resolve(strict=True)
    assert sha(worker) == a.worker_sha256
    assert sha(review) == a.review_sha256
    assert not J.exists(), 'Never duplicate or overwrite preview274'
    manifest = Q / 'review-repair-preview274-inputs.json'
    assert not manifest.exists(), 'Preserve any earlier attempt'
    files = {Path(__file__).resolve(), worker, review, R / 'guard_csd02.py', R / 'work/bin/review-repair268'}
    for job, name in [('csd-review-repair-timeline269', 'review-repair-timeline269-inputs.json')]:
        receipt, state = success(job)
        inputs = Q / name
        assert sha(inputs) == state['inputs_sha256']
        files.update((receipt, inputs))
        for row in json.loads(inputs.read_text())['files']:
            p = Path(row['path'])
            assert sha(p) == row['sha256'], str(p)
            files.add(p)
        files.update(p for p in (R / 'jobs' / job).rglob('*') if p.is_file())
    for job in ('csd-review-repair-build268', 'csd-review-repair-rebind269'):
        receipt, _ = success(job)
        files.add(receipt)
        files.update(p for p in (R / 'jobs' / job).rglob('*') if p.is_file())
    t = R / 'jobs/csd-review-repair-timeline269'
    comparison = json.loads((t / 'comparison.json').read_text())
    chapters = json.loads((R / 'jobs/csd-review-repair-build268/qualification.json').read_text())['chapters']
    assert len(chapters) == len(set(chapters)) == 15
    assert set(comparison['frames']) == set(chapters) and sum(comparison['frames'].values()) == 133662
    assert len(comparison['tables']) == 45
    rows = {(r['chapter'], r['kind']): r for r in comparison['tables']}
    assert len(rows) == 45
    for chapter in chapters:
        for kind in ('timeline', 'cues', 'cuelog'):
            row = rows[(chapter, kind)]
            new = t / 'timeline-output' / (chapter + '.' + kind + '.tsv')
            old = R / 'jobs/csd-production-timeline21/timeline-output' / new.name
            assert sha(new) == row['new_sha256'] and sha(old) == row['old_sha256']
            assert row['equal_after_named_narration_path_mapping']
            assert old.read_bytes().replace(str(OLD).encode(), str(N).encode()) == new.read_bytes()
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
