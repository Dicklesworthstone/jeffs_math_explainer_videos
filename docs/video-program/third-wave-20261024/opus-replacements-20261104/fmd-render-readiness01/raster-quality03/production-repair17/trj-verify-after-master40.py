"""Wait for successful qualified assembly, then freeze and verify once."""
from pathlib import Path
import hashlib
import json
import subprocess
import time

R = Path('/data/projects/jmev-render-worker-20261107')
D = Path('/data/projects/jmev-stroke-probe-20261106')
Q = R / 'qualification01/production19'
M = R / 'jobs/trj-production19-master01'
N = R / 'jobs/trj-production19-delivery02/narration'
JOB = 'trj-production19-verify01'
HELPER = Q / 'trj-final-verify19-02.py'

def sha(p):
    with Path(p).open('rb') as stream:
        return hashlib.file_digest(stream, 'sha256').hexdigest()

def read(p):
    return json.loads(Path(p).read_text())

def success(p):
    state = read(p)
    assert state.get('finished') and state.get('exit') == 0
    assert state['inputs_sha256'] == state['final_inputs_sha256']
    assert not any(state.get(k) for k in ('error', 'resource_stop', 'stopped_for_resource_limit'))
    return state

def main():
    assert not (R / 'jobs' / JOB).exists(), 'Verifier already exists; do not duplicate'
    fixed = [Path(__file__).resolve(), HELPER, Q / 'verify_stage09_final.py',
             Q / 'verify_video_preserve_logs01.sh', Q / 'caption-interval-audit01.py',
             R / 'guard_trj01.py']
    pins = {str(p): sha(p) for p in fixed}
    deadline = time.monotonic() + 48 * 3600
    receipt = M / 'resource-receipt.json'
    while not receipt.exists() or not read(receipt).get('finished'):
        if time.monotonic() >= deadline:
            raise RuntimeError('Master waiting deadline; no verifier launched')
        time.sleep(30)
    state = success(receipt)
    for p, digest in pins.items():
        assert sha(p) == digest, 'Prepared verifier dependency changed'
    inputs = Q / 'master01-inputs.json'
    assert sha(inputs) == state['inputs_sha256']
    files = set(fixed + [receipt, inputs, M / 'master-summary.json'])
    for row in read(inputs)['files']:
        assert sha(row['path']) == row['sha256'], row['path']
        files.add(Path(row['path']))
    summary = read(M / 'master-summary.json')
    assert summary['frames'] == 133662 and summary['native_bit_identical_replays'] == 15
    for row in summary['outputs'].values():
        assert sha(row['path']) == row['sha256']
    files.update(p for p in (M / 'final').rglob('*') if p.is_file())
    files.update(p for p in N.rglob('*') if p.is_file())
    files.add(D / 'work/production-repair19/workspace/explainer_corpus/complex_analysis_visual/script.tsv')
    manifest = Q / 'verify01-inputs.json'
    with manifest.open('x') as stream:
        json.dump({'files': [{'path': str(p), 'sha256': sha(p)}
                             for p in sorted(files)]}, stream, indent=2)
    subprocess.run(['python3', '-I', '-B', str(R / 'guard_trj01.py'),
                    '--phase', 'native', '--job', JOB, '--inputs', str(manifest),
                    '--', 'python3', '-I', '-B', str(HELPER)], check=True)
    success(R / 'jobs' / JOB / 'resource-receipt.json')

if __name__ == '__main__':
    main()
