"""Owner-workstation relay. No render or publication; preserve all failed attempts."""
from pathlib import Path
import json,subprocess,time,hashlib
Q=Path(__file__).resolve().parent
HQ='/data/projects/jmev-hz2-render-worker-20261107/qualification01/production19'
TQ='/data/projects/jmev-render-worker-20261107/qualification01/production19'
receipt='/data/projects/jmev-hz2-render-worker-20261107/jobs/hz2-production19-final-07-one-point84/resource-receipt.json'
def run(argv,**kwargs):return subprocess.run(argv,check=True,**kwargs)
for _ in range(720):
 result=subprocess.run(['ssh','hz2','cat '+receipt],capture_output=True)
 if result.returncode!=0:
  time.sleep(60);continue
 r=json.loads(result.stdout)
 if r.get('finished'):
  assert r.get('exit')==0 and r['inputs_sha256']==r['final_inputs_sha256'] and not any(r.get(k) for k in ('error','resource_stop','stopped_for_resource_limit'));break
 time.sleep(60)
else:raise RuntimeError('Final07 wait deadline, no import launched')
# Wait for completed chapter12 import60 before appending chapter07 files.
for _ in range(720):
 result=subprocess.run(['ssh','trj','cat /data/projects/jmev-render-worker-20261107/jobs/trj-import-hz2-final12-60/resource-receipt.json'],capture_output=True)
 if result.returncode==0:
  prior=json.loads(result.stdout)
  if prior.get('finished'):
   assert prior.get('exit')==0 and prior['inputs_sha256']==prior['final_inputs_sha256'] and not prior.get('resource_stop');break
 time.sleep(60)
else:raise RuntimeError('Prior import12 wait deadline; no import07 launched')
p=Q/'final07-export86.json'
with p.open('xb') as out:run(['ssh','hz2','python3 -I -B '+HQ+'/hz2-final07-export86.py'],stdout=out)
data=json.loads(p.read_text());assert data['files']
for host,target in [('hz2',HQ),('trj',TQ)]:
 run(['ssh',host,'test ! -e '+target+'/final07-export86.json'])
 run(['scp',str(p),host+':'+target+'/final07-export86.json'])
 digest=run(['ssh',host,'sha256sum '+target+'/final07-export86.json'],capture_output=True).stdout.decode().split()[0]
 assert digest==hashlib.sha256(p.read_bytes()).hexdigest()
run(['ssh','trj','python3 -I -B '+TQ+'/trj-prepare-import161.py'])
# Remote receiver waits its existing guarded exclusive lane. Sender blocks
# naturally on the pipe; no unguarded destination writes or duplicate render.
with (Q/'final07-relay86.stderr').open('xb') as err,(Q/'final07-relay86.stdout').open('xb') as out:
 sender=subprocess.Popen(['ssh','hz2','python3 -I -B '+HQ+'/hz2-final07-send86.py'],stdout=subprocess.PIPE,stderr=err)
 receiver=subprocess.Popen(['ssh','trj','python3 -I -B /data/projects/jmev-render-worker-20261107/guard_trj01.py --phase native --job trj-import-hz2-final07-86 --inputs '+TQ+'/import-hz2-final07-86-inputs.json -- python3 -I -B '+TQ+'/trj-import-hz2-final07-161.py'],stdin=sender.stdout,stdout=out,stderr=err)
 sender.stdout.close();b=receiver.wait();a=sender.wait()
with (Q/'final07-relay86-exit.json').open('x') as f:json.dump(dict(sender_exit=a,receiver_exit=b),f)
assert a==b==0,(a,b)
r=json.loads(run(['ssh','trj','cat /data/projects/jmev-render-worker-20261107/jobs/trj-import-hz2-final07-86/resource-receipt.json'],capture_output=True).stdout)
assert r.get('exit')==0 and r.get('finished') and r['inputs_sha256']==r['final_inputs_sha256'] and not any(r.get(k) for k in ('error','resource_stop','stopped_for_resource_limit'))
with (Q/'final07-import86-receipt.json').open('x') as f:json.dump(r,f,indent=2)
print('Exact HZ2 chapter07 import terminal success; native mixed-host master and full review still pending.')
