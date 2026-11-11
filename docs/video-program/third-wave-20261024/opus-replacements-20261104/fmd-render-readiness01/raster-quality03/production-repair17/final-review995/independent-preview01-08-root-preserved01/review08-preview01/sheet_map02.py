"""Reproduce csd-combined01-pictures.py's frame selection for 08_loop_sums from its own inputs (plan02 frame list,
the preview layout rows, the chapter's frame count), then map every plan02 window and lint anchor to picture
indices and sheet numbers (8 pictures per sheet). Read-only; prints JSON."""
import json,re,sys
from pathlib import Path
F=Path('/Users/jemanuel/projects/jeffs_math_explainer_videos/docs/video-program/third-wave-20261024/opus-replacements-20261104/fmd-render-readiness01/raster-quality03/production-repair17/final-review995')
old=[int(x) for x in (F/'combined-pictures-root01/frames-08_loop_sums-all.txt').read_text().splitlines()]
timeline=(F/'combined-timeline-terminal01/08_loop_sums.timeline.tsv').read_text().splitlines()
total=[int(r.split('\t')[1]) for r in timeline if r.split('\t')[0]=='frames'];assert len(total)==1;total=total[0]
layout=Path(sys.argv[1]).read_text().splitlines()
chosen=set(old)|set(range(10200,10380));anchors=[]
for row in layout:
 m=re.search(r'@([0-9.]+)s',row);at=round(float(m[1])*60);anchors.append((at,row[:60]))
 chosen.update(range(at,min(at+121,total),12));chosen.update(range(at,total,300))
frames=sorted(chosen);assert set(old)<=chosen and len(frames)<4000
idx={f:i+1 for i,f in enumerate(frames)}
def sheets(lo,hi):
 ps=[idx[f] for f in frames if lo<=f<=hi];return (min(ps),max(ps),(min(ps)-1)//8+1,(max(ps)-1)//8+1,len(ps)) if ps else None
plan=json.loads((F/'combined-pictures-root01/sampling-plan02.json').read_text())
out={'total_frames':total,'plan02_frames':len(old),'selected':len(frames),'sheets':(len(frames)+7)//8,'anchors':[],'windows':[]}
for at,row in anchors:out['anchors'].append({'frame':at,'row':row,'window':sheets(at,at+120),'in_plan02':at in set(old)})
for w in plan['windows']:
 if w['chapter']!='08_loop_sums':continue
 lo,hi=w['local_range'];out['windows'].append({'id':w['id'],'range':[lo,hi],'pictures_sheets':sheets(lo,hi)})
out['fade_out_c_lag']=sheets(13040,13110);out['required_run_10200_10380']=sheets(10200,10380);out['run_complete']=set(range(10200,10381))<=chosen
print(json.dumps(out,indent=1))
