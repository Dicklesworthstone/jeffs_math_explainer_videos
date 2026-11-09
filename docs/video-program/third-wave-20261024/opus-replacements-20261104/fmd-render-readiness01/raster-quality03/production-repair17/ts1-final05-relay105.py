"""Exact final05 return from TS1, with guarded import on the assembly host."""
from pathlib import Path
import hashlib
import json
import subprocess
import sys
import time

U = Path('/data/projects/jmev-ts1-render-worker-20261107')
T = Path('/data/projects/jmev-render-worker-20261107')
D = Path('/data/projects/jmev-stroke-probe-20261106')
UQ = U / 'qualification01'
TQ = T / 'qualification01/production19'
SOURCE = U / 'jobs/ts1-production19-final-05-gallery103'
JOB = 'trj-import-ts1-final05-105'
NAME = Path(__file__).name
MANIFEST = 'final05-export105.json'
INPUTS = TQ / 'import-ts1-final05-105-inputs.json'
DEST = T / 'work/imported-ts1-root105'


def sha(p):
    with p.open('rb') as f:
        return hashlib.file_digest(f, 'sha256').hexdigest()


def read(p):
    return json.loads(p.read_text())


def success(r):
    assert r.get('finished') and r.get('exit') == 0
    assert r['inputs_sha256'] == r['final_inputs_sha256']
    assert not any(r.get(k) for k in ('error', 'resource_stop', 'stopped_for_resource_limit'))


def allowed(p):
    assert p.is_absolute() and '..' not in p.parts
    assert any(p.is_relative_to(root) for root in (U, T, D)), str(p)


def export():
    receipt = SOURCE / 'resource-receipt.json'
    r = read(receipt)
    success(r)
    inputs = UQ / 'final05-103-inputs.json'
    assert sha(inputs) == r['inputs_sha256']
    paths = {inputs, receipt, Path(__file__).resolve()}
    for row in read(inputs)['files']:
        p = Path(row['path'])
        assert sha(p) == row['sha256']
        paths.add(p)
    summary = read(SOURCE / 'render-summary.json')
    assert summary['chapter'] == '05_gallery' and summary['frames'] == 9879
    for row in summary['files']:
        p = Path(row['path'])
        assert p.is_relative_to(SOURCE / 'render') and sha(p) == row['sha256']
        paths.add(p)
    paths.update(p for p in SOURCE.rglob('*') if p.is_file())
    rows = []
    for p in sorted(paths):
        allowed(p)
        rows.append(dict(path=str(p), sha256=sha(p), bytes=p.stat().st_size,
                         mode=p.stat().st_mode & 0o777))
    assert sha(inputs) == r['inputs_sha256']
    print(json.dumps(dict(source_job=str(SOURCE), receipt_sha256=sha(receipt),
                          files=rows, bytes=sum(x['bytes'] for x in rows)), indent=2))


def send():
    for row in read(UQ / MANIFEST)['files']:
        p = Path(row['path'])
        allowed(p)
        assert p.stat().st_size == row['bytes'] and sha(p) == row['sha256']
        h = hashlib.sha256()
        with p.open('rb') as f:
            while data := f.read(1024 * 1024):
                h.update(data)
                sys.stdout.buffer.write(data)
        assert h.hexdigest() == row['sha256'] == sha(p)
    sys.stdout.buffer.flush()


def prepare():
    assert not (T / 'jobs' / JOB).exists()
    assert not U.exists() and not U.is_symlink() and not DEST.exists()
    paths = {TQ / MANIFEST, TQ / NAME, T / 'guard_trj01.py'}
    for row in read(TQ / MANIFEST)['files']:
        p = Path(row['path'])
        allowed(p)
        if not p.is_relative_to(U):
            assert sha(p) == row['sha256'] and p.stat().st_size == row['bytes']
            paths.add(p)
    with INPUTS.open('x') as f:
        json.dump({'files': [dict(path=str(p), sha256=sha(p)) for p in sorted(paths)]}, f, indent=2)


