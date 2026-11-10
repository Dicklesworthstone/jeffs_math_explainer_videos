"""Extract every frame of the silent two-fps native800 draft under the resource guard."""
import hashlib
import json
from pathlib import Path
import subprocess
from PIL import Image, ImageDraw

R = Path('/Users/jemanuel/projects/jeffs_math_explainer_videos')
W = R/'narration_lab/third-wave-20261024/root/five-film-release-20261027'
P = W/'exponential-native-preview800'
O = W/'exponential-native-pictures802'


def sha(p):
    with p.open('rb') as f:
        return hashlib.file_digest(f, 'sha256').hexdigest()


def main():
    receipt = json.loads((P/'resource-receipt.json').read_text())
    assert receipt['exit'] == 0 and receipt['finished'] and not receipt.get('resource_stop')
    manifest = W/'exponential-native-preview800-inputs.json'
    assert sha(manifest) == receipt['inputs_sha256'] == receipt['final_inputs_sha256']
    for row in json.loads(manifest.read_text())['files']:
        assert sha(Path(row['path'])) == row['sha256'], row['path']
    preview = json.loads((P/'preview-receipt.json').read_text())
    chapters = preview['chapters']
    assert len(chapters) == len(set(chapters)) == 15
    assert sorted(p.stem for p in (P/'render').glob('*.mp4')) == sorted(chapters)
    pictures, sheets = O/'pictures', O/'sheets'
    pictures.mkdir()
    sheets.mkdir()
    reports = []
    for chapter in chapters:
        movie = P/'render'/(chapter+'.mp4')
        cues = P/'render'/(chapter+'.cues.tsv')
        layout = P/'render'/(chapter+'.layout.tsv')
        probe = json.loads(subprocess.check_output([
            '/opt/homebrew/bin/ffprobe', '-v', 'error', '-show_streams', '-of', 'json', str(movie)], timeout=60))
        video = [s for s in probe['streams'] if s['codec_type'] == 'video']
        assert len(video) == 1 and not [s for s in probe['streams'] if s['codec_type'] == 'audio']
        v = video[0]
        assert (v['width'], v['height'], v['avg_frame_rate']) == (1920, 1080, '2/1')
        frames = int(v['nb_frames'])
        assert 1 <= frames <= 5000
        cue_rows = []
        for line in cues.read_text().splitlines():
            name, start, length = line.split('\t')
            cue_rows.append((name, float(start), float(length)))
        target = pictures/chapter
        target.mkdir()
        # Every decoded frame is kept, including every available lint/persistence frame.
        subprocess.run([
            '/opt/homebrew/bin/ffmpeg', '-nostdin', '-n', '-v', 'error', '-xerror', '-threads', '2',
            '-i', str(movie), '-an', '-threads', '2', '-fps_mode', 'passthrough', '-q:v', '3',
            str(target/'frame-%06d.jpg')], check=True, timeout=1800)
        files = sorted(target.glob('frame-*.jpg'))
        assert len(files) == frames
        chapter_sheets = []
        for start in range(0, frames, 8):
            im = Image.new('RGB', (1280, 1536), '#222222')
            draw = ImageDraw.Draw(im)
            for slot, path in enumerate(files[start:start+8]):
                index = start+slot
                at = index/2
                cue = ','.join(name for name, begin, length in cue_rows if begin <= at < begin+length) or 'hold/transition'
                x, y = (slot % 2)*640, (slot//2)*384
                with Image.open(path) as pic:
                    assert pic.size == (1920, 1080)
                    im.paste(pic.resize((640, 360), Image.Resampling.LANCZOS), (x, y+24))
                draw.text((x+5, y+5), f'{chapter} f{index} {at:.1f}s {cue}', fill='white')
            page = sheets/f'{chapter}-{start//8+1:03d}.jpg'
            im.save(page, quality=93)
            chapter_sheets.append(dict(path=str(page), sha256=sha(page), first_frame=start,
                                       last_frame=min(start+7, frames-1)))
        reports.append(dict(chapter=chapter, movie=str(movie), movie_sha256=sha(movie), frames=frames,
                            cues_sha256=sha(cues), layout_sha256=sha(layout), layout=layout.read_text(),
                            all_frames_extracted=True, full_video_decode=True, sheets=chapter_sheets,
                            pictures=[dict(path=str(p), sha256=sha(p)) for p in files]))
        print(f'{chapter}: decoded/extracted all {frames} draft frames, {len(chapter_sheets)} sheets; unviewed', flush=True)
    with (O/'pictures802.json').open('x') as f:
        json.dump(dict(preview_receipt_sha256=sha(P/'preview-receipt.json'), chapters=reports,
                       frames=sum(c['frames'] for c in reports),
                       scope='Every frame of the 2fps silent estimated-timing native draft. Extraction is not picture viewing, smooth-motion, narration, fine-sync or release acceptance.'), f, indent=2)


if __name__ == '__main__':
    main()
