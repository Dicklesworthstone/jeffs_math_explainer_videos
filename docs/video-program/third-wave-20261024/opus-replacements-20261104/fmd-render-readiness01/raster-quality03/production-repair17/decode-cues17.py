"""Decode selected complete UI takes, preserving every source and recording real output samples."""
from pathlib import Path
import hashlib
import json
import subprocess
import sys
import wave

plan = json.loads(Path(sys.argv[1]).read_text())
job = Path(plan['job_path'])
admission = json.loads((job / 'resource-receipt.json').read_text())
assert admission['admitted'] and 'exit' not in admission
out = job / 'decoded'
out.mkdir()
voice = 'Frederick Surrey - Smooth and Velvety'
records = []
for selected in plan['takes']:
    source = Path(selected['path'])
    assert hashlib.sha256(source.read_bytes()).hexdigest() == selected['sha256']
    request = Path(selected['request'])
    submitted = json.loads(request.read_text())
    assert submitted['cue'] == selected['cue'] and submitted['voice'] == voice
    target = out / (selected['cue'] + '.wav')
    subprocess.run([plan['ffmpeg'], '-nostdin', '-n', '-v', 'error', '-i', str(source),
                    '-ac', '1', '-ar', '24000', '-c:a', 'pcm_s16le', str(target)], check=True, timeout=180)
    with wave.open(str(target)) as wav:
        assert wav.getparams()[:3] == (1, 2, 24000)
        samples = wav.getnframes()
    records.append(dict(cue=selected['cue'], text=submitted['text'], voice=voice,
                        wav=str(target), wav_sha256=hashlib.sha256(target.read_bytes()).hexdigest(),
                        samples=samples, rate=24000, source=str(source), source_sha256=selected['sha256'],
                        request=str(request), request_sha256=hashlib.sha256(request.read_bytes()).hexdigest()))
with (job / 'decode-receipt.json').open('x') as stream:
    json.dump(dict(schema='elevenlabs-ui-receipt-v2', source='ElevenLabs UI, Frederick Surrey',
                   provider='ElevenLabs', voice=voice, records=records,
                   scope='Complete take conversion only; no time or pitch processing. Listening and sync unreviewed.'), stream, indent=2)
print('Decoded complete takes:', len(records))
