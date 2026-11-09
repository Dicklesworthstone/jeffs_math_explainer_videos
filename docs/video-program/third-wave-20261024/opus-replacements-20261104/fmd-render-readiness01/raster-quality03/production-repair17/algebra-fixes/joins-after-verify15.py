from pathlib import Path
import hashlib
import json
import subprocess
import time

R = Path('/Users/jemanuel/projects/jeffs_math_explainer_videos')
B = R / 'narration_lab/third-wave-20261024/root/five-film-release-20261027'
Q = R / 'docs/video-program/third-wave-20261024/opus-replacements-20261104/fmd-render-readiness01/raster-quality03/production-repair17'
A = Q / 'algebra-fixes'
deadline = time.monotonic() + 90000
while True:
    p = B / 'algebra-final-verify14/resource-receipt.json'
    if p.exists():
        receipt = json.loads(p.read_text())
        if 'exit' in receipt:
            assert receipt['exit'] == 0 and receipt['inputs_sha256'] == receipt['final_inputs_sha256']
            assert not receipt.get('resource_stop')
            break
    assert time.monotonic() < deadline
    time.sleep(30)

def sha(p):
    with Path(p).open('rb') as stream:
        return hashlib.file_digest(stream, 'sha256').hexdigest()

seen = {row['path']: row for row in json.loads((A / 'final-verify14-inputs.json').read_text())['files']}
extra = [Path(__file__).resolve(), A / 'final-joins15.py', p,
         Path('/opt/homebrew/bin/fw').resolve(), Path('/opt/homebrew/bin/ffmpeg').resolve(),
         Path('/Users/jemanuel/.cache/franken_whisper/models/whisper/whisper-large-v3-turbo-f16-v1/ggml-large-v3-turbo.bin')]
for item in extra:
    seen[str(item)] = dict(path=str(item), sha256=sha(item))
for item in seen.values():
    assert sha(item['path']) == item['sha256'], item['path']
inputs = A / 'final-joins15-inputs.json'
assert not inputs.exists() and not (B / 'algebra-final-joins15').exists()
inputs.write_text(json.dumps(dict(files=list(seen.values())), indent=2))
with (A / 'final-joins15-guard.log').open('x') as log:
    subprocess.run(['python3', '-I', '-B', str(Q / 'guard_owner80g_target20g.py'), '--phase', 'native',
                    '--job', 'algebra-final-joins15', '--inputs', str(inputs), '--',
                    'python3', '-I', '-B', str(A / 'final-joins15.py')], cwd=R,
                   stdout=log, stderr=subprocess.STDOUT, check=True)
