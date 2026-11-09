"""After existing586/587 technical success, export/import/materialize09 once. Master still requires actual picture review."""
import hashlib,json,shlex,subprocess,time
from pathlib import Path
Q=Path(__file__).resolve().parent
D='/data/projects/jmev-stroke-probe-20261106';DQ=D+'/qualification01/production-repair17'
M='/Users/jemanuel/scratch/jmev-complex-native-20261109';MQ=M+'/qualification'
NAME='root-boundary593-export629.json';JOB='csd-mini-import594-09-a1'
SSH=['ssh','-o','BatchMode=yes','-o','ConnectTimeout=15','-o','ServerAliveInterval=30','-o','ServerAliveCountMax=120']
def remote(host,code,data=None,timeout=120):
 return subprocess.run(SSH+[host,'python3 -c '+shlex.quote(code)],input=data,capture_output=True,check=True,timeout=timeout).stdout

def read_state(host,path):
 return json.loads(remote(host,"import pathlib; p=pathlib.Path(%r); print(p.read_text() if p.exists() else '{}')"%path))
def success(d):
 assert d.get('finished') and d.get('admitted') and d.get('exit')==0 and not d.get('preflight_only')
 assert d['inputs_sha256']==d['final_inputs_sha256']
 assert not any(d.get(k) for k in ('error','resource_stop','stopped_for_resource_limit'))
def helper(host,path,pin,args=(),timeout=600):
 code="import pathlib,hashlib,runpy,sys; p=pathlib.Path(%r); assert hashlib.sha256(p.read_bytes()).hexdigest()==%r; sys.argv=%r; runpy.run_path(str(p),run_name='__main__')"%(path,pin,[path,*args])
 return remote(host,code,timeout=timeout)
def write_remote(host,path,data):
 digest=hashlib.sha256(data).hexdigest()
 code="import pathlib,sys,hashlib; p=pathlib.Path(%r); b=sys.stdin.buffer.read(); assert hashlib.sha256(b).hexdigest()==%r; f=p.open('xb'); f.write(b); f.close(); assert hashlib.sha256(p.read_bytes()).hexdigest()==%r"%(path,digest,digest)
 remote(host,code,data)
def main():
 assert not (Q/NAME).exists()
 remote('csd',"import pathlib; d=pathlib.Path(%r); q=pathlib.Path(%r); assert not list((d/'jobs').glob('csd-mini-import594-09-*')); assert not list((d/'jobs').glob('csd-native-stage595-09-*')); assert not list(q.glob('csd-mini-import594-09-*-inputs.json')); assert not (q/%r).exists()"%(D,DQ,NAME))
 deadline=time.monotonic()+86400
 while True:
  assert time.monotonic()<deadline,'Existing586/587 wait expired; no import launched'
  try:states=[read_state('mac-mini-max',M+'/jobs/'+n+'/resource-receipt.json') for n in ('mini-boundary-final586','mini-boundary-pictures587')]
  except (OSError,subprocess.SubprocessError):
   print('Observation failed; retry same prerequisite receipts, no duplicate',flush=True);time.sleep(30);continue
  for d in states:
   if d.get('finished'):success(d)
  if all(d.get('finished') for d in states):break
  time.sleep(30)
 data=helper('mac-mini-max',MQ+'/root-boundary593-export.py','95c6ed1fb4fcc989dbf453ab13b2542c6c06c0e8b7d450d7c56ca91cc54b6d02')
 manifest=json.loads(data);assert manifest['chapter']=='09_boundary' and manifest['frames']==16504
 with (Q/NAME).open('xb') as f:f.write(data)
 digest=hashlib.sha256(data).hexdigest()
 write_remote('mac-mini-max',MQ+'/'+NAME,data);write_remote('csd',DQ+'/'+NAME,data)
 prepared=json.loads(helper('csd',DQ+'/root-boundary598-prepare.py','9cd030170842202a95eeb7d8df71409fde04b3438298fa2eb06b8805e30ff3f3',['--manifest',DQ+'/'+NAME,'--manifest-sha',digest,'--attempt','1']))
 assert prepared['job']==JOB and not prepared['launched']
 with (Q/'root-boundary629-prepared.json').open('x') as f:json.dump(prepared,f,indent=2)
 receiver=subprocess.Popen(SSH+['csd',shlex.join(prepared['guard_command'])],stdin=subprocess.PIPE);sender=None
 try:
  while True:
   assert receiver.poll() is None,('receiver exited',receiver.returncode)
   assert time.monotonic()<deadline,'admission wait expired'
   try:d=read_state('csd',D+'/jobs/'+JOB+'/resource-receipt.json')
   except (OSError,subprocess.SubprocessError):
    print('Admission observation failed; retry same receiver',flush=True);time.sleep(30);continue
   assert not d.get('finished'),d
   if d.get('admitted'):
    assert d['inputs_sha256']==prepared['inputs_sha256'];break
   time.sleep(10)
  print('594_ADMITTED; starting source stream',flush=True)
  send=['python3','-I','-B',MQ+'/root-boundary594-transport.py','send','--manifest',MQ+'/'+NAME]
  sender=subprocess.Popen(SSH+['mac-mini-max',shlex.join(send)],stdout=receiver.stdin);receiver.stdin.close()
  assert sender.wait(timeout=14400)==0,'sender failure; no retry'
  assert receiver.wait(timeout=14400)==0,'receiver failure; no retry'
  success(read_state('csd',D+'/jobs/'+JOB+'/resource-receipt.json'))
 finally:
  if not receiver.stdin.closed:receiver.stdin.close()
  for process in (sender,receiver):
   if process is not None and process.poll() is None:
    process.terminate();print('SSH stopped on error; inspect exact remote processes before retry',flush=True)
 print('594_SUCCESS; running existing600 materialization controller',flush=True)
 result=helper('csd',DQ+'/root-boundary600-after-import.py','ee7caceda5893b76da202566ad26b91423bb63640fa61979530f8e0ee0848652',['--import-inputs-sha',prepared['inputs_sha256']],timeout=86400)
 with (Q/'root-boundary629-stage.log').open('xb') as f:f.write(result)
 success(read_state('csd',D+'/jobs/csd-native-stage595-09-a1/resource-receipt.json'))
 print('629_COMPLETE; final pictures must still be viewed before master607 gate; no release acceptance',flush=True)
if __name__=='__main__':main()
