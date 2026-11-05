"""Timeline, score, preview and pictures for the qualified ts1 complex binary.

These are the sealed macOS stages from complex-optimized-recovery-review01 and
complex-optimized-downstream01 in one file, with every check kept. What changes: the lane, the job
names, logical paths, the two isolated Pythons, and the time limits INTEGRATION.md lists. Nothing
synthesizes, and viewing stays a separate human step.

    --check STAGE    read-only: pins, frozen files, prior run, tools, unused names; launches nothing
    --launch STAGE   wait for upstream exit files, freeze, run ts1's guard (start-chain01.sh)
    --run STAGE      the guard's child; never call it by hand
"""
import importlib.util
import json
from pathlib import Path
import re
import shutil
import struct
import subprocess
import sys
import time
import wave

HERE = Path(__file__).resolve().parent
QUALIFIER = HERE.parent / 'qualify01/ts1_qualify01.py'
QUALIFIER_SHA256 = 'ee2d7719218be4e8d77a14f99a352e2f6a8d287fba1befed3bf8db57d0fb619b'
SPEC = importlib.util.spec_from_file_location('ts1_qualify', QUALIFIER)
Q = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(Q)
R, O, JOBS, C, BIN, CHAPTERS, POLICY, read, sha = Q.R, Q.O, Q.JOBS, Q.C, Q.BIN, Q.CHAPTERS, Q.POLICY, Q.read, Q.sha

AUDIO_ROWS_SHA256 = 'dd8dbc6fc6ca85ad859ce0f2a0a3024a56f97283e7c15218ac7fe58904086fd2'
CLOSURE_SHA256 = '694241727ca3b162e298624501ca0349138b3209927e9cac5bb92274df6582a7'
FAILED_MANIFEST_SHA256 = '7d6aeaf7d1f8b83a55fb7818f48ff01c2c6fa62ba135d84ea63c8a087307072c'
PILLOW = '12.1.1'
TIMELINE_SECONDS = PREVIEW_SECONDS = 172800  # a bound, not an estimate; see INTEGRATION.md
SCORE_SECONDS = 7200
WAIT_POLLS = 30240  # 7 days at 20 s, because each upstream limit is 48 h

ME = R / Path(__file__).resolve().relative_to(Q.PHYSICAL)
START = ME.parent / 'start-chain01.sh'
EC = R / Q.EC
NATIVE = EC / 'native-stage06-root03'
N = EC / 'production-audio-stage06-01/native-narration01'
AUDIO_ROWS = O / 'audio-support-transfer01.json'
CLOSURE = ME.parent.parent / 'narration-closure01.json'
B = R / 'narration_lab/third-wave-20261024/root/five-film-release-20261027'
FAILED = B / 'complex-stage06-timeline01'
PRIOR_OUT = FAILED / 'timeline-output'
PRIOR_CHAPTERS = CHAPTERS[:4]
TABLES = ('timeline', 'cues', 'cuelog')
PICTURE_ENV = 'complex-remote-picture-env01'
PICTURE_PYTHON = JOBS / PICTURE_ENV / 'env/bin/python'
NAMES = {'timeline': 'complex-remote-timeline01', 'score': 'complex-remote-score01',
         'preview': 'complex-remote-preview01', 'pictures': 'complex-remote-pictures01'}
UPSTREAM = {'timeline': (), 'score': ('timeline',), 'preview': ('timeline', 'score'),
            'pictures': ('timeline', 'score', 'preview')}
TIMELINE_OUT = JOBS / NAMES['timeline'] / 'timeline-output'
SCORE_OUT = JOBS / NAMES['score'] / 'scored'
PREVIEW_OUT = JOBS / NAMES['preview'] / 'preview'
PICTURES_OUT = JOBS / NAMES['pictures'] / 'pictures'


def job(stage):
    return JOBS / NAMES[stage]


def exit_file(stage):
    return O / f'chain-{stage}-controller01.exit'


