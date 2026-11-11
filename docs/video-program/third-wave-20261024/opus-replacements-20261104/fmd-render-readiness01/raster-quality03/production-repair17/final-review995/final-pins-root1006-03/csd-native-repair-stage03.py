"""PREPARED ONLY (CC3 scratch draft final-master-cc3). Build one repaired chapter's real proof bundle and
guarded stage job after its actual final review, or compose the four published stage manifests.

Successor of csd-native994-stage-launch.py (f0641032) for 01/08/09 (native-combined01) and 11
(native-counting02), with CC0's 991 successor as worker. Changes from 994, and nothing else:
  - stage --chapter: the chapter's chain, final and final extraction come from the adopted worker, final
    runner and final extraction helper, each pinned by sha256 (None until root adopts them; refuses);
  - prerequisites are the chain's build, delivery, timeline, preview and preview pictures, the chapter's
    final and its final extraction, each with its own guard inputs;
  - every file the final's run proofs read (admission, preview reviews and extraction, comparisons,
    delivery record, qualification, binary, scene, preview capture, stems) is bound again through the
    runner's own proofs();
  - the final extraction must hold every preview extraction frame (994: >= 814); reviews name the
    chapter's scene (994: cd1900aa);
  - the base manifest is master993's (8da77ed7): one row replaced, 14 unchanged;
  - compose: the four published stage manifests become one master manifest, master993's with exactly
    those four rows replaced. Each stage manifest must differ from 8da77ed7 at its own chapter only.
No polling/automatic render. Explicit --root-review and --independent-review must
already exist and describe the chapter's final and final extraction, not preview. Does not create
acceptance or any review scope (M1: the master gate derives it from the actual admission).
After stage success copies its exact manifest to Q; does NOT launch the master.

CC0 successor stage02 of CC3's 432301fa: 11's final runs under run11 and extracts under pictures11 (new
names; 7cafacfa and 60addbf3 stay bound to 01/08/09), so pair() chooses each chapter's run and extraction
helpers, each pinned by sha256. Nothing else changes; the final-review checks are 432301fa's.

CC0 successor (option b; owner via root, 2026-11-11): the actual final extraction's review may take a worker
route, as the previews did. final_review_checks() (identical in stage03, materializer03 and gate04) accepts
root's complete viewing plus an independent subset (the original rule), or root's targeted acceptance
(review_route in FINAL_ROUTES) that binds complete worker reviews of the actual final movie and extraction
by sha256, with honest authorship. The proofs name root_review, independent_review and author_review
(None where the route has none). Nothing else changes.
"""
import argparse
import hashlib
import importlib.util
import json
from pathlib import Path
import re
import subprocess
import sys

D=Path('/data/projects/jmev-stroke-probe-20261106')
Q=D/'qualification01/production-repair17'
MASTER_BASE_SHA='2b01ecaab2b409268d86fe13b83e97a7e374dd61f1c657680a32444fdb6aa524'
MAP_SHA='92407bbf10b0a83e8a19a106a15d0c17752503f9bf6c0887ee25fd246103bd8f'
# master993's manifest, as its summary names it: 994's stage970-09-a2 publication.
MANIFEST=Q/'csd-native-stage970-09-a2-master-manifest.json'
MANIFEST_SHA='8da77ed79e9ccc8a06e703e570c85c44de5cd884a047eaed61355167e397e100'
# Each chain's guard inputs prefix, as its launchers name them (build, delivery, timeline, preview, pictures).
CHAIN_INPUTS={'native-combined01':'csd-combined01','native-counting02':'csd-counting02'}
# The names the final run and extraction helpers bind for themselves (run RUNNER, after-final HELPER).
RUNNER=Q/'csd-native-repair-final01-run.py'
PICTURES=Q/'csd-native-repair-pictures01.py'
# 11's own pair (CC0's chapter 11 successors of 7cafacfa and 60addbf3).
RUNNER11=Q/'csd-native-repair-final01-run11.py'
PICTURES11=Q/'csd-native-repair-pictures11.py'
# UNRESOLVED pins: root sets each before use; both modes refuse while any is None.
WORKER_NAME='cc0-native-materialize-repaired03.py'    # Q name of the adopted 991 successor (CC0 materializer draft), pins filled
WORKER_SHA='d8230404870270331438d715e9ae5be4deab66f4e07c202458cb8c553209bcb2'
RUNNER_SHA='7cafacfa788b1579ce66b9db9880c25890112b7d1d7737ebcd44d7827daf1a09'     # the run helper staged once, with BOTH binary pins filled (CC2 S1)
PICTURES_SHA='60addbf322a21270ae5aa4751bef5cbb203bf7e12817574dcb0688df9edf90a8'
RUNNER11_SHA='01382a17b527b957fa7da7ce7b31e07c7c34023b01c948ac866761a1ef53bdeb'
PICTURES11_SHA='74e0990e9919ebc15097553275e8fb481834928ab0c3c6c11053bd1fe760e2a7'

