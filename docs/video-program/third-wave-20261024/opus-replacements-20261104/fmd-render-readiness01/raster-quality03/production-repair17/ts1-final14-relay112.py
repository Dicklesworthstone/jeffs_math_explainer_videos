"""Append exact TS1 continuation outputs after the existing gallery import."""
from pathlib import Path
import hashlib
import importlib.util
import json
import subprocess
import sys
import time

HERE = Path(__file__).resolve().parent
BASE = HERE/'ts1-final05-relay105.py'
spec = importlib.util.spec_from_file_location('base', BASE)
b = importlib.util.module_from_spec(spec)
spec.loader.exec_module(b)
SOURCE = b.U/'jobs/ts1-production19-final-14-continuation111'
JOB = 'trj-import-ts1-final14-112'
NAME = Path(__file__).name
MANIFEST = 'final14-export112.json'
INPUTS = b.TQ/'import-ts1-final14-112-inputs.json'
PRIOR = b.T/'jobs/trj-import-ts1-final05-105/resource-receipt.json'


def export():
    receipt = SOURCE/'resource-receipt.json'
    r = b.read(receipt)
    b.success(r)
    inputs = b.UQ/'final14-111-inputs.json'
    assert b.sha(inputs) == r['inputs_sha256']
    paths = {inputs, receipt, BASE, Path(__file__).resolve()}
    for row in b.read(inputs)['files']:
        p = Path(row['path'])
        assert b.sha(p) == row['sha256']
        paths.add(p)
    summary = b.read(SOURCE/'render-summary.json')
    assert summary['chapter'] == '14_continuation' and summary['frames'] == 11756
    for row in summary['files']:
        p = Path(row['path'])
        assert p.is_relative_to(SOURCE/'render') and b.sha(p) == row['sha256']
        paths.add(p)
    paths.update(p for p in SOURCE.rglob('*') if p.is_file())
    rows = []
    for p in sorted(paths):
        b.allowed(p)
        rows.append(dict(path=str(p),sha256=b.sha(p),bytes=p.stat().st_size,mode=p.stat().st_mode & 0o777))
    print(json.dumps(dict(source_job=str(SOURCE),files=rows),indent=2))


def prepare():
    b.success(b.read(PRIOR))
    assert b.U.is_symlink() and b.U.resolve() == b.DEST
    assert not (b.T/'jobs'/JOB).exists()
    paths = {BASE,Path(__file__).resolve(),PRIOR,b.TQ/MANIFEST,b.T/'guard_trj01.py'}
    for row in b.read(b.TQ/MANIFEST)['files']:
        p = Path(row['path']); b.allowed(p)
        if p.exists():
            assert b.sha(p) == row['sha256'] and p.stat().st_size == row['bytes']
            paths.add(p)
        else:
            assert p.is_relative_to(b.U)
    with INPUTS.open('x') as f:
        json.dump({'files':[dict(path=str(p),sha256=b.sha(p)) for p in sorted(paths)]},f,indent=2)


