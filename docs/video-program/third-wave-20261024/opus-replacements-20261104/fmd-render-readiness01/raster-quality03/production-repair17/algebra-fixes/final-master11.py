from pathlib import Path
import json,subprocess,hashlib,os,wave
R=Path.cwd();B=R/'narration_lab/third-wave-20261024/root/five-film-release-20261027';I=B/'algebra-fixes-inputs02';J=B/'algebra-final-master11';O=J/'final';T=B/'algebra-fixes-timeline04/timeline';N=B/'algebra-fixes-assemble03/narration';S=B/'algebra-fixes-score07/score/stems'
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
a=json.loads((J/'resource-receipt.json').read_text());assert a['admitted'] and 'exit' not in a
cs=(I/'native-chapters01.txt').read_text().splitlines();assert len(cs)==16
assert not O.exists();O.mkdir();links=[];total=0
for c in cs:
 job=B/('algebra-final11-'+c);receipt=json.loads((job/'resource-receipt.json').read_text());assert receipt['exit']==0 and receipt['inputs_sha256']==receipt['final_inputs_sha256'] and not receipt.get('resource_stop')
 source=job/'render';f=next(int(x.split('\t')[1]) for x in (T/(c+'.timeline.tsv')).read_text().splitlines() if x.startswith('frames\t'));total+=f
 for suffix in ['.timeline.tsv','.cues.tsv']:assert (source/(c+suffix)).read_bytes()==(T/(c+suffix)).read_bytes()
 for suffix in ['.mp4','.timeline.tsv','.cues.tsv','.cuelog.tsv','.layout.tsv']:
  src=source/(c+suffix);dst=O/src.name;os.link(src,dst);links.append(dict(source=str(src),path=str(dst),sha256=sha(src)))
 captures=list((source/'soundtrack'/c).glob('*.wav'));assert len(captures)==1,captures
 with wave.open(str(captures[0])) as w:assert (w.getframerate(),w.getnchannels(),w.getnframes())==(48000,2,f*800)
 dst=O/'soundtrack'/c/'soundtrack_0.wav';dst.parent.mkdir(parents=True);os.link(captures[0],dst);links.append(dict(source=str(captures[0]),path=str(dst),sha256=sha(dst)))
assert total==251862
(J/'staged-inputs.json').write_text(json.dumps(links,indent=2))
for command,args in [('master',[]),('captions',['all'])]:
 with (J/(command+'-native.log')).open('x') as log:subprocess.run([str(B/'algebra-stream-cap11-inputs/binary'),command,*args,'--narration',str(N),*(['--score',str(S)] if command=='master' else []),'--out',str(O)],cwd=R,stdout=log,stderr=subprocess.STDOUT,check=True,timeout=14400)
replays=[x for x in (J/'master-native.log').read_text().splitlines() if "replay = fmn's soundtrack, bit for bit" in x];assert [x.split()[0] for x in replays]==cs
for x in links:assert sha(Path(x['source']))==sha(Path(x['path']))==x['sha256']
outputs={k:O/('abstract_algebra_expanded'+suffix) for k,suffix in [('master','_master.mp4'),('srt','.en.srt'),('vtt','.en.vtt')]}
p=json.loads(subprocess.check_output(['ffprobe','-v','error','-show_streams','-of','json',str(outputs['master'])]));v=next(x for x in p['streams'] if x['codec_type']=='video');a=next(x for x in p['streams'] if x['codec_type']=='audio')
assert (v['width'],v['height'],v['avg_frame_rate'],int(v['nb_frames']))==(3840,2160,'60/1',total)
assert (a['sample_rate'],a['channels'],a['codec_name'])==('48000',2,'aac')
assert abs(float(a['duration'])-total/60)<=1024/48000+0.001
assert outputs['srt'].stat().st_size and outputs['vtt'].read_text().startswith('WEBVTT')
(J/'master-summary.json').write_text(json.dumps(dict(chapters=cs,frames=total,native_bit_identical_replays=16,outputs={k:dict(path=str(p),sha256=sha(p)) for k,p in outputs.items()},final_picture_review=False,full_decode=False,exact_caption_check=False,encoded_loudness=False,motion_review=False,listening_accepted=False,independent_av_acceptance=False,publication_accepted=False),indent=2))
