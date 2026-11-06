"""Small read-only measurement on the delivery copy of the master: when do low rows gain and lose ink?

Decodes only short stretches around each expected change, at low priority, into a pipe. Writes
one JSON file in scratch. This is measurement for choosing samples, not picture review.
"""
import hashlib
import json
from pathlib import Path
import subprocess
import sys

O = Path('narration_lab/third-wave-20261024/root/elevenlabs-logarithm-rebuild-20261104/remote-mac-mini-max01')
MASTER = O / 'jobs/logarithm-remote-render-work01/logarithm_mysteries_master.mp4'
OUT = Path(sys.argv[1])
TOP, HIGH, SHOWN = 1500, 660, (960, 165)
ROWS = {'-3.40': 1998, '-2.93': 1871, '-2.50': 1755, '-1.85': 1579}
RUNS = [(1483.2, 2.0), (1496.6, 1.2), (1546.8, 1.8), (1565.0, 1.6), (1719.0, 2.0), (1750.0, 2.6), (1761.6, 1.6), (1765.9, 1.6),
        (1772.6, 1.0), (1929.2, 1.6), (1930.8, 2.4), (1933.2, 2.2), (1936.5, 1.4), (1942.4, 2.2)]

with MASTER.open('rb') as handle:
    digest = hashlib.file_digest(handle, 'sha256').hexdigest()
assert digest.startswith('8534ec21'), digest
width, height = SHOWN
size = width * height
result = []
for start, span in RUNS:
    command = ['nice', '-n', '10', '/opt/homebrew/Cellar/ffmpeg/8.1.2_1/bin/ffmpeg', '-hide_banner', '-loglevel', 'error', '-threads', '2',
               '-ss', f'{start:.3f}', '-t', f'{span:.3f}', '-i', str(MASTER), '-map', '0:v:0',
               '-vf', f'crop=3840:{HIGH}:0:{TOP},scale={width}:{height}:flags=area,format=gray', '-f', 'rawvideo', 'pipe:1']
    raw = subprocess.run(command, check=True, capture_output=True, timeout=300).stdout
    frames = len(raw) // size
    rows = {}
    for label, pixel in ROWS.items():
        first = (pixel - 32 - TOP) * height // HIGH
        last = (pixel + 32 - TOP) * height // HIGH
        series = []
        for index in range(frames):
            frame = raw[index * size:(index + 1) * size]
            band = b''.join(frame[y * width:(y + 1) * width] for y in range(first, last))
            series.append(sum(band))
        rows[label] = series
    result.append({'start': start, 'span': span, 'frames': frames, 'rows': rows})
    print(start, span, frames, flush=True)
OUT.write_text(json.dumps(result))
