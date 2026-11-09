"""Wait for exact source552 transfer, then run one guarded build553. No render."""
import hashlib
import json
import os
from pathlib import Path
import subprocess
import time

ROOT = Path.home()/'scratch/jmev-complex-native-20261109'
Q = ROOT/'qualification'
PYTHON = '/opt/homebrew/opt/python@3.14/bin/python3.14'

def sha(p):
    with p.open('rb') as f:
        return hashlib.file_digest(f, 'sha256').hexdigest()

def main():
    manifest = json.loads((Q/'root-boundary553-inputs.json').read_text())
    assert not (ROOT/'jobs/native-boundary553').exists(), 'job already exists'
    deadline = time.monotonic()+1800
    while time.monotonic()<deadline:
        pending = []
        for row in manifest['files']:
            p = Path(row['path'])
            if not p.is_file() or p.is_symlink() or sha(p)!=row['sha256']:
                pending.append(str(p))
        if not pending:
            break
        # Only source552 may be incomplete while the existing transfer runs.
        assert all(p.startswith(str(ROOT/'source552')+'/') for p in pending), pending[:5]
        print('source files awaiting exact transfer:', len(pending), flush=True)
        time.sleep(10)
    else:
        raise SystemExit('source transfer deadline; build not launched')
    env = dict(os.environ, PATH=str(Path.home()/'.local/bin')+':/opt/homebrew/bin:/usr/bin:/bin')
    argv = [PYTHON, '-I', '-B', str(Q/'cc2-mini-max-guard508.py'), '--phase', 'native',
            '--job', 'native-boundary553', '--inputs', str(Q/'root-boundary553-inputs.json'),
            '--', PYTHON, '-I', '-B', str(Q/'root-boundary553-build.py'), '--job', 'native-boundary553']
    return subprocess.run(argv, env=env, check=False).returncode

if __name__=='__main__':
    raise SystemExit(main())