def unresolved():
    return [n for n,v in (('WORKER_NAME',WORKER_NAME),('WORKER_SHA',WORKER_SHA),
                          ('RUNNER_SHA',RUNNER_SHA),('PICTURES_SHA',PICTURES_SHA),
                          ('RUNNER11_SHA',RUNNER11_SHA),('PICTURES11_SHA',PICTURES11_SHA)) if not v]

def sha(p):
    with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest()
def read(p):return json.loads(Path(p).read_text())
def write(p,value):
    with p.open('x') as f:json.dump(value,f,indent=2);f.write('\n')
def success(p):
    r=read(p)
    assert r.get('admitted') and r.get('finished') and r.get('exit')==0,p
    assert r['inputs_sha256']==r['final_inputs_sha256'],p
    assert not any(r.get(k) for k in ('error','resource_stop','stopped_for_resource_limit','preflight_only')),p
    return r
def load(p,digest,name):
    assert sha(p)==digest,p
    spec=importlib.util.spec_from_file_location(name,p)
    m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m);return m
def pair(ch):
    """The run and extraction helpers, with pins, that the chapter's final and final extraction ran under."""
    if ch=='11_counting':return RUNNER11,RUNNER11_SHA,PICTURES11,PICTURES11_SHA
    return RUNNER,RUNNER_SHA,PICTURES,PICTURES_SHA
def helpers(ch=None):
    missing=unresolved();assert not missing,'unresolved pins: '+', '.join(missing)
    w=load(Q/WORKER_NAME,WORKER_SHA,'repaired_stage_worker')
    assert not w.unresolved() and w.MANIFEST_SHA==MANIFEST_SHA
    runner,runner_sha,pictures,pictures_sha=pair(ch)
    return w,load(runner,runner_sha,'repaired_final_run'),load(pictures,pictures_sha,'repaired_final_pictures')


# Historical aliases are immutable prerequisites, never new output destinations.
ALIAS_FILE = Path('/data/projects/jmev-stroke-probe-20261106/qualification01/production-repair17/historical-aliases991.json')
ALIAS_SHA = 'c6408b16d707164dd20e961a4db0ae05ca1c2c73830ca26118b8fe61e8aba958'
def alias_target(p):
    assert p.is_absolute() and p.is_file(), p
    raw = ALIAS_FILE.read_bytes()
    assert hashlib.sha256(raw).hexdigest() == ALIAS_SHA
    aliases = json.loads(raw)
    if str(p) not in aliases:
        assert not p.is_symlink(), p
        return None
    row = aliases[str(p)]
    assert p.is_symlink() and str(p.readlink()) == row['link'], p
    target = Path(row['target'])
    assert p.resolve(strict=True) == target and target.is_file() and not target.is_symlink(), p
    with target.open('rb') as f:
        assert hashlib.file_digest(f, 'sha256').hexdigest() == row['sha256'], p
    return target

FINAL_ROUTES = ('independent-complete-root-targeted', 'author-complete-root-targeted',
                'independent-and-author-complete-root-targeted')


