"""PREPARED, NOT LAUNCHED: extract one repaired chapter's final 4K pictures (01, 08, 09 or 11; 969 route).

Retains exactly the admitted preview extraction (the frames root viewed; they hold the plan02/counting02
sampling list and, for 09, all 814 prior775 frames) and adds the actual final render's lint anchors and
persistence, as 969 did. Same chapter table and run lineage: CHAPTERS and CHAINS come from the runner
the final job's guard bound. The record keeps pictures969.json's fields, so readers of that record can
read this one. Extraction only: no picture viewing, listening, fine-sync or AV acceptance, and no review
scope (root M1: a later step derives scope from the actual viewing of this extraction).

CC0 successor of CC3's 5fa242b8: the preview admission's evidence is checked by the bound runner's own
review_checks() (runner 7cafacfa), so 09's REVIEW_ROUTES admission (complete independent and author
viewing under root's explicit decision; no whole root viewing) is read as the runner reads it.
"""
import argparse
import hashlib
import importlib.util
import json
from pathlib import Path
import re
import signal
import sys
from PIL import Image, ImageDraw

R = Path('/data/projects/jmev-stroke-probe-20261106')
Q = R/'qualification01/production-repair17'
RUNNER = Q/'csd-native-repair-final01-run.py'
HELPER = Q/'csd-native-repair-pictures01.py'
PICTURES = 'csd-native-repair-pictures01-'
BASE761_SHA = '32bf6d9e08c1f21f2445debfd46ba090e0374007e4da5516310cb122774099e5'
POLICY = dict(admission_gib=10, stop_free_gib=8, phase_growth_gib=4,
              work_cap_gib=56, target_growth_cap_gib=20, sample_seconds=10)
CAP = 4000      # 969 line 93 had 1500; 09's retained preview set alone is above 1500 (plan02 735 + prior775 814)
# The folder each chapter's timeline and preview helpers normalized the narration column to (EXPECTED[2]).
NORMALIZE_TO = {
    '01_the_bump': '/data/projects/jmev-css-render-worker-20261107/jobs/css-repair-delivery309/narration',
    '08_loop_sums': '/data/projects/jmev-css-render-worker-20261107/jobs/css-repair-delivery309/narration',
    '09_boundary': '/data/projects/jmev-stroke-probe-20261106/jobs/csd-review-repair-rebind313/narration',
    '11_counting': '/data/projects/jmev-stroke-probe-20261106/jobs/csd-review-repair-rebind269/narration',
}
# 969 lines 17-20 and 53, kept for 09: both tables under the plain delivery-to-rebind313 replacement.
OLD09 = str(R/'jobs/csd-review-repair-rebind313/narration')
EXPECTED09 = {'cues.tsv': 'eed5a50ad5797fa258ee3fcf9319e29946f403a86b3909090e73a3a15dc74249',
              'timeline.tsv': '5a91b8f9e2e5b5d6223a49285c06039b1b43bd16c11cecdd163a8e2e545baa9e'}

def sha(p):
    with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest()
def read(p):return json.loads(Path(p).read_text())
def balanced(terms):
    assert terms
    if len(terms)==1:return terms[0]
    middle=len(terms)//2
    return '('+balanced(terms[:middle])+'+'+balanced(terms[middle:])+')'
def success(p):
    d=read(p)
    assert d.get('admitted') and d.get('finished') and d.get('exit')==0,p
    assert d['inputs_sha256']==d['final_inputs_sha256'],p
    assert not any(d.get(k) for k in ('error','resource_stop','stopped_for_resource_limit')),p
    return d

def job(chapter):
    return R/'jobs'/(PICTURES+chapter.replace('_','-'))

def lineage(chapter):
    """The runner the chapter's final job ran under, as its guard bound it, and that final job.
    Returns (run module, final job, the final job's bound inputs, files read)."""
    spec=importlib.util.spec_from_file_location('repair_final01_run_bound',RUNNER)
    run=importlib.util.module_from_spec(spec);spec.loader.exec_module(run)
    assert chapter in run.CHAPTERS,chapter
    final=run.job(chapter)
    receipt=success(final/'resource-receipt.json')
    assert [str(x) for x in receipt['command']]==[sys.executable,'-I','-B',str(RUNNER),'--chapter',chapter]
    inputs=Q/(final.name+'-inputs.json')
    assert sha(inputs)==receipt['inputs_sha256']
    entries=read(inputs)['files'];bound={x['path']:x['sha256'] for x in entries};assert len(bound)==len(entries)
    assert bound.get(str(RUNNER))==sha(RUNNER),'not the runner the final job bound'
    return run,final,bound,{final/'resource-receipt.json',inputs,RUNNER}

