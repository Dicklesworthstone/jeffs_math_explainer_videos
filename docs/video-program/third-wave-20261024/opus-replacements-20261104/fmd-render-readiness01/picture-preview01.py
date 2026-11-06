"""Prepared csd narrated-preview guard child; not a publication master.
Run only after terminal timeline comparison and score QA, with all inputs frozen.
No import side effects or automatic launcher. Actual picture review remains separate.
"""
import json
import os
import shutil
import struct
import subprocess
import sys
import importlib.util
from pathlib import Path

# Guard invokes python -I; load only the frozen adjacent helper explicitly.
_spec = importlib.util.spec_from_file_location('csd_score01', Path(__file__).with_name('score01.py'))
S = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(S)
ROOT, BIN, BINARY_SHA, CHAPTERS, POLICY = S.ROOT, S.BIN, S.BINARY_SHA, S.CHAPTERS, S.POLICY
NARRATION, TIMELINES, sha, read = S.NARRATION, S.TIMELINES, S.sha, S.read
BIN = ROOT / 'work/bin/production08c'
BINARY_SHA = '0d2501682af29f2fc0ba7fe54d5770b669bc97bef509d7d7c761e6b2e67a4676'
NARRATION = ROOT / 'jobs/stage08-picture-rebind01/narration'
TIMELINES = ROOT / 'jobs/stage08-picture-timeline01/timeline-output'
COMPARISON = ROOT / 'jobs/stage08-picture-timeline01/comparison.json'

JOB = ROOT / 'jobs/stage08-picture-preview01'
OUT = JOB / 'preview'
SCORE_OUT = S.OUT


def normalized(path):
    # Both products use the same exact host/path binding: no fuzzy normalization.
    return path.read_text().splitlines()


def check_score():
    assert not (ROOT / 'qualification01/native-detail08-hold.json').exists(), 'Native4K stair-step defect remains; preserve candidate, do not render production preview'
    comparison = S.check()
    for name in ('stage08-picture-repair-build03', 'stage08-picture-rebind01', 'stage08-picture-timeline01', 'stage08-picture-encoder-stop01'):
        S.success(name)
    assert sha(BIN) == BINARY_SHA
    repaired = read(COMPARISON)
    assert repaired['table_count'] == len(repaired['tables']) == 45
    assert {(r['chapter'], r['table']) for r in repaired['tables']} == {(c, k) for c in CHAPTERS for k in ('timeline', 'cues', 'cuelog')}
    for row in repaired['tables']:
        name = row['chapter'] + '.' + row['table'] + '.tsv'
        old, new = S.TIMELINES / name, TIMELINES / name
        assert sha(old) == row['old_sha256'] and sha(new) == row['new_sha256']
        assert old.read_bytes().replace(str(S.NARRATION).encode(), str(NARRATION).encode()) == new.read_bytes()
    stop = read(ROOT / 'jobs/stage08-picture-encoder-stop01/stop-summary.json')
    assert stop['binary_sha256'] == BINARY_SHA and stop['pass'] is True
    assert len(stop['tests']) == 2
    for test, size in zip(stop['tests'], ('960x540', '3840x2160')):
        assert test['size'] == size and test['pass'] is True and not test['survivors']
        assert test['native_exit'] < 0 and 0 <= test['elapsed'] < 41
        assert test['adapter_stop']['live_descendants'] == []
        assert test['command'][0] == str(BIN)
        width, height = map(int, size.split('x'))
        assert any(int(n) >= width*height*3 for n in test['encoder_read_bytes'].values())
    S.success('stage07-score01')
    summary = read(S.JOB / 'score-summary.json')
    assert summary['binary_sha256'] == S.BINARY_SHA
    assert summary['comparison_result_sha256'] == sha(S.COMPARISON / 'result.json')
    assert summary['chapter09_frame_shift'] == comparison['chapter09_frame_shift']
    assert list(summary['qa_sha256']) == list(CHAPTERS)
    for chapter, digest in summary['qa_sha256'].items():
        path = SCORE_OUT / 'stems/qa' / (chapter + '.json')
        report = read(path)
        assert sha(path) == digest and report['chapter'] == chapter and report['passed'] is True
        assert report['metrics'] and all(m.get('pass') is True or m.get('pass') is None for m in report['metrics'])
    return comparison


def score_files(chapter, frames):
    """What the kit lays for this chapter, in order: (file, first sample, samples)."""
    stem = SCORE_OUT / 'stems' / (chapter + '.music.f32.wav')
    parts = stem.with_suffix('.parts.tsv')
    if not parts.exists():
        return [(stem, 0, frames * 800)]
    rows = [line.split('\t') for line in parts.read_text().splitlines()[1:]]
    return [(stem.with_name(row[4]), int(row[0]), int(row[1])) for row in rows]


