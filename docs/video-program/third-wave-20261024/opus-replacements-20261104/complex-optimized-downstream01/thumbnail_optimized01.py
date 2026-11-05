"""Render the authored native thumbnail after the optimized preview; do not publish.

Root's thumbnail-after-preview01.py with new names; diff the two to audit it. Modes: none waits and
launches, --check reads only, --render is the guard's child.
"""
import importlib.util
import json
from pathlib import Path
import subprocess
import sys
from PIL import Image

O = Path(__file__).resolve().parent
SPEC = importlib.util.spec_from_file_location('complex_preview', O / 'preview_optimized01.py')
P = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(P)
T = P.T
PREVIEW = P.JOB
JOB = 'complex-stage06-optimized-thumbnail01'
MANIFEST = P.S.STAGE / 'optimized-thumbnail-inputs01.json'
OUT = T.B / JOB / 'output'


def render():
    receipt = T.read(T.B / JOB / 'resource-receipt.json')
    assert receipt.get('admitted') is True and 'exit' not in receipt
    assert receipt['command'] == [sys.executable, '-I', '-B', str(Path(__file__).resolve()), '--render']
    assert receipt['policy'] == T.POLICY, 'Guard policy changed'
    assert T.success(PREVIEW) and T.success(T.BUILD_JOB)
    for row in T.read(MANIFEST)['files']:
        assert T.sha(Path(row['path'])) == row['sha256'], row['path']
    assert not OUT.exists(), 'Preserve existing thumbnail output'
    subprocess.run([str(T.BIN), 'thumbnail', '--res', '3840x2160', '--out', str(OUT)],
                   cwd=T.R, check=True, timeout=600)
    path = OUT / 'complex_analysis_visual_thumbnail.png'
    with Image.open(path) as im:
        assert im.format == 'PNG' and im.size == (3840, 2160)
        im.verify()
    with Image.open(path) as im:
        im.load()
    with (OUT / 'thumbnail-receipt.json').open('x') as handle:
        json.dump({'path': str(path), 'sha256': T.sha(path), 'size': [3840, 2160],
                   'decode': 'passed', 'visual_review': 'pending', 'publication': 'not uploaded',
                   'scope': 'Native authored thumbnail only; no final AV or listening acceptance.'}, handle, indent=2)


def wait_and_launch():
    P.S.wait_for(P.S.TIMELINE_EXIT, P.S.EXIT, P.EXIT)
    assert T.success(PREVIEW) and T.success(T.BUILD_JOB)
    assert not MANIFEST.exists() and not (T.B / JOB).exists(), 'Duplicate thumbnail refused'
    files = {}
    for row in T.read(T.MANIFEST)['files']:
        assert T.sha(Path(row['path'])) == row['sha256'], row['path']
        files[row['path']] = row['sha256']
    for path in [T.MANIFEST, Path(__file__).resolve(), O / 'start-optimized-thumbnail01.sh',
                 T.B / PREVIEW / 'resource-receipt.json', T.B / PREVIEW / 'preview-summary.json',
                 T.B / T.BUILD_JOB / 'resource-receipt.json']:
        files[str(path)] = T.sha(path)
    with MANIFEST.open('x') as handle:
        json.dump({'files': [{'path': p, 'sha256': h} for p, h in sorted(files.items())]}, handle, indent=2)
    T.guard_policy()
    guard = T.GUARD
    subprocess.run([sys.executable, '-I', '-B', str(guard), '--phase', 'native', '--job', JOB,
                    '--inputs', str(MANIFEST), '--', sys.executable, '-I', '-B',
                    str(Path(__file__).resolve()), '--render'], cwd=T.R, check=True)
    assert T.success(JOB)


def preflight():
    report = P.preflight()
    assert not (T.B / JOB).exists() and not MANIFEST.exists(), 'New names already used'
    return report


if __name__ == '__main__':
    assert sys.argv[1:] in ([], ['--render'], ['--check'])
    if sys.argv[1:] == ['--check']:
        print(json.dumps({'job': JOB, **preflight(), 'launched': False}))
    else:
        render() if sys.argv[1:] else wait_and_launch()
