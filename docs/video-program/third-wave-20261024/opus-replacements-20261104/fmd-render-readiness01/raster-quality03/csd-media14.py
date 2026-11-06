import json,hashlib,tarfile,subprocess
from pathlib import Path
R=Path('/data/projects/jmev-stroke-probe-20261106');Q=R/'qualification01/raster-quality12';J=R/'jobs/csd-raster-media14';T=R/'work/raster-quality-fmd04/workspace';N=R/'jobs/fmd-raster-rebind12/narration'
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def read(p):return json.loads(p.read_text())
a=read(J/'resource-receipt.json');assert a['admitted'] and 'exit' not in a
assert Path('/data/projects/jmev-fmd-render-qualification-20261106-1111').resolve(strict=True)==R
rows=read(Q/'fmd-media-binding14.json')['files']
for row in rows:
 name=row['path']
 if name.startswith('production-inputs01/production-repo07/'):target=R/'work/production-repo07'/name.removeprefix('production-inputs01/production-repo07/')
 elif name.startswith('production-inputs01/scored/'):target=R/'jobs/stage07-score01/scored'/name.removeprefix('production-inputs01/scored/')
 elif name.endswith('.wav'):target=R/'jobs/stage08-picture-rebind01/narration'/Path(name).name
 else:continue
 assert sha(target)==row['sha256'],target
base=R/'production-inputs01';base.mkdir(exist_ok=True)
for name,target in [('production-repo07',R/'work/production-repo07'),('scored',R/'jobs/stage07-score01/scored')]:
 dst=base/name;assert not dst.exists() and not dst.is_symlink();dst.symlink_to(target,target_is_directory=True)
assert not N.exists();N.mkdir(parents=True)
with tarfile.open(Q/'csd-narration-meta14.tar') as tar:
 for m in tar.getmembers():
  assert m.isfile() and Path(m.name).name==m.name and not m.name.endswith('.wav')
  with (N/m.name).open('xb') as f:f.write(tar.extractfile(m).read())
for row in rows:
 if row['path'].startswith('jobs/fmd-raster-rebind12/narration/') and row['path'].endswith('.wav'):
  dst=N/Path(row['path']).name;dst.symlink_to(R/'jobs/stage08-picture-rebind01/narration'/dst.name)
for row in rows:assert sha(R/row['path'])==row['sha256'],row['path']
C=T/'explainer_corpus/complex_analysis_visual';bin=R/'work/bin/production09i';chapters=subprocess.check_output([str(bin),'chapters'],text=True).splitlines();assert len(chapters)==15
commands=[('coherent',['/usr/bin/python3','-I','-B',str(T/'tools/coherent_narration.py'),'verify',str(N),'--script',str(C/'script.tsv'),'--units',str(N/'external-units.json'),'--require-directions'])]
args=['/usr/bin/python3','-I','-B',str(T/'tools/narration_delivery.py'),'--topic',str(C),'--narration',str(N),'--consumer','render','--gain-db','0','--score',str(base/'scored/stems')]
for c in chapters:args+=['--chapter-key',c]
commands.append(('delivery',args))
for name,cmd in commands:
 d=subprocess.run(cmd,cwd=base/'production-repo07',capture_output=True,timeout=900)
 for ext,b in [('stdout',d.stdout),('stderr',d.stderr)]:
  with (J/(name+'.'+ext)).open('xb') as f:f.write(b)
 assert d.returncode==0,(name,d.stderr.decode()[-2000:])
d=read(J/'delivery.stdout');assert d['verified_cues']==d['selected_cues']==105
with (J/'media-summary.json').open('x') as f:json.dump({'verified_files':len(rows),'wav_count':105,'all_fmd_hashes_equal':True,'reuse':'Existing exact source/score and WAV files linked; only metadata copied. No takes regenerated.','delivery':d,'scope':'Native delivery and source/sample checks; no listening/fine-sync/independent AV acceptance.'},f,indent=2)
print('All candidate media hashes and105 cue native delivery verified on CSD.')
