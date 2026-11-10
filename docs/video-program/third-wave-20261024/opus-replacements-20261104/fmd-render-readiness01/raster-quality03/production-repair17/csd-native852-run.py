"""Guard-owned native844 timeline/preview with Linux cwd and complete child cleanup."""
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import signal
import sys
D=Path('/data/projects/jmev-stroke-probe-20261106')
Q=D/'qualification01/production-repair17'
def sha(p):
    with p.open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest()
def read(p):return json.loads(p.read_text())
def main():
    assert len(sys.argv)==2 and sys.argv[1] in ('timeline','preview')
    mode=sys.argv[1]
    job=D/'jobs'/('csd-native-timeline852' if mode=='timeline' else 'csd-native-preview853')
    base=Q/'csd-native844-base761.py'
    assert sha(base)=='32bf6d9e08c1f21f2445debfd46ba090e0374007e4da5516310cb122774099e5'
    spec=importlib.util.spec_from_file_location('owned852',base)
    h=importlib.util.module_from_spec(spec);spec.loader.exec_module(h)
    h.own_group();h.STATE['job']=job;h.CRATE=D/'work/production-repo07'
    for sig in h.STOP_SIGNALS:signal.signal(sig,h.on_stop)
    receipt=read(job/'resource-receipt.json')
    assert receipt['admitted'] is True and 'finished' not in receipt and 'exit' not in receipt
    assert receipt['phase']=='native' and receipt['policy']==dict(admission_gib=10,stop_free_gib=8,
        phase_growth_gib=4,work_cap_gib=56,target_growth_cap_gib=20,sample_seconds=10)
    assert os.environ['CARGO_TARGET_DIR']==str(D/'work')
    assert sorted(p.name for p in job.iterdir())==['command.log','resource-receipt.json']
    qualification=read(D/'jobs/csd-native-build844/qualification.json')
    binary=D/'jobs/csd-native-build844/complex_analysis_visual-844'
    assert sha(binary)==qualification['binary_sha256']
    os.sched_setaffinity(0,sorted(os.sched_getaffinity(0))[:4])
    os.environ.update(PATH='/usr/bin:/bin',TMPDIR=str(D/'work/tmp'),XDG_CACHE_HOME=str(D/'work/cache'))
    args=[binary,'render','09_boundary','--fps','60','--threads','4','--narration',
        D/'jobs/csd-native-delivery849/narration','--out',job/('timeline-output' if mode=='timeline' else 'preview')]
    args+=['--timeline-only'] if mode=='timeline' else ['--res','960x540','--lint','--crf','18','--preset','veryfast']
    h.run_step(job,'native',args,14400,None)
if __name__=='__main__':main()
