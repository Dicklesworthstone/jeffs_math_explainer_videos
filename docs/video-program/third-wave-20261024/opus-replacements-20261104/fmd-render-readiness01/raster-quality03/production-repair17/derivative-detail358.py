"""Extract existing preview frames; no synthesis or scene rendering."""
import hashlib,json,subprocess
from pathlib import Path
from PIL import Image,ImageDraw
Q=Path(__file__).resolve().parent
movie=Q/'preview243-review265/preview/06_one_derivative.mp4'
def sha(p):
    with p.open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest()
assert sha(movie)=='5e45d72da91701c9c7c346b98ded8491c13ba1befed6be1b6301c1c6d33ed62d'
out=Q/'derivative-detail358';out.mkdir(exist_ok=False)
frames=list(range(9630,9731))+list(range(9970,10030))
expr='between(n,9630,9730)+between(n,9970,10029)'
subprocess.run(['/opt/homebrew/bin/ffmpeg','-v','error','-nostdin','-n','-threads','2','-i',str(movie),'-vf',f"select='{expr}'",'-fps_mode','vfr','-q:v','2',str(out/'frame-%04d.jpg')],check=True,timeout=300)
pics=sorted(out.glob('frame-*.jpg'));assert len(pics)==len(frames)
rows=[{'frame':n,'path':str(p),'sha256':sha(p)} for n,p in zip(frames,pics)]
sheets=[]
for start in range(0,len(rows),12):
    page=Image.new('RGB',(1440,900),'#111318');draw=ImageDraw.Draw(page)
    for j,row in enumerate(rows[start:start+12]):
        im=Image.open(row['path']);im.thumbnail((480,270));x=(j%3)*480;y=(j//3)*300
        page.paste(im,(x,y));draw.text((x+8,y+274),f"frame {row['frame']} / {row['frame']/60:.3f}s",fill='white')
    p=out/f'sheet-{start//12+1:03d}.jpg';page.save(p,quality=95);sheets.append({'path':str(p),'sha256':sha(p)})
assert sha(movie)=='5e45d72da91701c9c7c346b98ded8491c13ba1befed6be1b6301c1c6d33ed62d'
(out/'manifest.json').write_text(json.dumps({'movie':str(movie),'movie_sha256':sha(movie),'intervals':[[9630,9730],[9970,10029]],'pictures':rows,'sheets':sheets,'scope':'unreviewed consecutive encoded preview frames; not final4K or listening'},indent=2)+'\n')
print(json.dumps({'frames':len(rows),'sheets':len(sheets),'manifest_sha256':sha(out/'manifest.json')}))
