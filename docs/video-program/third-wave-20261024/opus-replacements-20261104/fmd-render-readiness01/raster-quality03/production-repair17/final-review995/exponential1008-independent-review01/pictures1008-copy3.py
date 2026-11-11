"""Candidate picture-phase successor 1008 to pictures1007 (cf3e9f16); scratch only, no launch.

Root asked (2026-11-10 23:17Z) for one bounded change to 1007: decode each chapter file once, instead of decoding the
joined master from frame 0 for every chapter. 1007's serial lane decodes 1,280,843 frames (7.79 master lengths, about
18.6 h at the 19.16 fps verify1005 measured on CSS); 1008 decodes each chapter's own frames once, 164,420 in all.

The change:
- A chapter job reads OUT/KEY.mp4, the chapter file verify1005 bound and decoded, and selects local indices (the
  joined-master frame minus the chapter offset). The argv is verify1005 line 128's with that input and filter.
- Why the pictures are the master's: terminal verification1005 decoded the master and every chapter file to yuv420p
  framehash and required each chapter's sequence to equal the master slice at its offset. 1008 does not take the
  booleans on trust. chapter_identity() requires, for the chapter:
  - exactly one chapter_frame_identity record, naming OUT/KEY.mp4 at the chapter's offset and frame count;
  - the chapter file's sha256 equal in the verify1005 manifest row, in that record and on disk;
  - VERIFY/KEY.framehash and VERIFY/master-video.framehash present as plain files, at the sha256 that
    verification1005.json records (root pins verification1005.json itself by hash);
  - verify1005's own frame_sequence() (lines 95-102, unchanged) on both, and chapter == master[offset:offset+frames].
  A missing or mismatched record, file, hash or slice refuses. gather() runs it at make-inputs and at run, and for
  all 15 chapters in the recheck.
- Colour and conversion: framehash compares decoded planes, not the stream's range and colour tags, which steer the
  JPEG conversion. Before extracting, the job runs ffprobe (verify1005's /usr/bin/ffprobe, bound) on the chapter file
  and the master. stream_match() requires one video stream each, with equal STREAM_KEYS (codec, profile, pix_fmt,
  colour range, colour space (matrix), chroma location, size, rate, field order, bit depth) and yuv420p at 3840x2160.
  The transfer and primaries tags (TAG_KEYS) must each be unset in the chapter file or equal to the master's.
  The final983 probes on CSS record the chapters with them unset and the master with bt709, with all of STREAM_KEYS
  equal. In the local fixture, which mirrors that, the JPEGs are byte-identical. The record keeps the matched values
  and both tag sets; the recheck recomputes them from the job's probe outputs.
- Global metadata is kept: selection-plan.json, pictures1008.json and the sheet labels carry the joined-master frame
  numbers, the reasons and the master sha256 exactly as 1007 does. Each sample adds local_frame; the plan and the
  record add extract_source (chapter file, sha256, offset, frames and both framehash sha256s).
- Names: css-exponential-master-pictures1008-KEY writes pictures1008.json; css-exponential-pictures1008-recheck writes
  recheck1008.json. 1008 refuses a chapter with any pictures1005, 1006 or 1007 job: no mixed packaging.

Unchanged from 1007:
- the verification pin, and every mode refusing until it is set;
- the consumed-closure binding against the verify1005 manifest (the chapter file joins it; the master stays in it);
- selection() (verify1005 lines 60-67), the job estimate and the 3.5 GiB / 8 GiB checks;
- the guard receipt and run setup, the sheet layout and labels, the size cap and the frozen recheck;
- the packaging recheck's full verify1005 manifest rehash and its exact record, plan, filter, picture and sheet checks.

Scope: technical extraction only. Images unviewed; no listening, fine sync or publication acceptance.
"""
import argparse,hashlib,importlib.util,json,math,os,re,shutil,signal,sys
from pathlib import Path
from PIL import Image,ImageDraw
import PIL
D=Path('/data/projects/jmev-css-render-worker-20261107');Q=D/'qualification01/exponential933';REPO=D/'work/exponential959/repo'
MASTER=D/'jobs/css-exponential-master987';OUT=MASTER/'final';VERIFY=D/'jobs/css-exponential-verify1005';VM=Q/'css-exponential-verify1005-inputs.json'
FFMPEG=Path('/usr/bin/ffmpeg');FFPROBE=Path('/usr/bin/ffprobe')
STREAM_KEYS=('codec_name','profile','pix_fmt','color_range','color_space','chroma_location','width','height','r_frame_rate','field_order','bits_per_raw_sample')
TAG_KEYS=('color_transfer','color_primaries')
VERIFY_HELPER_SHA='d96795c42ac8826aa7a00e568f307e473cd05a529848f85a7b653ce75e7232c1'
# The actual verify1005 receipt: command and inputs_sha256, read on CSS at admission (2026-11-10T19:05:04Z).
VERIFY_COMMAND=['/usr/bin/python3','-I','-B',str(Q/'verify1005.py'),'run','--phase','verify']
VERIFY_INPUTS_SHA='5d45eac1c04c351ffdba66ca1ee97a88dc8536978a94e0a4928dc1c51189aa41'
# Root sets this to sha256(VERIFY/'verification1005.json') only after the verify job is terminal and stable.
VERIFICATION_SHA256=None
MOVIE_SHA='fe33b9e48bcb456b3cad41f7651bd8e5f8e3bb1b7c6deba657e6b4d9e5fa2b80'
DELIVERY_SRT_SHA='127f3d1ade016f125dfddff861b9614fd2850387be6e4cd9e05063e15c1d9f99'
DELIVERY_VTT_SHA='0d2698d0c386263e9b946a5a395423b998c70b08c96cb69d21649810fa8114d2'
# The last three are the files guard_css74 4f7edaba loads and pins itself (its PINS and owned-process-stop02).
TOOL_PINS={Q/'verify1005.py':VERIFY_HELPER_SHA,D/'guard_css74.py':'4f7edaba7963af580245a6809af0386becba8056daec5d65626cc0ff1ef8ce73',
 Q/'build959.py':'5fc0d94fcf65a4384253c23d5709aa85ba948f94771c70bd5dee47f49ec6fd83',
 Q/'css-native-build933-transfer.json':'8dfdc8e27baec1a1c288ee547af9a1cb1e3b613adb3f55e1aab263b09a410f67',
 D/'inputs/exponential933/root-exponential804-build.py':'18c201de7a0bf33a2afaf1955c31acb77a93655cd8e6c4b279fd431caba46db8',
 D/'inputs/repo/docs/video-program/third-wave-20261024/opus-replacements-20261104/engine-secondary-validation01/guard_owner56g_target20g.py':'1c19a9b8f817c3bca06e18fd0785ac21c3cb935be715d67ecc41e4e39d1f5097',
 D/'inputs/repo/narration_lab/production-20261020/second-wave-central/duration-and-storage-20261020/root/run_bounded_phase_oliver_owner48g_target14g_20261102.py':'819746fb33110d6c133e184812c80a00562d848816f623078ad86f0fb1a316da',
 D/'qualification01/owned-process-stop02.py':'a2fe7985bf76a1982e8d08b3dfb23354d2f60b322b95ed82ca83fc5e39a09236'}
