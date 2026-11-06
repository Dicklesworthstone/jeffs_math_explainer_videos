"""Explicit guarded native mastering of the separate csd chapter outputs.

Stage immutable input links in a fresh directory, then use the native master's
bit-identical soundtrack replay and fixed-gain path. No new encoding of pictures.
"""
import importlib.util
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys

_spec = importlib.util.spec_from_file_location('csd_final_launch', Path(__file__).with_name('launch-final02.py'))
L = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(L)
F, S, B = L.F, L.S, L.B
ROOT = S.ROOT
NAME = 'stage07-final-master01'
JOB = ROOT / 'jobs' / NAME
OUT = JOB / 'final'
MANIFEST = ROOT / (NAME + '-inputs.json')
ME = ROOT / 'qualification01/final-master02.py'
OUTPUTS = {key: OUT / ('complex_analysis_visual' + suffix) for key, suffix in
           [('master', '_master.mp4'), ('srt', '.en.srt'), ('vtt', '.en.vtt')]}


def chapters():
    review = F.G.check(F.REVIEW)
    records = [L.verify_completed(chapter, review) for chapter in S.CHAPTERS]
    for record in records:
        table = S.TIMELINES / (record['chapter'] + '.timeline.tsv')
        frames = int(next(row.split('\t')[1] for row in table.read_text().splitlines() if row.startswith('frames\t')))
        assert record['frames'] == frames and record['fps'] == 60
    return records


def stage(records, out):
    assert not out.exists(), 'Preserve previous assembly'
    out.mkdir()
    linked = []
    for record in records:
        chapter = record['chapter']
        for row in record['files']:
            source = Path(row['path'])
            assert S.sha(source) == row['sha256']
            dest = out / source.name
            assert not dest.exists()
            os.link(source, dest)
            assert S.sha(dest) == row['sha256']
            linked.append({'source': str(source), 'path': str(dest), 'sha256': row['sha256']})
        # Native master starts scanning at zero. The accepted complete capture
        # is staged as zero regardless of any preserved prior mux attempts.
        capture = record['soundtrack']
        source = Path(capture['path'])
        assert S.sha(source) == capture['sha256']
        directory = out / 'soundtrack' / chapter
        directory.mkdir(parents=True)
        dest = directory / 'soundtrack_0.wav'
        os.link(source, dest)
        linked.append({'source': str(source), 'path': str(dest), 'sha256': capture['sha256']})
    return linked


def child():
    return ['/usr/bin/python3', '-I', '-B', str(ME), '--run']


def run():
    receipt = S.read(JOB / 'resource-receipt.json')
    assert receipt['admitted'] and 'exit' not in receipt and receipt['policy'] == S.POLICY
    assert receipt['command'] == child()
    records = chapters()
    links = stage(records, OUT)
    with (JOB / 'staged-inputs.json').open('x') as stream:
        json.dump(links, stream, indent=2)
    with (JOB / 'master-native.log').open('x') as log:
        subprocess.run([str(S.BIN), 'master', '--narration', str(S.NARRATION), '--score', str(S.OUT / 'stems'), '--out', str(OUT)],
                       cwd=ROOT / 'work/production-repo07', stdout=log, stderr=subprocess.STDOUT, check=True, timeout=14400)
    replays = [line for line in (JOB / 'master-native.log').read_text().splitlines()
               if "replay = fmn's soundtrack, bit for bit" in line]
    assert [line.split()[0] for line in replays] == list(S.CHAPTERS)
    with (JOB / 'captions-native.log').open('x') as log:
        subprocess.run([str(S.BIN), 'captions', 'all', '--narration', str(S.NARRATION), '--out', str(OUT)],
                       cwd=ROOT / 'work/production-repo07', stdout=log, stderr=subprocess.STDOUT, check=True, timeout=1200)
    assert chapters() == records
    for link in links:
        assert S.sha(Path(link['path'])) == S.sha(Path(link['source'])) == link['sha256']
    frames = sum(row['frames'] for row in records)
    ffprobe = shutil.which('ffprobe')
    assert ffprobe
    probe = json.loads(subprocess.check_output([ffprobe, '-v', 'error', '-show_streams', '-of', 'json', str(OUTPUTS['master'])], timeout=120))
    F.video_facts(probe, frames)
    assert OUTPUTS['srt'].stat().st_size and OUTPUTS['vtt'].read_text().startswith('WEBVTT')
    with (JOB / 'master-summary.json').open('x') as stream:
        json.dump({'chapters': list(S.CHAPTERS), 'frames': frames, 'fps': 60,
                   'binary_sha256': S.BINARY_SHA, 'voice': 'ElevenLabs Frederick Surrey',
                   'native_bit_identical_replays': replays, 'chapter_records': records,
                   'outputs': {key: {'path': str(path), 'sha256': S.sha(path)} for key, path in OUTPUTS.items()},
                   'scope': 'Candidate native master and captions only; full decode, exact captions, encoded loudness, final pictures, ASR, listening, fine sync and independent AV remain.'}, stream, indent=2)
    print('MASTER candidate written; final verification and review remain.', flush=True)


def launch():
    assert Path(__file__).resolve() == ME
    assert not JOB.exists() and not MANIFEST.exists(), 'Never duplicate or overwrite master attempts'
    records = chapters()
    previous = L.manifest(S.CHAPTERS[0])
    extra = [ME, ROOT / 'qualification01/launch-final02.py']
    for chapter, record in zip(S.CHAPTERS, records):
        manifest = L.manifest(chapter)
        assert B.collect(manifest, []) == B.collect(previous, []), 'Chapter inputs differ'
        job = ROOT / 'jobs' / F.job_name(chapter)
        extra.extend([manifest, job / 'resource-receipt.json', job / 'render-summary.json', job / 'command.log'])
        extra.extend(Path(row['path']) for row in record['files'])
        extra.append(Path(record['soundtrack']['path']))
    frozen = B.collect(previous, extra)
    with MANIFEST.open('x') as stream:
        json.dump(frozen, stream, indent=2)
    subprocess.run(['/usr/bin/python3', '-I', '-B', str(ROOT / 'guard_csd02.py'), '--phase', 'native',
                    '--job', NAME, '--inputs', str(MANIFEST), '--', *child()], cwd=ROOT, check=True)
    S.success(NAME)


if __name__ == '__main__':
    if sys.argv[1:] == ['--launch']:
        launch()
    elif sys.argv[1:] == ['--run']:
        run()
    else:
        raise SystemExit('Use --launch explicitly after final chapters; --run is guard child only')
