"""Guarded csd encoded-master checks and fresh final picture extraction.

Actual viewing is separate. Preview acceptance never accepts these final frames.
"""
import importlib.util
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys


def load(name, path):
    spec = importlib.util.spec_from_file_location(name, path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


K = load('csd_master', Path(__file__).with_name('final-master02.py'))
F, S, B = K.F, K.S, K.B
ROOT = S.ROOT
REPO = ROOT / 'work/production-repo07'
SHARED = REPO / 'docs/video-program/third-wave-20261024/opus-replacements-20261104/logarithm-stage09-final-verification02/verify_stage09_final.py'
SHARED_SHA = 'f266a2315e4a9af247c648da416881f3a2999e2682e9c611ff82ca3727b70b94'
SCRIPT = ROOT / 'work/production-build07/workspace/explainer_corpus/complex_analysis_visual/script.tsv'
NAME = 'stage07-final-verify01'
JOB = ROOT / 'jobs' / NAME
MANIFEST = ROOT / (NAME + '-inputs.json')
ME = ROOT / 'qualification01/final-verify02.py'


def shared():
    assert S.sha(SHARED) == SHARED_SHA
    v = load('shared_media_verification', SHARED)
    for rel in (v.STRICT_REL, v.SCREEN_REL):
        assert S.sha(REPO / rel) == v.PINS[rel]
    return v


def plan():
    v = shared()
    receipt = S.success(K.NAME)
    assert receipt['command'] == K.child()
    assert S.sha(K.MANIFEST) == receipt['inputs_sha256']
    B.collect(K.MANIFEST, [])
    assert (K.JOB / 'command.log').read_text().splitlines()[-1] == 'MASTER candidate written; final verification and review remain.'
    summary = S.read(K.JOB / 'master-summary.json')
    assert summary['chapter_records'] == K.chapters()
    assert summary['binary_sha256'] == S.BINARY_SHA
    for key, path in K.OUTPUTS.items():
        assert summary['outputs'][key] == {'path': str(path), 'sha256': S.sha(path)}
    script = dict(line.split('\t', 1) for line in SCRIPT.read_text().splitlines() if line.strip() and not line.startswith('#'))
    assert len(script) == 105
    report = S.read(S.NARRATION / 'report.json')
    phrase_data = S.read(S.NARRATION / 'phrase-timings.json')
    v.check_phrases(phrase_data, script, S.NARRATION)
    chapters, placements, samples, layouts = [], [], [], {}
    offset = 0
    for chapter in S.CHAPTERS:
        table = K.OUT / (chapter + '.timeline.tsv')
        assert F.P.normalized(table) == F.P.normalized(S.TIMELINES / table.name)
        rows = [line.split('\t') for line in F.P.normalized(table)]
        frames = int(next(row[1] for row in rows if row[0] == 'frames'))
        assert next(row[1] for row in rows if row[0] == 'fps') == '60'
        for row in rows:
            if row[0] == 'line':
                assert Path(row[4]) == S.NARRATION / (row[1] + '.wav')
                duration = v.wav_seconds(row[4])
                assert int(row[2]) / 60 + duration <= frames / 60 + .001
                placements.append({'cue': row[1], 'chapter': chapter, 'start_frame': offset + int(row[2]), 'duration_s': duration})
        sources = F.G.X.layout_paths(chapter, [K.OUT, F.P.OUT])
        layouts[chapter] = [str(p) for p in sources]
        selected = F.G.X.select(chapter, rows, frames, phrase_data['cues'], report['lines'], sources)
        for item in selected:
            item['local_frame'] = item['frame']
            item['frame'] += offset
            item['seconds'] = item['frame'] / 60
        samples.extend(selected)
        chapters.append({'chapter': chapter, 'frames': frames, 'start_frame': offset})
        offset += frames
    assert offset == summary['frames'] and [p['cue'] for p in placements] == list(script)
    assert len({s['frame'] for s in samples}) == len(samples)
    captions = v.check_captions(K.OUTPUTS['srt'].read_text(), K.OUTPUTS['vtt'].read_text(), script, report, placements, offset)
    return {'frames': offset, 'chapters': chapters, 'placements': placements, 'samples': samples,
            'captions': captions, 'outputs': summary['outputs'], 'layout_sources': layouts}


def child():
    return ['/usr/bin/python3', '-I', '-B', str(ME), '--run']


def run():
    receipt = S.read(JOB / 'resource-receipt.json')
    assert receipt['admitted'] and 'exit' not in receipt and receipt['policy'] == S.POLICY
    assert receipt['command'] == child()
    cfg, v = plan(), shared()
    movie = str(K.OUTPUTS['master'])
    ffmpeg, ffprobe = shutil.which('ffmpeg'), shutil.which('ffprobe')
    assert ffmpeg and ffprobe
    (JOB / 'decode-logs').mkdir()
    env = dict(os.environ, **v.EXPECT, TMPDIR=str(JOB / 'decode-logs'))
    with (JOB / 'strict-video-verification.log').open('x') as log:
        subprocess.run(['bash', str(REPO / v.STRICT_REL), movie], env=env, stdout=log,
                       stderr=subprocess.STDOUT, cwd=REPO, check=True, timeout=14400)
    text = (JOB / 'strict-video-verification.log').read_text()
    assert text.splitlines()[-1] == 'VERIFY PASS'
    decodes = v.decode_lines(text)
    assert v.software_decode_clean(decodes)
    probe = json.loads(subprocess.check_output([ffprobe, '-v', 'error', '-show_streams', '-show_format', '-of', 'json', movie], timeout=600))
    packets = int(subprocess.check_output([ffprobe, '-v', 'error', '-select_streams', 'V:0', '-count_packets',
                                          '-show_entries', 'stream=nb_read_packets', '-of', 'csv=p=0', movie], timeout=3600).strip())
    media = v.media_facts(probe, packets, cfg['frames'])
    with (JOB / 'encoded-loudness.log').open('x') as log:
        subprocess.run([ffmpeg, '-nostdin', '-hide_banner', '-nostats', '-i', movie, '-vn', '-af',
                        'ebur128=peak=true', '-f', 'null', '-'], stdout=log, stderr=subprocess.STDOUT, check=True, timeout=14400)
    loudness = v.loudness_summary((JOB / 'encoded-loudness.log').read_text())
    assert float(loudness['true_peak_dbtp']) <= -1.0
    subprocess.run([sys.executable, '-I', '-B', str(REPO / v.SCREEN_REL), str(K.OUTPUTS['srt']),
                    str(cfg['frames']), '60', str(JOB / 'caption-screen.json')], check=True, timeout=600)
    pictures = []
    for index, sample in enumerate(cfg['samples']):
        path = JOB / f'frame-{index:04}.png'
        assert not path.exists()
        subprocess.run([ffmpeg, '-nostdin', '-n', '-v', 'error', '-xerror', '-threads', '2', '-ss',
                        f"{sample['seconds']:.9f}", '-i', movie, '-frames:v', '1', '-vf', 'scale=960:540',
                        '-threads', '2', str(path)], check=True, timeout=180)
        pictures.append({**sample, 'path': str(path), 'sha256': S.sha(path)})
    sheets = v.make_sheets(pictures, JOB)
    assert plan() == cfg
    with (JOB / 'verification.json').open('x') as stream:
        json.dump({**cfg, 'video': movie, 'sha256': cfg['outputs']['master']['sha256'], 'fps': 60,
                   'media': media, 'full_decode': decodes, 'encoded_loudness': loudness,
                   'strict_video_notes': [line for line in text.splitlines() if line.startswith('note:')],
                   'caption_screen': S.read(JOB / 'caption-screen.json'), 'pictures': pictures,
                   'sheets': sheets, 'sheet_sha256': {p: S.sha(Path(p)) for p in sheets},
                   'binary_sha256': S.BINARY_SHA, 'voice': 'ElevenLabs Frederick Surrey', 'pictures_viewed': False,
                   'scope': 'Encoded technical checks and selected final pictures only. Actual viewing, caption reading flags, ASR, listening, fine sync and independent AV acceptance remain open.'}, stream, indent=2)
    print('FINAL VERIFY technical checks passed; actual final picture review remains.', flush=True)


if __name__ == '__main__':
    assert sys.argv[1:] == ['--run'], 'Guard child only; freeze real master and runtime before launch'
    run()
