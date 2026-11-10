"""After Linux849 delivery, guard native844 timeline and compare all three full tables."""
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import time

R=Path('/data/projects/jmev-stroke-probe-20261106')
Q=R/'qualification01/production-repair17'
DELIVERY=R/'jobs/csd-native-delivery849'
JOB=R/'jobs/csd-native-timeline852'
INPUTS=Q/'csd-native852-inputs.json'
REF=R/'jobs/csd-review-repair-timeline313/timeline-output'

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
    deadline=time.monotonic()+86400
    while time.monotonic()<deadline:
        try:d=read(DELIVERY/'resource-receipt.json')
        except (FileNotFoundError,json.JSONDecodeError):d={}
        if 'finished' in d:break
        time.sleep(15)
    else:raise SystemExit('849 still pending; no timeline launched')
    d=success(DELIVERY/'resource-receipt.json')
    source=Q/'csd-native849-inputs.json'
    assert sha(source)==d['inputs_sha256']
    rows={x['path']:x['sha256'] for x in read(source)['files']}
    for name,digest in rows.items():assert sha(Path(name))==digest,name
    delivery=read(DELIVERY/'delivery849.json')
    assert delivery['wavs_byte_identical']==105 and delivery['delivery']['boundary']['selected_cues']==9
    for name,digest in delivery['source_after'].items():
        p=R/'work/native-boundary844'/name;assert sha(p)==digest,p;rows[str(p)]=digest
    for name,digest in read(Q/'csd-native852-reference.json')['files'].items():
        p=REF/name;assert sha(p)==digest,p;rows[str(p)]=digest
    binary=R/'jobs/csd-native-build844/complex_analysis_visual-844'
    assert sha(binary)==read(R/'jobs/csd-native-build844/qualification.json')['binary_sha256']
    success(R/'jobs/csd-review-repair-timeline313/resource-receipt.json')
    for p in [source,binary,Path(__file__).resolve(),Q/'csd-native852-reference.json',Path(sys.executable).resolve(),Q/'csd-native852-run.py',Path('/usr/bin/ffmpeg'),Path('/usr/bin/ffprobe'),R/'jobs/csd-review-repair-timeline313/resource-receipt.json']:
        rows[str(p)]=sha(p)
    for p in DELIVERY.rglob('*'):
        if p.is_file():rows[str(p)]=sha(p)
    with INPUTS.open('x') as f:json.dump(dict(files=[dict(path=k,sha256=v) for k,v in sorted(rows.items())]),f,indent=2)
    out=JOB/'timeline-output'
    command=[sys.executable,'-I','-B',str(R/'guard_csd02.py'),'--phase','native',
             '--job',JOB.name,'--inputs',str(INPUTS),'--',sys.executable,'-I','-B',
             str(Q/'csd-native852-run.py'),'timeline']
    subprocess.run(command,check=True)
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
    with (JOB/'comparison852.json').open('x') as f:
        json.dump(dict(frames=frames,tables=checks,scope='Full table equality after narration path substitution only; no picture or audio acceptance.'),f,indent=2)
    print('852 timeline passed:16504frames, all3 exact normalized tables',flush=True)

if __name__=='__main__':main()
