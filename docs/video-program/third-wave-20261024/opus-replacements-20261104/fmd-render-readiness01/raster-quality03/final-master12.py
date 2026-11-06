"""Guard child: master verified local and imported raster12 chapters natively.

This has no launch operation. A separate explicit guard admission must freeze
all chapter inputs and outputs before --run is used.
"""
import importlib.util
import json
import os
from pathlib import Path
import subprocess
import sys

HERE = Path('/data/projects/jmev-stroke-probe-20261106/qualification01/raster-quality12')

def load(name):
    spec = importlib.util.spec_from_file_location(name, HERE/name)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module

L = load('launch-csd-disjoint12.py')
I = load('verify-fmd-chapter12.py')
W = L.W
ROOT = W.ROOT
JOB = ROOT/'jobs/csd-raster-final-master12'
OUT = JOB/'final'

def chapters():
    assignment = W.read(W.ASSIGNMENT)
    permit = W.read(W.PERMIT)
    rows = assignment['assignments']
    assert len(rows) == 15
    assert [r['chapter'] for r in rows] == permit['chapters']
    records = []
    for row in rows:
        chapter = row['chapter']
        if row['host'] == 'csd':
            record = L.completed(chapter)
        else:
            assert row['host'] == 'fmd'
            record = I.verify(chapter)
        assert record['frames'] == row['frames'] and record['fps'] == 60
        assert record['review'] == permit['review_gate_result']
        records.append(record)
    assert sum(r['frames'] for r in records) == 117697
    return records

def run():
    assert sys.argv[1:] == ['--run']
    assert Path(__file__).resolve() == HERE/'final-master12.py'
    receipt = W.read(JOB/'resource-receipt.json')
    assert receipt['admitted'] and 'exit' not in receipt and receipt['policy'] == W.POLICY
    assert receipt['command'] == ['/usr/bin/python3','-I','-B',str(HERE/'final-master12.py'),'--run']
    records = chapters()
    assert not OUT.exists()
    OUT.mkdir()
    links = []
    for record in records:
        chapter = record['chapter']
        for row in record['files']:
            source = Path(row['path'])
            assert W.sha(source) == row['sha256']
            dest = OUT/source.name
            assert not dest.exists()
            os.link(source, dest)
            links.append(dict(source=str(source),path=str(dest),sha256=row['sha256']))
        capture = record['soundtrack']
        source = Path(capture['path'])
        assert W.sha(source) == capture['sha256']
        directory = OUT/'soundtrack'/chapter
        directory.mkdir(parents=True)
        dest = directory/'soundtrack_0.wav'
        os.link(source, dest)
        links.append(dict(source=str(source),path=str(dest),sha256=capture['sha256']))
    with (JOB/'staged-inputs.json').open('x') as f:
        json.dump(links,f,indent=2)
    with (JOB/'master-native.log').open('x') as log:
        subprocess.run([str(W.BIN),'master','--narration',str(W.NARRATION),'--score',str(W.STEMS),'--out',str(OUT)],cwd=ROOT/'work/production-repo07',stdout=log,stderr=subprocess.STDOUT,check=True,timeout=14400)
    replays = [line for line in (JOB/'master-native.log').read_text().splitlines() if "replay = fmn's soundtrack, bit for bit" in line]
    assert [line.split()[0] for line in replays] == W.read(W.PERMIT)['chapters']
    with (JOB/'captions-native.log').open('x') as log:
        subprocess.run([str(W.BIN),'captions','all','--narration',str(W.NARRATION),'--out',str(OUT)],cwd=ROOT/'work/production-repo07',stdout=log,stderr=subprocess.STDOUT,check=True,timeout=1200)
    assert chapters() == records
    for link in links:
        assert W.sha(Path(link['source'])) == W.sha(Path(link['path'])) == link['sha256']
    outputs = {key:OUT/('complex_analysis_visual'+suffix) for key,suffix in [('master','_master.mp4'),('srt','.en.srt'),('vtt','.en.vtt')]}
    probe = json.loads(subprocess.check_output(['ffprobe','-v','error','-show_streams','-of','json',str(outputs['master'])],timeout=120))
    videos = [s for s in probe['streams'] if s['codec_type']=='video']
    audios = [s for s in probe['streams'] if s['codec_type']=='audio']
    assert len(videos) == len(audios) == 1
    v,a = videos[0],audios[0]
    assert (v['width'],v['height'],v['avg_frame_rate'],int(v['nb_frames']),v['codec_name'],v['pix_fmt']) == (3840,2160,'60/1',117697,'h264','yuv420p')
    assert (a['codec_name'],a['sample_rate'],a['channels']) == ('aac','48000',2)
    assert abs(float(a['duration'])-117697/60) <= 1024/48000+0.001
    assert outputs['srt'].stat().st_size and outputs['vtt'].read_text().startswith('WEBVTT')
    with (JOB/'master-summary.json').open('x') as f:
        json.dump(dict(chapters=W.read(W.PERMIT)['chapters'],frames=117697,fps=60,binary_sha256=W.BINARY_SHA,voice='ElevenLabs Frederick Surrey',native_bit_identical_replays=replays,chapter_records=records,outputs={k:dict(path=str(p),sha256=W.sha(p)) for k,p in outputs.items()},scope='Candidate native master and captions only; full decode, exact captions, encoded loudness, final pictures, listening, fine sync and independent AV remain.'),f,indent=2)
    print('Native master candidate complete; final verification remains.')

if __name__ == '__main__':
    run()
