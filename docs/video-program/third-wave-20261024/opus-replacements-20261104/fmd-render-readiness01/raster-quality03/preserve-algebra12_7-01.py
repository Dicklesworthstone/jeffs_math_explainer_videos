import json,hashlib,subprocess,wave,sys
from pathlib import Path
R=Path('/Users/jemanuel/projects/jeffs_math_explainer_videos')
A=R/'narration_lab/third-wave-20261024/root/elevenlabs-expanded-algebra-20261103/frederick-stage03-root01'
J=R/'narration_lab/third-wave-20261024/root/five-film-release-20261027/algebra-frederick-cue12_7-01'
MODEL='/Users/jemanuel/.cache/franken_whisper/models/whisper/whisper-large-v3-turbo-f16-v1/ggml-large-v3-turbo.bin'
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def main():
 assert sys.argv[1:]==['--run']
 r=json.loads((J/'resource-receipt.json').read_text());assert r['admitted'] and 'exit' not in r
 rows=json.loads((A/'12_7-source-downloads01.json').read_text());out=[]
 for row in rows:
  src=Path(row['source']);assert sha(src)==row['sha256']
  p=J/('12_7-take%02d.wav'%row['take'])
  with p.open('xb') as f:f.write(src.read_bytes())
  assert sha(p)==sha(src)
  with wave.open(str(p)) as w:
   assert (w.getframerate(),w.getnchannels(),w.getsampwidth(),w.getnframes())==(48000,1,2,row['samples'])
   assert len(w.readframes(w.getnframes()))==row['samples']*2
  report=J/('recognition%02d.json'%row['take'])
  with report.open('x') as o,(J/('recognition%02d.stderr'%row['take'])).open('x') as e:
   subprocess.run(['/opt/homebrew/bin/fw','transcribe','--input',str(p),'--backend','whisper-cpp','--model',MODEL,'--language','en','--no-diarize','--no-persist','--split-on-word','--max-segment-length','1','--threads','4','--timeout','180','--json'],stdout=o,stderr=e,check=True,timeout=240)
  data=json.loads(report.read_text());data=data.get('result',data)
  out.append(dict(**row,path=str(p),duration=row['samples']/48000,rate=48000,channels=1,sample_width=2,recognized=' '.join(s['text'].strip() for s in data['segments']),report=str(report),report_sha256=sha(report)))
 with (J/'preservation-recognition.json').open('x') as f:json.dump(dict(cue='12_7',takes=out,listening=False,fine_sync=False,selected_take=None),f,indent=2)
if __name__=='__main__':main()
