"""Successor of csd-final-after-master995.py (9a055c26): wait for the actual repaired master's success, then serial
strict verification and both picture parts through the adopted 995 successor (its pins set; names from it).
Prepared only. Does not create visual, listening, independent AV or publication acceptance.
"""
import hashlib
import importlib.util
import json
from pathlib import Path
import subprocess
import time

D=Path('/data/projects/jmev-stroke-probe-20261106')
Q=D/'qualification01/production-repair17'
HELPER_NAME=None   # UNRESOLVED: Q name of the adopted 995 successor (CC3 csd-final-verify-repaired01.py)
HELPER_SHA=None

def read(p):return json.loads(p.read_text())
def sha(p):
    with p.open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest()
def success(p):
    r=read(p)
    assert r.get('admitted') and r.get('finished') and r.get('exit')==0,p
    assert r['inputs_sha256']==r['final_inputs_sha256'],p
    assert not any(r.get(k) for k in ('error','resource_stop','stopped_for_resource_limit')),p
    return r

def main():
    assert HELPER_NAME and HELPER_SHA,'unresolved pins: HELPER_NAME, HELPER_SHA'
    HELPER=Q/HELPER_NAME
    assert sha(HELPER)==HELPER_SHA
    spec=importlib.util.spec_from_file_location('verify_repaired',HELPER)
    h=importlib.util.module_from_spec(spec);spec.loader.exec_module(h)
    assert not h.unresolved(),h.unresolved()
    MASTER=h.MASTER
    jobs=[h.VERIFY]+[h.part_paths(p)[0] for p in h.PICTURE_PARTS]
    for job in jobs:
        assert not job.exists(), 'Existing job; do not duplicate: '+job.name
    deadline=time.monotonic()+43200
    while time.monotonic()<deadline:
        try:r=read(MASTER/'resource-receipt.json')
        except (FileNotFoundError,json.JSONDecodeError):r={}
        if r.get('finished'):
            success(MASTER/'resource-receipt.json')
            break
        time.sleep(15)
    else:raise SystemExit(MASTER.name+' not complete; no verifier launched')
    # Each mode rechecks the pinned repaired master's gates and the complete immutable closure.
    for mode,job in [('check',None),('launch-verify',jobs[0]),
                     ('launch-pictures',jobs[1]),
                     ('launch-pictures',jobs[2])]:
        assert sha(HELPER)==HELPER_SHA
        subprocess.run(['/usr/bin/python3','-I','-B',str(HELPER),mode],check=True)
        if job:success(job/'resource-receipt.json')
    print('Technical verification and both picture extractions finished; actual picture, motion, listening, fine-sync and publication review remain open',flush=True)
if __name__=='__main__':main()
