"""Verify the encoded complex-analysis master on ts1 and extract actual final review frames.

WhiteSwan's final_verify_optimized01.py on lane O; diff the two to audit it. The child runs under the
picture env's Python, because the shared sheet maker needs Pillow and ts1's python3 has none. The
preview review does not carry over: the final pictures this extracts are unviewed, and
verification.json says so.

    --pins                      read-only: pins and tools, before any master exists
    --check                     read-only: the full plan; needs the finished master
    --launch [--retry verify=rN]    one guarded job, then its postcheck
    --postcheck JOB             after the guard's final receipt: every recorded result still holds
    --run JOB                   the guard's child; never call it by hand
"""
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import subprocess
import sys

HERE = Path(__file__).resolve().parent


def module(name, path):
    spec = importlib.util.spec_from_file_location(name, path)
    loaded = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(loaded)
    return loaded


MASTER_HELPER = HERE / 'ts1_final_master01.py'
SHARED = HERE.parent / 'logarithm-stage09-final-verification02/verify_stage09_final.py'
IMPORTED_SHA256 = {
    MASTER_HELPER: '6ed3cb2ae08ec2920badf4c87c0e808d7ea0070b751d9732e3aaa85ed1b6cdb2',
    SHARED: 'f266a2315e4a9af247c648da416881f3a2999e2682e9c611ff82ca3727b70b94',
}
for imported, digest in IMPORTED_SHA256.items():
    with imported.open('rb') as handle:
        assert hashlib.file_digest(handle, 'sha256').hexdigest() == digest, 'Imported helper changed: ' + imported.name
K = module('complex_master', MASTER_HELPER)
F, L, M, Q = K.F, K.L, K.M, K.Q
R, C, JOBS, read, sha = F.R, F.C, F.JOBS, F.read, F.sha
V = module('qualified_media_checks', SHARED)
ME = R / Path(__file__).resolve().relative_to(Q.PHYSICAL)
START = ME.parent / 'start-final-verify01.sh'
STRICT = R / V.STRICT_REL
SCREEN = R / V.SCREEN_REL
JOB = 'complex-remote-final-verify01'
SUCCESS = 'FINAL VERIFY PASS: strict video check, frame count, A/V length and exact captions passed; every sheet still needs actual viewing.'


def job():
    return F.newest(JOB)


def manifest(name):
    return F.STAGE / (name + '-inputs.json')


def child_command(name):
    return [str(M.PICTURE_PYTHON), '-I', '-B', str(ME), '--run', name]


def plan():
    name = K.job()
    assert F.passed(name), 'Stable complex master required'
    assert F.last_line(JOBS / name / 'command.log') == K.SUCCESS, 'No success line: ' + name
    summary = read(JOBS / name / 'master-summary.json')
    F.same_source(summary)
    captures, total = K.check_chapters()
    assert captures == summary['soundtracks'] and total == summary['frames']
    for value in summary['outputs'].values():
        assert sha(Path(value['path'])) == value['sha256']
    for p in (STRICT, SCREEN):
        assert sha(p) == V.PINS[str(p.relative_to(R))]
    script = dict(line.split('\t', 1) for line in (C / 'script.tsv').read_text().splitlines()
                  if line.strip() and not line.startswith('#'))
    assert len(script) == 105
    report = read(M.N / 'report.json')
    phrases = read(M.N / 'phrase-timings.json')['cues']
    timeline = M.TIMELINE_OUT
    placements, chapters, samples, offset = [], [], [], 0
    layout_sources = {}
    for chapter in M.CHAPTERS:
        table = F.OUT / (chapter + '.timeline.tsv')
        rows = [line.split('\t') for line in M.normalized(table)]
        frames = int(next(row[1] for row in rows if row[0] == 'frames'))
        assert next(row[1] for row in rows if row[0] == 'fps') == '60'
        assert M.normalized(table) == M.normalized(timeline / table.name)
        for row in rows:
            if row[0] == 'line':
                placements.append({'cue': row[1], 'chapter': chapter, 'start_frame': offset + int(row[2]),
                                   'duration_s': V.wav_seconds(row[4])})
        layouts = M.layout_paths(chapter, [F.OUT, M.PREVIEW_OUT], [timeline])
        layout_sources[chapter] = [str(path) for path in layouts]
        selected = M.select(chapter, rows, frames, phrases, report['lines'], layouts)
        for item in selected:
            item['local_frame'] = item['frame']
            item['frame'] += offset
            item['seconds'] = item['frame'] / 60
            item['chapter'] = chapter
        samples += selected
        chapters.append({'chapter': chapter, 'start_frame': offset, 'frames': frames})
        offset += frames
    assert offset == total and [p['cue'] for p in placements] == list(script)
    assert len({s['frame'] for s in samples}) == len(samples)
    assert all(0 <= s['frame'] < total for s in samples)
    captions = V.check_captions(K.OUTPUTS['srt'].read_text(), K.OUTPUTS['vtt'].read_text(),
                                script, report, placements, total)
    return {'frames': total, 'chapters': chapters, 'placements': placements, 'master_job': name,
            'samples': samples, 'captions': captions, 'outputs': summary['outputs'],
            'layout_sources': layout_sources}


