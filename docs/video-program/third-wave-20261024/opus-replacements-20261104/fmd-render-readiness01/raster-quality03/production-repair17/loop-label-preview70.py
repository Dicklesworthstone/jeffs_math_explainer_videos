from pathlib import Path
import hashlib,json,subprocess,os,re
from PIL import Image,ImageDraw
R=Path('/data/projects/jmev-stroke-probe-20261106');Q=R/'qualification01/production-repair17';J=R/'jobs/csd-loop-label-preview70';S=J/'preview';N=R/'jobs/csd-two-label-rebind69/narration';B=R/'work/bin/two-label68';T=R/'jobs/csd-two-label-timeline69/timeline-output'
def sha(p):
 with p.open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest()
a=json.loads((J/'resource-receipt.json').read_text());assert a['admitted'] and 'exit' not in a
assert a['policy']['stop_free_gib']==8 and a['policy']['phase_growth_gib']==4
os.sched_setaffinity(0,set(range(16)));os.environ.update(PATH='/usr/bin:/bin',TMPDIR=str(R/'work/tmp'),XDG_CACHE_HOME=str(R/'work/cache'))
with (J/'native.stdout').open('xb') as out,(J/'native.stderr').open('xb') as err:
 subprocess.run([str(B),'render','08_loop_sums','--res','960x540','--fps','60','--threads','16','--narration',str(N),'--out',str(S),'--lint','--crf','18','--preset','veryfast'],cwd=R/'work/production-repo07',stdout=out,stderr=err,check=True,timeout=172800)
for kind in ('timeline','cues','cuelog'):assert (S/('08_loop_sums.'+kind+'.tsv')).read_bytes()==(T/('08_loop_sums.'+kind+'.tsv')).read_bytes()
p=json.loads(subprocess.check_output(['ffprobe','-v','error','-show_streams','-of','json',str(S/'08_loop_sums.mp4')]))
v=[x for x in p['streams'] if x['codec_type']=='video'];assert len(v)==1 and (v[0]['width'],v[0]['height'],v[0]['avg_frame_rate'],int(v[0]['nb_frames']))==(960,540,'60/1',14959)
picked={f:['one-second sample'] for f in range(0,14959,60)}
def add(f,why):
 if 0<=f<14959:picked.setdefault(f,[]).append(why)
for line in (S/'08_loop_sums.timeline.tsv').read_text().splitlines():
 parts=line.split('\t')
 if parts[0] in ('line','mark'):
  for delta in (0,30,60,120):add(int(parts[2])+delta,parts[0]+':'+parts[1])
for f in range(100*60,171*60,30):add(f,'Prior c label collision and surrounding proof: half-second sample')
for at in (119,120,121):
 for delta in (-.2,0,.2,1,2):add(round((at+delta)*60),'Prior collision anchor')
for line in (S/'08_loop_sums.layout.tsv').read_text().splitlines():
 if not line.strip():continue
 match=re.search(r'@([0-9]+(?:\.[0-9]+)?)s',line);assert match,line
 at=float(match[1]);later=re.search(r'still there at (\d+) later looks',line);count=int(later[1]) if later else 0
 for delta in (-.2,0,.2,1,2,1+.2*count,2+.2*count):add(round((at+delta)*60),'Current lint: '+line)
 if later:
  for f in range(round(at*60),14959,300):add(f,'Persistent current lint: '+line)
def balanced(xs):
 if len(xs)==1:return xs[0]
 k=len(xs)//2;return '('+balanced(xs[:k])+'+'+balanced(xs[k:])+')'
frames=sorted(picked);filter_path=J/'select.filter';filter_path.write_text("select='"+balanced(['eq(n,'+str(f)+')' for f in frames])+"'")
pictures=J/'pictures';pictures.mkdir()
subprocess.run(['ffmpeg','-nostdin','-n','-v','error','-xerror','-threads','2','-i',str(S/'08_loop_sums.mp4'),'-an','-filter_script:v',str(filter_path),'-vsync','0','-q:v','2',str(pictures/'frame-%06d.jpg')],check=True,timeout=900)
files=sorted(pictures.glob('frame-*.jpg'));assert len(files)==len(frames)
samples=[dict(frame=f,seconds=f/60,reasons=picked[f],path=str(p),sha256=sha(p)) for f,p in zip(frames,files)];sheets=[]
for k in range(0,len(samples),8):
 canvas=Image.new('RGB',(1920,2260),'#10151d');draw=ImageDraw.Draw(canvas)
 for i,sample in enumerate(samples[k:k+8]):
  with Image.open(sample['path']) as im:
   assert im.size==(960,540);x,y=(i%2)*960,(i//2)*565;canvas.paste(im.convert('RGB'),(x,y));draw.text((x+5,y+542),f"08_loop_sums frame {sample['frame']} {sample['seconds']:.3f}s",fill='white')
 p=J/f'sheet-{k//8+1:04d}.jpg';canvas.save(p,quality=94);sheets.append(dict(path=str(p),sha256=sha(p)))
with (J/'sampling.json').open('x') as f:json.dump(dict(movie_sha256=sha(S/'08_loop_sums.mp4'),samples=samples,sheets=sheets,scope='Label-repaired native540p preview extraction; actual viewing/motion,4K, listening/fine-sync and independent AV still required.'),f,indent=2)
