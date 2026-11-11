"""995: wait for actual master993 success, then serial strict verification and both picture parts.
Prepared only. Does not create visual, listening, independent AV or publication acceptance.
"""
import hashlib
import json
from pathlib import Path
import subprocess
import time

D=Path('/data/projects/jmev-stroke-probe-20261106')
Q=D/'qualification01/production-repair17'
MASTER=D/'jobs/csd-combined-master993'
HELPER=Q/'csd-final-verify995.py'
HELPER_SHA='69fa3ded0960f6344f3cbad557caf951a769dfe7e0d57426a614ea8d7dd2b7a2'

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
    assert sha(HELPER)==HELPER_SHA
    for job in ('csd-final-verify995','csd-final-pictures995-overview','csd-final-pictures995-native'):
        assert not (D/'jobs'/job).exists(), 'Existing job; do not duplicate: '+job
    deadline=time.monotonic()+43200
    while time.monotonic()<deadline:
        try:r=read(MASTER/'resource-receipt.json')
        except (FileNotFoundError,json.JSONDecodeError):r={}
        if r.get('finished'):
            success(MASTER/'resource-receipt.json')
            break
        time.sleep(15)
    else:raise SystemExit('Master993 not complete; no verifier launched')
    # Each mode rechecks pinned master993 gates and the complete immutable closure.
    for mode,job in [('check',None),('launch-verify','csd-final-verify995'),
                     ('launch-pictures','csd-final-pictures995-overview'),
                     ('launch-pictures','csd-final-pictures995-native')]:
        assert sha(HELPER)==HELPER_SHA
        subprocess.run(['/usr/bin/python3','-I','-B',str(HELPER),mode],check=True)
        if job:success(D/'jobs'/job/'resource-receipt.json')
    print('995 technical verification and both picture extractions finished; actual picture, motion, listening, fine-sync and publication review remain open',flush=True)
if __name__=='__main__':main()
