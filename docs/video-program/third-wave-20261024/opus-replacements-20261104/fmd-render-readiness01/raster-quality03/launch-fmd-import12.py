"""Guarded CSD receipt of the export inventory or one final chapter."""
import fcntl
import importlib.util
import json
from pathlib import Path
import subprocess
import sys

R=Path('/data/projects/jmev-stroke-probe-20261106')
Q=R/'qualification01/raster-quality12'
def load(name):
    s=importlib.util.spec_from_file_location(name,Q/name)
    m=importlib.util.module_from_spec(s);s.loader.exec_module(m);return m
L=load('launch-csd-disjoint12.py');W=L.W
EXPORT_JOB=R/'jobs/csd-final-export-receive12'
EXPORT=Q/'final-fmd-export12.json'

def export_child(expected):
    receipt=W.read(EXPORT_JOB/'resource-receipt.json')
    command=['/usr/bin/python3','-I','-B',str(Path(__file__).resolve()),'--export-child',expected]
    assert receipt['admitted'] and 'exit' not in receipt and receipt['policy']==W.POLICY and receipt['command']==command
    raw=sys.stdin.buffer.read(4*1024*1024+1)
    assert 0<len(raw)<=4*1024*1024
    import hashlib
    assert hashlib.sha256(raw).hexdigest()==expected
    data=json.loads(raw)
    assert data['host']=='fmd' and data['binary_sha256']==W.BINARY_SHA
    assignment=W.read(W.ASSIGNMENT);permit=W.read(W.PERMIT)
    keys=[r['chapter'] for r in assignment['assignments'] if r['host']=='fmd']
    assert [r['chapter'] for r in data['records']]==keys
    assert all(r['review']==permit['review_gate_result'] and r['permit_sha256']==W.sha(W.PERMIT) for r in data['records'])
    receiver=load('receive-fmd-chapter12.py')
    for chapter in keys:receiver.selected_rows(data,chapter)
    assert not EXPORT.exists() and not EXPORT.is_symlink()
    target=EXPORT_JOB/'final-fmd-export12.json'
    with target.open('xb') as f:f.write(raw)
    EXPORT.symlink_to(target)

def launch(mode,value):
    with (Q/'fmd-import-driver12.lock').open('a') as lock:
        fcntl.flock(lock,fcntl.LOCK_EX|fcntl.LOCK_NB)
        frozen=L.freeze()
        if mode=='--export':
            assert len(value)==64 and all(c in '0123456789abcdef' for c in value)
            name=EXPORT_JOB.name
            command=['/usr/bin/python3','-I','-B',str(Path(__file__).resolve()),'--export-child',value]
        else:
            assert mode=='--chapter'
            receipt=W.success(EXPORT_JOB.name)
            export_manifest=R/(EXPORT_JOB.name+'-inputs.json')
            assert W.sha(export_manifest)==receipt['inputs_sha256']
            L.verify_inputs(export_manifest)
            receiver=load('receive-fmd-chapter12.py')
            receiver.selected_rows(W.read(EXPORT),value)
            name='csd-fmd-final-import-'+value+'-12'
            command=['/usr/bin/python3','-I','-B',str(Q/'receive-fmd-chapter12.py'),'--chapter',value]
            frozen['files'].extend({'path':str(p),'sha256':W.sha(p)} for p in (EXPORT,export_manifest,EXPORT_JOB/'resource-receipt.json'))
        assert not (R/'jobs'/name).exists()
        manifest=R/(name+'-inputs.json')
        with manifest.open('x') as f:json.dump(frozen,f,indent=2)
        L.verify_inputs(manifest)
        subprocess.run(['/usr/bin/python3','-I','-B',str(R/'guard_csd02.py'),'--phase','native','--job',name,'--inputs',str(manifest),'--',*command],cwd=R,check=True)
        W.success(name)
        if mode=='--chapter':load('verify-fmd-chapter12.py').verify(value)
        print('Guarded import passed:',name)

if __name__=='__main__':
    assert len(sys.argv)==3 and Path(__file__).resolve().parent==Q
    if sys.argv[1]=='--export-child':export_child(sys.argv[2])
    else:launch(*sys.argv[1:])
