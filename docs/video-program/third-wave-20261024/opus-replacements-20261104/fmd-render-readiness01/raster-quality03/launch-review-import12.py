"""Explicit CSD guarded receive followed by semantic review-package import."""
import fcntl
import hashlib
import importlib.util
import json
from pathlib import Path
import subprocess
import sys

R = Path('/data/projects/jmev-stroke-probe-20261106')
Q = R/'qualification01/raster-quality12'

def sha(p):
    with p.open('rb') as f:
        return hashlib.file_digest(f,'sha256').hexdigest()

def read(p):
    return json.loads(p.read_text())

def freeze(extra):
    base = R/'csd-raster-media15-inputs.json'
    receipt = read(R/'jobs/csd-raster-media15/resource-receipt.json')
    assert receipt['exit']==0 and receipt['inputs_sha256']==receipt['final_inputs_sha256']==sha(base)
    assert not receipt.get('resource_stop')
    paths = {}
    for row in read(base)['files']:
        p=Path(row['path']);assert sha(p)==row['sha256'],p
        paths[str(p)]=row['sha256']
    candidates = {base,Path(__file__).resolve()}
    for name in ('csd-final-chapter12.py','csd-import-reviewed12.py',
                 'review-transfer-wire12.py','review-transfer-endpoint12.py'):
        candidates.add(Q/name)
    for folder in [R/'jobs/csd-raster-media15',*extra]:
        if folder.is_file():candidates.add(folder)
        else:candidates.update(p for p in folder.rglob('*') if p.is_file())
    for p in candidates:
        h=sha(p)
        if str(p) in paths:assert paths[str(p)]==h
        paths[str(p)]=h
    return {'files':[{'path':p,'sha256':h} for p,h in sorted(paths.items())]}

def guarded(name,command,extra):
    assert not (R/'jobs'/name).exists()
    manifest=R/(name+'-inputs.json')
    frozen=freeze(extra)
    with manifest.open('x') as f:json.dump(frozen,f,indent=2)
    subprocess.run(['/usr/bin/python3','-I','-B',str(R/'guard_csd02.py'),
                    '--phase','native','--job',name,'--inputs',str(manifest),
                    '--',*command],cwd=R,check=True)
    receipt=read(R/'jobs'/name/'resource-receipt.json')
    assert receipt['exit']==0 and receipt['inputs_sha256']==receipt['final_inputs_sha256']==sha(manifest)
    assert not receipt.get('resource_stop')
    return manifest

def main():
    assert len(sys.argv)==3 and sys.argv[1]=='--receive'
    expected=sys.argv[2]
    assert len(expected)==64 and all(c in '0123456789abcdef' for c in expected)
    assert Path(__file__).resolve().parent==Q
    with (Q/'review-import-driver12.lock').open('a') as lock:
        fcntl.flock(lock,fcntl.LOCK_EX|fcntl.LOCK_NB)
        assert not (R/'work/incoming-review12').exists()
        receive='csd-raster-review-receive12'
        manifest=guarded(receive,['/usr/bin/python3','-I','-B',str(Q/'review-transfer-endpoint12.py'),'--receive',expected],[])
        assert sha(R/'work/incoming-review12/transfer.json')==expected
        guarded('csd-raster-review-import12',['/usr/bin/python3','-I','-B',str(Q/'csd-import-reviewed12.py'),'--run'],
                [manifest,R/'jobs'/receive,R/'work/incoming-review12'])
        print('Guarded review transport and semantic import complete; final render is a separate launch.')

if __name__=='__main__':
    main()