def run(name):
    assert name == job(), 'This is not the newest attempt: ' + name
    F.gate(name, child_command(name), manifest(name))
    out = JOBS / name
    cfg = plan()
    movie = str(K.OUTPUTS['master'])
    ffmpeg, ffprobe = str(M.tool('ffmpeg')), str(M.tool('ffprobe'))
    (out / 'decode-logs').mkdir()
    env = dict(os.environ, **V.EXPECT, TMPDIR=str(out / 'decode-logs'))
    strict_log = out / 'strict-video-verification.log'
    with strict_log.open('x') as log:
        subprocess.run(['bash', str(STRICT), movie], env=env, stdout=log, stderr=subprocess.STDOUT,
                       cwd=R, check=True, timeout=14400)
    text = strict_log.read_text()
    assert text.splitlines()[-1] == 'VERIFY PASS'
    decodes = V.decode_lines(text)
    assert V.software_decode_clean(decodes)
    probe = json.loads(subprocess.check_output([ffprobe, '-v', 'error', '-show_streams',
                                               '-show_format', '-of', 'json', movie], timeout=600))
    packets = int(subprocess.check_output([ffprobe, '-v', 'error', '-select_streams', 'V:0',
                                          '-count_packets', '-show_entries', 'stream=nb_read_packets',
                                          '-of', 'csv=p=0', movie], timeout=3600).strip())
    media = V.media_facts(probe, packets, cfg['frames'])
    loudness_log = out / 'encoded-loudness.log'
    with loudness_log.open('x') as log:
        subprocess.run([ffmpeg, '-nostdin', '-hide_banner', '-nostats', '-i', movie,
                        '-vn', '-af', 'ebur128=peak=true', '-f', 'null', '-'],
                       stdout=log, stderr=subprocess.STDOUT, check=True, timeout=14400)
    loudness = V.loudness_summary(loudness_log.read_text())
    subprocess.run([sys.executable, '-I', '-B', str(SCREEN), str(K.OUTPUTS['srt']),
                    str(cfg['frames']), '60', str(out / 'caption-screen.json')], check=True, timeout=600)
    pictures = []
    for index, sample in enumerate(cfg['samples']):
        path = out / f'frame-{index:04}.png'
        assert not path.exists()
        subprocess.run([ffmpeg, '-nostdin', '-n', '-v', 'error', '-xerror', '-threads', '2',
                        '-ss', f"{sample['seconds']:.9f}", '-i', movie, '-frames:v', '1',
                        '-vf', 'scale=960:540', '-threads', '2', str(path)], check=True, timeout=180)
        pictures.append({**sample, 'path': str(path), 'sha256': sha(path)})
    sheets = V.make_sheets(pictures, out)
    review = read(F.REVIEW)
    with (out / 'verification.json').open('x') as handle:
        json.dump({'video': movie, 'sha256': cfg['outputs']['master']['sha256'], 'frames': cfg['frames'],
                   'fps': 60, 'chapters': cfg['chapters'], 'placements': cfg['placements'],
                   'captions': cfg['captions'], 'media': media, 'full_decode': decodes,
                   'encoded_loudness': loudness, 'strict_video_notes': [line for line in text.splitlines() if line.startswith('note:')],
                   'caption_screen': read(out / 'caption-screen.json'),
                   'pictures': pictures, 'sheets': sheets, 'sheet_sha256': {p: sha(Path(p)) for p in sheets},
                   'layout_sources': cfg['layout_sources'], 'host': 'ts1', 'voice': F.VOICE,
                   'pictures_viewed': False, 'binary_sha256': Q.BIN_SHA256, 'timeline_job': M.NAMES['timeline'],
                   'master_job': cfg['master_job'], 'narration': str(M.N), **F.narration_source(),
                   'full_baseline_equivalence_established': False,
                   'preview_review': {'path': str(F.REVIEW), 'sha256': sha(F.REVIEW), 'review_scope': review['review_scope'],
                                      'covers': 'Preview sheets only. Not these final pictures, not listening.'},
                   'scope': 'Encoded technical verification and selected final pictures only. The preview review does not carry over to these pictures. Caption reading flags require triage. Actual picture viewing, continuous listening, fine sync, final ASR and independent AV acceptance remain open. No upload.'}, handle, indent=2)
    print(f'{len(pictures)} pictures in {len(sheets)} sheets.', flush=True)
    print(SUCCESS, flush=True)


