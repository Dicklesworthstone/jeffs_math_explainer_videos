"""Verify the encoded complex-analysis master and extract actual final review frames.

Root's final-verify01.py with new names and pins; diff the two to audit it. The preview review does
not carry over: the final pictures this extracts are unviewed, and verification.json says so.
"""
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import subprocess
import sys

O = Path(__file__).resolve().parent


def module(name, path):
    spec = importlib.util.spec_from_file_location(name, path)
    loaded = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(loaded)
    return loaded


MASTER_HELPER = O / 'final_master_optimized01.py'
SHARED = O.parent / 'logarithm-stage09-final-verification02/verify_stage09_final.py'
IMPORTED_SHA256 = {
    MASTER_HELPER: 'ca2ea33a027b4396817eee75501ab8fc4d79c5039de954659a6de0e85e6ca841',
    SHARED: 'f266a2315e4a9af247c648da416881f3a2999e2682e9c611ff82ca3727b70b94',
}
for imported, digest in IMPORTED_SHA256.items():
    with imported.open('rb') as handle:
        assert hashlib.file_digest(handle, 'sha256').hexdigest() == digest, 'Imported helper changed: ' + imported.name
M = module('complex_master', MASTER_HELPER)
T, F = M.T, M.F
X = F.X  # the pinned pictures helper, for its picture selection
V = module('qualified_media_checks', SHARED)
STRICT = T.R / V.STRICT_REL
SCREEN = T.R / V.SCREEN_REL
JOB = 'complex-stage06-optimized-final-verify01'
OUT = T.B / JOB
MANIFEST = F.STAGE / 'optimized-final-verify-inputs01.json'


def plan():
    assert T.success(M.JOB), 'Stable complex master required'
    summary = T.read(T.B / M.JOB / 'master-summary.json')
    captures, total = M.check_chapters()
    assert captures == summary['soundtracks'] and total == summary['frames']
    for value in summary['outputs'].values():
        assert T.sha(Path(value['path'])) == value['sha256']
    for p in (STRICT, SCREEN):
        assert T.sha(p) == V.PINS[str(p.relative_to(T.R))]
    script = dict(line.split('\t', 1) for line in (T.C / 'script.tsv').read_text().splitlines()
                  if line.strip() and not line.startswith('#'))
    assert len(script) == 105
    report = T.read(T.N / 'report.json')
    phrases = T.read(T.N / 'phrase-timings.json')['cues']
    timeline = T.B / T.JOB / 'timeline-output'
    placements, chapters, samples, offset = [], [], [], 0
    layout_sources = {}
    for chapter in T.CHAPTERS:
        table = F.OUT / (chapter + '.timeline.tsv')
        rows = [line.split('\t') for line in F.P.normalized(table)]
        frames = int(next(row[1] for row in rows if row[0] == 'frames'))
        assert next(row[1] for row in rows if row[0] == 'fps') == '60'
        assert F.P.normalized(table) == F.P.normalized(timeline / table.name)
        for row in rows:
            if row[0] == 'line':
                placements.append({'cue': row[1], 'chapter': chapter, 'start_frame': offset + int(row[2]),
                                   'duration_s': V.wav_seconds(row[4])})
        layouts = X.layout_paths(chapter, [F.OUT, F.P.OUT], [timeline])
        layout_sources[chapter] = [str(path) for path in layouts]
        selected = X.select(chapter, rows, frames, phrases, report['lines'], layouts)
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
    captions = V.check_captions(M.OUTPUTS['srt'].read_text(), M.OUTPUTS['vtt'].read_text(),
                                script, report, placements, total)
    return {'frames': total, 'chapters': chapters, 'placements': placements,
            'samples': samples, 'captions': captions, 'outputs': summary['outputs'],
            'layout_sources': layout_sources}


