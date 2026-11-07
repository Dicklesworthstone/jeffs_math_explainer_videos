from pathlib import Path
import json,hashlib,subprocess,os
R=Path('/data/projects/jmev-stroke-probe-20261106');Q=R/'qualification01/production-repair17';T=R/'work/production-repair19/workspace';J=R/'jobs/csd-production-score20';N=R/'jobs/csd-production-assembly19/narration';TL=R/'jobs/csd-production-timeline20/timeline-output';BIN=R/'work/bin/production19'
def sha(p):
 with p.open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest()
a=json.loads((J/'resource-receipt.json').read_text());assert a['admitted'] and 'exit' not in a
os.sched_setaffinity(0,{0,1,2,3});os.environ.update(TMPDIR=str(R/'work/tmp'),XDG_CACHE_HOME=str(R/'work/cache'))
chapters=json.loads((R/'jobs/csd-production-repair-build19/qualification.json').read_text())['chapters'];out=J/'score'
assert not out.exists()
with (J/'score.stdout').open('xb') as o,(J/'score.stderr').open('xb') as e:
 subprocess.run([str(BIN),'score','all','--narration',str(N),'--timelines',str(TL),'--out',str(out)],cwd=R/'work/production-repo07',stdout=o,stderr=e,check=True,timeout=7200)
qa=out/'stems/qa';assert {p.stem for p in qa.glob('*.json')}==set(chapters)
reports={c:json.loads((qa/(c+'.json')).read_text()) for c in chapters};assert all(d['passed'] is True for d in reports.values())
(J/'score20-summary.json').write_text(json.dumps(dict(chapters=chapters,all_qa_passed=True,files={str(p.relative_to(out)):sha(p) for p in out.rglob('*') if p.is_file()},scope='Native score QA only; no listening or mix acceptance'),indent=2))
print('All15 native score QA passed')