def manifest(stage):
    return O / f'chain-{stage}-inputs01.json'


def child_command(stage):
    python = str(PICTURE_PYTHON) if stage == 'pictures' else sys.executable
    return [python, '-I', '-B', str(ME), '--run', stage]


def guard_command(stage):
    return [sys.executable, '-I', '-B', str(Q.GUARD), '--phase', 'native', '--job', NAMES[stage],
            '--inputs', str(manifest(stage)), '--', *child_command(stage)]


def tool(name):
    found = shutil.which(name)
    assert found, 'Missing tool: ' + name
    return Path(found).resolve()


def normalized(path):
    return [line for line in path.read_text().splitlines() if line and not line.startswith('#')]


def table_value(rows, key):
    found = [row[1] for row in rows if row[0] == key]
    assert len(found) == 1, key
    return found[0]


def script_ids():
    return [line.split('\t', 1)[0] for line in (C / 'script.tsv').read_text().splitlines()
            if line.strip() and not line.startswith('#')]


def check_pins():
    assert sha(QUALIFIER) == QUALIFIER_SHA256, 'Sealed qualifier changed'
    assert sha(AUDIO_ROWS) == AUDIO_ROWS_SHA256, 'Audio transfer rows changed'
    assert sha(CLOSURE) == CLOSURE_SHA256, 'Narration closure changed'


def check_qualified():
    """Downstream needs the qualifier's success and the original failed receipt together."""
    Q.check_build()
    Q.success(Q.JOB)
    s = read(JOBS / Q.JOB / 'qualification-summary.json')
    assert s['original_build_job'] == Q.BUILD and s['original_build_exit'] == 1, 'Original build was relabelled'
    assert s['original_build_relabelled'] is False, 'Original build was relabelled'
    assert s['original_receipt_sha256'] == sha(JOBS / Q.BUILD / 'resource-receipt.json'), 'Original evidence changed'
    assert s['original_log_sha256'] == sha(JOBS / Q.BUILD / 'command.log'), 'Original evidence changed'
    assert s['binary_sha256'] == Q.BIN_SHA256 == sha(BIN), 'Qualified binary differs'
    assert s['chapters'] == list(CHAPTERS) and s['fact_rows'] > 0, 'Qualification summary differs'


def check_failed_run():
    """The macOS debug job is what its receipts say: a timeout with four finished chapters."""
    d = read(FAILED / 'resource-receipt.json')
    assert d.get('admitted') is True and d.get('exit') == 1 and d.get('finished'), 'Failed receipt changed'
    assert d.get('inputs_sha256') == d.get('final_inputs_sha256') == FAILED_MANIFEST_SHA256
    assert not d.get('stopped_for_resource_limit') and 'error' not in d
    assert d.get('policy') == POLICY, 'Failed run used another policy'
    log = (FAILED / 'command.log').read_text()
    assert 'timed out after 14400 seconds' in log
    assert not (FAILED / 'timeline-summary.json').exists(), 'No full baseline may exist'
    assert sorted(p.name for p in PRIOR_OUT.glob('*.tsv')) == sorted(
        f'{chapter}.{table}.tsv' for chapter in PRIOR_CHAPTERS for table in TABLES)
    for chapter in PRIOR_CHAPTERS:
        rows = [line.split('\t') for line in (PRIOR_OUT / f'{chapter}.timeline.tsv').read_text().splitlines()]
        assert re.search(rf'^{chapter}\s+{table_value(rows, "frames")} frames\s', log, re.M), chapter


def base_rows():
    """The source rows, the audio rows and the 316 files the narration verifiers open. A file added
    to the audio or workspace folders is refused as firmly as a changed one."""
    rows = [*read(Q.INPUTS)['files'], *read(AUDIO_ROWS)['files']]
    rows += [{'path': str(R / row['path']), 'sha256': row['sha256']} for row in read(CLOSURE)['files']]
    files = {}
    for row in rows:
        assert files.setdefault(row['path'], row['sha256']) == row['sha256'], row['path']
    for root in (N, NATIVE / 'workspace'):
        listed = {path for path in files if path.startswith(str(root) + '/')}
        different = sorted(listed ^ {str(p) for p in root.rglob('*') if p.is_file()})
        assert not different, f'File set changed: {len(different)} paths, first {different[:3]}'
    for path, digest in files.items():
        assert sha(path) == digest, path
    return files


