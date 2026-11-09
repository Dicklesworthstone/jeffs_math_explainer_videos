from pathlib import Path
import subprocess,json
Q=Path(__file__).resolve().parent
R='/data/projects/jmev-ts1-render-worker-20261107'
with (Q/'ts1-runtime-transfer100.stderr').open('xb') as err,(Q/'ts1-runtime-transfer100.stdout').open('xb') as out:
 sender=subprocess.Popen(['ssh','-o','BatchMode=yes','trj','python3 -I -B /data/projects/jmev-render-worker-20261107/qualification01/production19/css-runtime-sender75.py'],stdout=subprocess.PIPE,stderr=err)
 receiver=subprocess.Popen(['ssh','-o','BatchMode=yes','ts1','python3 -I -B '+R+'/guard_ts1100.py --phase native --job ts1-runtime-transfer100 --inputs '+R+'/qualification01/runtime100-inputs.json -- python3 -I -B '+R+'/qualification01/ts1-runtime-receiver100.py'],stdin=sender.stdout,stdout=out,stderr=err)
 sender.stdout.close();b=receiver.wait();a=sender.wait()
with (Q/'ts1-runtime-transfer100-exits.json').open('x') as f:json.dump(dict(sender_exit=a,receiver_exit=b),f)
assert a==b==0,(a,b)
r=json.loads(subprocess.check_output(['ssh','-o','BatchMode=yes','ts1','cat '+R+'/jobs/ts1-runtime-transfer100/resource-receipt.json']))
assert r.get('finished') and r.get('exit')==0 and r['inputs_sha256']==r['final_inputs_sha256'] and not r.get('resource_stop')
with (Q/'ts1-runtime-transfer100-receipt.json').open('x') as f:json.dump(r,f,indent=2)
