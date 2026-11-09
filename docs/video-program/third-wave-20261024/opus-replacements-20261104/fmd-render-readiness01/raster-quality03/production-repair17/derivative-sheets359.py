"""Correct 358 sheet canvas, preserving every original and extracted frame."""
import json,hashlib
from pathlib import Path
from PIL import Image,ImageDraw
Q=Path(__file__).resolve().parent
def sha(p):
    with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest()
source=Q/'derivative-detail358/manifest.json';m=json.loads(source.read_text())
out=Q/'derivative-sheets359';out.mkdir(exist_ok=False)
rows=m['pictures'];sheets=[]
for row in rows:assert sha(row['path'])==row['sha256']
for start in range(0,len(rows),12):
    page=Image.new('RGB',(1440,1200),'#111318');draw=ImageDraw.Draw(page)
    for j,row in enumerate(rows[start:start+12]):
        im=Image.open(row['path']);im.thumbnail((480,270));x=(j%3)*480;y=(j//3)*300
        page.paste(im,(x,y));draw.text((x+8,y+274),f"frame {row['frame']} / {row['frame']/60:.3f}s",fill='white')
    p=out/f'sheet-{start//12+1:03d}.jpg';page.save(p,quality=95);sheets.append({'path':str(p),'sha256':sha(p)})
(out/'manifest.json').write_text(json.dumps({'source_manifest_sha256':sha(source),'movie_sha256':m['movie_sha256'],'intervals':m['intervals'],'pictures':rows,'sheets':sheets,'scope':'unreviewed corrected sheets, all original358 files retained'},indent=2)+'\n')
print(len(rows),len(sheets))
