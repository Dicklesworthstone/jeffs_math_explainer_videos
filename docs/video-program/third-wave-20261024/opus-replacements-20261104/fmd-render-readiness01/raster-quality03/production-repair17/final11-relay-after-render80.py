"""Wait for terminal CSS final11, then relay through the guarded TRJ lane."""
from pathlib import Path
import json,subprocess,time,hashlib
Q=Path(__file__).resolve().parent
CQ='/data/projects/jmev-css-render-worker-20261107/qualification01'
TQ='/data/projects/jmev-render-worker-20261107/qualification01/production19'
receipt='/data/projects/jmev-css-render-worker-20261107/jobs/css-production19-final-11-counting78/resource-receipt.json'
def run(argv,**kwargs):return subprocess.run(argv,check=True,**kwargs)
for _ in range(720):
 result=run(['ssh','-o','BatchMode=yes','css','cat '+receipt],capture_output=True)
 r=json.loads(result.stdout)
 if r.get('finished'):
  assert r.get('exit')==0 and r['inputs_sha256']==r['final_inputs_sha256'] and not any(r.get(k) for k in ('error','resource_stop','stopped_for_resource_limit'));break
 time.sleep(60)
else:raise RuntimeError('Final11 deadline; no import launched')
p=Q/'final11-export80.json'
with p.open('xb') as out:run(['ssh','css','python3 -I -B '+CQ+'/css-final11-export80.py'],stdout=out)
data=json.loads(p.read_text());assert data['files']
for host,target in [('css',CQ),('trj',TQ)]:
 run(['ssh',host,'test ! -e '+target+'/final11-export80.json'])
 run(['scp',str(p),host+':'+target+'/final11-export80.json'])
 digest=run(['ssh',host,'sha256sum '+target+'/final11-export80.json'],capture_output=True).stdout.decode().split()[0]
 assert digest==hashlib.sha256(p.read_bytes()).hexdigest()
run(['ssh','trj','python3 -I -B '+TQ+'/trj-prepare-css-import80.py'])
with (Q/'final11-relay80.stderr').open('xb') as err,(Q/'final11-relay80.stdout').open('xb') as out:
 sender=subprocess.Popen(['ssh','css','python3 -I -B '+CQ+'/css-final11-send80.py'],stdout=subprocess.PIPE,stderr=err)
 receiver=subprocess.Popen(['ssh','trj','python3 -I -B /data/projects/jmev-render-worker-20261107/guard_trj01.py --phase native --job trj-import-css-final11-80 --inputs '+TQ+'/import-css-final11-80-inputs.json -- python3 -I -B '+TQ+'/trj-import-css-final11-80.py'],stdin=sender.stdout,stdout=out,stderr=err)
 sender.stdout.close();b=receiver.wait();a=sender.wait()
with (Q/'final11-relay80-exit.json').open('x') as f:json.dump(dict(sender_exit=a,receiver_exit=b),f)
assert a==b==0,(a,b)
r=json.loads(run(['ssh','trj','cat /data/projects/jmev-render-worker-20261107/jobs/trj-import-css-final11-80/resource-receipt.json'],capture_output=True).stdout)
assert r.get('exit')==0 and r.get('finished') and r['inputs_sha256']==r['final_inputs_sha256'] and not any(r.get(k) for k in ('error','resource_stop','stopped_for_resource_limit'))
with (Q/'final11-import80-receipt.json').open('x') as f:json.dump(r,f,indent=2)
print('Exact CSS chapter11 import succeeded; master replay and final review remain pending.')