def stage_files(stage):
    if stage == 'timeline':
        files = base_rows()
        rows = read(Q.MANIFEST)['files']
        paths = [Q.MANIFEST, AUDIO_ROWS, CLOSURE, ME, START, tool('ffmpeg'), *Q.job_files(Q.JOB)]
    else:
        prior = UPSTREAM[stage][-1]
        files = {}
        rows = read(manifest(prior))['files']
        paths = [manifest(prior), ME, START, *Q.job_files(NAMES[prior])]
    if stage == 'preview':
        paths.append(tool('ffprobe'))
    if stage == 'pictures':
        paths += [tool('ffmpeg'), *Q.job_files(PICTURE_ENV)]
    for row in rows:
        assert files.setdefault(row['path'], row['sha256']) == row['sha256'], row['path']
        assert sha(row['path']) == row['sha256'], row['path']
    for path in paths:
        files[str(path)] = sha(path)
    return files


def child_gate(stage):
    d = read(job(stage) / 'resource-receipt.json')
    assert d.get('admitted') is True and 'exit' not in d
    assert d['command'] == child_command(stage)
    assert d['policy'] == POLICY, 'Guard policy changed'
    rows = read(manifest(stage))['files']
    for row in rows:
        assert sha(row['path']) == row['sha256'], row['path']
    assert str(tool('ffmpeg')) in {row['path'] for row in rows}, 'Child resolves an ffmpeg outside the manifest'


def check_facts(folder):
    """The macOS checks, with the oracle under its isolated env. Both outputs are kept first."""
    facts = {}
    for name, argv in [('native', [str(BIN), 'facts']), ('oracle', [str(Q.ENV_PYTHON), '-I', '-B', str(Q.ORACLE)])]:
        result = subprocess.run(argv, cwd=C, capture_output=True, timeout=900)
        for stream, data in [('stdout', result.stdout), ('stderr', result.stderr)]:
            with (folder / f'facts-{name}.{stream}').open('xb') as handle:
                handle.write(data)
        assert result.returncode == 0, (name, result.returncode)
        facts[name] = result.stdout
    assert facts['native'] and facts['native'] == facts['oracle'], 'Native facts differ from oracle; both outputs retained'
    assert facts['native'] == (FAILED / 'facts-native.stdout').read_bytes() == Q.RELEASE_FACTS.read_bytes(), 'ts1 facts differ from macOS'
    keys = [line.split(b'\t', 1)[0] for line in facts['native'].splitlines()]
    assert len(keys) == len(set(keys))
    return len(keys)


def verify_narration():
    subprocess.run([sys.executable, '-I', '-B', str(NATIVE / 'workspace/tools/coherent_narration.py'),
                    'verify', str(N), '--script', str(C / 'script.tsv'), '--units',
                    str(N / 'external-units.json'), '--require-directions'], cwd=R, check=True, timeout=900)


def compare_prior(out):
    """Write the comparison before judging it, so a mismatch leaves its evidence."""
    rows = []
    for chapter in PRIOR_CHAPTERS:
        for table in TABLES:
            name = f'{chapter}.{table}.tsv'
            old, new = normalized(PRIOR_OUT / name), normalized(out / name)
            rows.append({'table': name, 'lines': len(old), 'normalized_equal': old == new,
                         'prior_sha256': sha(PRIOR_OUT / name), 'new_sha256': sha(out / name)})
    with (job('timeline') / 'prior-table-comparison.json').open('x') as handle:
        json.dump({'prior_job': FAILED.name, 'new_binary_sha256': Q.BIN_SHA256, 'tables': rows}, handle, indent=2)
    different = [row['table'] for row in rows if not row['normalized_equal']]
    assert not different, f'ts1 tables differ from the macOS run: {different}'
    return rows


