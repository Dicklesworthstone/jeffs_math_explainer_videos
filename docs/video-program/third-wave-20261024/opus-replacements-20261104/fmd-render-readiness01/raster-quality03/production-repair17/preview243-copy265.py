from pathlib import Path
import hashlib
import json
import subprocess

Q = Path(__file__).resolve().parent
inventory = json.loads((Q/'preview243-copy-inventory265.json').read_text())
dest = Q/'preview243-review265'
assert not dest.exists()
subprocess.run(['scp', '-rq', 'csd:'+inventory['source'], str(dest)], check=True, timeout=900)
assert {str(p.relative_to(dest)) for p in dest.rglob('*') if p.is_file()} == {r['path'] for r in inventory['files']}
for row in inventory['files']:
    p = dest/row['path']
    assert p.stat().st_size == row['bytes']
    with p.open('rb') as f:
        assert hashlib.file_digest(f, 'sha256').hexdigest() == row['sha256'], str(p)
print('Preview243 files copied with exact hashes; picture review still pending')
