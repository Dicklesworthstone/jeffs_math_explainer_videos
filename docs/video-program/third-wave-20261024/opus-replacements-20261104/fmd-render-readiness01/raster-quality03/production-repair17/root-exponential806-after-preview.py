"""Wait for successful stable805, freeze its actual outputs, then guarded extraction806 once."""
import hashlib
import json
from pathlib import Path
import subprocess
import time

R = Path('/Users/jemanuel/projects/jeffs_math_explainer_videos')
Q = R/'docs/video-program/third-wave-20261024/opus-replacements-20261104/fmd-render-readiness01/raster-quality03/production-repair17'
W = R/'narration_lab/third-wave-20261024/root/five-film-release-20261027'
P = W/'exponential-native-preview805'


def sha(p):
    with p.open('rb') as f:
        return hashlib.file_digest(f, 'sha256').hexdigest()


def main():
    deadline = time.monotonic()+14400
    while True:
        assert time.monotonic() < deadline, 'Wait deadline; inspect805, do not duplicate'
        try:
            receipt = json.loads((P/'resource-receipt.json').read_text())
        except (FileNotFoundError, json.JSONDecodeError):
            time.sleep(15)
            continue
        if 'finished' not in receipt:
            time.sleep(15)
            continue
        assert receipt['exit'] == 0 and not receipt.get('resource_stop'), receipt
        break
    original = W/'exponential-native-preview805-inputs.json'
    assert sha(original) == receipt['inputs_sha256'] == receipt['final_inputs_sha256']
    files = {r['path']: r['sha256'] for r in json.loads(original.read_text())['files']}
    for p, h in files.items():
        assert sha(Path(p)) == h, p
    extra = [p for p in P.rglob('*') if p.is_file()]
    extra += [p for p in (W/'exponential-native-preview800'/'render').glob('*') if p.is_file() and p.stem in ['09_charging_gap','13_turning_rate','14_euler_identity']]
    extra += [original, Path(__file__), Q/'root-exponential806-pictures.py', Q/'guard_owner80g_target20g.py',
              Path('/opt/homebrew/bin/ffmpeg'), Path('/opt/homebrew/bin/ffprobe')]
    for p in extra:
        files[str(p)] = sha(p)
    manifest = W/'exponential-native-pictures806-inputs.json'
    assert not manifest.exists() and not (W/'exponential-native-pictures806').exists()
    with manifest.open('x') as f:
        json.dump(dict(files=[dict(path=p, sha256=files[p]) for p in sorted(files)]), f, indent=2)
    print(f'Launching guarded806 after successful stable805; {len(files)} frozen inputs', flush=True)
    subprocess.run(['/opt/homebrew/bin/python3', '-I', '-B', str(Q/'guard_owner80g_target20g.py'),
                    '--phase', 'native', '--job', 'exponential-native-pictures806', '--inputs', str(manifest),
                    '--', '/opt/homebrew/bin/python3', '-I', '-B', str(Q/'root-exponential806-pictures.py')], check=True)


if __name__ == '__main__':
    main()
