"""Actual image-review gate for repaired chapters. Prepared only; no acceptance is created."""
from pathlib import Path
import hashlib,json
D=Path('/data/projects/jmev-stroke-probe-20261106')
Q=D/'qualification01/production-repair17'
P=D/'jobs/csd-native-combined-preview01'
X=D/'jobs/csd-native-combined-pictures01'
S=D/'jobs/csd-production-score21'
CHAPTERS=('01_the_bump','08_loop_sums','09_boundary')
def sha(p):
    with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest()
def read(p):return json.loads(Path(p).read_text())
def stable(job):
    r=read(job/'resource-receipt.json')
    assert r.get('admitted') and r.get('finished') and r.get('exit')==0
    assert r['inputs_sha256']==r['final_inputs_sha256']
    assert not any(r.get(k) for k in ('error','resource_stop','stopped_for_resource_limit'))
    return r
def check():
    for job in (P,X,S):stable(job)
    records=X/'pictures-combined01.json';pictures=read(records)
    assert set(pictures)==set(CHAPTERS)
    review=read(Q/'csd-combined01-picture-admission.json')
    assert review['preview_job']==P.name and review['extraction_job']==X.name
    assert review['all_selected_pictures_viewed'] is True
    assert review['concrete_defects']==[] and review['independent_blockers']==[]
    assert review['safe_to_advance_to_final4k'] is True
    assert review['extraction_record']=={'path':str(records),'sha256':sha(records)}
    assert set(review['chapters'])==set(CHAPTERS)
    bound={str(records):sha(records),str(Q/'csd-combined01-picture-admission.json'):sha(Q/'csd-combined01-picture-admission.json')}
    source=D/'work/native-combined01/workspace/explainer_corpus/complex_analysis_visual'
    sources=read(Q/'combined-pictures01/sampling-plan02.json')['sources']['staged']
    for rel,digest in sources.items():assert sha(source/rel)==digest
    for chapter in CHAPTERS:
        pic=pictures[chapter];movie=P/'preview'/(chapter+'.mp4')
        assert pic['movie']==str(movie) and pic['movie_sha256']==sha(movie) and pic['full_decode'] is True
        frames=pic['selected_frames'];assert frames==sorted(set(frames)) and frames
        for item in pic['pictures']+pic['sheets']:assert sha(item['path'])==item['sha256']
        pair=review['chapters'][chapter]
        for role in ('root_review','independent_review'):
            item=pair[role];assert sha(item['path'])==item['sha256'];bound[item['path']]=item['sha256'];e=read(item['path'])
            assert e['movie_sha256']==pic['movie_sha256'] and e['extraction_sha256']==sha(records)
            assert e['source_hashes']==sources and e['concrete_defects']==[] and e['safe_to_advance_to_final4k'] is True
            assert e['viewed_frame_indices'] and set(e['viewed_frame_indices'])<=set(frames)
            if role=='root_review':
                assert e['viewed_frame_indices']==frames and e['viewed_sheets']==[x['path'] for x in pic['sheets']]
            else:
                assert e['authored_repair'] is False
                assert e['viewed_frame_indices']==frames
                assert e['viewed_sheets']==[x['path'] for x in pic['sheets']]
        assert pair['root_review']['path']!=pair['independent_review']['path']
        qa=S/'score/stems/qa'/(chapter+'.json');assert read(qa)['passed'] is True;bound[str(qa)]=sha(qa)
        for p in (S/'score/stems').glob(chapter+'.music*'):
            assert p.is_file() and not p.is_symlink();bound[str(p)]=sha(p)
    score_inputs=Q/'score21-inputs.json';assert sha(score_inputs)==stable(S)['inputs_sha256']
    for row in read(score_inputs)['files']:assert sha(row['path'])==row['sha256']
    return bound
if __name__=='__main__':
    result=check();print('Actual preview review gate passes for three named chapters; final render and AV still pending.',len(result))
