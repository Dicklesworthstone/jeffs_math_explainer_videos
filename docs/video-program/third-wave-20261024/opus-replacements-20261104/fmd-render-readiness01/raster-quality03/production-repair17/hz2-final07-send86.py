from pathlib import Path
import json,hashlib,sys
Q=Path('/data/projects/jmev-hz2-render-worker-20261107/qualification01/production19')
m=json.loads((Q/'final07-export86.json').read_text())
for x in m['files']:
 p=Path(x['path']);assert p.stat().st_size==x['bytes'];h=hashlib.sha256()
 with p.open('rb') as f:
  while b:=f.read(1024*1024):h.update(b);sys.stdout.buffer.write(b)
 assert h.hexdigest()==x['sha256'],str(p)
 with p.open('rb') as f:assert hashlib.file_digest(f,'sha256').hexdigest()==x['sha256']
sys.stdout.buffer.flush()
