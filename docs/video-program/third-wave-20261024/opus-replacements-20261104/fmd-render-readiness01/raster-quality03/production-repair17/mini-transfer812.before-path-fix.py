import json,hashlib,subprocess
from pathlib import Path
Q=Path(__file__).resolve().parent
R=Q.parents[5]
J=R/'narration_lab/third-wave-20261024/root/five-film-release-20261027/complex-mini-transfer812'
REMOTE=Path('/Users/jemanuel/scratch/jmev-complex-native-20261109')
d=json.load(open(Q/'mini772-terminal812.json'))
assert d['finished'] and d['exit']==-15 and d['stopped_for_resource_limit'] is True
assert d['inputs_sha256']==d['final_inputs_sha256']
m=Q/'mini772-inputs812.json'
assert hashlib.sha256(m.read_bytes()).hexdigest()==d['inputs_sha256']
rows=[r for r in json.load(open(m))['files'] if Path(r['path']).is_relative_to(REMOTE)]
out=J/'tree';out.mkdir()
subprocess.run(['/usr/bin/rsync','-a','--files-from='+str(Q/'mini-transfer812-files.txt'),'-e','ssh -o BatchMode=yes -o ConnectTimeout=8','mac-mini-max:'+str(REMOTE)+'/',str(out)+'/'],check=True,timeout=1800)
for r in rows:
 p=out/Path(r['path']).relative_to(REMOTE)
 assert not p.is_symlink()
 with p.open('rb') as f:assert hashlib.file_digest(f,'sha256').hexdigest()==r['sha256'],p
with (J/'transfer.json').open('x') as f:json.dump({'files_verified':len(rows),'bytes':sum((out/Path(r['path']).relative_to(REMOTE)).stat().st_size for r in rows),'source_attempt_failed_resource_stop':True,'render_launched':False},f,indent=2)