def final_review_checks(chapter, source_sha, movie_sha, extraction_sha, report, root, independent, author,
                        independent_sha, author_sha):
    """The actual final 4K extraction's review (owner via root, 2026-11-11, option b). Pure: reads no file.
    Every record names the actual final movie, the actual final extraction record and the chapter's scene,
    states no concrete defect and is safe to advance to the master. If root's record has no review_route,
    root viewed every selected frame and sheet and an independent reviewer (not the repair's author) viewed
    some: the original rule. If it names a FINAL_ROUTES route, each worker review that route names viewed
    every selected frame and sheet, the independent one is not the repair's author and the author one is,
    and root's record is a targeted acceptance of this chapter that binds those records and the extraction
    by sha256, counts their complete coverage and states root_whole_chapter_review false. Returns the record
    whose viewing is complete; the row's scope is derived from it."""
    frames = report['selected_frames']
    sheets = [item['path'] for item in report['sheets']]
    route = root.get('review_route')
    assert route is None or route in FINAL_ROUTES, route
    assert (independent is not None) == (route is None or 'independent' in route), route
    assert (author is not None) == (route is not None and 'author' in route), route
    for evidence in (root, independent, author):
        if evidence is not None:
            assert evidence['movie_sha256'] == movie_sha and evidence['extraction_sha256'] == extraction_sha
            assert evidence['source_sha256'] == source_sha and evidence['concrete_defects'] == []
            assert evidence['safe_to_advance_to_master'] is True
    if independent is not None:
        assert independent['authored_repair'] is False and independent['viewed_frame_indices']
        assert set(independent['viewed_frame_indices']) <= set(frames)
    if route is None:
        assert root['viewed_frame_indices'] == frames and root['viewed_sheets'] == sheets
        return root
    assert root['chapter'] == chapter and root['extraction_record']['sha256'] == extraction_sha
    assert independent is None or author is None or independent_sha != author_sha, 'one record named twice'
    coverage = root['coverage']
    assert coverage['root_whole_chapter_review'] is False
    complete = None
    for kind, record, record_sha in (('independent', independent, independent_sha), ('author', author, author_sha)):
        if record is None:
            assert kind+'_review' not in root and not coverage.get(kind+'_selected_frames'), kind
            continue
        assert record['viewed_frame_indices'] == frames and record['viewed_sheets'] == sheets, kind
        assert record['authored_repair'] is (kind == 'author'), kind
        assert coverage[kind+'_selected_frames'] == len(frames) and root[kind+'_review']['sha256'] == record_sha, kind
        complete = complete or record
    return complete


