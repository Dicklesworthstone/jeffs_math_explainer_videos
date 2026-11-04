#!/usr/bin/env python3
"""Measure unselected original take pairs; these measurements are not listening."""
from array import array
from datetime import datetime, timezone
from pathlib import Path
import hashlib
import json
import math
import sys
import wave


def peak_rms(samples):
    if not samples:
        return (0, 0.0)
    return (max(abs(x) for x in samples), math.sqrt(sum(x * x for x in samples) / len(samples)))


def dbfs(value):
    return None if value == 0 else round(20 * math.log10(value / 32768), 2)


assert peak_rms(array('h', [0, 0])) == (0, 0.0)
assert peak_rms(array('h', [0, 100]))[0] == 100
root = Path.cwd()
handoff_path = root / 'docs/video-program/third-wave-20261024/listening-handoff-five-new-videos-20261102.json'
handoff = json.loads(handoff_path.read_text())
job = root / 'narration_lab/third-wave-20261024/root/five-film-release-20261027/paired-original-narration-signal-triage01'
sha = lambda p: hashlib.sha256(p.read_bytes()).hexdigest()
results = []
for project in handoff['projects']:
    if 'cues' not in project:
        continue  # Complex-analysis grouped originals were measured separately.
    script = Path(project['script'])
    assert sha(script) == project['script_sha256']
    for cue in project['cues']:
        receipt = Path(cue['receipt'])
        assert sha(receipt) == cue['receipt_sha256'] and len(cue['takes']) == 2
        for take_number, take in enumerate(cue['takes'], 1):
            path = Path(take['path'])
            assert sha(path) == take['sha256']
            with wave.open(str(path), 'rb') as reader:
                assert (reader.getnchannels(), reader.getsampwidth(), reader.getframerate()) == (1, 2, 48000)
                frames = reader.getnframes()
                raw = reader.readframes(frames)
            assert frames == take['frames'] and len(raw) == frames * 2
            samples = array('h')
            samples.frombytes(raw)
            if sys.byteorder != 'little':
                samples.byteswap()
            peak, rms = peak_rms(samples)
            full_scale = sum(x == -32768 or x == 32767 for x in samples)
            first_peak, first_rms = peak_rms(samples[:1440])
            last_peak, last_rms = peak_rms(samples[-1440:])
            results.append({
                'project': project['title'], 'cue': cue['cue'], 'take': take_number,
                'wav': str(path), 'wav_sha256': take['sha256'], 'pcm_sha256': hashlib.sha256(raw).hexdigest(),
                'samples': frames, 'seconds': round(frames / 48000, 3),
                'peak_dbfs': dbfs(peak), 'rms_dbfs': dbfs(rms),
                'digital_full_scale_samples': full_scale,
                'first_30ms_peak_dbfs': dbfs(first_peak), 'first_30ms_rms_dbfs': dbfs(first_rms),
                'last_30ms_peak_dbfs': dbfs(last_peak), 'last_30ms_rms_dbfs': dbfs(last_rms),
            })
assert len(results) == 142
report = {
    'created_utc': datetime.now(timezone.utc).isoformat(), 'count': len(results),
    'method': 'Complete 16-bit mono 48 kHz source WAVs; sample-domain peak, RMS, exact digital full-scale count, and 30 ms edge metrics.',
    'limitations': [
        'Signal measurements are not listening, speech detection, take selection or fine synchronization.',
        'Digital full-scale counts do not measure intersample true peak or establish audible clipping.',
        'Differences between takes do not determine which take is better.',
        'Complex-analysis grouped source takes were measured in the separate grouped triage.',
        'No final encoded mix, mathematical correctness or independent review is established.',
    ],
    'takes': results,
}
(job / 'signal-triage.json').open('x').write(json.dumps(report, indent=2) + '\n')
print('Measured', len(results), 'complete original takes; digital full-scale samples in', sum(x['digital_full_scale_samples'] > 0 for x in results), 'takes.')
