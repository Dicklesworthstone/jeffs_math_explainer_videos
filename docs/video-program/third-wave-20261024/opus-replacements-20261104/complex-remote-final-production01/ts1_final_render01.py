"""Guarded serial 4K chapters on ts1, from chain02-n2. Requires a real, hash-bound preview review.

WhiteSwan's final_render_optimized01.py on lane O; diff the two to audit it, and HANDOFF.md lists
every change. It runs the qualified Linux binary on the derived narration folder and refuses to
import a chain file whose bytes changed. There is no flag that skips the review.

    --check                          read-only: pins, tools, transferred files, job states
    --check-review                   read-only: the review and everything it binds
    --all [--retry CHAPTER=rN]...    every chapter not yet done, one guarded job at a time
    --run CHAPTER JOB                the guard's child; never call it by hand

A chapter whose job passed and whose hashes hold is skipped. A failed attempt is never retried by
itself: --all stops at it, and only a named --retry goes on, after moving that attempt's outputs
aside. Nothing is deleted.

This helper is prepared only; it never invents a review or starts on import.
Mastering, final pictures, ASR, listening and publication remain separate.
"""
import fcntl
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import sys
import time
import wave

HERE = Path(__file__).resolve().parent
CHAIN = HERE.parent / 'complex-remote-ts1-pipeline01/chain02-n2/ts1_chain02_n2.py'
CHAIN_SHA256 = '64502efaea1bffb1a7e852f43d2679ea9baca29d0d9b622041d7205dd6b68a45'  # seal 625d162d
with CHAIN.open('rb') as handle:
    assert hashlib.file_digest(handle, 'sha256').hexdigest() == CHAIN_SHA256, 'Sealed chain02-n2 changed'
SPEC = importlib.util.spec_from_file_location('ts1_chain02_n2', CHAIN)
L = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(L)
M, Q = L.M, L.Q
R, O, JOBS, C, BIN, read, sha = L.R, L.O, L.JOBS, L.C, L.BIN, L.read, L.sha
ME = R / Path(__file__).resolve().relative_to(Q.PHYSICAL)
START = ME.parent / 'start-final-render01.sh'
TRANSFERRED = ME.parent / 'transfer-files01.json'
STAGE = O / 'final-production01'  # controllers, manifests and the review
OUT = JOBS / 'complex-remote-render-work01'
ASIDE = JOBS / 'complex-remote-render-aside01'
REVIEW = STAGE / 'preview-pictures-viewed01.json'
LANE_LOCK = O / 'expanded-native-lane.lock'
VOICE = 'ElevenLabs Frederick Surrey'
SUCCESS = 'RENDER PASS: 4K60 chapter; frame and cue schedule equal the timeline; final AV, pictures and listening remain.'
HEAD_BYTES = 16 << 20  # x264 writes its settings into the stream, near the front


def attempts(base):
    return [base, *(f'{base}-r{n}' for n in range(2, 10))]


def used(base):
    names = attempts(base)
    found = [name for name in names if (JOBS / name).exists()]
    assert found == names[:len(found)], f'Attempts have a gap: {found}'
    return found


def newest(base):
    return (used(base) or [base])[-1]


def passed(name):
    try:
        return Q.success(name)
    except (AssertionError, OSError, json.JSONDecodeError):
        return False


def next_attempt(base, label, retry):
    """(job to start, failed attempt it replaces), or (None, last) when the last attempt passed."""
    found = used(base)
    if not found:
        assert retry is None, f'{label} has no attempt to retry'
        return base, None
    last = found[-1]
    if passed(last):
        assert retry is None, f'{last} passed; there is nothing to retry'
        return None, last
    names = attempts(base)
    assert len(found) < len(names), f'{label} has no attempt names left'
    following = names[len(found)]
    suffix = following.rsplit('-', 1)[1]
    assert retry == suffix, f'{last} did not pass. Preserve it, read its log, then pass --retry {label}={suffix}'
    return following, last


