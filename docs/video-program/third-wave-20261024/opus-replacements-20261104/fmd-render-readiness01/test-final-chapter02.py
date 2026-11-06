"""Small real-table and explicit synthetic-record tests. No approval or rendering."""
import copy, hashlib, importlib.util, json
from pathlib import Path
HERE=Path(__file__).resolve().parent
spec=importlib.util.spec_from_file_location('worker',HERE/'final-chapter02.py')
W=importlib.util.module_from_spec(spec);spec.loader.exec_module(W)
A=json.loads((HERE/'chapter-assignment01.json').read_text())
keys=[r['chapter'] for r in A['assignments']]
sheets=[dict(chapter=k,path='fixture/'+k,sha256='fixture') for k in keys]
E=dict(sheets=sheets,samples=[dict(chapter=k) for k in keys],preview_summary_sha256='fixture')
rawE=json.dumps(E);hashE=hashlib.sha256(rawE.encode()).hexdigest()
V=dict(pictures_viewed=True,unresolved_visual_defects=[],reviewer='SYNTHETIC TEST ONLY',
    chapters=keys,review_scope='producer selected-frame review',listening_accepted=False,
    fine_sync_accepted=False,full_independent_av_accepted=False,
    viewed_sheets=[dict(path=r['path'],sha256=r['sha256']) for r in sheets],
    extraction_sha256=hashE,preview_summary_sha256='fixture')
rawV=json.dumps(V);hashV=hashlib.sha256(rawV.encode()).hexdigest()
G=dict(chapters=15,viewed_sheets=15,selected_pictures=15,extraction_sha256=hashE,
    review_sha256=hashV,preview_summary_sha256='fixture',binary_sha256=W.BINARY_SHA)
P=dict(origin_host='csd',binary_sha256=W.BINARY_SHA,chapters=keys,review_record=V,
    extraction_record=E,review_gate_result=G,review_json=rawV,extraction_json=rawE,
    review_sha256=hashV,extraction_sha256=hashE)
W.validate_permit(P,A,keys[0])
results=[]
def rejected(name,fn):
    try:fn()
    except (AssertionError,KeyError,FileNotFoundError):results.append(name);return
    raise AssertionError('Incorrectly accepted '+name)
for field,value in [('review_json',rawV+' '),('extraction_json',rawE+' '),('review_sha256','bad')]:
    bad=copy.deepcopy(P);bad[field]=value
    rejected(field,lambda:W.validate_permit(bad,A,keys[0]))
bad=copy.deepcopy(P);bad['review_record']['pictures_viewed']=False
rejected('unviewed record',lambda:W.validate_permit(bad,A,keys[0]))
bad=copy.deepcopy(P);bad['review_record']['listening_accepted']=True
rejected('false listening claim',lambda:W.validate_permit(bad,A,keys[0]))
rejected('wrong host chapter',lambda:W.validate_permit(P,A,keys[1]))
assert not W.PERMIT.exists()
rejected('no actual review permit',lambda:W.check(keys[0]))
real=HERE/'chapter-worker-tests01'
W.verify_tables(keys[0],real,6311)
for name,suffix in [('changed timeline','timeline.tsv'),('changed cue','cues.tsv'),('changed score log','cuelog.tsv')]:
    out=HERE/'chapter-worker-tests02'/name.replace(' ','-');out.mkdir(parents=True,exist_ok=False)
    for ext in ('timeline.tsv','cues.tsv','cuelog.tsv'):
        content=(real/(keys[0]+'.'+ext)).read_bytes()
        if ext==suffix:content+=b'INVALID TEST ROW\n'
        (out/(keys[0]+'.'+ext)).write_bytes(content)
    rejected(name,lambda:W.verify_tables(keys[0],out,6311))
report=dict(real_narrated_chapter_tables_pass=True,rejections=results,
    synthetic_permit_tests_only=True,actual_review_exists=False,render_launched=False)
with (HERE/'chapter-worker-tests02-result.json').open('x') as f:json.dump(report,f,indent=2)
print(json.dumps(report))
