"""CSD combined-repair scored 4K60 chapter worker, version 352. Prepared only.
Against 303 (preserved unchanged), only the review gate differs, as 344 did for 09 against 317, and 09
is left out (final344 renders it). 303 required actually_reviewed_motion true and motion_intervals. The
actual preview274/preview243 reviews are of still pictures: root's full sampled pass, framewise still
windows, and independent sampled or framewise stills. None of them is continuous motion. So 352 takes
344's sampled-preview disposition, for a technical 4K render only:
 - the permit has exactly 344's PERMIT_KEYS: no actually_reviewed_motion and no motion_intervals, six
   acceptance flags stated false, a review_scope and a named independent review (344's independent());
 - sampled_review_scope() is 344's, byte for byte;
 - review_basis() reads the viewed frames from the picture records the permit binds (format in its
   docstring). It counts only frames of the chapter's exact preview movie (MOVIES), and only from a record
   that a bound item declares it views ('views') and whose file cites it. Root evidence must view the
   chapter's own preview sampling.json. An independent_review file must view a record and name the movie.
   playback_seconds must be [], since 352 rests on no playback record.
A 303 job of the chapter is refused, like 244/278/287, so root decides. Every 303 encoder, resource,
source, table, score, binary, narration and preview gate is kept unchanged. This is not publication
authorization, and it accepts no final picture, motion, audio, fine-sync or caption review.

303's description, which still applies:
Requires a root-authored actual visual/motion review record and frozen inputs.
Never creates its own review authorization or launches another job.
Against 287 (preserved), 303 names CC3's real preview274 (job, manifest, worker and
receipt inputs as read on csd on 2026-11-08), so the nine 274 chapters can bind to it
instead of failing closed; pins each preview receipt's inputs hash; refuses an existing
244/278/287 job for the same chapter; and requires the permit to name host csd. Every
287 source/table/picture/motion/score/resource gate is kept unchanged.
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
CHAPTERS=('01_the_bump', '04_reading_color', '05_gallery', '06_one_derivative', '07_one_point', '08_loop_sums', '10_rigidity', '11_counting', '12_residues', '14_continuation')
# 352: each chapter's preview movie as reviewed (<preview job>/preview/<chapter>.mp4), read on csd on
# 2026-11-08 and equal to the reviewed copies; it is also the movie_sha256 of the preview's own sampling.json.
MOVIES={'01_the_bump':'1f6834a6b6fb5ed481f66c0a8eded204ede670501eae5d81a277814206353d19',
 '04_reading_color':'8836b7e6d87b170d9cb7327fd51fd3e93fb7b5bfd2b2a89605d7cc1ed2d7e890',
 '05_gallery':'f5123300084eba264276377f36518d263324bb9e28dd5df0f77ecf5f20cf5e48',
 '06_one_derivative':'5e45d72da91701c9c7c346b98ded8491c13ba1befed6be1b6301c1c6d33ed62d',
 '07_one_point':'2517772cb632db4eafe5be1ed266e6e8aa596778d7f3742ecdab8fa5d204b7b0',
 '08_loop_sums':'1ac1ee716f6aa383a9154d6305ffa571bd555df818565d28a07084eb4b5364d6',
 '10_rigidity':'78eb267faa780b2a4d38804deede56088852fed606ac544b7d05f02300672562',
 '11_counting':'0b6244a005f64ff9b9b381a8ec8a58ffb7c174d085aceb452871d7f5dd0f811e',
 '12_residues':'a78c20024c257eaa4783db80f9b99c23442fe67ac8f646e354e633074f842199',
 '14_continuation':'a0d34489e9757ff3db96e3d8b7a438845a90036453b2d4d7c9426efd8dc7c432'}
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
 build=D/'jobs/csd-review-repair-build238',source_copy_sha256='6d4c7e93e89a29930cf5b4c2f3101fba023a3e09b99888f3e9db083bdb1eb65c',
 # Read from csd's preview243 resource receipt on 2026-11-08 (inputs = final inputs).
 inputs_sha256='6510db7856deec2abfea54d585000ec09d0cf5334ff32b558adb71172a8dcf91')
# build268 source copy as read by cc1-preview-to-final-binding282.md (section "Bound records").
SOURCE_COPY_268_SHA='81d28272a7a23b63199e79e6aa1086f01f131f0ddd685d4ae859d04b1de1934c'
# The preview actually rendered for the other nine chapters: CC3's cc3-preview-fade274.py:3-15
# (job folder, preview/ tables, rebind269 narration, binary 268, build268), run by root's
# launch-reviewed-preview296.py, which wrote this manifest (12,669 rows). Read on csd on
# 2026-11-08: receipt finished 10:50:17 UTC, exit 0, inputs = final inputs = 20d91bdb...;
# its command holds this worker path; the manifest holds binary 268, this worker and build268's
# source-copy.json and qualification.json at their current hashes; all 27 preview tables are
# byte-equal to timeline269's (the narration folder is already rebind269, so the mapping is
# the identity). Same binary and build as the final, so no 238-to-268 delta item is needed.
PREVIEW274=dict(job='csd-review-repair-preview274',manifest='review-repair-preview274-inputs.json',
 worker='cc3-preview-fade274.py',worker_sha256='d80cf7b6ffba56968f7c6dfd7a2cca50e65769dce1c97deb5d34683ed3fc9669',
 tables='preview',narration=D/'jobs/csd-review-repair-rebind269/narration',
 binary=BIN,binary_sha256=BINARY_SHA,build=BUILD268,source_copy_sha256=SOURCE_COPY_268_SHA,
 inputs_sha256='20d91bdb8585fb229c105c717a0cf486e544b3b37b92f46907e009fe695fb38c')
PREVIEWS={c:(PREVIEW243 if c in ('06_one_derivative','08_loop_sums') else PREVIEW274) for c in CHAPTERS}
# CC2 review 300 L4: one chapter, one host. CSS's worker cannot see CSD jobs, so a permit for a
# CSD render must say so.
HOST='csd'
# Named 238-to-268 delta evidence: CC1's binding282 (root read it in full). It is source-level
# carry-forward only: no 540p pixel, audio or motion equivalence, and its schedule half stays
# conditional on timeline269's actual tables, which check() and preview_binding() assert.
DELTA_NAME='build238-to-build268-delta'
DELTA_EVIDENCE=Q/'cc1-preview-to-final-binding282.md'
DELTA_EVIDENCE_SHA='1fbcedef34f65933ff3d818740d124cb2492a39e276916eb600f6753d5c62066'
# 282 section 1: mod 236->256, rigidity247, gallery249, draw264 and the new test-only plate_fade_probe246.
DELTA_238_268=frozenset(SCENE_DIR+n for n in ('draw.rs','gallery.rs','mod.rs','plate_fade_probe.rs','rigidity.rs'))
EARLIER=('244','278','287','303')
VERSION='352'
# 344's permit fields, exactly (csd-reviewed-boundary-final344.py PERMIT_KEYS).
PERMIT_KEYS={'chapter','host','binary_sha256','preview_job','frames','timeline_sha256','actually_reviewed_pictures',
 'unresolved_defects','reviewer','observations','evidence','source_binding_evidence','independent_reviewer',
 'independent_review','review_scope','continuous_motion_acceptance','listening_acceptance','fine_sync_acceptance',
 'final_4k_acceptance','release_acceptance','publication_acceptance'}
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
def sampled_review_scope(review, frames, movie_sha256, sampled):
    """344's review scope, the part shared with later workers and masters. The permit states what was
    actually reviewed, never a motion boolean. It authorizes a technical 4K render from a sampled-preview
    disposition only, with final-picture, motion and audio review still open. Any key that claims an
    acceptance, a release or a publication and is not exactly false is refused.
    review: the permit; frames: the chapter's frames (60 fps); movie_sha256: the preview movie that was
    reviewed; sampled: the frame numbers of that preview's own sampled pictures. Returns the sampled
    intervals. Builtins only, so a later worker or master copies it unchanged and proves that by AST."""
    assert 'actually_reviewed_motion' not in review and 'motion_intervals' not in review, \
        'motion is stated by review_scope, not by actually_reviewed_motion or motion_intervals'
    def claims(value, at):
        if type(value) is dict:
            for key, item in value.items():
                if any(word in key.lower() for word in ('accept', 'release', 'publication')) and item is not False:
                    yield at+key
                yield from claims(item, at+key+'.')
        elif type(value) is list:
            for n, item in enumerate(value):
                yield from claims(item, at+str(n)+'.')
    found = list(claims(review, ''))
    assert not found, 'the permit claims an acceptance, release or publication: '+', '.join(found)
    for key in ('continuous_motion_acceptance', 'listening_acceptance', 'fine_sync_acceptance',
                'final_4k_acceptance', 'release_acceptance', 'publication_acceptance'):
        assert review.get(key) is False, key+' must be stated false'
    scope = review['review_scope']
    assert type(scope) is dict and set(scope) == {'basis', 'purpose', 'preview_movie_sha256', 'sampled_intervals',
                                                   'playback_seconds', 'open_reviews'}, 'review_scope fields'
    assert (scope['basis'], scope['purpose']) == ('sampled_preview', 'technical_4k_render'), 'not a sampled-preview technical 4K scope'
    assert scope['preview_movie_sha256'] == movie_sha256, 'review_scope names another preview movie'
    reviews = scope['open_reviews']
    assert type(reviews) is list and len(set(reviews)) == len(reviews) and all(type(x) is str for x in reviews)
    assert {'final_picture', 'motion', 'audio'} <= set(reviews), 'final-picture, motion and audio review must stay open'
    seconds = scope['playback_seconds']
    assert type(seconds) is list and all(type(x) in (int, float) and 0 <= x < frames/60 for x in seconds), 'playback_seconds'
    intervals = scope['sampled_intervals']
    assert type(intervals) is list and intervals, 'no sampled intervals'
    end = -1
    for pair in intervals:
        assert type(pair) is list and len(pair) == 2 and all(type(x) is int for x in pair), 'sampled interval %r' % (pair,)
        a, b = pair
        assert end < a < b < frames, 'sampled intervals must be sorted, disjoint and inside the chapter: %r' % (pair,)
        missing = [n for n in range(a, b+1) if n not in sampled]
        assert not missing, 'sampled interval [%d, %d] has unsampled frames, first %d' % (a, b, missing[0])
        end = b
    return [tuple(pair) for pair in intervals]
def independent(review):
 """344's: the permit names an independent review: someone other than its reviewer, files in Q at their hashes."""
 assert review['independent_reviewer'].strip() and review['independent_reviewer'].strip()!=review['reviewer'].strip()
 items=review['independent_review'];assert items
 for item in items:
  path=Path(item['path']);assert path.is_absolute() and path.parent==Q and path==path.resolve(strict=True) and sha(path)==item['sha256'],item['path']
 return [Path(item['path']) for item in items]