def stage(a):
    w,run,x=helpers(a.chapter)
    ch=a.chapter;c=w.CHAPTERS[ch];kind=c['kind']
    runner,runner_sha,pictures,pictures_sha=pair(ch)
    final,picked=run.job(ch),x.job(ch)
    assert x.RUNNER==run.RUNNER==runner and x.HELPER==pictures and final.name==w.FINAL_JOBS[ch]
    assert run.CHAPTERS[ch]['kind']==kind and run.CHAPTERS[ch]['scene_sha256']==c['source_sha']
    # CC2 S1: the runner is staged once, with both chains' binary pins, before any final.
    assert run.CHAPTERS[ch]['frames']==c['frames'] and not any(run.unresolved(k) for k in run.CHAPTERS)
    assert all(run.CHAINS[kind][k]==c[k] for k in ('build','delivery','timeline','preview','pictures'))
    prefix=CHAIN_INPUTS[kind];assert run.CHAINS[kind]['pictures_inputs']==prefix+'-pictures-inputs.json'
    assert re.fullmatch(r'csd-native-stage%s-%s-a[1-9][0-9]*' % (w.STAGE_VERSION,ch[:2]),a.job)
    job=D/'jobs'/a.job;inputs=Q/(a.job+'-inputs.json');proof_path=Q/(a.job+'-proofs.json')
    publish_manifest=Q/(a.job+'-master-manifest.json')
    assert all(not p.exists() for p in (job,inputs,proof_path,publish_manifest)), 'Preserve attempts; no duplicate'
    rows={}
    def add(p,expected=None):
        p=Path(p)
        target = alias_target(p)
        h=sha(p)
        if expected is not None:assert h==expected,p
        assert str(p) not in rows or rows[str(p)]==h,('input drift',p)
        rows[str(p)]=h
        if target is not None:
            rows[str(target)] = sha(target)
        return p
    add(ALIAS_FILE,ALIAS_SHA)
    prerequisites=[]
    for jobname,manifestname in [
        (c['build'],prefix+'-inputs.json'),
        (c['delivery'],prefix+'-delivery-inputs.json'),
        (c['timeline'],prefix+'-timeline-inputs.json'),
        (c['preview'],prefix+'-preview-inputs.json'),
        (c['pictures'],prefix+'-pictures-inputs.json'),
        (final.name,final.name+'-inputs.json'),
        (picked.name,picked.name+'-inputs.json')]:
        receipt=D/'jobs'/jobname/'resource-receipt.json';r=success(receipt)
        manifest=Q/manifestname;add(manifest,r['inputs_sha256']);add(receipt)
        entries=read(manifest)['files'];assert len(entries)==len({x['path'] for x in entries})
        for item in entries:add(item['path'],item['sha256'])
        prerequisites.append(dict(receipt=str(receipt),inputs=str(manifest)))
    # The final's run proofs, re-read now: the actual admission and every preview record the gate binds.
    _,_,used=run.proofs(ch)
    for p in sorted(used,key=str):add(p)
    add(run.admission(ch))
    extraction=add(picked/'pictures.json');report=read(extraction)
    movie=add(final/'render'/(ch+'.mp4'))
    assert report['movie']==str(movie) and report['movie_sha256']==sha(movie)
    assert report['full_decode'] is True and report['frames']==c['frames'] and report['resolution']==[3840,2160]
    previewed=read(add(D/'jobs'/c['pictures']/ch/'pictures.json'))
    frames=report['selected_frames'];assert frames==sorted(set(frames)) and set(previewed['selected_frames'])<=set(frames)
    assert len(report['pictures'])==len(frames) and len(report['sheets'])==(len(frames)+7)//8
    for item in report['pictures']+report['sheets']:add(item['path'],item['sha256'])
    given=[p for p in (a.root_review,a.independent_review,a.author_review) if p is not None]
    for p in given:
        assert p.parent==Q;add(p)
    assert len({sha(p) for p in given})==len(given),'one record given twice'
    worker=[(read(p),sha(p)) if p is not None else (None,None) for p in (a.independent_review,a.author_review)]
    final_review_checks(ch,c['source_sha'],sha(movie),sha(extraction),report,read(a.root_review),
                        worker[0][0],worker[1][0],worker[0][1],worker[1][1])
    # Read-only whole-film reference qualification: all15 schedule/score contracts.
    basepath=add(Q/'combined-master447.py',MASTER_BASE_SHA)
    spec=importlib.util.spec_from_file_location('reference973',basepath)
    master=importlib.util.module_from_spec(spec);spec.loader.exec_module(master)
    registry,frames_of,reference_files=master.film()
    assert len(registry)==15 and frames_of[ch]==c['frames']
    for p in reference_files:add(p)
    for p in given:assert not list(master.claimed(read(p)))
    original=add(MANIFEST,MANIFEST_SHA)
    assert [r['chapter'] for r in read(original)['chapters']]==registry
    # The actualstage uses313 as canonical audio while the master also binds269.
    for rootdir in (master.NARRATION313,D/'jobs'/c['delivery']/'narration',
                    D/'jobs'/c['build'],final,picked):
        for p in rootdir.rglob('*'):
            assert not p.is_symlink(),p
            if p.is_file():add(p)
    for e in ('timeline.tsv','cues.tsv','cuelog.tsv'):add(master.TIMELINES/(ch+'.'+e))
    add(master.SCORE/'score21-summary.json')
    worker=add(Q/WORKER_NAME,WORKER_SHA)
    add(Q/'root-native574-table-rebind.py',MAP_SHA)
    add(runner,runner_sha);add(pictures,pictures_sha)
    for p in (Path(__file__).resolve(),Path(sys.executable).resolve(),Path('/usr/bin/ffprobe'),D/'guard_csd02.py'):add(p)
    proofs=dict(prerequisites=prerequisites,extraction=str(extraction),root_review=str(a.root_review),
        independent_review=str(a.independent_review) if a.independent_review else None,
        author_review=str(a.author_review) if a.author_review else None)
    # Nothing above writes reviews or jobs. Only now preserve the real bound proofbundle.
    write(proof_path,proofs);add(proof_path)
    for p,h in rows.items():assert sha(p)==h,p
    write(inputs,dict(files=[dict(path=p,sha256=h) for p,h in sorted(rows.items())]))
    subprocess.run([sys.executable,'-I','-B',str(D/'guard_csd02.py'),'--phase','native','--job',a.job,
        '--inputs',str(inputs),'--',sys.executable,'-I','-B',str(worker),'--chapter',ch,'--job',a.job,
        '--inputs',str(inputs),'--proofs',str(proof_path),'--manifest',str(original)],check=True)
    success(job/'resource-receipt.json')
    made=read(job/'materialized.json');assert made['unchanged_other_manifest_rows']==14 and made['chapter']==ch
    raw=(job/'master-manifest.json').read_bytes();new=json.loads(raw)
    assert [r for r in new['chapters'] if r['chapter']!=ch]==[r for r in read(original)['chapters'] if r['chapter']!=ch]
    with publish_manifest.open('xb') as f:f.write(raw)
    assert sha(publish_manifest)==sha(job/'master-manifest.json')
    print(json.dumps(dict(stage=a.job,manifest=str(publish_manifest),manifest_sha256=sha(publish_manifest),
        next='Root must review stage, then compose all four; no master launched here.')))

