from pathlib import Path
import json,hashlib,subprocess,re
from PIL import Image,ImageDraw
R=Path.cwd();B=R/'narration_lab/third-wave-20261024/root/five-film-release-20261027';I=B/'algebra-fixes-inputs02';J=B/'algebra-fixes-preview08';N=B/'algebra-fixes-assemble03/narration';T=B/'algebra-fixes-timeline04/timeline';S=B/'algebra-fixes-score07/score/stems'
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
a=json.loads((J/'resource-receipt.json').read_text());assert a['admitted'] and 'exit' not in a
chapters=['11_permutations','14_cosets'];out=J/'preview';assert not out.exists()
for c in chapters:assert json.loads((S/'qa'/(c+'.json')).read_text())['passed'] is True
subprocess.run([str(I/'abstract_algebra_expanded'),'render',*chapters,'--res','960x540','--fps','60','--threads','4','--narration',str(N),'--score',str(S),'--out',str(out),'--lint','--crf','18','--preset','veryfast'],check=True,timeout=14400)
rows=[]
for c in chapters:
 assert (out/(c+'.timeline.tsv')).read_bytes()==(T/(c+'.timeline.tsv')).read_bytes()
 assert (out/(c+'.cues.tsv')).read_bytes()==(T/(c+'.cues.tsv')).read_bytes()
 movie=out/(c+'.mp4');probe=json.loads(subprocess.check_output(['ffprobe','-v','error','-show_streams','-of','json',str(movie)]));v=next(x for x in probe['streams'] if x['codec_type']=='video');frames=int(v['nb_frames']);assert (v['width'],v['height'],v['avg_frame_rate'])==(960,540,'60/1')
 table=[x.split('\t') for x in (T/(c+'.timeline.tsv')).read_text().splitlines()];assert [int(x[1]) for x in table if x[0]=='frames']==[frames]
 starts=[(x[1],int(x[2])) for x in table if x[0]=='line'];chosen={}
 def add(f,why):
  if 0<=f<frames:chosen.setdefault(f,set()).add(why)
 for f in range(0,frames,300):add(f,'whole chapter 5s sample')
 add(frames-1,'last frame')
 for cue,start in starts:
  for delta in (0,30,60):add(start+delta,cue+' beginning')
 target='11_17' if c.startswith('11') else '14_14';idx=next(i for i,x in enumerate(starts) if x[0]==target);start=starts[idx][1];end=starts[idx+1][1] if idx+1<len(starts) else frames
 for f in range(start,end,30):add(f,target+' full repaired cue half-second sample')
 if c.startswith('14'):
  for f in range(round(115.8*60),round(124.5*60),15):add(f,'historical crossing absolute anchor')
 layout=out/(c+'.layout.tsv');assert layout.is_file()
 for line in layout.read_text().splitlines():
  if not line.strip():continue
  m=re.search(r'@([0-9]+(?:\.[0-9]+)?)s',line.split('\t',1)[0]);assert m,line
  f=round(float(m[1])*60)
  for d in (-12,0,12,60,180,360,600):add(f+d,'layout anchor '+line)
  if 'later looks' in line:
   for n in range(f,frames,300):add(n,'layout recurrence scan '+line)
 folder=J/'pictures'/c;folder.mkdir(parents=True);samples=[dict(frame=f,seconds=f/60,reasons=sorted(v)) for f,v in sorted(chosen.items())]
 filt=folder/'select.filter';filt.write_text("select='"+'+'.join('eq(n,'+str(x['frame'])+')' for x in samples)+"'")
 subprocess.run(['ffmpeg','-nostdin','-n','-v','error','-i',str(movie),'-filter_script:v',str(filt),'-vsync','0',str(folder/'frame-%05d.png')],check=True,timeout=1200)
 pictures=sorted(folder.glob('frame-*.png'));assert len(pictures)==len(samples)
 for x,p in zip(samples,pictures):x.update(path=str(p),sha256=sha(p))
 sheets=[]
 for k in range(0,len(samples),8):
  canvas=Image.new('RGB',(1440,4*430),'#10151d');draw=ImageDraw.Draw(canvas)
  for i,x in enumerate(samples[k:k+8]):
   col=i%2;row=i//2;im=Image.open(x['path']).convert('RGB');im.thumbnail((720,405));canvas.paste(im,(col*720,row*430));draw.text((col*720+5,row*430+407),c+' '+str(x['frame'])+' '+str(round(x['seconds'],3))+'s',fill='white')
  p=folder/('sheet-'+str(k//8+1).zfill(4)+'.jpg');canvas.save(p,quality=94);sheets.append(dict(path=str(p),sha256=sha(p)))
 rows.append(dict(chapter=c,frames=frames,movie=str(movie),mp4_sha256=sha(movie),layout_sha256=sha(layout),samples=samples,sheets=sheets))
(J/'preview08-summary.json').write_text(json.dumps(dict(chapters=rows,pictures_reviewed=False,motion_reviewed=False,listening_accepted=False,scope='Two repaired native chapters, scored 540p60; extracted samples require actual review. Other chapters require reuse proof and final4K review.'),indent=2))