def retained(run,chapter):
    """Exactly the admitted preview extraction's frames: what root viewed (for a REVIEW_ROUTES chapter, what
    the independent reviewer and the repair author each viewed completely, under root's explicit decision),
    holding the sampling list and (09) prior775. Returns (frames, retained sample count, files read)."""
    spec=run.CHAPTERS[chapter];chain=run.CHAINS[spec['kind']]
    admission=run.admission(chapter);review=read(admission)
    assert review['chapter']==chapter and review['safe_to_advance_to_final4k'] is True
    record=R/'jobs'/chain['pictures']/chapter/'pictures.json'
    assert review['extraction_record']['path']==str(record) and review['extraction_record']['sha256']==sha(record)
    preview=read(record)
    movie=R/'jobs'/chain['preview']/'preview'/(chapter+'.mp4')
    assert preview['movie']==str(movie) and preview['movie_sha256']==sha(movie)
    assert preview['full_decode'] is True and preview['frames']==spec['frames']
    frames=preview['selected_frames']
    assert frames==sorted(set(frames)) and min(frames)>=0 and max(frames)<spec['frames']
    sampling=Q/chain['sampling']/('frames-'+chapter+'-all.txt')
    assert sha(sampling)==spec['sampling_sha256']
    required={int(x) for x in sampling.read_text().split()}
    files={admission,record,movie,sampling}
    if 'prior' in spec:
        prior=Q/spec['prior'];assert sha(prior)==spec['prior_sha256']
        oldframes=read(prior)['selected_frames']
        assert len(oldframes)==len(set(oldframes))==814 and oldframes==sorted(oldframes)
        required|=set(oldframes);files.add(prior)
    assert required<=set(frames),'the admitted preview extraction lacks retained samples'
    route=run.REVIEW_ROUTES.get(chapter)
    for key in ('root_review','independent_review')+(('author_review',) if route else ()):
        item=review[key];assert sha(item['path'])==item['sha256'];files.add(Path(item['path']))
    # The bound runner's own admission checks: whole root viewing, or 09's REVIEW_ROUTES.
    run.review_checks(chapter,review,read(review['root_review']['path']),read(review['independent_review']['path']),
                      read(review['author_review']['path']) if route else None,preview,sha(record))
    return frames,len(required),files

def choose(kept,layout_text,total):
    """969 lines 86-93: the kept frames plus every lint anchor, 2 s after it every 12 frames, and every 300
    frames from it to the end."""
    chosen=set(kept);anchors=[]
    for row in layout_text.splitlines():
        match=re.search(r'@([0-9.]+)s',row);assert match,row
        at=round(float(match[1])*60);assert 0<=at<total
        anchors.append(dict(frame=at,row=row))
        chosen.update(range(at,min(at+121,total),12))
        chosen.update(range(at,total,300))
    frames=sorted(chosen);assert set(kept)<=chosen and len(frames)<CAP
    return frames,anchors

def normalized(raw,narration,folder):
    """The timeline helpers' EXPECTED digest: each line row's narration path moved to folder."""
    out=[]
    for line in raw.split('\n'):
        cols=line.split('\t')
        if cols[0]=='line':
            assert cols[-1].startswith(narration+'/'),cols
            cols[-1]=folder+'/'+cols[-1].rsplit('/',1)[1]
        out.append('\t'.join(cols))
    return hashlib.sha256('\n'.join(out).encode()).hexdigest()

