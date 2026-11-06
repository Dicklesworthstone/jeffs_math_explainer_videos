"""One current-media native timeline to qualify cross-host schedule equality."""
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys

R = Path('/data/projects/jmev-stroke-probe-20261106')
Q = R/'qualification01/raster-quality12'
JOB = 'csd-raster-timeline-path17'
N = R/'jobs/csd-raster-media15/narration'
B = R/'work/bin/production09i'

def sha(p):
    with p.open('rb') as f:
        return hashlib.file_digest(f, 'sha256').hexdigest()

def main():
    if '--child' in sys.argv:
        out = R/'jobs'/JOB
        receipt = json.loads((out/'resource-receipt.json').read_text())
        assert receipt['admitted'] and 'exit' not in receipt
        assert receipt['command'] == ['/usr/bin/python3', '-I', '-B', str(Q/Path(__file__).name), '--child']
        os.sched_setaffinity(0, {0, 1, 2, 3})
        with (out/'native.stdout').open('x') as stdout, (out/'native.stderr').open('x') as stderr:
            subprocess.run([str(B), 'render', '01_the_bump', '--timeline-only', '--fps', '60', '--narration', str(N), '--out', str(out/'timeline'), '--lint'], cwd=R/'work/production-repo07', stdout=stdout, stderr=stderr, check=True, timeout=1800)
        return
    assert not (R/'jobs'/JOB).exists()
    paths = {Path(__file__).resolve(), B}
    for row in json.loads((R/'csd-raster-media15-inputs.json').read_text())['files']:
        p = Path(row['path'])
        assert sha(p) == row['sha256'], p
        paths.add(p)
    paths.update(p for p in N.rglob('*') if p.is_file())
    for job in ('csd-raster-media15', 'csd-raster-encoder-stop13'):
        p = R/'jobs'/job/'resource-receipt.json'
        r = json.loads(p.read_text())
        assert r['exit'] == 0 and r['inputs_sha256'] == r['final_inputs_sha256'] and not r.get('resource_stop')
        paths.add(p)
    manifest = R/(JOB+'-inputs.json')
    with manifest.open('x') as f:
        json.dump({'files': [{'path': str(p), 'sha256': sha(p)} for p in sorted(paths)]}, f, indent=2)
    subprocess.run(['/usr/bin/python3', '-I', '-B', str(R/'guard_csd02.py'), '--phase', 'native', '--job', JOB, '--inputs', str(manifest), '--', '/usr/bin/python3', '-I', '-B', str(Q/Path(__file__).name), '--child'], check=True)

if __name__ == '__main__':
    main()
