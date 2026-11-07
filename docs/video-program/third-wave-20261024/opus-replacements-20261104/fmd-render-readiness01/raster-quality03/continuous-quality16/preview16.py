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

ROOT=Path('/data/projects/jmev-stroke-probe-20261106')
Q=ROOT/'qualification01/continuous-quality16'
TREE=ROOT/'work/continuous-quality16/workspace'
BIN=ROOT/'work/bin/production16'
BINARY_SHA='8dc5e8c4f640b73d9e4e6c4575cb492ebcc0deb0c2de0a658bbccf40d46082d1'
NARRATION=ROOT/'jobs/csd-continuous-rebind16/narration'
TIMELINES=ROOT/'jobs/csd-continuous-timeline16/timeline-output'
COMPARISON=ROOT/'jobs/csd-continuous-timeline16/comparison.json'
BASE=Q/'original-timelines'
OLD_N='/data/projects/jmev-stroke-probe-20261106/jobs/stage07-narration-derive01/narration'
JOB=ROOT/'jobs/csd-continuous-preview16'
OUT=JOB/'preview'
SCORE_OUT=ROOT/'production-inputs01/scored'
POLICY=dict(admission_gib=10,stop_free_gib=8,phase_growth_gib=4,work_cap_gib=56,target_growth_cap_gib=20,sample_seconds=10)
import hashlib
from types import SimpleNamespace
def read(p):return json.loads(p.read_text())
def sha(p):
    with p.open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest()
def success(name):
    r=read(ROOT/'jobs'/name/'resource-receipt.json')
    assert r['exit']==0 and r['inputs_sha256']==r['final_inputs_sha256'] and not r.get('resource_stop'),name
CHAPTERS=tuple(read(Q/'original-score-summary12.json')['qa_sha256'])
assert len(CHAPTERS)==15
S=SimpleNamespace(ROOT=ROOT,CHAPTERS=CHAPTERS,POLICY=POLICY,sha=sha,read=read,success=success)

def normalized(path):
    # Both products use the same exact host/path binding: no fuzzy normalization.
    return path.read_text().splitlines()


def check_score():
    for name in ['csd-continuous-quality-build16','csd-continuous-rebind16','csd-continuous-timeline16','csd-continuous-encoder-stop16']:
        success(name)
    assert sha(BIN)==BINARY_SHA
    review=read(Q/'native-detail16-reviewed.json')
    assert review['binary_sha256']==BINARY_SHA and review['decision']=='pass_native_detail_for_pipeline' and not review['unresolved_picture_defects']
    repaired=read(COMPARISON)
    assert repaired['table_count']==len(repaired['tables'])==45
    assert {(r['chapter'],r['table']) for r in repaired['tables']}=={(c,k) for c in CHAPTERS for k in ('timeline','cues','cuelog')}
    for row in repaired['tables']:
        name=row['chapter']+'.'+row['table']+'.tsv';old,new=BASE/name,TIMELINES/name
        assert sha(old)==row['old_sha256'] and sha(new)==row['new_sha256']
        assert old.read_bytes().replace(OLD_N.encode(),str(NARRATION).encode())==new.read_bytes()
    stop=read(ROOT/'jobs/csd-continuous-encoder-stop16/stop-summary.json')
    assert stop['binary_sha256']==BINARY_SHA and stop['pass'] and len(stop['tests'])==2
    for test,size in zip(stop['tests'],('960x540','3840x2160')):
        assert test['size']==size and test['pass'] and not test['survivors']
        assert test['native_exit']<0 and 0<=test['elapsed']<41 and test['adapter_stop']['live_descendants']==[]
        assert test['command'][0]==str(BIN)
        w,h=map(int,size.split('x'))
        assert any(int(n)>=w*h*3 for n in test['encoder_read_bytes'].values())
    r=read(Q/'original-score-receipt12.json')
    assert r['exit']==0 and r['inputs_sha256']==r['final_inputs_sha256'] and not r.get('resource_stop')
    for row in read(Q/'original-score-files12.json'):assert sha(SCORE_OUT/row['name'])==row['sha256'],row['name']
    summary=read(Q/'original-score-summary12.json')
    assert summary['chapters']==15 and summary['cues']==105 and tuple(summary['qa_sha256'])==CHAPTERS
    for chapter,digest in summary['qa_sha256'].items():
        path=SCORE_OUT/'stems/qa'/(chapter+'.json');r=read(path)
        assert sha(path)==digest and r['chapter']==chapter and r['passed'] is True
        assert r['metrics'] and all(m.get('pass') is True or m.get('pass') is None for m in r['metrics'])
    return summary


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
    os.sched_setaffinity(0,{0,1,2,3})
    assert os.sched_getaffinity(0)=={0,1,2,3}
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
                   'binary_sha256': BINARY_SHA, 'comparison_result_sha256': comparison['comparison_result_sha256'],
                   'picture_comparison_sha256': sha(COMPARISON),
                   'chapter09_frame_shift': comparison['chapter09_frame_shift'],
                   'tables_equal_to_timeline': ['timeline', 'cues'],
                   'cue_log': 'Exact narration rows followed only by verified score rows covering all chapter samples.',
                   'scope': 'Technical preview only. Actual picture review, listening, fine-sync and independent final AV acceptance remain open.'}, handle, indent=2)
    print('PREVIEW technical checks passed; actual picture review pending.')


if __name__ == '__main__':
    main()
