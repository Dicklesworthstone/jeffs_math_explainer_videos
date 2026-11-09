from pathlib import Path
import json,hashlib
R=Path('/data/projects/jmev-stroke-probe-20261106')
receipts=[]
for n in ('csd-production-timeline21','csd-production-score21'):
 p=R/'jobs'/n/'resource-receipt.json'
 if not p.exists():print(json.dumps({'state':'waiting'}));raise SystemExit(0)
 d=json.loads(p.read_text())
 if not d.get('finished'):print(json.dumps({'state':'waiting'}));raise SystemExit(0)
 assert d['exit']==0 and d['inputs_sha256']==d['final_inputs_sha256'] and not d.get('stopped_for_resource_limit'),n
 receipts.append(p)
paths=set(receipts)
paths.add(R/'jobs/csd-production-timeline21/timeline21-summary.json')
for ext in ('*.timeline.tsv','*.cues.tsv','*.cuelog.tsv','*.layout.tsv'):paths.update((R/'jobs/csd-production-timeline21/timeline-output').glob(ext))
paths.add(R/'jobs/csd-production-score21/score21-summary.json')
paths.update(p for p in (R/'jobs/csd-production-score21/score').rglob('*') if p.is_file())
rows=[]
for p in sorted(paths):
 with p.open('rb') as f:h=hashlib.file_digest(f,'sha256').hexdigest()
 rows.append(dict(relative=str(p.relative_to(R)),sha256=h,bytes=p.stat().st_size))
print(json.dumps({'state':'ready','files':rows}))
