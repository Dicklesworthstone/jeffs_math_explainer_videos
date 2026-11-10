"""Wait for actual stable master770, then freeze and run guarded verifier771 once."""
import hashlib
import json
import os
from pathlib import Path
import subprocess
import time

D = Path('/data/projects/jmev-stroke-probe-20261106')
Q = D/'qualification01/production-repair17'
RECEIPT = D/'jobs/csd-apportionment-master770/resource-receipt.json'
HELPER = Q/'root-apportionment771-verify.py'
INPUTS = Q/'root-apportionment771-inputs.json'
JOB = D/'jobs/csd-apportionment-verify771'
PYTHON = '/usr/bin/python3'


def sha(p):
    return hashlib.sha256(p.read_bytes()).hexdigest()


def main():
    assert not JOB.exists() and not INPUTS.exists(), 'duplicate verifier refused'
    deadline = time.monotonic() + 43200
    while True:
        assert time.monotonic() < deadline, 'master wait deadline; inspect existing job, never duplicate'
        try:
            record = json.loads(RECEIPT.read_text())
        except (FileNotFoundError, json.JSONDecodeError):
            time.sleep(10)
            continue
        if 'finished' not in record:
            time.sleep(10)
            continue
        assert record.get('exit') == 0 and not record.get('error') and not record.get('resource_stop'), record
        assert record['inputs_sha256'] == record['final_inputs_sha256']
        before = sha(RECEIPT)
        time.sleep(15)
        assert sha(RECEIPT) == before, 'terminal receipt changed'
        break
    env = dict(os.environ, PATH='/home/ubuntu/.local/bin:/usr/local/bin:/usr/bin:/bin')
    subprocess.run([PYTHON, '-I', '-B', str(HELPER), 'inputs'], check=True, env=env)
    assert not JOB.exists()
    command = [PYTHON, '-I', '-B', str(D/'guard_csd02.py'), '--phase', 'native',
               '--job', JOB.name, '--inputs', str(INPUTS), '--', PYTHON, '-I', '-B', str(HELPER), 'run']
    print('771 launching guarded verification after stable770', flush=True)
    subprocess.run(command, check=True, env=env)


if __name__ == '__main__':
    main()
