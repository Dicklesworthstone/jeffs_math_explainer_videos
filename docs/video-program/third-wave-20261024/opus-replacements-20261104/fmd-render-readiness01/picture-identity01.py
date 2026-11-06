"""Guarded bounded cross-host picture qualification, silent lossless samples only."""
import hashlib,json,os,socket,subprocess,time
from pathlib import Path
HOST=socket.gethostname().split('.')[0]
ROOTS={'fixmydocuments':('/data/projects/jmev-fmd-render-qualification-20261106-1111','fmd-picture-identity01','production07d','work/probe-cwd'),'sensedemobox':('/data/projects/jmev-stroke-probe-20261106','fmd-picture-reference01','work/bin/production07d','work/production-repo07')}
r,j,b,c=ROOTS[HOST];ROOT=Path(r);JOB=ROOT/'jobs'/j;BIN=ROOT/b
assert hashlib.sha256(BIN.read_bytes()).hexdigest()=='2ccda773be0d6d9b6a83660a5481277d59b5ce2e6a4b80852d826388cad0cd0e'
receipt=json.loads((JOB/'resource-receipt.json').read_text());assert receipt['admitted'] and 'exit' not in receipt
assert receipt['policy']=={'admission_gib':10,'stop_free_gib':8,'phase_growth_gib':4,'work_cap_gib':56,'target_growth_cap_gib':20,'sample_seconds':10}
os.environ.update(TMPDIR=str(ROOT/'work/tmp'),XDG_CACHE_HOME=str(ROOT/'work/cache'))
results=[]
for name,size,chapter in [('preview','960x540','01_the_bump'),('four-k','3840x2160','01_the_bump')]:
 out=JOB/name;assert not out.exists()
 cmd=[str(BIN),'render',chapter,'--silent','--res',size,'--fps','1','--threads','4','--out',str(out),'--crf','0','--preset','veryfast']
 started=time.monotonic()
 with (JOB/(name+'.stdout')).open('xb') as so,(JOB/(name+'.stderr')).open('xb') as se:
  subprocess.run(cmd,cwd=ROOT/c,stdout=so,stderr=se,check=True,timeout=600)
 movie=out/(chapter+'.mp4');assert movie.is_file()
 probe=json.loads(subprocess.check_output(['/usr/bin/ffprobe','-v','error','-show_streams','-of','json',str(movie)],timeout=60))
 v=next(x for x in probe['streams'] if x['codec_type']=='video');w,h=map(int,size.split('x'));assert (v['width'],v['height'],v['avg_frame_rate'])==(w,h,'1/1')
 decoded=subprocess.check_output(['/usr/bin/ffmpeg','-v','error','-xerror','-i',str(movie),'-map','0:v:0','-f','framemd5','-'],timeout=180)
 with (JOB/(name+'.framemd5')).open('xb') as f:f.write(decoded)
 rows=[x for x in decoded.decode().splitlines() if x and not x.startswith('#')];assert len(rows)==int(v['nb_frames']) and len(rows)>1
 results.append({'name':name,'size':size,'chapter':chapter,'command':cmd,'frames':len(rows),'elapsed':time.monotonic()-started,'decoded_rows':rows,'movie_sha256':hashlib.sha256(movie.read_bytes()).hexdigest()})
with (JOB/'picture-identity-summary.json').open('x') as f:json.dump({'host':HOST,'tests':results,'scope':'Silent one-frame-per-second lossless chapter samples only. Cross-host equality requires both terminal receipts; no narrated timing, final encoding, full-rate picture, listening or release acceptance.'},f,indent=2)
print('Bounded picture samples and full decode complete',flush=True)
