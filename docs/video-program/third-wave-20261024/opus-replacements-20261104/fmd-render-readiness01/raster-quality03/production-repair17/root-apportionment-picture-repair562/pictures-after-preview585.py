"""Wait for existing569; prepare and run guarded584 extraction once. No picture acceptance."""
import hashlib
import json
import subprocess
import sys
import time
from pathlib import Path

R = Path('/Users/jemanuel/projects/jeffs_math_explainer_videos')
P = Path(__file__).resolve().parent
B = R/'narration_lab/third-wave-20261024/root/five-film-release-20261027'
PRIOR = B/'apportionment-picture-preview569'
JOB = B/'apportionment-picture-review584'
INPUTS = P/'pictures584-inputs.json'

def sha(path):
    with Path(path).open('rb') as f:
        return hashlib.file_digest(f, 'sha256').hexdigest()

def main():
    assert not JOB.exists() and not INPUTS.exists()
    helper = P/'pictures584.py'
    pins = {p: sha(p) for p in (helper, Path(__file__).resolve(), P.parent/'cc0-apportionment-preview-review462.py')}
    deadline = time.monotonic()+43200
    while time.monotonic() < deadline:
        for path, digest in pins.items():
            assert sha(path) == digest, path
        try:
            state = json.loads((PRIOR/'resource-receipt.json').read_text())
        except (FileNotFoundError, json.JSONDecodeError):
            time.sleep(15)
            continue
        if not state.get('finished'):
            time.sleep(15)
            continue
        assert state.get('exit') == 0 and state['inputs_sha256'] == state['final_inputs_sha256']
        assert not any(state.get(k) for k in ('error', 'resource_stop', 'stopped_for_resource_limit'))
        break
    else:
        raise SystemExit('569 wait expired; no extraction launched')
    subprocess.run([sys.executable, '-I', '-B', str(helper), 'inputs'], check=True, cwd=R)
    command = [sys.executable, '-I', '-B', str(P.parent/'guard_owner80g_target20g.py'),
               '--phase', 'native', '--job', JOB.name, '--inputs', str(INPUTS), '--',
               sys.executable, '-I', '-B', str(helper), 'extract']
    print('Launching guarded584 picture extraction', flush=True)
    return subprocess.call(command, cwd=R)

if __name__ == '__main__':
    raise SystemExit(main())
