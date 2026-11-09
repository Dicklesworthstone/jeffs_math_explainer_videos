"""CSD scored 4K60 worker for 09_boundary under build313, version 317. Prepared only.
Requires a root-authored actual visual/motion review record of preview313, a named independent
review and frozen inputs. Never creates its own review authorization or launches another job.
Against 303 (preserved): chapter 09_boundary only; binary 313 = build268 plus exactly
boundary-guide-repair310.rs, its hash taken from the completed build313 record (never written
here) and matched by the permit; narration rebind313, whose 105 WAVs are byte-identical to
rebind269's and assembly19's (Frederick takes retained, nothing synthesized); tables timeline313,
re-checked here as byte-equal to timeline269 and production timeline21 under the named narration
mappings; preview313 instead of preview274. build313, rebind313, timeline313 and preview313 must
be successful finished jobs whose bound inputs still hold. The permit must also name an independent
review, bind preview313's own sampling and zoom-probe records, and cover the measured zoom-line
exit frame with its motion intervals. An existing 303 09 job (old boundary) is refused like
244/278/287, so root decides. Every 303 encoder, resource, score, table and permit gate is kept unchanged.
"""
import argparse, hashlib, json, os, struct, subprocess, wave
from pathlib import Path
ROOT=Path('/data/projects/jmev-stroke-probe-20261106')
D=Path('/data/projects/jmev-stroke-probe-20261106')
Q=ROOT/'qualification01/production-repair17'
CHAPTER='09_boundary'
CHAPTERS=(CHAPTER,)
FRAMES=16504
BIN=D/'work/bin/review-repair313'
BUILD313=D/'jobs/csd-review-repair-build313'
REBIND313=D/'jobs/csd-review-repair-rebind313'
TIMELINE313=D/'jobs/csd-review-repair-timeline313'
PREVIEW313=D/'jobs/csd-review-repair-preview313'
# guard inputs of each 313 job, as cc0's launchers write them
MANIFESTS={'csd-review-repair-build313':'review-repair-build313-inputs.json','csd-review-repair-rebind313':'review-repair-rebind313-inputs.json',
 'csd-review-repair-timeline313':'review-repair-timeline313-inputs.json','csd-review-repair-preview313':'review-repair-preview313-inputs.json'}
PREVIEW_WORKER='review-repair-preview313.py'
# cc0's preview313 worker as pinned by preview-after-timeline313.py:19
PREVIEW_WORKER_SHA='67d780d83bf05fbcd0285f0599197649f8ce8813ef6599a5da932301dc354e50'
TIMELINES=TIMELINE313/'timeline-output'
NARRATION=ROOT/'jobs/csd-review-repair-rebind313/narration'
# timeline313 already names rebind313, so verify_tables' narration mapping is the identity.
FMD_NARRATION=D/'jobs/csd-review-repair-rebind313/narration'
TIMELINE269=D/'jobs/csd-review-repair-timeline269/timeline-output'
NARRATION269=D/'jobs/csd-review-repair-rebind269/narration'
BASELINE=D/'jobs/csd-production-timeline21/timeline-output'
ORIGINAL_NARRATION=D/'jobs/csd-production-assembly19/narration'
METADATA={'external-units.json','report.json','assembly.json'}
STEMS=D/'jobs/csd-production-score21/score/stems'
BUILD268=D/'jobs/csd-review-repair-build268'
BINARY_268='983954c508e42b90d443ad6ca09f2baba7565a4f609d09d2ea821640534d6ec7'
SOURCE_COPY_268_SHA='81d28272a7a23b63199e79e6aa1086f01f131f0ddd685d4ae859d04b1de1934c'
CRATE='explainer_corpus/complex_analysis_visual/'
BOUNDARY=CRATE+'src/chapters/boundary.rs'
OLD_BOUNDARY_SHA='dc80c541799209f10d02701d18c7fab5a14327000719929e5e2fad0c6fd62f9f'
REPAIR310='boundary-guide-repair310.rs'
REPAIR310_SHA='bda7d931498b23bd3203cd0c6106de69dc0a8c5e160e738d698376322ff39e53'
WORKSPACE313=D/'work/review-repair313/workspace'
# Binary 313 reads the script and runs the delivery contract from its CARGO_MANIFEST_DIR, the crate
# in WORKSPACE313 (as cc0-combined-master293.py:56-61 records for 268); rebind313 then rewrote
# narration_units.json there and added the review313 map (review-repair-rebind313.py:111-127).
CONTRACT=(CRATE+'script.tsv','tools/narration_delivery.py','tools/coherent_narration.py')
UNITS=CRATE+'narration_units.json'
MAP313=CRATE+'docs/narration-direction-map-frederick-review313.json'
HOST='csd'
EARLIER=('244','278','287','303')
VERSION='317'
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
def completed(job):
 """A finished 313 job: successful receipt, its own guard inputs at the hash it bound, every row unchanged."""
 receipt=success(D/'jobs'/job/'resource-receipt.json')
 manifest=Q/MANIFESTS[job];assert sha(manifest)==receipt['inputs_sha256'],'not the inputs '+job+' bound'
 rows={x['path']:x['sha256'] for x in read(manifest)['files']}
 for path,digest in rows.items():assert sha(path)==digest,'bound input of '+job+' changed: '+path
 return receipt,manifest,rows
