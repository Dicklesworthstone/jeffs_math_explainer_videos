from pathlib import Path
import hashlib,json,sys
R=Path('/data/projects/jmev-render-worker-20261107');Q=R/'qualification01/production19';J=R/'jobs/trj-production19-join-extract83'
def sha(p):
 with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest()
r=json.loads((J/'resource-receipt.json').read_text());assert r.get('finished') and r.get('exit')==0 and r['inputs_sha256']==r['final_inputs_sha256']==sha(Q/'join-extract83-inputs.json')
assert not any(r.get(k) for k in ('error','resource_stop','stopped_for_resource_limit'))
paths={J/'resource-receipt.json',J/'joins.json',Q/'join-extract83-inputs.json',R/'jobs/trj-production19-master01/master-summary.json',R/'jobs/trj-production19-verify01/verification.json'}
joins=json.loads((J/'joins.json').read_text());assert len(joins['joins'])==14
for row in joins['joins']:
 for item in row['files']:
  p=Path(item['path']);assert p.is_relative_to(J) and sha(p)==item['sha256'];paths.add(p)
files=[dict(path=str(p),sha256=sha(p),bytes=p.stat().st_size,name=p.name) for p in sorted(paths)]
assert len(files)==len({x['name'] for x in files})==33
if len(sys.argv)==1:print(json.dumps(dict(files=files,master_sha256=joins['master_sha256']),indent=2))
else:
 assert sys.argv[1]=='--stream'
 expected=json.loads(Path(sys.argv[2]).read_text());assert expected['files']==files
 for row in files:
  h=hashlib.sha256()
  with Path(row['path']).open('rb') as f:
   while b:=f.read(1048576):h.update(b);sys.stdout.buffer.write(b)
  assert h.hexdigest()==row['sha256']
 sys.stdout.buffer.flush()
