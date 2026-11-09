from pathlib import Path
import json,hashlib,subprocess,re
from PIL import Image,ImageDraw
R=Path.cwd();B=R/'narration_lab/third-wave-20261024/root/five-film-release-20261027';A=R/'docs/video-program/third-wave-20261024/opus-replacements-20261104/fmd-render-readiness01/raster-quality03/production-repair17/algebra-fixes';J=B/'algebra-final-pictures14';O=B/'algebra-final-master14/final';sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
r=json.loads((J/'resource-receipt.json').read_text());assert r['admitted'] and 'exit' not in r
for job in ['algebra-final-master14','algebra-final-verify14']:
 r=json.loads((B/job/'resource-receipt.json').read_text());assert r['exit']==0 and r['inputs_sha256']==r['final_inputs_sha256'] and not r.get('resource_stop')
plan=json.loads((A/'final-picture-plan14.json').read_text());assert plan['frames']==251862
for x in plan['sources']:assert sha(Path(x['path']))==x['sha256']
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
pics=J/'pictures';pics.mkdir();movie=O/'abstract_algebra_expanded_master.mp4'
subprocess.run(['ffmpeg','-nostdin','-n','-v','error','-xerror','-threads','2','-i',str(movie),'-an','-filter_script:v',str(f),'-vsync','0','-q:v','2',str(pics/'frame-%06d.jpg')],check=True,timeout=14400)
files=sorted(pics.glob('frame-*.jpg'));assert len(files)==len(samples)
for x,p in zip(samples,files):x.update(path=str(p),sha256=sha(p))
sheets=[]
for k in range(0,len(samples),8):
 canvas=Image.new('RGB',(1920,4*565),'#10151d');draw=ImageDraw.Draw(canvas)
 for i,x in enumerate(samples[k:k+8]):
  im=Image.open(x['path']).convert('RGB');col=i%2;row=i//2;canvas.paste(im,(col*960,row*565));draw.text((col*960+5,row*565+542),x['chapter']+' '+str(round(x['local_frame']/60,3))+'s globalframe '+str(x['frame']),fill='white')
 p=J/('sheet-'+str(k//8+1).zfill(4)+'.jpg');canvas.save(p,quality=94);sheets.append(dict(path=str(p),sha256=sha(p)))
(J/'sampling.json').write_text(json.dumps(dict(master_sha256=sha(movie),frames=251862,samples=samples,sheets=sheets,pictures_viewed=False,motion_reviewed=False,listening_accepted=False,native_4k_detail_reviewed=False,scope='Selected final master pictures scaled to960x540; layout review only; native detail, continuous motion, listening and independent acceptance remain.'),indent=2))