def run():
    receipt = T.read(OUT / 'resource-receipt.json')
    assert receipt.get('admitted') is True and 'exit' not in receipt
    assert receipt['command'] == [sys.executable, '-I', '-B', str(Path(__file__).resolve()), '--run']
    assert receipt['policy'] == T.POLICY, 'Guard policy changed'
    for row in T.read(MANIFEST)['files']:
        assert T.sha(Path(row['path'])) == row['sha256'], row['path']
    cfg = plan()
    movie = str(M.OUTPUTS['master'])
    (OUT / 'decode-logs').mkdir()
    env = dict(os.environ, **V.EXPECT, TMPDIR=str(OUT / 'decode-logs'))
    strict_log = OUT / 'strict-video-verification.log'
    with strict_log.open('x') as log:
        subprocess.run(['bash', str(STRICT), movie], env=env, stdout=log, stderr=subprocess.STDOUT,
                       cwd=T.R, check=True, timeout=14400)
    text = strict_log.read_text()
    assert text.splitlines()[-1] == 'VERIFY PASS'
    decodes = V.decode_lines(text)
    assert V.software_decode_clean(decodes)
    probe = json.loads(subprocess.check_output([str(F.P.PROBE), '-v', 'error', '-show_streams',
                                               '-show_format', '-of', 'json', movie], timeout=600))
    packets = int(subprocess.check_output([str(F.P.PROBE), '-v', 'error', '-select_streams', 'V:0',
                                          '-count_packets', '-show_entries', 'stream=nb_read_packets',
                                          '-of', 'csv=p=0', movie], timeout=3600).strip())
    media = V.media_facts(probe, packets, cfg['frames'])
    loudness_log = OUT / 'encoded-loudness.log'
    with loudness_log.open('x') as log:
        subprocess.run([str(X.FFMPEG), '-nostdin', '-hide_banner', '-nostats', '-i', movie,
                        '-vn', '-af', 'ebur128=peak=true', '-f', 'null', '-'],
                       stdout=log, stderr=subprocess.STDOUT, check=True, timeout=14400)
    loudness = V.loudness_summary(loudness_log.read_text())
    subprocess.run([sys.executable, '-I', '-B', str(SCREEN), str(M.OUTPUTS['srt']),
                    str(cfg['frames']), '60', str(OUT / 'caption-screen.json')], check=True, timeout=600)
    pictures = []
    for index, sample in enumerate(cfg['samples']):
        path = OUT / f'frame-{index:04}.png'
        assert not path.exists()
        subprocess.run([str(X.FFMPEG), '-nostdin', '-n', '-v', 'error', '-xerror', '-threads', '2',
                        '-ss', f"{sample['seconds']:.9f}", '-i', movie, '-frames:v', '1',
                        '-vf', 'scale=960:540', '-threads', '2', str(path)], check=True, timeout=180)
        pictures.append({**sample, 'path': str(path), 'sha256': T.sha(path)})
    sheets = V.make_sheets(pictures, OUT)
    review = T.read(F.REVIEW)
    with (OUT / 'verification.json').open('x') as handle:
        json.dump({'video': movie, 'sha256': cfg['outputs']['master']['sha256'], 'frames': cfg['frames'],
                   'fps': 60, 'chapters': cfg['chapters'], 'placements': cfg['placements'],
                   'captions': cfg['captions'], 'media': media, 'full_decode': decodes,
                   'encoded_loudness': loudness, 'strict_video_notes': [line for line in text.splitlines() if line.startswith('note:')],
                   'caption_screen': T.read(OUT / 'caption-screen.json'),
                   'pictures': pictures, 'sheets': sheets, 'sheet_sha256': {p: T.sha(Path(p)) for p in sheets},
                   'layout_sources': cfg['layout_sources'],
                   'pictures_viewed': False, 'binary_sha256': T.BIN_SHA256, 'timeline_job': T.JOB,
                   'full_baseline_equivalence_established': False,
                   'preview_review': {'path': str(F.REVIEW), 'sha256': T.sha(F.REVIEW), 'review_scope': review['review_scope'],
                                      'covers': 'Preview sheets only. Not these final pictures, not listening.'},
                   'scope': 'Encoded technical verification and selected final pictures only. The preview review does not carry over to these pictures. Caption reading flags require triage. Actual picture viewing, continuous listening, fine sync, final ASR and independent AV acceptance remain open. No upload.'}, handle, indent=2)


def launch():
    plan()
    assert not OUT.exists() and not MANIFEST.exists(), 'Duplicate final verification refused'
    T.guard_policy()
    files = {}
    for row in T.read(M.MANIFEST)['files']:
        assert T.sha(Path(row['path'])) == row['sha256'], row['path']
        files[row['path']] = row['sha256']
    paths = [Path(__file__).resolve(), O / 'start-optimized-final-verify01.sh', MASTER_HELPER, M.RENDER_HELPER,
             *F.IMPORTED_SHA256, SHARED, STRICT, SCREEN, X.FFMPEG, F.P.PROBE, M.MANIFEST,
             T.B / M.JOB / 'resource-receipt.json', T.B / M.JOB / 'master-summary.json']
    paths += list(M.OUTPUTS.values())
    for path in paths:
        files[str(path)] = T.sha(path)
    with MANIFEST.open('x') as handle:
        json.dump({'files': [{'path': p, 'sha256': h} for p, h in sorted(files.items())]}, handle, indent=2)
    subprocess.run([sys.executable, '-I', '-B', str(F.GUARD), '--phase', 'native', '--job', JOB,
                    '--inputs', str(MANIFEST), '--', sys.executable, '-I', '-B', str(Path(__file__).resolve()), '--run'],
                   cwd=T.R, check=True)
    assert T.success(JOB)


def preflight():
    """The pins, before any master exists. --check is root's full plan and needs the finished master."""
    for p in (STRICT, SCREEN):
        assert T.sha(p) == V.PINS[str(p.relative_to(T.R))]
    assert not OUT.exists() and not MANIFEST.exists(), 'New names already used'
    return {'job': JOB, 'pinned_helpers': len(IMPORTED_SHA256), 'launched': False}


if __name__ == '__main__':
    if sys.argv[1:] == ['--pins']:
        print(json.dumps(preflight()))
    elif sys.argv[1:] == ['--check']:
        plan()
    elif sys.argv[1:] == ['--launch']:
        launch()
    elif sys.argv[1:] == ['--run']:
        run()
    else:
        raise SystemExit('Use --pins, --check, --launch, or --run. No implicit launch.')
