"""Run reviewed seam diagnostic only after successful stable995 verify and both picture jobs."""
import hashlib,json,subprocess,time
from pathlib import Path
D=Path('/data/projects/jmev-stroke-probe-20261106')
Q=D/'qualification01/production-repair17'
H=Q/'master993-audio-seams02.py'
SHA='3d4ac2a3ff11a39e032ee3d4f910dd78fc62002d6ec22109a5d02a1e00cceec3'
def sha(p):
 with p.open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest()
def main():
 assert sha(H)==SHA
 assert not (D/'jobs/csd-master993-audio-seams02').exists()
 assert not (Q/'master993-audio-seams02-inputs.json').exists()
 for name in ['csd-final-verify995','csd-final-pictures995-overview','csd-final-pictures995-native']:
  deadline=time.monotonic()+43200
  while time.monotonic()<deadline:
   try:r=json.loads((D/'jobs'/name/'resource-receipt.json').read_text())
   except (FileNotFoundError,json.JSONDecodeError):r={}
   if r.get('finished'):
    assert r.get('admitted') is True and r.get('exit')==0,name
    assert not any(r.get(k) for k in ['error','resource_stop','stopped_for_resource_limit']),name
    assert r['inputs_sha256']==r['final_inputs_sha256'],name
    break
   time.sleep(15)
  else:raise TimeoutError(name)
 assert sha(H)==SHA
 subprocess.run(['/usr/bin/python3','-I','-B',str(H),'launch'],check=True)
 print('Diagnostic complete; no listening or release acceptance.',flush=True)
if __name__=='__main__':main()
