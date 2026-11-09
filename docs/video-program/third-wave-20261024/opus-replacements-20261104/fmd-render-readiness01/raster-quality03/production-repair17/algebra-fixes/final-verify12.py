from pathlib import Path
import json,hashlib,subprocess,importlib.util,os,sys
R=Path.cwd();B=R/'narration_lab/third-wave-20261024/root/five-film-release-20261027';J=B/'algebra-final-verify12';M=B/'algebra-final-master12';O=M/'final';I=B/'algebra-fixes-inputs02';N=B/'algebra-fixes-assemble03/narration'
SHARED=R/'docs/video-program/third-wave-20261024/opus-replacements-20261104/logarithm-stage09-final-verification02/verify_stage09_final.py'
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
assert sha(SHARED)=='f266a2315e4a9af247c648da416881f3a2999e2682e9c611ff82ca3727b70b94'
spec=importlib.util.spec_from_file_location('shared_verify',SHARED);v=importlib.util.module_from_spec(spec);spec.loader.exec_module(v)
for rel in [v.STRICT_REL,v.SCREEN_REL]:assert sha(R/rel)==v.PINS[rel]
r=json.loads((J/'resource-receipt.json').read_text());assert r['admitted'] and 'exit' not in r
r=json.loads((M/'resource-receipt.json').read_text());assert r['exit']==0 and r['inputs_sha256']==r['final_inputs_sha256'] and not r.get('resource_stop')
summary=json.loads((M/'master-summary.json').read_text());assert summary['frames']==251862 and summary['native_bit_identical_replays']==16
for row in summary['outputs'].values():assert sha(Path(row['path']))==row['sha256']
script=dict(x.split('\t',1) for x in (I/'workspace/explainer_corpus/abstract_algebra_expanded/script.tsv').read_text().splitlines() if x.strip() and not x.startswith('#'));assert len(script)==186
report=json.loads((N/'report.json').read_text());v.check_phrases(json.loads((N/'phrase-timings.json').read_text()),script,N)
placements=[];offset=0;chapters=[]
for c in (I/'native-chapters01.txt').read_text().splitlines():
 rows=[x.split('\t') for x in (O/(c+'.timeline.tsv')).read_text().splitlines()];f=next(int(x[1]) for x in rows if x[0]=='frames');assert next(x[1] for x in rows if x[0]=='fps')=='60'
 for x in rows:
  if x[0]=='line':
   assert sha(Path(x[4]))==sha(N/(x[1]+'.wav'));duration=v.wav_seconds(x[4]);assert int(x[2])/60+duration<=f/60+.001
   placements.append(dict(cue=x[1],chapter=c,start_frame=offset+int(x[2]),duration_s=duration))
 chapters.append(dict(chapter=c,frames=f,start_frame=offset));offset+=f
assert offset==251862 and [x['cue'] for x in placements]==list(script)
outputs={k:Path(x['path']) for k,x in summary['outputs'].items()};captions=v.check_captions(outputs['srt'].read_text(),outputs['vtt'].read_text(),script,report,placements,offset)
(J/'decode-logs').mkdir();env=dict(os.environ,**v.EXPECT,TMPDIR=str(J/'decode-logs'))
with (J/'strict-video.log').open('x') as log:subprocess.run(['bash',str(R/v.STRICT_REL),str(outputs['master'])],cwd=R,env=env,stdout=log,stderr=subprocess.STDOUT,check=True,timeout=14400)
text=(J/'strict-video.log').read_text();assert text.splitlines()[-1]=='VERIFY PASS';decodes=v.decode_lines(text);assert v.software_decode_clean(decodes)
probe=json.loads(subprocess.check_output(['ffprobe','-v','error','-show_streams','-show_format','-of','json',str(outputs['master'])]));packets=int(subprocess.check_output(['ffprobe','-v','error','-select_streams','V:0','-count_packets','-show_entries','stream=nb_read_packets','-of','csv=p=0',str(outputs['master'])],timeout=3600).strip());media=v.media_facts(probe,packets,offset)
with (J/'encoded-loudness.log').open('x') as log:subprocess.run(['ffmpeg','-nostdin','-hide_banner','-nostats','-i',str(outputs['master']),'-vn','-af','ebur128=peak=true','-f','null','-'],stdout=log,stderr=subprocess.STDOUT,check=True,timeout=14400)
loudness=v.loudness_summary((J/'encoded-loudness.log').read_text());assert float(loudness['true_peak_dbtp'])<=-1
subprocess.run([sys.executable,'-I','-B',str(R/v.SCREEN_REL),str(outputs['srt']),str(offset),'60',str(J/'caption-screen.json')],check=True,timeout=600)
(J/'verification.json').write_text(json.dumps(dict(frames=offset,chapters=chapters,placements=placements,captions=captions,media=media,full_decode=decodes,encoded_loudness=loudness,caption_screen=json.loads((J/'caption-screen.json').read_text()),outputs=summary['outputs'],final_pictures_reviewed=False,motion_reviewed=False,listening_accepted=False,independent_av_accepted=False,publication_accepted=False),indent=2))
