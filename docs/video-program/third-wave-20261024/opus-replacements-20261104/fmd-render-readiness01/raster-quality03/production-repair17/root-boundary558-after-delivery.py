"""After exact557 delivery, guard one native09 timeline and compare all three full tables."""
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import time

R=Path.home()/'scratch/jmev-complex-native-20261109'
Q=R/'qualification'
DELIVERY=R/'jobs/mini-boundary-delivery557'
JOB=R/'jobs/mini-boundary-timeline558'
INPUTS=Q/'root-boundary558-inputs.json'
REF=R/'reference558'

def sha(p):
    with p.open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest()

def read(p):return json.loads(p.read_text())

def success(p):
    d=read(p)
    assert d.get('admitted') and d.get('finished') and d.get('exit')==0,p
    assert d['inputs_sha256']==d['final_inputs_sha256'] and not d.get('resource_stop'),p
    return d

def main():
    assert not JOB.exists() and not INPUTS.exists()
    deadline=time.monotonic()+86400
    while time.monotonic()<deadline:
        try:d=read(DELIVERY/'resource-receipt.json')
        except (FileNotFoundError,json.JSONDecodeError):d={}
        if 'finished' in d:break
        time.sleep(15)
    else:raise SystemExit('557 still pending; no timeline launched')
    d=success(DELIVERY/'resource-receipt.json')
    source=Q/'root-boundary557-inputs.json'
    assert sha(source)==d['inputs_sha256']
    rows={x['path']:x['sha256'] for x in read(source)['files']}
    for name,digest in rows.items():assert sha(Path(name))==digest,name
    delivery=read(DELIVERY/'delivery557.json')
    assert delivery['wavs_byte_identical']==105 and delivery['delivery']['boundary']['selected_cues']==9
    for name,digest in delivery['source_after'].items():
        p=R/'source552'/name;assert sha(p)==digest,p;rows[str(p)]=digest
    for name,digest in read(Q/'root-boundary558-reference.json')['files'].items():
        p=REF/name;assert sha(p)==digest,p;rows[str(p)]=digest
    binary=R/'jobs/native-boundary553/complex_analysis_visual-553'
    assert sha(binary)=='119ebe43fbde39e90cb1352656be280b71d52aa736751f0e6f819af6ceb9613d'
    for p in [source,binary,Path(__file__).resolve(),Q/'root-boundary558-reference.json',Path(sys.executable).resolve()]:
        rows[str(p)]=sha(p)
    for p in DELIVERY.rglob('*'):
        if p.is_file():rows[str(p)]=sha(p)
    with INPUTS.open('x') as f:json.dump(dict(files=[dict(path=k,sha256=v) for k,v in sorted(rows.items())]),f,indent=2)
    out=JOB/'timeline-output'
    command=[sys.executable,'-I','-B',str(Q/'cc2-mini-max-guard508.py'),'--phase','native',
             '--job',JOB.name,'--inputs',str(INPUTS),'--',str(binary),'render','09_boundary',
             '--timeline-only','--fps','60','--threads','4','--narration',str(DELIVERY/'narration'),'--out',str(out)]
    env=dict(os.environ,PATH=str(Path.home()/'.local/bin')+':/opt/homebrew/bin:/usr/bin:/bin')
    subprocess.run(command,env=env,check=True)
    success(JOB/'resource-receipt.json')
    old=b'/data/projects/jmev-stroke-probe-20261106/jobs/csd-review-repair-rebind313/narration'
    new=str(DELIVERY/'narration').encode()
    checks=[]
    for kind in ['timeline','cues','cuelog']:
        name='09_boundary.'+kind+'.tsv';a=REF/name;b=out/name
        assert a.read_bytes().replace(old,new)==b.read_bytes(),kind
        checks.append(dict(kind=kind,reference_sha256=sha(a),actual_sha256=sha(b),equal_after_named_path_mapping=True))
    lines=(out/'09_boundary.timeline.tsv').read_text().splitlines()
    frames=next(int(x.split('\t')[1]) for x in lines if x.startswith('frames\t'))
    assert frames==16504
    with (JOB/'comparison558.json').open('x') as f:
        json.dump(dict(frames=frames,tables=checks,scope='Full table equality after narration path substitution only; no picture or audio acceptance.'),f,indent=2)
    print('558 timeline passed:16504frames, all3 exact normalized tables',flush=True)

if __name__=='__main__':main()
