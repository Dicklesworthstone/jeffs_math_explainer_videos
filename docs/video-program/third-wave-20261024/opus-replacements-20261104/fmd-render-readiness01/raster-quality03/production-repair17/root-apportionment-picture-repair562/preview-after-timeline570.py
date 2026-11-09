"""Wait for existing 567, freeze actual dependencies, launch 569 once."""
import hashlib
import json
import subprocess
import sys
import time
from pathlib import Path

R = Path('/Users/jemanuel/projects/jeffs_math_explainer_videos')
P = Path(__file__).resolve().parent
B = R/'narration_lab/third-wave-20261024/root/five-film-release-20261027'
PRIOR = B/'apportionment-picture-timeline567'
JOB = B/'apportionment-picture-preview569'
INPUTS = P/'preview569-inputs.json'

def sha(path):
    with Path(path).open('rb') as f:
        return hashlib.file_digest(f, 'sha256').hexdigest()

def main():
    assert not JOB.exists() and not INPUTS.exists()
    deadline = time.monotonic()+43200
    while time.monotonic() < deadline:
        try:
            raw = (PRIOR/'resource-receipt.json').read_bytes()
            receipt = json.loads(raw)
        except (FileNotFoundError, json.JSONDecodeError):
            time.sleep(10)
            continue
        if not receipt.get('finished'):
            time.sleep(10)
            continue
        assert receipt.get('exit') == 0
        assert not any(receipt.get(k) for k in ('error', 'resource_stop', 'stopped_for_resource_limit'))
        assert receipt['inputs_sha256'] == receipt['final_inputs_sha256']
        time.sleep(5)
        assert (PRIOR/'resource-receipt.json').read_bytes() == raw
        break
    else:
        raise SystemExit('567 wait expired; no preview launched')
    original = P/'timeline567-inputs.json'
    assert sha(original) == receipt['inputs_sha256']
    rows = {x['path']: x['sha256'] for x in json.loads(original.read_text())['files']}
    for name, digest in rows.items():
        assert sha(name) == digest, name
    extra = [original, PRIOR/'resource-receipt.json', PRIOR/'comparison567.json',
             P/'preview569.py', Path(__file__).resolve(), Path(sys.executable).resolve()]
    extra += sorted((PRIOR/'timeline').glob('*.tsv'))
    extra += [x for x in (B/'apportionment-frederick-score445/score/stems').rglob('*') if x.is_file()]
    for path in extra:
        rows[str(path)] = sha(path)
    assert not JOB.exists()
    with INPUTS.open('x') as f:
        json.dump(dict(files=[dict(path=k, sha256=v) for k, v in sorted(rows.items())]), f, indent=2)
    command = [sys.executable, '-I', '-B', str(P.parent/'guard_owner80g_target20g.py'),
               '--phase', 'native', '--job', JOB.name, '--inputs', str(INPUTS), '--',
               sys.executable, '-I', '-B', str(P/'preview569.py')]
    print('Launching guarded three-chapter preview569 with', len(rows), 'bindings', flush=True)
    return subprocess.call(command, cwd=R)

if __name__ == '__main__':
    raise SystemExit(main())
