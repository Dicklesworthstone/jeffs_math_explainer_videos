from pathlib import Path
import json,hashlib,subprocess,shutil,os
R=Path('/data/projects/jmev-hz2-render-worker-20261107');D=Path('/data/projects/jmev-stroke-probe-20261106');J=R/'jobs/hz2-production19-delivery48';T=D/'work/production-repair19/workspace';OLD=D/'jobs/csd-production-assembly19/narration';N=J/'narration';B=D/'work/bin/production19'
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
a=json.loads((J/'resource-receipt.json').read_text());assert a['admitted'] and 'exit' not in a
receipt=json.loads((R/'jobs/hz2-runtime-transfer47/resource-receipt.json').read_text())
assert receipt.get('finished') and receipt['exit']==0 and receipt['inputs_sha256']==receipt['final_inputs_sha256'] and not receipt.get('stopped_for_resource_limit')
assert not D.exists() and not D.is_symlink(), 'Existing host path; never replace'
D.symlink_to(R/'work/frozen-runtime47/data/projects/jmev-stroke-probe-20261106',target_is_directory=True)
assert D.resolve()==R/'work/frozen-runtime47/data/projects/jmev-stroke-probe-20261106'
os.environ.update(PATH='/usr/bin:/bin',TMPDIR=str(R/'work/tmp'),XDG_CACHE_HOME=str(R/'work/cache'))
shutil.copytree(OLD,N)
report=json.loads((N/'report.json').read_text());changes=[]
for cue,rows in report['lines'].items():
 for row in rows:
  old=Path(row['source']);new=old.resolve(strict=True)
  assert old.is_relative_to(D) and new==D.resolve()/old.relative_to(D)
  assert sha(new)==row['source_sha256']
  if str(old)!=str(new):changes.append(dict(cue=cue,old=str(old),new=str(new)));row['source']=str(new)
(N/'report.json').write_text(json.dumps(report,indent=2)+'\n')
assembly=json.loads((N/'assembly.json').read_text());assembly['report_sha256']=sha(N/'report.json');(N/'assembly.json').write_text(json.dumps(assembly,indent=2)+'\n')
oldreport=json.loads((OLD/'report.json').read_text())
for cue,rows in report['lines'].items():
 for row,prior in zip(rows,oldreport['lines'][cue]):
  assert {k:v for k,v in row.items() if k!='source'}=={k:v for k,v in prior.items() if k!='source'}
for f in OLD.iterdir():
 if f.is_file() and f.name not in ('report.json','assembly.json'):assert sha(f)==sha(N/f.name)
q=json.loads((D/'jobs/csd-production-repair-build19/qualification.json').read_text());assert sha(B)==q['binary_sha256']
for name in ['facts','chapters']:
 with (J/(name+'.stdout')).open('xb') as o,(J/(name+'.stderr')).open('xb') as e:subprocess.run([str(B),name],cwd=D/'work/production-repo07',stdout=o,stderr=e,check=True,timeout=120)
assert sha(J/'facts.stdout')==q['facts_sha256'];assert (J/'chapters.stdout').read_text().splitlines()==q['chapters']
cmd=['python3','-I','-B',str(T/'tools/narration_delivery.py'),'--topic',str(T/'explainer_corpus/complex_analysis_visual'),'--narration',str(N),'--consumer','render','--gain-db','0']
for c in q['chapters']:cmd+=['--chapter-key',c]
with (J/'delivery.stdout').open('xb') as o,(J/'delivery.stderr').open('xb') as e:subprocess.run(cmd,cwd=D/'work/production-repo07',stdout=o,stderr=e,check=True,timeout=1200)
x=json.loads((J/'delivery.stdout').read_text());assert x['verified_cues']==x['selected_cues']==105 and x['plan_mode']=='external-ui'
(J/'delivery-result.json').write_text(json.dumps(dict(binary_sha256=q['binary_sha256'],native_facts_and_registry_match=True,delivery=x,path_rebindings=changes,scope='Resolved alias paths only; 105 WAVs, sentence partitions and phrase marks unchanged; native delivery passed. No picture or listening acceptance.'),indent=2))
