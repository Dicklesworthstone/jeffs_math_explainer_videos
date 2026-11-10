"""Wait for stable844; freeze the retained delivery closure and run849 once."""
import hashlib
import json
from pathlib import Path
import subprocess
import sys
import time

D = Path('/data/projects/jmev-stroke-probe-20261106')
Q = D/'qualification01/production-repair17'
BUILD = D/'jobs/csd-native-build844'
JOB = D/'jobs/csd-native-delivery849'
SOURCE = D/'work/native-boundary844'
CANONICAL = SOURCE/'workspace/explainer_corpus/complex_analysis_visual/narration_units.json'
INPUTS = Q/'csd-native849-inputs.json'

def sha(p):
    with Path(p).open('rb') as f: return hashlib.file_digest(f, 'sha256').hexdigest()
def read(p): return json.loads(p.read_text())
def success(p):
    d=read(p)
    assert d.get('admitted') and d.get('finished') and d.get('exit') == 0, p
    assert d['inputs_sha256'] == d['final_inputs_sha256'], p
    assert not any(d.get(k) for k in ('error','resource_stop','stopped_for_resource_limit')), p
    return d

def main():
    assert not JOB.exists() and not INPUTS.exists()
    deadline=time.monotonic()+43200
    while time.monotonic()<deadline:
        try: d=read(BUILD/'resource-receipt.json')
        except (FileNotFoundError,json.JSONDecodeError): d={}
        if 'finished' in d: break
        time.sleep(15)
    else: raise SystemExit('844 not terminal; no849 launch')
    success(BUILD/'resource-receipt.json')
    rows={}
    for manifest,receipt in [
        (Q/'csd-native844-inputs.json',BUILD/'resource-receipt.json'),
        (Q/'review-repair-rebind313-inputs.json',D/'jobs/csd-review-repair-rebind313/resource-receipt.json')]:
        assert sha(manifest)==success(receipt)['inputs_sha256']
        for row in read(manifest)['files']:
            p=Path(row['path']); assert sha(p)==row['sha256'], p
            rows[str(p)]=row['sha256']
        rows[str(manifest)]=sha(manifest)
        rows[str(receipt)]=sha(receipt)
    # Build immutable original plan is already frozen in844-inputs; helper
    # checks844 qualification's entire source listing, preserves this canonical
    # then explicitly allows only its runtime binding plus two new source files.
    assert sha(CANONICAL)=='995e25c7788943deff65c4d84050d8a17edc3c2c3498be0d665d7ac5b661357a'
    assert str(CANONICAL) not in rows
    for root in (SOURCE,BUILD,D/'jobs/csd-review-repair-rebind313/narration'):
        for p in root.rglob('*'):
            assert not p.is_symlink(), p
            if p.is_file() and p!=CANONICAL: rows[str(p)]=sha(p)
    for p in [Q/'csd-native849-delivery.py',Q/'csd-native849-map.json',Path(__file__).resolve(),
              Path(sys.executable).resolve(),D/'guard_csd02.py']:
        rows[str(p)]=sha(p)
    with INPUTS.open('x') as f:
        json.dump(dict(files=[dict(path=k,sha256=v) for k,v in sorted(rows.items())]),f,indent=2)
    print('849 frozen inputs',len(rows),flush=True)
    subprocess.run([sys.executable,'-I','-B',str(D/'guard_csd02.py'),'--phase','native',
        '--job',JOB.name,'--inputs',str(INPUTS),'--',sys.executable,'-I','-B',
        str(Q/'csd-native849-delivery.py'),'--run'],check=True)
    success(JOB/'resource-receipt.json')
    assert read(JOB/'delivery849.json')['wavs_byte_identical']==105
    print('849 successful; no timeline or picture acceptance',flush=True)

if __name__=='__main__': main()
