"""PREPARED ONLY. Build real proof bundle and guard868 after actual875 final review.

No polling/automatic render. Explicit --root-review and --independent-review must
already exist and describe final864/875, not preview. Does not create acceptance.
After stage success copies its exact manifest to Q; does NOT launch master874.
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
WORKER_SHA='623253e84bafef9f8ecb48ca406d01a59b34598b2413ceb69e0494c6522e6997'
MASTER_BASE_SHA='2b01ecaab2b409268d86fe13b83e97a7e374dd61f1c657680a32444fdb6aa524'
MAP_SHA='92407bbf10b0a83e8a19a106a15d0c17752503f9bf6c0887ee25fd246103bd8f'
MANIFEST_SHA='6fde63a4029af7765491d0f38f5371feb701f954277d2cf3728d9f0ae4220942'

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

def main():
    ap=argparse.ArgumentParser(description=__doc__)
    ap.add_argument('--job',required=True)
    ap.add_argument('--root-review',type=Path,required=True)
    ap.add_argument('--independent-review',type=Path,required=True)
    a=ap.parse_args()
    assert re.fullmatch(r'csd-native-stage868-09-a[1-9][0-9]*',a.job)
    job=D/'jobs'/a.job;inputs=Q/(a.job+'-inputs.json');proof_path=Q/(a.job+'-proofs.json')
    publish_manifest=Q/(a.job+'-master-manifest.json')
    assert all(not p.exists() for p in (job,inputs,proof_path,publish_manifest)), 'Preserve attempts; no duplicate'
    rows={}
    def add(p,expected=None):
        p=Path(p)
        assert p.is_absolute() and p.is_file() and not p.is_symlink(),p
        h=sha(p)
        if expected is not None:assert h==expected,p
        assert str(p) not in rows or rows[str(p)]==h,('input drift',p)
        rows[str(p)]=h;return p
    prerequisites=[]
    for jobname,manifestname in [
        ('csd-native-build844','csd-native844-inputs.json'),
        ('csd-native-delivery849','csd-native849-inputs.json'),
        ('csd-native-final864','csd-native864-inputs.json'),
        ('csd-native-pictures875','csd-native875-inputs.json')]:
        receipt=D/'jobs'/jobname/'resource-receipt.json';r=success(receipt)
        manifest=Q/manifestname;add(manifest,r['inputs_sha256']);add(receipt)
        entries=read(manifest)['files'];assert len(entries)==len({x['path'] for x in entries})
        for item in entries:add(item['path'],item['sha256'])
        prerequisites.append(dict(receipt=str(receipt),inputs=str(manifest)))
    extraction=add(D/'jobs/csd-native-pictures875/pictures875.json');report=read(extraction)
    movie=add(D/'jobs/csd-native-final864/render/09_boundary.mp4')
    assert report['movie']==str(movie) and report['movie_sha256']==sha(movie)
    assert report['full_decode'] is True and report['frames']==16504 and report['resolution']==[3840,2160]
    frames=report['selected_frames'];assert frames==sorted(set(frames)) and len(frames)>=814
    assert len(report['pictures'])==len(frames) and len(report['sheets'])==(len(frames)+7)//8
    for item in report['pictures']+report['sheets']:add(item['path'],item['sha256'])
    for p in (a.root_review,a.independent_review):
        assert p.parent==Q;add(p)
    root,independent=read(a.root_review),read(a.independent_review)
    for review in (root,independent):
        assert review['source_sha256']=='cd1900aadd46090083729d9aa768e59e1551c42ab94e5bd055f520cb4eff5e6e'
        assert review['movie_sha256']==sha(movie) and review['extraction_sha256']==sha(extraction)
        assert review['concrete_defects']==[] and review['safe_to_advance_to_master'] is True
    assert root['viewed_frame_indices']==frames
    assert root['viewed_sheets']==[x['path'] for x in report['sheets']]
    assert independent['authored_repair'] is False and independent['viewed_frame_indices']
    assert set(independent['viewed_frame_indices'])<=set(frames)
    # Read-only whole-film reference qualification: all15 schedule/score contracts.
    basepath=add(Q/'combined-master447.py',MASTER_BASE_SHA)
    spec=importlib.util.spec_from_file_location('reference876',basepath)
    master=importlib.util.module_from_spec(spec);spec.loader.exec_module(master)
    registry,frames_of,reference_files=master.film()
    assert len(registry)==15 and frames_of['09_boundary']==16504
    for p in reference_files:add(p)
    for p in (a.root_review,a.independent_review):assert not list(master.claimed(read(p)))
    original=add(Q/'root-master-manifest625.json',MANIFEST_SHA)
    assert [r['chapter'] for r in read(original)['chapters']]==registry
    # The actualstage uses313 as canonical audio while the master also binds269.
    for rootdir in (master.NARRATION313,D/'jobs/csd-native-delivery849/narration',
                    D/'jobs/csd-native-build844',D/'jobs/csd-native-final864',D/'jobs/csd-native-pictures875'):
        for p in rootdir.rglob('*'):
            assert not p.is_symlink(),p
            if p.is_file():add(p)
    for e in ('timeline.tsv','cues.tsv','cuelog.tsv'):add(master.TIMELINES/('09_boundary.'+e))
    add(master.SCORE/'score21-summary.json')
    worker=add(Q/'native-linux-materialize868.py',WORKER_SHA)
    add(Q/'root-native574-table-rebind.py',MAP_SHA)
    for p in (Path(__file__).resolve(),Path(sys.executable).resolve(),Path('/usr/bin/ffprobe'),D/'guard_csd02.py'):add(p)
    proofs=dict(prerequisites=prerequisites,extraction=str(extraction),root_review=str(a.root_review),independent_review=str(a.independent_review))
    # Nothing above writes reviews or jobs. Only now preserve the real bound proofbundle.
    write(proof_path,proofs);add(proof_path)
    for p,h in rows.items():assert sha(p)==h,p
    write(inputs,dict(files=[dict(path=p,sha256=h) for p,h in sorted(rows.items())]))
    subprocess.run([sys.executable,'-I','-B',str(D/'guard_csd02.py'),'--phase','native','--job',a.job,
        '--inputs',str(inputs),'--',sys.executable,'-I','-B',str(worker),'--job',a.job,
        '--inputs',str(inputs),'--proofs',str(proof_path),'--manifest',str(original)],check=True)
    success(job/'resource-receipt.json')
    made=read(job/'materialized.json');assert made['unchanged_other_manifest_rows']==14
    raw=(job/'master-manifest.json').read_bytes();new=json.loads(raw)
    assert [r for r in new['chapters'] if r['chapter']!='09_boundary']==[r for r in read(original)['chapters'] if r['chapter']!='09_boundary']
    with publish_manifest.open('xb') as f:f.write(raw)
    assert sha(publish_manifest)==sha(job/'master-manifest.json')
    print(json.dumps(dict(stage=a.job,manifest=str(publish_manifest),manifest_sha256=sha(publish_manifest),
        next='Root must review stage and run fresh874 check/preflight/guarded launch; no master launched here.')))

if __name__=='__main__':main()
