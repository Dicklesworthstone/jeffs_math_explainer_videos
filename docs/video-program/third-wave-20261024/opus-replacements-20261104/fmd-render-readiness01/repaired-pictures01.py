"""Prepared guarded preview extraction; extracting pictures is not reviewing them.
All samples retain exact reasons, including provisional timing and persistence.
Run only with separately frozen inputs and a qualified Pillow environment.
"""
import json
import re
import shutil
import subprocess
import sys
import wave
from pathlib import Path
import importlib.util

# Use the pinned sibling path under python -I, without modifying sys.path.
_spec = importlib.util.spec_from_file_location('csd_preview01', Path(__file__).with_name('picture-preview01.py'))
P = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(P)
S = P.S
ROOT, CHAPTERS, POLICY, BINARY_SHA, sha, read = S.ROOT, S.CHAPTERS, S.POLICY, P.BINARY_SHA, S.sha, S.read
normalized = P.normalized
JOB = ROOT / 'jobs/stage08-repaired-pictures01'
PICTURES_OUT = JOB / 'pictures'
PREVIEW_OUT = P.OUT
N = P.NARRATION
PRIOR = ROOT / 'jobs/stage07-pictures01/pictures/extraction.json'
OLD_PREVIEW = ROOT / 'jobs/stage07-preview01/preview'


def layout_paths(chapter, required_folders, optional_folders=()):
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
            assert sha(wav) == phrases[cue]['wav_sha256'], cue
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


def run_pictures():
    from PIL import Image, ImageDraw  # only the picture env has Pillow
    admission = read(JOB / 'resource-receipt.json')
    assert admission['admitted'] and 'exit' not in admission and admission['policy'] == POLICY
    P.check_score()
    S.success('stage08-picture-preview01')
    assert not PICTURES_OUT.exists(), 'Preserve earlier review pictures'
    ffmpeg = shutil.which('ffmpeg')
    assert ffmpeg
    PICTURES_OUT.mkdir()
    phrases = read(N / 'phrase-timings.json')['cues']
    sentences = read(N / 'report.json')['lines']
    summary = read(P.JOB / 'preview-summary.json')
    assert [row['chapter'] for row in summary['chapters']] == list(CHAPTERS)
    assert summary['binary_sha256'] == BINARY_SHA
    assert summary['comparison_result_sha256'] == sha(S.COMPARISON / 'result.json')
    all_samples = []
    sheets = []
    layout_sources = {}
    for chapter_info in summary['chapters']:
        chapter = chapter_info['chapter']
        movie, table = PREVIEW_OUT / (chapter + '.mp4'), PREVIEW_OUT / (chapter + '.timeline.tsv')
        assert sha(movie) == chapter_info['mp4_sha256'] and sha(table) == chapter_info['timeline_sha256']
        rows = [line.split('\t') for line in normalized(table)]
        layouts = layout_paths(chapter, [PREVIEW_OUT, OLD_PREVIEW])
        assert sha(layouts[0]) == chapter_info['layout_sha256']
        layout_sources[chapter] = [str(path) for path in layouts]
        samples = select(chapter, rows, chapter_info['frames'], phrases, sentences, layouts)
        # Keep every previously inspected frame even if repaired lint is silent.
        # Full schedule equality is a prerequisite in P.check_score().
        indexed = {s['frame']: s for s in samples}
        for old in read(PRIOR)['samples']:
            if old['chapter'] != chapter:
                continue
            frame = old['frame']
            assert 0 <= frame < chapter_info['frames']
            current = indexed.setdefault(frame, {'chapter': chapter, 'frame': frame, 'seconds': frame / 60, 'reasons': []})
            current['reasons'] = sorted(set(current['reasons']) | {'prior reviewed frame: '+r for r in old['reasons']})
        samples = [indexed[frame] for frame in sorted(indexed)]
        folder = PICTURES_OUT / chapter
        folder.mkdir()
        for index, sample in enumerate(samples):
            dst = folder / f'{index:04}.png'
            subprocess.run([str(ffmpeg), '-v', 'error', '-xerror', '-nostdin', '-threads', '2',
                            '-ss', str(sample['seconds']), '-i', str(movie), '-frames:v', '1',
                            '-threads', '2', '-n', str(dst)], check=True, timeout=90)
            sample.update(picture=str(dst), sha256=sha(dst))
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
            sheet_path = folder / f'sheet-{first//6:03}.png'
            sheet.save(sheet_path)
            sheets.append({'path': str(sheet_path), 'sha256': sha(sheet_path), 'chapter': chapter, 'first_sample': first, 'count': len(samples[first:first+6])})
        assert sha(movie) == chapter_info['mp4_sha256'] and sha(table) == chapter_info['timeline_sha256']
        assert sha(layouts[0]) == chapter_info['layout_sha256']
        all_samples += samples
        print(chapter, len(samples), 'pictures extracted; actual viewing pending', flush=True)
    P.check_score()
    assert summary == read(P.JOB / 'preview-summary.json')
    with (PICTURES_OUT / 'extraction.json').open('x') as handle:
        json.dump({'samples': all_samples, 'sheets': sheets, 'preview_summary_sha256': sha(P.JOB / 'preview-summary.json'), 'pictures_viewed': False, 'layout_sources': layout_sources,
                   'prior_extraction_sha256': sha(PRIOR),
                   'timeline_lint_note': 'Both original and repaired rendered lint plus every original selected frame retained. Timeline-only mode installs no linter.',
                   'scope': 'Preview samples only; all full reason labels retained. Persistence looks can be nonconsecutive. Not exhaustive review, listening, fine sync or independent acceptance.'}, handle, indent=2)



if __name__ == '__main__':
    assert sys.argv[1:] == ['--run'], 'Guard child only'
    run_pictures()
