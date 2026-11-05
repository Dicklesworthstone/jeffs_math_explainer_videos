"""Master all reviewed 4K chapters and export native captions; no implicit launch.

Root's final-master01.py with new names; diff the two to audit it. The native master still replays
every chapter's cue log and proves it equals the captured soundtrack, bit for bit.
"""
import hashlib
import importlib.util
import json
from pathlib import Path
import subprocess
import sys

O = Path(__file__).resolve().parent
RENDER_HELPER = O / 'final_render_optimized01.py'
RENDER_SHA256 = '92a8875da6257833d9e82538e0062ec2a4d1ceea8aebac7a4da25761b4eb6b3e'
with RENDER_HELPER.open('rb') as handle:
    assert hashlib.file_digest(handle, 'sha256').hexdigest() == RENDER_SHA256, 'Render helper changed'
SPEC = importlib.util.spec_from_file_location('complex_final_render', RENDER_HELPER)
F = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(F)
T = F.T
JOB = 'complex-stage06-optimized-master01'
MANIFEST = F.STAGE / 'optimized-master-inputs01.json'
OUTPUTS = {key: F.OUT / ('complex_analysis_visual' + suffix)
           for key, suffix in [('master', '_master.mp4'), ('srt', '.en.srt'), ('vtt', '.en.vtt')]}


def check_chapters():
    F.check_review()
    timeline_summary = T.read(T.B / T.JOB / 'timeline-summary.json')
    assert [row['chapter'] for row in timeline_summary['chapters']] == list(T.CHAPTERS)
    by_chapter = {row['chapter']: row for row in timeline_summary['chapters']}
    captures = {}
    bindings = {}
    def bind(row):
        previous = bindings.setdefault(row['path'], row['sha256'])
        assert previous == row['sha256'], 'Conflicting frozen dependency: ' + row['path']
    for chapter in T.CHAPTERS:
        assert T.success(F.job(chapter)), f'Incomplete final chapter: {chapter}'
        for row in T.read(F.manifest(chapter))['files']:
            bind(row)
        summary = T.read(T.B / F.job(chapter) / 'render-summary.json')
        assert summary['chapter'] == chapter and summary['fps'] == 60
        assert summary['frames'] == by_chapter[chapter]['frames']
        for row in summary['files']:
            bind(row)
        capture = F.captured_soundtrack(F.OUT, chapter, summary['frames'])
        assert capture == summary['soundtrack'], f'Captured soundtrack changed: {chapter}'
        captures[chapter] = capture
    for path, digest in bindings.items():
        assert T.sha(Path(path)) == digest, path
    return captures, timeline_summary['total_frames']


def run():
    receipt = T.read(T.B / JOB / 'resource-receipt.json')
    assert receipt.get('admitted') is True and 'exit' not in receipt
    assert receipt['command'] == [sys.executable, '-I', '-B', str(Path(__file__).resolve()), '--run']
    assert receipt['policy'] == T.POLICY, 'Guard policy changed'
    for row in T.read(MANIFEST)['files']:
        assert T.sha(Path(row['path'])) == row['sha256'], row['path']
    captures, frames = check_chapters()
    assert not any(p.exists() for p in OUTPUTS.values()), 'Preserve existing master/captions'
    native_log = T.B / JOB / 'master-native.log'
    with native_log.open('x') as log:
        subprocess.run([str(T.BIN), 'master', '--narration', str(T.N), '--score',
                        str(F.P.S.OUT / 'stems'), '--out', str(F.OUT)],
                       cwd=T.R, stdout=log, stderr=subprocess.STDOUT, check=True, timeout=14400)
    replay_lines = [line for line in native_log.read_text().splitlines()
                    if "replay = fmn's soundtrack, bit for bit" in line]
    assert [line.split()[0] for line in replay_lines] == list(T.CHAPTERS), 'Missing native soundtrack replay proof'
    with (T.B / JOB / 'captions-native.log').open('x') as log:
        subprocess.run([str(T.BIN), 'captions', 'all', '--narration', str(T.N), '--out', str(F.OUT)],
                       cwd=T.R, stdout=log, stderr=subprocess.STDOUT, check=True, timeout=1200)
    after, after_frames = check_chapters()
    assert after == captures and after_frames == frames, 'Chapter changed during mastering'
    probe = json.loads(subprocess.check_output([str(F.P.PROBE), '-v', 'error', '-show_streams',
                                               '-of', 'json', str(OUTPUTS['master'])], timeout=120))
    video = [s for s in probe['streams'] if s['codec_type'] == 'video']
    audio = [s for s in probe['streams'] if s['codec_type'] == 'audio']
    assert len(probe['streams']) == 2 and len(video) == len(audio) == 1
    v = video[0]
    assert (v['width'], v['height'], v['avg_frame_rate'], int(v['nb_frames'])) == (3840, 2160, '60/1', frames)
    assert OUTPUTS['srt'].stat().st_size and OUTPUTS['vtt'].read_text().startswith('WEBVTT')
    with (T.B / JOB / 'master-summary.json').open('x') as handle:
        json.dump({'chapters': list(T.CHAPTERS), 'frames': frames, 'fps': 60, 'binary_sha256': T.BIN_SHA256,
                   'timeline_job': T.JOB, 'full_baseline_equivalence_established': False,
                   'soundtracks': captures, 'native_bit_identical_replays': replay_lines,
                   'outputs': {k: {'path': str(p), 'sha256': T.sha(p)} for k, p in OUTPUTS.items()},
                   'scope': 'Candidate master and captions only. Full decode, encoded loudness, exact caption comparison, final pictures, ASR, listening, fine sync and independent AV review remain open. No publication.'}, handle, indent=2)


def launch():
    check_chapters()
    assert not (T.B / JOB).exists() and not MANIFEST.exists(), 'Duplicate master refused'
    assert not any(p.exists() for p in OUTPUTS.values()), 'Existing output preserved'
    T.guard_policy()
    files = {}
    for chapter in T.CHAPTERS:
        for row in T.read(F.manifest(chapter))['files']:
            assert T.sha(Path(row['path'])) == row['sha256'], row['path']
            files[row['path']] = row['sha256']
    paths = [Path(__file__).resolve(), RENDER_HELPER, O / 'start-optimized-final-master01.sh']
    for chapter in T.CHAPTERS:
        paths += [F.manifest(chapter), T.B / F.job(chapter) / 'resource-receipt.json',
                  T.B / F.job(chapter) / 'render-summary.json']
    paths += sorted(p for p in F.OUT.rglob('*') if p.is_file())
    for path in paths:
        files[str(path)] = T.sha(path)
    with MANIFEST.open('x') as handle:
        json.dump({'files': [{'path': p, 'sha256': h} for p, h in sorted(files.items())]}, handle, indent=2)
    subprocess.run([sys.executable, '-I', '-B', str(F.GUARD), '--phase', 'native', '--job', JOB,
                    '--inputs', str(MANIFEST), '--', sys.executable, '-I', '-B', str(Path(__file__).resolve()), '--run'],
                   cwd=T.R, check=True)
    assert T.success(JOB)


if __name__ == '__main__':
    if sys.argv[1:] == ['--check']:
        check_chapters()
    elif sys.argv[1:] == ['--launch']:
        launch()
    elif sys.argv[1:] == ['--run']:
        run()
    else:
        raise SystemExit('Use --check, --launch, or --run. No implicit launch.')
