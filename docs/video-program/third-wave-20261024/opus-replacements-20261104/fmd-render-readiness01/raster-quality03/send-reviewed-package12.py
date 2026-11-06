"""Explicit workstation relay of an already qualified FMD review package."""
import hashlib
import json
from pathlib import Path
import subprocess
import sys

F = '/data/projects/jmev-fmd-render-qualification-20261106-1111'
C = '/data/projects/jmev-stroke-probe-20261106'
SSH = ['ssh','-o','BatchMode=yes','-o','ConnectTimeout=10','-o','ServerAliveInterval=30','-o','ServerAliveCountMax=3']

def main():
    assert sys.argv[1:] == ['--run']
    folder=Path(__file__).parent/'review-relay12'
    assert not folder.exists(), 'Preserve existing relay; inspect before recovery'
    package=F+'/jobs/fmd-raster-transfer12/production-inputs-raster12/transfer.json'
    raw=subprocess.check_output([*SSH,'fmd','cat '+package],timeout=60)
    data=json.loads(raw)
    assert data['origin_host']=='fmd' and len(data['files'])==53
    expected=hashlib.sha256(raw).hexdigest()
    folder.mkdir()
    (folder/'expected-transfer.json').write_bytes(raw)
    send_command='/usr/bin/python3 -I -B '+F+'/qualification01/raster-quality03/review-transfer-endpoint12.py --send '+expected
    receive_command='/usr/bin/python3 -I -B '+C+'/qualification01/raster-quality12/launch-review-import12.py --receive '+expected
    with (folder/'sender.stderr').open('xb') as se, (folder/'receiver.stderr').open('xb') as re, (folder/'receiver.stdout').open('xb') as ro:
        sender=subprocess.Popen([*SSH,'fmd',send_command],stdout=subprocess.PIPE,stderr=se)
        receiver=subprocess.Popen([*SSH,'csd',receive_command],stdin=sender.stdout,stdout=ro,stderr=re)
        sender.stdout.close()
        receive_exit=receiver.wait()
        send_exit=sender.wait()
    with (folder/'relay-result.json').open('x') as f:
        json.dump({'expected_transfer_sha256':expected,'sender_exit':send_exit,'receiver_exit':receive_exit,
                   'scope':'Transport result only; remote original-guard receipts and semantic import must also pass.'},f,indent=2)
    assert send_exit==receive_exit==0, 'Preserve failures; never blindly repeat the relay'
    for name in ('csd-raster-review-receive12','csd-raster-review-import12'):
        raw=subprocess.check_output([*SSH,'csd','cat '+C+'/jobs/'+name+'/resource-receipt.json'],timeout=60)
        receipt=json.loads(raw)
        assert receipt.get('finished') and receipt['exit']==0 and not receipt.get('resource_stop')
        assert receipt['inputs_sha256']==receipt['final_inputs_sha256']
        with (folder/(name+'.json')).open('xb') as f:f.write(raw)
    print('Both guarded review-transfer stages completed. No final renderer launched.')

if __name__=='__main__':
    main()
