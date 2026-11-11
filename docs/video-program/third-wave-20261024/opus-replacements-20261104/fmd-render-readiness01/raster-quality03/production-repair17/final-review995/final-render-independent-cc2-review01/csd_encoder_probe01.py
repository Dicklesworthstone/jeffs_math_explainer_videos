# Read-only on CSD: encoder state of master993's staged finals and of the combined/counting chain movies, plus the
# FrankenManim pin of each build workspace. Reads headers and the first 16 MiB of each movie; writes nothing.
import hashlib,json,os,subprocess
from pathlib import Path
D=Path('/data/projects/jmev-stroke-probe-20261106')
def x264(p):
 with open(p,'rb') as f:head=f.read(16*1024*1024)
 i=head.find(b'x264 - core');return head[i:i+5000].split(b'\0',1)[0].decode('utf8','replace') if i>=0 else None
def probe(p):
 r=subprocess.run(['/usr/bin/ffprobe','-v','error','-select_streams','v:0','-show_data_hash','SHA256','-show_entries','stream=codec_name,profile,level,width,height,pix_fmt,color_range,color_space,color_transfer,color_primaries,chroma_location,refs,has_b_frames,r_frame_rate,nb_frames,extradata_size,extradata_hash','-of','json',str(p)],capture_output=True,text=True,timeout=120)
 s=json.loads(r.stdout or '{}').get('streams') or [{}];return s[0]
def sha(p):
 with open(p,'rb') as f:return hashlib.file_digest(f,'sha256').hexdigest()
out={'jobs':sorted(x for x in os.listdir(D/'jobs') if 'combined' in x or 'counting' in x or '968' in x or '993' in x),'movies':{},'pins':{}}
cands=[]
fin=D/'jobs/csd-combined-master993/final'
if fin.is_dir():cands+=sorted(fin.glob('*.mp4'))
for j in out['jobs']:
 if 'preview' in j or 'final968' in j:
  for root,dirs,files in os.walk(D/'jobs'/j):
   if root.count('/')-str(D/'jobs'/j).count('/')>3:dirs[:]=[];continue
   cands+=[Path(root)/f for f in files if f.endswith('.mp4')]
for p in cands:
 try:
  st=os.stat(p);out['movies'][str(p)]={'size':st.st_size,'ino':st.st_ino,'x264':x264(p),'stream':probe(p)}
 except Exception as e:out['movies'][str(p)]={'error':repr(e)}
for w in ['work/native-combined01','work/native-counting02','work/production-repo07']:
 for n in ['FRANKEN_MANIM.pin','Cargo.lock','rust-toolchain.toml','workspace/FRANKEN_MANIM.pin','workspace/Cargo.lock','workspace/rust-toolchain.toml']:
  p=D/w/n
  if p.is_file():out['pins'][str(p)]=sha(p)
 out['pins'][w+' exists']=(D/w).is_dir()
print(json.dumps(out))
