"""Verify final555 native 4K chapter and extract actual full-resolution samples under guard508; never grants audiovisual acceptance."""
import hashlib
import json
from pathlib import Path
import re
import subprocess
from PIL import Image, ImageDraw

ROOT = Path('/Users/jemanuel/scratch/jmev-complex-native-20261109')
PREVIEW = ROOT/'jobs/mini-final555'
JOB = ROOT/'jobs/mini-final-pictures560'
NARRATION = str(ROOT/'jobs/mini-delivery540/narration')
OLD = '/data/projects/jmev-stroke-probe-20261106/jobs/csd-review-repair-rebind269/narration'
EXPECTED = {
    'cues.tsv': 'a58e14f5f22089df8902439f751b0a05cfe52b29502f7c530054745d6f7a6ca3',
    'timeline.tsv': '99fd22e62d5c6e53104ec3c622135d0748c3dc68e375141d3da4a38114d5860b',
    'cuelog.tsv': '11bc0949b31f6483f0a5df01d2c3c71485185384c5cc41c9ba8a6cd6df14bdde',
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
    # Bind this verification to the successful render inputs, not just its filename.
    manifest = ROOT/'qualification/root-mini-final555-inputs.json'
    assert sha(manifest) == receipt['inputs_sha256']
    for row in json.loads(manifest.read_text())['files']:
        assert sha(Path(row['path'])) == row['sha256'], row['path']
    tables = {}
    stem = ROOT/'inputs523/score21/10_rigidity.music.f32.wav'
    stem_sha = '3307c2c9bddcb58d2c21899f1fcc68a836d815d71e430de179fd41cdcf1e0083'
    assert sha(stem) == stem_sha and stem.stat().st_size == 47628880
    for ext, expected in EXPECTED.items():
        p = PREVIEW/'render'/('10_rigidity.'+ext)
        raw = p.read_bytes()
        if ext == 'cuelog.tsv':
            lines = raw.splitlines(keepends=True)
            stem_lines = [line for line in lines if line.split(b'\t', 1)[0] == str(stem).encode()]
            assert len(stem_lines) == 1, 'Final must contain exactly the approved score stem'
            row = stem_lines[0].decode().rstrip('\n').split('\t')
            assert len(row) == 13 and row[1] == stem_sha and int(row[2]) == 47628880
            assert row[4] == '60' and float(row[7]) == 0.0
            assert row[9:12] == ['48000', '2', '5953600'] and int(row[12]) == 0
            raw = b''.join(line for line in lines if line not in stem_lines)
        assert raw.count(NARRATION.encode()) == (0 if ext == 'cues.tsv' else 6)
        assert hashlib.sha256(raw.replace(NARRATION.encode(), OLD.encode())).hexdigest() == expected
        tables[ext] = sha(p)
    layout = PREVIEW/'render/10_rigidity.layout.tsv'
    assert layout.read_bytes() == b'', 'New layout findings require a revised sampling plan'
    movie = PREVIEW/'render/10_rigidity.mp4'
    probe = json.loads(subprocess.check_output([
        '/opt/homebrew/bin/ffprobe', '-v', 'error', '-show_streams', '-of', 'json', str(movie)], timeout=60))
    video = [s for s in probe['streams'] if s['codec_type'] == 'video']
    assert len(video) == 1
    v = video[0]
    assert (v['width'], v['height'], v['avg_frame_rate'], int(v['nb_frames'])) == (3840, 2160, '60/1', 7442)
    subprocess.run(['/opt/homebrew/bin/ffmpeg', '-nostdin', '-v', 'error', '-xerror', '-threads', '2',
                    '-i', str(movie), '-f', 'null', '-'], check=True, timeout=1800)
    select = ROOT/'inputs523/reference/10_rigidity.select.filter'
    frames = [int(x) for x in re.findall(r'eq\(n\\?,\s*(\d+)\)', select.read_text())]
    assert len(frames) == len(set(frames)) == 113, frames
    assert frames == sorted(frames) and min(frames) >= 0 and max(frames) < 7442
    pictures = JOB/'pictures'
    pictures.mkdir()
    subprocess.run(['/opt/homebrew/bin/ffmpeg', '-nostdin', '-n', '-v', 'error', '-xerror', '-threads', '2',
                    '-i', str(movie), '-an', '-filter_script:v', str(select), '-fps_mode', 'passthrough',
                    '-q:v', '2', str(pictures/'frame-%06d.jpg')], check=True, timeout=1800)
    files = sorted(pictures.glob('frame-*.jpg'))
    assert len(files) == 113
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
                assert im.size == (3840, 2160)
                sheet.paste(im.resize((640, 360), Image.Resampling.LANCZOS), (x, y+24))
            draw.text((x+8, y+5), f'{index+1:03d} | frame {frames[index]} | {frames[index]/60:.3f}s', fill='white')
        p = sheets/f'sheet-{start//8+1:03d}.jpg'
        sheet.save(p, quality=95)
        rows.append({'path': str(p), 'sha256': sha(p), 'first_picture': start+1,
                     'last_picture': min(start+8, len(files))})
    result = {'movie': str(movie), 'movie_sha256': sha(movie), 'frames': 7442, 'tables': tables,
              'full_decode': True, 'ffprobe': probe, 'selected_frames': frames, 'sheets': rows,
              'pictures': [{'path': str(p), 'sha256': sha(p)} for p in files],
              'scope': 'Technical checks and extraction only. No picture viewing, listening, fine sync or independent AV acceptance.'}
    with (JOB/'pictures560.json').open('x') as f:
        json.dump(result, f, indent=2)
        f.write('\n')
    print('560: exact tables; full decode; 113 selected pictures on 15 sheets; not yet viewed')


if __name__ == '__main__':
    main()