def review_basis(chapter,review):
 """352: one chapter's sampled-preview basis. It is read only from the pinned preview movie and from the
 files the permit binds in evidence and independent_review, each at its stated hash. It only reads, so it
 gives the same result on csd and, through the alias, on css.
 A frame record is a bound JSON object whose movie hash is this chapter's MOVIES hash. The hash sits
 under 'movie_sha256', 'source_sha256' or 'sha256', whichever of these keys comes first in that order.
 The record lists frames as 'samples' [{'frame': n}], 'frames' [n or {'frame': n}] or 'pictures'
 [{'frame': n}], read in that order.
 A frame record is viewed only when root says so in the permit. A bound item's 'views' lists the sha256
 of each frame record whose pictures that item's file records viewing. The file must cite each one, either
 by containing its sha256 or, for a 'samples' record, by containing the sha256 of every item of its
 'sheets' ({'path', 'sha256'}), each sheet being at that hash. Merely mentioning a hash views nothing.
 sampled is the set of frames of the viewed records, and runs are its runs of two or more consecutive
 frames. A bound frame record that no item views adds nothing and is listed as unviewed. An
 independent_review file confirms the chapter when it views a record and contains the movie hash."""
 movie=D/'jobs'/PREVIEWS[chapter]['job']/'preview'/(chapter+'.mp4')
 assert sha(movie)==MOVIES[chapter],'not the preview movie that was reviewed: '+str(movie)
 bound=review['evidence']+review['independent_review']
 items={x['path']:x['sha256'] for x in bound}
 data={}
 for path,digest in items.items():
  data[path]=Path(path).read_bytes();assert hashlib.sha256(data[path]).hexdigest()==digest,'not at the hash the permit binds: '+path
 found={}
 for path,raw in data.items():
  try:record=json.loads(raw)
  except ValueError:continue
  if type(record) is not dict:continue
  key=next((k for k in ('movie_sha256','source_sha256','sha256') if k in record),None)
  if key is None or record[key]!=MOVIES[chapter]:continue
  for name in ('samples','frames','pictures'):
   if type(record.get(name)) is list:
    frames=[x['frame'] if type(x) is dict else x for x in record[name]]
    assert name=='frames' or all(type(x) is dict for x in record[name]),'frame record items: '+path
    assert frames and all(type(n) is int for n in frames),'frame record without integer frames: '+path
    found[path]=(record,name,frames);break
 named={items[p]:p for p in found};by={p:set() for p in found};sheets=[]
 for x in bound:
  views=x.get('views',[]);assert type(views) is list and all(type(d) is str for d in views),'views must list sha256 strings: '+x['path']
  for digest in views:
   assert digest in named,'views names no bound frame record of '+chapter+': '+digest
   record,name,_=found[named[digest]];raw=data[x['path']]
   if digest.encode() not in raw:
    shown=[(s['path'],s['sha256']) for s in record.get('sheets') or []] if name=='samples' else []
    assert shown and all(d.encode() in raw for _,d in shown),x['path']+' does not cite the record it views: '+digest
    for p,d in shown:assert sha(p)==d,'sheet not at its hash: '+p
    sheets+=[p for p,_ in shown if p not in sheets]
   by[named[digest]].add(x['path'])
 records,unviewed,sampled=[],[],set()
 for path,(record,name,frames) in found.items():
  if not by[path]:unviewed.append(path);continue
  sampled.update(frames)
  records.append(dict(path=path,sha256=items[path],frames=len(set(frames)),viewed_by=sorted(by[path])))
 confirmed=sorted(x['path'] for x in review['independent_review'] if x.get('views') and MOVIES[chapter].encode() in data[x['path']])
 runs=[]
 for n in sorted(sampled):
  if runs and n==runs[-1][1]+1:runs[-1][1]=n
  else:runs.append([n,n])
 return dict(chapter=chapter,preview_job=PREVIEWS[chapter]['job'],movie=dict(path=str(movie),sha256=MOVIES[chapter]),
  frame_records=records,unviewed=sorted(unviewed),independent=confirmed,sampled=sorted(sampled),runs=[r for r in runs if r[0]<r[1]],sheets=sheets)