def receive():
    job = b.T/'jobs'/JOB
    receipt = b.read(job/'resource-receipt.json')
    assert receipt['admitted'] and 'exit' not in receipt
    assert receipt['policy']['stop_free_gib'] == 8 and receipt['policy']['phase_growth_gib'] == 4
    assert b.sha(INPUTS) == receipt['inputs_sha256']
    b.success(b.read(PRIOR))
    assert b.U.is_symlink() and b.U.resolve() == b.DEST
    m = b.read(b.TQ/MANIFEST)
    assert m['source_job'] == str(SOURCE)
    assert len(m['files']) == len({x['path'] for x in m['files']})
    for row in m['files']:
        p = Path(row['path']); b.allowed(p)
        exists = p.exists()
        if exists:
            assert b.sha(p) == row['sha256'] and p.stat().st_size == row['bytes']
        else:
            assert p.is_relative_to(b.U)
            p.parent.mkdir(mode=0o700,parents=True,exist_ok=True)
        f = None if exists else p.open('xb')
        h = hashlib.sha256(); remaining = row['bytes']
        try:
            while remaining:
                data = sys.stdin.buffer.read(min(remaining,1024*1024))
                assert data, 'truncated stream'
                h.update(data); remaining -= len(data)
                if f: f.write(data)
        finally:
            if f: f.close()
        assert h.hexdigest() == row['sha256'] == b.sha(p)
        if not exists: p.chmod(row['mode'])
    assert not sys.stdin.buffer.read(1)
    for row in m['files']: assert b.sha(Path(row['path'])) == row['sha256']
    with (job/'import-result.json').open('x') as f:
        json.dump(dict(files=m['files'],source_job=str(SOURCE),foreign_root=str(b.U),physical_root=str(b.DEST),scope='Exact append-only import; final master/replay/review remain.'),f,indent=2)


def relay():
    run = lambda args, **kw: subprocess.run(args,check=True,**kw)
    for host,path in [('ts1',SOURCE/'resource-receipt.json'),('trj',PRIOR)]:
        deadline = time.monotonic()+86400
        while True:
            p = subprocess.run(['ssh','-o','BatchMode=yes',host,'cat '+str(path)],capture_output=True)
            if p.returncode == 0:
                r = json.loads(p.stdout)
                if r.get('finished'):
                    b.success(r); break
            assert time.monotonic()<deadline, 'wait deadline exceeded'
            time.sleep(30)
    for host,q in [('ts1',b.UQ),('trj',b.TQ)]:
        for name in [NAME,BASE.name]:
            actual = run(['ssh',host,'sha256sum '+str(q/name)],capture_output=True).stdout.decode().split()[0]
            assert actual == b.sha(HERE/name)
    manifest = HERE/MANIFEST
    with manifest.open('xb') as f:
        run(['ssh','ts1','python3 -I -B '+str(b.UQ/NAME)+' export'],stdout=f)
    for host,q in [('ts1',b.UQ),('trj',b.TQ)]:
        run(['ssh',host,'test ! -e '+str(q/MANIFEST)])
        run(['scp',str(manifest),host+':'+str(q/MANIFEST)])
        actual = run(['ssh',host,'sha256sum '+str(q/MANIFEST)],capture_output=True).stdout.decode().split()[0]
        assert actual == b.sha(manifest)
    run(['ssh','trj','python3 -I -B '+str(b.TQ/NAME)+' prepare'])
    with (HERE/'final14-relay112.stderr').open('xb') as err,(HERE/'final14-relay112.stdout').open('xb') as out:
        sender = subprocess.Popen(['ssh','ts1','python3 -I -B '+str(b.UQ/NAME)+' send'],stdout=subprocess.PIPE,stderr=err)
        receiver = subprocess.Popen(['ssh','trj','python3 -I -B '+str(b.T/'guard_trj01.py')+' --phase native --job '+JOB+' --inputs '+str(INPUTS)+' -- python3 -I -B '+str(b.TQ/NAME)+' receive'],stdin=sender.stdout,stdout=out,stderr=err)
        sender.stdout.close(); rx=receiver.wait(); tx=sender.wait()
    with (HERE/'final14-relay112-exit.json').open('x') as f: json.dump(dict(sender_exit=tx,receiver_exit=rx),f)
    assert tx == rx == 0
    r=json.loads(run(['ssh','trj','cat '+str(b.T/'jobs'/JOB/'resource-receipt.json')],capture_output=True).stdout)
    b.success(r)
    with (HERE/'final14-import112-receipt.json').open('x') as f: json.dump(r,f,indent=2)


if __name__ == '__main__':
    b.MANIFEST=MANIFEST
    {'export':export,'prepare':prepare,'receive':receive,'send':b.send,'relay':relay}[sys.argv[1]]()
