"""Wait for boundary final586; freeze its actual files and run guarded pictures587 once."""
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import time
import PIL

ROOT = Path('/Users/jemanuel/scratch/jmev-complex-native-20261109')
Q = ROOT/'qualification'
PREVIEW = ROOT/'jobs/mini-boundary-final586'
JOB = ROOT/'jobs/mini-boundary-pictures587'
INPUTS = Q/'mini-boundary-pictures587-inputs.json'


def sha(p):
    with p.open('rb') as f:
        return hashlib.file_digest(f, 'sha256').hexdigest()


def main():
    assert not JOB.exists() and not INPUTS.exists()
    deadline = time.monotonic()+86400
    while time.monotonic() < deadline:
        try:
            raw = (PREVIEW/'resource-receipt.json').read_bytes()
            r = json.loads(raw)
        except (FileNotFoundError, json.JSONDecodeError):
            time.sleep(10)
            continue
        if 'finished' not in r:
            time.sleep(10)
            continue
        assert r.get('exit') == 0 and not r.get('resource_stop'), r
        assert r['inputs_sha256'] == r['final_inputs_sha256']
        time.sleep(5)
        assert (PREVIEW/'resource-receipt.json').read_bytes() == raw
        break
    else:
        raise SystemExit('588: preview still pending at wait deadline; no job launched')
    original = Q/'root-boundary586-final-inputs.json'
    assert sha(original) == r['inputs_sha256']
    manifest = json.loads(original.read_text())
    rows = {x['path']: x['sha256'] for x in manifest['files']}
    for name, digest in rows.items():
        assert sha(Path(name)) == digest, name
    for base in [PREVIEW, Path(PIL.__file__).parent]:
        for p in base.rglob('*'):
            if p.is_file() and '__pycache__' not in p.parts:
                assert not p.is_symlink(), p
                rows[str(p)] = sha(p)
    for p in [Path(__file__).resolve(), Q/'root-boundary587-pictures.py', original, Path(sys.executable).resolve()]:
        rows[str(p)] = sha(p)
    assert not JOB.exists()
    with INPUTS.open('x') as f:
        json.dump({'files': [{'path': k, 'sha256': v} for k, v in sorted(rows.items())]}, f, indent=2)
        f.write('\n')
    command = [sys.executable, '-I', '-B', str(Q/'cc2-mini-max-guard508.py'), '--phase', 'native',
               '--job', JOB.name, '--inputs', str(INPUTS), '--', sys.executable, '-I', '-B',
               str(Q/'root-boundary587-pictures.py')]
    print('588 launching guarded extraction with %d frozen files' % len(rows), flush=True)
    env = dict(os.environ, PATH='/Users/jemanuel/.local/bin:/opt/homebrew/bin:/usr/bin:/bin')
    return subprocess.run(command, env=env).returncode


if __name__ == '__main__':
    raise SystemExit(main())
