"""Copy a completed preview under the existing local resource guard; never accept pictures."""
from pathlib import Path
import hashlib
import json
import subprocess

Q = Path(__file__).resolve().parent
SOURCE = '/data/projects/jmev-stroke-probe-20261106/jobs/csd-review-repair-preview313'
DEST = Q / 'preview313-review333'
INVENTORY = Q / 'preview313-copy-inventory333.json'

def sha(path):
    with path.open('rb') as f:
        return hashlib.file_digest(f, 'sha256').hexdigest()

def main():
    inventory = json.loads(INVENTORY.read_text())
    assert inventory['source'] == SOURCE
    assert inventory['terminal_success_verified'] is True
    rows = inventory['files']
    names = [r['path'] for r in rows]
    assert len(names) == len(set(names))
    assert 'resource-receipt.json' in names and 'summary.json' in names
    for name in names:
        p = Path(name)
        assert not p.is_absolute() and '..' not in p.parts
    assert sum(r['bytes'] for r in rows) < 4 * 1024**3
    assert not DEST.exists(), 'Preserve existing attempt; choose a new version'
    subprocess.run(['scp', '-rq', 'csd:' + SOURCE, str(DEST)], check=True, timeout=900)
    files = {str(p.relative_to(DEST)) for p in DEST.rglob('*') if p.is_file()}
    assert files == set(names), 'Remote output changed or transfer incomplete'
    for row in rows:
        p = DEST / row['path']
        assert not p.is_symlink()
        assert p.stat().st_size == row['bytes'] and sha(p) == row['sha256'], str(p)
    receipt = json.loads((DEST / 'resource-receipt.json').read_text())
    assert receipt.get('exit') == 0 and receipt.get('finished')
    assert receipt['inputs_sha256'] == receipt['final_inputs_sha256']
    assert not receipt.get('stop_reason')
    print('Completed preview copied with exact hashes. Actual picture and motion review remain open.')

if __name__ == '__main__':
    main()
