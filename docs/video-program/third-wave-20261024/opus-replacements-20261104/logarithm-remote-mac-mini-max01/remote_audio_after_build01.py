"""Guarded rebind and retained-audio assembly on mac-mini-max, after root's successful remote build (bead 87do.63).

Root's S/audio-after-build01.py for the remote lane. What differs:
- it waits for O/jobs/logarithm-remote-build01 and launches through O/guard01.py;
- the freeze starts from O/build-inputs01.json, which has no old target rows;
- that manifest binds narration_units.json, which the rebind must rewrite, so this one row is dropped before the rebind
  freeze and added back, as rewritten, for the assembly;
- the rebind helper's hash is pinned, and the assembly is this bundle's remote_assemble01.py.
The controller, remote_continuation01.sh, runs this once. Exit 0 only after both jobs pass.
"""
import hashlib
import json
from pathlib import Path
import subprocess
import sys
import time

HERE = Path(__file__).resolve()
BUNDLE = HERE.parent
R = HERE.parents[5]
EL = R / 'narration_lab/third-wave-20261024/root/elevenlabs-logarithm-rebuild-20261104'
S = EL / 'final-picture-repair01'
O = EL / 'remote-mac-mini-max01'
JOBS = O / 'jobs'
H = R / 'docs/video-program/third-wave-20261024/opus-replacements-20261104/logarithm-final-audio-rebind01/rebind_audio.py'
H_SHA = '072e74a5f5202d0dd3cb0f2662237d7d0cf14a64f0c971b0c66c54b34dac238e'
GUARD = O / 'guard01.py'
ASSEMBLE = BUNDLE / 'remote_assemble01.py'
BUILD, REBIND, ASSEMBLY = 'logarithm-remote-build01', 'logarithm-remote-rebind01', 'logarithm-remote-assemble01'
CRATE = S / 'workspace/explainer_corpus/logarithm_mysteries'
UNITS = CRATE / 'narration_units.json'


def read(p):
    return json.loads(p.read_text())


def sha(p):
    with p.open('rb') as f:
        return hashlib.file_digest(f, 'sha256').hexdigest()


def success(job):
    p = JOBS / job / 'resource-receipt.json'
    if not p.exists():
        return False
    try:
        d = read(p)
    except json.JSONDecodeError:
        return False
    if not d.get('finished'):
        return False
    assert d.get('admitted') is True and d.get('exit') == 0, job
    assert not d.get('error') and not d.get('stopped_for_resource_limit'), job
    assert d['inputs_sha256'] == d['final_inputs_sha256'], job
    return True


def inputs():
    rows = {x['path']: x['sha256'] for x in read(O / 'build-inputs01.json')['files']}
    assert sha(UNITS) == rows.pop(str(UNITS)), 'The unit plan is not the one the build froze'
    assert not [p for p in rows if '/target/' in p], 'Old target paths do not belong in this freeze'
    paths = [H, HERE, BUNDLE / 'SHA256SUMS', ASSEMBLE, S / 'initial-copy.json',
             EL / 'combined-repair-stage09/bound-plan/hold-audit.json']
    initial = read(S / 'initial-copy.json')
    original = Path(initial['source'])
    paths += [original / x['path'] for x in initial['files']]
    prepared = EL / 'production-audio-stage09-01/prepared02'
    paths += [p for p in prepared.rglob('*') if p.is_file()]
    for unit in read(prepared / 'spans.json')['units']:
        paths += [R / unit['wav'], R / unit['generation_receipt']]
    paths += [S / 'workspace' / x['path'] for x in initial['files']
              if not x['path'].endswith('/narration_units.json')]
    for p in paths:
        digest = sha(p)
        assert str(p) not in rows or rows[str(p)] == digest, p
        rows[str(p)] = digest
    for p, digest in rows.items():
        assert sha(Path(p)) == digest, p
    return rows


def launch(job, manifest, rows, command):
    assert not (JOBS / job).exists() and not manifest.exists(), 'Duplicate job refused'
    with manifest.open('x') as f:
        json.dump({'files': [{'path': p, 'sha256': h} for p, h in sorted(rows.items())]}, f, indent=2)
    subprocess.run([sys.executable, '-I', '-B', str(GUARD), '--phase', 'native', '--job', job,
                    '--inputs', str(manifest), '--', *command], cwd=R, check=True)
    assert success(job), job


def main():
    assert sha(H) == H_SHA, 'rebind_audio.py is not the sealed helper'
    for _ in range(8640):
        if success(BUILD):
            break
        time.sleep(20)
    else:
        raise RuntimeError('Build wait expired; no audio jobs launched')
    rows = inputs()
    assert str(UNITS) not in rows, 'Rebind output cannot be a frozen input'
    launch(REBIND, O / 'rebind-inputs01.json', rows, [sys.executable, '-I', '-B', str(H), '--apply'])
    subprocess.run([sys.executable, '-I', '-B', str(H), '--verify'], cwd=R, check=True,
                   stdout=subprocess.DEVNULL, timeout=900)
    for p in [S / 'spans01.json', S / 'holds-bound01.json', UNITS,
              CRATE / 'docs/narration-direction-map-oliver17.json', JOBS / REBIND / 'resource-receipt.json']:
        rows[str(p)] = sha(p)
    launch(ASSEMBLY, O / 'assemble-inputs01.json', rows, [sys.executable, '-I', '-B', str(ASSEMBLE)])


if __name__ == '__main__':
    main()
