"""Exact new-path repair runtime transfer. No overwrite or release acceptance."""
import hashlib
import json
import sys
from pathlib import Path

D = Path('/data/projects/jmev-stroke-probe-20261106')
H = Path('/data/projects/jmev-hz2-render-worker-20261107')
MODE = sys.argv[1]
MANIFEST = Path(sys.argv[2])
rows = json.loads(MANIFEST.read_text())['files']
assert len(rows) == 218 and len({x['path'] for x in rows}) == 218
assert sum(x['bytes'] for x in rows) == 125622921

def sha(p):
    with p.open('rb') as f:
        return hashlib.file_digest(f, 'sha256').hexdigest()

if MODE == 'send':
    for row in rows:
        p = Path(row['path'])
        assert p.is_relative_to(D) and p.stat().st_size == row['bytes']
        assert sha(p) == row['sha256']
    for row in rows:
        p = Path(row['path'])
        with p.open('rb') as f:
            while b := f.read(1024 * 1024):
                sys.stdout.buffer.write(b)
        assert sha(p) == row['sha256']
    sys.stdout.buffer.flush()
elif MODE == 'receive':
    job = H/'jobs/hz2-repair-runtime-transfer156'
    receipt = json.loads((job/'resource-receipt.json').read_text())
    assert receipt['admitted'] and 'exit' not in receipt
    assert receipt['policy']['stop_free_gib'] == 8
    assert receipt['policy']['phase_growth_gib'] == 4
    inputs = H/'qualification01/production19/repair-runtime-inputs156.json'
    assert sha(inputs) == receipt['inputs_sha256']
    bound = json.loads(inputs.read_text())['files']
    for x in bound:
        assert sha(Path(x['path'])) == x['sha256']
    assert D.is_symlink()
    assert D.resolve() == H/'work/frozen-runtime47/data/projects/jmev-stroke-probe-20261106'
    for row in rows:
        p = Path(row['path'])
        assert p.is_relative_to(D) and '..' not in p.parts
        assert p.resolve().is_relative_to(H/'work/frozen-runtime47')
        assert not p.exists() and not p.is_symlink()
    for row in rows:
        p = Path(row['path'])
        p.parent.mkdir(parents=True, exist_ok=True, mode=0o700)
        remaining = row['bytes']
        with p.open('xb') as f:
            while remaining:
                b = sys.stdin.buffer.read(min(remaining, 1024 * 1024))
                assert b, 'Truncated transfer'
                f.write(b)
                remaining -= len(b)
        assert sha(p) == row['sha256']
        if p == D/'work/bin/two-label68':
            p.chmod(0o700)
    assert not sys.stdin.buffer.read(1), 'Trailing bytes'
    for row in rows:
        assert sha(Path(row['path'])) == row['sha256']
    with (job/'transfer-result.json').open('x') as f:
        json.dump({'files': rows, 'all_hashes_match': True,
                   'scope': '218 new files copied; no native delivery, visual, or release acceptance.'}, f, indent=2)
    print('218 new files transferred with exact hashes; old runtime untouched')
else:
    raise ValueError(MODE)
