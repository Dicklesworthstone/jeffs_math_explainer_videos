"""CSD combined-repair scored 4K60 chapter worker, version 287. Prepared only.
Requires a root-authored actual visual/motion review record and frozen inputs.
Never creates its own review authorization or launches another job.
Against 278 (preserved), 287 adds: the reviewed preview's three tables must equal
timeline269 after only the named narration-path mapping; the preview's binary, worker
and build source copy are bound through its frozen manifest; a preview built before 268
needs the named binding282 evidence at its pinned hash, and the two pinned build source
copies must differ in exactly 282's file set, never in the chapter's scene file; the
permit path must be absolute and free of symlinks; an existing 244/278 job for the same
chapter is refused. Every 278 picture/motion/resource/score gate is kept.
"""
import argparse, hashlib, json, os, struct, subprocess, wave
from pathlib import Path
ROOT=Path('/data/projects/jmev-stroke-probe-20261106')
D=Path('/data/projects/jmev-stroke-probe-20261106')
Q=ROOT/'qualification01/production-repair17'
BIN=D/'work/bin/review-repair268'
BINARY_SHA='983954c508e42b90d443ad6ca09f2baba7565a4f609d09d2ea821640534d6ec7'
TIMELINES=D/'jobs/csd-review-repair-timeline269/timeline-output'
NARRATION=ROOT/'jobs/csd-review-repair-rebind269/narration'
FMD_NARRATION=D/'jobs/csd-review-repair-rebind269/narration'
CHAPTERS=('01_the_bump', '04_reading_color', '05_gallery', '06_one_derivative', '07_one_point', '08_loop_sums', '09_boundary', '10_rigidity', '11_counting', '12_residues', '14_continuation')
STEMS=D/'jobs/csd-production-score21/score/stems'
BUILD268=D/'jobs/csd-review-repair-build268'
SCENE_DIR='explainer_corpus/complex_analysis_visual/src/chapters/'
# Chapter key -> scene file, from the production19 registry (mod.rs `make:` entries).
SCENES={'01_the_bump':'the_bump.rs','04_reading_color':'reading_color.rs','05_gallery':'gallery.rs','06_one_derivative':'one_derivative.rs','07_one_point':'one_point.rs','08_loop_sums':'loop_sums.rs','09_boundary':'boundary.rs','10_rigidity':'rigidity.rs','11_counting':'counting.rs','12_residues':'residues.rs','14_continuation':'continuing.rs'}
# The preview root actually reviewed for 06/08. Names from review-repair-after-timeline243.py:18-25
# (job, manifest, worker); binary 238, narration rebind242 and the preview/ table folder from
# review-repair-preview243.py:3,11. Its manifest holds the binary and build238's job files by
# way of review-repair-after-build242.py:10 and the timeline242 manifest rows.
PREVIEW243=dict(job='csd-review-repair-preview243',manifest='review-repair-preview243-inputs.json',
 worker='review-repair-preview243.py',worker_sha256='3e5dd06c48b888e4a947a71f57929121fdb76590b0a6e5dfc3e21bd5f1e23c17',
 tables='preview',narration=D/'jobs/csd-review-repair-rebind242/narration',
 binary=D/'work/bin/review-repair238',binary_sha256='f31690e648393923c373922b513a77e65dc44fef10dbd39f8b88b561e283e9da',
 build=D/'jobs/csd-review-repair-build238',source_copy_sha256='6d4c7e93e89a29930cf5b4c2f3101fba023a3e09b99888f3e9db083bdb1eb65c')
# build268 source copy as read by cc1-preview-to-final-binding282.md (section "Bound records").
SOURCE_COPY_268_SHA='81d28272a7a23b63199e79e6aa1086f01f131f0ddd685d4ae859d04b1de1934c'
# CC3's preview274 job, manifest, worker, table folder and narration folder were in no file
# when 287 was written. The other nine chapters therefore fail closed here until a later
# version names them from CC3's actual helper and outputs, with build=BUILD268, binary=BIN
# and binary_sha256=BINARY_SHA.
PREVIEW274=None
PREVIEWS={c:(PREVIEW243 if c in ('06_one_derivative','08_loop_sums') else PREVIEW274) for c in CHAPTERS}
# Named 238-to-268 delta evidence: CC1's binding282 (root read it in full). It is source-level
# carry-forward only: no 540p pixel, audio or motion equivalence, and its schedule half stays
# conditional on timeline269's actual tables, which check() and preview_binding() assert.
DELTA_NAME='build238-to-build268-delta'
DELTA_EVIDENCE=Q/'cc1-preview-to-final-binding282.md'
DELTA_EVIDENCE_SHA='1fbcedef34f65933ff3d818740d124cb2492a39e276916eb600f6753d5c62066'
# 282 section 1: mod 236->256, rigidity247, gallery249, draw264 and the new test-only plate_fade_probe246.
DELTA_238_268=frozenset(SCENE_DIR+n for n in ('draw.rs','gallery.rs','mod.rs','plate_fade_probe.rs','rigidity.rs'))
EARLIER=('244','278')
def sha(path):
 with Path(path).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest()
