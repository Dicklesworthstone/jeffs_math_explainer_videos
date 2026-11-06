"""Native 4K colour diagnostic at one frame per second; no narration/final acceptance."""
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys

ROOT = Path('/data/projects/jmev-fmd-render-qualification-20261106-1111')
JOB = ROOT/'jobs/fmd-raster-native-detail10'
BIN = ROOT/'work/bin/production09h'
PIN = json.loads((ROOT/'jobs/fmd-raster-quality-build04/qualification.json').read_text())['binary_sha256']
def read(p): return json.loads(p.read_text())
def sha(p):
    with p.open('rb') as f: return hashlib.file_digest(f, 'sha256').hexdigest()

assert sys.argv[1:] == ['--run']
admit = read(JOB/'resource-receipt.json')
assert admit['admitted'] and 'exit' not in admit
assert sha(BIN) == PIN
stop_job = ROOT/'jobs/fmd-raster-encoder-stop10'
receipt = read(stop_job/'resource-receipt.json')
assert receipt['exit'] == 0 and receipt['inputs_sha256'] == receipt['final_inputs_sha256'] and not receipt.get('resource_stop')
stop = read(stop_job/'stop-summary.json')
assert stop['binary_sha256'] == PIN and stop['pass'] is True
assert len(stop['tests']) == 2 and all(t['pass'] and not t['survivors'] and t['native_exit'] < 0 for t in stop['tests'])
out = JOB/'render'
assert not out.exists()
os.environ.update(TMPDIR=str(ROOT/'work/tmp'), XDG_CACHE_HOME=str(ROOT/'work/cache'))
command = [str(BIN), 'render', '04_reading_color', '--silent', '--res', '3840x2160', '--fps', '1',
           '--out', str(out), '--crf', '0', '--preset', 'veryfast']
with (JOB/'native.stdout').open('xb') as stdout, (JOB/'native.stderr').open('xb') as stderr:
    subprocess.run(command, cwd=ROOT/'work/probe-cwd', stdout=stdout, stderr=stderr, check=True, timeout=7200)
movie = out/'04_reading_color.mp4'
probe = json.loads(subprocess.check_output(['ffprobe', '-v', 'error', '-show_streams', '-of', 'json', str(movie)], timeout=120))
video = [s for s in probe['streams'] if s['codec_type'] == 'video']
assert len(video) == 1
stream = video[0]
assert (stream['width'], stream['height'], stream['avg_frame_rate']) == (3840, 2160, '1/1')
frames = int(stream['nb_frames'])
assert 1 < frames < 1000
images = JOB/'native-pictures'; images.mkdir()
subprocess.run(['ffmpeg', '-nostdin', '-n', '-v', 'error', '-xerror', '-threads', '2', '-i', str(movie),
                '-vsync', '0', '-threads', '2', str(images/'frame-%04d.png')], check=True, timeout=1800)
paths = sorted(images.glob('*.png'))
assert len(paths) == frames
rows = []
for index, path in enumerate(paths):
    with path.open('rb') as f: header = f.read(24)
    assert header[:8] == b'\x89PNG\r\n\x1a\n'
    assert (int.from_bytes(header[16:20], 'big'), int.from_bytes(header[20:24], 'big')) == (3840,2160)
    rows.append({'frame': index, 'seconds': index, 'path': str(path), 'sha256': sha(path), 'viewed': False})
with (JOB/'detail-summary.json').open('x') as f:
    json.dump({'binary_sha256': PIN, 'movie_sha256': sha(movie), 'frames': frames, 'pictures': rows,
               'scope': 'Actual native4K silent colour-field samples at1fps. No narrated timing, motion, full-film, listening or independent AV acceptance.'}, f, indent=2)
print('Native4K diagnostic extracted; actual image review still required.')
