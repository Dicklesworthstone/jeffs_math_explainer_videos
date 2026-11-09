"""Extract native 4K review frames from a completed, hash-bound chapter."""
from pathlib import Path
import argparse
import hashlib
import json
import re
import subprocess
from PIL import Image, ImageDraw

p=argparse.ArgumentParser()
p.add_argument('--source',type=Path,required=True)
p.add_argument('--job',type=Path,required=True)
p.add_argument('--chapter',required=True)
a=p.parse_args()
def sha(path):
    with path.open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest()
receipt=json.loads((a.job/'resource-receipt.json').read_text())
assert receipt['admitted'] and 'exit' not in receipt
source=json.loads((a.source/'resource-receipt.json').read_text())
assert source['exit']==0 and source['finished'] and source['inputs_sha256']==source['final_inputs_sha256']
summary=json.loads((a.source/'render-summary.json').read_text())
assert summary['chapter']==a.chapter
frames=summary['frames']
for row in summary['files']:
    local=a.source/Path(row['path']).name
    if local.exists():assert sha(local)==row['sha256']
layout=(a.source/(a.chapter+'.layout.tsv')).read_text().splitlines()
picked={f:['one-second sample'] for f in range(0,frames,60)}
picked.setdefault(frames-1,[]).append('last frame')
for finding in layout:
    if not finding.strip():continue
    match=re.search(r'@([0-9.]+)s',finding)
    assert match, 'Unrecognized layout anchor: '+finding
    anchor=float(match.group(1))
    times=[anchor,anchor+0.2,anchor+1,anchor+2]
    times.extend(anchor+5*i for i in range(1,int(frames/300)+1))
    for second in times:
        frame=round(second*60)
        if 0<=frame<frames:picked.setdefault(frame,[]).append('layout anchor/persistence: '+finding)
for line in (a.source/(a.chapter+'.timeline.tsv')).read_text().splitlines():
    row=line.split('\t')
    if row[0] in ('line','mark'):
        for delta in (0,30,60,120):
            frame=int(row[2])+delta
            if frame<frames:picked.setdefault(frame,[]).append(row[0]+':'+row[1])
def balanced(xs):
    if len(xs)==1:return xs[0]
    k=len(xs)//2
    return '('+balanced(xs[:k])+'+'+balanced(xs[k:])+')'
picked={f:['every frame 171.0 through 172.0 seconds; independent review219 predicted crossing'] for f in range(10260,10321)}
selected=sorted(picked)
filterfile=a.job/'select.filter'
filterfile.write_text("select='"+balanced(['eq(n,'+str(f)+')' for f in selected])+"'")
pictures=a.job/'pictures';pictures.mkdir()
movie=a.source/(a.chapter+'.mp4')
subprocess.run(['/opt/homebrew/bin/ffmpeg','-nostdin','-n','-v','error','-xerror','-threads','2','-i',str(movie),'-an','-filter_script:v',str(filterfile),'-vsync','0','-q:v','2',str(pictures/'frame-%06d.jpg')],check=True,timeout=600)
files=sorted(pictures.glob('frame-*.jpg'));assert len(files)==len(selected)
samples=[dict(frame=f,seconds=f/60,reasons=picked[f],path=str(path),sha256=sha(path)) for f,path in zip(selected,files)]
sheets=[]
for k in range(0,len(samples),8):
    canvas=Image.new('RGB',(1920,2260),'#10151d');draw=ImageDraw.Draw(canvas)
    for i,s in enumerate(samples[k:k+8]):
        im=Image.open(s['path']).convert('RGB');assert im.size==(3840,2160)
        x,y=(i%2)*960,(i//2)*565
        canvas.paste(im.resize((960,540),Image.Resampling.LANCZOS),(x,y))
        draw.text((x+5,y+542),f"{a.chapter} frame {s['frame']} {s['seconds']:.3f}s",fill='white')
    path=a.job/f'sheet-{k//8+1:04d}.jpg';canvas.save(path,quality=94)
    sheets.append(dict(path=str(path),sha256=sha(path)))
(a.job/'sampling.json').write_text(json.dumps(dict(movie_sha256=sha(movie),samples=samples,sheets=sheets,scope='Selected native4K chapter frames; not actual review, full master, listening, fine-sync or independentAV acceptance.'),indent=2)+'\n')
