"""Render the authored native thumbnail after the optimized preview; do not publish.

thumbnail_optimized01.py with new names and one repair. Its manifest froze the recovery's rows but
not the preview and score helpers it imports. This one starts from the preview manifest, as pictures
does, lists all four imported helpers by name, and refuses to import any of them if its bytes changed.
Diff the two to audit it. Modes: none waits and launches, --check reads only and hashes only the
four helpers, --render is the guard's child.
"""
import hashlib
import importlib.util
import json
from pathlib import Path
import subprocess
import sys
from PIL import Image

O = Path(__file__).resolve().parent
SEALED = O.parent / 'complex-optimized-downstream01'
PREVIEW_HELPER = SEALED / 'preview_optimized01.py'
IMPORTED_SHA256 = {
    PREVIEW_HELPER: 'b823a34c2cab7f6cd6d8e2b55e26871aa25b6caa3041ff43a80cb27d32166c19',
    SEALED / 'score_optimized01.py': '22225d74f7a4362872fd62149ab9ad217249680a38829ff9d628a997adda5fb8',
    O.parent / 'complex-optimized-recovery-review01/optimized_timeline_recovery01.py':
        '4560e0dcd5534e0b8b3eaf205622f7cc36657c109344107d976c7b852dc086d7',
    O.parents[4] / 'narration_lab/third-wave-20261024/root/elevenlabs-complex-rebuild-20261104/native-stage06-root03/timeline-after-audio04.py':
        'e3ab6203a565b020d988faa6008f926abc5c08aa2840c915b759f32e831051ae',
}
for imported, digest in IMPORTED_SHA256.items():
    with imported.open('rb') as handle:
        assert hashlib.file_digest(handle, 'sha256').hexdigest() == digest, 'Imported helper changed: ' + imported.name
SPEC = importlib.util.spec_from_file_location('complex_preview', PREVIEW_HELPER)
P = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(P)
T = P.T
PREVIEW = P.JOB
JOB = 'complex-stage06-optimized-thumbnail02'
MANIFEST = T.E / 'optimized-thumbnail02/optimized-thumbnail02-inputs01.json'
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
    for row in T.read(P.MANIFEST)['files']:
        assert T.sha(Path(row['path'])) == row['sha256'], row['path']
        files[row['path']] = row['sha256']
    for path in [P.MANIFEST, Path(__file__).resolve(), O / 'start-optimized-thumbnail02.sh', *IMPORTED_SHA256,
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
    """The pins already held at import. This hashes no frozen input and samples no disk."""
    assert not (T.B / JOB).exists() and not MANIFEST.exists(), 'New names already used'
    return {'pinned_helpers': len(IMPORTED_SHA256)}


if __name__ == '__main__':
    assert sys.argv[1:] in ([], ['--render'], ['--check'])
    if sys.argv[1:] == ['--check']:
        print(json.dumps({'job': JOB, **preflight(), 'launched': False}))
    else:
        render() if sys.argv[1:] else wait_and_launch()