# 1008's record scope: verify1005 line 137's wording no longer fits, since the pictures come from the chapter files.
SCOPE1008='All selected985 frames at their joined-master frame numbers, extracted from each chapter file whose decoded frames verification1005 proved identical to that master slice (re-proved here from the bound framehash files); images unviewed. No listening, fine-sync or publication acceptance.'
def sha(p):
 with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest()
def read(p):return json.loads(Path(p).read_text())
def write(p,v):
 with Path(p).open('x') as f:json.dump(v,f,indent=2)
def module(n,p):
 s=importlib.util.spec_from_file_location(n,p);m=importlib.util.module_from_spec(s);s.loader.exec_module(m);return m
def stable(job):
 r=read(job/'resource-receipt.json');assert r.get('admitted') is True and r.get('finished') and r.get('exit')==0 and not any(r.get(k) for k in ('resource_stop','error','stopped_for_resource_limit')) and r['inputs_sha256']==r['final_inputs_sha256'];return r
def balanced(xs):
 assert xs
 if len(xs)==1:return xs[0]
 n=len(xs)//2;return '('+balanced(xs[:n])+'+'+balanced(xs[n:])+')'
def selection(key,record,count,offset):
 """One chapter's frame -> reasons union: verify1005 lines 60-67 unchanged, inside its own loop header over this chapter."""
 selected={}
 for key in [key]:
  assert len(record['chapters'])==1;c=record['chapters'][0];assert c['chapter']==key and c['frames']==count and c['full_av_decode'] is True
  selected[key]={}
  for x in c['pictures']:
   assert 0<=x['frame']<count
   f=offset+x['frame'];reasons=selected[key].setdefault(f,[])
   for reason in x['reasons']:
    if reason not in reasons:reasons.append(reason)
  assert selected[key]
 return selected[key]
