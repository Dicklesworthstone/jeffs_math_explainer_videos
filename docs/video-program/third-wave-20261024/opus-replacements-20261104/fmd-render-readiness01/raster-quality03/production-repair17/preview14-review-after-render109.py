"""Prepare actual chapter14 review pictures after successful preview21; never approve them."""
from pathlib import Path
import hashlib
import json
import subprocess
import sys
import time

R = Path('/Users/jemanuel/projects/jeffs_math_explainer_videos')
Q = Path(__file__).resolve().parent
S = Q / 'preview21-14-review109'
B = R / 'narration_lab/third-wave-20261024/root/five-film-release-20261027'
J = B / 'complex-preview14-pictures109'
D = '/data/projects/jmev-stroke-probe-20261106/jobs/csd-production-preview21'
CHAPTER = '14_continuation'
FRAMES = 11756
GUARD = Q / 'guard_owner80g_target20g.py'


def sha(p):
    with p.open('rb') as f:
        return hashlib.file_digest(f, 'sha256').hexdigest()


def extract():
    from PIL import Image, ImageDraw
    r = json.loads((J / 'resource-receipt.json').read_text())
    assert r['admitted'] and 'exit' not in r
    for row in json.loads((Q / 'preview14-pictures109-inputs.json').read_text())['files']:
        assert sha(Path(row['path'])) == row['sha256']
    assert not (S / (CHAPTER + '.layout.tsv')).read_text().strip(), 'New layout findings need explicit sampling'
    picked = {f: ['one-second sample'] for f in range(0, FRAMES, 60)}
    picked[FRAMES - 1] = ['last frame']
    for line in (S / (CHAPTER + '.timeline.tsv')).read_text().splitlines():
        parts = line.split('\t')
        if parts[0] in ('line', 'mark'):
            for delta in (0, 30, 60, 120):
                frame = int(parts[2]) + delta
                if frame < FRAMES:
                    picked.setdefault(frame, []).append(parts[0] + ':' + parts[1])
    def balanced(xs):
        if len(xs) == 1:
            return xs[0]
        k = len(xs) // 2
        return '(' + balanced(xs[:k]) + '+' + balanced(xs[k:]) + ')'
    frames = sorted(picked)
    select = J / 'select.filter'
    select.write_text("select='" + balanced(['eq(n,' + str(f) + ')' for f in frames]) + "'")
    pictures = J / 'pictures'
    pictures.mkdir()
    subprocess.run(['/opt/homebrew/bin/ffmpeg', '-nostdin', '-n', '-v', 'error', '-xerror',
                    '-threads', '2', '-i', str(S / (CHAPTER + '.mp4')), '-an',
                    '-filter_script:v', str(select), '-vsync', '0', '-q:v', '2',
                    str(pictures / 'frame-%06d.jpg')], check=True, timeout=600)
    files = sorted(pictures.glob('frame-*.jpg'))
    assert len(files) == len(frames)
    samples = [dict(frame=f, seconds=f/60, reasons=picked[f], path=str(p), sha256=sha(p))
               for f, p in zip(frames, files)]
    sheets = []
    for k in range(0, len(samples), 8):
        canvas = Image.new('RGB', (1920, 2260), '#10151d')
        draw = ImageDraw.Draw(canvas)
        for i, sample in enumerate(samples[k:k+8]):
            with Image.open(sample['path']) as raw:
                im = raw.convert('RGB')
            assert im.size == (960, 540)
            x, y = (i % 2)*960, (i // 2)*565
            canvas.paste(im, (x, y))
            draw.text((x+5, y+542), f"{CHAPTER} frame {sample['frame']} {sample['seconds']:.3f}s", fill='white')
        p = J / f'sheet-{k//8+1:04d}.jpg'
        canvas.save(p, quality=94)
        sheets.append(dict(path=str(p), sha256=sha(p)))
    with (J / 'sampling.json').open('x') as f:
        json.dump(dict(movie_sha256=sha(S/(CHAPTER+'.mp4')), samples=samples, sheets=sheets,
                       scope='Unviewed selected preview stills. No visual, motion, audio, independent review or final-render approval.'), f, indent=2)


def wait_and_prepare():
    for _ in range(1440):
        result = subprocess.run(['ssh', '-o', 'BatchMode=yes', 'csd', 'cat '+D+'/resource-receipt.json'], capture_output=True)
        if result.returncode == 0:
            r = json.loads(result.stdout)
            if r.get('finished'):
                assert r.get('exit') == 0 and r['inputs_sha256'] == r['final_inputs_sha256']
                assert not any(r.get(k) for k in ('error', 'resource_stop', 'stopped_for_resource_limit'))
                break
        time.sleep(60)
    else:
        raise RuntimeError('Preview wait deadline; no extraction launched')
    S.mkdir()
    with (S / 'preview21-terminal.json').open('x') as f:
        json.dump(r, f, indent=2)
    rows = []
    for suffix in ('mp4', 'timeline.tsv', 'cues.tsv', 'cuelog.tsv', 'layout.tsv'):
        name = CHAPTER + '.' + suffix
        remote = D + '/preview/' + name
        before = subprocess.check_output(['ssh', 'csd', 'sha256sum '+remote]).decode().split()[0]
        subprocess.run(['scp', 'csd:'+remote, str(S/name)], check=True)
        after = subprocess.check_output(['ssh', 'csd', 'sha256sum '+remote]).decode().split()[0]
        assert before == after == sha(S/name)
        rows.append(dict(name=name, sha256=before))
    timeline = (S/(CHAPTER+'.timeline.tsv')).read_text()
    assert 'frames\t11756\n' in timeline and 'fps\t60\n' in timeline
    with (S/'copy-evidence.json').open('x') as f:
        json.dump(dict(files=rows, scope='Terminal stable preview21; actual picture and motion review still pending.'), f, indent=2)
    paths = {Path(__file__).resolve(), GUARD} | {p for p in S.iterdir() if p.is_file()}
    manifest = Q/'preview14-pictures109-inputs.json'
    with manifest.open('x') as f:
        json.dump({'files': [dict(path=str(p), sha256=sha(p)) for p in sorted(paths)]}, f, indent=2)
    subprocess.run(['python3', '-I', '-B', str(GUARD), '--phase', 'native', '--job', J.name,
                    '--inputs', str(manifest), '--', 'python3', '-I', '-B', str(Path(__file__).resolve()), '--extract'], check=True)


if __name__ == '__main__':
    extract() if '--extract' in sys.argv else wait_and_prepare()
