"""Return one terminal repaired chapter without overwriting retained runtime files."""
from pathlib import Path
import hashlib
import json
import subprocess
import sys
import time

H = Path('/data/projects/jmev-stroke-probe-20261106')
T = Path('/data/projects/jmev-render-worker-20261107')
D = Path('/data/projects/jmev-stroke-probe-20261106')
HQ = H/'qualification01/production-repair17'
TQ = T/'qualification01/production19'
SOURCE = H/'jobs/csd-repaired-final-08-loop-sums170'
DEST = T/'jobs/trj-import-csd-final08-171'
PRIOR = T/'jobs/trj-import-hz2-final09-162/resource-receipt.json'
NAME = 'repaired08-return171.py'
EXPORT = 'repaired08-export171.json'
INPUTS = TQ/'repaired08-import-inputs171.json'

def read(p):
    return json.loads(p.read_text())

def sha(p):
    with p.open('rb') as f:
        return hashlib.file_digest(f, 'sha256').hexdigest()

def success(r):
    assert r.get('finished') and r.get('exit') == 0
    assert r['inputs_sha256'] == r['final_inputs_sha256']
    assert not any(r.get(k) for k in ('error', 'resource_stop', 'stopped_for_resource_limit'))
    return r

def allowed(p):
    assert p.is_absolute() and '..' not in p.parts
    assert p.is_relative_to(H) or p.is_relative_to(D)

def aliases():
    assert D.is_symlink() and D.resolve() == T/'work/imported-csd-root01'

mode = sys.argv[1]
if mode == 'export':
    receipt = success(read(SOURCE/'resource-receipt.json'))
    manifest = HQ/'final08-inputs170.json'
    assert sha(manifest) == receipt['inputs_sha256']
    paths = {manifest, Path(__file__).resolve()}
    for row in read(manifest)['files']:
        p = Path(row['path'])
        assert sha(p) == row['sha256']
        paths.add(p)
    summary = read(SOURCE/'render-summary.json')
    assert summary['chapter'] == '08_loop_sums' and summary['frames'] == 14959
    for row in summary['files']:
        p = Path(row['path'])
        assert p.is_relative_to(SOURCE/'render') and sha(p) == row['sha256']
    paths.update(p for p in SOURCE.rglob('*') if p.is_file())
    rows = []
    for p in sorted(paths):
        allowed(p)
        rows.append(dict(path=str(p), sha256=sha(p), bytes=p.stat().st_size, mode=p.stat().st_mode & 0o777))
    print(json.dumps(dict(files=rows, source_job=str(SOURCE), bytes=sum(x['bytes'] for x in rows)), indent=2))
elif mode == 'prepare':
    aliases()
    assert not DEST.exists()
    success(read(PRIOR))
    paths = {PRIOR, TQ/EXPORT, Path(__file__).resolve(), T/'guard_trj01.py'}
    rows = read(TQ/EXPORT)['files']
    assert len(rows) == len({x['path'] for x in rows})
    for row in rows:
        p = Path(row['path']); allowed(p)
        assert p.resolve().is_relative_to(T/'work')
        if p.exists():
            assert sha(p) == row['sha256'], str(p)
            paths.add(p)
    with INPUTS.open('x') as f:
        json.dump(dict(files=[dict(path=str(p), sha256=sha(p)) for p in sorted(paths)]), f, indent=2)
elif mode == 'send':
    for row in read(HQ/EXPORT)['files']:
        p = Path(row['path']); allowed(p)
        assert p.stat().st_size == row['bytes'] and sha(p) == row['sha256']
        with p.open('rb') as f:
            while b := f.read(1024 * 1024): sys.stdout.buffer.write(b)
        assert sha(p) == row['sha256']
    sys.stdout.buffer.flush()
elif mode == 'receive':
    aliases(); success(read(PRIOR))
    receipt = read(DEST/'resource-receipt.json')
    assert receipt['admitted'] and 'exit' not in receipt
    assert receipt['policy']['stop_free_gib'] == 8 and receipt['policy']['phase_growth_gib'] == 4
    assert sha(INPUTS) == receipt['inputs_sha256']
    rows = read(TQ/EXPORT)['files']
    for row in rows:
        p = Path(row['path']); allowed(p)
        assert p.resolve().is_relative_to(T/'work')
        if p.exists(): assert sha(p) == row['sha256'], str(p)
    for row in rows:
        p = Path(row['path']); existed = p.exists(); remaining = row['bytes']; h = hashlib.sha256()
        if not existed: p.parent.mkdir(parents=True, exist_ok=True, mode=0o700)
        f = None if existed else p.open('xb')
        try:
            while remaining:
                b = sys.stdin.buffer.read(min(remaining, 1024 * 1024)); assert b, 'Truncated transfer'
                h.update(b); remaining -= len(b)
                if f: f.write(b)
        finally:
            if f: f.close()
        assert h.hexdigest() == row['sha256'] and sha(p) == row['sha256']
        if not existed: p.chmod(row['mode'])
    assert not sys.stdin.buffer.read(1), 'Trailing bytes'
    with (DEST/'import-result.json').open('x') as f:
        json.dump(dict(files=rows, source_job=str(SOURCE), scope='Exact copied files only; master replay and finalAV remain open.'), f, indent=2)
elif mode == 'relay':
    local = Path(__file__).resolve().parent
    for host, p in [('csd', SOURCE/'resource-receipt.json'), ('trj', PRIOR)]:
        for _ in range(720):
            result = subprocess.run(['ssh', host, 'cat '+str(p)], capture_output=True)
            if result.returncode == 0:
                r = json.loads(result.stdout)
                if r.get('finished'): success(r); break
            time.sleep(60)
        else: raise RuntimeError('Prerequisite wait expired; no import launched')
    p = local/EXPORT
    with p.open('xb') as f:
        subprocess.run(['ssh', 'csd', 'python3 -I -B '+str(HQ/NAME)+' export'], stdout=f, check=True)
    for host, q in [('csd', HQ), ('trj', TQ)]:
        subprocess.run(['ssh', host, 'test ! -e '+str(q/EXPORT)], check=True)
        subprocess.run(['scp', str(p), host+':'+str(q/EXPORT)], check=True)
        digest = subprocess.check_output(['ssh', host, 'sha256sum '+str(q/EXPORT)]).decode().split()[0]
        assert digest == sha(p)
    subprocess.run(['ssh', 'trj', 'python3 -I -B '+str(TQ/NAME)+' prepare'], check=True)
    with (local/'repaired08-relay171.stderr').open('xb') as err, (local/'repaired08-relay171.stdout').open('xb') as out:
        sender = subprocess.Popen(['ssh', 'csd', 'python3 -I -B '+str(HQ/NAME)+' send'], stdout=subprocess.PIPE, stderr=err)
        receiver = subprocess.Popen(['ssh', 'trj', 'python3 -I -B '+str(T/'guard_trj01.py')+' --phase native --job '+DEST.name+' --inputs '+str(INPUTS)+' -- python3 -I -B '+str(TQ/NAME)+' receive'], stdin=sender.stdout, stdout=out, stderr=err)
        sender.stdout.close(); b = receiver.wait(); a = sender.wait()
    with (local/'repaired08-relay171-exit.json').open('x') as f: json.dump(dict(sender_exit=a, receiver_exit=b), f)
    assert a == b == 0
    r = success(json.loads(subprocess.check_output(['ssh', 'trj', 'cat '+str(DEST/'resource-receipt.json')])))
    with (local/'repaired08-import171-receipt.json').open('x') as f: json.dump(r, f, indent=2)
else:
    raise ValueError(mode)