def parse_retries(args, labels):
    assert len(args) % 2 == 0 and args[::2] == ['--retry'] * (len(args) // 2), 'Expected --retry NAME=rN'
    retries = {}
    for value in args[1::2]:
        label, _, attempt = value.partition('=')
        assert label in labels and label not in retries and re.fullmatch(r'r[2-9]', attempt), '--retry ' + value
        retries[label] = attempt
    return retries


def last_line(path):
    lines = [line for line in path.read_text(errors='replace').splitlines() if line.strip()]
    return lines[-1] if lines else None


def job_base(chapter):
    assert chapter in M.CHAPTERS
    return 'complex-remote-render-' + chapter + '-01'


def job(chapter):
    return newest(job_base(chapter))


def manifest(name):
    return STAGE / (name + '-inputs.json')


def child_tail(chapter, name):
    return ['-I', '-B', str(ME), '--run', chapter, name]


def live_receipt(name):
    """This job's receipt, while the guard owns it. The guard rewrites it in place just after it
    starts the job, so a read can land inside that write. Only a parse failure is read again."""
    for _ in range(100):
        try:
            return read(JOBS / name / 'resource-receipt.json')
        except json.JSONDecodeError:
            time.sleep(0.1)
    raise AssertionError('The guard receipt never settled: ' + name)


def gate(name, command, inputs):
    """The guard is running exactly this child on exactly these files, with these tools."""
    d = live_receipt(name)
    assert d.get('admitted') is True and 'exit' not in d
    assert d['command'] == command
    assert d['policy'] == Q.POLICY, 'Guard policy changed'
    rows = read(inputs)['files']
    for row in rows:
        assert sha(row['path']) == row['sha256'], row['path']
    bound = {row['path'] for row in rows}
    for tool in ('python3', 'ffmpeg', 'ffprobe'):
        assert str(M.tool(tool)) in bound, f'Child resolves a {tool} outside the manifest'


def check_transferred():
    """The files the 4K stages need that no earlier ts1 transfer list carried."""
    rows = read(TRANSFERRED)['files']
    assert rows
    for row in rows:
        path = R / row['path']
        assert path.is_file() and sha(path) == row['sha256'], 'Transfer this file to ts1: ' + row['path']
    return rows


def check_review():
    assert REVIEW.is_file(), 'Actual narrated preview review is missing; no final render authorized by this helper'
    L.check_qualified()
    L.check_derived()
    assert Q.success(M.NAMES['preview']) and Q.success(M.NAMES['pictures'])
    M.check_score()
    review = read(REVIEW)
    assert review['chapters'] == list(M.CHAPTERS)
    assert review['unresolved_visual_defects'] == []
    # pictures_viewed covers the preview sheets only. It asserts nothing about listening or final pictures.
    assert review['pictures_viewed'] is True and review['reviewer'].strip(), 'Review names no viewer'
    assert review['review_scope'] in ('source-author', 'independent'), 'Review scope must be explicit'
    folder = M.PICTURES_OUT
    sheets = sorted(folder.glob('*/sheet-*.png'))
    assert sheets and {p.parent.name for p in sheets} == set(M.CHAPTERS)
    samples = {}
    for sample in read(folder / 'extraction.json')['samples']:
        samples[sample['chapter']] = samples.get(sample['chapter'], 0) + 1
    expected = {folder / chapter / f'sheet-{index:03}.png' for chapter, count in samples.items()
                for index in range((count + 5) // 6)}
    assert set(sheets) == expected, 'Sheets differ from the extraction'
    assert review['sheet_sha256'] == {str(p): sha(p) for p in sheets}
    assert review['extraction_sha256'] == sha(folder / 'extraction.json')
    summary_path = M.job('preview') / 'preview-summary.json'
    assert review['preview_summary_sha256'] == sha(summary_path)
    summary = read(summary_path)
    assert [row['chapter'] for row in summary['chapters']] == list(M.CHAPTERS)
    assert summary['voice'] == VOICE and summary['binary_sha256'] == Q.BIN_SHA256, 'Preview is another film'
    for row in summary['chapters']:
        for suffix, key in [('.mp4', 'mp4_sha256'), ('.timeline.tsv', 'timeline_sha256'),
                            ('.layout.tsv', 'layout_sha256')]:
            assert sha(M.PREVIEW_OUT / (row['chapter'] + suffix)) == row[key]
    return review


def captured_soundtrack(work, chapter, frames):
    # Match the native master's exact selection: last consecutive numbered capture.
    folder = work / 'soundtrack' / chapter
    count = 0
    while (folder / f'soundtrack_{count}.wav').exists():
        count += 1
    assert count, f'{chapter}: missing soundtrack capture'
    path = folder / f'soundtrack_{count-1}.wav'
    samples = frames * 800  # exact 48kHz/60fps
    with wave.open(str(path)) as wav:
        assert (wav.getframerate(), wav.getnchannels(), wav.getsampwidth(), wav.getnframes()) == (48000, 2, 2, samples)
        remaining = samples
        while remaining:
            requested = min(remaining, 48000)
            assert len(wav.readframes(requested)) == requested * 4, 'Truncated soundtrack data'
            remaining -= requested
    return {'path': str(path), 'sha256': sha(path), 'samples': samples, 'captures': count}


def movie_facts(path, frames):
    """Read from the file. ts1's ffmpeg is not the macOS one, so x264's own record proves CRF 16."""
    probe = json.loads(subprocess.check_output([str(M.tool('ffprobe')), '-v', 'error', '-show_streams', '-of', 'json', str(path)], timeout=120))
    video = [s for s in probe['streams'] if s['codec_type'] == 'video']
    audio = [s for s in probe['streams'] if s['codec_type'] == 'audio']
    assert len(video) == len(audio) == 1
    v, a = video[0], audio[0]
    assert (v['width'], v['height'], v['avg_frame_rate'], int(v['nb_frames'])) == (3840, 2160, '60/1', frames)
    assert (v['codec_name'], v['pix_fmt'], a['codec_name'], a['sample_rate'], a['channels']) == ('h264', 'yuv420p', 'aac', '48000', 2)
    with path.open('rb') as handle:
        head = handle.read(HEAD_BYTES)
    start = head.find(b'x264 - core')
    settings = head[start:start + 2048].split(b'\0')[0] if start >= 0 else b''
    assert b' rc=crf' in settings and b' crf=16.0' in settings, 'No x264 record of CRF 16: ' + path.name
    return settings.decode('ascii', 'replace')


def render(chapter, name):
    assert name == job(chapter), 'This is not the newest attempt: ' + name
    gate(name, [sys.executable, *child_tail(chapter, name)], manifest(name))
    check_review()
    movie = OUT / (chapter + '.mp4')
    assert not movie.exists(), 'Existing render preserved; name a retry after failure'
    OUT.mkdir(exist_ok=True)
    subprocess.run([str(BIN), 'render', chapter, '--res', '3840x2160', '--fps', '60',
                    '--narration', str(M.N), '--score', str(M.SCORE_OUT / 'stems'), '--out', str(OUT),
                    '--lint', '--crf', '16', '--preset', 'veryfast'], cwd=R, check=True, timeout=43200)
    table = OUT / (chapter + '.timeline.tsv')
    timeline = M.TIMELINE_OUT
    for kind in ('timeline', 'cues'):
        rendered = OUT / f'{chapter}.{kind}.tsv'
        assert M.normalized(rendered) == M.normalized(timeline / rendered.name), 'Final table differs: ' + rendered.name
    frames = int(next(line.split('\t')[1] for line in M.normalized(table) if line.startswith('frames\t')))
    M.check_cue_log(chapter, frames, OUT / (chapter + '.cuelog.tsv'), timeline / (chapter + '.cuelog.tsv'))
    assert (OUT / (chapter + '.layout.tsv')).is_file(), 'Rendered lint report missing: ' + chapter
    encoder = movie_facts(movie, frames)
    track = captured_soundtrack(OUT, chapter, frames)
    paths = [movie, table, OUT / (chapter + '.layout.tsv'), OUT / (chapter + '.cuelog.tsv')]
    with (JOBS / name / 'render-summary.json').open('x') as handle:
        json.dump({'chapter': chapter, 'frames': frames, 'fps': 60, 'soundtrack': track, 'encoder': encoder,
                   'binary_sha256': Q.BIN_SHA256, 'narration': str(M.N), 'host': 'ts1',
                   'files': [{'path': str(p), 'sha256': sha(p)} for p in paths],
                   'scope': '4K chapter render with exact schedule and complete capture. Final AV, picture review, listening and fine sync remain open.'}, handle, indent=2)
    print(SUCCESS, flush=True)


def verify_chapter(chapter, name):
    """A chapter counts only if its job, its frozen inputs and its files all still hold."""
    assert Q.success(name), name
    assert read(JOBS / name / 'resource-receipt.json')['command'][1:] == child_tail(chapter, name), name
    for row in read(manifest(name))['files']:
        assert sha(row['path']) == row['sha256'], row['path']
    assert last_line(JOBS / name / 'command.log') == SUCCESS, 'No success line: ' + name
    summary = read(JOBS / name / 'render-summary.json')
    assert summary['chapter'] == chapter and summary['binary_sha256'] == Q.BIN_SHA256
    for row in summary['files']:
        assert sha(row['path']) == row['sha256'], row['path']
    assert captured_soundtrack(OUT, chapter, summary['frames']) == summary['soundtrack']
    return summary


def frozen_base():
    """The last chain manifest, then what it could not hold: the pictures, the review and this bundle."""
    base = {}
    for row in read(M.manifest('pictures'))['files']:
        assert sha(row['path']) == row['sha256'], row['path']
        base[row['path']] = row['sha256']
    paths = [ME, START, TRANSFERRED, REVIEW, M.manifest('pictures'), Q.GUARD, Q.PRESERVED_GUARD, M.tool('python3'),
             *(Path(row['path']) for row in read(O / 'guard-pins01.json')['files']),
             *(R / row['path'] for row in check_transferred()), *Q.job_files(M.NAMES['pictures'])]
    for path in paths:
        base[str(path)] = sha(path)
    return base


def admission():
    """The guard's own test, first: a job it refuses keeps its folder, so its name is used up."""
    need = Q.POLICY['admission_gib']
    assert shutil.which('sbh'), 'sbh is not on PATH; the guard needs it'
    run = subprocess.run(['sbh', 'check', '--target-free', '0.2', '--need', f'{need}G', '--json'], cwd=R, text=True,
                         capture_output=True, timeout=45)
    assert run.returncode == 0, f'sbh refuses admission: {run.stdout[-200:]}{run.stderr[-200:]}'
    disk = os.statvfs(R)
    assert disk.f_bavail * disk.f_frsize >= need * 1024 ** 3, f'Under {need} GiB free; the guard would refuse'


def write_new(path, data):
    """A new file, or the identical one an interrupted launch left. Anything else refuses."""
    if path.exists():
        assert path.read_bytes() == data, f'{path} exists with other bytes: inspect before resuming'
        return
    with path.open('xb') as handle:
        handle.write(data)


def write_once(path, files):
    write_new(path, json.dumps({'files': [{'path': p, 'sha256': h} for p, h in sorted(files.items())]}, indent=2).encode())


def guard_run(name, inputs, command):
    """One guarded job. A refused launch costs no name; a failed job stays preserved."""
    admission()
    subprocess.run([sys.executable, '-I', '-B', str(Q.GUARD), '--phase', 'native', '--job', name,
                    '--inputs', str(inputs), '--', *command], cwd=R, check=True)
    assert Q.success(name)


def lane_busy():
    if not LANE_LOCK.is_file():
        return False
    with LANE_LOCK.open('r') as handle:
        try:
            fcntl.flock(handle, fcntl.LOCK_EX | fcntl.LOCK_NB)
        except BlockingIOError:
            return True
        fcntl.flock(handle, fcntl.LOCK_UN)
    return False


def set_aside(name, paths):
    """A named retry's only move: the failed attempt's outputs, kept whole, still under the guard's caps."""
    assert not lane_busy(), 'The lane is busy; a retry moves files only when it is free'
    paths = [path for path in paths if path.exists()]
    if not paths:
        return
    folder = ASIDE / name
    folder.mkdir(parents=True)
    with (folder / 'moved.json').open('x') as handle:
        json.dump([{'from': str(p), 'sha256': sha(p) if p.is_file() else None} for p in paths], handle, indent=2)
    for path in paths:
        path.rename(folder / path.name)
    print(f'ASIDE {name}: {len(paths)} paths in {folder}', flush=True)


def leftovers(chapter):
    return [*sorted(OUT.glob(chapter + '.*')), OUT / 'soundtrack' / chapter]


def plan(chapter, retry):
    name, last = next_attempt(job_base(chapter), chapter, retry)
    if name is None:
        verify_chapter(chapter, last)
    return name, last


def launch_all(retries):
    STAGE.mkdir(exist_ok=True)
    with (STAGE / 'render.lock').open('a') as lock:
        try:
            fcntl.flock(lock, fcntl.LOCK_EX | fcntl.LOCK_NB)
        except BlockingIOError:
            raise SystemExit('Another render driver holds this lane') from None
        check_review()
        Q.guard_policy()
        base = frozen_base()
        plans = {chapter: plan(chapter, retries.get(chapter)) for chapter in M.CHAPTERS}
        for chapter, (name, last) in plans.items():
            if name is None:
                print(f'DONE {last}: passed, hashes hold, not launched', flush=True)
                continue
            if last is not None:
                set_aside(last, leftovers(chapter))
            write_once(manifest(name), base)
            guard_run(name, manifest(name), [sys.executable, *child_tail(chapter, name)])
            verify_chapter(chapter, name)
    print('RENDER PASS: 15 chapters at 4K60; final AV, pictures and listening remain.', flush=True)


def preflight():
    """Read-only and valid before the chain has finished. Hashes no film file and samples no disk."""
    Q.guard_policy()
    rows = check_transferred()
    for tool in ('python3', 'ffmpeg', 'ffprobe', 'sbh'):
        M.tool(tool)
    encoders = subprocess.check_output([str(M.tool('ffmpeg')), '-hide_banner', '-encoders'], text=True, timeout=60)
    assert 'libx264' in encoders, 'ffmpeg has no libx264'
    return {'chapter_jobs': len(M.CHAPTERS), 'transferred_files': len(rows), 'review_present': REVIEW.is_file(),
            'attempts_used': {chapter: len(used(job_base(chapter))) for chapter in M.CHAPTERS}, 'launched': False}


if __name__ == '__main__':
    if sys.argv[1:] == ['--check']:
        print(json.dumps(preflight()))
    elif sys.argv[1:] == ['--check-review']:
        check_review()
    elif sys.argv[1:2] == ['--all']:
        launch_all(parse_retries(sys.argv[2:], M.CHAPTERS))
    elif len(sys.argv) == 4 and sys.argv[1] == '--run':
        render(sys.argv[2], sys.argv[3])
    else:
        raise SystemExit('Use --check, --check-review, --all, or --run CHAPTER JOB. No implicit launch.')
