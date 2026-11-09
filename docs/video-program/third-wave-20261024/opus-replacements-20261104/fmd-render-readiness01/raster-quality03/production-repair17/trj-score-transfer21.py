"""Copy completed CSD schedules and score under TRJ's unchanged resource guard."""
from pathlib import Path
import hashlib,json,subprocess,shlex
R=Path('/data/projects/jmev-render-worker-20261107');D=Path('/data/projects/jmev-stroke-probe-20261106');Q=R/'qualification01/production19';J=R/'jobs/trj-production21-score-transfer01';M=Q/'score-transfer21-files.json'
def sha(p):
 with p.open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest()
a=json.loads((J/'resource-receipt.json').read_text());assert a['admitted'] and 'exit' not in a
rows=json.loads(M.read_text())['files'];assert rows
for row in rows:
 rel=Path(row['relative']);assert not rel.is_absolute() and '..' not in rel.parts
 assert rel.parts[:2] in [('jobs','csd-production-timeline21'),('jobs','csd-production-score21')]
 dest=D/rel
 if dest.exists():assert dest.is_file() and sha(dest)==row['sha256'],str(dest)
listing=J/'files.txt';listing.write_text(''.join(x['relative']+'\n' for x in rows))
subprocess.run(['rsync','-a','--ignore-existing','--files-from='+str(listing),'csd:'+str(D)+'/',str(D)+'/'],check=True,timeout=3600)
for row in rows:
 p=D/row['relative'];assert p.stat().st_size==row['bytes'] and sha(p)==row['sha256'],str(p)
for name in ('csd-production-timeline21','csd-production-score21'):
 d=json.loads((D/'jobs'/name/'resource-receipt.json').read_text());assert d.get('exit')==0 and d['inputs_sha256']==d['final_inputs_sha256'] and not d.get('stopped_for_resource_limit')
summary=json.loads((D/'jobs/csd-production-score21/score21-summary.json').read_text());assert len(summary['chapters'])==15 and summary['all_qa_passed']
for c in summary['chapters']:assert json.loads((D/'jobs/csd-production-score21/score/stems/qa'/(c+'.json')).read_text())['passed'] is True
(J/'transfer-result.json').write_text(json.dumps(dict(files=len(rows),bytes=sum(x['bytes'] for x in rows),all_hashes_match=True,all15_score_qa_passed=True,scope='Transferred exact completed schedules and score. No final render or visual/motion acceptance.'),indent=2))