def launch(retries):
    plan()
    Q.guard_policy()
    M.check_tools('pictures')
    name, _ = F.next_attempt(JOB, 'verify', retries.get('verify'))
    assert name, 'Duplicate final verification refused'
    master = K.job()
    files = {}
    for row in read(K.manifest(master))['files']:
        assert sha(row['path']) == row['sha256'], row['path']
        files[row['path']] = row['sha256']
    paths = [ME, START, K.ME, F.ME, R / SHARED.relative_to(Q.PHYSICAL), STRICT, SCREEN, M.tool('ffmpeg'), M.tool('ffprobe'),
             K.manifest(master), JOBS / master / 'resource-receipt.json', JOBS / master / 'master-summary.json', *K.OUTPUTS.values()]
    for path in paths:
        files[str(path)] = sha(path)
    F.write_once(manifest(name), files)
    F.guard_run(name, manifest(name), child_command(name))
    print(json.dumps(postcheck(name)), flush=True)


def postcheck(name):
    """After the guard's final receipt: this job and everything it recorded still hold. Writes one record."""
    assert Q.success(name), name
    out = JOBS / name
    d = read(out / 'resource-receipt.json')
    assert d['command'] == child_command(name) and d['phase'] == 'native', 'Another command: ' + name
    assert d['inputs_sha256'] == sha(manifest(name)), 'Manifest changed: ' + name
    for row in read(manifest(name))['files']:
        assert sha(row['path']) == row['sha256'], row['path']
    assert F.last_line(out / 'command.log') == SUCCESS, 'No success line: ' + name
    v = read(out / 'verification.json')
    master = v['master_job']
    assert master == K.job() and Q.success(master), f'Another master: {master}'
    s = read(JOBS / master / 'master-summary.json')
    outputs = {key: value['sha256'] for key, value in s['outputs'].items()}
    assert {key: sha(path) for key, path in K.OUTPUTS.items()} == outputs, 'Master or captions changed'
    assert v['video'] == str(K.OUTPUTS['master']) and v['sha256'] == outputs['master'], 'Verified another master'
    assert (v['frames'], v['fps']) == (s['frames'], 60) and [row['chapter'] for row in v['chapters']] == list(M.CHAPTERS)
    assert v['binary_sha256'] == s['binary_sha256'] == Q.BIN_SHA256, 'Another binary'
    F.same_source(v)
    F.same_source(s)
    assert all(sha(p['path']) == p['sha256'] for p in v['pictures']), 'A picture changed'
    assert v['sheets'] and len(v['sheets']) == (len(v['pictures']) + 3) // 4, 'Sheets differ from the pictures'
    assert v['sheet_sha256'] == {p: sha(Path(p)) for p in v['sheets']}, 'A sheet changed'
    strict = (out / 'strict-video-verification.log').read_text()
    assert strict.splitlines()[-1] == 'VERIFY PASS' and v['full_decode'] == V.decode_lines(strict), 'Strict log differs'
    assert V.software_decode_clean(v['full_decode']), 'No clean software decode'
    assert read(out / 'caption-screen.json')['srt_sha256'] == outputs['srt'], 'Caption screen read another SRT'
    assert v['pictures_viewed'] is False and v['full_baseline_equivalence_established'] is False, 'A claim changed'
    record = {'job': name, 'master_job': master, 'master_sha256': outputs['master'], 'srt_sha256': outputs['srt'], 'vtt_sha256': outputs['vtt'],
              'derived_from': v['derived_from'], 'derive_summary_sha256': v['derive_summary_sha256'],
              'verification_sha256': sha(out / 'verification.json'), 'frames': v['frames'], 'pictures': len(v['pictures']), 'sheets': len(v['sheets']),
              'pictures_viewed': False, 'scope': 'Technical records only. Actual picture viewing, listening, fine sync, final ASR and independent AV acceptance remain open.'}
    F.write_new(F.STAGE / (name + '-postcheck.json'), (json.dumps(record, indent=2) + '\n').encode())
    return record


def preflight():
    """The pins, before any master exists. --check is the full plan and needs the finished master."""
    for p in (STRICT, SCREEN):
        assert sha(p) == V.PINS[str(p.relative_to(R))]
    Q.guard_policy()
    return {'job': job(), 'attempts_used': len(F.used(JOB)), 'pinned_helpers': len(IMPORTED_SHA256), 'launched': False}


if __name__ == '__main__':
    if sys.argv[1:] == ['--pins']:
        print(json.dumps(preflight()))
    elif sys.argv[1:] == ['--check']:
        plan()
    elif sys.argv[1:2] == ['--launch']:
        launch(F.parse_retries(sys.argv[2:], ['verify']))
    elif len(sys.argv) == 3 and sys.argv[1] == '--postcheck':
        print(json.dumps(postcheck(sys.argv[2])))
    elif len(sys.argv) == 3 and sys.argv[1] == '--run':
        run(sys.argv[2])
    else:
        raise SystemExit('Use --pins, --check, --launch, --postcheck JOB, or --run JOB. No implicit launch.')