def check_tables(out, script):
    seen, chapters = [], []
    for chapter in CHAPTERS:
        path = out / (chapter + '.timeline.tsv')
        rows = [line.split('\t') for line in path.read_text().splitlines()]
        assert table_value(rows, 'chapter') == chapter and table_value(rows, 'fps') == '60'
        frames = int(table_value(rows, 'frames'))
        assert frames > 0
        lines = [row for row in rows if row[0] == 'line']
        starts = [int(row[2]) for row in lines]
        assert starts == sorted(starts) and all(0 <= frame < frames for frame in starts)
        for row in lines:
            assert Path(row[4]).resolve() == (N / (row[1] + '.wav')).resolve()
        seen.extend(row[1] for row in lines)
        chapters.append({'chapter': chapter, 'frames': frames, 'cues': [row[1] for row in lines],
                         'timeline_sha256': sha(path)})
    assert seen == script, 'Cue placements differ from the full ordered script'
    return chapters, seen


def run_timeline():
    child_gate('timeline')
    check_qualified()
    check_failed_run()
    listed = subprocess.check_output([str(BIN), 'chapters'], text=True).splitlines()
    assert listed == list(CHAPTERS), listed
    script = script_ids()
    assert len(script) == len(set(script)) == 105
    fact_rows = check_facts(job('timeline'))
    verify_narration()
    assert not TIMELINE_OUT.exists(), 'Existing timeline output retained'
    subprocess.run([str(BIN), 'render', 'all', '--timeline-only', '--fps', '60', '--narration', str(N),
                    '--out', str(TIMELINE_OUT)], cwd=R, check=True, timeout=TIMELINE_SECONDS)
    compared = compare_prior(TIMELINE_OUT)
    chapters, seen = check_tables(TIMELINE_OUT, script)
    with (job('timeline') / 'timeline-summary.json').open('x') as handle:
        json.dump({'chapters': chapters, 'cues': len(seen), 'fps': 60, 'native_oracle_fact_rows_matched': fact_rows,
                   'total_frames': sum(row['frames'] for row in chapters), 'binary_sha256': Q.BIN_SHA256,
                   'profile': 'release', 'host': 'ts1', 'qualifier_job': Q.JOB, 'failed_job': FAILED.name,
                   'prior_chapters_compared': list(PRIOR_CHAPTERS), 'prior_tables_compared': len(compared),
                   'chapters_without_prior_table': list(CHAPTERS[len(PRIOR_CHAPTERS):]),
                   'full_baseline_equivalence_established': False,
                   'scope': 'ts1 release native timeline only. Tables for chapters 01-04 equal the macOS debug run line for line; chapters 05-15 have no prior table. No speedup, frame, PCM, listening or AV equivalence established.'}, handle, indent=2)
    print('TIMELINE PASS: 15 chapters, 105 cues, 12 prior tables equal; no full baseline equivalence.', flush=True)