def seed_path(key):return D/'jobs'/('css-exponential-pictures985-'+key)/'pictures985.json'
def framehash_frames(path):
 """verify1005 lines 95-102 unchanged, at their own indentation."""
 if True:
  def frame_sequence(path):
   seq=[]
   for line in path.read_text().splitlines():
    if not line or line.startswith('#'):continue
    fields=[x.strip() for x in line.split(',')];assert len(fields)==6 and fields[0]=='0'
    assert int(fields[4])==3840*2160*3//2 and re.fullmatch('[0-9a-f]{64}',fields[5])
    seq.append((int(fields[4]),fields[5]))
   return seq
 return frame_sequence(path)
def chapter_identity(key,vc,v,vrows):
 """verification1005's chapter-frame identity for one chapter, re-proved from the bound framehash files.
 Returns the chapter file, its extract_source binding and the two framehash paths."""
 source=OUT/(key+'.mp4');hits=[x for x in v['chapter_frame_identity'] if x.get('chapter')==key];assert len(hits)==1,('need exactly one chapter_frame_identity record',key)
 x=hits[0];assert x.get('source')==str(source) and x.get('offset_frames')==vc['offset_frames'] and x.get('frames')==vc['frames'],('identity record is not this chapter file at its offset',key)
 assert not source.is_symlink() and vrows.get(str(source))==x.get('source_sha256')==sha(source),('chapter file differs from the one verify1005 bound',key)
 fh,mh=VERIFY/(key+'.framehash'),VERIFY/'master-video.framehash'
 for p,digest in [(fh,x.get('framehash_sha256')),(mh,v.get('master_video_framehash_sha256'))]:assert p.is_file() and not p.is_symlink() and sha(p)==digest,('framehash missing or differs from verification1005',str(p))
 seq,master=framehash_frames(fh),framehash_frames(mh);start=vc['offset_frames']
 assert len(master)==v['frames'] and len(seq)==vc['frames'] and seq==master[start:start+vc['frames']],('Decoded frame mismatch',key)
 es={'path':str(source),'sha256':x['source_sha256'],'offset_frames':start,'frames':vc['frames'],'framehash_sha256':x['framehash_sha256'],'master_video_framehash_sha256':v['master_video_framehash_sha256']}
 return source,es,{fh,mh}
def local_frames(frames,es):
 """Joined-master frame -> chapter-file frame (frame - offset). Every frame must fall inside the chapter."""
 start=es['offset_frames'];assert all(start<=f<start+es['frames'] for f in frames),('frame outside the chapter',es['path']);return [f-start for f in frames]
def select_filter(local):return "select='"+balanced([f'eq(n\\,{n})' for n in local])+"'\n"
def extract_argv(source,filt,count,dest):
 """verify1005 line 128's argv with the chapter file as input; select.filter carries local indices."""
 return [str(FFMPEG),'-nostdin','-n','-v','error','-xerror','-threads','2','-i',source,'-an','-filter_script:v',filt,'-fps_mode','passthrough','-frames:v',str(count),'-q:v','2',dest/'frame-%06d.jpg']
def probe_argv(path):return [str(FFPROBE),'-v','error','-select_streams','v:0','-show_streams','-of','json',path]
def stream_match(job):
 """The chapter file's video stream carries the master's decode and colour metadata, so the JPEG conversion matches."""
 a,b=(read(job/f'probe-{n}.stdout').get('streams') for n in ('source','master'));assert isinstance(a,list) and isinstance(b,list) and len(a)==len(b)==1,('need exactly one video stream in each probe',str(job))
 fa,fb=({k:s[0].get(k) for k in STREAM_KEYS} for s in (a,b));assert fa==fb,('chapter file stream metadata differs from the master',sorted(k for k in STREAM_KEYS if fa[k]!=fb[k]))
 assert fb['pix_fmt']=='yuv420p' and fb['width']==3840 and fb['height']==2160,('master stream is not 3840x2160 yuv420p',fb)
 ta,tb=({k:s[0].get(k) for k in TAG_KEYS} for s in (a,b));assert all(ta[k] in (None,tb[k]) for k in TAG_KEYS),('chapter file transfer or primaries tag differs from the master',ta,tb)
 return {'matched':fb,'tags':{'source':ta,'master':tb}}
