import json,hashlib,tarfile,subprocess,os
from pathlib import Path
R=Path('/data/projects/jmev-stroke-probe-20261106');Q=R/'qualification01/raster-quality12';J=R/'jobs/csd-raster-stage12';A=Path('/data/projects/jmev-fmd-render-qualification-20261106-1111')
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
r=json.loads((J/'resource-receipt.json').read_text());assert r['admitted'] and 'exit' not in r
binding=json.loads((Q/'csd-raster-stage12-binding.json').read_text());archive=Q/'csd-raster-stage12.tar';assert sha(archive)==binding['archive_sha256']
assert not A.exists() and not A.is_symlink()
rows={x['name']:x for x in binding['files']}
with tarfile.open(archive) as tar:
 members=tar.getmembers();assert len(members)==len(rows) and {m.name for m in members}==set(rows)
 for m in members:
  p=Path(m.name);assert not p.is_absolute() and '..' not in p.parts and m.isfile();assert not (R/p).exists(),p
 for m in members:
  dst=R/m.name;dst.parent.mkdir(parents=True,exist_ok=True)
  with dst.open('xb') as f:f.write(tar.extractfile(m).read())
  os.chmod(dst,m.mode);assert sha(dst)==rows[m.name]['sha256']
# Retain compiled absolute source paths without modifying any existing location.
A.symlink_to(R,target_is_directory=True);assert A.resolve(strict=True)==R
binary=R/'work/bin/production09i';q=json.loads((R/'jobs/fmd-raster-quality-build05/qualification.json').read_text());assert sha(binary)==q['binary_sha256']
r=subprocess.run(['ldd',str(binary)],capture_output=True,text=True,check=True);assert 'not found' not in r.stdout
with (J/'ldd.txt').open('x') as f:f.write(r.stdout)
result=subprocess.run([str(binary),'chapters'],cwd=R/'work/probe-cwd',capture_output=True,text=True,timeout=60)
assert result.returncode==0,result.stderr
assert result.stdout.splitlines()==q['chapters'] and len(q['chapters'])==15
with (J/'stage-summary.json').open('x') as f:json.dump({'files':binding['files'],'binary_sha256':sha(binary),'chapters':q['chapters'],'compiled_root_alias':str(A),'alias_target':str(R),'scope':'Exact transfer, ELF dependencies and native registry only. Encoder/picture and narration path qualification remain.'},f,indent=2)
print('Transferred68 exact files; compatible native binary lists15 chapters. Actual render qualification pending.')
