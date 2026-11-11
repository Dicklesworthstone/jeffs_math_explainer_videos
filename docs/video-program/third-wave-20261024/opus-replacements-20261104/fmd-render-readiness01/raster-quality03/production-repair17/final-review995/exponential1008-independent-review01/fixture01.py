"""CC2 fixture for pictures1008: does extracting a chapter file at local n give the same JPEG bytes as 1007's
extraction from the joined master at global offset+n, when the master was joined by explainer_kit master.rs join()
(video-only remux, concat, h264_metadata bt709 rewrite)? Local ffmpeg only; synthetic 4K inputs; writes only under
the run directory given as argv[1] (must not exist). argv[2] is the pictures1008 copy to import functions from."""
import hashlib,importlib.util,json,subprocess,sys
from pathlib import Path
from PIL import Image
FF,FP='/opt/homebrew/bin/ffmpeg','/opt/homebrew/bin/ffprobe'
run_dir=Path(sys.argv[1]);helper=Path(sys.argv[2]);run_dir.mkdir()
spec=importlib.util.spec_from_file_location('p1008',helper);m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m)
def sha(p):
 with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest()
def ff(args,log):
 r=subprocess.run([str(a) for a in args],capture_output=True,text=True);(run_dir/(log+'.log')).open('x').write(r.stderr)
 assert r.returncode==0,(log,r.stderr[-2000:]);return r.stdout
def local(argv):
 """The helper's argv with /usr/bin tools swapped for the local ones; everything else unchanged."""
 a=[str(x) for x in argv];assert a[0] in ('/usr/bin/ffmpeg','/usr/bin/ffprobe');a[0]={'/usr/bin/ffmpeg':FF,'/usr/bin/ffprobe':FP}[a[0]];return a
def probe(p):return json.loads(subprocess.run(local(m.probe_argv(p)),capture_output=True,text=True,check=True).stdout)['streams'][0]
def tags(s):return {k:s.get(k) for k in ('color_range','color_space','color_transfer','color_primaries','pix_fmt','profile','chroma_location')}
W='3840x2160'
# Chapters like native final-output/KEY.mp4: libx264 High yuv420p, range tv, matrix bt709, primaries/transfer unset, B-frames, AAC.
src={'ch01':(97,f'testsrc2=s={W}:r=60'),'ch02':(131,f'gradients=s={W}:r=60:n=5:speed=0.03:seed=7')}
chap={}
for k,(n,lav) in src.items():
 p=run_dir/(k+'.mp4')
 ff([FF,'-nostdin','-v','error','-f','lavfi','-i',lav,'-f','lavfi','-i','sine=frequency=440:sample_rate=48000','-frames:v',n,'-map','0:v','-map','1:a','-c:v','libx264','-preset','veryfast','-crf','16','-g','60','-bf','3','-pix_fmt','yuv420p','-colorspace','bt709','-color_range','tv','-c:a','aac','-shortest',p],'make-'+k)
 chap[k]=p
# master.rs join(): video-only remux per chapter, concat demuxer, h264_metadata bt709 rewrite, AAC from a wav.
parts=[]
for i,k in enumerate(src):
 part=run_dir/f'{i:02}.video.mp4';ff([FF,'-nostdin','-v','error','-i',chap[k],'-map','0:v:0','-c:v','copy','-an',part],'part-'+k);parts.append(part)
lst=run_dir/'chapters.txt';lst.open('x').write(''.join(f"file '{p.resolve()}'\n" for p in parts))
total=sum(n for n,_ in src.values());wav=run_dir/'master.wav'
ff([FF,'-nostdin','-v','error','-f','lavfi','-i',f'sine=frequency=330:sample_rate=48000:duration={total/60}','-ac','2',wav],'wav')
master=run_dir/'master.mp4'
ff([FF,'-nostdin','-v','error','-f','concat','-safe','0','-i',lst,'-i',wav,'-map','0:v:0','-map','1:a:0','-c:v','copy','-bsf:v','h264_metadata=colour_primaries=1:transfer_characteristics=1:matrix_coefficients=1','-c:a','aac','-b:a','256k','-ar','48000','-movflags','+faststart',master],'join')
res={'ffmpeg':subprocess.run([FF,'-hide_banner','-version'],capture_output=True,text=True).stdout.splitlines()[0],'helper_sha256':sha(helper),
 'tags':{'master':tags(probe(master)),**{k:tags(probe(p)) for k,p in chap.items()}}}
