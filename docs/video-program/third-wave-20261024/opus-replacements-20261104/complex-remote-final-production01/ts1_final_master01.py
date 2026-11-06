"""Master all reviewed 4K chapters on ts1 and export native captions; no implicit launch.

WhiteSwan's final_master_optimized01.py on lane O; diff the two to audit it. The native master still
replays every chapter's cue log and proves it equals the captured soundtrack, bit for bit.

    --check                     read-only: every chapter and hash, and the growth this job will need
    --launch [--retry master=rN]    one guarded job; a failed attempt moves aside only when named
    --run JOB                   the guard's child; never call it by hand
"""
import hashlib
import importlib.util
import json
from pathlib import Path
import subprocess
import sys

HERE = Path(__file__).resolve().parent
RENDER_HELPER = HERE / 'ts1_final_render01.py'
RENDER_SHA256 = '23d99bfb8a5cfc30fed702cae186180eacdae4ea482c5666e282fab08bd7fc70'
with RENDER_HELPER.open('rb') as handle:
    assert hashlib.file_digest(handle, 'sha256').hexdigest() == RENDER_SHA256, 'Render helper changed'
SPEC = importlib.util.spec_from_file_location('complex_final_render', RENDER_HELPER)
F = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(F)
L, M, Q = F.L, F.M, F.Q
R, JOBS, BIN, read, sha = F.R, F.JOBS, F.BIN, F.read, F.sha
ME = R / Path(__file__).resolve().relative_to(Q.PHYSICAL)
START = ME.parent / 'start-final-master01.sh'
JOB = 'complex-remote-master01'
SUCCESS = 'MASTER WRITTEN: candidate master and captions; strict verify, pictures, ASR, loudness and listening remain.'
OUTPUTS = {key: F.OUT / ('complex_analysis_visual' + suffix)
           for key, suffix in [('master', '_master.mp4'), ('srt', '.en.srt'), ('vtt', '.en.vtt')]}


def job():
    return F.newest(JOB)


def manifest(name):
    return F.STAGE / (name + '-inputs.json')


def child_tail(name):
    return ['-I', '-B', str(ME), '--run', name]


def check_chapters():
    F.check_review()
    timeline_summary = read(M.job('timeline') / 'timeline-summary.json')
    assert [row['chapter'] for row in timeline_summary['chapters']] == list(M.CHAPTERS)
    by_chapter = {row['chapter']: row for row in timeline_summary['chapters']}
    captures = {}
    bindings = {}
    def bind(row):
        previous = bindings.setdefault(row['path'], row['sha256'])
        assert previous == row['sha256'], 'Conflicting frozen dependency: ' + row['path']
    for chapter in M.CHAPTERS:
        name = F.job(chapter)
        assert F.passed(name), f'Incomplete final chapter: {chapter}'
        assert read(JOBS / name / 'resource-receipt.json')['command'][1:] == F.child_tail(chapter, name), name
        for row in read(F.manifest(name))['files']:
            bind(row)
        summary = read(JOBS / name / 'render-summary.json')
        assert summary['chapter'] == chapter and summary['fps'] == 60 and summary['binary_sha256'] == Q.BIN_SHA256
        assert summary['frames'] == by_chapter[chapter]['frames']
        F.same_source(summary)
        for row in summary['files']:
            bind(row)
        capture = F.captured_soundtrack(F.OUT, chapter, summary['frames'])
        assert capture == summary['soundtrack'], f'Captured soundtrack changed: {chapter}'
        captures[chapter] = capture
    for path, digest in bindings.items():
        assert sha(Path(path)) == digest, path
    return captures, timeline_summary['total_frames']


def growth():
    """Logarithm's real master grew 1.02 GiB; this estimate gives 1.07 there. It decides nothing."""
    video = sum((F.OUT / (chapter + '.mp4')).stat().st_size for chapter in M.CHAPTERS)
    audio = sum(Path(F.captured_soundtrack(F.OUT, chapter, read(JOBS / F.job(chapter) / 'render-summary.json')['frames'])['path']).stat().st_size
                for chapter in M.CHAPTERS)
    estimate = (2 * video + 2 * audio) / 1024 ** 3
    cap = Q.POLICY['phase_growth_gib']
    return {'chapter_video_gib': round(video / 1024 ** 3, 3), 'soundtrack_gib': round(audio / 1024 ** 3, 3),
            'estimated_growth_gib': round(estimate, 3), 'guard_growth_cap_gib': cap, 'over_cap': estimate > cap}


