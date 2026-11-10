"""Verify/extract actual boundary final772 4K pictures under guard508; never grants visual acceptance."""
import hashlib
import json
from pathlib import Path
import re
import subprocess
from PIL import Image, ImageDraw

ROOT = Path('/Users/jemanuel/scratch/jmev-complex-native-20261109')
PREVIEW = Path('/Users/jemanuel/projects/jeffs_math_explainer_videos/narration_lab/third-wave-20261024/root/five-film-release-20261027/complex-local-final819')
JOB = Path('/Users/jemanuel/projects/jeffs_math_explainer_videos/narration_lab/third-wave-20261024/root/five-film-release-20261027/complex-local-pictures820')
NARRATION = '/Users/jemanuel/projects/jeffs_math_explainer_videos/narration_lab/third-wave-20261024/root/five-film-release-20261027/complex-local-delivery818/narration'
OLD = '/data/projects/jmev-stroke-probe-20261106/jobs/csd-review-repair-rebind313/narration'
EXPECTED = {
    'cues.tsv': 'eed5a50ad5797fa258ee3fcf9319e29946f403a86b3909090e73a3a15dc74249',
    'timeline.tsv': '5a91b8f9e2e5b5d6223a49285c06039b1b43bd16c11cecdd163a8e2e545baa9e',
    'cuelog.tsv': '7ed46c80cede7e49daeaf729337b24c66c8e7ccc125bb18ca6ce2c411d201bbf',
}


def sha(p):
    with p.open('rb') as f:
        return hashlib.file_digest(f, 'sha256').hexdigest()


def balanced_sum(terms):
    assert terms
    if len(terms) == 1:
        return terms[0]
    middle = len(terms)//2
    return '('+balanced_sum(terms[:middle])+'+'+balanced_sum(terms[middle:])+')'


