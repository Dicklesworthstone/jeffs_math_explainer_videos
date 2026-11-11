"""Wait for repaired previews before guarded chapter11 candidate qualification."""
from pathlib import Path
import hashlib,json,subprocess,time
D=Path('/data/projects/jmev-stroke-probe-20261106')
Q=D/'qualification01/production-repair17'
receipt=D/'jobs/csd-native-combined-preview01/resource-receipt.json'
launcher=Q/'csd-counting02-launch.py'
assert hashlib.sha256(launcher.read_bytes()).hexdigest()=='fc34004ae9d2fa8e2ad82aa91410ec2fd3014664008e404609a23735e25937bb'
deadline=time.monotonic()+43200
while time.monotonic()<deadline:
    try:r=json.loads(receipt.read_text())
    except (FileNotFoundError,json.JSONDecodeError):
        time.sleep(15);continue
    if 'finished' not in r:
        time.sleep(15);continue
    assert r.get('admitted') and r.get('exit')==0 and r['inputs_sha256']==r['final_inputs_sha256'],r
    assert not any(r.get(k) for k in ('error','resource_stop','stopped_for_resource_limit')),r
    assert len(json.loads((receipt.parent/'comparison-combined01.json').read_text())['chapters'])==3
    break
else:raise SystemExit('Preview not terminal; no build launched')
assert hashlib.sha256(launcher.read_bytes()).hexdigest()=='fc34004ae9d2fa8e2ad82aa91410ec2fd3014664008e404609a23735e25937bb'
subprocess.run(['/usr/bin/python3','-I','-B',str(launcher),'--launch'],check=True)
