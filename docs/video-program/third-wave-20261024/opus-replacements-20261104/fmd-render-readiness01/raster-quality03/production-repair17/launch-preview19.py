"""Freeze completed production19 inputs and launch one existing CSD guard phase.
No waiting loop, remote transport, approval generation or launch at import time.
"""
import argparse
import hashlib
import importlib.util
import json
from pathlib import Path
import shutil
import subprocess
import sys

ROOT = Path('/data/projects/jmev-stroke-probe-20261106')
Q = ROOT / 'qualification01/production-repair17'


def sha(path):
    with Path(path).open('rb') as stream:
        return hashlib.file_digest(stream, 'sha256').hexdigest()


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--phase', required=True, choices=('timeline', 'preview'))
    parser.add_argument('--chapter', action='append', default=[])
    parser.add_argument('--assembly-inputs', type=Path, default=Q / 'assembly19-inputs.json')
    args = parser.parse_args()
    assert args.phase == 'preview' or not args.chapter
    assert Path(__file__).resolve().parent == Q
    spec = importlib.util.spec_from_file_location('production19_preview', Q / 'preview19.py')
    worker = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(worker)
    worker.check_dependencies()
    job = worker.TIMELINE_JOB if args.phase == 'timeline' else worker.PREVIEW_JOB
    assert not job.exists(), 'Never duplicate an admitted job or discard its evidence'
    frozen = {}

    def add(path, expected=None):
        path = Path(path)
        digest = sha(path)
        assert expected is None or digest == expected, path
        assert str(path) not in frozen or frozen[str(path)] == digest, path
        frozen[str(path)] = digest

    def inherit(manifest, receipt):
        add(manifest, receipt['inputs_sha256'])
        rows = worker.read(manifest)['files']
        assert len(rows) == len({row['path'] for row in rows})
        for row in rows:
            add(row['path'], row['sha256'])

    for parent, manifest in ((worker.BUILD, Q / 'build19-inputs.json'),
                             (worker.ASSEMBLY, args.assembly_inputs)):
        inherit(manifest, worker.success(parent))
        for path in parent.rglob('*'):
            if path.is_file():
                add(path)
    if args.phase == 'preview':
        inherit(Q / 'timeline19-inputs.json', worker.success(worker.TIMELINE_JOB))
        for path in worker.TIMELINE_JOB.rglob('*'):
            if path.is_file():
                add(path)
    if args.phase == 'preview':
        for prior_job in (ROOT / 'jobs/csd-continuous-preview16', ROOT / 'jobs/csd-continuous-pictures16'):
            worker.success(prior_job)
            add(prior_job / 'resource-receipt.json')
        add(worker.PRIOR_EXTRACTION)
        add(worker.PRIOR_PREVIEW.parent / 'preview-summary.json')
        for directory in (worker.PRIOR_PREVIEW, worker.PRIOR12):
            assert directory.is_dir(), directory
            for path in directory.glob('*.layout.tsv'):
                add(path)
    for directory in (worker.TREE, worker.NARRATION, worker.OLD_TIMELINES):
        assert directory.is_dir(), directory
        for path in directory.rglob('*'):
            if path.is_file():
                add(path)
    for path in (Path(__file__).resolve(), Q / 'preview19.py', ROOT / 'guard_csd02.py', worker.BIN):
        add(path)
    for tool in ('python3', 'ffmpeg', 'ffprobe', 'timeout'):
        executable = shutil.which(tool)
        assert executable, tool
        add(Path(executable).resolve())
    worker.check_dependencies()
    manifest = Q / (args.phase + '17-inputs.json')
    with manifest.open('x') as stream:
        json.dump({'files': [dict(path=path, sha256=digest) for path, digest in sorted(frozen.items())]}, stream, indent=2)
    command = ['/usr/bin/python3', '-I', '-B', str(ROOT / 'guard_csd02.py'),
               '--phase', 'native', '--job', job.name, '--inputs', str(manifest), '--',
               '/usr/bin/timeout', '172900', '/usr/bin/python3', '-I', '-B', str(Q / 'preview19.py'),
               '--phase', args.phase]
    for chapter in args.chapter:
        command.extend(['--chapter', chapter])
    subprocess.run(command, check=True)
    worker.success(job)
    print('Terminal ' + args.phase + ' receipt passed; actual pictures, motion and media reuse remain separately reviewed.')


if __name__ == '__main__':
    main()
