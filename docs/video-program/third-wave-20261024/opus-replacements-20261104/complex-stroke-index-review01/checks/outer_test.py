"""Scratch only. When root's OUTER timeout expires, does it reach the render? Same shape as orphan_test.py, no guard signal."""
import os
import subprocess
import sys
import tempfile
import time
from pathlib import Path

HEART = """#!/bin/bash
for i in $(seq 1 40); do date +%s >> "$BEATS"; sleep 0.5; done
"""
STEP = """#!/bin/bash
set -euo pipefail
ab() {
    (cd "$PWD" && %s) 2>&1 | tee "$LOG"
}
ab
"""


def beats(path):
    return len(path.read_text().splitlines()) if path.exists() else 0


for label, inner in {'inner timeout': 'timeout 21600 bash "$HEART"', 'no inner timeout': 'bash "$HEART"'}.items():
    root = Path(tempfile.mkdtemp(prefix='outer-', dir=sys.argv[1]))
    heart, step = root / 'heart.sh', root / 'step.sh'
    heart.write_text(HEART)
    step.write_text(STEP % inner)
    env = dict(os.environ, HEART=str(heart), BEATS=str(root / 'beats'), LOG=str(root / 'log'))
    child = subprocess.Popen(['/usr/bin/env', 'WORK=x', 'timeout', '2', '/bin/bash', str(step)], cwd=root, env=env,
                             stdout=open(root / 'command.log', 'x'), stderr=subprocess.STDOUT, start_new_session=True)
    child.wait(timeout=20)
    time.sleep(0.2)
    at_stop = beats(root / 'beats')
    time.sleep(3)
    after = beats(root / 'beats')
    print(f'{label}: outer timeout exit {child.returncode}; beats {at_stop}, 3 s later {after}: '
          + ('RENDER SURVIVED' if after > at_stop else 'render stopped'))