# verify1005 line 108's identity decode, then 1008's framehash_frames() and the slice check.
def fh(p,name):
 out=run_dir/(name+'.framehash')
 ff([FF,'-nostdin','-n','-v','error','-xerror','-threads','2','-i',p,'-map','0:v:0','-an','-c:v','rawvideo','-pix_fmt','yuv420p','-threads:v','2','-fps_mode','passthrough','-f','framehash','-hash','sha256',out],'fh-'+name)
 return m.framehash_frames(out)
mseq=fh(master,'master');assert len(mseq)==total
offsets={};off=0
for k,(n,_) in src.items():
 seq=fh(chap[k],k);assert len(seq)==n and seq==mseq[off:off+n],('identity',k);offsets[k]=off;off+=n
res['identity']={k:{'offset':offsets[k],'frames':src[k][0],'equal_to_master_slice':True} for k in src}
# 1008's stream_match() on these probes, as a chapter job would run it.
sm=run_dir/'stream-match-ch02';sm.mkdir()
for n,p in (('source',chap['ch02']),('master',master)):(sm/f'probe-{n}.stdout').open('x').write(subprocess.run(local(m.probe_argv(p)),capture_output=True,text=True,check=True).stdout)
try:m.stream_match(sm);res['stream_match_ch02']='passed'
except AssertionError as e:res['stream_match_ch02']='refused: '+repr(e.args[0])
# Picks: local frames inside each chapter (keyframe, B-frames, GOP boundary, last frame).
picks={'ch01':[0,1,2,48,59,60,96],'ch02':[0,1,2,3,4,5,30,59,60,61,62,100,130]}
def extract(name,source,filter_text,count):
 d=run_dir/name;d.mkdir();filt=run_dir/(name+'.filter');filt.open('x').write(filter_text)
 ff(local(m.extract_argv(source,filt,count,d)),'x-'+name);fs=sorted(d.glob('frame-*.jpg'));assert len(fs)==count,(name,len(fs));return fs
def rgb(p):
 with Image.open(p) as im:return hashlib.sha256(im.convert('RGB').tobytes()).hexdigest()
def compare(a,b):
 return {'jpeg_bytes_equal':[sha(x)==sha(y) for x,y in zip(a,b)].count(True),'decoded_rgb_equal':[rgb(x)==rgb(y) for x,y in zip(a,b)].count(True),'of':len(a)}
res['cases']={}
for k in src:
 es={'path':str(chap[k]),'offset_frames':offsets[k],'frames':src[k][0]}
 frames=[offsets[k]+l for l in picks[k]];loc=m.local_frames(frames,es);assert loc==picks[k]
 # 1007: verify1005 line 128 on the master, select over joined-master frame numbers (1007's filter text).
 a=extract(f'm1007-{k}',master,"select='"+m.balanced([f'eq(n\\,{n})' for n in frames])+"'\n",len(frames))
 # 1008 as drafted: the chapter file, select_filter(local).
 b=extract(f'c1008-{k}',chap[k],m.select_filter(loc),len(loc))
 res['cases'][f'{k} offset {offsets[k]}: 1008 chapter vs 1007 master']=compare(a,b)
 if k!='ch02':continue
 # Controls on ch02: copies with only SPS VUI changed (bitstream filter, -c copy), extracted with 1008's argv.
 for name,bsf in [('range-pc','video_full_range_flag=1'),('matrix-bt470bg','matrix_coefficients=5'),('prim-trc-bt2020-pq','colour_primaries=9:transfer_characteristics=16'),('prim-trc-bt709-like-master','colour_primaries=1:transfer_characteristics=1')]:
  cp=run_dir/f'ch02-{name}.mp4';ff([FF,'-nostdin','-v','error','-i',chap[k],'-map','0','-c','copy','-bsf:v','h264_metadata='+bsf,cp],'retag-'+name)
  assert fh(cp,'ch02-'+name)==mseq[offsets[k]:offsets[k]+src[k][0]],('retag changed decoded planes',name)
  c=extract(f'c1008-{k}-{name}',cp,m.select_filter(loc),len(loc))
  res['cases'][f'{k} control {name} {tags(probe(cp))}: vs 1007 master']=compare(a,c)
 # Explicit matched conversion in the filter script: tag the chapter frames as the master's before select.
 d=extract(f'c1008-{k}-setparams',chap[k],'setparams=color_primaries=bt709:color_trc=bt709:colorspace=bt709:range=tv,'+m.select_filter(loc),len(loc))
 res['cases'][f'{k} setparams bt709 prefix: vs 1007 master']=compare(a,d)
(run_dir/'fixture01-result.json').open('x').write(json.dumps(res,indent=1)+'\n')
print(json.dumps(res,indent=1))
