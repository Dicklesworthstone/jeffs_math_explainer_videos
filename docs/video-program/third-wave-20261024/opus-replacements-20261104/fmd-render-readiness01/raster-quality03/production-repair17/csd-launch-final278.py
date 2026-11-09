"""Launch a combined-repair chapter only with an explicit, actually reviewed permit."""
from pathlib import Path
import argparse
import importlib.util
import json
import subprocess

R = Path('/data/projects/jmev-stroke-probe-20261106')
Q = R / 'qualification01/production-repair17'

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--chapter', choices=['01_the_bump', '04_reading_color', '05_gallery', '06_one_derivative', '07_one_point', '08_loop_sums', '09_boundary', '10_rigidity', '11_counting', '12_residues', '14_continuation'], required=True)
    parser.add_argument('--permit', type=Path, required=True)
    args = parser.parse_args()
    helper = Q / 'csd-reviewed-final278.py'
    spec = importlib.util.spec_from_file_location('worker', helper)
    worker = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(worker)
    review, _ = worker.check(args.chapter, args.permit)
    assert review['frames'] > 0
    job = 'csd-reviewed-final-'+args.chapter.replace('_','-')+'278'
    assert not (R / 'jobs' / job).exists(), 'Existing job must not be duplicated'
    files = {Path(__file__).resolve(), helper, args.permit.resolve(),
             R / 'guard_csd02.py', worker.BIN}
    for name, manifest in (
        ('csd-review-repair-timeline269', 'review-repair-timeline269-inputs.json'),
        (review['preview_job'], 'review-repair-preview'+('243' if args.chapter in ('06_one_derivative','08_loop_sums') else '274')+'-inputs.json'),
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
    files.update(p for p in worker.NARRATION.rglob('*') if p.is_file())
    score = R / 'jobs/csd-production-score21'
    files.update((score / 'score21-summary.json', score / 'resource-receipt.json',
                  R / 'jobs/csd-raster-encoder-stop13/resource-receipt.json',
                  R / 'jobs/csd-review-repair-build268/qualification.json'))
    files.update(score / 'score' / name for name in worker.read(score / 'score21-summary.json')['files'])
    inputs = Q / ('final-'+args.chapter+'-inputs278.json')
    with inputs.open('x') as stream:
        json.dump({'files': [dict(path=str(p), sha256=worker.sha(p))
                             for p in sorted(files)]}, stream, indent=2)
    subprocess.run([
        '/usr/bin/python3', '-I', '-B', str(R / 'guard_csd02.py'),
        '--phase', 'native', '--job', job, '--inputs', str(inputs), '--',
        '/usr/bin/python3', '-I', '-B', str(helper), '--chapter', args.chapter,
        '--job', job, '--inputs', str(inputs), '--permit', str(args.permit),
        '--threads', '16',
    ], check=True)

if __name__ == '__main__':
    main()
