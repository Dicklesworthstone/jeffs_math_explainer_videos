"""Guarded serial 4K chapters from the optimized recovery. Requires a real, hash-bound preview review.

Root's final-render01.py with new names and tighter gates; diff the two to audit it. It runs the
release binary through the sealed chain and refuses to import any file of that chain whose bytes
changed. There is no flag that skips the review.

This helper is prepared only; it never invents a review or starts on import.
Mastering, final pictures, ASR, listening and publication remain separate.
"""
import hashlib
import importlib.util
import json
from pathlib import Path
import subprocess
import sys
import wave

O = Path(__file__).resolve().parent
SEALED = O.parent / 'complex-optimized-downstream01'
PICTURES_HELPER = SEALED / 'pictures_optimized01.py'
IMPORTED_SHA256 = {
    PICTURES_HELPER: '2f4075262147acfe80c0cf4b14c0b8f6ecdbe169d62188c3a96968832f39d6e7',
    SEALED / 'preview_optimized01.py': 'b823a34c2cab7f6cd6d8e2b55e26871aa25b6caa3041ff43a80cb27d32166c19',
    SEALED / 'score_optimized01.py': '22225d74f7a4362872fd62149ab9ad217249680a38829ff9d628a997adda5fb8',
    O.parent / 'complex-optimized-recovery-review01/optimized_timeline_recovery01.py':
        '4560e0dcd5534e0b8b3eaf205622f7cc36657c109344107d976c7b852dc086d7',
    O.parents[4] / 'narration_lab/third-wave-20261024/root/elevenlabs-complex-rebuild-20261104/native-stage06-root03/timeline-after-audio04.py':
        'e3ab6203a565b020d988faa6008f926abc5c08aa2840c915b759f32e831051ae',
}
for imported, digest in IMPORTED_SHA256.items():
    with imported.open('rb') as handle:
        assert hashlib.file_digest(handle, 'sha256').hexdigest() == digest, 'Imported helper changed: ' + imported.name
SPEC = importlib.util.spec_from_file_location('complex_pictures', PICTURES_HELPER)
X = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(X)
P = X.P
T = P.T
STAGE = T.E / 'optimized-final-production01'  # controllers, manifests and the review
OUT = T.B / 'complex-stage06-optimized-render-work01'
PICTURES_JOB = X.JOB
REVIEW = STAGE / 'preview-pictures-viewed01.json'
GUARD = T.GUARD


def job(chapter):
    assert chapter in T.CHAPTERS
    return 'complex-stage06-optimized-render-' + chapter + '-01'


def manifest(chapter):
    return STAGE / (job(chapter) + '-inputs.json')


