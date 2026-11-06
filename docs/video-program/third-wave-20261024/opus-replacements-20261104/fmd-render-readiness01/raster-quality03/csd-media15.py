import json,hashlib,subprocess,os
from pathlib import Path
R=Path('/data/projects/jmev-stroke-probe-20261106');Q=R/'qualification01/raster-quality12';J=R/'jobs/csd-raster-media15';T=R/'work/raster-quality-fmd04/workspace'
OLD=R/'jobs/fmd-raster-rebind12/narration';N=J/'narration';BASE=R/'production-inputs01/production-repo07'
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def read(p):return json.loads(p.read_text())
r=read(J/'resource-receipt.json');assert r['admitted'] and 'exit' not in r
prior=read(R/'jobs/csd-raster-media14/resource-receipt.json');assert prior['exit']==1 and prior['inputs_sha256']==prior['final_inputs_sha256'] and not prior.get('resource_stop')
for row in read(Q/'fmd-media-binding14.json')['files']:assert sha(R/row['path'])==row['sha256']
N.mkdir()
for p in OLD.iterdir():
 dst=N/p.name
 if p.suffix=='.wav':dst.symlink_to(p.resolve(strict=True))
 else:
  with dst.open('xb') as f:f.write(p.read_bytes())
report=read(N/'report.json');before=read(OLD/'report.json');changes=[]
for cue,rows in report['lines'].items():
 for row in rows:
  old=row['source'];new=str(Path(old).resolve(strict=True));assert sha(Path(new))==row['source_sha256'];row['source']=new;changes.append([cue,old,new])
# Preserve exact transferred metadata; change only canonical source paths and report digest.
(N/'report.json').rename(N/'before-csd-path-report.json')
(N/'assembly.json').rename(N/'before-csd-path-assembly.json')
with (N/'report.json').open('x') as f:json.dump(report,f,indent=2)
a=read(OLD/'assembly.json');a['report_sha256']=sha(N/'report.json')
with (N/'assembly.json').open('x') as f:json.dump(a,f,indent=2)
for cue,rows in report['lines'].items():
 for x,y in zip(rows,before['lines'][cue]):assert {k:v for k,v in x.items() if k!='source'}=={k:v for k,v in y.items() if k!='source'}
wavs=list(N.glob('*.wav'));assert len(wavs)==105 and all(sha(p)==sha(OLD/p.name) for p in wavs)
assert sha(N/'phrase-timings.json')==sha(OLD/'phrase-timings.json')
C=T/'explainer_corpus/complex_analysis_visual';binary=R/'work/bin/production09i'
chapters=subprocess.check_output([str(binary),'chapters'],text=True).splitlines();assert len(chapters)==15
commands=[('coherent',['/usr/bin/python3','-I','-B',str(T/'tools/coherent_narration.py'),'verify',str(N),'--script',str(C/'script.tsv'),'--units',str(N/'external-units.json'),'--require-directions'])]
args=['/usr/bin/python3','-I','-B',str(T/'tools/narration_delivery.py'),'--topic',str(C),'--narration',str(N),'--consumer','render','--gain-db','0','--score',str(R/'production-inputs01/scored/stems')]
for c in chapters:args+=['--chapter-key',c]
commands.append(('delivery',args))
for name,cmd in commands:
 d=subprocess.run(cmd,cwd=BASE,capture_output=True,timeout=900)
 for ext,b in [('stdout',d.stdout),('stderr',d.stderr)]:
  with (J/(name+'.'+ext)).open('xb') as f:f.write(b)
 assert d.returncode==0,(name,d.stderr.decode()[-2000:])
d=read(J/'delivery.stdout');assert d['verified_cues']==d['selected_cues']==105
with (J/'media-summary.json').open('x') as f:json.dump(dict(wav_files_byte_identical=105,phrase_timings_byte_identical=True,source_path_changes=changes,delivery=d,scope='Source/sample verification only; no listening, fine sync or independent AV acceptance.'),f,indent=2)
