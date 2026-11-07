"""Guarded production19 timelines and selected native 540p previews.
No import-time IO, remote actions, approval generation or old score reuse.
Root launches timeline first, then preview for seven changed/unresolved chapters.
"""
import argparse
import hashlib
import json
import os
import re
from pathlib import Path
import shutil
import subprocess
import sys

ROOT = Path('/data/projects/jmev-stroke-probe-20261106')
Q = ROOT / 'qualification01/production-repair17'
TREE = ROOT / 'work/production-repair19/workspace'
C = TREE / 'explainer_corpus/complex_analysis_visual'
BIN = ROOT / 'work/bin/production19'
ASSEMBLY = ROOT / 'jobs/csd-production-assembly19'
NARRATION = ASSEMBLY / 'narration'
BUILD = ROOT / 'jobs/csd-production-repair-build19'
TIMELINE_JOB = ROOT / 'jobs/csd-production-timeline20'
TIMELINES = TIMELINE_JOB / 'timeline-output'
PREVIEW_JOB = ROOT / 'jobs/csd-production-preview20'
OLD_TIMELINES = ROOT / 'jobs/csd-continuous-timeline16/timeline-output'
OLD_NARRATION = ROOT / 'jobs/csd-continuous-rebind16/narration'
PRIOR_PREVIEW = ROOT / 'jobs/csd-continuous-preview16/preview'
PRIOR_EXTRACTION = ROOT / 'jobs/csd-continuous-pictures16/pictures/extraction.json'
PRIOR12 = ROOT / 'qualification01/continuous-quality16/prior-review12'
POLICY = dict(admission_gib=10, stop_free_gib=8, phase_growth_gib=4,
              work_cap_gib=56, target_growth_cap_gib=20, sample_seconds=10)
FRESH_PREFIXES = ('03', '05', '08', '09', '11', '13', '14')
REUSE_CANDIDATE_PREFIXES = ('01', '02', '04', '06', '07', '10', '12', '15')


def read(path):
    return json.loads(Path(path).read_text())


def sha(path):
    with Path(path).open('rb') as stream:
        return hashlib.file_digest(stream, 'sha256').hexdigest()


def write_new(path, value):
    with path.open('x') as stream:
        json.dump(value, stream, indent=2)


def success(job):
    receipt = read(job / 'resource-receipt.json')
    assert receipt.get('exit') == 0 and receipt.get('finished'), job
    assert receipt['inputs_sha256'] == receipt['final_inputs_sha256'], job
    assert not any(receipt.get(key) for key in ('error', 'resource_stop', 'stopped_for_resource_limit')), job
    return receipt


def check_dependencies():
    success(BUILD)
    success(ASSEMBLY)
    build = read(BUILD / 'qualification.json')
    assembled = read(ASSEMBLY / 'assembly19-summary.json')
    assert sha(BIN) == build['binary_sha256'] == assembled['build_binary_sha256']
    assert assembled['narration'] == str(NARRATION)
    for path, key in [(C / 'script.tsv', 'script_sha256'),
                      (NARRATION / 'external-units.json', 'units_sha256'),
                      (NARRATION / 'assembly.json', 'assembly_sha256'),
                      (NARRATION / 'report.json', 'report_sha256'),
                      (NARRATION / 'phrase-timings.json', 'phrase_timings_sha256'),
                      (BUILD / 'source-copy.json', 'build_source_copy_sha256'),
                      (TREE / 'tools/coherent_narration.py', 'adapter_sha256')]:
        assert sha(path) == assembled[key], (path, key)
    chapters = build['chapters']
    assert len(chapters) == len(set(chapters)) == 15
    assert tuple(c[:2] for c in chapters) == tuple(f'{n:02}' for n in range(1, 16))
    return build, assembled, chapters


def run(job, name, command, timeout=172800):
    with (job / (name + '.stdout')).open('xb') as out, (job / (name + '.stderr')).open('xb') as err:
        subprocess.run(list(map(str, command)), cwd=ROOT / 'work/production-repo07', stdout=out, stderr=err,
                       check=True, timeout=timeout)
    return (job / (name + '.stdout')).read_bytes()


def frames_from_text(text):
    rows = [line.split('\t') for line in text.splitlines()]
    found = [int(row[1]) for row in rows if row[0] == 'frames']
    assert len(found) == 1 and found[0] > 0
    fps = [int(row[1]) for row in rows if row[0] == 'fps']
    assert fps == [60]
    return found[0]


