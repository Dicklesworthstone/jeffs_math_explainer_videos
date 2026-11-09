"""Retry terminal failed native10 transport once; start sender only after guarded admission."""
import json,subprocess,time,shlex
D='/data/projects/jmev-stroke-probe-20261106'
Q=D+'/qualification01/production-repair17'
M='/Users/jemanuel/scratch/jmev-complex-native-20261109/qualification'
JOB='csd-mini-import576-10-a2'
SHA='036fdc7fea3f9ce459fd8e5ffd8ea2a8fba96304b55a9a034ec647f7ac406f93'
SSH=['ssh','-o','BatchMode=yes','-o','ConnectTimeout=15','-o','ServerAliveInterval=30','-o','ServerAliveCountMax=120']
def state():
 code="import pathlib; p=pathlib.Path(%r); print(p.read_text() if p.exists() else '{}')"%(D+'/jobs/'+JOB+'/resource-receipt.json')
 r=subprocess.run(SSH+['csd','/usr/bin/python3 -c '+shlex.quote(code)],capture_output=True,text=True,timeout=60,check=True)
 return json.loads(r.stdout)
def main():
 assert not state(),'Existing attempt; inspect, never duplicate'
 inputs=Q+'/'+JOB+'-inputs.json'
 command=['/usr/bin/python3','-I','-B',D+'/guard_csd02.py','--phase','native','--job',JOB,'--inputs',inputs,'--','/usr/bin/timeout','14400','/usr/bin/python3','-I','-B',Q+'/root-mini576-transport.py','receive','--manifest',Q+'/root-native575-export589.json','--job',JOB,'--inputs',inputs]
 receiver=subprocess.Popen(SSH+['csd',shlex.join(command)],stdin=subprocess.PIPE)
 sender=None
 try:
  deadline=time.monotonic()+86400
  while True:
   assert receiver.poll() is None,('receiver exited before sender',receiver.returncode)
   d=state()
   assert not d.get('finished'),d
   if d.get('admitted'):
    assert d['inputs_sha256']==SHA
    break
   assert time.monotonic()<deadline,'admission wait expired'
   time.sleep(10)
  print('ADMITTED; starting source transport',flush=True)
  send=['python3','-I','-B',M+'/root-mini576-transport.py','send','--manifest',M+'/root-native575-export589.json']
  sender=subprocess.Popen(SSH+['mac-mini-max',shlex.join(send)],stdout=receiver.stdin)
  receiver.stdin.close()
  assert sender.wait(timeout=14400)==0,'sender failed; preserve and inspect'
  assert receiver.wait(timeout=14400)==0,'receiver failed; preserve and inspect'
  d=state();assert d.get('finished') and d.get('exit')==0 and d['inputs_sha256']==d['final_inputs_sha256']==SHA and not d.get('resource_stop')
  print('IMPORT621_COMPLETED; materialization and final verification remain',flush=True)
 finally:
  if receiver.stdin and not receiver.stdin.closed:receiver.stdin.close()
  for p in (sender,receiver):
   if p is not None and p.poll() is None:
    p.terminate()
    print('SSH stopped after error; inspect exact remote processes before retry',flush=True)
if __name__=='__main__':main()
