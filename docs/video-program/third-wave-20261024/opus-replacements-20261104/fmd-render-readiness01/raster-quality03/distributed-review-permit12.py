"""Bind actual raster12 picture review to a disjoint two-host chapter plan.
No review is created, no render is started. Missing review fails closed.
"""
import importlib.util,json,sys
from pathlib import Path
HERE=Path('/data/projects/jmev-fmd-render-qualification-20261106-1111/qualification01/raster-quality03')
def load(name):
 spec=importlib.util.spec_from_file_location(name,HERE/name);m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m);return m

def prepare():
 assert Path(__file__).resolve().parent==HERE
 G=load('final-review-gate12.py');P=G.P;R=load('chapter-routing12.py')
 review_path=HERE/'preview-reviewed12.json'
 result=G.check(review_path)
 summary_path=P.JOB/'preview-summary.json';summary=P.read(summary_path)
 log=P.ROOT/'jobs/fmd-raster-timeline12/native.stdout'
 assignment=R.build(summary,log.read_text())
 assignment['inputs']={str(p):P.sha(p) for p in [summary_path,log,HERE/'chapter-routing12.py']}
 target=HERE/'chapter-assignment12.json'
 if target.exists():assert P.read(target)==assignment
 else:
  with target.open('x') as f:json.dump(assignment,f,indent=2)
 R.validate(assignment,summary['chapters'])
 extraction=G.X.PICTURES_OUT/'extraction.json'
 permit=dict(origin_host='fmd',binary_sha256=P.BINARY_SHA,chapters=list(P.CHAPTERS),assignment_sha256=P.sha(target),review_gate_result=result,scope='Actual producer selected-frame review only; no listening, fine sync or full independent AV acceptance.')
 for name,path in [('review',review_path),('extraction',extraction)]:
  permit[name+'_json']=path.read_text();permit[name+'_record']=P.read(path);permit[name+'_sha256']=P.sha(path)
 assert G.check(review_path)==result
 path=HERE/'distributed-review-permit12.json'
 if path.exists():assert P.read(path)==permit
 else:
  with path.open('x') as f:json.dump(permit,f,indent=2)
 print('Actual current review validated and bound; no renderer launched.')
if __name__=='__main__':
 assert sys.argv[1:]==['--prepare'];prepare()
