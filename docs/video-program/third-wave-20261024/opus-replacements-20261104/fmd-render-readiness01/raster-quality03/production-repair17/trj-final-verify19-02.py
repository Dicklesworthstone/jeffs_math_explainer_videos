"""Prepared TRJ complex final technical verifier. Not run; requires frozen guard manifest. No visual/listening acceptance."""
from pathlib import Path
import json,hashlib,subprocess,importlib.util,os,sys
R=Path('/data/projects/jmev-render-worker-20261107');D=Path('/data/projects/jmev-stroke-probe-20261106');Q=R/'qualification01/production19'
J=R/'jobs/trj-production19-verify01';M=R/'jobs/trj-production19-master01';O=M/'final';N=R/'jobs/trj-production19-delivery02/narration'
I=D/'work/production-repair19/workspace/explainer_corpus/complex_analysis_visual'
SHARED=Q/'verify_stage09_final.py'
STRICT=Q/'verify_video_preserve_logs01.sh';SCREEN=Q/'caption-interval-audit01.py'
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
assert sha(SHARED)=='f266a2315e4a9af247c648da416881f3a2999e2682e9c611ff82ca3727b70b94'
spec=importlib.util.spec_from_file_location('shared_verify',SHARED);v=importlib.util.module_from_spec(spec);spec.loader.exec_module(v)
assert sha(STRICT)==v.PINS[v.STRICT_REL] and sha(SCREEN)==v.PINS[v.SCREEN_REL]
r=json.loads((J/'resource-receipt.json').read_text());assert r['admitted'] and 'exit' not in r
assert r['policy']['stop_free_gib']==8 and r['policy']['phase_growth_gib']==4
manifest=Q/'verify01-inputs.json';assert sha(manifest)==r['inputs_sha256']
bindings=json.loads(manifest.read_text())['files'];bound={x['path']:x['sha256'] for x in bindings};assert len(bound)==len(bindings)
for p,h in bound.items():assert sha(Path(p))==h
for p in [Path(__file__).resolve(),SHARED,STRICT,SCREEN,M/'resource-receipt.json',M/'master-summary.json',I/'script.tsv',N/'report.json',N/'phrase-timings.json']:assert bound.get(str(p))==sha(p)
r=json.loads((M/'resource-receipt.json').read_text());assert r.get('finished') and r['exit']==0 and r['inputs_sha256']==r['final_inputs_sha256'] and not any(r.get(k) for k in ('resource_stop','error','stopped_for_resource_limit'))
summary=json.loads((M/'master-summary.json').read_text());assert summary['frames']==133662 and summary['native_bit_identical_replays']==15
for row in summary['outputs'].values():assert bound.get(row['path'])==sha(Path(row['path']))==row['sha256']
script=dict(x.split('\t',1) for x in (I/'script.tsv').read_text().splitlines() if x.strip() and not x.startswith('#'));assert len(script)==105
report=json.loads((N/'report.json').read_text());v.check_phrases(json.loads((N/'phrase-timings.json').read_text()),script,N)
placements=[];offset=0;chapters=[]
assert len(summary['chapters'])==len(set(summary['chapters']))==15
for c in summary['chapters']:
 for suffix in ('timeline.tsv','cues.tsv','cuelog.tsv','layout.tsv'):assert bound.get(str(O/(c+'.'+suffix)))==sha(O/(c+'.'+suffix))
 rows=[x.split('\t') for x in (O/(c+'.timeline.tsv')).read_text().splitlines()];f=next(int(x[1]) for x in rows if x[0]=='frames');assert next(x[1] for x in rows if x[0]=='fps')=='60'
 for x in rows:
  if x[0]=='line':
   assert bound.get(str(N/(x[1]+'.wav')))==sha(Path(x[4]))==sha(N/(x[1]+'.wav'));duration=v.wav_seconds(x[4]);assert int(x[2])/60+duration<=f/60+.001
   placements.append(dict(cue=x[1],chapter=c,start_frame=offset+int(x[2]),duration_s=duration))
 chapters.append(dict(chapter=c,frames=f,start_frame=offset));offset+=f
assert offset==133662 and [x['cue'] for x in placements]==list(script)
outputs={k:Path(x['path']) for k,x in summary['outputs'].items()};captions=v.check_captions(outputs['srt'].read_text(),outputs['vtt'].read_text(),script,report,placements,offset)
(J/'decode-logs').mkdir();env=dict(os.environ,**v.EXPECT,TMPDIR=str(J/'decode-logs'))
with (J/'strict-video.log').open('x') as log:subprocess.run(['bash',str(STRICT),str(outputs['master'])],cwd=R,env=env,stdout=log,stderr=subprocess.STDOUT,check=True,timeout=14400)
text=(J/'strict-video.log').read_text();assert text.splitlines()[-1]=='VERIFY PASS';decodes=v.decode_lines(text);assert v.software_decode_clean(decodes)
probe=json.loads(subprocess.check_output(['ffprobe','-v','error','-show_streams','-show_format','-of','json',str(outputs['master'])]));packets=int(subprocess.check_output(['ffprobe','-v','error','-select_streams','V:0','-count_packets','-show_entries','stream=nb_read_packets','-of','csv=p=0',str(outputs['master'])],timeout=3600).strip());media=v.media_facts(probe,packets,offset)
with (J/'encoded-loudness.log').open('x') as log:subprocess.run(['ffmpeg','-nostdin','-hide_banner','-nostats','-i',str(outputs['master']),'-vn','-af','ebur128=peak=true','-f','null','-'],stdout=log,stderr=subprocess.STDOUT,check=True,timeout=14400)
loudness=v.loudness_summary((J/'encoded-loudness.log').read_text());assert float(loudness['true_peak_dbtp'])<=-1
subprocess.run([sys.executable,'-I','-B',str(SCREEN),str(outputs['srt']),str(offset),'60',str(J/'caption-screen.json')],check=True,timeout=600)
for p,h in bound.items():assert sha(Path(p))==h
(J/'verification.json').write_text(json.dumps(dict(frames=offset,chapters=chapters,placements=placements,captions=captions,media=media,full_decode=decodes,encoded_loudness=loudness,caption_screen=json.loads((J/'caption-screen.json').read_text()),outputs=summary['outputs'],final_pictures_reviewed=False,motion_reviewed=False,listening_accepted=False,independent_av_accepted=False,publication_accepted=False),indent=2))
