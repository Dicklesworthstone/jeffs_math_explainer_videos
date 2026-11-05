"""Extract review pictures from the optimized narrated preview, never claim viewing.

Root's pictures-after-preview01.py with new names; diff the two to audit it. Modes: none waits and
launches, --check reads only, --extract is the guard's child.
"""
import importlib.util
import json
from pathlib import Path
import re
import shutil
import subprocess
import sys
import wave
from PIL import Image, ImageDraw

O = Path(__file__).resolve().parent
SPEC = importlib.util.spec_from_file_location('complex_preview', O / 'preview_optimized01.py')
P = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(P)
T = P.T
JOB = 'complex-stage06-optimized-pictures01'
OUT = T.B / JOB / 'pictures'
MANIFEST = P.S.STAGE / 'optimized-picture-inputs01.json'
FFMPEG = Path(shutil.which('ffmpeg')).resolve()


def layout_paths(chapter, required_folders, optional_folders=()):
    """Narrated renders must lint; native timeline-only runs do not install a linter."""
    required = [folder / (chapter + '.layout.tsv') for folder in required_folders]
    for path in required:
        assert path.is_file(), 'Required rendered layout report missing: ' + str(path)
    optional = [folder / (chapter + '.layout.tsv') for folder in optional_folders]
    return required + [path for path in optional if path.is_file()]


