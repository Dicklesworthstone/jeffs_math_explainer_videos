from pathlib import Path
import json,hashlib,subprocess,os
R=Path('/data/projects/jmev-stroke-probe-20261106')
J=R/'jobs/csd-two-label-timeline165'
N=R/'jobs/csd-two-label-rebind165/narration'
OLD=R/'jobs/csd-production-assembly19/narration'
B=R/'work/bin/two-label164'
def sha(p):
 with p.open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest()
a=json.loads((J/'resource-receipt.json').read_text());assert a['admitted'] and 'exit' not in a
os.sched_setaffinity(0,{0,1,2,3})
os.environ.update(PATH='/usr/bin:/bin',TMPDIR=str(R/'work/tmp'),XDG_CACHE_HOME=str(R/'work/cache'))
with (J/'native.stdout').open('xb') as out,(J/'native.stderr').open('xb') as err:
 subprocess.run([str(B),'render','08_loop_sums','09_boundary','--timeline-only','--fps','60','--threads','4','--narration',str(N),'--out',str(J/'timeline-output')],cwd=R/'work/production-repo07',stdout=out,stderr=err,check=True,timeout=172800)
rows=[]
for chapter,frames in [('08_loop_sums',14959),('09_boundary',16504)]:
 for kind in ('timeline','cues','cuelog'):
  old=R/'jobs/csd-production-timeline21/timeline-output'/(chapter+'.'+kind+'.tsv');new=J/'timeline-output'/old.name
  assert old.read_bytes().replace(str(OLD).encode(),str(N).encode())==new.read_bytes(),(chapter,kind)
  rows.append(dict(chapter=chapter,kind=kind,old_sha256=sha(old),new_sha256=sha(new),equal_after_named_narration_path_mapping=True))
 timeline=(J/'timeline-output'/(chapter+'.timeline.tsv')).read_text().splitlines()
 assert next(int(x.split('\t')[1]) for x in timeline if x.startswith('frames\t'))==frames
(J/'comparison.json').write_text(json.dumps(dict(tables=rows,frames={'08_loop_sums':14959,'09_boundary':16504},named_path_mapping={str(OLD):str(N)},scope='Exact complete chapter08/09 tables only; repaired pictures and final4K remain unreviewed.'),indent=2))
