"""Render only the eight assigned CSD chapters, under the original driver lock."""
import fcntl
import importlib.util
import json
from pathlib import Path
import subprocess
import sys

HERE = Path('/data/projects/jmev-stroke-probe-20261106/qualification01')

def load(name):
    spec = importlib.util.spec_from_file_location(name, HERE/name)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module

def launch():
    assert Path(__file__).resolve().parent == HERE
    L, R = load('launch-final02.py'), load('chapter-routing01.py')
    with (HERE/'final-driver01.lock').open('a') as lock:
        fcntl.flock(lock, fcntl.LOCK_EX | fcntl.LOCK_NB)
        review = L.F.G.check(L.F.REVIEW)
        assignment_path = HERE/'chapter-assignment01.json'
        assignment = L.S.read(assignment_path)
        R.validate(assignment, L.S.read(L.ROOT/'jobs/stage07-timeline01/timeline-summary.json')['chapters'])
        permit_path = HERE/'distributed-review-permit01.json'
        permit = L.S.read(permit_path)
        assert permit['review_gate_result'] == review
        assert permit['assignment_sha256'] == L.S.sha(assignment_path)
        frozen = L.frozen_inputs(review)
        paths = {r['path']: r['sha256'] for r in frozen['files']}
        for path in (Path(__file__).resolve(), assignment_path, permit_path,
                     HERE/'chapter-routing01.py', HERE/'launch-final02.py'):
            digest = L.S.sha(path)
            if str(path) in paths:
                assert paths[str(path)] == digest
            else:
                frozen['files'].append(dict(path=str(path), sha256=digest))
        rows = [r for r in assignment['assignments'] if r['host'] == 'csd']
        assert len(rows) == 8
        results = []
        for row in rows:
            chapter = row['chapter']
            assert chapter in L.S.CHAPTERS
            name = L.F.job_name(chapter)
            job = L.ROOT/'jobs'/name
            if job.exists():
                results.append(L.verify_completed(chapter, review))
                continue
            path = L.manifest(chapter)
            if path.exists():
                assert L.S.read(path) == frozen
            else:
                with path.open('x') as stream:
                    json.dump(frozen, stream, indent=2)
            assert L.F.G.check(L.F.REVIEW) == review
            L.B.collect(path, [])
            subprocess.run(['/usr/bin/python3', '-I', '-B',
                '/data/projects/jmev-stroke-probe-20261106/guard_csd02.py',
                '--phase', 'native', '--job', name, '--inputs', str(path), '--',
                '/usr/bin/python3', '-I', '-B',
                '/data/projects/jmev-stroke-probe-20261106/qualification01/final-render01.py',
                '--chapter', chapter], cwd=L.ROOT, check=True)
            results.append(L.verify_completed(chapter, review))
        result = dict(host='csd', chapters=results, review=review,
                      assignment_sha256=L.S.sha(assignment_path),
                      scope='Eight assigned chapters; FMD import and final master verification remain.')
        path = HERE/'final-csd-assigned01.json'
        if path.exists():
            assert L.S.read(path) == result
        else:
            with path.open('x') as stream:
                json.dump(result, stream, indent=2)

if __name__ == '__main__':
    assert sys.argv[1:] == ['--run']
    launch()
