import sys,json,hashlib,subprocess,math
from pathlib import Path
from PIL import Image,ImageDraw
T=Path('/data/projects/jmev-stroke-probe-20261106');J=T/'jobs/csd-gallery-early-review01'
r=json.loads((J/'resource-receipt.json').read_text());assert r['admitted'] and 'exit' not in r
h=hashlib.sha256();n=0;p=J/'05_gallery.mp4'
with p.open('xb') as f:
 while b:=sys.stdin.buffer.read(1048576):
  n+=len(b);assert n<=8344533;h.update(b);f.write(b)
assert n==8344533 and h.hexdigest()=='f5653796ed5f5cedab6d0b3374ebc136614aac5b74a584b5ba3bc05208b84d79'
probe=json.loads(subprocess.check_output(['ffprobe','-v','error','-show_streams','-of','json',str(p)]));v=next(s for s in probe['streams'] if s['codec_type']=='video');assert (v['width'],v['height'],v['r_frame_rate'],int(v['nb_frames']))==(960,540,'60/1',9879)
with (J/'full-decode.log').open('x') as f:subprocess.run(['ffmpeg','-v','error','-xerror','-i',str(p),'-f','null','-'],stdout=f,stderr=subprocess.STDOUT,check=True,timeout=300)
frames=J/'frames';frames.mkdir();subprocess.run(['ffmpeg','-v','error','-i',str(p),'-vf','fps=1','-threads','2',str(frames/'%04d.png')],check=True,timeout=300)
ps=sorted(frames.glob('*.png'));assert len(ps)==165,len(ps)
for start in range(0,len(ps),8):
 sheet=Image.new('RGB',(1920,4*570),(16,16,20));d=ImageDraw.Draw(sheet)
 for k,f in enumerate(ps[start:start+8]):
  x=(k%2)*960;y=(k//2)*570
  with Image.open(f) as im:assert im.size==(960,540);sheet.paste(im,(x,y+30))
  d.text((x+12,y+8),f'05_gallery sample {start+k+1}/165 - fps=1 preview extraction',fill='white')
 sheet.save(J/f'sheet-{start//8+1:02d}.jpg',quality=95)
(J/'extraction.json').write_text(json.dumps(dict(source_host='fmd',source='/data/projects/jmev-fmd-render-qualification-20261106-1111/jobs/fmd-raster-preview12/preview/05_gallery.mp4',source_sha256=h.hexdigest(),frames=9879,samples=165,sheets=math.ceil(len(ps)/8),scope='Early completed chapter only. Full preview guard still running; sampled narrated540p pictures, not full film, 4K, continuous motion, listening, fine sync or independent AV acceptance.',pictures_viewed=False),indent=2))
