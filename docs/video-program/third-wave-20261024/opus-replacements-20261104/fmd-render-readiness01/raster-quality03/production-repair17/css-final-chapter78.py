"""CSS production19 scored 4K60 chapter worker.
Requires a root-authored actual visual/motion review record and frozen inputs.
Never creates its own review authorization or launches another job.
"""
import argparse, hashlib, json, os, struct, subprocess, wave
from pathlib import Path
ROOT=Path('/data/projects/jmev-css-render-worker-20261107')
D=Path('/data/projects/jmev-stroke-probe-20261106')
BIN=D/'work/bin/production19'
BINARY_SHA='f4836eeeb4f2465c0d27ca0497f123f0af6bc0db7bc65096098bb20c5abb0f22'
TIMELINES=D/'jobs/csd-production-timeline21/timeline-output'
NARRATION=ROOT/'jobs/css-production19-delivery76/narration'
FMD_NARRATION=D/'jobs/csd-production-assembly19/narration'
STEMS=D/'jobs/csd-production-score21/score/stems'
def sha(path):
 with Path(path).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest()
def read(path):return json.loads(Path(path).read_text())
def success(path):
 r=read(path);assert r.get('finished') and r.get('exit')==0 and r['inputs_sha256']==r['final_inputs_sha256']
 assert not any(r.get(k) for k in ('error','resource_stop','stopped_for_resource_limit'))
 return r
def check(chapter,permit):
 assert sha(BIN)==BINARY_SHA
 for p in (ROOT/'jobs/css-runtime-transfer75/resource-receipt.json',ROOT/'jobs/css-encoder-stop79/resource-receipt.json',ROOT/'jobs/css-production19-delivery76/resource-receipt.json',D/'jobs/csd-production-timeline21/resource-receipt.json',D/'jobs/csd-production-score21/resource-receipt.json'):success(p)
 qualification=read(D/'jobs/csd-production-repair-build19/qualification.json')
 assert chapter in qualification['chapters']
 score=read(D/'jobs/csd-production-score21/score21-summary.json');assert score['all_qa_passed'] and score['chapters']==qualification['chapters']
 for relative,digest in score['files'].items():assert sha(D/'jobs/csd-production-score21/score'/relative)==digest
 review=read(permit)
 assert review['binary_sha256']==BINARY_SHA and review['chapter']==chapter
 assert review['actually_reviewed_pictures'] is True and review['actually_reviewed_motion'] is True
 assert review['unresolved_defects']==[] and review['reviewer'].strip() and review['observations'].strip()
 assert review['timeline_sha256']==sha(TIMELINES/(chapter+'.timeline.tsv'))
 assert review['evidence'] and review['motion_intervals']
 for item in review['evidence']:assert sha(item['path'])==item['sha256']
 frames=review['frames'];assert type(frames) is int and frames>0
 for a,b in review['motion_intervals']:assert 0<=a<b<frames
 assert read(STEMS/'qa'/(chapter+'.json'))['passed'] is True
 return review,sha(permit)
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

