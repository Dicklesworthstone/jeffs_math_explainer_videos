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


K = load('current_master', Path(__file__).with_name('final-master16.py'))
S = K.W
ROOT = S.ROOT
REPO = ROOT/'work/production-repo07'
X = load('final_samples', Path(__file__).with_name('final-samples16.py'))
SHARED = REPO/'docs/video-program/third-wave-20261024/opus-replacements-20261104/logarithm-stage09-final-verification02/verify_stage09_final.py'
SHARED_SHA = 'f266a2315e4a9af247c648da416881f3a2999e2682e9c611ff82ca3727b70b94'
SCRIPT = S.P.TREE/'explainer_corpus/complex_analysis_visual/script.tsv'
NAME = 'csd-continuous-final-verify16'
JOB = ROOT/'jobs'/NAME
MANIFEST = ROOT/(NAME+'-inputs.json')
ME = Path(__file__).resolve()
OUTPUTS = {key:K.OUT/('complex_analysis_visual'+suffix) for key,suffix in [('master','_master.mp4'),('srt','.en.srt'),('vtt','.en.vtt')]}
def normalized(p):
    # Mixed-host final chapters retain their original source paths. Compare
    # every timeline row after only the exact qualified directory substitution.
    return p.read_text().replace(str(S.NARRATION),str(S.FMD_NARRATION)).splitlines()


def shared():
    assert S.sha(SHARED) == SHARED_SHA
    v = load('shared_media_verification', SHARED)
    for rel in (v.STRICT_REL, v.SCREEN_REL):
        assert S.sha(REPO / rel) == v.PINS[rel]
    return v


def plan():
    v = shared()
    receipt = S.success('csd-continuous-final-master16')
    assert receipt['command'] == ['/usr/bin/python3','-I','-B',str(K.HERE/'final-master16.py'),'--run']
    assert S.sha(ROOT/'csd-continuous-final-master16-inputs.json') == receipt['inputs_sha256']
    K.verify_inputs(ROOT/'csd-continuous-final-master16-inputs.json')
    assert (K.JOB / 'command.log').read_text().splitlines()[-1] == 'Native master candidate complete; final verification remains.'
    summary = S.read(K.JOB / 'master-summary.json')
    assert summary['chapter_records'] == K.chapters()
    assert summary['binary_sha256'] == S.BINARY_SHA
    for key, path in OUTPUTS.items():
        assert summary['outputs'][key] == {'path': str(path), 'sha256': S.sha(path)}
    script = dict(line.split('\t', 1) for line in SCRIPT.read_text().splitlines() if line.strip() and not line.startswith('#'))
    assert len(script) == 105
    report = S.read(S.NARRATION / 'report.json')
    phrase_data = S.read(S.NARRATION / 'phrase-timings.json')
    v.check_phrases(phrase_data, script, S.NARRATION)
    chapters, placements, samples, layouts = [], [], [], {}
    offset = 0
    for chapter in S.P.CHAPTERS:
        table = K.OUT / (chapter + '.timeline.tsv')
        assert normalized(table) == normalized(S.TIMELINES / table.name)
        rows = [line.split('\t') for line in normalized(table)]
        frames = int(next(row[1] for row in rows if row[0] == 'frames'))
        assert next(row[1] for row in rows if row[0] == 'fps') == '60'
        for row in rows:
            if row[0] == 'line':
                assert S.sha(Path(row[4])) == S.sha(S.NARRATION/(row[1]+'.wav'))
                duration = v.wav_seconds(row[4])
                assert int(row[2]) / 60 + duration <= frames / 60 + .001
                placements.append({'cue': row[1], 'chapter': chapter, 'start_frame': offset + int(row[2]), 'duration_s': duration})
        sources = X.layout_paths(chapter, [K.OUT])
        layouts[chapter] = [str(p) for p in sources]
        selected = X.select(chapter, rows, frames, phrase_data['cues'], report['lines'], sources)
        indexed = {item['frame']:item for item in selected}
        prior = S.read(ROOT/'jobs/csd-continuous-pictures16/pictures/extraction.json')
        for old in prior['samples']:
            if old['chapter'] != chapter:continue
            frame = old['frame'];assert 0 <= frame < frames
            current = indexed.setdefault(frame,dict(chapter=chapter,frame=frame,seconds=frame/60,reasons=[]))
            current['reasons'] = sorted(set(current['reasons']) | {'preview review: '+r for r in old['reasons']})
        selected = [indexed[frame] for frame in sorted(indexed)]
        for item in selected:
            item['local_frame'] = item['frame']
            item['frame'] += offset
            item['seconds'] = item['frame'] / 60
        samples.extend(selected)
        chapters.append({'chapter': chapter, 'frames': frames, 'start_frame': offset})
        offset += frames
    assert offset == summary['frames'] and [p['cue'] for p in placements] == list(script)
    assert len({s['frame'] for s in samples}) == len(samples)
    captions = v.check_captions(OUTPUTS['srt'].read_text(), OUTPUTS['vtt'].read_text(), script, report, placements, offset)
    return {'frames': offset, 'chapters': chapters, 'placements': placements, 'samples': samples,
            'captions': captions, 'outputs': summary['outputs'], 'layout_sources': layouts}