def check_timeline():
    Q.success(NAMES['timeline'])
    summary = read(job('timeline') / 'timeline-summary.json')
    assert summary['binary_sha256'] == Q.BIN_SHA256 == sha(BIN) and summary['profile'] == 'release', 'Timeline binary differs'
    assert summary['failed_job'] == FAILED.name, 'Timeline names another failed run'
    assert summary['full_baseline_equivalence_established'] is False, 'Full baseline equivalence cannot be claimed'
    assert summary['prior_chapters_compared'] == list(PRIOR_CHAPTERS)
    assert summary['chapters_without_prior_table'] == list(CHAPTERS[len(PRIOR_CHAPTERS):])
    assert [row['chapter'] for row in summary['chapters']] == list(CHAPTERS)
    assert summary['cues'] == 105 and summary['fps'] == 60
    assert summary['native_oracle_fact_rows_matched'] > 0
    folder = TIMELINE_OUT
    assert {p.stem.removesuffix('.timeline') for p in folder.glob('*.timeline.tsv')} == set(CHAPTERS)
    assert all((folder / f'{chapter}.{table}.tsv').is_file() for chapter in CHAPTERS for table in TABLES)
    compared = read(job('timeline') / 'prior-table-comparison.json')
    assert compared['prior_job'] == FAILED.name and compared['new_binary_sha256'] == Q.BIN_SHA256
    assert len(compared['tables']) == summary['prior_tables_compared'] == 12
    for row in compared['tables']:
        assert row['normalized_equal'] is True, 'Prior table differs: ' + row['table']
        assert sha(folder / row['table']) == row['new_sha256'], 'New table changed: ' + row['table']
        assert sha(PRIOR_OUT / row['table']) == row['prior_sha256'], 'Prior table changed: ' + row['table']
    seen = []
    total = 0
    for item in summary['chapters']:
        path = folder / (item['chapter'] + '.timeline.tsv')
        assert sha(path) == item['timeline_sha256']
        rows = [line.split('\t') for line in path.read_text().splitlines()]
        assert table_value(rows, 'chapter') == item['chapter'] and table_value(rows, 'fps') == '60'
        frames = int(table_value(rows, 'frames'))
        assert frames == item['frames'] and frames > 0
        cues = [row for row in rows if row[0] == 'line']
        assert [row[1] for row in cues] == item['cues']
        starts = [int(row[2]) for row in cues]
        assert starts == sorted(starts) and all(0 <= n < frames for n in starts)
        assert all(Path(row[4]).resolve() == (N / (row[1] + '.wav')).resolve() for row in cues)
        seen.extend(row[1] for row in cues)
        total += frames
    ordered = script_ids()
    assert seen == ordered and len(seen) == len(set(seen)) == 105
    assert total == summary['total_frames']
    return folder


def run_score():
    child_gate('score')
    check_qualified()
    folder = check_timeline()
    assert not SCORE_OUT.exists(), 'Preserve existing score outputs'
    subprocess.run([str(BIN), 'score', '--narration', str(N), '--timelines', str(folder), '--out', str(SCORE_OUT)],
                   cwd=R, check=True, timeout=SCORE_SECONDS)
    qa_folder = SCORE_OUT / 'stems/qa'
    assert {p.stem for p in qa_folder.glob('*.json')} == set(CHAPTERS)
    qa = {}
    for chapter in CHAPTERS:
        path = qa_folder / (chapter + '.json')
        report = read(path)
        assert report['chapter'] == chapter and report['passed'] is True, chapter
        assert report['metrics'] and all(m.get('pass') is True or m.get('pass') is None for m in report['metrics']), chapter
        qa[chapter] = sha(path)
    with (job('score') / 'score-summary.json').open('x') as handle:
        json.dump({'qa_sha256': qa, 'chapters': 15, 'cues': 105,
                   'scope': 'Native score QA only. Listening, actual pictures, fine sync and independent final AV review remain open.'}, handle, indent=2)
    print('SCORE PASS: all 15 registry chapter QA reports passed.', flush=True)


def check_score():
    Q.success(NAMES['score'])
    timeline = check_timeline()
    summary = read(job('score') / 'score-summary.json')
    assert list(summary['qa_sha256']) == list(CHAPTERS)
    for chapter, digest in summary['qa_sha256'].items():
        path = SCORE_OUT / 'stems/qa' / (chapter + '.json')
        report = read(path)
        assert sha(path) == digest and report['chapter'] == chapter and report['passed'] is True
        assert report['metrics'] and all(m['pass'] is True or m['pass'] is None for m in report['metrics'])
    return timeline


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


