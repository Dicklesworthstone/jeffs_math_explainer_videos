"""Review the held final08 comprehensively before another expensive render."""
from pathlib import Path
import hashlib,json,re,subprocess
from PIL import Image,ImageDraw
R=Path('/Users/jemanuel/projects/jeffs_math_explainer_videos')
Q=R/'docs/video-program/third-wave-20261024/opus-replacements-20261104/fmd-render-readiness01/raster-quality03/production-repair17'
S=Q/'final08-terminal64'
J=R/'narration_lab/third-wave-20261024/root/five-film-release-20261027/complex-final08-pictures71'
def sha(p):
 with p.open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest()
a=json.loads((J/'resource-receipt.json').read_text());assert a['admitted'] and 'exit' not in a
summary=json.loads((S/'render-summary.json').read_text())
for name in ('08_loop_sums.mp4','08_loop_sums.timeline.tsv','08_loop_sums.layout.tsv'):
 row=next(x for x in summary['files'] if Path(x['path']).name==name);assert sha(S/name)==row['sha256']
picked={f:['one-second sample'] for f in range(0,14959,60)}
def add(f,why):
 if 0<=f<14959:picked.setdefault(f,[]).append(why)
for line in (S/'08_loop_sums.timeline.tsv').read_text().splitlines():
 x=line.split('\t')
 if x[0] in ('line','mark'):
  for delta in (0,30,60,120):add(int(x[2])+delta,x[0]+':'+x[1])
for f in range(100*60,171*60,30):add(f,'c label region')
for line in (S/'08_loop_sums.layout.tsv').read_text().splitlines():
 if not line.strip():continue
 m=re.search(r'@([0-9]+(?:\.[0-9]+)?)s',line);assert m,line
 anchor=round(float(m[1])*60)
 for delta in (-12,0,12,60,120,300,600):add(anchor+delta,'layout: '+line)
 if 'later looks' in line:
  for f in range(anchor,14959,300):add(f,'persistent layout: '+line)
def balanced(xs):
 if len(xs)==1:return xs[0]
 k=len(xs)//2;return '('+balanced(xs[:k])+'+'+balanced(xs[k:])+')'
frames=sorted(picked);f=J/'select.filter';f.write_text("select='"+balanced(['eq(n,'+str(n)+')' for n in frames])+"',scale=960:540")
pics=J/'pictures';pics.mkdir()
subprocess.run(['/opt/homebrew/bin/ffmpeg','-nostdin','-n','-v','error','-xerror','-threads','2','-i',str(S/'08_loop_sums.mp4'),'-an','-filter_script:v',str(f),'-vsync','0','-q:v','2',str(pics/'frame-%06d.jpg')],check=True,timeout=1800)
files=sorted(pics.glob('frame-*.jpg'));assert len(files)==len(frames)
samples=[dict(frame=n,seconds=n/60,reasons=picked[n],path=str(p),sha256=sha(p)) for n,p in zip(frames,files)]
sheets=[]
for start in range(0,len(samples),8):
 im=Image.new('RGB',(1920,2260),'#10151d');draw=ImageDraw.Draw(im)
 for k,row in enumerate(samples[start:start+8]):
  with Image.open(row['path']) as frame:
   assert frame.size==(960,540);x,y=(k%2)*960,(k//2)*565;im.paste(frame,(x,y));draw.text((x+5,y+542),f"08 {row['seconds']:.3f}s frame {row['frame']}",fill='white')
 p=J/f'sheet-{start//8+1:04d}.jpg';im.save(p,quality=94);sheets.append(dict(path=str(p),sha256=sha(p)))
(J/'sampling.json').write_text(json.dumps(dict(movie_sha256=sha(S/'08_loop_sums.mp4'),samples=samples,sheets=sheets,actually_viewed=False,scope='Held final08 review samples. Not repaired-picture, motion, listening or independent acceptance.'),indent=2))
