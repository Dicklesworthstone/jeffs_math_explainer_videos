from pathlib import Path
import hashlib, json, subprocess
from PIL import Image, ImageDraw

R = Path('/Users/jemanuel/projects/jeffs_math_explainer_videos')
Q = R / 'docs/video-program/third-wave-20261024/opus-replacements-20261104/fmd-render-readiness01/raster-quality03/production-repair17'
S = Q / 'preview21-08-review'
J = R / 'narration_lab/third-wave-20261024/root/five-film-release-20261027/complex-preview08-pictures27'
def sha(p):
    with p.open('rb') as f:
        return hashlib.file_digest(f, 'sha256').hexdigest()
r = json.loads((J / 'resource-receipt.json').read_text())
assert r['admitted'] and 'exit' not in r
evidence = json.loads((S / 'copy-evidence.json').read_text())
for item in evidence['files']:
    assert sha(S / item['name']) == item['sha256']
picked = {f: ['one-second sample'] for f in range(0, 14959, 60)}
for line in (S / '08_loop_sums.timeline.tsv').read_text().splitlines():
    parts = line.split('\t')
    if parts[0] in ('line', 'mark'):
        for delta in (0, 30, 60, 120):
            f = int(parts[2]) + delta
            if f < 14959:
                picked.setdefault(f, []).append(parts[0] + ':' + parts[1])
def balanced(xs):
    if len(xs) == 1:
        return xs[0]
    k = len(xs) // 2
    return '(' + balanced(xs[:k]) + '+' + balanced(xs[k:]) + ')'
frames = sorted(picked)
filter_path = J / 'select.filter'
filter_path.write_text("select='" + balanced(['eq(n,' + str(f) + ')' for f in frames]) + "'")
pictures = J / 'pictures'
pictures.mkdir()
subprocess.run(['/opt/homebrew/bin/ffmpeg', '-nostdin', '-n', '-v', 'error', '-xerror', '-threads', '2', '-i', str(S / '08_loop_sums.mp4'), '-an', '-filter_script:v', str(filter_path), '-vsync', '0', '-q:v', '2', str(pictures / 'frame-%06d.jpg')], check=True, timeout=300)
files = sorted(pictures.glob('frame-*.jpg'))
assert len(files) == len(frames)
samples = [dict(frame=f, seconds=f/60, reasons=picked[f], path=str(p), sha256=sha(p)) for f,p in zip(frames, files)]
sheets = []
for k in range(0, len(samples), 8):
    canvas = Image.new('RGB', (1920, 2260), '#10151d')
    draw = ImageDraw.Draw(canvas)
    for i, sample in enumerate(samples[k:k+8]):
        im = Image.open(sample['path']).convert('RGB')
        assert im.size == (960, 540)
        x,y = (i%2)*960, (i//2)*565
        canvas.paste(im, (x,y))
        draw.text((x+5,y+542), f"08_loop_sums frame {sample['frame']} {sample['seconds']:.3f}s", fill='white')
    p = J / f'sheet-{k//8+1:04d}.jpg'
    canvas.save(p, quality=94)
    sheets.append(dict(path=str(p), sha256=sha(p)))
(J / 'sampling.json').write_text(json.dumps(dict(movie_sha256=sha(S/'08_loop_sums.mp4'), samples=samples, sheets=sheets, scope='Completed third preview chapter only. Overall seven-chapter job still active. Selected 540p stills; not final 4K, motion, listening or independent AV acceptance.'), indent=2)+'\n')