def run_preview():
    child_gate('preview')
    check_qualified()
    timeline = check_score()
    probe_tool = tool('ffprobe')
    assert not PREVIEW_OUT.exists(), 'Existing preview must be preserved'
    subprocess.run([str(BIN), 'render', 'all', '--res', '960x540', '--fps', '60', '--narration', str(N),
                    '--score', str(SCORE_OUT / 'stems'), '--out', str(PREVIEW_OUT), '--lint', '--crf', '18',
                    '--preset', 'veryfast'], cwd=R, check=True, timeout=PREVIEW_SECONDS)
    chapters = []
    assert {p.stem for p in PREVIEW_OUT.glob('*.mp4')} == set(CHAPTERS)
    for chapter in CHAPTERS:
        table = PREVIEW_OUT / (chapter + '.timeline.tsv')
        for name in ('timeline', 'cues'):
            rendered = PREVIEW_OUT / f'{chapter}.{name}.tsv'
            assert normalized(rendered) == normalized(timeline / rendered.name), 'Preview table differs: ' + rendered.name
        movie = PREVIEW_OUT / (chapter + '.mp4')
        probe = json.loads(subprocess.check_output([str(probe_tool), '-v', 'error', '-show_streams', '-of', 'json', str(movie)], timeout=120))
        video = [s for s in probe['streams'] if s['codec_type'] == 'video']
        audio = [s for s in probe['streams'] if s['codec_type'] == 'audio']
        assert len(video) == len(audio) == 1
        v = video[0]
        assert (v['width'], v['height'], v['avg_frame_rate']) == (960, 540, '60/1')
        frames = int(next(line.split('\t')[1] for line in normalized(table) if line.startswith('frames\t')))
        assert int(v['nb_frames']) == frames
        check_cue_log(chapter, frames, PREVIEW_OUT / (chapter + '.cuelog.tsv'), timeline / (chapter + '.cuelog.tsv'))
        layout = PREVIEW_OUT / (chapter + '.layout.tsv')
        assert layout.is_file(), 'Rendered lint report missing: ' + layout.name
        chapters.append({'chapter': chapter, 'frames': frames, 'mp4_sha256': sha(movie), 'timeline_sha256': sha(table),
                         'layout_sha256': sha(layout),
                         'layout_findings': len([line for line in layout.read_text().splitlines() if line.strip()])})
    with (job('preview') / 'preview-summary.json').open('x') as handle:
        json.dump({'chapters': chapters, 'res': '960x540', 'fps': 60, 'voice': 'ElevenLabs Frederick Surrey',
                   'binary_sha256': Q.BIN_SHA256, 'timeline_job': NAMES['timeline'], 'tables_equal_to_timeline': ['timeline', 'cues'],
                   'cue_log': 'all timeline rows unchanged, then only this chapter score rows, every field checked',
                   'rendered_lint': True, 'full_baseline_equivalence_established': False,
                   'scope': 'Narrated preview rendered and schedules compared; actual picture review, listening, fine sync and independent AV review remain open. Not a publication master.'}, handle, indent=2)


def layout_paths(chapter, required_folders, optional_folders=()):
    required = [folder / (chapter + '.layout.tsv') for folder in required_folders]
    for path in required:
        assert path.is_file(), 'Required rendered layout report missing: ' + str(path)
    optional = [folder / (chapter + '.layout.tsv') for folder in optional_folders]
    return required + [path for path in optional if path.is_file()]


