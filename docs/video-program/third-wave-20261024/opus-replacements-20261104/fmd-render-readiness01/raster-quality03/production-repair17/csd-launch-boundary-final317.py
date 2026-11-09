"""Launch the 09_boundary build313 final only with an explicit, actually reviewed permit (version 317).
Prepared only; not run. The worker and permit are hashed against the values root passes before the
worker is loaded, and both paths are resolved before admission and passed on resolved. Against
csd-launch-final303.py (preserved): chapter 09_boundary only, the 317 worker, job and inputs
names, and the input closure of the four completed 313 jobs (build313, rebind313, timeline313,
preview313: receipt, own guard inputs, every row re-verified, job folder) in place of preview274.
Nothing about binary 313 is written here in advance: its hash is read from the completed build313
qualification.json, checked against the binary and the permit, frozen as a row of the new inputs
and printed. Guard, single native lane, 16 CPUs and the worker command are 303's.
"""
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
    parser.add_argument('--chapter', choices=['09_boundary'], required=True)
    parser.add_argument('--permit', type=Path, required=True)
    parser.add_argument('--permit-sha256', required=True)
    parser.add_argument('--worker-sha256', required=True)
    args = parser.parse_args()
    helper = (Q / 'csd-reviewed-boundary-final317.py').resolve(strict=True)
    # CC3 review 297 F2: refuse any worker but the one root names, before any of its code runs.
    assert digest(helper) == args.worker_sha256, 'worker is not at the hash root passed: ' + str(helper)
    permit = args.permit.resolve(strict=True)
    assert digest(permit) == args.permit_sha256, 'permit is not at the hash root passed: ' + str(permit)
    spec = importlib.util.spec_from_file_location('worker', helper)
    worker = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(worker)
    assert worker.VERSION == '317' and worker.CHAPTERS == (args.chapter,)
    # check() refuses a 244/278/287/303 job for 09, an unfinished or changed 313 job, a binary that is
    # not build313's, schedules unequal to timeline269/21, a WAV unequal to rebind269/assembly19, and
    # a permit without host csd, actual picture and motion review, independent review or preview313 records.
    review, _, binding = worker.check(args.chapter, permit)
    assert review['frames'] == worker.FRAMES
    job = worker.job_name(args.chapter, worker.VERSION)
    assert not (R / 'jobs' / job).exists(), 'Existing job must not be duplicated'
    # The worker pins itself to the first THREADS CPUs it inherits; refuse here, before any file is written.
    assert len(os.sched_getaffinity(0)) >= THREADS, 'fewer than 16 CPUs in the inherited affinity'
    files = {Path(__file__).resolve(), helper, permit,
             R / 'guard_csd02.py', worker.BIN}
    pins = {}
    for name in worker.MANIFESTS:
        receipt, manifest, rows = worker.completed(name)
        files.update((R / 'jobs' / name / 'resource-receipt.json', manifest))
        files.update(Path(path) for path in rows)
        files.update(p for p in (R / 'jobs' / name).rglob('*') if p.is_file())
        pins[name] = receipt['inputs_sha256']
    for name, manifest in (('csd-review-repair-timeline269', 'review-repair-timeline269-inputs.json'),):
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
    files.update(worker.independent(review))
    files.update(binding)
    files.update(p for p in worker.NARRATION.rglob('*') if p.is_file())
    files.update(worker.narration313())
    files.update(worker.schedule313())
    files.update(worker.WORKSPACE313 / r for r in worker.CONTRACT + (worker.BOUNDARY, worker.UNITS, worker.MAP313))
    files.add(Q / worker.REPAIR310)
    score = R / 'jobs/csd-production-score21'
    files.update((score / 'score21-summary.json', score / 'resource-receipt.json',
                  R / 'jobs/csd-raster-encoder-stop13/resource-receipt.json',
                  R / 'jobs/csd-production-timeline21/resource-receipt.json',
                  R / 'jobs/csd-review-repair-build268/resource-receipt.json',
                  R / 'jobs/csd-review-repair-rebind269/resource-receipt.json',
                  R / 'jobs/csd-review-repair-build268/qualification.json'))
    files.update(score / 'score' / name for name in worker.read(score / 'score21-summary.json')['files'])
    binary = worker.read(worker.BUILD313 / 'qualification.json')['binary_sha256']
    assert worker.sha(worker.BIN) == binary == review['binary_sha256']
    inputs = Q / ('final-'+args.chapter+'-inputs'+worker.VERSION+'.json')
    with inputs.open('x') as stream:
        json.dump({'files': [dict(path=str(p), sha256=worker.sha(p))
                             for p in sorted(files)]}, stream, indent=2)
    print(json.dumps(dict(job=job, inputs=str(inputs), inputs_sha256=worker.sha(inputs), binary_sha256=binary,
                          completed_inputs_sha256=pins), indent=2), flush=True)
    subprocess.run([
        '/usr/bin/python3', '-I', '-B', str(R / 'guard_csd02.py'),
        '--phase', 'native', '--job', job, '--inputs', str(inputs), '--',
        '/usr/bin/python3', '-I', '-B', str(helper), '--chapter', args.chapter,
        '--job', job, '--inputs', str(inputs), '--permit', str(permit),
        '--threads', str(THREADS),
    ], check=True)

if __name__ == '__main__':
    main()
