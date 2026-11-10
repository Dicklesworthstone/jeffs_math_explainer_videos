"""Wait for successful final772, freeze actual outputs, and run guarded pictures776 once."""
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import time

ROOT = Path('/Users/jemanuel/scratch/jmev-complex-native-20261109')
Q = ROOT/'qualification'
PRIOR = ROOT/'jobs/mini-boundary-final772'
JOB = ROOT/'jobs/mini-boundary-pictures776'
INPUTS = Q/'mini-boundary-pictures776-inputs.json'


def sha(p):
    with Path(p).open('rb') as f:
        return hashlib.file_digest(f, 'sha256').hexdigest()


def read(p):
    return json.loads(Path(p).read_text())


def success(d):
    assert d.get('admitted') and d.get('finished') and d.get('exit') == 0
    assert not d.get('preflight_only')
    assert d['inputs_sha256'] == d['final_inputs_sha256']
    assert not any(d.get(k) for k in ('error', 'resource_stop', 'stopped_for_resource_limit'))


def main():
    assert not JOB.exists() and not INPUTS.exists()
    receipt = PRIOR/'resource-receipt.json'
    deadline = time.monotonic()+86400
    while time.monotonic() < deadline:
        try:
            raw = receipt.read_bytes()
            d = json.loads(raw)
        except (FileNotFoundError, json.JSONDecodeError):
            time.sleep(15)
            continue
        if d.get('finished'):
            success(d)
            time.sleep(5)
            assert receipt.read_bytes() == raw
            break
        time.sleep(15)
    else:
        raise SystemExit('772 pending at wait deadline; no extraction launched')
    original = Q/'root-boundary772-final-inputs.json'
    assert sha(original) == d['inputs_sha256']
    entries = read(original)['files']
    rows = {x['path']: x['sha256'] for x in entries}
    assert len(rows) == len(entries)
    for name, digest in rows.items():
        assert sha(name) == digest, name
    helper = Q/'root-boundary776-pictures.py'
    guard = Q/'cc2-mini-max-guard508.py'
    assert sha(helper) == '9ba6a9b5e4941a2428487d6778a1796364cf81d6e3118c682a803e004efbfd39'
    assert sha(guard) == '135e4b2fe68f644d0d6b5b10494570b37d246b56e3fd2fd83748d545fa0a3e32'
    for p in list(PRIOR.rglob('*')) + [original, helper, guard, Path(__file__).resolve(), Path(sys.executable).resolve()]:
        if p.is_file():
            assert not p.is_symlink(), p
            digest = sha(p)
            assert str(p) not in rows or rows[str(p)] == digest, p
            rows[str(p)] = digest
    assert receipt.read_bytes() == raw
    with INPUTS.open('x') as f:
        json.dump({'files': [{'path': p, 'sha256': h} for p,h in sorted(rows.items())]}, f, indent=2)
        f.write('\n')
    command = [sys.executable, '-I', '-B', str(guard), '--phase', 'native',
               '--job', JOB.name, '--inputs', str(INPUTS), '--', sys.executable, '-I', '-B', str(helper)]
    env = dict(os.environ, PATH='/Users/jemanuel/.local/bin:/opt/homebrew/bin:/usr/bin:/bin')
    print('788 launching guarded776 with', len(rows), 'frozen files', flush=True)
    subprocess.run(command, env=env, check=True)
    success(read(JOB/'resource-receipt.json'))
    result = read(JOB/'pictures776.json')
    print('776 extracted', len(result['pictures']), 'pictures;', len(result['sheets']), 'sheets. NOT YET VIEWED.', flush=True)


if __name__ == '__main__':
    main()
