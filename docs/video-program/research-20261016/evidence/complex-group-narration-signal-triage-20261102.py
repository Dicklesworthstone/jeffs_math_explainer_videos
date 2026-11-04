#!/usr/bin/env python3
"""Measure the six intact complex-analysis takes; not listening or cue timing."""
from array import array
from datetime import datetime, timezone
from pathlib import Path
import hashlib
import json
import math
import sys
import wave


def dbfs(value):
    return None if value == 0 else round(20 * math.log10(value / 32768), 2)


def peak_rms(samples):
    if not samples:
        return (0, 0.0)
    return (max(abs(x) for x in samples), math.sqrt(sum(x * x for x in samples) / len(samples)))


assert peak_rms(array('h', [0, 0])) == (0, 0.0)
assert peak_rms(array('h', [0, 100]))[0] == 100
root = Path.cwd()
w = root / 'narration_lab/third-wave-20261024/root/elevenlabs-complex-analysis-20261101'
j = root / 'narration_lab/third-wave-20261024/root/five-film-release-20261027'
out = j / 'complex-group-narration-signal-triage01'
script = dict(line.split('\t', 1) for line in (root / 'explainer_corpus/complex_analysis_visual/script.tsv').read_text().splitlines())
normalize = lambda text: ' '.join(text.split())
receipts = [
    ('opening', 'opening-takes-receipt01.json', ['00_1']),
    ('supplement', 'additional-phase-examples-takes-receipt01.json', ['01_2']),
    ('remaining', 'remaining-draft-takes-receipt01.json', [cue for cue in script if cue not in ('00_1', '01_2')]),
]
results = []
for group, receipt_name, cues in receipts:
    receipt = json.loads((w / receipt_name).read_text())
    submitted = normalize(receipt['submitted_text'])
    positions = []
    for cue in cues:
        text = normalize(script[cue])
        assert submitted.count(text) == 1
        positions.append(submitted.index(text))
    assert positions == sorted(positions) and len(receipt['records']) == 2
    for take, rec in enumerate(receipt['records'], 1):
        path = Path(rec['path'])
        if not path.is_absolute():
            path = root / path
        assert hashlib.sha256(path.read_bytes()).hexdigest() == rec['sha256']
        with wave.open(str(path), 'rb') as reader:
            assert (reader.getnchannels(), reader.getsampwidth(), reader.getframerate()) == (1, 2, 48000)
            count = reader.getnframes()
            raw = reader.readframes(count)
        assert count == rec['frames'] and len(raw) == count * 2
        samples = array('h')
        samples.frombytes(raw)
        if sys.byteorder != 'little':
            samples.byteswap()
        peak, _ = peak_rms(samples)
        full_scale = sum(x == -32768 or x == 32767 for x in samples)
        first_peak, first_rms = peak_rms(samples[:1440])
        last_peak, last_rms = peak_rms(samples[-1440:])
        longest = run = 0
        for start in range(0, len(samples) - 2399, 2400):
            _, rms = peak_rms(samples[start:start + 2400])
            run = run + 1 if rms < 32768 * 10 ** (-50 / 20) else 0
            longest = max(longest, run)
        results.append({
            'group': group, 'take': take, 'script_text_cues': cues,
            'wav': str(path), 'wav_sha256': rec['sha256'], 'samples': count,
            'seconds': round(count / 48000, 3), 'peak_dbfs': dbfs(peak),
            'digital_full_scale_samples': full_scale,
            'first_30ms_peak_dbfs': dbfs(first_peak), 'first_30ms_rms_dbfs': dbfs(first_rms),
            'last_30ms_peak_dbfs': dbfs(last_peak), 'last_30ms_rms_dbfs': dbfs(last_rms),
            'longest_consecutive_complete_50ms_windows_below_minus50dbfs_seconds': round(longest * 0.05, 2),
        })
assert len(results) == 6
report = {
    'created_utc': datetime.now(timezone.utc).isoformat(), 'count': len(results),
    'method': '16-bit mono 48 kHz sample-domain metrics; 30 ms edges and complete non-overlapping 50 ms RMS windows.',
    'limitations': [
        'Text matches to 15 script cues do not establish audible boundaries or spoken words.',
        'Signal metrics are not listening, take selection, speech detection or fine synchronization.',
        'Digital full-scale counts do not measure true peak or prove audible clipping.',
        'Quiet windows and energetic edges can be intentional; this is not final encoded-audio qualification.',
    ],
    'takes': results,
}
(out / 'signal-triage.json').open('x').write(json.dumps(report, indent=2) + '\n')
print('Measured', len(results), 'intact grouped takes; full-scale samples in', sum(x['digital_full_scale_samples'] > 0 for x in results), 'takes.')
