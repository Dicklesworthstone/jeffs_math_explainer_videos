"""Prepare chapter inputs only. This file cannot start a render or approve review."""
import argparse
import importlib.util
import json
from pathlib import Path

Q = Path('/data/projects/jmev-stroke-probe-20261106/qualification01/continuous-quality16')
ROOT = Q.parents[1]

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--chapter', required=True)
    args = parser.parse_args()
    spec = importlib.util.spec_from_file_location('chapter16', Q/'final-chapter16.py')
    worker = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(worker)
    assert args.chapter in worker.P.CHAPTERS
    row, review, permit_sha = worker.check(args.chapter)
    job = 'csd-continuous-final-' + args.chapter + '-16'
    manifest = ROOT/(job+'-inputs.json')
    assert not manifest.exists() and not (ROOT/'jobs'/job).exists()
    files = {}
    def add(path, expected=None):
        path = Path(path)
        digest = worker.sha(path)
        assert expected is None or digest == expected
        assert str(path) not in files or files[str(path)] == digest
        files[str(path)] = digest
    for phase in ('preview', 'pictures'):
        name = 'csd-continuous-' + phase + '16'
        receipt = worker.success(name)
        prior = ROOT/(name+'-inputs.json')
        add(prior, receipt['inputs_sha256'])
        for item in worker.read(prior)['files']:
            add(item['path'], item['sha256'])
        for path in (ROOT/'jobs'/name).rglob('*'):
            if path.is_file():
                add(path)
    for path in [Path(__file__).resolve(), worker.PERMIT, worker.REVIEW,
                 worker.MOTION, ROOT/'guard_csd02.py', Q/'final-chapter16.py',
                 Q/'final-review-gate16.py', Q/'preview16.py', Q/'pictures16.py']:
        add(path)
    assert worker.check(args.chapter) == (row, review, permit_sha)
    for path, digest in files.items():
        assert worker.sha(Path(path)) == digest
    with manifest.open('x') as handle:
        json.dump({'files': [{'path': path, 'sha256': digest}
                            for path, digest in sorted(files.items())]}, handle, indent=2)
    print('Inputs prepared only; explicit original resource-guard admission still required:', manifest)

if __name__ == '__main__':
    main()