def child():
    return ['/usr/bin/python3', '-I', '-B', str(ME), '--run']


def run():
    receipt = S.read(JOB / 'resource-receipt.json')
    assert receipt['admitted'] and 'exit' not in receipt and receipt['policy'] == S.POLICY
    assert receipt['command'] == child()
    cfg, v = plan(), shared()
    movie = str(OUTPUTS['master'])
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
    subprocess.run([sys.executable, '-I', '-B', str(REPO / v.SCREEN_REL), str(OUTPUTS['srt']),
                    str(cfg['frames']), '60', str(JOB / 'caption-screen.json')], check=True, timeout=600)
    pictures = []
    for index, sample in enumerate(cfg['samples']):
        path = JOB / f'frame-{index:04}.png'
        assert not path.exists()
        subprocess.run([ffmpeg, '-nostdin', '-n', '-v', 'error', '-xerror', '-threads', '2', '-ss',
                        f"{sample['seconds']:.9f}", '-i', movie, '-frames:v', '1', '-vf', 'scale=960:540',
                        '-threads', '2', str(path)], check=True, timeout=180)
        pictures.append({**sample, 'path': str(path), 'sha256': S.sha(path)})
    # Contact sheets remain useful for layout, but cannot establish 4K field detail.
    # Retain additional native-resolution frames without scaling or interpolation.
    details = []
    detail_dir = JOB / 'native-detail'
    detail_dir.mkdir()
    chapter_ends = {c['chapter']: c['start_frame'] + c['frames'] for c in cfg['chapters']}
    selected_detail = {}
    for cue in cfg['placements']:
        # Every chapter can reuse a colour field or dense geometry. Include
        # all cues, especially rigidity, counting, residues and continuation.
        for fraction in (0.25, 0.5, 0.9):
            frame = min(chapter_ends[cue['chapter']] - 1,
                        cue['start_frame'] + round(cue['duration_s'] * fraction * 60))
            selected_detail.setdefault(frame, []).append(f"{cue['cue']} at {fraction:.0%}: native colour/detail review")
    assert {cue['chapter'] for cue in cfg['placements']} == set(chapter_ends)
    assert selected_detail, 'Missing native-detail placements'
    for index, (frame, reasons) in enumerate(sorted(selected_detail.items())):
        path = detail_dir / f'frame-{frame:07}.png'
        subprocess.run([ffmpeg, '-nostdin', '-n', '-v', 'error', '-xerror', '-threads', '2',
                        '-ss', f'{frame/60:.9f}', '-i', movie, '-frames:v', '1',
                        '-threads', '2', str(path)], check=True, timeout=180)
        dimensions = json.loads(subprocess.check_output([ffprobe, '-v', 'error',
                    '-show_streams', '-of', 'json', str(path)], timeout=60))['streams'][0]
        assert (dimensions['width'], dimensions['height']) == (3840, 2160)
        details.append(dict(frame=frame, reasons=reasons, path=str(path), sha256=S.sha(path),
                            width=3840, height=2160, viewed=False))
    sheets = v.make_sheets(pictures, JOB)
    assert plan() == cfg
    with (JOB / 'verification.json').open('x') as stream:
        json.dump({**cfg, 'video': movie, 'sha256': cfg['outputs']['master']['sha256'], 'fps': 60,
                   'media': media, 'full_decode': decodes, 'encoded_loudness': loudness,
                   'strict_video_notes': [line for line in text.splitlines() if line.startswith('note:')],
                   'native_detail_pictures': details, 'native_detail_viewed': False,
                   'caption_screen': S.read(JOB / 'caption-screen.json'), 'pictures': pictures,
                   'sheets': sheets, 'sheet_sha256': {p: S.sha(Path(p)) for p in sheets},
                   'binary_sha256': S.BINARY_SHA, 'voice': 'ElevenLabs Frederick Surrey', 'pictures_viewed': False,
                   'scope': 'Encoded technical checks and selected final pictures only. Actual viewing, caption reading flags, ASR, listening, fine sync and independent AV acceptance remain open.'}, stream, indent=2)
    print('FINAL VERIFY technical checks passed; actual final picture review remains.', flush=True)


if __name__ == '__main__':
    assert sys.argv[1:] == ['--run'], 'Guard child only; freeze real master and runtime before launch'
    run()
