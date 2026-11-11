"""PREPARED, NOT LAUNCHED: require one repaired chapter's stable final render, then its guarded 4K extraction
(969 after-final route); no picture acceptance, no retry. Successor of after-final c27f4ae2 (kept): the
chapter is checked against the run table before the 30 h wait (CC2 L1). Run it as /usr/bin/python3, the
interpreter the final's launcher used, since lineage() compares commands through sys.executable."""
import hashlib
import importlib.util
import json
from pathlib import Path
import subprocess
import sys
import time
import PIL

R=Path('/data/projects/jmev-stroke-probe-20261106')
Q=R/'qualification01/production-repair17'
HELPER=Q/'csd-native-repair-pictures01.py'
def sha(p):
    with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest()
def read(p):return json.loads(Path(p).read_text())
def success(p):
    d=read(p)
    assert d.get('admitted') and d.get('finished') and d.get('exit')==0,p
    assert d['inputs_sha256']==d['final_inputs_sha256'],p
    assert not any(d.get(k) for k in ('error','resource_stop','stopped_for_resource_limit')),p
    return d
def main():
    assert len(sys.argv)==3 and sys.argv[1]=='--chapter'
    chapter=sys.argv[2]
    spec=importlib.util.spec_from_file_location('repair_pictures01',HELPER)
    x=importlib.util.module_from_spec(spec);spec.loader.exec_module(x)
    JOB=x.job(chapter);INPUTS=Q/(JOB.name+'-inputs.json')
    assert not JOB.exists() and not INPUTS.exists()
    spec=importlib.util.spec_from_file_location('repair_final01_run_unbound',x.RUNNER)
    run=importlib.util.module_from_spec(spec);spec.loader.exec_module(run)
    assert chapter in run.CHAPTERS,chapter
    PRIOR=run.job(chapter)
    deadline=time.monotonic()+108000
    while time.monotonic()<deadline:
        try:d=read(PRIOR/'resource-receipt.json')
        except (FileNotFoundError,json.JSONDecodeError):d={}
        if 'finished' in d:
            success(PRIOR/'resource-receipt.json')
            break
        time.sleep(15)
    else:raise SystemExit(chapter+' final not successful; no extraction launched')
    run,final,rows,used=x.lineage(chapter)
    assert final==PRIOR
    for name,digest in rows.items():assert sha(Path(name))==digest,name
    kept,_,more=x.retained(run,chapter)
    def add(p):
        p=str(p);h=sha(p)
        assert rows.get(p,h)==h,('input drift',p)
        rows[p]=h
    for root in (PRIOR,Path(PIL.__file__).parent):
        for p in root.rglob('*'):
            if p.is_file() and p.suffix!='.pyc':add(p)
    for p in sorted(used|more,key=str):add(p)
    for p in (Path(sys.executable).resolve(),Path(__file__).resolve(),HELPER,x.RUNNER,Q/'csd-native844-base761.py'):
        add(p)
    assert rows[str(Q/'csd-native844-base761.py')]==x.BASE761_SHA
    with INPUTS.open('x') as f:
        json.dump(dict(files=[dict(path=k,sha256=v) for k,v in sorted(rows.items())]),f,indent=2)
    subprocess.run([sys.executable,'-I','-B',str(R/'guard_csd02.py'),'--phase','native',
        '--job',JOB.name,'--inputs',str(INPUTS),'--',sys.executable,'-I','-B',
        str(HELPER),'--chapter',chapter],check=True)
    success(JOB/'resource-receipt.json')
    assert set(kept)<=set(read(JOB/'pictures.json')['selected_frames'])
    print(chapter,'final4K pictures extracted; every sheet must still be viewed',flush=True)
if __name__=='__main__':main()
