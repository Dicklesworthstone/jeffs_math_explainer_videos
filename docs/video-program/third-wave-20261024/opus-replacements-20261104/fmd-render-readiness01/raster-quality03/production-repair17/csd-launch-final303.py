"""Launch a combined-repair chapter only with an explicit, actually reviewed permit (version 303).
The worker and permit are hashed against the values root passes before the worker is loaded,
and both paths are resolved before admission and passed on resolved. Against 287 (preserved):
--worker-sha256 and --permit-sha256 (as in launch-reviewed-preview296.py), the 303 worker,
job and inputs names, and refusal before the inputs file is written when fewer than 16 CPUs
are available or the job exists."""
from pathlib import Path
import argparse
import hashlib
import importlib.util
import json
import os
import subprocess

R = Path('/data/projects/jmev-stroke-probe-20261106')
Q = R / 'qualification01/production-repair17'
THREADS = 16

def digest(path):
    with Path(path).open('rb') as stream:
        return hashlib.file_digest(stream, 'sha256').hexdigest()

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--chapter', choices=['01_the_bump', '04_reading_color', '05_gallery', '06_one_derivative', '07_one_point', '08_loop_sums', '09_boundary', '10_rigidity', '11_counting', '12_residues', '14_continuation'], required=True)
    parser.add_argument('--permit', type=Path, required=True)
    parser.add_argument('--permit-sha256', required=True)
    parser.add_argument('--worker-sha256', required=True)
    args = parser.parse_args()
    helper = (Q / 'csd-reviewed-final303.py').resolve(strict=True)
    # CC3 review 297 F2: refuse any worker but the one root names, before any of its code runs.
    assert digest(helper) == args.worker_sha256, 'worker is not at the hash root passed: ' + str(helper)
    permit = args.permit.resolve(strict=True)
    assert digest(permit) == args.permit_sha256, 'permit is not at the hash root passed: ' + str(permit)
    spec = importlib.util.spec_from_file_location('worker', helper)
    worker = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(worker)
    assert worker.VERSION == '303' and args.chapter in worker.CHAPTERS
    # check() refuses a 244/278/287 job for this chapter, a permit without host csd and an unbound preview.
    review, _, binding = worker.check(args.chapter, permit)
    assert review['frames'] > 0
    job = worker.job_name(args.chapter, worker.VERSION)
    assert not (R / 'jobs' / job).exists(), 'Existing job must not be duplicated'
    # The worker pins itself to the first THREADS CPUs it inherits; refuse here, before any file is written.
    assert len(os.sched_getaffinity(0)) >= THREADS, 'fewer than 16 CPUs in the inherited affinity'
    files = {Path(__file__).resolve(), helper, permit,
             R / 'guard_csd02.py', worker.BIN}
    preview = worker.PREVIEWS[args.chapter]
    for name, manifest in (
        ('csd-review-repair-timeline269', 'review-repair-timeline269-inputs.json'),
        (preview['job'], preview['manifest']),
    ):
        receipt = R / 'jobs' / name / 'resource-receipt.json'
        state = worker.success(receipt)
        source = Q / manifest
        assert worker.sha(source) == state['inputs_sha256']
        files.update((receipt, source))
        for row in worker.read(source)['files']:
            path = Path(row['path'])
            assert worker.sha(path) == row['sha256'], str(path)
            files.add(path)
        files.update(p for p in (R / 'jobs' / name).rglob('*') if p.is_file())
    files.update(Path(row['path']) for row in review['evidence']+review['source_binding_evidence'])
    files.update(binding)
    files.update(p for p in worker.NARRATION.rglob('*') if p.is_file())
    score = R / 'jobs/csd-production-score21'
    files.update((score / 'score21-summary.json', score / 'resource-receipt.json',
                  R / 'jobs/csd-raster-encoder-stop13/resource-receipt.json',
                  R / 'jobs/csd-review-repair-build268/qualification.json'))
    files.update(score / 'score' / name for name in worker.read(score / 'score21-summary.json')['files'])
    inputs = Q / ('final-'+args.chapter+'-inputs'+worker.VERSION+'.json')
    with inputs.open('x') as stream:
        json.dump({'files': [dict(path=str(p), sha256=worker.sha(p))
                             for p in sorted(files)]}, stream, indent=2)
    subprocess.run([
        '/usr/bin/python3', '-I', '-B', str(R / 'guard_csd02.py'),
        '--phase', 'native', '--job', job, '--inputs', str(inputs), '--',
        '/usr/bin/python3', '-I', '-B', str(helper), '--chapter', args.chapter,
        '--job', job, '--inputs', str(inputs), '--permit', str(permit),
        '--threads', str(THREADS),
    ], check=True)

if __name__ == '__main__':
    main()
