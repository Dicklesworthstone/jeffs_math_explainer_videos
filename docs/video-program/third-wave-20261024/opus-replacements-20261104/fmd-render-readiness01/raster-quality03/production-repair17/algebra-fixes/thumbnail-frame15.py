from pathlib import Path
import hashlib
import json
import subprocess

R = Path.cwd()
B = R / 'narration_lab/third-wave-20261024/root/five-film-release-20261027'
J = B / 'algebra-final14-05_order'
O = B / 'algebra-thumbnail-frame15'
r = json.loads((J / 'resource-receipt.json').read_text())
assert r['exit'] == 0 and not r.get('resource_stop')
assert r['inputs_sha256'] == r['final_inputs_sha256']
movie = J / 'render/05_order.mp4'
frame = 2700
output = O / 'abstract_algebra_thumbnail.png'
subprocess.run([
    'ffmpeg', '-nostdin', '-n', '-v', 'error', '-xerror', '-threads', '2',
    '-i', str(movie), '-an', '-vf', f'select=eq(n\\,{frame})',
    '-frames:v', '1', str(output),
], check=True, timeout=1800)
probe = json.loads(subprocess.check_output([
    'ffprobe', '-v', 'error', '-show_streams', '-of', 'json', str(output)
]))['streams'][0]
assert (probe['width'], probe['height']) == (3840, 2160)
(O / 'thumbnail-result.json').write_text(json.dumps({
    'source': str(movie), 'source_sha256': hashlib.sha256(movie.read_bytes()).hexdigest(),
    'frame': frame, 'seconds': frame / 60, 'image': str(output),
    'image_sha256': hashlib.sha256(output.read_bytes()).hexdigest(),
    'viewed': False,
    'scope': 'Unmodified native final frame, candidate thumbnail. Actual visual review pending.',
}, indent=2))