def build313():
 """Binary 313 is build268's source with exactly one substitution, repair310's boundary.rs."""
 completed('csd-review-repair-build313')
 q=read(BUILD313/'qualification.json');copied=read(BUILD313/'source-copy.json')
 binary=q['binary_sha256'];assert sha(BIN)==binary and binary!=BINARY_268 and q['base_binary_sha256']==BINARY_268
 assert q['tests_passed']==233 and q['explicit_native_plate_fade_probe_passed'] is True and q['clippy_passed'] is True and q['fmt_check_passed'] is True
 assert q['changed']==copied['changed']==[BOUNDARY] and copied['base_source_copy_sha256']==SOURCE_COPY_268_SHA
 assert sha(BUILD268/'source-copy.json')==SOURCE_COPY_268_SHA
 before,after=read(BUILD268/'source-copy.json')['files'],copied['files']
 assert set(before)==set(after) and {k for k in before if before[k]!=after[k]}=={BOUNDARY}
 assert (before[BOUNDARY],after[BOUNDARY])==(OLD_BOUNDARY_SHA,REPAIR310_SHA)==(OLD_BOUNDARY_SHA,sha(Q/REPAIR310))
 for relative in CONTRACT+(BOUNDARY,):assert sha(WORKSPACE313/relative)==after[relative],relative
 q268=read(BUILD268/'qualification.json');assert q['chapters']==q268['chapters'] and q['facts_sha256']==q268['facts_sha256']
 return binary,q
def narration313():
 """Rebind313's folder: rebind269's and assembly19's files, every WAV byte-identical; only the three metadata files differ."""
 completed('csd-review-repair-rebind313')
 summary=read(REBIND313/'rebind-summary.json')
 assert summary['wav_files_byte_identical']==summary['wav_files_equal_rebind269']==105
 names={p.name for p in NARRATION.iterdir()}
 assert names=={p.name for p in NARRATION269.iterdir()}=={p.name for p in ORIGINAL_NARRATION.iterdir()}
 wavs=sorted(n for n in names if n.endswith('.wav'));assert len(wavs)==105
 for name in wavs:assert sha(NARRATION/name)==sha(NARRATION269/name)==sha(ORIGINAL_NARRATION/name),name
 assert {n for n in names if sha(NARRATION/n)!=sha(NARRATION269/n)}<=METADATA
 assert sha(WORKSPACE313/UNITS)==summary['canonical_sha256'] and sha(WORKSPACE313/MAP313)==summary['map_sha256']
 return [NARRATION/n for n in sorted(names)]+[NARRATION269/n for n in sorted(names)]+[ORIGINAL_NARRATION/n for n in sorted(names)]
def schedule313():
 """The full 09 schedule: timeline313's three tables equal timeline269's and production timeline21's under the named mappings."""
 completed('csd-review-repair-timeline313')
 comparison=read(TIMELINE313/'comparison.json')
 assert comparison['frames']=={CHAPTER:FRAMES}
 assert comparison['named_path_mapping']=={str(NARRATION269):str(NARRATION)}
 assert comparison['production_path_mapping']=={str(ORIGINAL_NARRATION):str(NARRATION)}
 rows={(r['chapter'],r['kind']):r for r in comparison['tables']};assert len(rows)==len(comparison['tables'])==3
 for kind in ('timeline','cues','cuelog'):
  name=CHAPTER+'.'+kind+'.tsv';new=(TIMELINES/name).read_bytes()
  assert new==(TIMELINE269/name).read_bytes().replace(str(NARRATION269).encode(),str(NARRATION).encode()),'timeline313 differs from timeline269: '+name
  assert new==(BASELINE/name).read_bytes().replace(str(ORIGINAL_NARRATION).encode(),str(NARRATION).encode()),'timeline313 differs from timeline21: '+name
  row=rows[(CHAPTER,kind)];assert row['new_sha256']==hashlib.sha256(new).hexdigest()
  assert row['equal_to_timeline269_after_named_narration_path_mapping'] is True and row['equal_to_production21_after_named_narration_path_mapping'] is True
 return [TIMELINE313/'comparison.json']+[d/(CHAPTER+'.'+k+'.tsv') for d in (TIMELINES,TIMELINE269,BASELINE) for k in ('timeline','cues','cuelog')]
