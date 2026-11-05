"""Make the full narrated native preview from the optimized recovery; viewing is a separate step.

Root's preview-after-score01.py with new names and tighter gates; diff the two to audit it.
Same modes as score_optimized01.py.
"""
import importlib.util
import json
from pathlib import Path
import shutil
import subprocess
import sys

O = Path(__file__).resolve().parent
SPEC = importlib.util.spec_from_file_location('complex_score', O / 'score_optimized01.py')
S = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(S)
T = S.T
JOB = 'complex-stage06-optimized-preview01'
EXIT = S.STAGE / 'optimized-preview-controller01.exit'
MANIFEST = S.STAGE / 'optimized-preview-inputs01.json'
OUT = T.B / JOB / 'preview'
PROBE = Path(shutil.which('ffprobe')).resolve()


def check_score():
    assert T.success(S.JOB)
    timeline = S.check_timeline()
    summary = S.read(T.B / S.JOB / 'score-summary.json')
    assert list(summary['qa_sha256']) == list(T.CHAPTERS)
    for chapter, digest in summary['qa_sha256'].items():
        path = S.OUT / 'stems/qa' / (chapter + '.json')
        report = S.read(path)
        assert S.sha(path) == digest and report['chapter'] == chapter and report['passed'] is True
        assert report['metrics'] and all(m['pass'] is True or m['pass'] is None for m in report['metrics'])
    return timeline


def normalized(path):
    return [line for line in path.read_text().splitlines() if line and not line.startswith('#')]


def score_files(chapter):
    """The files the kit lays for this chapter, in order: the whole stem, or its parts list."""
    stem = S.OUT / 'stems' / (chapter + '.music.f32.wav')
    parts = stem.with_suffix('.parts.tsv')
    if not parts.exists():
        return [stem]
    return [stem.with_name(line.split('\t')[4]) for line in parts.read_text().splitlines()[1:]]


def check_cue_log(chapter, frames, rendered, timeline):
    """A scored render logs the timeline's cues unchanged, then its own score at the last frame.
    Plain equality could never pass: Kit::finish logs the music, and a timeline has none."""
    base, rows = normalized(timeline), normalized(rendered)
    assert rows[:len(base)] == base, 'Preview cue log differs: ' + rendered.name
    laid = [row.split('\t') for row in rows[len(base):]]
    wanted = [(str(path), S.sha(path), str(frames), '0.0') for path in score_files(chapter)]
    assert [(row[0], row[1], row[3], row[7]) for row in laid] == wanted, 'Preview score rows differ: ' + rendered.name


def run_preview():
    receipt = S.read(T.B / JOB / 'resource-receipt.json')
    assert receipt.get('admitted') is True and 'exit' not in receipt
    assert receipt['command'] == [sys.executable, '-I', '-B', str(Path(__file__).resolve()), '--run']
    assert receipt['policy'] == T.POLICY, 'Guard policy changed'
    for row in S.read(MANIFEST)['files']:
        assert S.sha(Path(row['path'])) == row['sha256'], row['path']
    timeline = check_score()
    assert not OUT.exists(), 'Existing preview must be preserved'
    subprocess.run([str(T.BIN), 'render', 'all', '--res', '960x540', '--fps', '60',
                    '--narration', str(T.N), '--score', str(S.OUT / 'stems'), '--out', str(OUT),
                    '--lint', '--crf', '18', '--preset', 'veryfast'], cwd=T.R, check=True,
                   timeout=T.RENDER_TIMEOUT_SECONDS)
    chapters = []
    assert {p.stem for p in OUT.glob('*.mp4')} == set(T.CHAPTERS)
    for chapter in T.CHAPTERS:
        table = OUT / (chapter + '.timeline.tsv')
        for name in ('timeline', 'cues'):
            rendered = OUT / f'{chapter}.{name}.tsv'
            assert normalized(rendered) == normalized(timeline / rendered.name), 'Preview table differs: ' + rendered.name
        movie = OUT / (chapter + '.mp4')
        probe = json.loads(subprocess.check_output([str(PROBE), '-v', 'error', '-show_streams',
                                                   '-of', 'json', str(movie)], timeout=120))
        video = [s for s in probe['streams'] if s['codec_type'] == 'video']
        audio = [s for s in probe['streams'] if s['codec_type'] == 'audio']
        assert len(video) == len(audio) == 1
        v = video[0]
        assert (v['width'], v['height'], v['avg_frame_rate']) == (960, 540, '60/1')
        frames = int(next(line.split('\t')[1] for line in normalized(table) if line.startswith('frames\t')))
        assert int(v['nb_frames']) == frames
        check_cue_log(chapter, frames, OUT / (chapter + '.cuelog.tsv'), timeline / (chapter + '.cuelog.tsv'))
        layout = OUT / (chapter + '.layout.tsv')
        assert layout.is_file(), 'Rendered lint report missing: ' + layout.name
        chapters.append({'chapter': chapter, 'frames': frames, 'mp4_sha256': S.sha(movie),
                         'timeline_sha256': S.sha(table), 'layout_sha256': S.sha(layout),
                         'layout_findings': len([line for line in layout.read_text().splitlines() if line.strip()])})
    with (T.B / JOB / 'preview-summary.json').open('x') as handle:
        json.dump({'chapters': chapters, 'res': '960x540', 'fps': 60, 'voice': 'ElevenLabs Frederick Surrey',
                   'binary_sha256': T.BIN_SHA256, 'timeline_job': T.JOB, 'tables_equal_to_timeline': ['timeline', 'cues'],
                   'cue_log': 'all timeline rows unchanged, then only this chapter score rows',
                   'rendered_lint': True, 'full_baseline_equivalence_established': False,
                   'scope': 'Narrated preview rendered and schedules compared; actual picture review, listening, fine sync and independent AV review remain open. Not a publication master.'}, handle, indent=2)


def wait_and_launch():
    S.wait_for(S.TIMELINE_EXIT, S.EXIT)
    check_score()
    assert not (T.B / JOB).exists() and not MANIFEST.exists(), 'Duplicate preview refused'
    files = {}
    for row in S.read(S.MANIFEST)['files']:
        assert S.sha(Path(row['path'])) == row['sha256'], row['path']
        files[row['path']] = row['sha256']
    paths = [S.MANIFEST, Path(__file__).resolve(), O / 'start-optimized-preview01.sh', PROBE]
    paths.extend(p for p in (T.B / S.JOB).rglob('*') if p.is_file())
    for path in paths:
        files[str(path)] = S.sha(path)
    with MANIFEST.open('x') as handle:
        json.dump({'files': [{'path': p, 'sha256': h} for p, h in sorted(files.items())]}, handle, indent=2)
    T.guard_policy()
    guard = T.GUARD
    subprocess.run([sys.executable, '-I', '-B', str(guard), '--phase', 'native', '--job', JOB,
                    '--inputs', str(MANIFEST), '--', sys.executable, '-I', '-B',
                    str(Path(__file__).resolve()), '--run'], cwd=T.R, check=True)
    assert T.success(JOB)


def preflight():
    report = S.preflight()
    assert not (T.B / JOB).exists() and not MANIFEST.exists(), 'New names already used'
    return report


if __name__ == '__main__':
    assert sys.argv[1:] in ([], ['--run'], ['--check'])
    if sys.argv[1:] == ['--check']:
        print(json.dumps({'job': JOB, **preflight(), 'launched': False}))
    else:
        run_preview() if sys.argv[1:] else wait_and_launch()
