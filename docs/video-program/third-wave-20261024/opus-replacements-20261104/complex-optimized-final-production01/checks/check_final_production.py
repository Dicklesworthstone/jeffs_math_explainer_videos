#!/usr/bin/env python3
"""Check the three final helpers in a temporary folder.

Real: the helpers, their pinned imports, the sealed downstream controllers (run through the sealed
rehearsal's fixture), the pinned strict video check and caption screen, ffmpeg, ffprobe, the 105 real
narration takes and script, the start scripts' shell, and two read-only controls on logarithm's
finished master. Fake: guard, binary, job tree and the tiny movies the binary hands back. It runs no
real guard, render or master, hashes no frozen input, samples no disk and writes nothing in the
repository. This is a source author's check, not independent acceptance. From the repository root:

    python3 -I -B <bundle>/checks/check_final_production.py
"""
import ast
import hashlib
import importlib.util
import json
import math
from pathlib import Path
import re
import shutil
import subprocess
import sys
import tempfile
from PIL import Image

HERE = Path(__file__).resolve()
BUNDLE = HERE.parents[1]
SEALED = BUNDLE.parent / 'complex-optimized-downstream01'
SEALED_SUMS = 'bd8508c3e88bb9c529a8369663665c22a9aaa7e83d62a677594846b12443182a'
FRAMES = 36  # long enough for every cue to get its own frame and for sentence samples to land
FINAL_FAKE = r'''
import wave
FINAL_LINT = '02_1 +0.0s @0.10s\tstuck\t"w" still there at 3 later looks\n'
if args[0] == 'render' and args[1] != 'all':
    chapter = args[1]
    assert args[2:7] == ['--res', '3840x2160', '--fps', '60', '--narration']
    assert args[-5:] == ['--lint', '--crf', '16', '--preset', 'veryfast']
    if case == 'render-exit':
        sys.exit(3)
    out.mkdir(exist_ok=True)
    preview = Path(c['preview_out'])
    for name in ('timeline', 'cues', 'cuelog', 'layout'):
        if case == 'layout-missing' and name == 'layout':
            continue
        text = (preview / f'{chapter}.{name}.tsv').read_text()
        if name == 'layout' and chapter == c['chapters'][0]:
            text = FINAL_LINT
        (out / f'{chapter}.{name}.tsv').write_text(text + ('x\t1\n' if case == name + '-differs' else ''))
    shutil.copy(here / ('tiny.mp4' if case == 'res-wrong' else 'final.mp4'), out / f'{chapter}.mp4')
    folder = out / 'soundtrack' / chapter
    folder.mkdir(parents=True)
    if case != 'capture-missing':
        with wave.open(str(folder / 'soundtrack_0.wav'), 'wb') as handle:
            handle.setnchannels(2)
            handle.setsampwidth(2)
            handle.setframerate(48000)
            handle.writeframes(bytes(4 * (c['frames'] * 800 - (1 if case == 'capture-short' else 0))))
    sys.exit(0)
if args[0] == 'master':
    assert args[1:] == ['--narration', c['narration'], '--score', c['stems'], '--out', str(out)]
    if case == 'master-exit':
        sys.exit(3)
    for n, chapter in enumerate(c['chapters']):
        if not (case == 'proof-missing' and n == 7):
            print(f"{chapter} replay = fmn's soundtrack, bit for bit (7 cues, 4800 samples)")
    name = {'master-wrong': 'tiny.mp4', 'master-quiet': 'quiet.mp4'}.get(case, 'master.mp4')
    shutil.copy(here / name, out / 'complex_analysis_visual_master.mp4')
    sys.exit(0)
if args[0] == 'captions':
    assert args[1:3] == ['all', '--narration'] and args[3] == c['narration']
    if case == 'captions-exit':
        sys.exit(3)
    srt, vtt = (here / 'fixture.srt').read_text(), (here / 'fixture.vtt').read_text()
    if case == 'srt-empty':
        srt = ''
    if case == 'caption-text':
        srt, vtt = srt[:-2] + ' extra\n\n', vtt[:-2] + ' extra\n\n'
    if case == 'caption-differs':
        vtt = vtt[:-2] + ' extra\n\n'
    (out / 'complex_analysis_visual.en.srt').write_text(srt)
    (out / 'complex_analysis_visual.en.vtt').write_text(vtt)
    sys.exit(0)
'''
GUARD = """import subprocess, sys
sys.exit(subprocess.run([sys.executable, '-I', '-B', CHECK, '--guard', WORK, *sys.argv[1:]]).returncode)
"""
WRONG_SIZE = "assert (v['width'], v['height'], v['avg_frame_rate'], int(v['nb_frames'])) == (3840, 2160"
ROW_CHANGED = "assert T.sha(Path(row['path'])) == row['sha256']"
RENDER_CASES = {
    'render-exit': 'CalledProcessError',
    'timeline-differs': 'Final table differs',
    'cues-differs': 'Final table differs',
    'cuelog-differs': 'Preview score rows differ',
    'layout-missing': 'Rendered lint report missing',
    'res-wrong': WRONG_SIZE,
    'capture-missing': 'missing soundtrack capture',
    'capture-short': 'assert (wav.getframerate(), wav.getnchannels(), wav.getsampwidth(), wav.getnframes())',
    'guard-policy': 'Guard policy changed',
    'input-changed': ROW_CHANGED,
    'existing-movie': 'Existing render preserved',
}
MASTER_CASES = {
    'master-exit': 'CalledProcessError',
    'proof-missing': 'Missing native soundtrack replay proof',
    'captions-exit': 'CalledProcessError',
    'master-wrong': WRONG_SIZE,
    'srt-empty': "assert OUTPUTS['srt'].stat().st_size and OUTPUTS['vtt']",
    'guard-policy': 'Guard policy changed',
}
results = []