def independent(review):
 """The permit names an independent review: someone other than its reviewer, files in Q at their hashes."""
 assert review['independent_reviewer'].strip() and review['independent_reviewer'].strip()!=review['reviewer'].strip()
 items=review['independent_review'];assert items
 for item in items:
  path=Path(item['path']);assert path.is_absolute() and path.parent==Q and path==path.resolve(strict=True) and sha(path)==item['sha256'],item['path']
 return [Path(item['path']) for item in items]
def preview_binding(chapter,review):
 """Preview313 is the run root watched: same binary and build as this final, its 09 tables equal timeline313's."""
 assert chapter==CHAPTER and review['preview_job']==PREVIEW313.name
 receipt,manifest,rows=completed(PREVIEW313.name)
 worker=Q/PREVIEW_WORKER;assert str(worker) in receipt['command'] and rows.get(str(worker))==sha(worker)==PREVIEW_WORKER_SHA
 frozen=[BIN,worker,BUILD313/'source-copy.json',BUILD313/'qualification.json']
 for p in frozen:assert rows.get(str(p))==sha(p),'not in the preview manifest as is: '+str(p)
 tables=[PREVIEW313/'preview'/(chapter+'.'+kind+'.tsv') for kind in ('timeline','cues','cuelog')]
 for shown in tables:assert shown.read_bytes()==(TIMELINES/shown.name).read_bytes(),'preview table differs from timeline313: '+str(shown)
 summary=read(PREVIEW313/'summary.json');assert summary['binary_sha256']==sha(BIN)
 found=summary['chapters'][chapter];assert found['frames']==FRAMES and found['zoom_probe_fails']==[] and found['zoom_probe_exit_frame'] is not None
 movie=PREVIEW313/'preview'/(chapter+'.mp4');sampling=PREVIEW313/chapter/'sampling.json';probe=PREVIEW313/chapter/'zoom-probe.json'
 assert read(sampling)['movie_sha256']==read(probe)['movie_sha256']==sha(movie)
 # The acceptance is of these preview313 records: the permit binds them, and its motion covers the measured zoom-line exit (312 L2).
 evidence={item['path']:item['sha256'] for item in review['evidence']}
 assert evidence.get(str(sampling))==sha(sampling) and evidence.get(str(probe))==sha(probe),'permit evidence must bind preview313 09 sampling.json and zoom-probe.json'
 exit_frame=found['zoom_probe_exit_frame'];assert any(a<=exit_frame<=b for a,b in review['motion_intervals']),'motion review must cover the zoom-line exit frame %d'%exit_frame
 return frozen+tables+[PREVIEW313/'resource-receipt.json',manifest,PREVIEW313/'summary.json',movie,sampling,probe,BUILD313/'resource-receipt.json',BUILD268/'source-copy.json',BUILD268/'qualification.json']
