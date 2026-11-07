import json,hashlib,math
from pathlib import Path
from PIL import Image,ImageDraw
R=Path('/data/projects/jmev-stroke-probe-20261106');J=R/'jobs/csd-continuous-sheets16'
def sha(p):
 with p.open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest()
a=json.loads((J/'resource-receipt.json').read_text());assert a['admitted'] and 'exit' not in a
result=[]
for chapter in ['04_reading_color','05_gallery']:
 source=R/'jobs'/('csd-continuous-detail16-'+chapter);d=json.loads((source/'detail-summary.json').read_text());ps=d['pictures'];assert len(ps)==d['frames'];assert all(not p['viewed'] for p in ps)
 out=J/chapter;out.mkdir();sheets=[]
 for start in range(0,len(ps),8):
  sheet=Image.new('RGB',(1920,2280),(16,16,20));pen=ImageDraw.Draw(sheet)
  for k,p in enumerate(ps[start:start+8]):
   path=Path(p['path']);assert sha(path)==p['sha256'];x=k%2*960;y=k//2*570
   with Image.open(path) as im:
    assert im.size==(3840,2160);sheet.paste(im.resize((960,540),Image.Resampling.LANCZOS),(x,y+30))
   pen.text((x+10,y+8),f"{chapter} frame {p['frame']} / {d['frames']}  native4K silent1fps",fill='white')
  path=out/f'sheet-{start//8+1:02d}.jpg';assert not path.exists();sheet.save(path,quality=95);sheets.append(dict(path=str(path),sha256=sha(path),frames=[p['frame'] for p in ps[start:start+8]]))
 result.append(dict(chapter=chapter,source_summary_sha256=sha(source/'detail-summary.json'),frames=d['frames'],sheets=sheets,viewed=False))
(J/'sheets-summary.json').write_text(json.dumps(dict(chapters=result,scope='Contact sheets only. Actual viewing, native detail and60fps motion still required.'),indent=2))
print('Contact sheets extracted; no visual acceptance.')