def run(args):
 chapter=args.chapter;review,permit_sha=check(chapter,args.permit)
 job=ROOT/'jobs'/args.job
 assert job.parent==ROOT/'jobs' and args.job.startswith('css-production19-final-')
 admitted=read(job/'resource-receipt.json');assert admitted['admitted'] and 'exit' not in admitted
 assert admitted['policy']['stop_free_gib']==8 and admitted['policy']['phase_growth_gib']==4
 inputs=Path(args.inputs);assert sha(inputs)==admitted['inputs_sha256']
 frozen=read(inputs)['files'];assert len(frozen)==len({x['path'] for x in frozen})
 bound={x['path']:x['sha256'] for x in frozen}
 for x in frozen:assert sha(x['path'])==x['sha256']
 required=[BIN,Path(__file__).resolve(),args.permit,TIMELINES/(chapter+'.timeline.tsv'),TIMELINES/(chapter+'.cues.tsv'),TIMELINES/(chapter+'.cuelog.tsv')]
 inherited=ROOT/'qualification01/delivery76-inputs.json'
 delivery=success(ROOT/'jobs/css-production19-delivery76/resource-receipt.json');assert sha(inherited)==delivery['inputs_sha256']
 required += [inherited]+[Path(x['path']) for x in read(inherited)['files']]
 required += [p for p in NARRATION.rglob('*') if p.is_file()]
 required += [D/'jobs/csd-production-score21/score'/p for p in read(D/'jobs/csd-production-score21/score21-summary.json')['files']]
 for p in required:assert bound.get(str(p))==sha(p),str(p)
 available=sorted(os.sched_getaffinity(0));assert len(available)>=args.threads
 os.sched_setaffinity(0,set(available[:args.threads]))
 out=job/'render';assert not out.exists()
 os.environ.update(PATH='/usr/bin:/bin',TMPDIR=str(ROOT/'work/tmp'),XDG_CACHE_HOME=str(ROOT/'work/cache'))
 command=[str(BIN),'render',chapter,'--res','3840x2160','--fps','60','--threads',str(args.threads),'--narration',str(NARRATION),'--score',str(STEMS),'--out',str(out),'--lint','--crf','16','--preset','veryfast']
 with (job/'native.stdout').open('xb') as o,(job/'native.stderr').open('xb') as e:subprocess.run(command,cwd=D/'work/production-repo07',stdout=o,stderr=e,check=True,timeout=43200)
 frames=review['frames'];verify_tables(chapter,out,frames)
 movie=out/(chapter+'.mp4');assert set(out.glob('*.mp4'))=={movie}
 probe=json.loads(subprocess.check_output(['ffprobe','-v','error','-show_streams','-of','json',str(movie)],timeout=120))
 vs=[x for x in probe['streams'] if x['codec_type']=='video'];aus=[x for x in probe['streams'] if x['codec_type']=='audio'];assert len(vs)==len(aus)==1
 v,a=vs[0],aus[0];assert (v['width'],v['height'],v['avg_frame_rate'],int(v['nb_frames']))==(3840,2160,'60/1',frames)
 assert (v['codec_name'],v['pix_fmt'],a['codec_name'],a['sample_rate'],a['channels'])==('h264','yuv420p','aac','48000',2)
 with movie.open('rb') as f:head=f.read(16*1024*1024)
 pos=head.find(b'x264 - core');assert pos>=0
 settings=head[pos:pos+5000].split(b'\0',1)[0].decode('utf8',errors='replace');assert ' rc=crf' in settings and ' crf=16.0' in settings
 captures=sorted((out/'soundtrack'/chapter).glob('soundtrack_*.wav'),key=lambda p:int(p.stem.split('_')[-1]));assert captures
 with wave.open(str(captures[-1])) as w:assert (w.getframerate(),w.getnchannels(),w.getsampwidth(),w.getnframes())==(48000,2,2,frames*800)
 assert check(chapter,args.permit)==(review,permit_sha)
 for x in frozen:assert sha(x['path'])==x['sha256']
 result=dict(chapter=chapter,frames=frames,binary_sha256=BINARY_SHA,permit_sha256=permit_sha,encoder=settings,files=[dict(path=str(p),sha256=sha(p)) for p in sorted(out.rglob('*')) if p.is_file()],scope='Technical scored4K60 chapter only; full decode, master replay, final pictures/motion/audio/captions and publication remain.')
 with (job/'render-summary.json').open('x') as f:json.dump(result,f,indent=2)
if __name__=='__main__':
 p=argparse.ArgumentParser();p.add_argument('--chapter',required=True);p.add_argument('--job',required=True);p.add_argument('--inputs',type=Path,required=True);p.add_argument('--permit',type=Path,required=True);p.add_argument('--threads',type=int,default=12);args=p.parse_args();assert 1<=args.threads<=16;run(args)