def check_cue_log(chapter, frames, rendered, timeline):
    """A scored render logs the timeline's cues unchanged, then its own score at the last frame.
    At 60 fps a frame is 800 samples at 48 kHz, and a row starts at
    frame * 800 + round(offset * 48000), as narration_delivery.py computes it. The rows must run end
    to end with no gap or overlap and fill the chapter exactly."""
    base, rows = normalized(timeline), normalized(rendered)
    assert rows[:len(base)] == base, 'Preview cue log differs: ' + rendered.name
    laid = [row.split('\t') for row in rows[len(base):]]
    files = score_files(chapter, frames)
    differ = 'Preview score rows differ: ' + rendered.name
    assert len(laid) == len(files), differ
    end = 0
    for row, (path, start, samples) in zip(laid, files):
        offset = float(row[5])
        exact = struct.unpack('>Q', struct.pack('>d', offset))[0]
        assert row[:5] == [str(path), sha(path), str(path.stat().st_size), str(frames), '60'], differ
        assert row[6:] == [f'{exact:#018x}', '0.0', '0x0000000000000000', '48000', '2', str(samples), str(start)], differ
        assert frames * 800 + round(offset * 48000) == start == end, differ
        end += samples
    assert end == frames * 800, differ


def main():
    assert sys.argv[1:] == ['--run'], 'Guard child only; freeze dependencies before launch'
    admission = read(JOB / 'resource-receipt.json')
    assert admission['admitted'] and 'exit' not in admission and admission['policy'] == POLICY
    comparison = check_score()
    probe_tool = shutil.which('ffprobe')
    assert probe_tool
    assert not OUT.exists(), 'Preserve existing preview'
    os.environ.update(TMPDIR=str(ROOT / 'work/tmp'), XDG_CACHE_HOME=str(ROOT / 'work/cache'))
    with (JOB / 'native.stdout').open('xb') as stdout, (JOB / 'native.stderr').open('xb') as stderr:
        subprocess.run([str(BIN), 'render', 'all', '--res', '960x540', '--fps', '60', '--narration', str(NARRATION),
                        '--score', str(SCORE_OUT / 'stems'), '--out', str(OUT), '--lint', '--crf', '18', '--preset', 'veryfast'],
                       cwd=ROOT / 'work/production-repo07', stdout=stdout, stderr=stderr, check=True, timeout=172800)
    assert {p.stem for p in OUT.glob('*.mp4')} == set(CHAPTERS)
    chapters = []
    for chapter in CHAPTERS:
        table = OUT / (chapter + '.timeline.tsv')
        for name in ('timeline', 'cues'):
            rendered = OUT / f'{chapter}.{name}.tsv'
            assert normalized(rendered) == normalized(TIMELINES / rendered.name), rendered.name
        movie = OUT / (chapter + '.mp4')
        probe = json.loads(subprocess.check_output([probe_tool, '-v', 'error', '-show_streams', '-of', 'json', str(movie)], timeout=120))
        video = [x for x in probe['streams'] if x['codec_type'] == 'video']
        audio = [x for x in probe['streams'] if x['codec_type'] == 'audio']
        assert len(video) == len(audio) == 1
        v = video[0]
        assert (v['width'], v['height'], v['avg_frame_rate']) == (960, 540, '60/1')
        frames = int(next(line.split('\t')[1] for line in normalized(table) if line.startswith('frames\t')))
        assert int(v['nb_frames']) == frames
        check_cue_log(chapter, frames, OUT / (chapter + '.cuelog.tsv'), TIMELINES / (chapter + '.cuelog.tsv'))
        layout = OUT / (chapter + '.layout.tsv')
        assert layout.is_file(), layout.name
        chapters.append({'chapter': chapter, 'frames': frames, 'mp4_sha256': sha(movie), 'timeline_sha256': sha(table),
                         'layout_sha256': sha(layout), 'layout_findings': len([x for x in layout.read_text().splitlines() if x.strip()])})
    check_score()
    with (JOB / 'preview-summary.json').open('x') as handle:
        json.dump({'chapters': chapters, 'res': '960x540', 'fps': 60, 'voice': 'ElevenLabs Frederick Surrey',
                   'binary_sha256': BINARY_SHA, 'comparison_result_sha256': sha(S.COMPARISON / 'result.json'),
                   'picture_comparison_sha256': sha(COMPARISON),
                   'chapter09_frame_shift': comparison['chapter09_frame_shift'],
                   'tables_equal_to_timeline': ['timeline', 'cues'],
                   'cue_log': 'Exact narration rows followed only by verified score rows covering all chapter samples.',
                   'scope': 'Technical preview only. Actual picture review, listening, fine-sync and independent final AV acceptance remain open.'}, handle, indent=2)
    print('PREVIEW technical checks passed; actual picture review pending.')


if __name__ == '__main__':
    main()
