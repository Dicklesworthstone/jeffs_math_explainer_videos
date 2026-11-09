"""Extract complete last/first cue windows at all14 verified encoded-master joins.
No listening or recognition verdict; no source audio or timing changes.
"""
from pathlib import Path
import hashlib,json,subprocess,wave
R=Path('/data/projects/jmev-render-worker-20261107')
D=Path('/data/projects/jmev-stroke-probe-20261106')
Q=R/'qualification01/production19'
J=R/'jobs/trj-production19-join-extract83'
M=R/'jobs/trj-production19-master01'
V=R/'jobs/trj-production19-verify01'
I=D/'work/production-repair19/workspace/explainer_corpus/complex_analysis_visual/script.tsv'
def sha(p):
 with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest()
def read(p):return json.loads(Path(p).read_text())
def success(p):
 r=read(p);assert r.get('finished') and r.get('exit')==0 and r['inputs_sha256']==r['final_inputs_sha256']
 assert not any(r.get(k) for k in ('error','resource_stop','stopped_for_resource_limit'))
 return r
r=read(J/'resource-receipt.json');assert r['admitted'] and 'exit' not in r
assert r['policy']['stop_free_gib']==8 and r['policy']['phase_growth_gib']==4
manifest=Q/'join-extract83-inputs.json';assert sha(manifest)==r['inputs_sha256']
bound={x['path']:x['sha256'] for x in read(manifest)['files']}
for p,h in bound.items():assert sha(p)==h,p
for p in (Path(__file__).resolve(),M/'master-summary.json',V/'verification.json',I):assert bound.get(str(p))==sha(p)
success(M/'resource-receipt.json');success(V/'resource-receipt.json')
s=read(M/'master-summary.json');v=read(V/'verification.json')
assert s['frames']==133662 and s['native_bit_identical_replays']==15
movie=Path(s['outputs']['master']['path']);assert bound.get(str(movie))==sha(movie)==s['outputs']['master']['sha256']
script=dict(line.split('\t',1) for line in I.read_text().splitlines() if line.strip() and not line.startswith('#'));assert len(script)==105
chapters=s['chapters'];assert len(chapters)==15 and [x['chapter'] for x in v['chapters']]==chapters
placements=v['placements'];assert [x['cue'] for x in placements]==list(script)
results=[]
for n,(left,right) in enumerate(zip(chapters,chapters[1:]),1):
 before=[x for x in placements if x['chapter']==left][-1]
 after=[x for x in placements if x['chapter']==right][0]
 start_sample=max(0,before['start_frame']*800-12000)
 end_sample=min(133662*800,after['start_frame']*800+round(after['duration_s']*48000)+12000)
 assert start_sample<end_sample
 stem=J/f'join-{n:02d}';audio=stem.with_suffix('.wav');spectrum=stem.with_suffix('.png')
 with stem.with_suffix('.extract.log').open('x') as log:
  subprocess.run(['ffmpeg','-nostdin','-n','-v','error','-i',str(movie),'-vn','-af',f'atrim=start_sample={start_sample}:end_sample={end_sample},asetpts=PTS-STARTPTS','-ar','16000','-ac','1','-c:a','pcm_s16le',str(audio)],stdout=log,stderr=subprocess.STDOUT,check=True,timeout=1800)
 with wave.open(str(audio)) as w:
  assert (w.getframerate(),w.getnchannels(),w.getsampwidth())==(16000,1,2)
  assert abs(w.getnframes()/16000-(end_sample-start_sample)/48000)<.001
 with stem.with_suffix('.spectrum.log').open('x') as log:
  subprocess.run(['ffmpeg','-nostdin','-n','-v','error','-i',str(audio),'-lavfi','showspectrumpic=s=1600x600:legend=1:scale=log:color=intensity','-frames:v','1',str(spectrum)],stdout=log,stderr=subprocess.STDOUT,check=True,timeout=600)
 results.append(dict(index=n,left=left,right=right,start_sample_48k=start_sample,end_sample_48k=end_sample,before=dict(before,text=script[before['cue']]),after=dict(after,text=script[after['cue']]),scheduled_gap_seconds=(after['start_frame']-before['start_frame'])/60-before['duration_s'],files=[dict(path=str(p),sha256=sha(p)) for p in (audio,spectrum)]))
assert len(results)==14
for p,h in bound.items():assert sha(p)==h,p
with (J/'joins.json').open('x') as f:json.dump(dict(master_sha256=sha(movie),joins=results,transcripts_read=False,spectrograms_viewed=False,listening_accepted=False,independent_av_accepted=False,scope='Actual encoded-master14 joins with complete adjacent cues; awaiting transcription and visual inspection.'),f,indent=2)
