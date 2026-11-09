from pathlib import Path
import json,hashlib,subprocess,os
R=Path('/data/projects/jmev-stroke-probe-20261106');Q=R/'qualification01/production-repair17';J=R/'jobs/csd-boundary-label-timeline55';N=R/'jobs/csd-boundary-label-rebind55/narration';OLD=R/'jobs/csd-production-assembly19/narration';B=R/'work/bin/boundary-label52'
def sha(p):
 with p.open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest()
a=json.loads((J/'resource-receipt.json').read_text());assert a['admitted'] and 'exit' not in a
os.sched_setaffinity(0,{0,1,2,3})
os.environ.update(PATH='/usr/bin:/bin',TMPDIR=str(R/'work/tmp'),XDG_CACHE_HOME=str(R/'work/cache'))
with (J/'native.stdout').open('xb') as out,(J/'native.stderr').open('xb') as err:
 subprocess.run([str(B),'render','09_boundary','--timeline-only','--fps','60','--threads','4','--narration',str(N),'--out',str(J/'timeline-output')],cwd=R/'work/production-repo07',stdout=out,stderr=err,check=True,timeout=172800)
rows=[]
for kind in ('timeline','cues','cuelog'):
 old=R/'jobs/csd-production-timeline21/timeline-output'/('09_boundary.'+kind+'.tsv');new=J/'timeline-output'/old.name
 assert old.read_bytes().replace(str(OLD).encode(),str(N).encode())==new.read_bytes(),kind
 rows.append(dict(kind=kind,old_sha256=sha(old),new_sha256=sha(new),equal_after_named_narration_path_mapping=True))
(J/'comparison.json').write_text(json.dumps(dict(tables=rows,frames=16504,named_path_mapping={str(OLD):str(N)},scope='Exact complete chapter09 tables only; repaired pictures and final4K remain unreviewed.'),indent=2))
