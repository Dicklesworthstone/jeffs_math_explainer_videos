"""Require stable chapter11 preview and exact schedule comparison before guarded picture extraction."""
import hashlib
import json
from pathlib import Path
import subprocess
import sys
import time
import PIL

R=Path('/data/projects/jmev-stroke-probe-20261106')
Q=R/'qualification01/production-repair17'
PRIOR=R/'jobs/csd-native-counting-preview02'
JOB=R/'jobs/csd-native-counting-pictures02'
INPUTS=Q/'csd-counting02-pictures-inputs.json'
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
            if (PRIOR/'comparison-counting02.json').exists():break
        time.sleep(15)
    else:raise SystemExit('Chapter11 preview not accepted; no pictures launched')
    assert len(read(PRIOR/'comparison-counting02.json')['chapters'])==1
    source=Q/'csd-counting02-preview-inputs.json'
    assert sha(source)==d['inputs_sha256']
    rows={x['path']:x['sha256'] for x in read(source)['files']}
    for name,digest in rows.items():assert sha(Path(name))==digest,name
    for root in (PRIOR,Path(PIL.__file__).parent,Path(PIL.__file__).parent.parent/'pillow.libs',Q/'counting-pictures02'):
        for p in root.rglob('*'):
            if p.is_file() and p.suffix!='.pyc':rows[str(p)]=sha(p)
    for p in (source,Path(sys.executable).resolve(),Path(__file__).resolve(),
              Q/'csd-counting02-pictures.py'):
        rows[str(p)]=sha(p)
    with INPUTS.open('x') as f:
        json.dump(dict(files=[dict(path=k,sha256=v) for k,v in sorted(rows.items())]),f,indent=2)
    subprocess.run([sys.executable,'-I','-B',str(R/'guard_csd02.py'),'--phase','native',
        '--job',JOB.name,'--inputs',str(INPUTS),'--',sys.executable,'-I','-B',
        str(Q/'csd-counting02-pictures.py')],check=True)
    success(JOB/'resource-receipt.json')
    assert len(read(JOB/'pictures-counting02.json'))==1
    print('Chapter11 preview pictures extracted; every sheet must still be viewed',flush=True)
if __name__=='__main__':main()