def verify_delivery(job, chapters):
    command = [sys.executable, '-I', '-B', TREE / 'tools/narration_delivery.py',
               '--topic', C, '--narration', NARRATION, '--consumer', 'render', '--gain-db', '0']
    for chapter in chapters:
        command.extend(['--chapter-key', chapter])
    result = json.loads(run(job, 'delivery', command))
    assert result['verified_cues'] == 105 and result['selected_cues'] == 105
    assert result['promotion'] is False and result['listening_acceptance'] is False


def timeline(job, build, chapters):
    assert not TIMELINES.exists(), 'Preserve previous schedules'
    run(job, 'native-timeline', [BIN, 'render', 'all', '--timeline-only', '--fps', '60',
                               '--threads', '4', '--narration', NARRATION, '--out', TIMELINES])
    assert {p.stem.split('.')[0] for p in TIMELINES.glob('*.timeline.tsv')} == set(chapters)
    rows = []
    cumulative = 0
    for chapter in chapters:
        table = TIMELINES / (chapter + '.timeline.tsv')
        frames = frames_from_text(table.read_text())
        comparisons = []
        for kind in ('timeline', 'cues', 'cuelog'):
            old = OLD_TIMELINES / f'{chapter}.{kind}.tsv'
            new = TIMELINES / old.name
            # This reports schedule equality only. Audio/source/review promotion
            # still requires the independent per-chapter reuse proof.
            expected = old.read_bytes().replace(str(OLD_NARRATION).encode(), str(NARRATION).encode())
            comparisons.append(dict(kind=kind, old_sha256=sha(old), new_sha256=sha(new),
                                    exact_equal_after_named_directory_mapping=expected == new.read_bytes()))
        rows.append(dict(chapter=chapter, frames=frames, start_frame=cumulative,
                         end_frame_exclusive=cumulative + frames, seconds=frames / 60,
                         raw_yuv420p_bytes_4k=frames * 3840 * 2160 * 3 // 2,
                         tables=comparisons))
        cumulative += frames
    write_new(job / 'timeline20-summary.json', dict(
        binary_sha256=build['binary_sha256'], assembly_summary_sha256=sha(ASSEMBLY / 'assembly19-summary.json'),
        chapters=rows, fps=60, total_frames=cumulative, seconds=cumulative / 60,
        named_path_mapping={str(OLD_NARRATION): str(NARRATION)},
        scope='New all15 native schedules; table equality alone does not authorize prior media reuse.'))


def cue_starts(table_text):
    return {row[1]: int(row[2]) for row in (line.split('\t') for line in table_text.splitlines())
            if row[0] == 'line'}


