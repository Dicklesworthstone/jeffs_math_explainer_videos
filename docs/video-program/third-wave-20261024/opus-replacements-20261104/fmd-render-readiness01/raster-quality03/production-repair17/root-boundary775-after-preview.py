"""Wait for actual766 success and exact schedules, then extract guarded review pictures."""
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import time

R = Path.home()/'scratch/jmev-complex-native-20261109'
Q = R/'qualification'
PRIOR = R/'jobs/mini-boundary-preview766'
JOB = R/'jobs/mini-boundary-pictures775'
INPUTS = Q/'root-boundary775-inputs.json'


def sha(p):
    with Path(p).open('rb') as f:
        return hashlib.file_digest(f, 'sha256').hexdigest()


def read(p):
    return json.loads(Path(p).read_text())


def success(p):
    d = read(p)
    assert d.get('admitted') and d.get('finished') and d.get('exit') == 0, p
    assert d['inputs_sha256'] == d['final_inputs_sha256'], p
    assert not any(d.get(k) for k in ('error', 'resource_stop', 'stopped_for_resource_limit')), p
    return d


def main():
    assert not JOB.exists() and not INPUTS.exists()
    deadline = time.monotonic()+86400
    while time.monotonic() < deadline:
        try:
            d = read(PRIOR/'resource-receipt.json')
        except (FileNotFoundError, json.JSONDecodeError):
            d = {}
        if d.get('finished'):
            success(PRIOR/'resource-receipt.json')
            if (PRIOR/'comparison766.json').exists():
                break
        time.sleep(15)
    else:
        raise SystemExit('766 pending; no extraction launched')
    comparison = read(PRIOR/'comparison766.json')
    assert comparison['frames'] == 16504 and len(comparison['tables']) == 3
    rows = {}
    # Retain the actual previously qualified Pillow/ffmpeg runtime bindings.
    for manifest, receipt in [(Q/'mini-boundary-pictures572-inputs.json', R/'jobs/mini-boundary-pictures572/resource-receipt.json'),
                              (Q/'root-boundary766-inputs.json', PRIOR/'resource-receipt.json')]:
        assert sha(manifest) == success(receipt)['inputs_sha256']
        for row in read(manifest)['files']:
            p = row['path']
            assert sha(p) == row['sha256'], p
            if p in rows:
                assert rows[p] == row['sha256'], p
            rows[p] = row['sha256']
        rows[str(manifest)] = sha(manifest)
        rows[str(receipt)] = sha(receipt)
    failed = R/'jobs/mini-boundary-pictures767'
    failure = read(failed/'resource-receipt.json')
    assert failure.get('finished') and failure.get('exit') == 1
    assert failure['inputs_sha256'] == failure['final_inputs_sha256']
    helper = Q/'root-boundary775-pictures.py'
    assert sha(helper) == '37c33249f1e59cf86330f539620ec1b9f7bd00b967d3639e1d1f07dd4f3067a8'
    assert sha(Q/'cc2-mini-max-guard508.py') == '135e4b2fe68f644d0d6b5b10494570b37d246b56e3fd2fd83748d545fa0a3e32'
    for p in [helper, Path(__file__).resolve(), Path(sys.executable).resolve(), Q/'cc2-mini-max-guard508.py',
              failed/'select767.filter', failed/'resource-receipt.json', failed/'command.log']:
        rows[str(p)] = sha(p)
    for p in PRIOR.rglob('*'):
        if p.is_file():
            assert not p.is_symlink(), p
            rows[str(p)] = sha(p)
    with INPUTS.open('x') as f:
        json.dump(dict(files=[dict(path=k, sha256=v) for k,v in sorted(rows.items())]), f, indent=2)
    env = dict(os.environ, PATH=str(Path.home()/'.local/bin')+':/opt/homebrew/bin:/usr/bin:/bin')
    cmd = [sys.executable, '-I', '-B', str(Q/'cc2-mini-max-guard508.py'), '--phase', 'native',
           '--job', JOB.name, '--inputs', str(INPUTS), '--', sys.executable, '-I', '-B', str(helper)]
    print('775 launching extraction with', len(rows), 'frozen bindings', flush=True)
    subprocess.run(cmd, env=env, check=True)
    success(JOB/'resource-receipt.json')
    result = read(JOB/'pictures775.json')
    print('775 extracted', len(result['pictures']), 'pictures on', len(result['sheets']), 'sheets; not yet viewed', flush=True)


if __name__ == '__main__':
    main()
