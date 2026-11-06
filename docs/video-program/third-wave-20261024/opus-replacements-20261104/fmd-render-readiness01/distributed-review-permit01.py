"""Carry a real CSD review to FMD. Never creates a review or starts a renderer."""
import hashlib
import importlib.util
import json
from pathlib import Path
import sys
ROOT = Path('/data/projects/jmev-stroke-probe-20261106')
HERE = ROOT/'qualification01'
def load(name):
    spec=importlib.util.spec_from_file_location(name,HERE/name)
    module=importlib.util.module_from_spec(spec);spec.loader.exec_module(module)
    return module

def prepare():
    assert Path(__file__).resolve().parent == HERE
    G=load('final-review-gate01.py')
    R=load('chapter-routing01.py')
    L=load('launch-final02.py')
    review_path=HERE/'preview-reviewed01.json'
    result=G.check(review_path)
    assignment_path=HERE/'chapter-assignment01.json'
    assignment=json.loads(assignment_path.read_text())
    # Current completed timeline is authoritative, not the routing proposal.
    timeline=json.loads((ROOT/'jobs/stage07-timeline01/timeline-summary.json').read_text())
    R.validate(assignment,timeline['chapters'])
    assert [r['chapter'] for r in assignment['assignments']] == list(G.S.CHAPTERS)
    extraction_path=G.X.PICTURES_OUT/'extraction.json'
    review_json=review_path.read_text();extraction_json=extraction_path.read_text()
    frozen=L.frozen_inputs(result)
    permit=dict(origin_host='csd',binary_sha256=G.S.BINARY_SHA,
        chapters=list(G.S.CHAPTERS),assignment_sha256=G.S.sha(assignment_path),
        review_record=json.loads(review_json),extraction_record=json.loads(extraction_json),
        review_json=review_json,extraction_json=extraction_json,
        review_sha256=G.S.sha(review_path),extraction_sha256=G.S.sha(extraction_path),
        review_gate_result=result,
        scope='Recorded selected-picture review only; no listening, fine-sync or full independent AV acceptance.')
    for name in ('review','extraction'):
        assert hashlib.sha256(permit[name+'_json'].encode()).hexdigest()==permit[name+'_sha256']
    assert G.check(review_path)==result
    for name,data in [('distributed-review-inputs01.json',frozen),('distributed-review-permit01.json',permit)]:
        path=HERE/name
        if path.exists():assert json.loads(path.read_text())==data,'Existing record differs; preserve and inspect'
        else:
            with path.open('x') as f:json.dump(data,f,indent=2)
    print('Actual review revalidated and bound; no renderer launched.')
if __name__=='__main__':
    assert sys.argv[1:]==['--prepare']
    prepare()
