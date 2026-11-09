"""Run existing guarded imports serially after successful existing01 import. Never launches master."""
import pathlib,subprocess,json,time,hashlib,sys
Q=pathlib.Path(__file__).resolve().parent
ROWS=[{'chapter': '04_reading_color', 'helper': 'cc3-css352-import-relay394.py', 'helper_sha': '371f5d1ff415cc1309beebc7feba0f59bd4ba9b139153453d94326be16e64c8c', 'manifest': 'css352-import371-04-reading-color-manifest396.json', 'manifest_sha': '04d1cd99e0114e63abfeaefa23aab7d3ce6e56092d77d8a4459fa3cb2da6896c'}, {'chapter': '12_residues', 'helper': 'cc3-css352-import-relay394.py', 'helper_sha': '371f5d1ff415cc1309beebc7feba0f59bd4ba9b139153453d94326be16e64c8c', 'manifest': 'css352-import371-12-residues-manifest397.json', 'manifest_sha': '73c62b8d3c92ced7b4f48a9316ce12440b72424ae29ddc6d8b9a8346451a5359'}, {'chapter': '14_continuation', 'helper': 'cc3-css352-import-relay394.py', 'helper_sha': '371f5d1ff415cc1309beebc7feba0f59bd4ba9b139153453d94326be16e64c8c', 'manifest': 'css352-import371-14-continuation-manifest424.json', 'manifest_sha': '79fc3bf9e5c9ffa86f5f30df19a22431acb2b51d0ac620ac259706344caa5206'}, {'chapter': '08_loop_sums', 'helper': 'cc1-css352-import-relay485.py', 'helper_sha': '05617cb88da8786232ad7234a4e3208d024fe3560bd4713ac035f82f07f162d0', 'manifest': 'css352-import371-08-loop-sums-manifest533.json', 'manifest_sha': '0c827a62138270ba5a06d054f95a3d045a3dcacc8c99f073ee92e29baaa0b11a'}, {'chapter': '07_one_point', 'helper': 'root-css07-relay547.py', 'helper_sha': 'e9a2304985fe19bdb114cc082acb9ac6ace26715e027984fa31b469c20b61f75', 'manifest': 'css352-import371-07-one-point-manifest549.json', 'manifest_sha': '5a7efc1db8cf18c16e0251e9207feb12546915394c966b60ac1740f6aa00f762'}]
state=Q/'root-imports590-events.jsonl'
def event(**d):
 with state.open('a') as f:f.write(json.dumps(dict(utc=time.time(),**d))+'\n')
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def main():
 deadline=time.monotonic()+86400
 probe="import pathlib,json; p=pathlib.Path('/data/projects/jmev-stroke-probe-20261106/jobs/csd-css352-import371-01-the-bump-a1/resource-receipt.json'); print(p.read_text() if p.exists() else '{}')"
 while True:
  p=subprocess.run(['ssh','-o','BatchMode=yes','-o','ConnectTimeout=15','csd','/usr/bin/python3 -'],input=probe,text=True,capture_output=True,timeout=60)
  if p.returncode: raise RuntimeError('SSH observation failed; no import started, inspect existing job')
  d=json.loads(p.stdout)
  if d.get('finished'):
   assert d.get('exit')==0 and d.get('inputs_sha256')==d.get('final_inputs_sha256') and not d.get('resource_stop'),d
   break
  assert time.monotonic()<deadline,'existing01 wait deadline; no imports launched'
  time.sleep(60)
 event(prerequisite='01 terminal stable success')
 for row in ROWS:
  assert sha(Q/row['helper'])==row['helper_sha'] and sha(Q/row['manifest'])==row['manifest_sha']
  command=[sys.executable,'-B',str(Q/row['helper']),'run','--chapter',row['chapter'],'--attempt','1','--manifest',row['manifest'],'--manifest-sha',row['manifest_sha'],'--lane-wait-minutes','1440','--record-dir',str(Q/'root-imports590-records')]
  event(start=row['chapter'],command=command)
  result=subprocess.run(command,cwd=Q)
  event(finished=row['chapter'],exit=result.returncode)
  assert result.returncode==0,'Import stopped; inspect actual attempt, never auto-retry'
 event(done=True,master_launched=False)
if __name__=='__main__':main()
