"""Wait for the existing delivery job, freeze its complete inputs, run one timeline job."""
import hashlib
import json
from pathlib import Path
import subprocess
import sys
import time

D = Path('/data/projects/jmev-stroke-probe-20261106')
Q = D/'qualification01/production-repair17'
BUILD = D/'jobs/csd-native-combined-build01'
DELIVERY = D/'jobs/csd-native-combined-delivery01'
JOB = D/'jobs/csd-native-combined-timeline01'
INPUTS = Q/'csd-combined01-timeline-inputs.json'
SOURCE = D/'work/native-combined01'


def sha(p):
    with p.open('rb') as f:
        return hashlib.file_digest(f, 'sha256').hexdigest()


def read(p):
    return json.loads(p.read_text())


def success(p):
    r = read(p)
    assert r.get('admitted') and r.get('finished') and r.get('exit') == 0, p
    assert r['inputs_sha256'] == r['final_inputs_sha256'], p
    assert not any(r.get(k) for k in ('error', 'resource_stop', 'stopped_for_resource_limit')), p
    return r


def main():
    assert not JOB.exists() and not INPUTS.exists()
    deadline = time.monotonic()+43200
    while time.monotonic() < deadline:
        for job in (BUILD, DELIVERY):
            try:
                r = read(job/'resource-receipt.json')
            except (FileNotFoundError, json.JSONDecodeError):
                continue
            if 'finished' in r:
                success(job/'resource-receipt.json')
                if job == DELIVERY:
                    break
        else:
            time.sleep(15)
            continue
        break
    else:
        raise SystemExit('Delivery not terminal; timeline not launched')
    receipt = success(DELIVERY/'resource-receipt.json')
    manifest = Q/'csd-combined01-delivery-inputs.json'
    assert sha(manifest) == receipt['inputs_sha256']
    rows = {x['path']: x['sha256'] for x in read(manifest)['files']}
    for name, digest in rows.items():
        assert sha(Path(name)) == digest, name
    delivery = read(DELIVERY/'delivery-combined01.json')
    assert delivery['wavs_byte_identical'] == 105
    assert delivery['delivery']['repaired']['selected_cues'] == 20
    for name, digest in delivery['source_after'].items():
        p = SOURCE/name
        assert sha(p) == digest, p
        rows[str(p)] = digest
    for root in (BUILD, DELIVERY):
        for p in root.rglob('*'):
            assert not p.is_symlink(), p
            if p.is_file():
                rows[str(p)] = sha(p)
    for p in (manifest, Path(__file__).resolve(), Q/'csd-combined01-timeline.py',
              Path(sys.executable).resolve(), Path('/usr/bin/ffmpeg'), Path('/usr/bin/ffprobe')):
        rows[str(p)] = sha(p)
    with INPUTS.open('x') as f:
        json.dump(dict(files=[dict(path=k, sha256=v) for k, v in sorted(rows.items())]), f, indent=2)
    subprocess.run([sys.executable, '-I', '-B', str(D/'guard_csd02.py'), '--phase', 'native',
        '--job', JOB.name, '--inputs', str(INPUTS), '--', sys.executable, '-I', '-B',
        str(Q/'csd-combined01-timeline.py'), '--run'], check=True)
    success(JOB/'resource-receipt.json')
    assert len(read(JOB/'comparison-combined01.json')['chapters']) == 3
    print('Three normalized timelines equal; rendered pictures and AV remain unaccepted.', flush=True)


if __name__ == '__main__':
    main()
