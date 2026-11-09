from pathlib import Path
import sys,json,hashlib,os
R=Path('/data/projects/jmev-ts1-render-worker-20261107');Q=R/'qualification01';J=R/'jobs/ts1-runtime-transfer100'
r=json.loads((J/'resource-receipt.json').read_text());assert r['admitted'] and 'exit' not in r
assert r['policy']['stop_free_gib']==8 and r['policy']['phase_growth_gib']==4
manifest=Q/'runtime100-inputs.json';assert hashlib.sha256(manifest.read_bytes()).hexdigest()==r['inputs_sha256']
for x in json.loads(manifest.read_text())['files']:
 with Path(x['path']).open('rb') as f: assert hashlib.file_digest(f,'sha256').hexdigest()==x['sha256']
m=json.loads((Q/'css-runtime-manifest75.json').read_text());dest=R/'work/frozen-runtime100';dest.mkdir(mode=0o700)
rows=[]
for x in m['files']:
 source=Path(x['path']);assert source.is_absolute() and '..' not in source.parts
 assert str(source).startswith(('/data/projects/jmev-render-worker-20261107/','/data/projects/jmev-stroke-probe-20261106/'))
 p=dest/str(source).lstrip('/');p.parent.mkdir(mode=0o700,parents=True,exist_ok=True);remaining=x['bytes'];h=hashlib.sha256()
 with p.open('xb') as f:
  while remaining:
   b=sys.stdin.buffer.read(min(remaining,1024*1024));assert b,'Truncated transfer';f.write(b);h.update(b);remaining-=len(b)
 assert h.hexdigest()==x['sha256'],str(p)
 os.chmod(p,x['mode'])
 with p.open('rb') as f:assert hashlib.file_digest(f,'sha256').hexdigest()==x['sha256']
 rows.append(dict(source=str(source),path=str(p),sha256=x['sha256'],bytes=x['bytes']))
assert not sys.stdin.buffer.read(1),'Unexpected trailing data'
with (J/'transfer-result.json').open('x') as f:json.dump(dict(files=rows,bytes=sum(x['bytes'] for x in rows),all_hashes_match=True,scope='Frozen copy only; native runtime and render not qualified'),f,indent=2)
print('Transferred and verified',len(rows),'files')
