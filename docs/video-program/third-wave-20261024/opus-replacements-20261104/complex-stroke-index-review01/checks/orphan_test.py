"""Scratch only. Does the guard's stop reach a render started under probe02's inner `timeout`?

Same process shape as the guard: a new session running env -> timeout -> bash -> subshell | tee,
then SIGTERM to that one process group, as stop_owned does. The render is a 25 s heartbeat, so a
survivor ends by itself. Nothing here touches the repository or a real job.
"""
import os
import signal
import subprocess
import sys
import tempfile
import time
from pathlib import Path

INNER = {
    'inner timeout (probe02 ab/film/profile)': 'timeout 21600 bash "$HEART"',
    'no inner timeout (proposed)': 'bash "$HEART"',
    'inner timeout --foreground': 'timeout --foreground 21600 bash "$HEART"',
}

HEART = """#!/bin/bash
for i in $(seq 1 50); do date +%s >> "$BEATS"; sleep 0.5; done
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


def trial(label, inner):
    root = Path(tempfile.mkdtemp(prefix='orphan-', dir=sys.argv[1]))
    heart, step = root / 'heart.sh', root / 'step.sh'
    heart.write_text(HEART)
    step.write_text(STEP % inner)
    env = dict(os.environ, HEART=str(heart), BEATS=str(root / 'beats'), LOG=str(root / 'log'))
    child = subprocess.Popen(['/usr/bin/env', 'WORK=x', 'timeout', '60', '/bin/bash', str(step)], cwd=root, env=env,
                             stdout=open(root / 'command.log', 'x'), stderr=subprocess.STDOUT, start_new_session=True)
    time.sleep(2)
    before = beats(root / 'beats')
    os.killpg(child.pid, signal.SIGTERM)
    try:
        child.wait(timeout=20)
    except subprocess.TimeoutExpired:
        os.killpg(child.pid, signal.SIGKILL)
        child.wait(timeout=10)
    at_stop = beats(root / 'beats')
    time.sleep(3)
    after = beats(root / 'beats')
    print(f'{label}: guard child exit {child.returncode}; beats at stop {at_stop} (had {before}), 3 s later {after}: '
          + ('RENDER SURVIVED THE STOP' if after > at_stop else 'render stopped'))


for name, inner in INNER.items():
    trial(name, inner)