def select(chapter, rows, frames, phrases, sentences, layouts):
    selected = {}
    def add(seconds, reason):
        frame = round(seconds * 60)
        if 0 <= frame < frames:
            selected.setdefault(frame, set()).add(reason)
    for sec in range(1, frames // 60, 5):
        add(sec, 'five-second chapter progression')
    add((frames - 1) / 60, 'last frame')
    for row in rows:
        if row[0] == 'line':
            cue, start, wav = row[1], int(row[2]) / 60, Path(row[4])
            assert T.sha(wav) == phrases[cue]['wav_sha256'], cue
            with wave.open(str(wav)) as handle:
                duration = handle.getnframes() / handle.getframerate()
            for delta, label in [(-.2, 'before'), (.2, 'start'), (duration / 2, 'middle'),
                                 (max(0, duration - .1), 'end')]:
                add(start + delta, cue + ' ' + label)
            for index, sentence in enumerate(sentences[cue]):
                add(start + sentence['start'] + .5, f'{cue} provisional sentence {index+1}')
            for mark in phrases[cue]['marks']:
                for delta in (-.2, .5, 1.5):
                    add(start + mark['seconds'] + delta, f"{cue} provisional phrase byte{mark['byte']} +{delta}")
        elif row[0] == 'breath':
            begin, duration = int(row[1]) / 60, float(row[2])
            for delta in (0, duration / 2, max(0, duration - .1), duration + .5):
                add(begin + delta, 'authored hold +' + str(delta))
    for path in layouts:
        for line in path.read_text().splitlines():
            if not line.strip():
                continue
            match = re.search(r'@([0-9]+(?:\.[0-9]+)?)s', line)
            assert match, (path, line)
            anchor = float(match[1])
            deltas = [-.2, 0, .2, 1, 3, 6, 10]
            persistent = re.search(r'still there at (\d+) later looks', line)
            if persistent:
                count = int(persistent[1])
                deltas += [1 + .2 * count, 2 + .2 * count]
                deltas += list(range(5, max(5, int(frames / 60 - anchor) + 1), 5))
            for delta in deltas:
                add(anchor + delta, f'{path.name} {line} +{delta}s')
    return [{'chapter': chapter, 'frame': frame, 'seconds': frame / 60,
             'reasons': sorted(reasons)} for frame, reasons in sorted(selected.items())]


def extract():
    receipt = T.read(T.B / JOB / 'resource-receipt.json')
    assert receipt.get('admitted') is True and 'exit' not in receipt
    assert receipt['command'] == [sys.executable, '-I', '-B', str(Path(__file__).resolve()), '--extract']
    assert receipt['policy'] == T.POLICY, 'Guard policy changed'
    assert T.success(P.JOB)
    for row in T.read(MANIFEST)['files']:
        assert T.sha(Path(row['path'])) == row['sha256'], row['path']
    assert not OUT.exists(), 'Preserve earlier review pictures'
    OUT.mkdir()
    phrases = T.read(T.N / 'phrase-timings.json')['cues']
    sentences = T.read(T.N / 'report.json')['lines']
    summary = T.read(T.B / P.JOB / 'preview-summary.json')
    assert [row['chapter'] for row in summary['chapters']] == list(T.CHAPTERS)
    all_samples = []
    layout_sources = {}
    for chapter_info in summary['chapters']:
        chapter = chapter_info['chapter']
        movie, table = P.OUT / (chapter + '.mp4'), P.OUT / (chapter + '.timeline.tsv')
        assert T.sha(movie) == chapter_info['mp4_sha256'] and T.sha(table) == chapter_info['timeline_sha256']
        rows = [line.split('\t') for line in P.normalized(table)]
        layouts = layout_paths(chapter, [P.OUT])
        layout_sources[chapter] = [str(path) for path in layouts]
        samples = select(chapter, rows, chapter_info['frames'], phrases, sentences, layouts)
        folder = OUT / chapter
        folder.mkdir()
        for index, sample in enumerate(samples):
            dst = folder / f'{index:04}.png'
            subprocess.run([str(FFMPEG), '-v', 'error', '-xerror', '-nostdin', '-threads', '2',
                            '-ss', str(sample['seconds']), '-i', str(movie), '-frames:v', '1',
                            '-threads', '2', '-n', str(dst)], check=True, timeout=90)
            sample.update(picture=str(dst), sha256=T.sha(dst))
        for first in range(0, len(samples), 6):
            sheet = Image.new('RGB', (1920, 1740), '#0f0f14')
            draw = ImageDraw.Draw(sheet)
            for slot, sample in enumerate(samples[first:first+6]):
                x, y = slot % 2 * 960, slot // 2 * 580
                with Image.open(sample['picture']) as im:
                    assert im.size == (960, 540)
                    sheet.paste(im, (x, y))
                draw.text((x+8, y+545), f"{first+slot:04} {chapter} {sample['seconds']:.3f}s", fill='white')
                draw.text((x+8, y+560), ' | '.join(sample['reasons'])[:150], fill='white')
            sheet.save(folder / f'sheet-{first//6:03}.png')
        all_samples += samples
        print(chapter, len(samples), 'pictures extracted; actual viewing pending', flush=True)
    with (OUT / 'extraction.json').open('x') as handle:
        json.dump({'samples': all_samples, 'pictures_viewed': False, 'layout_sources': layout_sources,
                   'timeline_lint_note': 'Rendered preview lint reports only; timeline-only mode installs no linter.',
                   'scope': 'Preview samples only; all full reason labels retained. Persistence looks can be nonconsecutive. Not exhaustive review, listening, fine sync or independent acceptance.'}, handle, indent=2)


def wait_and_launch():
    P.S.wait_for(P.S.TIMELINE_EXIT, P.S.EXIT, P.EXIT)
    assert T.success(P.JOB)
    assert not MANIFEST.exists() and not (T.B / JOB).exists(), 'Duplicate extraction refused'
    files = {}
    for row in T.read(P.MANIFEST)['files']:
        assert T.sha(Path(row['path'])) == row['sha256'], row['path']
        files[row['path']] = row['sha256']
    paths = [Path(__file__).resolve(), O / 'start-optimized-pictures01.sh', P.MANIFEST, FFMPEG]
    paths += [p for p in (T.B / P.JOB).rglob('*') if p.is_file()]
    for path in paths:
        files[str(path)] = T.sha(path)
    with MANIFEST.open('x') as handle:
        json.dump({'files': [{'path': p, 'sha256': h} for p, h in sorted(files.items())]}, handle, indent=2)
    T.guard_policy()
    guard = T.GUARD
    subprocess.run([sys.executable, '-I', '-B', str(guard), '--phase', 'native', '--job', JOB,
                    '--inputs', str(MANIFEST), '--', sys.executable, '-I', '-B',
                    str(Path(__file__).resolve()), '--extract'], cwd=T.R, check=True)
    assert T.success(JOB)


def preflight():
    report = P.preflight()
    assert not (T.B / JOB).exists() and not MANIFEST.exists(), 'New names already used'
    return report


if __name__ == '__main__':
    assert sys.argv[1:] in ([], ['--extract'], ['--check'])
    if sys.argv[1:] == ['--check']:
        print(json.dumps({'job': JOB, **preflight(), 'launched': False}))
    else:
        extract() if sys.argv[1:] else wait_and_launch()
