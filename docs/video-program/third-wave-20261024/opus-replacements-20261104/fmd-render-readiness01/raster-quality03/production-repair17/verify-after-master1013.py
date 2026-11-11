"""Resume only verification of completed master1006; never rerender or overwrite."""
import hashlib
import json
from pathlib import Path
import subprocess
import time

D = Path('/data/projects/jmev-stroke-probe-20261106')
Q = D/'qualification01/production-repair17'
HELPER = Q/'csd-final-verify-repaired03.py'
PIN = 'c63a011c93224cc40c485d0cc9889440310ae7eadcb5260e4340d5142658f767'


def run(mode):
    assert hashlib.sha256(HELPER.read_bytes()).hexdigest() == PIN
    print(time.strftime('%Y-%m-%dT%H:%M:%SZ', time.gmtime()), mode, flush=True)
    subprocess.run(['/usr/bin/python3', '-I', '-B', str(HELPER), mode], check=True)


def main():
    receipt = json.loads((D/'jobs/csd-combined-master1006/resource-receipt.json').read_text())
    assert receipt['exit'] == 0 and receipt['finished']
    assert receipt['inputs_sha256'] == receipt['final_inputs_sha256']
    assert not receipt.get('error') and not receipt.get('resource_stop')
    for name in ('csd-final-verify1013', 'csd-final-pictures1013-overview',
                 'csd-final-pictures1013-native-a', 'csd-final-pictures1013-native-b'):
        assert not (D/'jobs'/name).exists(), 'Never duplicate '+name
    run('check')
    run('launch-verify')
    for part in ('overview', 'native-a', 'native-b'):
        run('launch-pictures')
        job = D/'jobs'/('csd-final-pictures1013-'+part)
        receipt = json.loads((job/'resource-receipt.json').read_text())
        assert receipt['exit'] == 0 and receipt['finished']
        digest = hashlib.sha256((job/'sampling.json').read_bytes()).hexdigest()
        with (Q/('final-pictures1013-'+part+'-anchor.json')).open('x') as f:
            json.dump(dict(part=part, sampling_sha256=digest), f, indent=2)
        print(part, 'sampling_sha256', digest, flush=True)
    print('Technical jobs completed; actual final picture review and publication remain.', flush=True)


if __name__ == '__main__':
    main()
