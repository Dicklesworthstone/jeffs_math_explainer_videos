from pathlib import Path
import json,hashlib,subprocess,re
from PIL import Image,ImageDraw
R=Path('/data/projects/jmev-render-worker-20261107');Q=R/'qualification01/production19'
J=R/'jobs/trj-production19-pictures01';O=R/'jobs/trj-production19-master01/final'
def sha(p):
 with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest()
r=json.loads((J/'resource-receipt.json').read_text());assert r['admitted'] and 'exit' not in r
assert r['policy']['stop_free_gib']==8 and r['policy']['phase_growth_gib']==4
manifest=Q/'pictures01-inputs.json';assert sha(manifest)==r['inputs_sha256']
rows=json.loads(manifest.read_text())['files'];bound={x['path']:x['sha256'] for x in rows};assert len(bound)==len(rows)
for p,h in bound.items():assert sha(p)==h,p
def require(p):assert bound.get(str(p))==sha(p),str(p)
require(Path(__file__).resolve())
for job in ['trj-production19-master01','trj-production19-verify01']:
 receipt=R/'jobs'/job/'resource-receipt.json';require(receipt);r=json.loads(receipt.read_text())
 assert r.get('finished') and r['exit']==0 and r['inputs_sha256']==r['final_inputs_sha256']
 assert not any(r.get(k) for k in ('resource_stop','error','stopped_for_resource_limit'))
plan_path=Q/'final-picture-plan41.json';require(plan_path);plan=json.loads(plan_path.read_text());assert plan['frames']==133662
for x in plan['sources']:require(Path(x['path']));assert sha(x['path'])==x['sha256']
summary_path=O.parent/'master-summary.json';require(summary_path);summary=json.loads(summary_path.read_text())
assert [c['chapter'] for c in plan['chapters']]==summary['chapters'] and summary['frames']==133662
movie=O/'complex_analysis_visual_master.mp4';require(movie)
assert sha(movie)==summary['outputs']['master']['sha256']
for c in plan['chapters']:
 timeline=O/(c['chapter']+'.timeline.tsv');require(timeline)
 rows=[line.split('\t') for line in timeline.read_text().splitlines()]
 assert next(int(x[1]) for x in rows if x[0]=='frames')==c['frames']
 require(O/(c['chapter']+'.layout.tsv'))
picked={x['frame']:x for x in plan['samples']}
for c in plan['chapters']:
 p=O/(c['chapter']+'.layout.tsv');assert p.exists()
 for line in p.read_text().splitlines():
  if not line.strip():continue
  m=re.search(r'@([0-9]+(?:\.[0-9]+)?)s',line.split('\t',1)[0]);assert m,line
  anchor=round(float(m[1])*60);local=[anchor+d for d in [-12,0,12,60,180,360,600]]
  if 'later looks' in line:local+=list(range(anchor,c['frames'],300))
  for f in local:
   if 0<=f<c['frames']:
    frame=c['start_frame']+f;x=picked.setdefault(frame,dict(chapter=c['chapter'],frame=frame,local_frame=f,reasons=[]));x['reasons']=sorted(set(x['reasons']+['final layout: '+line]))
samples=[x for _,x in sorted(picked.items())]
def balanced(xs):
 if len(xs)==1:return xs[0]
 k=len(xs)//2;return '('+balanced(xs[:k])+'+'+balanced(xs[k:])+')'
f=J/'select.filter';f.write_text("select='"+balanced(['eq(n,'+str(x['frame'])+')' for x in samples])+"',scale=960:540")
pics=J/'pictures';pics.mkdir();movie=O/'complex_analysis_visual_master.mp4'
subprocess.run(['ffmpeg','-nostdin','-n','-v','error','-xerror','-threads','2','-i',str(movie),'-an','-filter_script:v',str(f),'-vsync','0','-q:v','2',str(pics/'frame-%06d.jpg')],check=True,timeout=14400)
files=sorted(pics.glob('frame-*.jpg'));assert len(files)==len(samples)
for x,p in zip(samples,files):x.update(path=str(p),sha256=sha(p))
sheets=[]
for k in range(0,len(samples),8):
 canvas=Image.new('RGB',(1920,4*565),'#10151d');draw=ImageDraw.Draw(canvas)
 for i,x in enumerate(samples[k:k+8]):
  im=Image.open(x['path']).convert('RGB');col=i%2;row=i//2;canvas.paste(im,(col*960,row*565));draw.text((col*960+5,row*565+542),x['chapter']+' '+str(round(x['local_frame']/60,3))+'s globalframe '+str(x['frame']),fill='white')
 p=J/('sheet-'+str(k//8+1).zfill(4)+'.jpg');canvas.save(p,quality=94);sheets.append(dict(path=str(p),sha256=sha(p)))
(J/'sampling.json').write_text(json.dumps(dict(master_sha256=sha(movie),frames=133662,samples=samples,sheets=sheets,pictures_viewed=False,motion_reviewed=False,listening_accepted=False,native_4k_detail_reviewed=False,scope='Selected final master pictures scaled to960x540; layout review only; native detail, continuous motion, listening and independent acceptance remain.'),indent=2))

for p,h in bound.items():assert sha(p)==h,p
