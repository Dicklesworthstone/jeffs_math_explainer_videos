"""Qualify complete native schedules after the21 wrapper's directory assertion fails.
Keeps the failed receipt; does not repeat native rendering or turn it into success.
"""
from pathlib import Path
import json,hashlib,shutil
R=Path('/data/projects/jmev-stroke-probe-20261106');Q=R/'qualification01/production-repair17';OLD=R/'jobs/csd-production-timeline21';J=R/'jobs/csd-production-timeline22';N=R/'jobs/csd-production-assembly19/narration'
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
d=json.loads((OLD/'resource-receipt.json').read_text());assert d.get('finished') and d['exit']==1 and d['inputs_sha256']==d['final_inputs_sha256'] and not d.get('stopped_for_resource_limit')
trace=(OLD/'command.log').read_text();assert 'in timeline' in trace and "TIMELINES.glob('*.timeline.tsv')" in trace and 'AssertionError' in trace and 'CalledProcessError' not in trace
source=OLD/'timeline-output/timeline';out=J/'timeline-output';out.mkdir()
build=json.loads((R/'jobs/csd-production-repair-build19/qualification.json').read_text());chapters=build['chapters'];assert {p.name for p in source.glob('*.timeline.tsv')}=={c+'.timeline.tsv' for c in chapters}
rows=[];cues=[];total=0
for c in chapters:
 for suffix in ['timeline','cues','cuelog']:
  p=source/(c+'.'+suffix+'.tsv');assert p.is_file();shutil.copy2(p,out/p.name);assert sha(p)==sha(out/p.name)
 tab=out/(c+'.timeline.tsv');lines=[l.split('\t') for l in tab.read_text().splitlines()];frames=[int(x[1]) for x in lines if x[0]=='frames'];fps=[int(x[1]) for x in lines if x[0]=='fps'];assert len(frames)==1 and frames[0]>0 and fps==[60];cues.extend(x[1] for x in lines if x[0]=='line');comparisons=[]
 for kind in ['timeline','cues','cuelog']:
  before=R/'jobs/csd-continuous-timeline16/timeline-output'/(c+'.'+kind+'.tsv');after=out/before.name;expected=before.read_bytes().replace(str(R/'jobs/csd-continuous-rebind16/narration').encode(),str(N).encode());comparisons.append(dict(kind=kind,old_sha256=sha(before),new_sha256=sha(after),exact_equal_after_named_directory_mapping=expected==after.read_bytes()))
 rows.append(dict(chapter=c,frames=frames[0],start_frame=total,end_frame_exclusive=total+frames[0],seconds=frames[0]/60,tables=comparisons));total+=frames[0]
script=R/'work/production-repair19/workspace/explainer_corpus/complex_analysis_visual/script.tsv';expected=[x.split('\t',1)[0] for x in script.read_text().splitlines() if x and not x.startswith('#')];assert cues==expected and len(cues)==105
(J/'timeline22-summary.json').write_text(json.dumps(dict(binary_sha256=build['binary_sha256'],assembly_summary_sha256=sha(R/'jobs/csd-production-assembly19/assembly19-summary.json'),chapters=rows,fps=60,total_frames=total,seconds=total/60,failed_wrapper_receipt_sha256=sha(OLD/'resource-receipt.json'),qualification_reason='Native render returned0 before wrapper asserted flat output directory. Timeline-only actually writes nested timeline directory; all45 tables now copied unchanged and all105 orderedcues verified. Original failedreceipt retained.',scope='Native schedules only; no image, motion, listening or release acceptance'),indent=2))
print('Qualified complete native schedules without repeating render:',total,'frames')