def select(chapter, rows, frames, phrases, sentences, layouts):
    selected = {}
    def add(seconds, reason):
        frame = round(seconds * 60)
        if 0 <= frame < frames:
            selected.setdefault(frame, set()).add(reason)
    for sec in range(1, frames // 60, 5):
        add(sec, 'five-second chapter progression')
    add((frames - 1) / 60, 'last frame')
    for row in rows:
        if row[0] == 'line':
            cue, start, wav = row[1], int(row[2]) / 60, Path(row[4])
            assert sha(wav) == phrases[cue]['wav_sha256'], cue
            with wave.open(str(wav)) as handle:
                duration = handle.getnframes() / handle.getframerate()
            for delta, label in [(-.2, 'before'), (.2, 'start'), (duration / 2, 'middle'),
                                 (max(0, duration - .1), 'end')]:
                add(start + delta, cue + ' ' + label)
            for index, sentence in enumerate(sentences[cue]):
                add(start + sentence['start'] + .5, f'{cue} provisional sentence {index+1}')
            for mark in phrases[cue]['marks']:
                for delta in (-.2, .5, 1.5):
                    add(start + mark['seconds'] + delta, f"{cue} provisional phrase byte{mark['byte']} +{delta}")
        elif row[0] == 'breath':
            begin, duration = int(row[1]) / 60, float(row[2])
            for delta in (0, duration / 2, max(0, duration - .1), duration + .5):
                add(begin + delta, 'authored hold +' + str(delta))
    for path in layouts:
        for line in path.read_text().splitlines():
            if not line.strip():
                continue
            match = re.search(r'@([0-9]+(?:\.[0-9]+)?)s', line)
            assert match, (path, line)
            anchor = float(match[1])
            deltas = [-.2, 0, .2, 1, 3, 6, 10]
            persistent = re.search(r'still there at (\d+) later looks', line)
            if persistent:
                count = int(persistent[1])
                deltas += [1 + .2 * count, 2 + .2 * count]
                deltas += list(range(5, max(5, int(frames / 60 - anchor) + 1), 5))
            for delta in deltas:
                add(anchor + delta, f'{path.name} {line} +{delta}s')
    return [{'chapter': chapter, 'frame': frame, 'seconds': frame / 60,
             'reasons': sorted(reasons)} for frame, reasons in sorted(selected.items())]


def run_pictures():
    from PIL import Image, ImageDraw  # only the picture env has Pillow
    child_gate('pictures')
    check_qualified()
    Q.success(NAMES['preview'])
    assert not PICTURES_OUT.exists(), 'Preserve earlier review pictures'
    ffmpeg = tool('ffmpeg')
    PICTURES_OUT.mkdir()
    phrases = read(N / 'phrase-timings.json')['cues']
    sentences = read(N / 'report.json')['lines']
    summary = read(job('preview') / 'preview-summary.json')
    assert [row['chapter'] for row in summary['chapters']] == list(CHAPTERS)
    all_samples = []
    layout_sources = {}
    for chapter_info in summary['chapters']:
        chapter = chapter_info['chapter']
        movie, table = PREVIEW_OUT / (chapter + '.mp4'), PREVIEW_OUT / (chapter + '.timeline.tsv')
        assert sha(movie) == chapter_info['mp4_sha256'] and sha(table) == chapter_info['timeline_sha256']
        rows = [line.split('\t') for line in normalized(table)]
        layouts = layout_paths(chapter, [PREVIEW_OUT])
        layout_sources[chapter] = [str(path) for path in layouts]
        samples = select(chapter, rows, chapter_info['frames'], phrases, sentences, layouts)
        folder = PICTURES_OUT / chapter
        folder.mkdir()
        for index, sample in enumerate(samples):
            dst = folder / f'{index:04}.png'
            subprocess.run([str(ffmpeg), '-v', 'error', '-xerror', '-nostdin', '-threads', '2',
                            '-ss', str(sample['seconds']), '-i', str(movie), '-frames:v', '1',
                            '-threads', '2', '-n', str(dst)], check=True, timeout=90)
            sample.update(picture=str(dst), sha256=sha(dst))
        for first in range(0, len(samples), 6):
            sheet = Image.new('RGB', (1920, 1740), '#0f0f14')
            draw = ImageDraw.Draw(sheet)
            for slot, sample in enumerate(samples[first:first+6]):
                x, y = slot % 2 * 960, slot // 2 * 580
                with Image.open(sample['picture']) as im:
                    assert im.size == (960, 540)
                    sheet.paste(im, (x, y))
                draw.text((x+8, y+545), f"{first+slot:04} {chapter} {sample['seconds']:.3f}s", fill='white')
                draw.text((x+8, y+560), ' | '.join(sample['reasons'])[:150], fill='white')
            sheet.save(folder / f'sheet-{first//6:03}.png')
        all_samples += samples
        print(chapter, len(samples), 'pictures extracted; actual viewing pending', flush=True)
    with (PICTURES_OUT / 'extraction.json').open('x') as handle:
        json.dump({'samples': all_samples, 'pictures_viewed': False, 'layout_sources': layout_sources,
                   'timeline_lint_note': 'Rendered preview lint reports only; timeline-only mode installs no linter.',
                   'scope': 'Preview samples only; all full reason labels retained. Persistence looks can be nonconsecutive. Not exhaustive review, listening, fine sync or independent acceptance.'}, handle, indent=2)


def check_tools(stage):
    tool('python3')
    if stage in ('timeline', 'preview'):  # the binary encodes through ffmpeg in both
        encoders = subprocess.check_output([str(tool('ffmpeg')), '-hide_banner', '-encoders'], text=True, timeout=60)
        assert 'libx264' in encoders, 'ffmpeg has no libx264'
    if stage == 'preview':
        tool('ffprobe')
    if stage == 'pictures':
        tool('ffmpeg')
        Q.success(PICTURE_ENV)
        version = subprocess.check_output([str(PICTURE_PYTHON), '-I', '-B', '-c', 'import PIL; from PIL import Image, ImageDraw; print(PIL.__version__)'],
                                          cwd=R, text=True, timeout=120).strip()
        assert version == PILLOW, 'Picture env differs: ' + version


def check_upstream(stage):
    check_qualified()
    if stage == 'timeline':
        check_failed_run()
    elif stage == 'score':
        check_timeline()
    elif stage == 'preview':
        check_score()
    else:
        Q.success(NAMES['preview'])
        check_score()


def wait_for(stage):
    """Each controller writes its exit file after its job's receipt is final."""
    exits = [O / 'qualify-controller01.exit', *(exit_file(name) for name in UPSTREAM[stage])]
    for _ in range(WAIT_POLLS):
        codes = [path.read_text().strip() if path.exists() else '' for path in exits]
        failed = [path.name for path, code in zip(exits, codes) if code not in ('', '0')]
        if failed:
            raise RuntimeError(f'Upstream controller failed; nothing launched: {failed}')
        if all(code == '0' for code in codes):
            return
        time.sleep(20)
    raise RuntimeError('Wait expired; nothing launched')


def launch(stage):
    wait_for(stage)
    check_pins()
    check_upstream(stage)
    check_tools(stage)
    assert not job(stage).exists() and not manifest(stage).exists(), f'Duplicate {stage} refused'
    files = stage_files(stage)
    with manifest(stage).open('x') as handle:
        json.dump({'files': [{'path': path, 'sha256': digest} for path, digest in sorted(files.items())]}, handle, indent=2)
    Q.guard_policy()
    subprocess.run(guard_command(stage), cwd=R, check=True)
    Q.success(NAMES[stage])


def preflight(stage):
    """Read-only, and valid before the upstream jobs have finished."""
    check_pins()
    files = base_rows()
    check_failed_run()
    Q.guard_policy()
    check_tools(stage)
    assert not job(stage).exists() and not manifest(stage).exists(), f'Duplicate {stage} refused'
    print(json.dumps({'stage': stage, 'job': NAMES[stage], 'root': str(R), 'frozen_files': len(files), 'launched': False}))


if __name__ == '__main__':
    assert len(sys.argv) == 3 and sys.argv[1] in ('--check', '--launch', '--run') and sys.argv[2] in NAMES, __doc__
    mode, name = sys.argv[1:]
    if mode == '--check':
        preflight(name)
    elif mode == '--launch':
        launch(name)
    else:
        {'timeline': run_timeline, 'score': run_score, 'preview': run_preview, 'pictures': run_pictures}[name]()
