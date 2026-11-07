from pathlib import Path
import json,hashlib,subprocess
R=Path('/data/projects/jmev-render-worker-20261107');D=Path('/data/projects/jmev-stroke-probe-20261106');J=R/'jobs/trj-production19-delivery01';T=D/'work/production-repair19/workspace';N=D/'jobs/csd-production-assembly19/narration';B=D/'work/bin/production19'
a=json.loads((J/'resource-receipt.json').read_text());assert a['admitted'] and 'exit' not in a
q=json.loads((D/'jobs/csd-production-repair-build19/qualification.json').read_text());assert hashlib.sha256(B.read_bytes()).hexdigest()==q['binary_sha256']
for name in ['facts','chapters']:
 with (J/(name+'.stdout')).open('xb') as o,(J/(name+'.stderr')).open('xb') as e:subprocess.run([str(B),name],cwd=D/'work/production-repo07',stdout=o,stderr=e,check=True,timeout=120)
assert hashlib.sha256((J/'facts.stdout').read_bytes()).hexdigest()==q['facts_sha256'];assert (J/'chapters.stdout').read_text().splitlines()==q['chapters']
cmd=['python3','-I','-B',str(T/'tools/narration_delivery.py'),'--topic',str(T/'explainer_corpus/complex_analysis_visual'),'--narration',str(N),'--consumer','render','--gain-db','0']
for c in q['chapters']:cmd+=['--chapter-key',c]
with (J/'delivery.stdout').open('xb') as o,(J/'delivery.stderr').open('xb') as e:subprocess.run(cmd,cwd=D/'work/production-repo07',stdout=o,stderr=e,check=True,timeout=1200)
x=json.loads((J/'delivery.stdout').read_text());assert x['verified_cues']==x['selected_cues']==105 and x['plan_mode']=='external-ui'
(J/'delivery-result.json').write_text(json.dumps(dict(binary_sha256=q['binary_sha256'],native_facts_and_registry_match=True,delivery=x,scope='Native executable and full runtime narration/source contract on TRJ only; no new image or motion qualification'),indent=2))