def run(name):
    assert name == job(), 'This is not the newest attempt: ' + name
    F.gate(name, [sys.executable, *child_tail(name)], manifest(name))
    captures, frames = check_chapters()
    assert not any(p.exists() for p in OUTPUTS.values()), 'Preserve existing master/captions'
    native_log = JOBS / name / 'master-native.log'
    with native_log.open('x') as log:
        subprocess.run([str(BIN), 'master', '--narration', str(M.N), '--score',
                        str(M.SCORE_OUT / 'stems'), '--out', str(F.OUT)],
                       cwd=R, stdout=log, stderr=subprocess.STDOUT, check=True, timeout=14400)
    replay_lines = [line for line in native_log.read_text().splitlines()
                    if "replay = fmn's soundtrack, bit for bit" in line]
    assert [line.split()[0] for line in replay_lines] == list(M.CHAPTERS), 'Missing native soundtrack replay proof'
    with (JOBS / name / 'captions-native.log').open('x') as log:
        subprocess.run([str(BIN), 'captions', 'all', '--narration', str(M.N), '--out', str(F.OUT)],
                       cwd=R, stdout=log, stderr=subprocess.STDOUT, check=True, timeout=1200)
    after, after_frames = check_chapters()
    assert after == captures and after_frames == frames, 'Chapter changed during mastering'
    probe = json.loads(subprocess.check_output([str(M.tool('ffprobe')), '-v', 'error', '-show_streams',
                                               '-of', 'json', str(OUTPUTS['master'])], timeout=120))
    video = [s for s in probe['streams'] if s['codec_type'] == 'video']
    audio = [s for s in probe['streams'] if s['codec_type'] == 'audio']
    assert len(probe['streams']) == 2 and len(video) == len(audio) == 1
    v = video[0]
    assert (v['width'], v['height'], v['avg_frame_rate'], int(v['nb_frames'])) == (3840, 2160, '60/1', frames)
    assert OUTPUTS['srt'].stat().st_size and OUTPUTS['vtt'].read_text().startswith('WEBVTT')
    with (JOBS / name / 'master-summary.json').open('x') as handle:
        json.dump({'chapters': list(M.CHAPTERS), 'frames': frames, 'fps': 60, 'binary_sha256': Q.BIN_SHA256, 'host': 'ts1',
                   'timeline_job': M.NAMES['timeline'], 'narration': str(M.N), **F.narration_source(), 'full_baseline_equivalence_established': False,
                   'render_jobs': {chapter: F.job(chapter) for chapter in M.CHAPTERS},
                   'soundtracks': captures, 'native_bit_identical_replays': replay_lines,
                   'outputs': {k: {'path': str(p), 'sha256': sha(p)} for k, p in OUTPUTS.items()},
                   'scope': 'Candidate master and captions only. Full decode, encoded loudness, exact caption comparison, final pictures, ASR, listening, fine sync and independent AV review remain open. No publication.'}, handle, indent=2)
    print(SUCCESS, flush=True)


def launch(retries):
    check_chapters()
    Q.guard_policy()
    name, last = F.next_attempt(JOB, 'master', retries.get('master'))
    assert name, 'Duplicate master refused'
    if last is not None:
        F.set_aside(last, [*OUTPUTS.values(), F.OUT / 'master'])
    assert not any(p.exists() for p in OUTPUTS.values()), 'Existing output preserved'
    print(json.dumps(growth()), flush=True)
    files = {}
    for chapter in M.CHAPTERS:
        for row in read(F.manifest(F.job(chapter)))['files']:
            if row['path'] not in files:
                assert sha(row['path']) == row['sha256'], row['path']
                files[row['path']] = row['sha256']
    paths = [ME, F.ME, START]
    for chapter in M.CHAPTERS:
        name_of = F.job(chapter)
        paths += [F.manifest(name_of), JOBS / name_of / 'resource-receipt.json', JOBS / name_of / 'render-summary.json']
    paths += sorted(p for p in F.OUT.rglob('*') if p.is_file())
    for path in paths:
        files[str(path)] = sha(path)
    F.write_once(manifest(name), files)
    F.guard_run(name, manifest(name), [sys.executable, *child_tail(name)])


if __name__ == '__main__':
    if sys.argv[1:] == ['--check']:
        check_chapters()
        print(json.dumps(growth()))
    elif sys.argv[1:2] == ['--launch']:
        launch(F.parse_retries(sys.argv[2:], ['master']))
    elif len(sys.argv) == 3 and sys.argv[1] == '--run':
        run(sys.argv[2])
    else:
        raise SystemExit('Use --check, --launch, or --run JOB. No implicit launch.')
