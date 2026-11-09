from pathlib import Path
import subprocess,json
Q=Path(__file__).resolve().parent
R='/data/projects/jmev-ts2-render-worker-20261107'
with (Q/'ts2-runtime-transfer118.stderr').open('xb') as err,(Q/'ts2-runtime-transfer118.stdout').open('xb') as out:
 sender=subprocess.Popen(['ssh','-o','BatchMode=yes','trj','python3 -I -B /data/projects/jmev-render-worker-20261107/qualification01/production19/css-runtime-sender75.py'],stdout=subprocess.PIPE,stderr=err)
 receiver=subprocess.Popen(['ssh','-o','BatchMode=yes','ts2','python3 -I -B '+R+'/guard_ts2118.py --phase native --job ts2-runtime-transfer118 --inputs '+R+'/qualification01/runtime118-inputs.json -- python3 -I -B '+R+'/qualification01/ts2-runtime-receiver118.py'],stdin=sender.stdout,stdout=out,stderr=err)
 sender.stdout.close();b=receiver.wait();a=sender.wait()
with (Q/'ts2-runtime-transfer118-exits.json').open('x') as f:json.dump(dict(sender_exit=a,receiver_exit=b),f)
assert a==b==0,(a,b)
r=json.loads(subprocess.check_output(['ssh','-o','BatchMode=yes','ts2','cat '+R+'/jobs/ts2-runtime-transfer118/resource-receipt.json']))
assert r.get('finished') and r.get('exit')==0 and r['inputs_sha256']==r['final_inputs_sha256'] and not r.get('resource_stop')
with (Q/'ts2-runtime-transfer118-receipt.json').open('x') as f:json.dump(r,f,indent=2)
