from pathlib import Path
import json,hashlib,subprocess,shlex
R=Path('/data/projects/jmev-render-worker-20261107');D=Path('/data/projects/jmev-stroke-probe-20261106');Q=R/'qualification01/production19';J=R/'jobs/trj-production19-transfer01';M=Q/'trj-production19-transfer-manifest01.json'
def sha(p):
 with p.open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest()
a=json.loads((J/'resource-receipt.json').read_text());assert a['admitted'] and 'exit' not in a
assert D.is_symlink() and D.resolve()==R/'work/imported-csd-root01'
d=json.loads(M.read_text());rows=d['files'];assert len(rows)==536
for row in rows:
 p=Path(row['relative']);assert not p.is_absolute() and '..' not in p.parts
 dest=D/p
 if dest.exists():assert dest.is_file() and sha(dest)==row['sha256'],str(dest)
remote="from pathlib import Path;import hashlib,json;R=Path('/data/projects/jmev-stroke-probe-20261106');M=R/'qualification01/production-repair17/trj-production19-transfer-manifest01.json';assert hashlib.sha256(M.read_bytes()).hexdigest()=="+repr(sha(M))+";d=json.loads(M.read_text());assert all(hashlib.sha256((R/x['relative']).read_bytes()).hexdigest()==x['sha256'] for x in d['files'])"
subprocess.run(['ssh','-o','BatchMode=yes','csd',shlex.join(['python3','-c',remote])],check=True,timeout=300)
listing=J/'files.txt';listing.write_text(''.join(x['relative']+'\n' for x in rows))
subprocess.run(['rsync','-a','--ignore-existing','--files-from='+str(listing),'csd:'+str(D)+'/',str(D)+'/'],check=True,timeout=3600)
for row in rows:
 p=D/row['relative'];assert p.stat().st_size==row['bytes'] and sha(p)==row['sha256'],str(p)
(J/'transfer-result.json').write_text(json.dumps(dict(files=len(rows),bytes=sum(x['bytes'] for x in rows),manifest_sha256=sha(M),all_hashes_match=True,scope='Runtime assets copied exactly; no new native render, portability, picture or release qualification'),indent=2))
print('Transferred and verified536runtime assets')