def main():
    guard = json.loads((JOB/'resource-receipt.json').read_text())
    assert guard['admitted'] and guard['phase'] == 'native' and not guard['preflight_only']
    assert 'finished' not in guard and 'exit' not in guard
    assert sorted(p.name for p in JOB.iterdir()) == ['command.log', 'resource-receipt.json']
    receipt = json.loads((PREVIEW/'resource-receipt.json').read_text())
    assert receipt['exit'] == 0 and receipt['finished'] and not receipt.get('resource_stop') and not receipt.get('stopped_for_resource_limit')
    assert receipt['inputs_sha256'] == receipt['final_inputs_sha256']
    manifest = Path('/Users/jemanuel/projects/jeffs_math_explainer_videos/docs/video-program/third-wave-20261024/opus-replacements-20261104/fmd-render-readiness01/raster-quality03/production-repair17/local-final819-inputs.json')
    assert sha(manifest) == receipt['inputs_sha256']
    for row in json.loads(manifest.read_text())['files']:
        assert sha(Path(row['path'])) == row['sha256'], row['path']
    parts = ROOT/'score586/09_boundary.music.f32.parts.tsv'
    assert sha(parts) == 'db31e88404eb9af836c6575cf5b4178a292eaeceea283d717bf2a4a7166ffe2e'
    stems = {}
    offset = 0
    for line in parts.read_text().splitlines()[1:]:
        start, count, rate, digest, name = line.split('\t')
        assert int(start) == offset and int(rate) == 48000 and Path(name).name == name
        p = parts.parent/name
        assert sha(p) == digest
        stems[str(p)] = (digest, p.stat().st_size, int(count), int(start))
        offset += int(count)
    assert len(stems) == 3 and offset == 16504*800
    tables = {}
    for ext, expected in EXPECTED.items():
        p = PREVIEW/'render'/('09_boundary.'+ext)
        raw = p.read_bytes()
        if ext == 'cuelog.tsv':
            kept, found = [], []
            for line in raw.splitlines(keepends=True):
                row = line.decode().rstrip('\n').split('\t')
                if row[0] not in stems:
                    kept.append(line)
                    continue
                digest, size, count, start = stems[row[0]]
                assert len(row) == 13 and row[1] == digest and int(row[2]) == size
                assert row[4] == '60' and float(row[7]) == 0.0
                assert row[9:11] == ['48000', '2'] and int(row[11]) == count and int(row[12]) == start
                found.append(row[0])
            assert len(found) == 3 and sorted(found) == sorted(stems)
            raw = b''.join(kept)
        assert raw.count(NARRATION.encode()) == (0 if ext == 'cues.tsv' else 9)
        assert hashlib.sha256(raw.replace(NARRATION.encode(), OLD.encode())).hexdigest() == expected
        tables[ext] = sha(p)
    layout = PREVIEW/'render/09_boundary.layout.tsv'
    movie = PREVIEW/'render/09_boundary.mp4'
    probe = json.loads(subprocess.check_output([
        '/opt/homebrew/bin/ffprobe', '-v', 'error', '-show_streams', '-of', 'json', str(movie)], timeout=60))
    video = [s for s in probe['streams'] if s['codec_type'] == 'video']
    assert len(video) == 1
    v = video[0]
    assert (v['width'], v['height'], v['avg_frame_rate'], int(v['nb_frames'])) == (3840, 2160, '60/1', 16504)
    subprocess.run(['/opt/homebrew/bin/ffmpeg', '-nostdin', '-v', 'error', '-xerror', '-threads', '2',
                    '-i', str(movie), '-f', 'null', '-'], check=True, timeout=1800)
    select = ROOT/'reference558/select.filter'
    assert sha(select) == 'af3bf5a06eb73f44f7a7c83a0889e92849bfa5d9b02ac48658b4c49964a90d9c'
    frames = [int(x) for x in re.findall(r'eq\(n\\?,\s*(\d+)\)', select.read_text())]
    assert len(frames) == len(set(frames)) == 313, frames
    assert frames == sorted(frames) and min(frames) >= 0 and max(frames) < 16504
    # Retain every actually reviewed preview sample; add all final lint anchors.
    prior = ROOT/'jobs/mini-boundary-pictures775/pictures775.json'
    assert sha(prior) == '2a1864771d82f9edb436823e100a3c6b97f8421e272a87bd7c2a8c95c1b4cf99'
    report775 = json.loads(prior.read_text())
    original = set(frames)
    assert len(report775['selected_frames']) == 814
    chosen = set(report775['selected_frames'])
    assert original <= chosen
    anchors = []
    for row in layout.read_text().splitlines():
        match = re.search(r'@([0-9.]+)s', row)
        assert match, row
        at = round(float(match[1])*60)
        assert 0 <= at < 16504
        anchors.append(dict(frame=at, row=row))
        chosen.update(range(at, min(at+121, 16504), 12))
        chosen.update(range(at, 16504, 300))
    frames = sorted(chosen)
    assert 814 <= len(frames) < 1500 and frames[0] >= 0 and frames[-1] < 16504
    select = JOB/'select820.filter'
    with select.open('x') as f:
        f.write("select='" + balanced_sum([f'eq(n\\,{n})' for n in frames]) + "'\n")
    pictures = JOB/'pictures'
    pictures.mkdir()
    subprocess.run(['/opt/homebrew/bin/ffmpeg', '-nostdin', '-n', '-v', 'error', '-xerror', '-threads', '2',
                    '-i', str(movie), '-an', '-filter_script:v', str(select), '-fps_mode', 'passthrough',
                    '-q:v', '2', str(pictures/'frame-%06d.jpg')], check=True, timeout=1800)
    files = sorted(pictures.glob('frame-*.jpg'))
    assert len(files) == len(frames)
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
    result = {'movie': str(movie), 'movie_sha256': sha(movie), 'frames': 16504, 'tables': tables,
              'full_decode': True, 'layout_sha256': sha(layout), 'layout_anchors': anchors,
              'preview_sampling_sha256': sha(prior), 'ffprobe': probe, 'selected_frames': frames, 'sheets': rows,
              'pictures': [{'path': str(p), 'sha256': sha(p)} for p in files],
              'scope': 'Technical checks and extraction only. No picture viewing, listening, fine sync or independent AV acceptance.'}
    with (JOB/'pictures820.json').open('x') as f:
        json.dump(result, f, indent=2)
        f.write('\n')
    print(f'820: exact scored tables; full decode; {len(files)} final4K pictures on {len(rows)} sheets; not yet viewed')


if __name__ == '__main__':
    main()