def compose(a):
    """Master993's manifest with exactly the four repaired rows replaced, each taken from its own
    successful stage's published manifest. Writes one new Q file; runs nothing."""
    w,_,_=helpers()
    base=read(MANIFEST);assert sha(MANIFEST)==MANIFEST_SHA
    base_rows=base['chapters'];assert len(base_rows)==15
    out=Q/('csd-native-stage%s-composed-master-manifest.json' % w.STAGE_VERSION)
    assert not out.exists(),'Preserve attempts; no duplicate'
    replaced={}
    for name in a.stage:
        published=Q/(name+'-master-manifest.json');job=D/'jobs'/name
        success(job/'resource-receipt.json')
        assert sha(published)==sha(job/'master-manifest.json')
        made=read(job/'materialized.json');ch=made['chapter']
        assert ch in w.CHAPTERS and ch not in replaced and made['unchanged_other_manifest_rows']==14
        assert made['original_manifest_sha256']==MANIFEST_SHA
        assert re.fullmatch(r'csd-native-stage%s-%s-a[1-9][0-9]*' % (w.STAGE_VERSION,ch[:2]),name)
        rows=read(published)['chapters']
        assert [r['chapter'] for r in rows]==[r['chapter'] for r in base_rows]
        assert [r for r in rows if r['chapter']!=ch]==[r for r in base_rows if r['chapter']!=ch]
        (row,)=[r for r in rows if r['chapter']==ch]
        assert row['job']==name and row['kind']==w.CHAPTERS[ch]['kind']
        replaced[ch]=row
    assert set(replaced)==set(w.CHAPTERS)
    write(out,dict(base,chapters=[replaced.get(r['chapter'],r) for r in base_rows]))
    composed=read(out)['chapters']
    assert [r for r in composed if r['chapter'] not in replaced]==[r for r in base_rows if r['chapter'] not in replaced]
    print(json.dumps(dict(manifest=str(out),manifest_sha256=sha(out),stages=a.stage,
        next='Root runs the master successor check on this manifest; no master launched here.')))

def main():
    ap=argparse.ArgumentParser(description=__doc__)
    modes=ap.add_subparsers(dest='mode',required=True)
    one=modes.add_parser('stage')
    one.add_argument('--chapter',required=True)
    one.add_argument('--job',required=True)
    one.add_argument('--root-review',type=Path,required=True)
    one.add_argument('--independent-review',type=Path)   # given exactly when the final route names one
    one.add_argument('--author-review',type=Path)        # given exactly when the final route names one
    one=modes.add_parser('compose')
    one.add_argument('--stage',required=True,nargs=4)
    a=ap.parse_args()
    {'stage':stage,'compose':compose}[a.mode](a)

if __name__=='__main__':main()
