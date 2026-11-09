"""Guarded final technical verification. No listening, motion or independent AV acceptance."""
import hashlib,importlib.util,json,re,signal,sys,wave
from pathlib import Path
P=Path(__file__).resolve().parent
R=Path('/Users/jemanuel/projects/jeffs_math_explainer_videos')
B=R/'narration_lab/third-wave-20261024/root/five-film-release-20261027'
MASTER=B/'apportionment-master619'
JOB=B/'apportionment-verify623'
INPUTS=P/'verify623-inputs.json'
SCRIPT=R/'explainer_corpus/apportionment/script.tsv'
def sha(p):
 with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest()
def read(p):return json.loads(Path(p).read_text())
def stamp(s):
 assert re.fullmatch(r'\d{2,}:\d{2}:\d{2}[.,]\d{3}',s),s
 h,m,t=s.replace(',','.').split(':');sec,ms=t.split('.')
 assert int(m)<60 and int(sec)<60
 return ((int(h)*60+int(m))*60+int(sec))*1000+int(ms)
def captions(p,frames,vtt=False):
 blocks=Path(p).read_text().strip().split('\n\n');out=[]
 if vtt:assert blocks.pop(0)=='WEBVTT'
 for i,block in enumerate(blocks,1):
  lines=block.splitlines()
  if '-->' not in lines[0]:assert lines.pop(0)==str(i)
  a,z=map(stamp,lines.pop(0).split(' --> '))
  assert 0<=a<z<=round(frames*1000/60)
  assert 1<=len(lines)<=2 and all(0<len(x)<=42 for x in lines)
  assert not out or a>=out[-1]['end_ms']
  out.append(dict(start_ms=a,end_ms=z,text='\n'.join(lines)))
 assert out
 return out
def frame_digests(p):
 result=[]
 for line in Path(p).read_text().splitlines():
  if not line or line.startswith('#'):continue
  row=[x.strip() for x in line.split(',')]
  assert len(row)==6 and row[0]=='0' and int(row[3])==1 and int(row[4])>0,row
  assert re.fullmatch('[0-9a-f]{64}',row[5]),row
  result.append((int(row[4]),row[5]))
 assert result
 return result
