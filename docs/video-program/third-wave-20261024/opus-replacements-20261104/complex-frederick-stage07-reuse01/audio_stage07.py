"""Stage07 narration on macOS: decode, prepare, assemble, timings, install and rows, one guarded job each, in order.

These are root's stage06 scripts (decode01.sh, prepare-after-decode01.py, install-after-timings01.py) in one file, with
the stage07 tools and names. It selects nothing, synthesizes nothing and hears nothing.

    --check     read-only: every input present, every name unused; launches nothing
    --launch    a guard preflight on a spare name, then the six jobs in order; a failed job stops the rest
    --install   the install job's child; never call it by hand
"""
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import sys

ME = Path(__file__).resolve()
R = ME.parents[5]
EC = 'narration_lab/third-wave-20261024/root/elevenlabs-complex-rebuild-20261104'
E = R / EC
B = R / 'narration_lab/third-wave-20261024/root/five-film-release-20261027'
O = E / 'production-audio-stage07-01'
G = R / 'docs/video-program/third-wave-20261024/opus-replacements-20261104/engine-secondary-validation01/guard_owner56g_target20g.py'
H = ME.parent / 'frederick_timing_bridge_stage07.py'
INTAKE = ME.parent / 'frederick_intake_stage07.py'
BASE = E / 'production-audio-stage06-01/decode-inputs01.json'
NATIVE = E / 'native-stage07-root01'
W = NATIVE / 'workspace'
C = W / 'explainer_corpus/complex_analysis_visual'
LANE = E / 'recognition-stage07-root01'
TAKES = E / 'remote-ts1-01/cauchy-cue-repair01'
DECISIONS = E / 'stage07-timing-decisions01'
SELECTION, ANSWERS = DECISIONS / 'selection.json', DECISIONS / 'answers.json'
ROWS = E / 'remote-ts1-01/stage07-rows01.json'
REVIEWER = 'BlueSalmon provisional ASR review; unlistened'
JOBS = {stage: f'complex-stage07-audio-{stage}01' for stage in ('decode', 'prepare', 'assemble', 'timings', 'rows')}
JOBS['install'] = 'complex-stage07-native-audio-install01'
DECODED, PREPARED = O / 'decoded01', O / 'prepared01'
NARRATION, TIMINGS = O / 'prepared01-narration', O / 'prepared01-timings'
OUT = O / 'native-narration01'


def digest(path):
    with Path(path).open('rb') as handle:
        return hashlib.file_digest(handle, 'sha256').hexdigest()


def read(path):
    return json.loads(Path(path).read_text())


def successful(stage):
    data = read(B / JOBS[stage] / 'resource-receipt.json')
    assert data.get('finished') and data.get('admitted') is True and data.get('exit') == 0, stage
    assert data.get('inputs_sha256') == data.get('final_inputs_sha256'), stage
    assert not data.get('stopped_for_resource_limit') and 'error' not in data, stage


def trees(*folders):
    return [path for folder in folders for path in Path(folder).rglob('*') if path.is_file()]


def base_files():
    files = {row['path']: row['sha256'] for row in read(BASE)['files']}
    for path, value in files.items():
        assert digest(path) == value, path
    for path in [ME, H, INTAKE, SELECTION, ANSWERS, NATIVE / 'tree-receipt.json', *trees(W, LANE, TAKES)]:
        files[str(path)] = digest(path)
    return files


def run(stage, argv, base, *folders):
    job = JOBS[stage]
    assert not (B / job).exists(), f'Existing job: {job}'
    files = dict(base)
    done = [B / JOBS[name] / 'resource-receipt.json' for name in JOBS if (B / JOBS[name]).exists()]
    for path in [*done, *trees(*folders)]:
        files[str(path)] = digest(path)
    manifest = O / f'{job}-inputs.json'
    with manifest.open('x') as handle:
        json.dump({'files': [{'path': path, 'sha256': value} for path, value in sorted(files.items())]}, handle, indent=2)
    print('Launching', job, 'bindings', len(files), flush=True)
    subprocess.run([sys.executable, '-I', '-B', str(G), '--phase', 'native', '--job', job, '--inputs', str(manifest), '--', *argv],
                   cwd=R, check=True)
    successful(stage)


