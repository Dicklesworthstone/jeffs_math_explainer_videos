"""Serial continuation of existing CSS352 work; never creates a duplicate job."""
import hashlib
import json
from pathlib import Path
import subprocess
import time

ROOT = Path('/data/projects/jmev-css-render-worker-20261107')
Q = ROOT / 'qualification01'
WORKER = Q / 'css-final-repair352.py'
WORKER_SHA = '1074051b6063f1cd400de58422f7491d2d011f764884b63c4968b03feaf01a8a'
ROWS = [
    ('12_residues', '12_residues-css-permit370.json',
     '689164d64ba6e16a123dd9cfc68d84612af3bb698eb0b757f70f4dbbdd74c62a',
     'css-repair-permit352-12-residues370', 'css-reviewed-final-04-reading-color352'),
    ('14_continuation', '14_continuation-css-permit370.json',
     '778ca1114a4ae70dea3f03cf1af44d5aad5310bb293acbcefb191d9d3c40c661',
     'css-repair-permit352-14-continuation370', 'css-reviewed-final-12-residues352'),
    ('05_gallery', '05_gallery-css-permit375.json',
     'f3ef9aeabeb5220ad0cdd42e10868845719e67feac2680fbbad469dfae126b10',
     'css-repair-permit352-05-gallery375', 'css-reviewed-final-14-continuation352'),
]

def sha(path):
    with path.open('rb') as stream:
        return hashlib.file_digest(stream, 'sha256').hexdigest()

def terminal(path):
    if not path.exists():
        return False
    row = json.loads(path.read_text())
    if not row.get('finished'):
        return False
    assert row.get('exit') == 0, str(path)
    assert row['inputs_sha256'] == row['final_inputs_sha256'], str(path)
    assert not any(row.get(k) for k in ('error', 'resource_stop', 'stopped_for_resource_limit')), str(path)
    return True

def main():
    own = Path(__file__).resolve()
    own_sha = sha(own)
    assert sha(WORKER) == WORKER_SHA
    for chapter, _, _, _, _ in ROWS:
        assert not (ROOT / 'jobs' / ('css-reviewed-final-' + chapter.replace('_', '-') + '352')).exists()
        assert not (Q / ('final-' + chapter + '-inputs352.json')).exists()
    deadline = time.monotonic() + 24 * 3600
    for chapter, name, digest, transfer, prior in ROWS:
        waits = [ROOT / 'jobs' / j / 'resource-receipt.json' for j in (transfer, prior)]
        while True:
            assert sha(own) == own_sha and sha(WORKER) == WORKER_SHA
            states = [terminal(p) for p in waits]
            if all(states):
                break
            assert time.monotonic() < deadline, 'Prerequisites pending; nothing launched for ' + chapter
            time.sleep(20)
        permit = Q / 'repair352-permits' / name
        assert sha(permit) == digest
        job = 'css-reviewed-final-' + chapter.replace('_', '-') + '352'
        assert not (ROOT / 'jobs' / job).exists()
        assert not (Q / ('final-' + chapter + '-inputs352.json')).exists()
        command = ['/usr/bin/python3', '-I', '-B', str(WORKER), 'launch',
                   '--chapter', chapter, '--permit', str(permit), '--permit-sha256', digest,
                   '--permit-transfer', transfer, '--after', prior, '--threads', '16']
        print('Launching through original guarded worker:', chapter, flush=True)
        subprocess.run(command, check=True)
        assert terminal(ROOT / 'jobs' / job / 'resource-receipt.json')
        print('Terminal stable success:', chapter, flush=True)

if __name__ == '__main__':
    main()