def main():
    ap=argparse.ArgumentParser(description=__doc__);ap.add_argument('--chapter',required=True)
    chapter=ap.parse_args().chapter
    JOB=job(chapter)
    base=Q/'csd-native844-base761.py'
    assert sha(base)==BASE761_SHA
    spec=importlib.util.spec_from_file_location('owned_repair_pictures01',base)
    h=importlib.util.module_from_spec(spec);spec.loader.exec_module(h)
    h.own_group();h.STATE['job']=JOB;h.CRATE=R/'work/production-repo07'
    for sig in h.STOP_SIGNALS:signal.signal(sig,h.on_stop)
    guard=read(JOB/'resource-receipt.json')
    assert guard['admitted'] and guard['phase']=='native' and not guard['preflight_only']
    assert 'finished' not in guard and 'exit' not in guard
    assert guard['policy']==POLICY
    assert sorted(p.name for p in JOB.iterdir())==['command.log','resource-receipt.json']
    assert Path(__file__).resolve()==HELPER.resolve()
    assert [str(x) for x in guard['command']]==[sys.executable,'-I','-B',str(Path(__file__).resolve()),'--chapter',chapter]
    run,FINAL,_,_=lineage(chapter)
    spec=run.CHAPTERS[chapter];chain=run.CHAINS[spec['kind']];total=spec['frames']
    PREVIEW=R/'jobs'/chain['preview']
    NARRATION=str(R/'jobs'/chain['delivery']/'narration')
    assert read(PREVIEW/chain['comparison'])['chapters'][chapter]['frames']==total
    tables={}
    for ext in ('timeline.tsv','cues.tsv'):
        p=FINAL/'render'/(chapter+'.'+ext);raw=p.read_bytes()
        assert raw==(PREVIEW/'preview'/p.name).read_bytes()
        if chapter=='09_boundary':
            assert hashlib.sha256(raw.replace(NARRATION.encode(),OLD09.encode())).hexdigest()==EXPECTED09[ext]
        tables[ext]=sha(p)
    raw=(FINAL/'render'/(chapter+'.timeline.tsv')).read_text()
    assert normalized(raw,NARRATION,NORMALIZE_TO[chapter])==spec['timeline_normalized']
    # The scored cuelog is checked by schedule() at the stage step (narration rows against timeline269,
    # stem rows against score21's digests); 969's 09-only reference586 comparison is not carried.
    cuelog=FINAL/'render'/(chapter+'.cuelog.tsv')
    tables['cuelog.tsv']=sha(cuelog)
    movie=FINAL/'render'/(chapter+'.mp4')
    layout=FINAL/'render'/(chapter+'.layout.tsv')
    probe=json.loads(h.run_step(JOB,'probe',['/usr/bin/ffprobe','-v','error','-show_streams',
        '-of','json',movie],60,None))
    videos=[s for s in probe['streams'] if s['codec_type']=='video'];assert len(videos)==1
    v=videos[0]
    assert (v['width'],v['height'],v['avg_frame_rate'],int(v['nb_frames']))==(3840,2160,'60/1',total)
    h.run_step(JOB,'decode',['/usr/bin/ffmpeg','-nostdin','-v','error','-xerror','-threads','2',
        '-i',movie,'-f','null','-'],3600,None)
    kept,retained_samples,_=retained(run,chapter)
    frames,anchors=choose(kept,layout.read_text(),total)
    select=JOB/'select.filter'
    with select.open('x') as f:f.write("select='"+balanced([f'eq(n\\,{n})' for n in frames])+"'\n")
    pictures=JOB/'pictures';pictures.mkdir()
    h.run_step(JOB,'extract',['/usr/bin/ffmpeg','-nostdin','-n','-v','error','-xerror','-threads','2',
        '-i',movie,'-an','-filter_script:v',select,'-fps_mode','passthrough','-q:v','2',
        pictures/'frame-%06d.jpg'],3600,None)
    files=sorted(pictures.glob('frame-*.jpg'));assert len(files)==len(frames)
    sheets=JOB/'sheets';sheets.mkdir();rows=[]
    for start in range(0,len(files),8):
        sheet=Image.new('RGB',(1280,1536),'#222222');draw=ImageDraw.Draw(sheet)
        for slot,p in enumerate(files[start:start+8]):
            index=start+slot;x,y=(slot%2)*640,(slot//2)*384
            with Image.open(p) as im:
                assert im.size==(3840,2160)
                sheet.paste(im.resize((640,360),Image.Resampling.LANCZOS),(x,y+24))
            draw.text((x+8,y+5),f'{index+1:03d} | frame {frames[index]} | {frames[index]/60:.3f}s',fill='white')
        p=sheets/f'sheet-{start//8+1:03d}.jpg';sheet.save(p,quality=95)
        rows.append(dict(path=str(p),sha256=sha(p),first_picture=start+1,last_picture=min(start+8,len(files))))
    record=R/'jobs'/chain['pictures']/chapter/'pictures.json'
    result=dict(movie=str(movie),movie_sha256=sha(movie),frames=total,tables=tables,full_decode=True,
        selected_frames=frames,sheets=rows,layout_sha256=sha(layout),layout_anchors=anchors,
        retained_samples=retained_samples,retained_preview_samples=len(kept),resolution=[3840,2160],
        pictures=[dict(path=str(p),sha256=sha(p)) for p in files],
        chapter=chapter,final_job=FINAL.name,preview_extraction=dict(path=str(record),sha256=sha(record)),
        sampling='All admitted preview samples (the sampling list'+(' and all814 prior775' if 'prior' in spec else '')+
            ') plus current final lint anchors and persistence.',
        scope='Extraction only. No picture viewing, listening, fine-sync or independent AV acceptance.')
    with (JOB/'pictures.json').open('x') as f:json.dump(result,f,indent=2)
    print(chapter,'extracted',len(files),'pictures on',len(rows),'sheets; not viewed',flush=True)

if __name__=='__main__':main()