def install():
    started = read(B / JOBS['install'] / 'resource-receipt.json')
    assert started.get('admitted') is True and 'exit' not in started
    assert started.get('command') == [sys.executable, '-I', '-B', str(ME), '--install']
    for stage in ('decode', 'prepare', 'assemble', 'timings'):
        successful(stage)
    assert not OUT.exists(), 'Existing handoff retained; refusing overwrite'
    receipt = read(TIMINGS / 'timings-receipt.json')
    cues = [cue for unit in read(C / 'narration_units.json')['units'] for cue in unit['line_ids']]
    assert len(cues) == len(set(cues)) == 105
    assert receipt['cues'] == len(cues) and receipt['file'] == 'phrase-timings.json'
    assert receipt['assembly_sha256'] == digest(NARRATION / 'assembly.json')
    assert receipt['report_sha256'] == digest(NARRATION / 'report.json')
    phrases = read(TIMINGS / 'phrase-timings.json')
    assert set(phrases['cues']) == set(cues)
    assert phrases['script_sha256'] == digest(C / 'script.tsv')
    assert phrases['prepare_receipt_sha256'] == digest(PREPARED / 'prepare-receipt.json')
    for cue in cues:
        assert phrases['cues'][cue]['wav_sha256'] == digest(NARRATION / (cue + '.wav')), cue
    assert not (NARRATION / 'phrase-timings.json').exists() and not (NARRATION / 'external-units.json').exists()
    OUT.mkdir()
    copied = []
    pairs = [(path, OUT / path.relative_to(NARRATION)) for path in sorted(trees(NARRATION))]
    pairs += [(TIMINGS / 'phrase-timings.json', OUT / 'phrase-timings.json'), (C / 'narration_units.json', OUT / 'external-units.json')]
    for path, target in pairs:
        target.parent.mkdir(parents=True, exist_ok=True)
        with target.open('xb') as handle:
            handle.write(path.read_bytes())
        assert digest(target) == digest(path)
        copied.append({'path': str(target), 'sha256': digest(target)})
    subprocess.run([sys.executable, '-I', '-B', str(W / 'tools/coherent_narration.py'), 'verify', str(OUT), '--script', str(C / 'script.tsv'),
                    '--units', str(OUT / 'external-units.json'), '--require-directions'], cwd=R, check=True)
    with (OUT / 'native-install-receipt.json').open('x') as handle:
        json.dump({'cues': len(cues), 'files': copied, 'prerequisites': [JOBS[s] for s in ('decode', 'prepare', 'assemble', 'timings')],
                   'scope': 'Byte-identical copied narration, phrase marks and external unit plan; direction/sample verification.',
                   'listening_acceptance': False, 'fine_sync_acceptance': False, 'native_render_acceptance': False}, handle, indent=2)
    print('105-cue stage07 audio handoff installed; listening and final AV review remain open.', flush=True)


def check():
    assert shutil.which('ffmpeg'), 'No ffmpeg on PATH'
    for path in (H, INTAKE, BASE, G, SELECTION, ANSWERS, NATIVE / 'tree-receipt.json', C / 'script.tsv'):
        assert path.is_file(), f'Missing {path}'
    assert list(LANE.glob('ingest-[0-9][0-9]/ingest.json')), 'The stage07 lane has no ingest'
    assert not O.exists() and not ROWS.exists(), 'Output folder or rows already exist'
    assert not any((B / job).exists() for job in JOBS.values()), 'A job name is used'
    print(json.dumps({'jobs': list(JOBS.values()), 'output': str(O), 'rows': str(ROWS), 'launched': False}))


def launch():
    check()
    number = next(n for n in range(1, 100) if not (B / f'complex-stage07-audio-preflight{n:02d}').exists())
    subprocess.run([sys.executable, '-I', '-B', str(G), '--phase', 'native', '--job', f'complex-stage07-audio-preflight{number:02d}',
                    '--preflight-only'], cwd=R, check=True)
    base = base_files()
    O.mkdir()
    bridge = [sys.executable, '-I', '-B', str(H)]
    common = ['--crate', str(C), '--lane', str(LANE), '--jobs', str(B)]
    run('decode', [*bridge, 'decode', *common, '--selection', str(SELECTION), '--out', str(DECODED)], base)
    run('prepare', [*bridge, 'prepare', *common, '--selection', str(SELECTION), '--answers', str(ANSWERS), '--decoded', str(DECODED),
                    '--reviewer', REVIEWER, '--out', str(PREPARED)], base, DECODED)
    commands = read(PREPARED / 'assemble-command.json')
    assert commands['argv'][4] == 'assemble'
    run('assemble', commands['argv'], base, DECODED, PREPARED)
    run('timings', [*commands['then'], '--crate', str(C)], base, DECODED, PREPARED, NARRATION)
    run('install', [sys.executable, '-I', '-B', str(ME), '--install'], base, DECODED, PREPARED, NARRATION, TIMINGS)
    run('rows', [*bridge, 'rows', '--crate', str(C), '--narration', str(OUT), '--tree', str(NATIVE), '--out', str(ROWS)],
        base, DECODED, PREPARED, NARRATION, TIMINGS, OUT)
    print('Stage07 audio complete. Listening, fine sync and native final AV review remain open.', flush=True)


if __name__ == '__main__':
    assert sys.argv[1:] in (['--check'], ['--launch'], ['--install']), __doc__
    {'--check': check, '--launch': launch, '--install': install}[sys.argv[1]]()
