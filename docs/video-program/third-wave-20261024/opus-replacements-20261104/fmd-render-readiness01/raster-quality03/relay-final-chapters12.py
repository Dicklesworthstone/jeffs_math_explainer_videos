"""Explicit workstation relay of completed FMD chapters into the guarded CSD lane."""
import hashlib,json,re,shlex,subprocess,sys
from pathlib import Path
F='/data/projects/jmev-fmd-render-qualification-20261106-1111'
C='/data/projects/jmev-stroke-probe-20261106'
SSH=['ssh','-o','BatchMode=yes','-o','ConnectTimeout=10','-o','ServerAliveInterval=30','-o','ServerAliveCountMax=3']
def command(*args):return shlex.join(args)
def read(host,path):return subprocess.check_output([*SSH,host,command('cat',path)],timeout=90)
def receipt(folder,name):
    raw=read('csd',C+'/jobs/'+name+'/resource-receipt.json');r=json.loads(raw)
    with (folder/(name+'.json')).open('xb') as f:f.write(raw)
    assert r.get('finished') and r['exit']==0 and not r.get('stopped_for_resource_limit')
    assert r['inputs_sha256']==r['final_inputs_sha256']
def relay(folder,label,send,receive):
    with (folder/(label+'-sender.stderr')).open('xb') as se,(folder/(label+'-receiver.stderr')).open('xb') as re,(folder/(label+'-receiver.stdout')).open('xb') as ro:
        sender=subprocess.Popen([*SSH,'fmd',send],stdout=subprocess.PIPE,stderr=se)
        receiver=subprocess.Popen([*SSH,'csd',receive],stdin=sender.stdout,stdout=ro,stderr=re)
        sender.stdout.close()
        rx=receiver.wait();tx=sender.wait()
    with (folder/(label+'-transport.json')).open('x') as f:json.dump(dict(sender_exit=tx,receiver_exit=rx),f)
    assert tx==rx==0,'Preserve failure; inspect before any recovery'
def main():
    assert sys.argv[1:]==['--run']
    folder=Path(__file__).parent/'final-relay12'
    assert not folder.exists(),'Existing relay must be inspected, never overwritten'
    # Exporter revalidates completed chapters, receipts and current frozen inputs.
    subprocess.run([*SSH,'fmd',command('/usr/bin/python3','-I','-B',F+'/qualification01/raster-quality03/export-fmd-final12.py','--export')],check=True,timeout=600)
    source=F+'/qualification01/raster-quality03/final-export12.json'
    raw=read('fmd',source);data=json.loads(raw);digest=hashlib.sha256(raw).hexdigest()
    assert data['host']=='fmd'
    keys=[r['chapter'] for r in data['records']]
    assert 0<len(keys)<15 and len(set(keys))==len(keys)
    assert all(re.fullmatch(r'[a-z0-9_]+',k) for k in keys)
    assert len(data['files'])==2+10*len(keys)
    folder.mkdir()
    with (folder/'expected-export.json').open('xb') as f:f.write(raw)
    launcher=C+'/qualification01/raster-quality12/launch-fmd-import12.py'
    relay(folder,'inventory',command('cat',source),command('/usr/bin/python3','-I','-B',launcher,'--export',digest))
    receipt(folder,'csd-final-export-receive12')
    for key in keys:
        relay(folder,key,command('/usr/bin/python3','-I','-B',F+'/qualification01/raster-quality03/stream-fmd-chapter12.py',key,digest),command('/usr/bin/python3','-I','-B',launcher,'--chapter',key))
        receipt(folder,'csd-fmd-final-import-'+key+'-12')
    with (folder/'completed.json').open('x') as f:json.dump(dict(chapters=keys,export_sha256=digest,scope='Guarded transfers and per-chapter import verification; native master replay and final AV review remain.'),f,indent=2)
    print('All assigned FMD chapters imported and checked; no master or publication launched.')
if __name__=='__main__':main()
