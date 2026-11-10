"""Run full stage11 native qualification only after successful stable apportionment verification771."""
import hashlib
import json
import os
from pathlib import Path
import subprocess
import time

D = Path('/data/projects/jmev-stroke-probe-20261106')
Q = D/'qualification01/production-repair17'
JOB = D/'jobs/csd-exponential-build774'
INPUTS = Q/'csd-exponential-build774-inputs.json'
PYTHON = '/usr/bin/python3'


def sha(path):
    with path.open('rb') as f:
        return hashlib.file_digest(f, 'sha256').hexdigest()


def main():
    assert not JOB.exists(), 'duplicate build refused'
    deadline = time.monotonic() + 43200
    receipt = D/'jobs/csd-apportionment-verify771/resource-receipt.json'
    while True:
        assert time.monotonic() < deadline, 'verification wait expired; inspect existing jobs'
        try:
            master = json.loads((D/'jobs/csd-apportionment-master770/resource-receipt.json').read_text())
        except (FileNotFoundError, json.JSONDecodeError):
            master = {}
        if 'finished' in master:
            assert master.get('exit') == 0 and not any(master.get(k) for k in ('error', 'resource_stop', 'stopped_for_resource_limit'))
            assert master['inputs_sha256'] == master['final_inputs_sha256']
        try:
            r = json.loads(receipt.read_text())
        except (FileNotFoundError, json.JSONDecodeError):
            time.sleep(10)
            continue
        if 'finished' not in r:
            time.sleep(10)
            continue
        assert r.get('exit') == 0 and not any(r.get(k) for k in ('error', 'resource_stop', 'stopped_for_resource_limit'))
        assert r['inputs_sha256'] == r['final_inputs_sha256']
        before = sha(receipt)
        time.sleep(15)
        assert sha(receipt) == before
        break
    rows = json.loads(INPUTS.read_text())['files']
    assert len(rows) == len({r['path'] for r in rows})
    for row in rows:
        assert sha(Path(row['path'])) == row['sha256'], row['path']
    assert not JOB.exists()
    command = [PYTHON, '-I', '-B', str(D/'guard_csd02.py'), '--phase', 'native', '--job', JOB.name,
               '--inputs', str(INPUTS), '--', PYTHON, '-I', '-B', str(Q/'root-exponential774-build.py'),
               'run', '--host', 'csd', '--job', JOB.name, '--source', str(Q/'root-exponential774-source')]
    print('774 launching full stage11 native qualification after stable771', flush=True)
    subprocess.run(command, check=True, env=dict(os.environ, PATH='/home/ubuntu/.local/bin:/usr/local/bin:/usr/bin:/bin'))


if __name__ == '__main__':
    main()