def check_review():
    assert REVIEW.is_file(), 'Actual narrated preview review is missing; no final render authorized by this helper'
    assert T.success(P.JOB) and T.success(PICTURES_JOB)
    P.check_score()
    review = T.read(REVIEW)
    assert review['chapters'] == list(T.CHAPTERS)
    assert review['unresolved_visual_defects'] == []
    # pictures_viewed covers the preview sheets only. It asserts nothing about listening or final pictures.
    assert review['pictures_viewed'] is True and review['reviewer'].strip(), 'Review names no viewer'
    assert review['review_scope'] in ('source-author', 'independent'), 'Review scope must be explicit'
    folder = T.B / PICTURES_JOB / 'pictures'
    sheets = sorted(folder.glob('*/sheet-*.png'))
    assert sheets and {p.parent.name for p in sheets} == set(T.CHAPTERS)
    samples = {}
    for sample in T.read(folder / 'extraction.json')['samples']:
        samples[sample['chapter']] = samples.get(sample['chapter'], 0) + 1
    expected = {folder / chapter / f'sheet-{index:03}.png' for chapter, count in samples.items()
                for index in range((count + 5) // 6)}
    assert set(sheets) == expected, 'Sheets differ from the extraction'
    assert review['sheet_sha256'] == {str(p): T.sha(p) for p in sheets}
    assert review['extraction_sha256'] == T.sha(folder / 'extraction.json')
    assert review['preview_summary_sha256'] == T.sha(T.B / P.JOB / 'preview-summary.json')
    summary = T.read(T.B / P.JOB / 'preview-summary.json')
    assert [row['chapter'] for row in summary['chapters']] == list(T.CHAPTERS)
    for row in summary['chapters']:
        for suffix, key in [('.mp4', 'mp4_sha256'), ('.timeline.tsv', 'timeline_sha256'),
                            ('.layout.tsv', 'layout_sha256')]:
            assert T.sha(P.OUT / (row['chapter'] + suffix)) == row[key]
    return review


def captured_soundtrack(work, chapter, frames):
    # Match the native master's exact selection: last consecutive numbered capture.
    folder = work / 'soundtrack' / chapter
    count = 0
    while (folder / f'soundtrack_{count}.wav').exists():
        count += 1
    assert count, f'{chapter}: missing soundtrack capture'
    path = folder / f'soundtrack_{count-1}.wav'
    samples = frames * 800  # exact 48kHz/60fps
    with wave.open(str(path)) as wav:
        assert (wav.getframerate(), wav.getnchannels(), wav.getsampwidth(), wav.getnframes()) == (48000, 2, 2, samples)
        remaining = samples
        while remaining:
            requested = min(remaining, 48000)
            assert len(wav.readframes(requested)) == requested * 4, 'Truncated soundtrack data'
            remaining -= requested
    return {'path': str(path), 'sha256': T.sha(path), 'samples': samples, 'captures': count}


def render(chapter):
    name = job(chapter)
    receipt = T.read(T.B / name / 'resource-receipt.json')
    assert receipt.get('admitted') is True and 'exit' not in receipt
    assert receipt['command'] == [sys.executable, '-I', '-B', str(Path(__file__).resolve()), '--run', chapter]
    assert receipt['policy'] == T.POLICY, 'Guard policy changed'
    for row in T.read(manifest(chapter))['files']:
        assert T.sha(Path(row['path'])) == row['sha256'], row['path']
    check_review()
    movie = OUT / (chapter + '.mp4')
    assert not movie.exists(), 'Existing render preserved; use a reviewed new attempt after failure'
    OUT.mkdir(exist_ok=True)
    subprocess.run([str(T.BIN), 'render', chapter, '--res', '3840x2160', '--fps', '60',
                    '--narration', str(T.N), '--score', str(P.S.OUT / 'stems'), '--out', str(OUT),
                    '--lint', '--crf', '16', '--preset', 'veryfast'], cwd=T.R, check=True, timeout=43200)
    table = OUT / (chapter + '.timeline.tsv')
    timeline = T.B / T.JOB / 'timeline-output'
    for kind in ('timeline', 'cues'):
        rendered = OUT / f'{chapter}.{kind}.tsv'
        assert P.normalized(rendered) == P.normalized(timeline / rendered.name), 'Final table differs: ' + rendered.name
    frames = int(next(line.split('\t')[1] for line in P.normalized(table) if line.startswith('frames\t')))
    P.check_cue_log(chapter, frames, OUT / (chapter + '.cuelog.tsv'), timeline / (chapter + '.cuelog.tsv'))
    assert (OUT / (chapter + '.layout.tsv')).is_file(), 'Rendered lint report missing: ' + chapter
    probe = json.loads(subprocess.check_output([str(P.PROBE), '-v', 'error', '-show_streams', '-of', 'json', str(movie)], timeout=120))
    video = [s for s in probe['streams'] if s['codec_type'] == 'video']
    audio = [s for s in probe['streams'] if s['codec_type'] == 'audio']
    assert len(video) == len(audio) == 1
    v = video[0]
    assert (v['width'], v['height'], v['avg_frame_rate'], int(v['nb_frames'])) == (3840, 2160, '60/1', frames)
    track = captured_soundtrack(OUT, chapter, frames)
    paths = [movie, table, OUT / (chapter + '.layout.tsv'), OUT / (chapter + '.cuelog.tsv')]
    with (T.B / name / 'render-summary.json').open('x') as handle:
        json.dump({'chapter': chapter, 'frames': frames, 'fps': 60, 'soundtrack': track,
                   'files': [{'path': str(p), 'sha256': T.sha(p)} for p in paths],
                   'scope': '4K chapter render with exact schedule and complete capture. Final AV, picture review, listening and fine sync remain open.'}, handle, indent=2)


def launch_all():
    check_review()
    T.guard_policy()
    base = {}
    for row in T.read(X.MANIFEST)['files']:
        assert T.sha(Path(row['path'])) == row['sha256'], row['path']
        base[row['path']] = row['sha256']
    paths = [Path(__file__).resolve(), O / 'start-optimized-final-render01.sh', *IMPORTED_SHA256, REVIEW,
             X.MANIFEST, GUARD, T.GUARD_SOURCE,
             T.B / PICTURES_JOB / 'resource-receipt.json',
             T.B / PICTURES_JOB / 'pictures/extraction.json',
             T.B / P.JOB / 'resource-receipt.json', T.B / P.JOB / 'preview-summary.json']
    paths += sorted((T.B / PICTURES_JOB / 'pictures').glob('*/sheet-*.png'))
    paths += sorted(p for p in P.OUT.rglob('*') if p.is_file())
    for path in paths:
        base[str(path)] = T.sha(path)
    for chapter in T.CHAPTERS:
        name = job(chapter)
        if (T.B / name).exists():
            assert T.success(name), 'Existing incomplete/failed chapter; preserve it and investigate'
            for row in T.read(manifest(chapter))['files']:
                assert T.sha(Path(row['path'])) == row['sha256'], row['path']
            summary = T.read(T.B / name / 'render-summary.json')
            assert summary['chapter'] == chapter
            for row in summary['files']:
                assert T.sha(Path(row['path'])) == row['sha256'], row['path']
            assert captured_soundtrack(OUT, chapter, summary['frames']) == summary['soundtrack']
            continue
        assert not manifest(chapter).exists(), 'Existing manifest without job; inspect before resuming'
        with manifest(chapter).open('x') as handle:
            json.dump({'files': [{'path': p, 'sha256': h} for p, h in sorted(base.items())]}, handle, indent=2)
        subprocess.run([sys.executable, '-I', '-B', str(GUARD), '--phase', 'native', '--job', name,
                        '--inputs', str(manifest(chapter)), '--', sys.executable, '-I', '-B',
                        str(Path(__file__).resolve()), '--run', chapter], cwd=T.R, check=True)
        assert T.success(name)


def preflight():
    """The pins already held at import. This hashes no frozen input and samples no disk."""
    assert not OUT.exists() and not any((T.B / job(chapter)).exists() for chapter in T.CHAPTERS), 'New names already used'
    return {'chapter_jobs': len(T.CHAPTERS), 'pinned_helpers': len(IMPORTED_SHA256), 'launched': False}


if __name__ == '__main__':
    if sys.argv[1:] == ['--check']:
        print(json.dumps(preflight()))
    elif sys.argv[1:] == ['--check-review']:
        check_review()
    elif sys.argv[1:] == ['--all']:
        launch_all()
    elif len(sys.argv) == 3 and sys.argv[1] == '--run':
        render(sys.argv[2])
    else:
        raise SystemExit('Use --check, --check-review, --all, or --run CHAPTER. No implicit launch.')
