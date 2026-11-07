"""Prepared CSD continuous16 chapter worker. Not launched. Root must actually review pictures and motion before creating the permit.

The permit carries that exact review and extraction record; this worker validates
their binding, but does not claim to have viewed the images itself.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import struct
import subprocess
import wave

import importlib.util
Q = Path('/data/projects/jmev-stroke-probe-20261106/qualification01/continuous-quality16')
spec = importlib.util.spec_from_file_location('review16', Q/'final-review-gate16.py')
G = importlib.util.module_from_spec(spec)
spec.loader.exec_module(G)
P = G.P
ROOT, BIN, BINARY_SHA, POLICY = P.ROOT, P.BIN, P.BINARY_SHA, P.POLICY
TIMELINES, NARRATION = P.TIMELINES, P.NARRATION
FMD_NARRATION = NARRATION  # no host-path rewrite for this local source-bound candidate
STEMS = P.SCORE_OUT/'stems'
INPUTS = ROOT/'work'
PERMIT = Q/'final-production-permit16.json'
REVIEW = Q/'preview16-pictures-viewed.json'
MOTION = Q/'preview16-motion-reviewed.json'

def sha(path):
    with Path(path).open('rb') as f:
        return hashlib.file_digest(f, 'sha256').hexdigest()

def read(path):
    return json.loads(Path(path).read_text())

def success(name):
    r = read(ROOT/'jobs'/name/'resource-receipt.json')
    assert r.get('finished') and r.get('exit') == 0 and r['policy'] == POLICY
    assert not any(r.get(k) for k in ('error','resource_stop','stopped_for_resource_limit'))
    assert r['inputs_sha256'] == r['final_inputs_sha256']
    return r

def check(chapter):
    result = G.check(REVIEW)
    permit = read(PERMIT)
    motion = read(MOTION)
    assert permit['binary_sha256'] == BINARY_SHA
    assert permit['review_sha256'] == sha(REVIEW)
    assert permit['motion_sha256'] == sha(MOTION)
    assert permit['launch_authorized'] is True
    assert motion['binary_sha256'] == BINARY_SHA
    assert motion['preview_summary_sha256'] == sha(P.JOB/'preview-summary.json')
    assert motion['actually_reviewed'] is True and motion['unresolved_defects'] == []
    assert motion['reviewer'].strip() and motion['intervals']
    summary = read(P.JOB/'preview-summary.json')
    chapters = {r['chapter']: r for r in summary['chapters']}
    assert chapter in chapters
    assert {r['chapter'] for r in motion['intervals']} == set(chapters)
    for r in motion['intervals']:
        c = chapters[r['chapter']]
        assert r['mp4_sha256'] == c['mp4_sha256']
        assert 0 <= r['first_frame'] < r['last_frame'] < c['frames']
        assert r['observations'].strip()
    row = chapters[chapter]
    assert sha(TIMELINES/(chapter+'.timeline.tsv')) == row['timeline_sha256']
    return row, result, sha(PERMIT)

def verify_tables(chapter, out, frames):
    # Timeline line rows also contain absolute narration paths. Only the exact
    # qualified narration directory changes; every other byte must match.
    expected = (TIMELINES/(chapter+'.timeline.tsv')).read_bytes().replace(
        str(FMD_NARRATION).encode(), str(NARRATION).encode())
    assert (out/(chapter+'.timeline.tsv')).read_bytes() == expected
    assert (out/(chapter+'.cues.tsv')).read_bytes() == (TIMELINES/(chapter+'.cues.tsv')).read_bytes()
    base = (TIMELINES/(chapter+'.cuelog.tsv')).read_text().replace(str(FMD_NARRATION),str(NARRATION)).splitlines()
    actual = (out/(chapter+'.cuelog.tsv')).read_text().splitlines()
    assert actual[:len(base)] == base
    stem = STEMS/(chapter+'.music.f32.wav')
    parts = stem.with_suffix('.parts.tsv')
    files = [(stem,0,frames*800)]
    if parts.exists():
        files = [(stem.with_name(r[4]),int(r[0]),int(r[1]))
                 for r in [s.split('\t') for s in parts.read_text().splitlines()[1:]]]
    rows = [r.split('\t') for r in actual[len(base):]]
    assert len(rows) == len(files)
    end = 0
    for row,(path,start,samples) in zip(rows,files):
        offset = float(row[5])
        bits = struct.unpack('>Q',struct.pack('>d',offset))[0]
        assert row[:5] == [str(path),sha(path),str(path.stat().st_size),str(frames),'60']
        assert row[6:] == [f'{bits:#018x}','0.0','0x0000000000000000','48000','2',str(samples),str(start)]
        assert frames*800+round(offset*48000) == start == end
        end += samples
    assert end == frames*800

def run(chapter):
    row, review, permit_sha = check(chapter)
    os.sched_setaffinity(0,{0,1,2,3})
    assert os.sched_getaffinity(0)=={0,1,2,3}
    job = ROOT/'jobs'/('csd-continuous-final-'+chapter+'-16')
    receipt = read(job/'resource-receipt.json')
    assert receipt['admitted'] and 'exit' not in receipt and receipt['policy'] == POLICY
    assert receipt['command'] == ['/usr/bin/python3','-I','-B',str(Path(__file__).resolve()),'--chapter',chapter]
    out = job/'render'
    assert not out.exists()
    os.environ.update(TMPDIR=str(ROOT/'work/tmp'),XDG_CACHE_HOME=str(ROOT/'work/cache'))
    command = [str(BIN),'render',chapter,'--res','3840x2160','--fps','60',
               '--narration',str(NARRATION),'--score',str(STEMS),'--out',str(out),
               '--lint','--crf','16','--preset','veryfast']
    with (job/'native.stdout').open('xb') as stdout,(job/'native.stderr').open('xb') as stderr:
        subprocess.run(command,cwd=INPUTS/'production-repo07',stdout=stdout,stderr=stderr,check=True,timeout=43200)
    frames = row['frames']
    verify_tables(chapter,out,frames)
    movie = out/(chapter+'.mp4')
    assert set(out.glob('*.mp4')) == {movie}
    probe = json.loads(subprocess.check_output(['ffprobe','-v','error','-show_streams','-of','json',str(movie)],timeout=120))
    videos = [s for s in probe['streams'] if s['codec_type']=='video']
    audios = [s for s in probe['streams'] if s['codec_type']=='audio']
    assert len(videos)==len(audios)==1
    v,a = videos[0],audios[0]
    assert (v['width'],v['height'],v['avg_frame_rate'],int(v['nb_frames'])) == (3840,2160,'60/1',frames)
    assert (v['codec_name'],v['pix_fmt'],a['codec_name'],a['sample_rate'],a['channels']) == ('h264','yuv420p','aac','48000',2)
    with movie.open('rb') as f: head=f.read(16*1024*1024)
    pos=head.find(b'x264 - core');assert pos>=0
    settings=head[pos:pos+5000].split(b'\0',1)[0].decode('utf-8',errors='replace')
    assert ' rc=crf' in settings and ' crf=16.0' in settings
    captures=sorted((out/'soundtrack'/chapter).glob('soundtrack_*.wav'),key=lambda p:int(p.stem.split('_')[-1]))
    assert captures
    indices=[int(p.stem.split('_')[-1]) for p in captures]
    assert indices==list(range(indices[0],indices[-1]+1))
    capture=captures[-1]
    with wave.open(str(capture)) as w:
        assert (w.getframerate(),w.getnchannels(),w.getsampwidth(),w.getnframes())==(48000,2,2,frames*800)
        total=0
        while data:=w.readframes(48000):total+=len(data)
        assert total==frames*800*4
    assert check(chapter)==(row,review,permit_sha)
    result=dict(chapter=chapter,host='csd',frames=frames,fps=60,binary_sha256=BINARY_SHA,
        review=review,permit_sha256=permit_sha,encoder=settings,
        files=[dict(path=str(out/(chapter+s)),sha256=sha(out/(chapter+s)))
               for s in ('.mp4','.timeline.tsv','.cues.tsv','.cuelog.tsv','.layout.tsv')],
        soundtrack=dict(path=str(capture),sha256=sha(capture),samples=frames*800,rate=48000,channels=2,sample_width=2),
        scope='Technical chapter only; import verification, bit-identical master replay and full final AV/picture checks remain.')
    with (job/'render-summary.json').open('x') as f:json.dump(result,f,indent=2)
    print('CSD CONTINUOUS16 FINAL CHAPTER technical checks passed: '+chapter)

if __name__=='__main__':
    parser=argparse.ArgumentParser();parser.add_argument('--chapter',required=True);parser.add_argument('--check',action='store_true');args=parser.parse_args()
    if args.check:print(check(args.chapter))
    else:run(args.chapter)