def select_picture_frames(table_text, frames, phrase_cues, layout_sources=(), prior_samples=(), old_table_text=''):
    chosen = {0: {'first frame'}, frames - 1: {'last frame'}}
    starts = cue_starts(table_text)
    old_starts = sorted(cue_starts(old_table_text).items(), key=lambda row: row[1])

    def add(frame, reason):
        if 0 <= frame < frames:
            chosen.setdefault(frame, set()).add(reason)

    for second in range(1, (frames - 1) // 60 + 1, 5):
        add(second * 60, 'five-second chapter progression; sampled, not every frame')
    for cue, start in starts.items():
        for offset in (-12, 12, 30):
            add(start + offset, cue + ' cue transition')
        for mark in phrase_cues.get(cue, {}).get('marks', []):
            for delta in (-0.2, 0.5, 1.5):
                add(start + round((mark['seconds'] + delta) * 60),
                    cue + ' provisional phrase byte ' + str(mark['byte']) + ' offset ' + str(delta))

    # The native linter stores its first checkpoint plus an aggregate recurrence
    # count. Once seen, a finding increments again even after gaps: the count
    # cannot identify its final timestamp or imply consecutive persistence.
    for source in layout_sources:
        for line in source['text'].splitlines():
            if not line.strip():
                continue
            checkpoint = line.split('\t', 1)[0]
            match = re.search(r'@([0-9]+(?:\.[0-9]+)?)s', checkpoint)
            assert match, ('unrecognized native layout checkpoint', source['path'], checkpoint)
            anchor = float(match[1])
            anchors = [(anchor, 'original absolute anchor')]
            if not source['current']:
                relative = re.search(r'^(\d+_\d+) \+([0-9]+(?:\.[0-9]+)?)s', checkpoint)
                if relative and relative[1] in starts:
                    anchors.append((starts[relative[1]] / 60 + float(relative[2]),
                                    'cue-relative historical anchor; correspondence requires viewing'))
            repeated = re.search(r'still there at (\d+) later looks', line)
            for at, kind in anchors:
                reason = source['path'] + ' ' + kind + ': ' + line
                for delta in (-0.2, 0, 0.2, 1, 3, 6, 10):
                    add(round((at + delta) * 60), reason + ' offset ' + str(delta))
                if repeated:
                    # Scan to chapter end instead of inventing a recurrence end
                    # from count*0.2. The extracted frames remain samples only.
                    for frame in range(max(0, round(at * 60)), frames, 5 * 60):
                        add(frame, reason + '; recurrence scan, later-look count is not a duration')
                    add(frames - 1, reason + '; final-frame recurrence sample')

    # Preserve all production16 selected timestamps, even when repaired lint is
    # silent, and also follow their cue-relative offsets after08/09 lengthening.
    for prior in prior_samples:
        old_frame = int(prior['frame'])
        reason = 'production16 selected timestamp; historical, not current approval: ' + ' | '.join(prior['reasons'])
        add(old_frame, reason)
        owner = next(((cue, start) for cue, start in reversed(old_starts) if start <= old_frame), None)
        if owner and owner[0] in starts:
            cue, old_start = owner
            add(starts[cue] + old_frame - old_start,
                reason + '; cue-relative remap ' + cue + ' (changed-cue meaning requires viewing)')
    return [dict(frame=frame, seconds=frame / 60, reasons=sorted(reasons)) for frame, reasons in sorted(chosen.items())]


def preview(job, build, chapters, requested):
    success(TIMELINE_JOB)
    schedule = read(TIMELINE_JOB / 'timeline20-summary.json')
    assert schedule['binary_sha256'] == build['binary_sha256']
    assert schedule['assembly_summary_sha256'] == sha(ASSEMBLY / 'assembly19-summary.json')
    selected = requested or [c for c in chapters if c[:2] in FRESH_PREFIXES]
    assert selected and len(selected) == len(set(selected)) and set(selected) <= set(chapters)
    out, pictures = job / 'preview', job / 'pictures'
    assert not out.exists() and not pictures.exists(), 'Preserve previous review artifacts'
    run(job, 'native-preview', [BIN, 'render', *selected, '--res', '960x540', '--fps', '60', '--threads', '4',
                                '--narration', NARRATION, '--out', out, '--lint', '--crf', '18', '--preset', 'veryfast'])
    assert {p.stem for p in out.glob('*.mp4')} == set(selected)
    phrase_cues = read(NARRATION / 'phrase-timings.json')['cues']
    success(ROOT / 'jobs/csd-continuous-preview16')
    success(ROOT / 'jobs/csd-continuous-pictures16')
    prior_extraction = read(PRIOR_EXTRACTION)
    prior_summary_path = PRIOR_PREVIEW.parent / 'preview-summary.json'
    prior_summary = read(prior_summary_path)
    assert prior_extraction['preview_summary_sha256'] == sha(prior_summary_path)
    rows = []
    for chapter in selected:
        expected = next(r for r in schedule['chapters'] if r['chapter'] == chapter)
        frames = expected['frames']
        for kind in ('timeline', 'cues', 'cuelog'):
            table = TIMELINES / f'{chapter}.{kind}.tsv'
            assert sha(table) == next(r['new_sha256'] for r in expected['tables'] if r['kind'] == kind)
            assert table.read_bytes() == (out / table.name).read_bytes(), (chapter, kind)
        movie = out / (chapter + '.mp4')
        streams = json.loads(subprocess.check_output(['ffprobe', '-v', 'error', '-show_streams', '-of', 'json', str(movie)], timeout=120))['streams']
        video = [s for s in streams if s['codec_type'] == 'video']
        audio = [s for s in streams if s['codec_type'] == 'audio']
        assert len(video) == len(audio) == 1
        v = video[0]
        assert (v['width'], v['height'], v['avg_frame_rate'], int(v['nb_frames'])) == (960, 540, '60/1', frames)
        run(job, chapter + '-decode', ['ffmpeg', '-nostdin', '-v', 'error', '-xerror', '-threads', '2',
                                      '-i', movie, '-f', 'null', '-'])
        folder = pictures / chapter
        folder.mkdir(parents=True)
        run(job, chapter + '-sheets', ['ffmpeg', '-nostdin', '-n', '-v', 'error', '-i', movie,
                                     '-vf', 'fps=1/5,scale=480:-1,tile=4x4:padding=6:margin=6',
                                     '-vsync', '0', '-q:v', '2', folder / 'sheet-%03d.jpg'])
        layout = out / (chapter + '.layout.tsv')
        old_layout = PRIOR_PREVIEW / (chapter + '.layout.tsv')
        old_info = next(r for r in prior_summary['chapters'] if r['chapter'] == chapter)
        assert sha(old_layout) == old_info['layout_sha256']
        layout_sources = []
        for path in (layout, old_layout, PRIOR12 / (chapter + '.layout.tsv')):
            assert path.is_file(), path
            layout_sources.append(dict(path=str(path), sha256=sha(path), text=path.read_text(), current=path == layout))
        historical_samples = [r for r in prior_extraction['samples'] if r['chapter'] == chapter]
        assert historical_samples, chapter
        samples = select_picture_frames((TIMELINES / (chapter + '.timeline.tsv')).read_text(), frames, phrase_cues,
                                        layout_sources, historical_samples,
                                        (OLD_TIMELINES / (chapter + '.timeline.tsv')).read_text())
        selector = '+'.join('eq(n,' + str(s['frame']) + ')' for s in samples)
        filter_path = folder / 'select.filter'
        with filter_path.open('x') as stream:
            stream.write("select='" + selector + "'")
        run(job, chapter + '-cue-pictures', ['ffmpeg', '-nostdin', '-n', '-v', 'error', '-i', movie,
                                           '-filter_script:v', filter_path, '-vsync', '0', '-q:v', '2',
                                           folder / 'cue-%05d.jpg'])
        actual = sorted(folder.glob('cue-*.jpg'))
        assert len(actual) == len(samples)
        for item, path in zip(samples, actual):
            item.update(path=str(path), sha256=sha(path))
        layout = out / (chapter + '.layout.tsv')
        assert layout.is_file()
        rows.append(dict(chapter=chapter, frames=frames, mp4_sha256=sha(movie),
                         timeline_sha256=sha(out / (chapter + '.timeline.tsv')),
                         layout_sha256=sha(layout), samples=samples,
                         layout_sources=[{key: value for key, value in r.items() if key != 'text'} for r in layout_sources],
                         prior_extraction_sha256=sha(PRIOR_EXTRACTION),
                         historical_sample_count=len(historical_samples),
                         sampling_scope='Selected frames only; later-look counts do not identify consecutive persistence or exact recurrence times',
                         sheets=[dict(path=str(p), sha256=sha(p)) for p in sorted(folder.glob('sheet-*.jpg'))]))
    write_new(job / 'preview20-summary.json', dict(
        chapters=rows, not_rendered=[c for c in chapters if c not in selected],
        binary_sha256=build['binary_sha256'], timeline_summary_sha256=sha(TIMELINE_JOB / 'timeline20-summary.json'),
        fps=60, resolution=[960, 540], score_used=False, pictures_reviewed=False, motion_reviewed=False,
        reuse_approved=False, listening_accepted=False, fine_sync_accepted=False,
        scope='Technical native previews and extracted pictures only. Skipped chapters require exact independent reuse proof; no all15 permit.'))
    write_new(job / 'motion-review-targets.json', dict(
        reviewed=False, binary_sha256=build['binary_sha256'],
        intervals=[dict(chapter=r['chapter'], first_frame=0, last_frame=r['frames'] - 1,
                        path=str(out / (r['chapter'] + '.mp4')), mp4_sha256=r['mp4_sha256']) for r in rows],
        scope='Full native movies available for actual review; targets are not observations.'))


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--phase', required=True, choices=('timeline', 'preview'))
    parser.add_argument('--chapter', action='append', default=[])
    args = parser.parse_args()
    assert args.phase == 'preview' or not args.chapter
    job = TIMELINE_JOB if args.phase == 'timeline' else PREVIEW_JOB
    admission = read(job / 'resource-receipt.json')
    assert admission['admitted'] and 'exit' not in admission and admission['policy'] == POLICY
    os.sched_setaffinity(0, {0, 1, 2, 3})
    assert os.sched_getaffinity(0) == {0, 1, 2, 3}
    assert shutil.which('ffmpeg') and shutil.which('ffprobe')
    os.environ.update(TMPDIR=str(ROOT / 'work/tmp'), XDG_CACHE_HOME=str(ROOT / 'work/cache'))
    before = check_dependencies()
    build, assembled, chapters = before
    assert run(job, 'registry', [BIN, 'chapters']).decode().splitlines() == chapters
    verify_delivery(job, chapters)
    if args.phase == 'timeline':
        timeline(job, build, chapters)
    else:
        preview(job, build, chapters, args.chapter)
    assert check_dependencies() == before
    print('production19 ' + args.phase + ' technical work complete; no review or publication approval generated.')


if __name__ == '__main__':
    main()