def preview_binding(chapter,review):
 preview=PREVIEWS[chapter]
 assert review['preview_job']==preview['job']
 job=D/'jobs'/preview['job'];receipt=success(job/'resource-receipt.json')
 assert receipt['inputs_sha256']==preview['inputs_sha256'],'not the preview run that was read: '+preview['job']
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
 # 352: the sampled-preview scope, on this preview's movie and the frames the bound records viewed.
 basis=review_basis(chapter,review)
 sampling=job/chapter/'sampling.json'
 evidence={x['path'] for x in review['evidence']}
 assert any(r['path']==str(sampling) and evidence&set(r['viewed_by']) for r in basis['frame_records']),'the permit must bind '+str(sampling)+' and root evidence that views it'
 assert basis['independent'],'no independent_review file views a frame record of this preview and names its movie: '+chapter
 assert basis['runs'],'no two consecutive viewed frames of '+chapter+': a sampled interval needs a framewise picture record'
 sampled_review_scope(review,review['frames'],MOVIES[chapter],set(basis['sampled']))
 assert review['review_scope']['playback_seconds']==[],'352 rests on no playback record: playback_seconds must be []'
 return frozen+tables+[job/'resource-receipt.json',manifest,build/'resource-receipt.json',BUILD268/'source-copy.json',BUILD268/'qualification.json']+delta+[Path(basis['movie']['path'])]+[Path(p) for p in basis['sheets']]
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
 assert set(review)==PERMIT_KEYS,'permit fields are not the 344 set; differing: %s'%sorted(set(review)^PERMIT_KEYS)
 assert review['binary_sha256']==BINARY_SHA and review['chapter']==chapter
 assert review.get('host')==HOST,'permit must name host csd (one chapter, one host)'
 assert review['actually_reviewed_pictures'] is True
 assert review['unresolved_defects']==[] and review['reviewer'].strip() and review['observations'].strip()
 assert review['timeline_sha256']==sha(TIMELINES/(chapter+'.timeline.tsv'))
 assert review['evidence']
 assert review['source_binding_evidence'], 'Bind preview source to final binary; preview243 predates combined268'
 for item in review['source_binding_evidence']:assert sha(item['path'])==item['sha256']
 for item in review['evidence']:assert sha(item['path'])==item['sha256']
 independent(review)
 binding=preview_binding(chapter,review)
 frames=review['frames'];assert type(frames) is int and frames>0
 header=dict(line.split('\t',1) for line in (TIMELINES/(chapter+'.timeline.tsv')).read_text().splitlines() if line.startswith(('chapter\t','fps\t','frames\t')))
 assert header == {'chapter':chapter,'fps':'60','frames':str(frames)}
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
 assert job.parent==ROOT/'jobs' and args.job == job_name(chapter,VERSION)
 admitted=read(job/'resource-receipt.json');assert admitted['admitted'] and 'exit' not in admitted
 assert admitted['policy']['stop_free_gib']==8 and admitted['policy']['phase_growth_gib']==4
 inputs=Path(args.inputs);assert sha(inputs)==admitted['inputs_sha256']
 frozen=read(inputs)['files'];assert len(frozen)==len({x['path'] for x in frozen})
 bound={x['path']:x['sha256'] for x in frozen}
 for x in frozen:assert sha(x['path'])==x['sha256']
 required=[Path(x['path']) for x in review['source_binding_evidence']]+binding+independent(review)+[BIN,Path(__file__).resolve(),args.permit,TIMELINES/(chapter+'.timeline.tsv'),TIMELINES/(chapter+'.cues.tsv'),TIMELINES/(chapter+'.cuelog.tsv')]
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
 preview=PREVIEWS[chapter];basis=review_basis(chapter,review)
 result=dict(chapter=chapter,frames=frames,binary_sha256=BINARY_SHA,permit_sha256=permit_sha,host=HOST,preview_job=preview['job'],preview_inputs_sha256=preview['inputs_sha256'],preview_binary_sha256=preview['binary_sha256'],preview_binding=[dict(path=str(p),sha256=sha(p)) for p in binding],independent_review=review['independent_review'],review_scope=review['review_scope'],review_basis={k:basis[k] for k in ('movie','frame_records','independent','runs')},encoder=settings,files=[dict(path=str(p),sha256=sha(p)) for p in sorted(out.rglob('*')) if p.is_file()],scope='Technical scored4K60 chapter only, authorized from a sampled-preview disposition (review_scope); final-picture, motion and audio review remain open, as do full decode, master replay, captions and publication. Not publication authorization.')
 with (job/'render-summary.json').open('x') as f:json.dump(result,f,indent=2)
if __name__=='__main__':
 p=argparse.ArgumentParser();p.add_argument('--chapter',required=True);p.add_argument('--job',required=True);p.add_argument('--inputs',type=Path,required=True);p.add_argument('--permit',type=Path,required=True);p.add_argument('--threads',type=int,default=12);args=p.parse_args();assert 1<=args.threads<=16;run(args)