def check(name, ok, detail):
    results.append(bool(ok))
    print(f"{'PASS' if ok else 'FAIL'} {name}: {detail}", flush=True)


def sha(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def load(path, name='module_under_check'):
    spec = importlib.util.spec_from_file_location(name, path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


class EarlyMarks:
    """The verifier's own view of T. Only its report and phrase reads move; the sealed chain keeps the real folder."""

    def __init__(self, T, folder):
        self.T, self.N = T, folder

    def __getattr__(self, name):
        return getattr(self.T, name)


def chain(tmp, which):
    """One helper with its whole chain pointed at the fixture, never at the real tree."""
    H = load(BUNDLE / f'final_{which}_optimized01.py')
    M = H if which == 'master' else H.M if which == 'verify' else None
    F = M.F if M else H
    X, P, T = F.X, F.P, F.T
    S = P.S
    B = tmp / 'B'
    T.B = T.T.B = B
    T.BIN, T.MANIFEST, T.PRIOR_OUT = tmp / 'fake', tmp / 'timeline-inputs.json', tmp / 'prior'
    T.STAGE, T.GUARD, F.GUARD = tmp / 'timeline-stage', tmp / 'fake-guard', tmp / 'fake-guard'
    T.BIN_SHA256 = sha(tmp / 'fake')
    T.guard_policy = lambda: None
    S.STAGE = tmp / 'downstream'
    S.TIMELINE_EXIT = T.STAGE / 'controller01.exit'
    S.EXIT, S.MANIFEST = S.STAGE / S.EXIT.name, S.STAGE / S.MANIFEST.name
    S.OUT = B / S.JOB / 'scored'
    P.EXIT, P.MANIFEST, P.OUT = S.STAGE / P.EXIT.name, S.STAGE / P.MANIFEST.name, B / P.JOB / 'preview'
    X.MANIFEST, X.OUT = S.STAGE / X.MANIFEST.name, B / X.JOB / 'pictures'
    F.STAGE, F.OUT = tmp / 'final-stage', B / F.OUT.name
    F.REVIEW = F.STAGE / F.REVIEW.name
    if M:
        M.MANIFEST = F.STAGE / M.MANIFEST.name
        M.OUTPUTS = {key: F.OUT / path.name for key, path in M.OUTPUTS.items()}
    if which == 'verify':
        H.OUT, H.MANIFEST = B / H.JOB, F.STAGE / H.MANIFEST.name
        H.T = EarlyMarks(T, tmp / 'early-marks')
    return H


def fake_guard(tmp, argv):
    """What the real guard does around a child, with no resource sampling."""
    T = chain(tmp, 'render').T
    case = json.loads((tmp / 'config.json').read_text())['case']
    job = T.B / argv[argv.index('--job') + 1]
    job.mkdir()
    with (tmp / 'guard-calls.jsonl').open('a') as handle:
        handle.write(json.dumps(argv) + '\n')
    command = argv[argv.index('--') + 1:]
    policy = dict(T.POLICY, work_cap_gib=48) if case == 'guard-policy' else T.POLICY
    (job / 'resource-receipt.json').write_text(json.dumps({'admitted': True, 'command': command, 'policy': policy}))
    if case == 'input-changed':
        (tmp / 'frozen.txt').write_text('changed')
    with (job / 'command.log').open('x') as log:
        code = subprocess.run([sys.executable, '-I', '-B', str(HERE), '--child', str(tmp), *command[3:]],
                              stdout=log, stderr=subprocess.STDOUT).returncode
    (job / 'resource-receipt.json').write_text(json.dumps({
        'admitted': True, 'command': command, 'policy': policy, 'exit': code, 'finished': 'fixture',
        'inputs_sha256': 'a', 'final_inputs_sha256': 'a'}))
    return code


def write_review(F, tmp):
    folder = F.X.OUT
    F.STAGE.mkdir(exist_ok=True)
    review = {'reviewer': 'fixture reviewer', 'review_scope': 'source-author', 'pictures_viewed': True,
              'chapters': list(F.T.CHAPTERS), 'unresolved_visual_defects': [],
              'sheet_sha256': {str(p): sha(p) for p in sorted(folder.glob('*/sheet-*.png'))},
              'extraction_sha256': sha(folder / 'extraction.json'),
              'preview_summary_sha256': sha(F.T.B / F.P.JOB / 'preview-summary.json')}
    F.REVIEW.write_text(json.dumps(review))
    return review


def refused(action, words):
    try:
        action()
    except (AssertionError, RuntimeError, OSError, KeyError) as error:
        return words in f'{type(error).__name__} {error}'
    return False


def act(tmp, which):
    run = subprocess.run([sys.executable, '-I', '-B', str(HERE), '--act', str(tmp), which], capture_output=True, text=True)
    logs = ''.join(p.read_text(errors='replace') for p in (tmp / 'B').glob('complex-stage06-optimized-*/command.log')
                   if not re.search(r'-(score|preview|pictures)01$', p.parent.name))
    return run.returncode, run.stdout + run.stderr + logs


def set_case(tmp, case):
    path = tmp / 'config.json'
    config = json.loads(path.read_text())
    config['case'] = case
    path.write_text(json.dumps(config))


def shelve(tmp, label, paths):
    """Move a finished case aside; nothing is removed, so each failure stays inspectable."""
    shelf = tmp / 'shelf' / label
    shelf.mkdir(parents=True)
    for path in paths:
        if path.exists():
            path.rename(shelf / path.name)
    (tmp / 'frozen.txt').write_text('frozen')


def mode_flags(text):
    main = ast.parse(text).body[-1]
    return {node.value for node in ast.walk(main) if isinstance(node, ast.Constant)
            and isinstance(node.value, str) and node.value.startswith('--')}


def timeouts(path):
    return sorted(re.findall(r'timeout=(\d+)', Path(path).read_text()))


def static_checks(H):
    M, F = H.M, H.M.F
    T = F.T
    native = T.E / 'native-stage06-root03'
    texts = {name: (BUNDLE / f'final_{name}_optimized01.py').read_text() for name in ('render', 'master', 'verify')}
    shells = [subprocess.run(['/bin/bash', '-n', str(BUNDLE / f'start-optimized-final-{n}01.sh')]).returncode for n in texts]
    check('a-parse', shells == [0, 0, 0], 'all three helpers load and all three start scripts parse')
    sums = (SEALED / 'SHA256SUMS').read_text()
    pinned = {path.name: digest for path, digest in F.IMPORTED_SHA256.items()}
    check('a-pins', sha(SEALED / 'SHA256SUMS') == SEALED_SUMS and len(pinned) == 5
          and all(f'{pinned[n]}  {n}\n' in sums for n in ('pictures_optimized01.py', 'preview_optimized01.py', 'score_optimized01.py'))
          and F.P.S.RECOVERY_SHA256 == pinned['optimized_timeline_recovery01.py']
          and M.RENDER_SHA256 == sha(BUNDLE / 'final_render_optimized01.py') == sha(M.RENDER_HELPER)
          and H.IMPORTED_SHA256[H.MASTER_HELPER] == sha(BUNDLE / 'final_master_optimized01.py')
          and H.IMPORTED_SHA256[H.SHARED] == sha(H.SHARED) and H.X is F.X and F.X.P is F.P,
          'verify pins master, master pins render, render pins the five sealed files, and one pictures module serves all three')
    calls = {node.func.attr if isinstance(node.func, ast.Attribute) else node.func.id: node for name in ('render', 'master')
             for node in ast.walk(ast.parse(texts[name])) if isinstance(node, ast.Call) and isinstance(node.func, (ast.Attribute, ast.Name))}
    bodies = {node.name: {sub.func.attr if isinstance(sub.func, ast.Attribute) else getattr(sub.func, 'id', '')
                          for sub in ast.walk(node) if isinstance(sub, ast.Call)}
              for name in ('render', 'master') for node in ast.parse(texts[name]).body if isinstance(node, ast.FunctionDef)}
    check('a-no-bypass', 'check_review' in calls and all('check_review' in bodies[name] for name in ('render', 'launch_all', 'check_chapters'))
          and mode_flags(texts['render']) == {'--check', '--check-review', '--all', '--run'}
          and mode_flags(texts['master']) == {'--check', '--launch', '--run'}
          and mode_flags(texts['verify']) == {'--pins', '--check', '--launch', '--run'},
          'launch, each chapter child and the master all run check_review, and no mode skips it')
    four_k = ["'--res', '3840x2160', '--fps', '60'", "'--lint', '--crf', '16', '--preset', 'veryfast'"]
    pairs = [(BUNDLE / f'final_{name}_optimized01.py', native / f'final-{name}01.py') for name in texts]
    check('a-delivery', all(item in texts['render'] for item in four_k) and all(timeouts(new) == timeouts(old) for new, old in pairs)
          and "[str(T.BIN), 'master'" in texts['master'] and "[str(T.BIN), 'captions', 'all'" in texts['master']
          and T.BIN.name == 'complex_analysis_visual-release01',
          "4K60, CRF 16, lint, master and captions use the release binary, and every timeout equals root's")
    old = (native / 'final-verify01.py').read_text()
    used = lambda text, name: set(re.findall(rf'\b{name}\.\w+', text))
    check('a-verify-same-checks', used(texts['verify'], 'V') == used(old, 'V') and used(texts['verify'], 'X') == used(old, 'X')
          and "'pictures_viewed': False" in texts['verify'] and "text.splitlines()[-1] == 'VERIFY PASS'" in texts['verify'],
          "the verifier makes the same shared-check and picture-selection calls as root's and still records pictures_viewed false")
    check('a-names', H.JOB == 'complex-stage06-optimized-final-verify01' and M.JOB == 'complex-stage06-optimized-master01'
          and F.job(T.CHAPTERS[0]).startswith('complex-stage06-optimized-render-') and F.OUT.name == 'complex-stage06-optimized-render-work01'
          and M.OUTPUTS['master'] == T.B / 'complex-stage06-optimized-render-work01/complex_analysis_visual_master.mp4'
          and F.REVIEW == T.E / 'optimized-final-production01/preview-pictures-viewed01.json' and H.OUT == T.B / H.JOB
          and F.preflight()['launched'] is False and H.preflight()['launched'] is False and not (T.B / M.JOB).exists(),
          'all names are the ones root confirmed, unused, and the master path is the one root gave')


def review_checks(tmp, F):
    good = write_review(F, tmp)
    check('v-valid', F.check_review() == good, 'a complete hash-bound review of every sheet passes')
    first = next(iter(good['sheet_sha256']))
    extra = str(Path(first).with_name('sheet-099.png'))

    def attempt(change, words):
        review = json.loads(json.dumps(good))
        change(review)
        F.REVIEW.write_text(json.dumps(review))
        ok = refused(F.check_review, words)
        F.REVIEW.write_text(json.dumps(good))
        return ok

    changes = {
        'no-viewer': (lambda r: r.update(reviewer=' '), 'Review names no viewer'),
        'no-viewer-field': (lambda r: r.pop('reviewer'), 'KeyError'),
        'not-viewed': (lambda r: r.update(pictures_viewed=False), 'Review names no viewer'),
        'no-scope': (lambda r: r.pop('review_scope'), 'KeyError'),
        'vague-scope': (lambda r: r.update(review_scope='reviewed'), 'Review scope must be explicit'),
        'open-defect': (lambda r: r.update(unresolved_visual_defects=['x']), 'AssertionError'),
        'chapters-short': (lambda r: r['chapters'].pop(), 'AssertionError'),
        'sheet-dropped': (lambda r: r['sheet_sha256'].pop(first), 'AssertionError'),
        'sheet-hash': (lambda r: r['sheet_sha256'].update({first: '0' * 64}), 'AssertionError'),
        'sheet-extra': (lambda r: r['sheet_sha256'].update({extra: '0' * 64}), 'AssertionError'),
        'extraction-hash': (lambda r: r.update(extraction_sha256='0' * 64), 'AssertionError'),
        'summary-hash': (lambda r: r.update(preview_summary_sha256='0' * 64), 'AssertionError'),
    }
    for name, (change, words) in changes.items():
        check(f'v-{name}', attempt(change, words), 'refused')
    sheet = Path(first)
    moved = tmp / 'moved-sheet.png'
    sheet.rename(moved)
    check('v-sheet-gone-from-both', attempt(lambda r: r['sheet_sha256'].pop(first), 'Sheets differ from the extraction'),
          'a sheet removed from the folder and the review together is refused from the extraction alone')
    moved.rename(sheet)
    movie = F.P.OUT / (F.T.CHAPTERS[3] + '.mp4')
    data = movie.read_bytes()
    movie.write_bytes(data + b'x')
    check('v-preview-changed', refused(F.check_review, 'AssertionError'), 'a preview movie changed after review is refused')
    movie.write_bytes(data)
    F.REVIEW.rename(tmp / 'moved-review.json')
    code, text = act(tmp, 'render')
    check('v-missing-review', code != 0 and 'Actual narrated preview review is missing' in text
          and not list(F.STAGE.glob('*-inputs.json')) and not list((tmp / 'B').glob('complex-stage06-optimized-render-*')),
          '--all with no review refuses before any manifest or job')
    (tmp / 'moved-review.json').rename(F.REVIEW)


def render_jobs(tmp, F):
    return [*(tmp / 'B').glob('complex-stage06-optimized-render-*'), *F.STAGE.glob('complex-stage06-*-inputs.json')]


def render_checks(tmp, F):
    first = F.T.CHAPTERS[0]
    for case, words in RENDER_CASES.items():
        set_case(tmp, 'pass' if case == 'existing-movie' else case)
        F.OUT.mkdir(exist_ok=True)
        if case == 'existing-movie':
            (F.OUT / f'{first}.mp4').write_text('keep')
        code, text = act(tmp, 'render')
        job = tmp / 'B' / F.job(first)
        kept = case != 'existing-movie' or (F.OUT / f'{first}.mp4').read_text() == 'keep'
        ok = code != 0 and words in text and not (job / 'render-summary.json').exists() and kept
        check(f'r-{case}', ok, f'refused ({words[:48]}) with no summary' + ('' if ok else ' ' + text[-300:]))
        if case == 'render-exit':
            set_case(tmp, 'pass')
            code, text = act(tmp, 'render')
            check('r-failed-blocks-all', code != 0 and "assert d.get('admitted') is True and d.get('exit') == 0, job" in text
                  and not (tmp / 'B' / F.job(F.T.CHAPTERS[1])).exists() and job.exists(),
                  "as in root's helper, a failed chapter stops --all, with no automatic retry, and stays preserved")
        shelve(tmp, 'render-' + case, render_jobs(tmp, F))


def render_pass(tmp, F):
    set_case(tmp, 'pass')
    code, text = act(tmp, 'render')
    summaries = [tmp / 'B' / F.job(chapter) / 'render-summary.json' for chapter in F.T.CHAPTERS]
    check('e-render', code == 0 and all(path.exists() for path in summaries), '15 guarded chapters render, each with real ffprobe at 3840x2160, 60/1'
          + ('' if code == 0 else ' ' + text[-400:]))
    calls = (tmp / 'guard-calls.jsonl').read_text().splitlines()
    code, text = act(tmp, 'render')
    check('e-resume', code == 0 and (tmp / 'guard-calls.jsonl').read_text().splitlines() == calls, 'a second --all reruns nothing')
    argv = json.loads(calls[-1])
    shown = [Path(x).name if x.startswith('/') and x != sys.executable else x for x in argv]
    last = F.T.CHAPTERS[-1]
    check('e-guard-argv', shown == ['--phase', 'native', '--job', F.job(last), '--inputs', F.job(last) + '-inputs.json',
                                    '--', sys.executable, '-I', '-B', 'final_render_optimized01.py', '--run', last],
          'the guard gets the same phase and the new job, manifest and child')
    listed = {row['path'] for row in json.loads(F.manifest(last).read_text())['files']}
    needed = {*map(str, F.IMPORTED_SHA256), str(BUNDLE / 'final_render_optimized01.py'), str(BUNDLE / 'start-optimized-final-render01.sh'),
              str(F.REVIEW), str(F.X.MANIFEST), str(F.GUARD), str(F.T.GUARD_SOURCE), str(tmp / 'frozen.txt'), str(F.X.OUT / 'extraction.json')}
    needed |= {str(p) for p in F.X.OUT.glob('*/sheet-*.png')}
    needed |= {row['path'] for row in json.loads(F.X.MANIFEST.read_text())['files']}
    check('e-manifest', needed <= listed, 'each chapter manifest binds the pictures manifest rows, all imports, the helper, its start script, the review, every sheet and the guard')
    summary = json.loads(summaries[0].read_text())
    check('e-capture', summary['soundtrack']['samples'] == FRAMES * 800 and summary['soundtrack']['captures'] == 1 and len(summary['files']) == 4,
          'the summary binds the exact-length capture, the movie and the three tables')


def master_failures(tmp, F, M, H):
    def paths():
        return [tmp / 'B' / M.JOB, M.MANIFEST, *M.OUTPUTS.values()]
    for case, words in MASTER_CASES.items():
        set_case(tmp, case)
        code, text = act(tmp, 'master')
        ok = code != 0 and words in text and not (tmp / 'B' / M.JOB / 'master-summary.json').exists()
        check(f'm-{case}', ok, f'refused ({words[:48]}) with no summary' + ('' if ok else ' ' + text[-300:]))
        shelve(tmp, 'master-' + case, paths())
    set_case(tmp, 'pass')
    code, text = act(tmp, 'verify')
    check('w-no-master', code != 0 and 'Stable complex master required' in text and not H.MANIFEST.exists() and not H.OUT.exists(),
          'the verifier refuses before any manifest or job when no successful master exists')
    job = tmp / 'B' / F.job(F.T.CHAPTERS[5])
    job.rename(tmp / 'moved-job')
    code, text = act(tmp, 'master')
    check('m-chapter-incomplete', code != 0 and 'Incomplete final chapter' in text and not M.MANIFEST.exists(), 'a missing chapter refuses before any manifest or job')
    (tmp / 'moved-job').rename(job)
    capture = F.OUT / 'soundtrack' / F.T.CHAPTERS[5] / 'soundtrack_0.wav'
    data = capture.read_bytes()
    capture.write_bytes(data + b'x')
    code, text = act(tmp, 'master')
    check('m-capture-changed', code != 0 and not M.MANIFEST.exists(), 'a capture changed after its chapter refuses')
    capture.write_bytes(data)
    M.OUTPUTS['srt'].write_text('keep')
    code, text = act(tmp, 'master')
    check('m-existing-output', code != 0 and 'Existing output preserved' in text and M.OUTPUTS['srt'].read_text() == 'keep', 'an existing output is kept and refused')
    M.OUTPUTS['srt'].rename(tmp / 'moved.srt')


def master_pass(tmp, F, M):
    code, text = act(tmp, 'master')
    summary_path = tmp / 'B' / M.JOB / 'master-summary.json'
    check('e-master', code == 0 and summary_path.exists(), f'the master runs, with real ffprobe at 3840x2160, 60/1 and {FRAMES * 15} frames' + ('' if code == 0 else ' ' + text[-400:]))
    summary = json.loads(summary_path.read_text())
    check('e-null-test-proof', [line.split()[0] for line in summary['native_bit_identical_replays']] == list(F.T.CHAPTERS) and len(summary['soundtracks']) == 15,
          'all 15 proof lines and all 15 captures are in the summary, in chapter order')
    check('e-captions', set(summary['outputs']) == {'master', 'srt', 'vtt'} and summary['full_baseline_equivalence_established'] is False,
          'master, SRT and VTT are hash-bound, and no baseline equivalence is claimed')
    listed = {row['path'] for row in json.loads(M.MANIFEST.read_text())['files']}
    chapter = F.T.CHAPTERS[5]
    needed = {str(BUNDLE / 'final_render_optimized01.py'), str(BUNDLE / 'final_master_optimized01.py'), str(BUNDLE / 'start-optimized-final-master01.sh'),
              str(F.OUT / 'soundtrack' / chapter / 'soundtrack_0.wav'), str(F.OUT / f'{chapter}.mp4'), str(F.manifest(chapter)), str(F.REVIEW)}
    check('e-master-manifest', needed <= listed, 'the master manifest binds both helpers, its start script, the review, the chapter manifests and every chapter file')


def verify_refusals(tmp, H):
    M = H.M
    master = M.OUTPUTS['master']
    data = master.read_bytes()
    master.write_bytes(data + b'x')
    code, text = act(tmp, 'verify')
    check('w-master-changed', code != 0 and "assert T.sha(Path(value['path'])) == value['sha256']" in text and not H.MANIFEST.exists(),
          'a master changed after mastering refuses before any manifest or job')
    master.write_bytes(data)
    pins = H.V.PINS
    H.V.PINS = {name: '0' * 64 for name in pins}
    check('w-tool-pin', refused(H.plan, 'AssertionError'), 'a strict check or caption screen off its pin is refused')
    H.V.PINS = pins
    for case, words in (('guard-policy', 'Guard policy changed'), ('input-changed', ROW_CHANGED)):
        set_case(tmp, case)
        code, text = act(tmp, 'verify')
        ok = code != 0 and words in text and not (H.OUT / 'verification.json').exists()
        check(f'w-{case}', ok, f'refused ({words[:48]}) with no verification.json' + ('' if ok else ' ' + text[-300:]))
        shelve(tmp, 'verify-' + case, [H.OUT, H.MANIFEST])
    set_case(tmp, 'pass')


def verify_pass(tmp, H):
    M, F, T = H.M, H.M.F, H.M.F.T
    code, text = act(tmp, 'verify')
    path = H.OUT / 'verification.json'
    check('e-verify', code == 0 and path.exists(), 'the verifier finishes on the fixture master' + ('' if code == 0 else ' ' + text[-600:]))
    data = json.loads(path.read_text())
    strict = (H.OUT / 'strict-video-verification.log').read_text().splitlines()
    check('e-strict-real', strict[-1] == 'VERIFY PASS' and H.V.software_decode_clean(data['full_decode'])
          and data['media']['video_frames'] == FRAMES * 15 and abs(float(data['encoded_loudness']['integrated_lufs']) + 16) <= 1,
          'the real pinned strict check, full decode, frame and A/V facts and ebur128 all ran on the fixture master')
    check('e-captions-real', data['captions']['caption_blocks'] == 105 and data['captions']['text_equals_script'] == 'whole and per cue'
          and data['caption_screen']['caption_count'] == 105, 'SRT equals VTT and the script, whole and per cue, over the 105 real cue texts, and the real caption screen ran')
    pictures = data['pictures']
    spans = {row['chapter']: (row['start_frame'], row['start_frame'] + row['frames']) for row in data['chapters']}
    with Image.open(pictures[-1]['path']) as last:
        size = last.size
    check('e-final-sheets', len(data['sheets']) == math.ceil(len(pictures) / 4) and size == (960, 540)
          and all(sha(p['path']) == p['sha256'] for p in pictures) and data['sheet_sha256'] == {p: sha(p) for p in data['sheets']}
          and all(spans[p['chapter']][0] <= p['frame'] < spans[p['chapter']][1] for p in pictures),
          f"{len(pictures)} final frames at master time and {len(data['sheets'])} sheets are extracted and hash-bound")
    joined = ' | '.join(reason for p in pictures for reason in p['reasons'])
    wanted = ['last frame', ' start', 'provisional sentence', 'provisional phrase', 'authored hold', 'still there at 3 later looks', 'stands 0.20 from the edge']
    sources = data['layout_sources'][T.CHAPTERS[0]]
    missing = [word for word in wanted if word not in joined]
    check('e-samples', not missing and sources[:2] == [str(F.OUT / (T.CHAPTERS[0] + '.layout.tsv')), str(F.P.OUT / (T.CHAPTERS[0] + '.layout.tsv'))],
          'cue, sentence, phrase and hold samples are kept; a persistence line only in the final lint and a line only in the preview lint both get samples'
          + (f' MISSING {missing}' if missing else ''))
    check('e-no-picture-claim', data['pictures_viewed'] is False and 'does not carry over' in data['scope'] and data['preview_review']['review_scope'] == 'source-author'
          and data['preview_review']['sha256'] == sha(F.REVIEW) and data['full_baseline_equivalence_established'] is False,
          'verification.json records pictures_viewed false, says the preview review does not carry over, and names the review scope')
    listed = {row['path'] for row in json.loads(H.MANIFEST.read_text())['files']}
    needed = {str(BUNDLE / f'final_{name}_optimized01.py') for name in ('render', 'master', 'verify')} | {str(BUNDLE / 'start-optimized-final-verify01.sh')}
    needed |= {*map(str, F.IMPORTED_SHA256), str(H.SHARED), str(H.STRICT), str(H.SCREEN), str(H.X.FFMPEG), str(F.P.PROBE), str(M.MANIFEST), str(F.REVIEW)}
    needed |= {str(path) for path in M.OUTPUTS.values()} | {str(T.B / M.JOB / 'master-summary.json'), str(T.B / M.JOB / 'resource-receipt.json')}
    check('e-verify-manifest', needed <= listed, 'the manifest binds all three helpers, all pins, the shared checks, both tools, the master outputs and the review')
    argv = json.loads((tmp / 'guard-calls.jsonl').read_text().splitlines()[-1])
    shown = [Path(x).name if x.startswith('/') and x != sys.executable else x for x in argv]
    check('e-verify-argv', shown == ['--phase', 'native', '--job', H.JOB, '--inputs', 'optimized-final-verify-inputs01.json',
                                     '--', sys.executable, '-I', '-B', 'final_verify_optimized01.py', '--run'],
          'the guard gets the same phase and the new job, manifest and child')
    code, text = act(tmp, 'verify')
    check('w-duplicate', code != 0 and 'Duplicate final verification refused' in text and path.exists(), 'a second verifier refuses and keeps the first result')


def bad_masters(tmp, H):
    """Each case needs its own hash-bound master, so these rerun the fake master."""
    M = H.M
    shelve(tmp, 'master-pass', [tmp / 'B' / M.JOB, M.MANIFEST, *M.OUTPUTS.values()])
    shelve(tmp, 'verify-pass', [H.OUT, H.MANIFEST])
    for case, words in (('caption-text', "the captions' text differs from the script"), ('caption-differs', 'SRT and VTT differ'),
                        ('master-quiet', 'CalledProcessError')):
        set_case(tmp, case)
        code, text = act(tmp, 'master')
        assert code == 0, text[-600:]
        code, text = act(tmp, 'verify')
        ok = code != 0 and words in text and not (H.OUT / 'verification.json').exists()
        if case == 'master-quiet':
            ok = ok and (H.OUT / 'strict-video-verification.log').read_text().splitlines()[-1] == 'VERIFY FAIL'
        check(f'w-{case}', ok, f'refused ({words[:48]}) with no verification.json' + ('' if ok else ' ' + text[-300:]))
        shelve(tmp, 'bad-' + case, [tmp / 'B' / M.JOB, M.MANIFEST, *M.OUTPUTS.values(), H.OUT, H.MANIFEST])


def start_scripts(root):
    tree = root / 'start'
    folder = tree / 'a/b/c/d/e'
    folder.mkdir(parents=True)
    work = tree / 'narration_lab/third-wave-20261024/root/elevenlabs-complex-rebuild-20261104/optimized-final-production01'
    for name, mode in (('render', '--all'), ('master', '--launch'), ('verify', '--launch')):
        script = folder / f'start-optimized-final-{name}01.sh'
        shutil.copy(BUNDLE / script.name, script)
        (folder / f'final_{name}_optimized01.py').write_text("import sys\nprint(sys.argv[1:])\nraise SystemExit(3)\n")
        first = subprocess.run(['/bin/bash', str(script)])
        log, code = work / f'optimized-final-{name}-controller01.log', work / f'optimized-final-{name}-controller01.exit'
        second = subprocess.run(['/bin/bash', str(script)], capture_output=True, text=True)
        check(f's-{name}', first.returncode == 3 and code.read_text() == '3\n' and log.read_text() == f"['{mode}']\n" and second.returncode == 1,
              f'the script passes only {mode}, records the exit code under the new names, and refuses a second start')


def real_controls(F):
    lane, chapter = F.T.B, '00_ratio_ruler'
    work = lane / 'logarithm-stage09-render-work02'
    frames = int(next(line.split('\t')[1] for line in (work / f'{chapter}.timeline.tsv').read_text().splitlines() if line.startswith('frames\t')))
    capture = F.captured_soundtrack(work, chapter, frames)
    check('x-real-capture', capture['samples'] == frames * 800 and capture['captures'] == 1, 'the capture check reads a real logarithm capture at 48 kHz, 16-bit stereo, frames x 800 samples')
    log = (lane / 'logarithm-stage09-master02/command.log').read_text()
    lines = [line for line in log.splitlines() if "replay = fmn's soundtrack, bit for bit" in line]
    keys = sorted(p.name[:-len('.cuelog.tsv')] for p in work.glob('*.cuelog.tsv'))
    check('x-real-proof', [line.split()[0] for line in lines] == keys and len(keys) == 13, "the proof-line parse reads logarithm's real master log, 13 of 13 chapters in order")


def preserved(F):
    native = F.T.E / 'native-stage06-root03'
    files = [native / 'final-render01.py', native / 'final-master01.py', native / 'final-verify01.py', SEALED / 'SHA256SUMS',
             BUNDLE.parent / 'complex-optimized-recovery-review01/SHA256SUMS', BUNDLE.parent / 'complex-optimized-thumbnail02/SHA256SUMS']
    return {str(p): sha(p) for p in files}


def movie(path, frames, size='3840x2160', level=5):
    """A real H.264 and AAC movie, tagged BT.709. Level 5 dB on this stereo sine measures about -16 LUFS."""
    tone = f'sine=frequency=1000:sample_rate=48000,volume={level}dB,aformat=channel_layouts=stereo'
    subprocess.run(['ffmpeg', '-y', '-v', 'error', '-nostdin', '-f', 'lavfi', '-i', f'color=c=black:s={size}:r=60', '-f', 'lavfi', '-i', tone,
                    '-frames:v', str(frames), '-t', str(frames / 60), '-c:v', 'libx264', '-preset', 'ultrafast', '-pix_fmt', 'yuv420p',
                    '-bsf:v', 'h264_metadata=colour_primaries=1:transfer_characteristics=1:matrix_coefficients=1', '-c:a', 'aac', '-b:a', '256k', str(path)], check=True)


def early_marks(tmp, T):
    """The real report and phrase marks, with marks scaled by a tenth: the earliest real one is past a fixture chapter's end."""
    folder = tmp / 'early-marks'
    folder.mkdir()
    shutil.copy(T.N / 'report.json', folder)
    data = json.loads((T.N / 'phrase-timings.json').read_text())
    for cue in data['cues'].values():
        for mark in cue['marks']:
            mark['seconds'] /= 10
    (folder / 'phrase-timings.json').write_text(json.dumps(data))


def stamp(ms, mark):
    return f'{ms // 3600000:02}:{ms // 60000 % 60:02}:{ms // 1000 % 60:02}{mark}{ms % 1000:03}'


def write_captions(tmp, T):
    """The kit's exact caption format, one block per real cue, placed on the fixture timeline."""
    script = dict(line.split('\t', 1) for line in (T.C / 'script.tsv').read_text().splitlines() if line.strip() and not line.startswith('#'))
    timeline = tmp / 'B' / T.JOB / 'timeline-output'
    starts, offset = [], 0
    for chapter in T.CHAPTERS:
        for line in (timeline / f'{chapter}.timeline.tsv').read_text().splitlines():
            row = line.split('\t')
            if row[0] == 'line':
                starts.append((row[1], round((offset + int(row[2])) * 1000 / 60)))
        offset += FRAMES
    assert [cue for cue, _ in starts] == list(script)
    srt, vtt = '', 'WEBVTT\n\n'
    for k, (cue, start) in enumerate(starts, 1):
        end = starts[k][1] if k < len(starts) else start + 10
        srt += f"{k}\n{stamp(start, ',')} --> {stamp(end, ',')}\n{script[cue]}\n\n"
        vtt += f"{stamp(start, '.')} --> {stamp(end, '.')}\n{script[cue]}\n\n"
    (tmp / 'fixture.srt').write_text(srt)
    (tmp / 'fixture.vtt').write_text(vtt)


def main():
    H = load(BUNDLE / 'final_verify_optimized01.py')
    F = H.M.F
    before = preserved(F)
    static_checks(H)
    R = load(SEALED / 'checks/rehearse_downstream.py', 'sealed_rehearsal')
    fake = R.FAKE
    for old, new in (("if args[0] == 'score':", FINAL_FAKE + "if args[0] == 'score':"),
                     ("k * (2401 if case == 'score-parts-gap' else 2400)", "k * c['frames'] * 400"),
                     ("2399 if k and case == 'score-parts-short' else 2400", "c['frames'] * 400")):
        assert fake.count(old) == 1, old
        fake = fake.replace(old, new)
    R.FAKE, R.FRAMES = fake, FRAMES
    jobs = {stage: R.load(stage).JOB for stage in R.STAGES}
    with tempfile.TemporaryDirectory() as folder:
        root = Path(folder)
        shared = root / 'shared'
        shared.mkdir()
        R.movies(shared)
        movie(shared / 'tiny.mp4', FRAMES, '960x540')
        tmp = root / 'tree'
        R.fixture('pass', tmp, shared)
        movie(tmp / 'final.mp4', FRAMES)
        movie(tmp / 'master.mp4', FRAMES * 15)
        movie(tmp / 'quiet.mp4', FRAMES * 15, level=-30)
        write_captions(tmp, F.T)
        early_marks(tmp, F.T)
        for stage in ('score', 'preview', 'pictures'):
            code, text = R.stage_run(stage, tmp, jobs)
            assert code == 0, text[-800:]
        config = json.loads((tmp / 'config.json').read_text())
        config.update(preview_out=str(tmp / 'B' / jobs['preview'] / 'preview'), stems=str(tmp / 'B' / jobs['score'] / 'scored/stems'))
        (tmp / 'config.json').write_text(json.dumps(config))
        (tmp / 'fake-guard').write_text(GUARD.replace('CHECK', repr(str(HERE))).replace('WORK', repr(str(tmp))))
        V = chain(tmp, 'verify')
        M, fixture_F = V.M, V.M.F
        review_checks(tmp, fixture_F)
        render_checks(tmp, fixture_F)
        render_pass(tmp, fixture_F)
        master_failures(tmp, fixture_F, M, V)
        master_pass(tmp, fixture_F, M)
        verify_refusals(tmp, V)
        verify_pass(tmp, V)
        bad_masters(tmp, V)
        start_scripts(root)
    real_controls(F)
    check('p-preserved', preserved(F) == before and sha(SEALED / 'SHA256SUMS') == SEALED_SUMS
          and not (F.T.B / F.job(F.T.CHAPTERS[0])).exists() and not (F.T.B / H.M.JOB).exists() and not (F.T.B / H.JOB).exists() and not F.OUT.exists(),
          "root's three final helpers and the three seals are byte-identical, and the real names are still unused")
    print(f"RESULT {'PASS' if all(results) else 'FAIL'}, {results.count(False)} failed")
    return 0 if all(results) else 1


if __name__ == '__main__':
    mode = sys.argv[1:2]
    if mode == ['--guard']:
        sys.exit(fake_guard(Path(sys.argv[2]), sys.argv[3:]))
    elif mode == ['--child']:
        work, args = Path(sys.argv[2]), sys.argv[3:]
        which = Path(args[0]).name.split('_')[1]
        helper = chain(work, which)
        helper.render(args[2]) if which == 'render' else helper.run()
    elif mode == ['--act']:
        which = sys.argv[3]
        helper = chain(Path(sys.argv[2]), which)
        helper.launch_all() if which == 'render' else helper.launch()
    else:
        sys.exit(main())
