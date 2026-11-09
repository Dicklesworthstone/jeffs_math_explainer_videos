from pathlib import Path
import hashlib,json,subprocess,wave
R=Path('/Users/jemanuel/projects/jeffs_math_explainer_videos');Q=R/'docs/video-program/third-wave-20261024/opus-replacements-20261104/fmd-render-readiness01/raster-quality03/production-repair17';B=R/'narration_lab/third-wave-20261024/root/five-film-release-20261027';J=B/'complex-final-join-recognition88'
MODEL=Path('/Users/jemanuel/.cache/franken_whisper/models/whisper/whisper-large-v3-turbo-f16-v1/ggml-large-v3-turbo.bin');FW=Path('/opt/homebrew/bin/fw')
TQ='/data/projects/jmev-render-worker-20261107/qualification01/production19'
def sha(p):
 with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest()
r=json.loads((J/'resource-receipt.json').read_text());assert r['admitted'] and 'exit' not in r
manifest=Q/'join-recognition88-inputs.json';assert sha(manifest)==r['inputs_sha256']
bound=json.loads(manifest.read_text())['files']
for row in bound:assert sha(row['path'])==row['sha256']
assert sha(MODEL)=='1fc70f774d38eb169993ac391eea357ef47c88757ef72ee5943879b7e8e2bc69'
expected=json.loads((Q/'join-export88.json').read_text());assert len(expected['files'])==33
incoming=J/'source';incoming.mkdir()
with (J/'transfer.stderr').open('xb') as err:
 p=subprocess.Popen(['ssh','-o','BatchMode=yes','trj','python3 -I -B '+TQ+'/trj-join-export88.py --stream '+TQ+'/join-export88.json'],stdout=subprocess.PIPE,stderr=err)
 try:
  for row in expected['files']:
   name=row['name'];assert name==Path(name).name and name not in ('.','..')
   h=hashlib.sha256();left=row['bytes'];target=incoming/name
   with target.open('xb') as f:
    while left:
     b=p.stdout.read(min(left,1048576));assert b,'Truncated export stream';left-=len(b);h.update(b);f.write(b)
   assert h.hexdigest()==row['sha256']==sha(target)
  assert not p.stdout.read(1),'Trailing export bytes'
 finally:p.stdout.close()
 assert p.wait(timeout=120)==0
source_receipt=json.loads((incoming/'resource-receipt.json').read_text());assert source_receipt.get('exit')==0 and source_receipt['inputs_sha256']==source_receipt['final_inputs_sha256']==sha(incoming/'join-extract83-inputs.json')
joins=json.loads((incoming/'joins.json').read_text());assert len(joins['joins'])==14 and joins['master_sha256']==expected['master_sha256']
summary=json.loads((incoming/'master-summary.json').read_text());assert summary['outputs']['master']['sha256']==joins['master_sha256']
results=[]
for row in joins['joins']:
 audio=incoming/f"join-{row['index']:02d}.wav"
 with wave.open(str(audio)) as w:assert (w.getframerate(),w.getnchannels(),w.getsampwidth())==(16000,1,2)
 stem=J/f"join-{row['index']:02d}"
 with stem.with_suffix('.json').open('x') as out,stem.with_suffix('.asr.log').open('x') as err:
  subprocess.run([str(FW),'transcribe','--input',str(audio),'--model',str(MODEL),'--language','en','--no-diarize','--no-persist','--json','--max-segment-length','1','--split-on-word','--threads','4','--timeout','1800'],stdout=out,stderr=err,check=True,timeout=1900)
 parsed=json.loads(stem.with_suffix('.json').read_text());assert isinstance(parsed,dict) and parsed
 results.append(dict(index=row['index'],transcript=str(stem.with_suffix('.json')),sha256=sha(stem.with_suffix('.json')),expected_before=row['before']['text'],expected_after=row['after']['text'],transcript_read=False,spectrogram_viewed=False))
for row in bound:assert sha(row['path'])==row['sha256']
with (J/'recognition-summary.json').open('x') as f:json.dump(dict(master_sha256=joins['master_sha256'],joins=results,listening_accepted=False,independent_av_accepted=False,scope='Single-model actual encoded-master join transcription. Every transcript and spectrogram needs reading; discrepancies are questions, not speech verdicts.'),f,indent=2)
