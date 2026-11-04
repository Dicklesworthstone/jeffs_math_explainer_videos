#!/usr/bin/env python3
"""Measure full assembled PCM cues; signal observations are not listening."""
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
j = root / 'narration_lab/third-wave-20261024/root/five-film-release-20261027'
out = j / 'four-video-narration-signal-triage01'
projects = [
    ('five_paradoxes', 'paradox-native-full-review-phrase-complete01', 'paradox-native-full-review-verify02', 21),
    ('abstract_algebra', 'abstract-native-full-review-timing-index01', 'abstract-native-full-review-verify01', 20),
    ('exponential', 'exponential-native-full-review-assemble01', 'exponential-native-full-review-verify01', 16),
    ('logarithm', 'logarithm-native-full-review-assemble01', 'logarithm-native-full-review-verify01', 14),
]
results = []
for name, assembly, verify, expected_count in projects:
    directory = j / assembly / 'narration'
    verified = {x['cue']: x for x in json.loads((j / verify / 'pcm-comparison.json').read_text())}
    assert len(verified) == expected_count and all(x['byte_identical'] for x in verified.values())
    files = sorted(directory.glob('*.wav'))
    assert len(files) == expected_count and {x.stem for x in files} == set(verified)
    for path in files:
        with wave.open(str(path), 'rb') as reader:
            assert (reader.getnchannels(), reader.getsampwidth(), reader.getframerate()) == (1, 2, 24000)
            count = reader.getnframes()
            raw = reader.readframes(count)
        assert len(raw) == count * 2 and count == verified[path.stem]['samples']
        assert hashlib.sha256(raw).hexdigest() == verified[path.stem]['pcm_sha256']
        samples = array('h')
        samples.frombytes(raw)
        if sys.byteorder != 'little':
            samples.byteswap()
        peak, _ = peak_rms(samples)
        full_scale = sum(x == -32768 or x == 32767 for x in samples)
        first_peak, first_rms = peak_rms(samples[:720])
        last_peak, last_rms = peak_rms(samples[-720:])
        # These fixed 50 ms windows are a triage heuristic, not speech detection.
        quiet = []
        for start in range(0, len(samples) - 1199, 1200):
            _, rms = peak_rms(samples[start:start + 1200])
            quiet.append(rms < 32768 * 10 ** (-50 / 20))
        longest = run = 0
        for item in quiet:
            run = run + 1 if item else 0
            longest = max(longest, run)
        results.append({
            'project': name, 'cue': path.stem, 'wav': str(path),
            'wav_sha256': hashlib.sha256(path.read_bytes()).hexdigest(),
            'pcm_sha256': hashlib.sha256(raw).hexdigest(), 'samples': count,
            'seconds': round(count / 24000, 3), 'peak_dbfs': dbfs(peak),
            'digital_full_scale_samples': full_scale,
            'first_30ms_peak_dbfs': dbfs(first_peak), 'first_30ms_rms_dbfs': dbfs(first_rms),
            'last_30ms_peak_dbfs': dbfs(last_peak), 'last_30ms_rms_dbfs': dbfs(last_rms),
            'longest_consecutive_complete_50ms_windows_below_minus50dbfs_seconds': round(longest * 0.05, 2),
        })
assert len(results) == 71
report = {
    'created_utc': datetime.now(timezone.utc).isoformat(), 'count': len(results),
    'method': '16-bit mono 24 kHz sample-domain metrics; 30 ms edges and non-overlapping complete 50 ms RMS windows.',
    'limitations': [
        'These are signal measurements, not listening, speech detection, take selection or fine synchronization.',
        'Digital full-scale sample counts do not measure intersample true peak or prove audible clipping.',
        'Quiet windows and energetic edges can be intentional and do not prove an audible defect.',
        'Results describe these candidate PCM cues only, not an encoded final video.',
    ],
    'cues': results,
}
(out / 'signal-triage.json').open('x').write(json.dumps(report, indent=2) + '\n')
print('Measured', len(results), 'complete PCM cues; full-scale samples in', sum(x['digital_full_scale_samples'] > 0 for x in results), 'cues.')