def receive():
    job = T / 'jobs' / JOB
    r = read(job / 'resource-receipt.json')
    assert r['admitted'] and 'exit' not in r
    assert r['policy']['stop_free_gib'] == 8 and r['policy']['phase_growth_gib'] == 4
    assert not U.exists() and not U.is_symlink() and not DEST.exists()
    m = read(TQ / MANIFEST)
    assert m['source_job'] == str(SOURCE)
    assert len(m['files']) == len({x['path'] for x in m['files']})
    for row in m['files']:
        p = Path(row['path'])
        allowed(p)
        if not p.is_relative_to(U):
            assert sha(p) == row['sha256'] and p.stat().st_size == row['bytes']
    DEST.mkdir(mode=0o700)
    U.symlink_to(DEST, target_is_directory=True)
    rows = []
    for row in m['files']:
        p = Path(row['path'])
        existing = p.exists()
        if existing:
            assert sha(p) == row['sha256']
        else:
            assert p.is_relative_to(U)
            p.parent.mkdir(mode=0o700, parents=True, exist_ok=True)
        f = None if existing else p.open('xb')
        remaining = row['bytes']
        h = hashlib.sha256()
        try:
            while remaining:
                data = sys.stdin.buffer.read(min(remaining, 1024 * 1024))
                assert data, 'Truncated stream'
                remaining -= len(data)
                h.update(data)
                if f:
                    f.write(data)
        finally:
            if f:
                f.close()
        assert h.hexdigest() == row['sha256'] == sha(p)
        if not existing:
            p.chmod(row['mode'])
        rows.append(dict(row, existing_identical=existing))
    assert not sys.stdin.buffer.read(1), 'Trailing bytes'
    for row in rows:
        assert sha(Path(row['path'])) == row['sha256']
    with (job / 'import-result.json').open('x') as f:
        json.dump(dict(files=rows, source_job=str(SOURCE), foreign_root=str(U),
                       physical_root=str(DEST), scope='Exact import only; master replay and final review remain.'), f, indent=2)


def relay():
    q = Path(__file__).resolve().parent
    for _ in range(1440):
        result = subprocess.run(['ssh', '-o', 'BatchMode=yes', 'ts1',
                                 'cat ' + str(SOURCE / 'resource-receipt.json')], capture_output=True)
        if result.returncode == 0:
            r = json.loads(result.stdout)
            if r.get('finished'):
                success(r)
                break
        time.sleep(60)
    else:
        raise RuntimeError('Final05 wait deadline; no import started')
    run = lambda argv, **kw: subprocess.run(argv, check=True, **kw)
    for host, remote_q in [('ts1', UQ), ('trj', TQ)]:
        actual = run(['ssh', host, 'sha256sum ' + str(remote_q / NAME)], capture_output=True).stdout.decode().split()[0]
        assert actual == sha(Path(__file__).resolve())
    manifest = q / MANIFEST
    with manifest.open('xb') as f:
        run(['ssh', 'ts1', 'python3 -I -B ' + str(UQ / NAME) + ' export'], stdout=f)
    assert read(manifest)['files']
    for host, remote_q in [('ts1', UQ), ('trj', TQ)]:
        run(['ssh', host, 'test ! -e ' + str(remote_q / MANIFEST)])
        run(['scp', str(manifest), host + ':' + str(remote_q / MANIFEST)])
        actual = run(['ssh', host, 'sha256sum ' + str(remote_q / MANIFEST)], capture_output=True).stdout.decode().split()[0]
        assert actual == sha(manifest)
    run(['ssh', 'trj', 'python3 -I -B ' + str(TQ / NAME) + ' prepare'])
    with (q / 'final05-relay105.stderr').open('xb') as err, (q / 'final05-relay105.stdout').open('xb') as out:
        sender = subprocess.Popen(['ssh', 'ts1', 'python3 -I -B ' + str(UQ / NAME) + ' send'], stdout=subprocess.PIPE, stderr=err)
        receiver = subprocess.Popen(['ssh', 'trj', 'python3 -I -B ' + str(T / 'guard_trj01.py') +
                                     ' --phase native --job ' + JOB + ' --inputs ' + str(INPUTS) +
                                     ' -- python3 -I -B ' + str(TQ / NAME) + ' receive'],
                                    stdin=sender.stdout, stdout=out, stderr=err)
        sender.stdout.close()
        b = receiver.wait()
        a = sender.wait()
    with (q / 'final05-relay105-exit.json').open('x') as f:
        json.dump(dict(sender_exit=a, receiver_exit=b), f)
    assert a == b == 0, (a, b)
    r = json.loads(run(['ssh', 'trj', 'cat ' + str(T / 'jobs' / JOB / 'resource-receipt.json')], capture_output=True).stdout)
    success(r)
    with (q / 'final05-import105-receipt.json').open('x') as f:
        json.dump(r, f, indent=2)


if __name__ == '__main__':
    {'export': export, 'send': send, 'prepare': prepare, 'receive': receive, 'relay': relay}[sys.argv[1]]()
