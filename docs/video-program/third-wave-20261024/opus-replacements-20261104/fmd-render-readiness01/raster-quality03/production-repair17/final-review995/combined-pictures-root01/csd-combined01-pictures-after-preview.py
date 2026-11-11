"""Require stable853 and exact schedule comparison before guarded picture extraction."""
import hashlib
import json
from pathlib import Path
import subprocess
import sys
import time
import PIL

R=Path('/data/projects/jmev-stroke-probe-20261106')
Q=R/'qualification01/production-repair17'
PRIOR=R/'jobs/csd-native-combined-preview01'
JOB=R/'jobs/csd-native-combined-pictures01'
INPUTS=Q/'csd-combined01-pictures-inputs.json'
def sha(p):
    with p.open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest()
def read(p):return json.loads(p.read_text())
def success(p):
    d=read(p)
    assert d.get('admitted') and d.get('finished') and d.get('exit')==0,p
    assert d['inputs_sha256']==d['final_inputs_sha256'],p
    assert not any(d.get(k) for k in ('error','resource_stop','stopped_for_resource_limit')),p
    return d
def main():
    assert not JOB.exists() and not INPUTS.exists()
    deadline=time.monotonic()+43200
    while time.monotonic()<deadline:
        try:d=read(PRIOR/'resource-receipt.json')
        except (FileNotFoundError,json.JSONDecodeError):d={}
        if 'finished' in d:
            success(PRIOR/'resource-receipt.json')
            if (PRIOR/'comparison-combined01.json').exists():break
        time.sleep(15)
    else:raise SystemExit('853 not accepted; no855 launched')
    assert len(read(PRIOR/'comparison-combined01.json')['chapters'])==3
    source=Q/'csd-combined01-preview-inputs.json'
    assert sha(source)==d['inputs_sha256']
    rows={x['path']:x['sha256'] for x in read(source)['files']}
    for name,digest in rows.items():assert sha(Path(name))==digest,name
    for root in (PRIOR,Path(PIL.__file__).parent,Path(PIL.__file__).parent.parent/'pillow.libs',Q/'combined-pictures01'):
        for p in root.rglob('*'):
            if p.is_file() and p.suffix!='.pyc':rows[str(p)]=sha(p)
    for p in (source,Path(sys.executable).resolve(),Path(__file__).resolve(),
              Q/'csd-combined01-pictures.py',Q/'csd-native855-prior775.json'):
        rows[str(p)]=sha(p)
    with INPUTS.open('x') as f:
        json.dump(dict(files=[dict(path=k,sha256=v) for k,v in sorted(rows.items())]),f,indent=2)
    subprocess.run([sys.executable,'-I','-B',str(R/'guard_csd02.py'),'--phase','native',
        '--job',JOB.name,'--inputs',str(INPUTS),'--',sys.executable,'-I','-B',
        str(Q/'csd-combined01-pictures.py')],check=True)
    success(JOB/'resource-receipt.json')
    assert len(read(JOB/'pictures-combined01.json'))==3
    print('Three repaired preview picture sets extracted; every sheet must still be viewed',flush=True)
if __name__=='__main__':main()
