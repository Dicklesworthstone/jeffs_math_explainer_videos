"""Wait for existing269, then invoke independently reviewed296 once."""
from pathlib import Path
import hashlib
import json
import subprocess
import time

R = Path('/data/projects/jmev-stroke-probe-20261106')
Q = R / 'qualification01/production-repair17'
receipt = R / 'jobs/csd-review-repair-timeline269/resource-receipt.json'
launcher = Q / 'launch-reviewed-preview296.py'
assert hashlib.sha256(launcher.read_bytes()).hexdigest() == '8f754ec7b743615cfc6fa27642936462f24a24237a9c83cea1864c8b78d9a2b0'
assert not (R / 'jobs/csd-review-repair-preview274').exists()
for _ in range(2160):
    try:
        state = json.loads(receipt.read_text())
    except json.JSONDecodeError:
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
    '--worker-sha256', 'd80cf7b6ffba56968f7c6dfd7a2cca50e65769dce1c97deb5d34683ed3fc9669',
    '--review', str(Q / 'cc2-preview-worker-review284.md'),
    '--review-sha256', '6e9b312f6febb95f6eeacd53a6ef9bb72513113b8519cf12f101dcb065066365',
], check=True)
print('Preview274 guard completed; actual picture/motion/audio review remains separate.', flush=True)
