"""Wait for timeline313, then invoke the preview313 launcher once.

Adapted from preview-after-timeline301.py by cc0 (not run by cc0). The launcher and worker are
pinned here; the independent review of the preview worker is root's to name, so it is passed in.
Usage: python3 -I -B preview-after-timeline313.py --review PATH --review-sha256 HASH
"""
from pathlib import Path
import argparse
import hashlib
import json
import subprocess
import time

R = Path('/data/projects/jmev-stroke-probe-20261106')
Q = R / 'qualification01/production-repair17'
receipt = R / 'jobs/csd-review-repair-timeline313/resource-receipt.json'
launcher = Q / 'launch-review-repair-preview313.py'
LAUNCHER_SHA256 = 'c983ecbff973cf5c300ec1f8b8837a175b0a41f8aa294bd7826f9a1b4d42806a'
WORKER_SHA256 = '67d780d83bf05fbcd0285f0599197649f8ce8813ef6599a5da932301dc354e50'
ap = argparse.ArgumentParser()
ap.add_argument('--review', type=Path, required=True)
ap.add_argument('--review-sha256', required=True)
a = ap.parse_args()
assert hashlib.sha256(launcher.read_bytes()).hexdigest() == LAUNCHER_SHA256
assert hashlib.sha256((Q / 'review-repair-preview313.py').read_bytes()).hexdigest() == WORKER_SHA256
assert hashlib.sha256(a.review.resolve(strict=True).read_bytes()).hexdigest() == a.review_sha256
assert not (R / 'jobs/csd-review-repair-preview313').exists()
for _ in range(2160):
    try:
        state = json.loads(receipt.read_text())
    except (FileNotFoundError, json.JSONDecodeError):
        time.sleep(20)
        continue
    if state.get('finished'):
        assert state.get('exit') == 0, 'Timeline failed; preview not launched'
        break
    time.sleep(20)
else:
    raise RuntimeError('Wait deadline; existing timeline untouched, preview not launched')
subprocess.run([
    '/usr/bin/python3', '-I', '-B', str(launcher),
    '--worker-sha256', WORKER_SHA256,
    '--review', str(a.review.resolve(strict=True)),
    '--review-sha256', a.review_sha256,
], check=True)
print('Preview313 guard completed; actual picture/motion/audio review remains separate.', flush=True)
