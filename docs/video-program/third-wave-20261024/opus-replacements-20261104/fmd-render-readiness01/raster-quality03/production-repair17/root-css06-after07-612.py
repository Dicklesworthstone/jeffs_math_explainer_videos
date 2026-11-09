"""Run existing guarded imports serially after successful existing07 import. Never launches master."""
import pathlib,subprocess,json,time,hashlib,sys
Q=pathlib.Path(__file__).resolve().parent
ROWS=[{'chapter': '06_one_derivative', 'helper': 'root-css06-relay599.py', 'helper_sha': '7c73196defdbf8cbb3fd2a96f3be3ae460869a7a0370ee4f218f4f5994bf0760', 'manifest': 'css352-import371-06-one-derivative-manifest611.json', 'manifest_sha': '6b12cc889dbc15a39b0d88d068d5d819ab5699e10c644a2eff2a225554d5da0c'}]
state=Q/'root-css06-612-events.jsonl'
def event(**d):
 with state.open('a') as f:f.write(json.dumps(dict(utc=time.time(),**d))+'\n')
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def main():
 deadline=time.monotonic()+86400
 probe="import pathlib,json; p=pathlib.Path('/data/projects/jmev-stroke-probe-20261106/jobs/csd-css352-import371-07-one-point-a1/resource-receipt.json'); print(p.read_text() if p.exists() else '{}')"
 while True:
  p=subprocess.run(['ssh','-o','BatchMode=yes','-o','ConnectTimeout=15','csd','/usr/bin/python3 -'],input=probe,text=True,capture_output=True,timeout=60)
  if p.returncode: raise RuntimeError('SSH observation failed; no import started, inspect existing job')
  d=json.loads(p.stdout)
  if d.get('finished'):
   assert d.get('exit')==0 and d.get('inputs_sha256')==d.get('final_inputs_sha256') and not d.get('resource_stop'),d
   break
  assert time.monotonic()<deadline,'existing07 wait deadline; no imports launched'
  time.sleep(60)
 event(prerequisite='07 terminal stable success')
 for row in ROWS:
  assert sha(Q/row['helper'])==row['helper_sha'] and sha(Q/row['manifest'])==row['manifest_sha']
  command=[sys.executable,'-B',str(Q/row['helper']),'run','--chapter',row['chapter'],'--attempt','1','--manifest',row['manifest'],'--manifest-sha',row['manifest_sha'],'--lane-wait-minutes','1440','--record-dir',str(Q/'root-css06-612-records')]
  event(start=row['chapter'],command=command)
  result=subprocess.run(command,cwd=Q)
  event(finished=row['chapter'],exit=result.returncode)
  assert result.returncode==0,'Import stopped; inspect actual attempt, never auto-retry'
 event(done=True,master_launched=False)
if __name__=='__main__':main()
