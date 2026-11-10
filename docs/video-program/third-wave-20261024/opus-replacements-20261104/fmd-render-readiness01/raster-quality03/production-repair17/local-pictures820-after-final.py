"""Wait for successful final772, freeze actual outputs, and run guarded pictures776 once."""
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import time

ROOT = Path('/Users/jemanuel/scratch/jmev-complex-native-20261109')
Q = Path('/Users/jemanuel/projects/jeffs_math_explainer_videos/docs/video-program/third-wave-20261024/opus-replacements-20261104/fmd-render-readiness01/raster-quality03/production-repair17')
PRIOR = Path('/Users/jemanuel/projects/jeffs_math_explainer_videos/narration_lab/third-wave-20261024/root/five-film-release-20261027/complex-local-final819')
JOB = Path('/Users/jemanuel/projects/jeffs_math_explainer_videos/narration_lab/third-wave-20261024/root/five-film-release-20261027/complex-local-pictures820')
INPUTS = Q/'local-pictures820-inputs.json'


def sha(p):
    with Path(p).open('rb') as f:
        return hashlib.file_digest(f, 'sha256').hexdigest()


def read(p):
    return json.loads(Path(p).read_text())


def success(d):
    assert d.get('admitted') and d.get('finished') and d.get('exit') == 0
    assert not d.get('preflight_only')
    assert d['inputs_sha256'] == d['final_inputs_sha256']
    assert not any(d.get(k) for k in ('error', 'resource_stop', 'stopped_for_resource_limit'))


def main():
    assert not JOB.exists() and not INPUTS.exists()
    receipt = PRIOR/'resource-receipt.json'
    deadline = time.monotonic()+86400
    while time.monotonic() < deadline:
        try:
            raw = receipt.read_bytes()
            d = json.loads(raw)
        except (FileNotFoundError, json.JSONDecodeError):
            time.sleep(15)
            continue
        if d.get('finished'):
            success(d)
            time.sleep(5)
            assert receipt.read_bytes() == raw
            break
        time.sleep(15)
    else:
        raise SystemExit('819 pending at wait deadline; no extraction launched')
    original = Q/'local-final819-inputs.json'
    assert sha(original) == d['inputs_sha256']
    entries = read(original)['files']
    rows = {x['path']: x['sha256'] for x in entries}
    assert len(rows) == len(entries)
    for name, digest in rows.items():
        assert sha(name) == digest, name
    helper = Q/'local-pictures820.py'
    guard = Q/'local-guard814.py'
    assert sha(helper) == '90266000ee97698460893d2a1fcdca51842d51d042265c90323c3ecbde34704b'
    assert sha(guard) == 'a3de7e3e856f132b5a4ba5c5e0fc6b273d47c78916779e799d1d3897301f1947'
    import PIL
    pillow = Path(PIL.__file__).resolve().parent
    runtime = [p for p in pillow.rglob('*') if p.is_file() and '__pycache__' not in p.parts]
    runtime += [Path('/opt/homebrew/bin/ffprobe').resolve(), Path('/opt/homebrew/bin/ffmpeg').resolve()]
    for p in list(PRIOR.rglob('*')) + runtime + [original, helper, guard, Path(__file__).resolve(), Path(sys.executable).resolve()]:
        if p.is_file():
            assert not p.is_symlink(), p
            digest = sha(p)
            assert str(p) not in rows or rows[str(p)] == digest, p
            rows[str(p)] = digest
    assert receipt.read_bytes() == raw
    with INPUTS.open('x') as f:
        json.dump({'files': [{'path': p, 'sha256': h} for p,h in sorted(rows.items())]}, f, indent=2)
        f.write('\n')
    command = [sys.executable, '-I', '-B', str(guard), '--phase', 'native',
               '--job', JOB.name, '--inputs', str(INPUTS), '--', sys.executable, '-I', '-B', str(helper)]
    env = dict(os.environ, PATH='/Users/jemanuel/.local/bin:/opt/homebrew/bin:/usr/bin:/bin')
    print('820 launching extraction with', len(rows), 'frozen files', flush=True)
    subprocess.run(command, env=env, check=True)
    success(read(JOB/'resource-receipt.json'))
    result = read(JOB/'pictures820.json')
    print('820 extracted', len(result['pictures']), 'pictures;', len(result['sheets']), 'sheets. NOT YET VIEWED.', flush=True)


if __name__ == '__main__':
    main()