def check(chapter,permit):
 assert chapter in CHAPTERS
 permit=absolute_permit(permit)
 for version in EARLIER:assert not (ROOT/'jobs'/job_name(chapter,version)).exists(),'existing same-chapter final job: '+job_name(chapter,version)
 binary,qualification=build313()
 for p in (D/'jobs/csd-raster-encoder-stop13/resource-receipt.json',D/'jobs/csd-production-timeline21/resource-receipt.json',D/'jobs/csd-production-score21/resource-receipt.json'):success(p)
 for p in (D/'jobs/csd-review-repair-build268/resource-receipt.json',D/'jobs/csd-review-repair-rebind269/resource-receipt.json',D/'jobs/csd-review-repair-timeline269/resource-receipt.json'):success(p)
 assert read(D/'jobs/csd-review-repair-build268/qualification.json')['binary_sha256']==BINARY_268
 for key in qualification['chapters']:
  for suffix in ('timeline.tsv','cues.tsv','cuelog.tsv'):
   assert (TIMELINE269/(key+'.'+suffix)).read_bytes()==(BASELINE/(key+'.'+suffix)).read_bytes().replace(str(ORIGINAL_NARRATION).encode(),str(NARRATION269).encode())
 assert chapter in qualification['chapters']
 narration313()
 schedule313()
 score=read(D/'jobs/csd-production-score21/score21-summary.json');assert score['all_qa_passed'] and score['chapters']==qualification['chapters']
 for relative,digest in score['files'].items():assert sha(D/'jobs/csd-production-score21/score'/relative)==digest
 review=read(permit)
 assert review['binary_sha256']==binary and review['chapter']==chapter
 assert review.get('host')==HOST,'permit must name host csd (one chapter, one host)'
 assert review['actually_reviewed_pictures'] is True and review['actually_reviewed_motion'] is True
 assert review['unresolved_defects']==[] and review['reviewer'].strip() and review['observations'].strip()
 assert review['timeline_sha256']==sha(TIMELINES/(chapter+'.timeline.tsv'))
 assert review['evidence'] and review['motion_intervals']
 assert review['source_binding_evidence'], 'Bind the preview313 source to the final binary'
 for item in review['source_binding_evidence']:assert sha(item['path'])==item['sha256']
 for item in review['evidence']:assert sha(item['path'])==item['sha256']
 independent(review)
 binding=preview_binding(chapter,review)
 frames=review['frames'];assert type(frames) is int and frames>0 and frames==FRAMES
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
 binary=review['binary_sha256']
 job=ROOT/'jobs'/args.job
 assert job.parent==ROOT/'jobs' and args.job == job_name(chapter,VERSION)
 admitted=read(job/'resource-receipt.json');assert admitted['admitted'] and 'exit' not in admitted
 assert admitted['policy']['stop_free_gib']==8 and admitted['policy']['phase_growth_gib']==4
 inputs=Path(args.inputs);assert sha(inputs)==admitted['inputs_sha256']
 frozen=read(inputs)['files'];assert len(frozen)==len({x['path'] for x in frozen})
 bound={x['path']:x['sha256'] for x in frozen}
 for x in frozen:assert sha(x['path'])==x['sha256']
 required=[Path(x['path']) for x in review['source_binding_evidence']]+binding+independent(review)+[BIN,Path(__file__).resolve(),args.permit,TIMELINES/(chapter+'.timeline.tsv'),TIMELINES/(chapter+'.cues.tsv'),TIMELINES/(chapter+'.cuelog.tsv')]
 inherited=ROOT/'qualification01/production-repair17/review-repair-timeline313-inputs.json'
 delivery=success(ROOT/'jobs/csd-review-repair-timeline313/resource-receipt.json');assert sha(inherited)==delivery['inputs_sha256']
 required += [inherited]+[Path(x['path']) for x in read(inherited)['files']]
 required += [p for p in NARRATION.rglob('*') if p.is_file()]
 required += narration313()+schedule313()
 required += [WORKSPACE313/r for r in CONTRACT+(BOUNDARY,UNITS,MAP313)]+[Q/REPAIR310]
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
 preview=success(PREVIEW313/'resource-receipt.json')
 result=dict(chapter=chapter,frames=frames,binary_sha256=binary,build313_qualification_sha256=sha(BUILD313/'qualification.json'),boundary_sha256=REPAIR310_SHA,permit_sha256=permit_sha,host=HOST,preview_job=PREVIEW313.name,preview_inputs_sha256=preview['inputs_sha256'],preview_binary_sha256=binary,preview_binding=[dict(path=str(p),sha256=sha(p)) for p in binding],independent_review=review['independent_review'],encoder=settings,files=[dict(path=str(p),sha256=sha(p)) for p in sorted(out.rglob('*')) if p.is_file()],scope='Technical scored4K60 chapter only; full decode, master replay, final pictures/motion/audio/captions and publication remain.')
 with (job/'render-summary.json').open('x') as f:json.dump(result,f,indent=2)
if __name__=='__main__':
 p=argparse.ArgumentParser();p.add_argument('--chapter',required=True);p.add_argument('--job',required=True);p.add_argument('--inputs',type=Path,required=True);p.add_argument('--permit',type=Path,required=True);p.add_argument('--threads',type=int,default=12);args=p.parse_args();assert 1<=args.threads<=16;run(args)
