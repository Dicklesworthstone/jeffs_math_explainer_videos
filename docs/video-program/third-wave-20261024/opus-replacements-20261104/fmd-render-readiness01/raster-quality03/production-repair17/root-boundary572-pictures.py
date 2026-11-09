"""Extract actual boundary preview571 pictures under guard508; never grants visual acceptance."""
import hashlib
import json
from pathlib import Path
import re
import subprocess
from PIL import Image, ImageDraw

ROOT = Path('/Users/jemanuel/scratch/jmev-complex-native-20261109')
PREVIEW = ROOT/'jobs/mini-boundary-preview571'
JOB = ROOT/'jobs/mini-boundary-pictures572'
NARRATION = str(ROOT/'jobs/mini-boundary-delivery557/narration')
OLD = '/data/projects/jmev-stroke-probe-20261106/jobs/csd-review-repair-rebind313/narration'
EXPECTED = {
    'cues.tsv': 'eed5a50ad5797fa258ee3fcf9319e29946f403a86b3909090e73a3a15dc74249',
    'timeline.tsv': '5a91b8f9e2e5b5d6223a49285c06039b1b43bd16c11cecdd163a8e2e545baa9e',
    'cuelog.tsv': '7ed46c80cede7e49daeaf729337b24c66c8e7ccc125bb18ca6ce2c411d201bbf',
}


def sha(p):
    with p.open('rb') as f:
        return hashlib.file_digest(f, 'sha256').hexdigest()


def main():
    guard = json.loads((JOB/'resource-receipt.json').read_text())
    assert guard['admitted'] and guard['phase'] == 'native' and not guard['preflight_only']
    assert 'finished' not in guard and 'exit' not in guard
    assert sorted(p.name for p in JOB.iterdir()) == ['command.log', 'resource-receipt.json']
    receipt = json.loads((PREVIEW/'resource-receipt.json').read_text())
    assert receipt['exit'] == 0 and receipt['finished'] and not receipt.get('resource_stop')
    assert receipt['inputs_sha256'] == receipt['final_inputs_sha256']
    tables = {}
    for ext, expected in EXPECTED.items():
        p = PREVIEW/'preview'/('09_boundary.'+ext)
        raw = p.read_bytes()
        assert raw.count(NARRATION.encode()) == (0 if ext == 'cues.tsv' else 9)
        assert hashlib.sha256(raw.replace(NARRATION.encode(), OLD.encode())).hexdigest() == expected
        tables[ext] = sha(p)
    layout = PREVIEW/'preview/09_boundary.layout.tsv'
    assert sha(layout) == 'f91bb8a1bb6b7ea895414e739250922aa86eb02f9ecab57f851e95ee06b59a7e', 'Changed layout findings require a revised sampling plan'
    movie = PREVIEW/'preview/09_boundary.mp4'
    probe = json.loads(subprocess.check_output([
        '/opt/homebrew/bin/ffprobe', '-v', 'error', '-show_streams', '-of', 'json', str(movie)], timeout=60))
    video = [s for s in probe['streams'] if s['codec_type'] == 'video']
    assert len(video) == 1
    v = video[0]
    assert (v['width'], v['height'], v['avg_frame_rate'], int(v['nb_frames'])) == (960, 540, '60/1', 16504)
    subprocess.run(['/opt/homebrew/bin/ffmpeg', '-nostdin', '-v', 'error', '-xerror', '-threads', '2',
                    '-i', str(movie), '-f', 'null', '-'], check=True, timeout=600)
    select = ROOT/'reference558/select.filter'
    assert sha(select) == 'af3bf5a06eb73f44f7a7c83a0889e92849bfa5d9b02ac48658b4c49964a90d9c'
    frames = [int(x) for x in re.findall(r'eq\(n\\?,\s*(\d+)\)', select.read_text())]
    assert len(frames) == len(set(frames)) == 313, frames
    assert frames == sorted(frames) and min(frames) >= 0 and max(frames) < 16504
    pictures = JOB/'pictures'
    pictures.mkdir()
    subprocess.run(['/opt/homebrew/bin/ffmpeg', '-nostdin', '-n', '-v', 'error', '-xerror', '-threads', '2',
                    '-i', str(movie), '-an', '-filter_script:v', str(select), '-fps_mode', 'passthrough',
                    '-q:v', '2', str(pictures/'frame-%06d.jpg')], check=True, timeout=600)
    files = sorted(pictures.glob('frame-*.jpg'))
    assert len(files) == 313
    sheets = JOB/'sheets'
    sheets.mkdir()
    rows = []
    for start in range(0, len(files), 8):
        sheet = Image.new('RGB', (1280, 1536), '#222222')
        draw = ImageDraw.Draw(sheet)
        for slot, p in enumerate(files[start:start+8]):
            index = start+slot
            x, y = (slot % 2)*640, (slot//2)*384
            with Image.open(p) as im:
                assert im.size == (960, 540)
                sheet.paste(im.resize((640, 360), Image.Resampling.LANCZOS), (x, y+24))
            draw.text((x+8, y+5), f'{index+1:03d} | frame {frames[index]} | {frames[index]/60:.3f}s', fill='white')
        p = sheets/f'sheet-{start//8+1:03d}.jpg'
        sheet.save(p, quality=95)
        rows.append({'path': str(p), 'sha256': sha(p), 'first_picture': start+1,
                     'last_picture': min(start+8, len(files))})
    result = {'movie': str(movie), 'movie_sha256': sha(movie), 'frames': 16504, 'tables': tables,
              'full_decode': True, 'selected_frames': frames, 'sheets': rows,
              'pictures': [{'path': str(p), 'sha256': sha(p)} for p in files],
              'scope': 'Technical checks and extraction only. No picture viewing, listening, fine sync or independent AV acceptance.'}
    with (JOB/'pictures572.json').open('x') as f:
        json.dump(result, f, indent=2)
        f.write('\n')
    print('572: exact tables; full decode; 313 selected pictures on 40 sheets; not yet viewed')


if __name__ == '__main__':
    main()