def read(path):return json.loads(Path(path).read_text())
def success(path):
 r=read(path);assert r.get('finished') and r.get('exit')==0 and r['inputs_sha256']==r['final_inputs_sha256']
 assert not any(r.get(k) for k in ('error','resource_stop','stopped_for_resource_limit'))
 return r
def job_name(chapter,version):return 'csd-reviewed-final-'+chapter.replace('_','-')+version
def absolute_permit(permit):
 permit=Path(permit)
 assert permit.is_absolute() and permit==permit.resolve(strict=True),'permit path must be absolute and free of symlinks: '+str(permit)
 return permit
def preview_binding(chapter,review):
 preview=PREVIEWS[chapter]
 assert preview is not None,'preview274 names are unconfirmed in files; no launch until a later version names them from CC3 actual outputs'
 assert review['preview_job']==preview['job']
 job=D/'jobs'/preview['job'];receipt=success(job/'resource-receipt.json')
 worker=Q/preview['worker'];assert str(worker) in receipt['command'] and sha(worker)==preview['worker_sha256']
 manifest=Q/preview['manifest'];assert sha(manifest)==receipt['inputs_sha256']
 rows={x['path']:x['sha256'] for x in read(manifest)['files']}
 build=preview['build'];success(build/'resource-receipt.json')
 assert sha(preview['binary'])==preview['binary_sha256']==read(build/'qualification.json')['binary_sha256']
 frozen=[preview['binary'],worker,build/'source-copy.json',build/'qualification.json']
 for p in frozen:assert rows.get(str(p))==sha(p),'not in the preview manifest as is: '+str(p)
 tables=[job/preview['tables']/(chapter+'.'+kind+'.tsv') for kind in ('timeline','cues','cuelog')]
 for shown in tables:
  assert shown.read_bytes().replace(str(preview['narration']).encode(),str(FMD_NARRATION).encode())==(TIMELINES/shown.name).read_bytes(),'preview table differs from timeline269: '+str(shown)
 assert sha(build/'source-copy.json')==preview['source_copy_sha256'] and sha(BUILD268/'source-copy.json')==SOURCE_COPY_268_SHA
 before=read(build/'source-copy.json')['files'];after=read(BUILD268/'source-copy.json')['files']
 scene=SCENE_DIR+SCENES[chapter];assert before[scene]==after[scene],scene
 delta=[]
 if build==BUILD268:
  assert preview['binary']==BIN and preview['binary_sha256']==BINARY_SHA
 else:
  named=[x for x in review['source_binding_evidence'] if x.get('name')==DELTA_NAME]
  assert [(x['path'],x['sha256']) for x in named]==[(str(DELTA_EVIDENCE),DELTA_EVIDENCE_SHA)],'one named '+DELTA_NAME+' item for binding282 required'
  delta=[DELTA_EVIDENCE]
  differ={k for k in before.keys()|after.keys() if before.get(k)!=after.get(k)}
  assert differ==DELTA_238_268,sorted(differ^DELTA_238_268)
 return frozen+tables+[job/'resource-receipt.json',manifest,build/'resource-receipt.json',BUILD268/'source-copy.json',BUILD268/'qualification.json']+delta
