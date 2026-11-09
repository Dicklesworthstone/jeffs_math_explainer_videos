"""task455 trial c: master350_455.py with two more gallery variants, applied in memory (asserted single replacements).
447 chapter_gate (:1095-1097) stages the LAST soundtrack_N.wav of a chapter's render. Here the gallery render holds
two valid captures (soundtrack_0 and soundtrack_1, equal length, different samples; the render summary lists both,
as 442's render() lists every file under its render folder). Variant 'two_captures_447_choice' stages soundtrack_1
(447's choice); 'two_captures_older' stages soundtrack_0 (not 447's choice). Also records the refusing 451 line.
Usage as master350_455.py."""
import hashlib
from pathlib import Path
import sys

base = Path(__file__).resolve().parent/'master350_455.py'
src = base.read_text()
reps = [
    ("""        capture = wav(render/'soundtrack'/key/('soundtrack_1.wav' if key == '05_gallery' else 'soundtrack_0.wav'), F)
""",
     """        capture = wav(render/'soundtrack'/key/('soundtrack_1.wav' if key == '05_gallery' else 'soundtrack_0.wav'), F)
        if key == '05_gallery' and variant.startswith('two_captures'):
            older = render/'soundtrack'/key/'soundtrack_0.wav'
            with wave.open(str(older), 'wb') as w:
                w.setnchannels(2)
                w.setsampwidth(2)
                w.setframerate(48000)
                w.writeframes(b'\\1\\0' * (F*800*2))
            if variant == 'two_captures_older':
                capture = older
"""),
    ("""VARIANTS = ['positive', 'gallery_host_D', 'gallery_old_css_job', 'gallery_job_ssd440', 'ssd_worker_on_04',
            'gallery_summary_lists_css_paths', 'gallery_render_frames_mismatch', 'gallery_staged_from_css_copy',
            'summary_version_350']""",
     """VARIANTS = ['positive', 'two_captures_447_choice', 'two_captures_older']"""),
    ("""        results[variant] = dict(outcome='refused', error='%s: %s' % (type(error).__name__,
                                str(error).replace(str(OUTDIR), 'T')[:200]))""",
     """        import traceback
        frames = [f for f in traceback.extract_tb(error.__traceback__) if f.filename == m.__file__]
        results[variant] = dict(outcome='refused', error='%s: %s' % (type(error).__name__, str(error).replace(str(OUTDIR), 'T')[:200]),
                                v451_line=frames[-1].lineno if frames else None)"""),
    ("""                                files=len(r['files']))""",
     """                                files=len(r['files']),
                                gallery_capture_source=str(r['sources'][m.OUT/'soundtrack/05_gallery/soundtrack_0.wav']).rsplit('/', 1)[-1],
                                gallery_render_captures=sorted(p.name for p in (m.SSD442/'jobs'/m.GALLERY442_JOB/'render/soundtrack/05_gallery').iterdir()))"""),
]
for old, new in reps:
    assert src.count(old) == 1, old[:60]
    src = src.replace(old, new)
print('# base master350_455.py sha256', hashlib.sha256(base.read_bytes()).hexdigest(), file=sys.stderr)
exec(compile(src, str(base), 'exec'), {'__name__': '__main__'})