def pictures(job,key,plan,vc,source,es,movie,movie_sha,run_step):
  """1007's extraction and sheets for the chapter file: local select; global frame numbers in the plan, labels and record."""
  frames=sorted(plan);local=local_frames(frames,es)
  run_step('probe-source',probe_argv(source));run_step('probe-master',probe_argv(movie));stream=stream_match(job)
  write(job/'selection-plan.json',{'chapter':key,'master_sha256':movie_sha,'samples':[{'frame':f,'local_frame':l,'reasons':plan[f]} for f,l in zip(frames,local)],'source':vc,'extract_source':es})
  dest=job/'pictures';dest.mkdir();filt=job/'select.filter'
  with filt.open('x') as f:f.write(select_filter(local))
  run_step('extract',extract_argv(source,filt,len(frames),dest))
  files=sorted(dest.glob('frame-*.jpg'));assert len(files)==len(frames);sd=job/'sheets';sd.mkdir();sheets=[]
  for start in range(0,len(files),8):
   sheet=Image.new('RGB',(1920,2280),'#222222');draw=ImageDraw.Draw(sheet)
   for slot,p in enumerate(files[start:start+8]):
    n=start+slot;x,y=(slot%2)*960,(slot//2)*570
    with Image.open(p) as im:assert im.size==(3840,2160);sheet.paste(im.resize((960,540),Image.Resampling.LANCZOS),(x,y+30))
    draw.text((x+7,y+8),f'{key} | masterframe{frames[n]} | {frames[n]/60:.3f}s | '+plan[frames[n]][0][:70],fill='white')
   p=sd/f'sheet-{start//8+1:03}.jpg';sheet.save(p,quality=95);sheets.append({'path':str(p),'sha256':sha(p),'frames':frames[start:start+8]})
  return {'chapter':key,'movie_sha256':movie_sha,'pictures':[{'path':str(p),'sha256':sha(p),'frame':f,'local_frame':l,'reasons':plan[f]} for p,f,l in zip(files,frames,local)],'sheets':sheets,'samples':len(frames),'source':vc,'extract_source':es,'stream':stream}
def gather(key):
 """Pins, the verified run, the consumed closure, the re-proved chapter identity and (for a chapter) the selection."""
 assert isinstance(VERIFICATION_SHA256,str) and re.fullmatch('[0-9a-f]{64}',VERIFICATION_SHA256),'refused: root has not pinned the terminal verification1005.json sha256'
 q=read(D/'jobs/css-exponential-build959/qualification.json');keys=q['chapters'];assert len(keys)==15
 if key is not None:assert key in keys
 vr=stable(VERIFY);assert vr['command']==VERIFY_COMMAND and vr['inputs_sha256']==VERIFY_INPUTS_SHA and sha(VM)==vr['inputs_sha256']
 for p,digest in TOOL_PINS.items():assert sha(p)==digest,p
 vrows={x['path']:x['sha256'] for x in read(VM)['files']}
 vp=VERIFY/'verification1005.json';assert sha(vp)==VERIFICATION_SHA256,'verification1005.json differs from the root pin';v=read(vp)
 movie=OUT/'exponential_magic_master.mp4'
 assert v['movie']==str(movie) and v['movie_sha256']==MOVIE_SHA and v['frames']==164420 and v['full_av_decode'] is True and v['all_decoded_video_frames_match_chapters'] is True
 assert v['fast_captions']==[] and v['delivery_srt_sha256']==DELIVERY_SRT_SHA and v['delivery_vtt_sha256']==DELIVERY_VTT_SHA
 chapters=v['chapters'];assert [c['chapter'] for c in chapters]==keys;offset=0
 for c in chapters:assert c['offset_frames']==offset and c['frames']>0 and c['samples']>0 and c['seed_path']==str(seed_path(c['chapter']));offset+=c['frames']
 assert offset==164420 and [(x['chapter'],x['offset_frames'],x['frames']) for x in v['chapter_frame_identity']]==[(c['chapter'],c['offset_frames'],c['frames']) for c in chapters]
 r=stable(MASTER);mf=Q/'css-exponential-master987-inputs.json';assert sha(mf)==r['inputs_sha256']
 summary=read(MASTER/'master987-summary.json');assert summary['movie']==str(movie) and summary['movie_sha256']==MOVIE_SHA and summary['chapters']==keys
 assert vrows[str(movie)]==MOVIE_SHA and not movie.is_symlink() and sha(movie)==MOVIE_SHA
 consumed={Path(sys.executable).resolve(),FFMPEG,FFPROBE,*TOOL_PINS,movie,D/'jobs/css-exponential-build959/qualification.json',MASTER/'resource-receipt.json',MASTER/'master987-summary.json',mf}
 for root in [Path(PIL.__file__).parent]:
  for p in root.rglob('*'):
   if '__pycache__' in p.parts:continue
   assert not p.is_symlink()
   if p.is_file():consumed.add(p)
 selected={}
 if key is not None:
  for n in (1005,1006,1007):assert not (D/'jobs'/(f'css-exponential-master-pictures{n}-'+key)).exists(),(f'chapter has a pictures{n} job; 1008 does not mix',key)
  vc=chapters[keys.index(key)];pr=Path(vc['seed_path']);pj=pr.parent;assert pj==D/'jobs'/('css-exponential-pictures985-'+key) and pr.name=='pictures985.json'
  assert vrows[str(pr)]==vc['seed_sha256']==sha(pr);stable(pj);consumed|={pr,pj/'resource-receipt.json'}
  record=read(pr);count=vc['frames'];offset=vc['offset_frames']
  selected[key]=selection(key,record,count,offset)
  assert len(selected[key])==vc['samples'] and all(offset<=f<offset+count for f in selected[key])
  ident={key:chapter_identity(key,vc,v,vrows)}
 else:
  # The packaging recheck re-derives every chapter's union from its bound seed and re-proves every chapter's identity.
  consumed|={Path(c['seed_path']) for c in chapters}
  ident={c['chapter']:chapter_identity(c['chapter'],c,v,vrows) for c in chapters}
 consumed|={source for source,_,_ in ident.values()}
 for p in consumed:assert vrows.get(str(p))==sha(p),('consumed file not bound with this hash by verify1005',str(p))
 paths=consumed|{Path(__file__).resolve(),VERIFY/'resource-receipt.json',VM,vp}|{p for _,_,proof in ident.values() for p in proof}
 return keys,chapters,vr,movie,paths,selected,ident
def finished(keys,chapters,ident):
 """Recheck: every chapter has exactly one stable pictures1008 job and no pictures1005/1006/1007 job, with this helper's
 exact command and an inputs file binding this helper, the verification pin, the movie, the seed, the chapter file and
 both framehash files."""
 jobs=[];helper=Path(__file__).resolve();made_sha=sha(helper)
 for key,vc in zip(keys,chapters):
  for n in (1005,1006,1007):assert not (D/'jobs'/(f'css-exponential-master-pictures{n}-'+key)).exists(),(f'a pictures{n} job exists; 1008 does not mix',key)
  j=D/'jobs'/('css-exponential-master-pictures1008-'+key);rec=j/'pictures1008.json'
  assert j.exists() and not j.is_symlink(),('need the chapter pictures1008 job, not a symlink',key)
  r=stable(j);m=Q/(j.name+'-inputs.json');assert sha(m)==r['inputs_sha256']
  assert r['command']==['/usr/bin/python3','-I','-B',str(helper),'run','--phase','pictures','--chapter',key],('picture job command is not this helper run --phase pictures --chapter KEY',key,r['command'])
  rows={x['path']:x['sha256'] for x in read(m)['files']}
  assert rows.get(str(helper))==made_sha,('picture job inputs do not bind this helper at its hash',key)
  assert rows.get(str(VERIFY/'verification1005.json'))==VERIFICATION_SHA256,('picture job inputs bind another verification1005.json',key)
  assert rows.get(str(OUT/'exponential_magic_master.mp4'))==MOVIE_SHA and rows.get(vc['seed_path'])==vc['seed_sha256'],('picture job inputs bind another movie or seed',key)
  es=ident[key][1]
  assert rows.get(es['path'])==es['sha256'] and rows.get(str(VERIFY/(key+'.framehash')))==es['framehash_sha256'] and rows.get(str(VERIFY/'master-video.framehash'))==es['master_video_framehash_sha256'],('picture job inputs bind another chapter file or framehash',key)
  jobs.append((key,j,rec,m,rows))
 return jobs
def recheck(jobs,chapters,ident):
 """Packaging tripwire: the full verify1005 manifest once, then each chapter's exact record, plan, filter, pictures and sheets."""
 files=read(VM)['files'];drift=[x['path'] for x in files if sha(x['path'])!=x['sha256']];assert not drift,('upstream drift since verify1005',drift[:20])
 out=[]
 for (key,j,rec,m,rows),vc in zip(jobs,chapters):
  assert vc['chapter']==key;es=ident[key][1]
  seed=Path(vc['seed_path']);assert seed==seed_path(key) and not seed.is_symlink() and sha(seed)==vc['seed_sha256'],('seed differs from verification1005',key)
  plan=selection(key,read(seed),vc['frames'],vc['offset_frames']);frames=sorted(plan);assert len(frames)==vc['samples'],('seed union size differs',key)
  local=local_frames(frames,es)
  pd,sd=j/'pictures',j/'sheets'
  for x in [pd,sd,rec,j/'selection-plan.json',j/'select.filter']:assert not x.is_symlink(),('symlink inside the picture job',str(x))
  assert pd.is_dir() and sd.is_dir(),('picture job has no pictures/ or sheets/ directory',key)
  names=[f'frame-{n+1:06d}.jpg' for n in range(len(frames))];snames=[f'sheet-{i//8+1:03}.jpg' for i in range(0,len(frames),8)]
  assert sorted(x.name for x in pd.iterdir())==names,('pictures/ has extra or missing files',key)
  assert sorted(x.name for x in sd.iterdir())==snames,('sheets/ has extra or missing files',key)
  for x in [pd/n for n in names]+[sd/s for s in snames]:assert not x.is_symlink() and x.is_file() and x.resolve()==j.resolve()/x.parent.name/x.name,('picture or sheet is not a plain file inside its job',str(x))
  pics=[{'path':str(pd/n),'sha256':sha(pd/n),'frame':f,'local_frame':l,'reasons':plan[f]} for n,f,l in zip(names,frames,local)]
  sheets=[{'path':str(sd/s),'sha256':sha(sd/s),'frames':frames[i:i+8]} for i,s in zip(range(0,len(frames),8),snames)]
  assert read(j/'selection-plan.json')=={'chapter':key,'master_sha256':MOVIE_SHA,'samples':[{'frame':f,'local_frame':l,'reasons':plan[f]} for f,l in zip(frames,local)],'source':vc,'extract_source':es},('selection-plan.json differs from the seed union and chapter identity',key)
  assert (j/'select.filter').read_text()==select_filter(local),('select.filter differs from the local seed union',key)
  p=read(rec);assert isinstance(p.get('pictures'),list) and isinstance(p.get('sheets'),list),('record has no picture or sheet list',key)
  assert [(x.get('frame'),x.get('local_frame'),x.get('reasons')) for x in p['pictures']]==[(x['frame'],x['local_frame'],x['reasons']) for x in pics],('record frames, local frames or reasons differ from the seed union',key)
  assert [x.get('path') for x in p['pictures']]==[x['path'] for x in pics],('record picture paths are not canonical',key)
  assert [(x.get('path'),x.get('frames')) for x in p['sheets']]==[(x['path'],x['frames']) for x in sheets],('record sheets are not eight frames each in order at canonical paths',key)
  assert [x.get('sha256') for x in p['pictures']+p['sheets']]==[x['sha256'] for x in pics+sheets],('picture or sheet changed after its job',key)
  for x in [j/'probe-source.stdout',j/'probe-master.stdout']:assert x.is_file() and not x.is_symlink(),('probe output missing or a symlink',str(x))
  expected={'chapter':key,'movie_sha256':MOVIE_SHA,'pictures':pics,'sheets':sheets,'samples':len(frames),'source':vc,'extract_source':es,'stream':stream_match(j),'scope':SCOPE1008,
   'verification_sha256':VERIFICATION_SHA256,'verification_receipt_sha256':rows.get(str(VERIFY/'resource-receipt.json')),'verification_inputs_sha256':VERIFY_INPUTS_SHA,'verify_helper_sha256':VERIFY_HELPER_SHA}
  assert expected['verification_receipt_sha256']==sha(VERIFY/'resource-receipt.json'),('picture job bound another verify receipt',key)
  assert p==expected,('record differs from the exact expected record',key,sorted(k for k in set(p)|set(expected) if p.get(k)!=expected.get(k)))
  out.append({'chapter':key,'job':str(j),'helper':str(Path(__file__).resolve()),'command':read(j/'resource-receipt.json')['command'],'extract_source':es,
   'record':str(rec),'record_sha256':sha(rec),'inputs_sha256':sha(m),'selection_plan_sha256':sha(j/'selection-plan.json'),'select_filter_sha256':sha(j/'select.filter'),'pictures':len(pics),'sheets':len(sheets)})
 return {'verification_sha256':VERIFICATION_SHA256,'verify_inputs_sha256':VERIFY_INPUTS_SHA,'upstream_rows':len(files),'upstream_rows_unchanged':True,'chapters':out,'samples':sum(x['pictures'] for x in out),'records_exact':True,
  'scope':'Packaging tripwire only: verify1005 inputs unchanged; every chapter identity re-proved from the bound framehash files; each chapter job ran this helper with the exact command, and its record, selection plan, select filter, pictures and sheets equal what the bound seed union, the chapter offset and the copied layout give. Images unviewed; no listening, fine-sync or publication acceptance.'}
def main():
 ap=argparse.ArgumentParser();ap.add_argument('mode',choices=['make-inputs','run']);ap.add_argument('--phase',choices=['pictures','recheck'],required=True);ap.add_argument('--chapter');a=ap.parse_args();assert sys.flags.isolated and sys.dont_write_bytecode
 assert (a.phase=='pictures')==(a.chapter is not None)
 name='css-exponential-master-pictures1008-'+a.chapter if a.phase=='pictures' else 'css-exponential-pictures1008-recheck'
 job=D/'jobs'/name;inputs=Q/(name+'-inputs.json')
 keys,chapters,vr,movie,paths,selected,ident=gather(a.chapter)
 if a.phase=='pictures':
  estimate=len(selected[a.chapter])*2500000+128*1024**2;assert estimate<3.5*1024**3 and shutil.disk_usage(D).free-estimate>=8*1024**3
 else:
  jobs=finished(keys,chapters,ident)
  for key,j,rec,m,rows in jobs:paths|={j/'resource-receipt.json',m,rec,j/'selection-plan.json',j/'select.filter'}
 if a.mode=='make-inputs':
  assert not job.exists();write(inputs,{'files':[{'path':str(p),'sha256':sha(p)} for p in sorted(paths)]});return
 h=module('build959',Q/'build959.py');base=h.base();base.own_group();base.STATE['job']=job
 for sig in base.STOP_SIGNALS:signal.signal(sig,base.on_stop)
 r=read(job/'resource-receipt.json');assert r.get('admitted') and 'exit' not in r and not r['preflight_only'] and r['phase']=='native' and r['policy']==h.POLICY
 assert r['command']==['/usr/bin/python3','-I','-B',str(Path(__file__).resolve()),*sys.argv[1:]] and r['inputs_sha256']==sha(inputs)
 frozen=read(inputs)['files'];bound={x['path']:x['sha256'] for x in frozen}
 for p in paths:assert str(p) in bound and sha(p)==bound[str(p)]
 os.environ.update(TMPDIR=str(D/'work/exponential959/tmp'),XDG_CACHE_HOME=str(D/'work/exponential959/cache'));os.sched_setaffinity(0,set(sorted(os.sched_getaffinity(0))[:4]))
 if a.phase=='pictures':
  for n in (1005,1006,1007):assert not (D/'jobs'/(f'css-exponential-master-pictures{n}-'+a.chapter)).exists()
  key=a.chapter;source,es,_=ident[key]
  result=pictures(job,key,selected[key],next(x for x in chapters if x['chapter']==key),source,es,movie,sha(movie),lambda name,argv:base.run_step(job,REPO,name,argv,14400,None))
  result|={'verification_sha256':VERIFICATION_SHA256,'verification_receipt_sha256':bound[str(VERIFY/'resource-receipt.json')],'verification_inputs_sha256':vr['inputs_sha256'],'verify_helper_sha256':VERIFY_HELPER_SHA,'scope':SCOPE1008};output=job/'pictures1008.json'
 else:
  result=recheck(jobs,chapters,ident);output=job/'recheck1008.json'
 assert sum(p.stat().st_size for p in job.rglob('*') if p.is_file())<3.75*1024**3 and shutil.disk_usage(D).free>=8*1024**3
 for x in frozen:assert sha(x['path'])==x['sha256']
 write(output,result)
if __name__=='__main__':main()
