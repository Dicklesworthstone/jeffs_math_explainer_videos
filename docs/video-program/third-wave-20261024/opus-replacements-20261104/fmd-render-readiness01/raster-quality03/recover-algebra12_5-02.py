"""Preserve the completed asset download, then recognize without altering audio."""
import hashlib,json,subprocess,sys,wave
from pathlib import Path
R=Path('/Users/jemanuel/projects/jeffs_math_explainer_videos')
A=R/'narration_lab/third-wave-20261024/root/elevenlabs-expanded-algebra-20261103/frederick-stage03-root01'
J=R/'narration_lab/third-wave-20261024/root/five-film-release-20261027/algebra-frederick-recovery12_5-02'
SOURCE=Path('/Users/jemanuel/Downloads/The operation and its labels must travel together. (1).wav')
MODEL=Path('/Users/jemanuel/.cache/franken_whisper/models/whisper/whisper-large-v3-turbo-f16-v1/ggml-large-v3-turbo.bin')
FW=Path('/opt/homebrew/bin/fw')
def sha(p):
    with p.open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest()
def main():
    assert sys.argv[1:]==['--run']
    receipt=json.loads((J/'resource-receipt.json').read_text())
    assert receipt['admitted'] and 'exit' not in receipt
    assert sha(SOURCE)=='e40ad8801f6e18f521ac8d8e4dec846473b611090e9d68ded779785cd818bfed'
    request=json.loads((A/'12_5-request01.json').read_text())
    assert hashlib.sha256(request['text'].encode()).hexdigest()==request['text_sha256']
    wav=J/'12_5-recovered-asset.wav'
    with wav.open('xb') as f:f.write(SOURCE.read_bytes())
    assert sha(wav)==sha(SOURCE)
    with wave.open(str(wav)) as w:
        params=dict(rate=w.getframerate(),channels=w.getnchannels(),sample_width=w.getsampwidth(),samples=w.getnframes())
        assert params==dict(rate=48000,channels=1,sample_width=2,samples=1121280)
        assert len(w.readframes(w.getnframes()))==1121280*2
    evidence=dict(cue='12_5',source=str(SOURCE),path=str(wav),sha256=sha(wav),bytes=wav.stat().st_size,**params,duration=23.36,
                  request_sha256=sha(A/'12_5-request01.json'),text_sha256=request['text_sha256'],
                  asset_evidence_sha256=sha(A/'12_5-second-asset-recovery02.json'),
                  provenance='Recovered via newly saved Assets row after history details showed oMdZbkzrsANkSo5Yhxhr; association is UI provenance, not embedded in WAV.',
                  selected_take=None,listening=False,fine_sync=False,regenerated=False)
    with (J/'preservation.json').open('x') as f:json.dump(evidence,f,indent=2)
    command=[str(FW),'transcribe','--input',str(wav),'--backend','whisper-cpp','--model',str(MODEL),'--language','en','--no-diarize','--no-persist','--split-on-word','--max-segment-length','1','--threads','4','--timeout','180','--json']
    with (J/'recognition.json').open('x') as out,(J/'recognition.stderr').open('x') as err:
        subprocess.run(command,stdout=out,stderr=err,check=True,timeout=240)
    raw=json.loads((J/'recognition.json').read_text());result=raw.get('result',raw)
    recognized=' '.join(s['text'].strip() for s in result['segments'])
    assert recognized and sha(wav)==sha(SOURCE)
    with (J/'recognition-summary.json').open('x') as f:json.dump(dict(expected=request['text'],recognized=recognized,wav_sha256=sha(wav),report_sha256=sha(J/'recognition.json'),scope='Recognition lead only; actual transcript triage, listening and fine sync remain.'),f,indent=2)
    print('Complete original WAV preserved and recognized; no listening acceptance.')
if __name__=='__main__':main()