def check(chapter,permit):
 assert chapter in CHAPTERS
 permit=absolute_permit(permit)
 for version in EARLIER:assert not (ROOT/'jobs'/job_name(chapter,version)).exists(),'existing same-chapter final job: '+job_name(chapter,version)
 assert sha(BIN)==BINARY_SHA
 for p in (D/'jobs/csd-raster-encoder-stop13/resource-receipt.json',D/'jobs/csd-production-timeline21/resource-receipt.json',D/'jobs/csd-production-score21/resource-receipt.json'):success(p)
 for p in (D/'jobs/csd-review-repair-build268/resource-receipt.json',D/'jobs/csd-review-repair-rebind269/resource-receipt.json',D/'jobs/csd-review-repair-timeline269/resource-receipt.json'):success(p)
 qualification=read(D/'jobs/csd-review-repair-build268/qualification.json')
 assert qualification['binary_sha256']==BINARY_SHA
 baseline=D/'jobs/csd-production-timeline21/timeline-output'
 original_narration=D/'jobs/csd-production-assembly19/narration'
 for key in qualification['chapters']:
  for suffix in ('timeline.tsv','cues.tsv','cuelog.tsv'):
   assert (TIMELINES/(key+'.'+suffix)).read_bytes()==(baseline/(key+'.'+suffix)).read_bytes().replace(str(original_narration).encode(),str(FMD_NARRATION).encode())
 assert chapter in qualification['chapters']
 score=read(D/'jobs/csd-production-score21/score21-summary.json');assert score['all_qa_passed'] and score['chapters']==qualification['chapters']
 for relative,digest in score['files'].items():assert sha(D/'jobs/csd-production-score21/score'/relative)==digest
 review=read(permit)
 assert review['binary_sha256']==BINARY_SHA and review['chapter']==chapter
 assert review['actually_reviewed_pictures'] is True and review['actually_reviewed_motion'] is True
 assert review['unresolved_defects']==[] and review['reviewer'].strip() and review['observations'].strip()
 assert review['timeline_sha256']==sha(TIMELINES/(chapter+'.timeline.tsv'))
 assert review['evidence'] and review['motion_intervals']
 assert review['source_binding_evidence'], 'Bind preview source to final binary; preview243 predates combined268'
 for item in review['source_binding_evidence']:assert sha(item['path'])==item['sha256']
 for item in review['evidence']:assert sha(item['path'])==item['sha256']
 binding=preview_binding(chapter,review)
 frames=review['frames'];assert type(frames) is int and frames>0
 header=dict(line.split('\t',1) for line in (TIMELINES/(chapter+'.timeline.tsv')).read_text().splitlines() if line.startswith(('chapter\t','fps\t','frames\t')))
 assert header == {'chapter':chapter,'fps':'60','frames':str(frames)}
 for a,b in review['motion_intervals']:assert 0<=a<b<frames
 assert read(STEMS/'qa'/(chapter+'.json'))['passed'] is True
 return review,sha(permit),binding
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
 chapter=args.chapter;review,permit_sha,binding=check(chapter,args.permit)
 job=ROOT/'jobs'/args.job
 assert job.parent==ROOT/'jobs' and args.job == job_name(chapter,'287')
 admitted=read(job/'resource-receipt.json');assert admitted['admitted'] and 'exit' not in admitted
 assert admitted['policy']['stop_free_gib']==8 and admitted['policy']['phase_growth_gib']==4
 inputs=Path(args.inputs);assert sha(inputs)==admitted['inputs_sha256']
 frozen=read(inputs)['files'];assert len(frozen)==len({x['path'] for x in frozen})
 bound={x['path']:x['sha256'] for x in frozen}
 for x in frozen:assert sha(x['path'])==x['sha256']
 required=[Path(x['path']) for x in review['source_binding_evidence']]+binding+[BIN,Path(__file__).resolve(),args.permit,TIMELINES/(chapter+'.timeline.tsv'),TIMELINES/(chapter+'.cues.tsv'),TIMELINES/(chapter+'.cuelog.tsv')]
 inherited=ROOT/'qualification01/production-repair17/review-repair-timeline269-inputs.json'
 delivery=success(ROOT/'jobs/csd-review-repair-timeline269/resource-receipt.json');assert sha(inherited)==delivery['inputs_sha256']
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
 assert check(chapter,args.permit)==(review,permit_sha,binding)
 for x in frozen:assert sha(x['path'])==x['sha256']
 preview=PREVIEWS[chapter]
 result=dict(chapter=chapter,frames=frames,binary_sha256=BINARY_SHA,permit_sha256=permit_sha,preview_job=preview['job'],preview_binary_sha256=preview['binary_sha256'],preview_binding=[dict(path=str(p),sha256=sha(p)) for p in binding],encoder=settings,files=[dict(path=str(p),sha256=sha(p)) for p in sorted(out.rglob('*')) if p.is_file()],scope='Technical scored4K60 chapter only; full decode, master replay, final pictures/motion/audio/captions and publication remain.')
 with (job/'render-summary.json').open('x') as f:json.dump(result,f,indent=2)
if __name__=='__main__':
 p=argparse.ArgumentParser();p.add_argument('--chapter',required=True);p.add_argument('--job',required=True);p.add_argument('--inputs',type=Path,required=True);p.add_argument('--permit',type=Path,required=True);p.add_argument('--threads',type=int,default=12);args=p.parse_args();assert 1<=args.threads<=16;run(args)
