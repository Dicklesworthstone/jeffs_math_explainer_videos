"""Keyframes only, whole film, low priority: where does ink sit in the rows a proxy caption would cover?

Coarse map for finding footers, not a frame-accurate measure and not picture review.
"""
import json
from pathlib import Path
import re
import subprocess
import sys

O = Path('narration_lab/third-wave-20261024/root/elevenlabs-logarithm-rebuild-20261104/remote-mac-mini-max01')
MASTER = O / 'jobs/logarithm-remote-render-work01/logarithm_mysteries_master.mp4'
OUT = Path(sys.argv[1])
TOP, HIGH = 1830, 300
WIDTH, HEIGHT = 960, 75
command = ['nice', '-n', '10', '/opt/homebrew/Cellar/ffmpeg/8.1.2_1/bin/ffmpeg', '-hide_banner', '-threads', '2', '-skip_frame', 'nokey', '-i', str(MASTER),
           '-map', '0:v:0', '-fps_mode', 'passthrough',
           '-vf', f'crop=3840:{HIGH}:0:{TOP},scale={WIDTH}:{HEIGHT}:flags=area,format=gray,showinfo', '-f', 'rawvideo', 'pipe:1']
done = subprocess.run(command, capture_output=True, timeout=500)
assert done.returncode == 0, done.stderr[-500:]
times = [float(x) for x in re.findall(rb'pts_time:([0-9.]+)', done.stderr)]
size = WIDTH * HEIGHT
frames = len(done.stdout) // size
print('keyframes', frames, 'times', len(times))
rows = []
for index in range(min(frames, len(times))):
    frame = done.stdout[index * size:(index + 1) * size]
    per_row = [sum(1 for value in frame[y * WIDTH:(y + 1) * WIDTH] if value >= 45) for y in range(HEIGHT)]
    rows.append({'t': times[index], 'rows': per_row})
OUT.write_text(json.dumps(rows))