def main():
 support=P.parent/'root-apportionment-final-support613.py'
 assert sha(support)=='dfa086950e9b771e03d0feb59f755348a9df5b4fd40519ba07ea7936df4133b0'
 spec=importlib.util.spec_from_file_location('owned623',support);h=importlib.util.module_from_spec(spec);spec.loader.exec_module(h)
 h.own_group();h.STATE['job']=JOB
 for sig in h.STOP_SIGNALS:signal.signal(sig,h.on_stop)
 state=read(JOB/'resource-receipt.json')
 assert state.get('admitted') and not state.get('finished') and state['policy']==h.POLICY
 assert str(Path(__file__).resolve()) in state['command']
 assert sorted(p.name for p in JOB.iterdir())==['command.log','resource-receipt.json']
 assert sha(INPUTS)==state['inputs_sha256']
 rows=read(INPUTS)['files'];bound={x['path']:x['sha256'] for x in rows};assert len(bound)==len(rows)
 for path,digest in bound.items():assert sha(path)==digest,path
 h.success(MASTER/'resource-receipt.json')
 summary=read(MASTER/'master619-summary.json');material=read(MASTER/'materialized619.json')
 assert summary['frames']==material['frames']==100886
 chapters=summary['chapters'];assert len(chapters)==len(set(chapters))==15 and set(chapters)==set(material['chapters'])
 out=MASTER/'final';movie=out/'apportionment_master.mp4'
 assert str(movie)==summary['movie'] and sha(movie)==summary['movie_sha256']
 script=dict(line.split('\t',1) for line in SCRIPT.read_text().splitlines() if line and not line.startswith('#'))
 assert len(script)==93
 placements=[];offset=0
 for ch in chapters:
  lines=[line.split('\t') for line in (out/(ch+'.timeline.tsv')).read_text().splitlines() if line and not line.startswith('#')]
  meta={r[0]:r[1] for r in lines if len(r)==2};frames=int(meta['frames']);assert int(meta['fps'])==60 and meta['chapter']==ch
  assert frames==material['chapters'][ch]['frames'];previous=0
  for row in lines:
   if row[0]!='line':continue
   _,cue,start,gain,source=row;start=int(start)
   assert source in bound and sha(source)==bound[source]
   with wave.open(source) as w:
    assert (w.getnchannels(),w.getsampwidth(),w.getframerate())==(1,2,24000)
    samples=w.getnframes()
   end=start/60+samples/24000
   assert start/60>=previous-1e-6 and end<=frames/60+1e-6
   previous=end;placements.append(dict(cue=cue,chapter=ch,frame=offset+start,samples=samples,gain_db=float(gain),source=source))
  offset+=frames
 assert offset==100886 and [x['cue'] for x in placements]==list(script)
 caps=captions(out/'apportionment.en.srt',offset)
 assert caps==captions(out/'apportionment.en.vtt',offset,True)
 assert ' '.join(' '.join(c['text'].split()) for c in caps)==' '.join(' '.join(t.split()) for t in script.values())
 fast=[dict(index=i+1,**c) for i,c in enumerate(caps) if len(c['text'].replace('\n',' '))*1000/(c['end_ms']-c['start_ms'])>20]
 assert not fast,('fast captions require repair',fast)
 def run(name,args,limit=14400):
  code,tracking=h.run_owned(name,args,limit);assert code==0,(name,code)
  print('VERIFIED_STEP',name,tracking,flush=True)
 probe=JOB/'probe.json'
 run('probe',[h.FFPROBE,'-v','error','-show_streams','-show_format','-of','json','-o',str(probe),str(movie)],120)
 streams=read(probe)['streams'];assert len(streams)==2
 v=next(s for s in streams if s['codec_type']=='video');a=next(s for s in streams if s['codec_type']=='audio')
 assert (v['codec_name'],v['width'],v['height'],v['r_frame_rate'],int(v['nb_frames']),v['pix_fmt'])==('h264',3840,2160,'60/1',offset,'yuv420p')
 assert all(v[k]=='bt709' for k in ('color_space','color_transfer','color_primaries'))
 assert a['codec_name']=='aac' and int(a['sample_rate'])==48000 and a['channels']==2
 assert abs(float(v['duration'])-offset/60)<.002 and abs(float(a['duration'])-offset/60)<.03
 final_hash=JOB/'master-frames.sha256'
 run('full-av-decode-and-video-hash',[h.FFMPEG,'-nostdin','-v','error','-xerror','-threads','4','-i',str(movie),'-map','0:v:0','-fps_mode','passthrough','-f','framehash','-hash','sha256',str(final_hash),'-map','0:a:0','-f','null','-'])
 expected=[]
 for ch in chapters:
  target=JOB/(ch+'-frames.sha256')
  run('chapter-identity-'+ch,[h.FFMPEG,'-nostdin','-v','error','-xerror','-threads','4','-i',str(out/(ch+'.mp4')),'-map','0:v:0','-fps_mode','passthrough','-f','framehash','-hash','sha256',str(target)])
  digests=frame_digests(target);assert len(digests)==material['chapters'][ch]['frames'];expected.extend(digests)
 actual=frame_digests(final_hash);assert len(actual)==offset and actual==expected,'Chapter-to-master decoded video differs'
 run('encoded-loudness',[h.FFMPEG,'-nostdin','-hide_banner','-nostats','-threads','4','-i',str(movie),'-map','0:a:0','-af','ebur128=peak=true','-f','null','-'])
 log=(JOB/'command.log').read_text();match=list(re.finditer(r'Integrated loudness:\s*I:\s*(-?[\d.]+) LUFS.*?True peak:\s*Peak:\s*(-?[\d.]+) dBFS',log,re.S));assert match,'No final ebur128 summary'
 loud,peak=map(float,match[-1].groups());assert peak<=-1.0 and -30<loud<=-15
 nominal_exception=loud < -17
 if nominal_exception:assert -1.7<=peak<=-1.0,('Quieter programme without ceiling binding',loud,peak)
 for path,digest in bound.items():assert sha(path)==digest,path
 result=dict(frames=offset,seconds=offset/60,placements=placements,captions=len(caps),srt_vtt_equal=True,caption_words_exact=True,fast_captions=fast,full_av_decode=True,all_decoded_video_frames_match_chapters=True,lufs=loud,true_peak_dbfs=peak,nominal_loudness_exception=nominal_exception,movie_sha256=sha(movie),pictures_viewed=False,listening=False,fine_sync=False,independent_av_acceptance=False)
 with (JOB/'verification623.json').open('x') as f:json.dump(result,f,indent=2)
 print('TECHNICAL623_COMPLETE; actual picture/motion/listening/fine-sync/independentAV review remains',flush=True)
if __name__=='__main__':main()
