#!/usr/bin/env python3
"""Rehearse the three final helpers on the sealed N2 fixture, in a tree whose repository path is a symlink.

Real: the helpers (only their hash pins change, in the fixture copies), the sealed qualifier and the
sealed chain named below, run first through their own start scripts, all 105 cues and takes, the unmodified
narration_delivery.py and coherent_narration.py, the real guard (hostname changed), the pinned shared
media checks, the strict video check and caption screen, ffmpeg, ffprobe and PIL, and the three start
scripts. Fake: the binary, its tiny 36-frame movies, its tables, the macOS receipts and sbh.

It proves refusals, names and records. It renders no real frame, proves no 4K speed or size, and says
nothing about pictures, sound or sync. Nothing is written in the repository except its own output
file, which the caller redirects. The fixture stays where tempfile puts it; set TMPDIR first. From the
repository root:

    python3 -I -B <bundle>/checks/rehearse_final.py
"""
import ast
from collections import Counter
import contextlib
import fcntl
import hashlib
import importlib.util
import json
from pathlib import Path
import re
import shutil
import subprocess
import sys
import tempfile

HERE = Path(__file__).resolve()
BUNDLE = HERE.parents[1]
P = BUNDLE.parent
# Everything chain-specific in this file. A new chain changes these, and its fixture must keep rehearse_n2's Real.
CHAIN_FOLDER = P / 'complex-remote-ts1-pipeline01/chain03'
CHAIN_FILE = 'ts1_chain03.py'
CHAIN_SEAL = 'bbf00548ba5f804fcecefe31854ffa9af60494ba71003cfc2ccd491fa871daf9'
CHAIN_REHEARSAL = CHAIN_FOLDER / 'checks/rehearse_chain03.py'
STAGE_JOB = {stage: f'complex-remote-{stage}03' for stage in ('timeline', 'score', 'preview', 'pictures')}
PICTURES_INPUTS = 'chain-pictures-inputs03.json'
BUILD_SUMMARY = 'complex-remote-build02/build-summary.json'
SPEC = importlib.util.spec_from_file_location('rehearse_chain03', CHAIN_REHEARSAL)
N = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(N)
C, Q = N.C, N.Q
REAL = C.REAL
sha = Q.sha
FRAMES = 36  # every cue gets its own frame, as in WhiteSwan's check
C.FRAMES = FRAMES
FOLDER = str(BUNDLE.relative_to(REAL))
WRONG_SIZE = "assert (v['width'], v['height'], v['avg_frame_rate'], int(v['nb_frames'])) == (3840, 2160"
HELPERS = ('render', 'master', 'verify')
PATHS = {'render': 'ts1_final_render01.py', 'master': 'ts1_final_master01.py', 'verify': 'ts1_final_verify01.py'}
WHITE = {name: P / f'complex-optimized-final-production01/final_{name}_optimized01.py' for name in HELPERS}
MASTER = 'complex-remote-master01'
VERIFY = 'complex-remote-final-verify01'

FINAL = r'''
import wave
FINAL_LINT = '02_1 +0.0s @0.10s\tstuck\t"w" still there at 3 later looks\n'
def cue_row(path, frame):
    with wave.open(path) as sound:
        rate, channels, samples = sound.getframerate(), sound.getnchannels(), sound.getnframes()
    data = Path(path).read_bytes()
    zero = '0x0000000000000000'
    return '\t'.join([path, hashlib.sha256(data).hexdigest(), str(len(data)), str(frame), '60', '0.0', zero, '0.0', zero,
                      str(rate), str(channels), str(samples), str(frame * 800)]) + '\n'
faults = c.get('faults', {})
out = Path(args[args.index('--out') + 1]) if '--out' in args else None
if args[0] == 'render' and args[1] != 'all':
    chapter = args[1]
    fault = faults.get(chapter, '')
    assert args[2:12] == ['--res', '3840x2160', '--fps', '60', '--narration', c['n2'], '--score', c['stems'], '--out', str(out)]
    assert args[12:] == ['--lint', '--crf', '16', '--preset', 'veryfast']
    deliver('render', c['n2'], ['--score', c['stems']])
    if fault == 'render-exit':
        sys.exit(3)
    if fault == 'input-changed':
        with open(c['frozen'], 'a') as handle:
            handle.write('\n')
    out.mkdir(exist_ok=True)
    preview = Path(c['preview_out'])
    for name in ('timeline', 'cues', 'cuelog', 'layout'):
        if fault == 'layout-missing' and name == 'layout':
            continue
        text = (preview / f'{chapter}.{name}.tsv').read_text()
        if name == 'layout' and chapter == c['chapters'][0]:
            text = FINAL_LINT
        (out / f'{chapter}.{name}.tsv').write_text(text + ('x\t1\n' if fault == name + '-differs' else ''))
    movie = {'res-wrong': 'fake-tiny.mp4', 'crf-wrong': 'fake-crf23.mp4'}.get(fault, 'fake-final.mp4')
    shutil.copy(here / movie, out / f'{chapter}.mp4')
    folder = out / 'soundtrack' / chapter
    folder.mkdir(parents=True)
    if fault != 'capture-missing':
        with wave.open(str(folder / 'soundtrack_0.wav'), 'wb') as handle:
            handle.setnchannels(2)
            handle.setsampwidth(2)
            handle.setframerate(48000)
            handle.writeframes(bytes(4 * (c['frames'] * 800 - (1 if fault == 'capture-short' else 0))))
    sys.exit(0)
if args[0] == 'master':
    assert args[1:] == ['--narration', c['n2'], '--score', c['stems'], '--out', str(out)]
    deliver('master', c['n2'], ['--timelines', str(out), '--score', c['stems']])
    fault = faults.get('master', '')
    (out / 'master').mkdir()
    (out / 'master' / '01.video.mp4').write_text('a kit work file')
    if fault == 'master-exit':
        sys.exit(3)
    for n, chapter in enumerate(c['chapters']):
        if not (fault == 'proof-missing' and n == 7):
            print(f"{chapter} replay = fmn's soundtrack, bit for bit (7 cues, 4800 samples)")
    name = {'master-wrong': 'fake-tiny.mp4', 'master-quiet': 'fake-quiet.mp4'}.get(fault, 'fake-master.mp4')
    shutil.copy(here / name, out / 'complex_analysis_visual_master.mp4')
    sys.exit(0)
if args[0] == 'captions':
    assert args[1:] == ['all', '--narration', c['n2'], '--out', str(out)]
    deliver('captions', c['n2'], ['--timelines', str(out)])
    fault = faults.get('master', '')
    if fault == 'captions-exit':
        sys.exit(3)
    srt, vtt = (here / 'fixture.srt').read_text(), (here / 'fixture.vtt').read_text()
    if fault == 'srt-empty':
        srt = ''
    if fault == 'caption-text':
        srt, vtt = srt[:-2] + ' extra\n\n', vtt[:-2] + ' extra\n\n'
    if fault == 'caption-differs':
        vtt = vtt[:-2] + ' extra\n\n'
    (out / 'complex_analysis_visual.en.srt').write_text(srt)
    (out / 'complex_analysis_visual.en.vtt').write_text(vtt)
    sys.exit(0)
'''
EDITS = [
    ("if args[0] == 'facts':\n", FINAL + "if args[0] == 'facts':\n"),
    (r'''f"{wav}{cue}.wav\t{'y' if case == 'hash-cell' and k == 2 else 'x'}\t8\t{min(k, c['frames'] - 1)}\t60\t0.0\t0x0\t-\t-\t24000\t1\t1\t0\n" for k, cue in enumerate(ids)''',
     r'''cue_row(f'{wav}{cue}.wav', min(k, c['frames'] - 1)) for k, cue in enumerate(ids)'''),
    (r"''.join(f'{k * 2400}\t2400\t48000\tx\t{name}\n' for k, name in enumerate(names))",
     r'''''.join(f"{k * c['frames'] * 400}\t{c['frames'] * 400}\t48000\t{hashlib.sha256((stem.parent / name).read_bytes()).hexdigest()}\t{name}\n" for k, name in enumerate(names))'''),
]
for old, new in EDITS:
    assert C.FAKE.count(old) == 1, old
    C.FAKE = C.FAKE.replace(old, new)

PROBE_HEAD = r'''
import importlib.util, json, sys
from pathlib import Path
spec = importlib.util.spec_from_file_location('f', sys.argv[1])
F = importlib.util.module_from_spec(spec)
spec.loader.exec_module(F)
M, L, Q = F.M, F.L, F.Q
def seen(action):
    try:
        action()
        return 'no refusal'
    except (AssertionError, KeyError, OSError, ValueError) as error:
        return f'{type(error).__name__} {error}'
def say(name, text):
    print(json.dumps([name, text]), flush=True)
'''
PROBE_REVIEW = PROBE_HEAD + r'''
good = F.read(F.REVIEW)
text = json.dumps(good)
first = next(iter(good['sheet_sha256']))
extra = str(Path(first).with_name('sheet-099.png'))
def attempt(name, change=lambda r: None):
    review = json.loads(text)
    change(review)
    F.REVIEW.write_text(json.dumps(review))
    say(name, seen(F.check_review))
    F.REVIEW.write_text(text)
attempt('valid')
for name, change in {
        'no-viewer': lambda r: r.update(reviewer=' '), 'no-viewer-field': lambda r: r.pop('reviewer'),
        'not-viewed': lambda r: r.update(pictures_viewed=False), 'no-scope': lambda r: r.pop('review_scope'),
        'vague-scope': lambda r: r.update(review_scope='reviewed'), 'open-defect': lambda r: r.update(unresolved_visual_defects=['x']),
        'chapters-short': lambda r: r['chapters'].pop(), 'sheet-dropped': lambda r: r['sheet_sha256'].pop(first),
        'sheet-hash': lambda r: r['sheet_sha256'].update({first: '0' * 64}), 'sheet-extra': lambda r: r['sheet_sha256'].update({extra: '0' * 64}),
        'extraction-hash': lambda r: r.update(extraction_sha256='0' * 64), 'summary-hash': lambda r: r.update(preview_summary_sha256='0' * 64)}.items():
    attempt(name, change)
def damaged(name, path, change=lambda r: None):
    data = path.read_bytes()
    path.write_bytes(data + b'x')
    attempt(name, change)
    path.write_bytes(data)
damaged('preview-movie-changed', M.PREVIEW_OUT / (M.CHAPTERS[3] + '.mp4'))
damaged('n2-take-changed', M.N / '01_1.wav')
path = F.JOBS / 'complex-remote-build02/build-summary.json'
saved = path.read_bytes()
path.write_text(json.dumps(dict(json.loads(saved), binary_sha256='0' * 64)))
attempt('qualification-changed')
path.write_bytes(saved)
sheet, moved = Path(first), Path(first).with_suffix('.moved')
sheet.rename(moved)
attempt('sheet-gone-from-both', lambda r: r['sheet_sha256'].pop(first))
moved.rename(sheet)
path = M.job('preview') / 'preview-summary.json'
saved = path.read_bytes()
for name, key in (('other-voice', 'voice'), ('other-binary', 'binary_sha256')):
    other = json.loads(saved)
    other[key] = 'another'
    path.write_text(json.dumps(other))
    attempt(name, lambda r: r.update(preview_summary_sha256=F.sha(path)))
path.write_bytes(saved)
movie = M.PREVIEW_OUT / (M.CHAPTERS[0] + '.mp4')
kept = movie.read_bytes()
movie.write_bytes((F.O / 'fake-crf23.mp4').read_bytes())
other = json.loads(saved)
other['chapters'][0]['mp4_sha256'] = F.sha(movie)
path.write_text(json.dumps(other))
attempt('preview-no-x264-record', lambda r: r.update(preview_summary_sha256=F.sha(path)))
movie.write_bytes(kept)
path.write_bytes(saved)
attempt('valid-again')
say('chain-names-hold', seen(F.check_chain_names))
kept = L.check_derived
del L.check_derived
say('chain-name-missing', seen(F.check_chain_names))
L.check_derived = kept
stage = M.NAMES.pop('derive')
say('chain-stage-missing', seen(F.check_chain_names))
M.NAMES['derive'] = stage
summary_path = M.job('derive') / 'derive-summary.json'
kept = summary_path.read_bytes()
summary_path.write_text(json.dumps(dict(json.loads(kept), derived_from='another folder')))
say('derive-summary-names-another', seen(F.narration_source))
summary_path.write_bytes(kept)
source = F.narration_source()
say('source-matches', seen(lambda: F.same_source(source)))
say('source-other-hash', seen(lambda: F.same_source(dict(source, derive_summary_sha256='0' * 64))))
say('source-absent', seen(lambda: F.same_source({})))
real, calls = F.read, []
def flaky(where):
    calls.append(where)
    if len(calls) < 3:
        raise json.JSONDecodeError('half written', '', 0)
    return {'admitted': True}
F.read = flaky
say('receipt-reread', f"{F.live_receipt('x')['admitted']} {len(calls)}")
F.time.sleep = lambda seconds: None
def broken(where):
    raise json.JSONDecodeError('half written', '', 0)
F.read = broken
say('receipt-never-settles', seen(lambda: F.live_receipt('x')))
def absent(where):
    raise FileNotFoundError('no receipt')
F.read = absent
say('receipt-missing-not-retried', seen(lambda: F.live_receipt('x')))
'''
PROBE_GATE = PROBE_HEAD + r'''
name = F.job(M.CHAPTERS[0])
inputs = F.manifest(name)
base = F.read(F.JOBS / name / 'resource-receipt.json')
base.pop('exit')
command = base['command']
rows = F.read(inputs)['files']
def attempt(label, change=lambda r: None, manifest=inputs):
    receipt = json.loads(json.dumps(base))
    change(receipt)
    F.live_receipt = lambda job: receipt
    say(label, seen(lambda: F.gate(name, command, manifest)))
attempt('gate-holds')
attempt('gate-not-admitted', lambda r: r.update(admitted=False))
attempt('gate-already-exited', lambda r: r.update(exit=0))
attempt('gate-other-command', lambda r: r.update(command=['other']))
attempt('gate-other-policy', lambda r: r.update(policy=dict(Q.POLICY, work_cap_gib=48)))
other = Path(sys.argv[2])
other.write_text(json.dumps({'files': [dict(rows[0], sha256='0' * 64), *rows[1:]]}))
attempt('gate-row-changed', manifest=other)
other = Path(sys.argv[3])
gone = str(M.tool('ffprobe'))
other.write_text(json.dumps({'files': [row for row in rows if row['path'] != gone]}))
attempt('gate-tool-unbound', manifest=other)
'''


def check(name, ok, detail=''):
    C.check(name, ok, detail)


def last(text, empty='no output'):
    return C.last(text, empty)


def text_of(path):
    return path.read_text(errors='replace') if path.exists() else ''


def movie(path, frames, size='3840x2160', level=5, crf=16):
    """WhiteSwan's fixture movie, with x264's rate settings named. Level 5 dB measures about -16 LUFS."""
    tone = f'sine=frequency=1000:sample_rate=48000,volume={level}dB,aformat=channel_layouts=stereo'
    subprocess.run(['ffmpeg', '-y', '-v', 'error', '-nostdin', '-f', 'lavfi', '-i', f'color=c=black:s={size}:r=60', '-f', 'lavfi', '-i', tone,
                    '-frames:v', str(frames), '-t', str(frames / 60), '-c:v', 'libx264', '-preset', 'ultrafast', '-crf', str(crf), '-pix_fmt', 'yuv420p',
                    '-bsf:v', 'h264_metadata=colour_primaries=1:transfer_characteristics=1:matrix_coefficients=1', '-c:a', 'aac', '-b:a', '256k', str(path)], check=True)


def stamp(ms, mark):
    return f'{ms // 3600000:02}:{ms // 60000 % 60:02}:{ms // 1000 % 60:02}{mark}{ms % 1000:03}'


def mode_flags(text):
    main = ast.parse(text).body[-1]
    return {node.value for node in ast.walk(main) if isinstance(node, ast.Constant) and isinstance(node.value, str) and node.value.startswith('--')}


def timeouts(path):
    return Counter(re.findall(r'timeout=(\d+)', Path(path).read_text()))


def static_checks():
    texts = {name: (BUNDLE / PATHS[name]).read_text() for name in HELPERS}
    shells = [subprocess.run(['/bin/bash', '-n', str(BUNDLE / f'start-final-{name}01.sh')]).returncode for name in HELPERS]
    for text in texts.values():
        ast.parse(text)
    check('a-parse', shells == [0, 0, 0], 'all three helpers and start scripts parse')
    chain = CHAIN_FOLDER
    sums = (chain / 'SHA256SUMS').read_text()
    pin = lambda name, key: texts[name].split(key)[1].split("'")[1]
    check('a-pins', sha(chain / 'SHA256SUMS') == CHAIN_SEAL and f"{pin('render', 'CHAIN_SHA256 = ')}  {CHAIN_FILE}\n" in sums
          and pin('master', 'RENDER_SHA256 = ') == sha(BUNDLE / PATHS['render']) and pin('verify', 'MASTER_HELPER: ') == sha(BUNDLE / PATHS['master'])
          and pin('verify', 'SHARED: ') == sha(P / 'logarithm-stage09-final-verification02/verify_stage09_final.py'),
          "verify pins master and the shared checks, master pins render, render pins WhiteSwan's sealed chain")
    check('a-modes', mode_flags(texts['render']) == {'--check', '--check-review', '--all', '--run'} and mode_flags(texts['master']) == {'--check', '--launch', '--run'}
          and mode_flags(texts['verify']) == {'--pins', '--check', '--launch', '--postcheck', '--run'}, 'no mode skips the review or the guard')
    check('a-4k', all(item in texts['render'] for item in ("'--res', '3840x2160', '--fps', '60'", "'--lint', '--crf', '16', '--preset', 'veryfast'"))
          and "[str(BIN), 'master'" in texts['master'] and "[str(BIN), 'captions', 'all'" in texts['master'], '4K60, CRF 16, veryfast, lint, master and captions')
    lost = {name: Counter() for name in HELPERS}
    added = {}
    for name in HELPERS:
        ours, theirs = timeouts(BUNDLE / PATHS[name]), timeouts(WHITE[name])
        lost[name], added[name] = theirs - ours, sorted((ours - theirs).elements())
    check('a-timeouts', not any(lost.values()), f"every timeout of WhiteSwan's helpers kept; added for new checks: {added}")
    names = {node.func.attr for text in texts.values() for node in ast.walk(ast.parse(text)) if isinstance(node, ast.Call) and isinstance(node.func, ast.Attribute)}
    check('a-no-deletion', not names & {'unlink', 'rmtree', 'rmdir', 'remove', 'removedirs'}, 'no helper calls a deleting function')
    check('a-no-unused-chain', not any("chain02/" in text for text in texts.values()), 'the unused chain02 attempt is never loaded')
    used = {'L': set(), 'M': set(), 'Q': set()}
    for text in texts.values():
        for node in ast.walk(ast.parse(text)):
            if isinstance(node, ast.Attribute) and isinstance(node.value, ast.Name) and node.value.id in used:
                used[node.value.id].add(node.attr)
    declared = next(ast.literal_eval(node.value) for node in ast.parse(texts['render']).body if isinstance(node, ast.Assign) and getattr(node.targets[0], 'id', '') == 'CHAIN_NAMES')
    check('a-chain-names-complete', {key: set(names) for key, names in declared.items()} == used, f"{sum(map(len, used.values()))} names, exactly those the three helpers use")


def repin_checks(root):
    """The tool, on copies, with a stand-in sealed chain. Nothing here reads a real new chain."""
    tree = root / 'repin'
    bundle = tree / 'bundle'
    (bundle / 'checks').mkdir(parents=True)
    for name in PATHS.values():
        shutil.copy(BUNDLE / name, bundle)
    shutil.copy(BUNDLE / 'checks/repin_chain01.py', bundle / 'checks')
    folder = tree / 'next-chain'
    folder.mkdir()
    (folder / 'ts1_next.py').write_text('# a stand-in chain\n')
    good = sha(folder / 'ts1_next.py')
    (folder / 'SHA256SUMS').write_text(f'{good}  ts1_next.py\n')
    state = lambda: {name: sha(bundle / name) for name in PATHS.values()}
    call = lambda digest, *more: subprocess.run([sys.executable, '-I', '-B', str(bundle / 'checks/repin_chain01.py'), '--chain', 'next-chain/ts1_next.py', '--sha256', digest, *more], capture_output=True, text=True)
    before = state()
    dry = call(good)
    check('a-repin-dry-run', dry.returncode == 0 and json.loads(dry.stdout)['changed'] == list(PATHS.values()) and state() == before, 'prints three new hashes and writes nothing')
    wrong = call('0' * 64, '--write')
    check('a-repin-wrong-hash', wrong.returncode != 0 and 'does not have this hash' in wrong.stderr and state() == before, last(wrong.stderr))
    (folder / 'SHA256SUMS').rename(folder / 'moved-sums')
    unsealed = call(good, '--write')
    (folder / 'moved-sums').rename(folder / 'SHA256SUMS')
    check('a-repin-unsealed-chain', unsealed.returncode != 0 and 'seal does not list this hash' in unsealed.stderr and state() == before, last(unsealed.stderr))
    done = call(good, '--write')
    render, master, verify = ((bundle / PATHS[name]).read_text() for name in HELPERS)
    check('a-repin-writes-in-order', done.returncode == 0 and "CHAIN = HERE.parent / 'next-chain/ts1_next.py'\n" in render and f"CHAIN_SHA256 = '{good}'\n" in render
          and f"RENDER_SHA256 = '{sha(bundle / PATHS['render'])}'\n" in master and f"MASTER_HELPER: '{sha(bundle / PATHS['master'])}',\n" in verify, 'chain, then render into master, then master into verify')
    (bundle / 'SHA256SUMS').write_text('sealed\n')
    sealed = call(good, '--write')
    check('a-repin-sealed-bundle', sealed.returncode != 0 and 'This bundle is sealed' in sealed.stderr, last(sealed.stderr))


class Final(N.Real):
    shift = 0  # the fixture soundtrack needs every chapter at FRAMES, so chapter 09's real growth is not rehearsed here

    def __init__(self, base, case, shared):
        self.runs = Counter()
        super().__init__(base, case, shared)

    def config(self, case):
        super().config(case)
        path = self.O / 'fake-config.json'
        data = json.loads(path.read_text())
        data.update(n2=str(self.n2), stems=str(self.J / STAGE_JOB['score'] / 'scored/stems'), preview_out=str(self.J / STAGE_JOB['preview'] / 'preview'),
                    frozen=str(self.R / FOLDER / 'transfer-files01.json'), faults={})
        path.write_text(json.dumps(data))

    def bundle(self):
        self.D = self.R / FOLDER
        self.D.mkdir()
        self.stage_dir = self.O / 'final-production01'
        self.OUT = self.J / 'complex-remote-render-work01'
        self.ASIDE = self.J / 'complex-remote-render-aside01'
        for name in ('start-final-render01.sh', 'start-final-master01.sh', 'start-final-verify01.sh', 'transfer-files01.json'):
            shutil.copy(BUNDLE / name, self.D)
        rows = json.loads((BUNDLE / 'transfer-files01.json').read_text())['files']
        for row in rows:
            target = self.R / row['path']
            target.parent.mkdir(parents=True, exist_ok=True)
            shutil.copy(REAL / row['path'], target)
        text = (BUNDLE / PATHS['render']).read_text()
        (self.D / PATHS['render']).write_text(text.replace(text.split("CHAIN_SHA256 = '")[1].split("'")[0], sha(self.script)))
        text = (BUNDLE / PATHS['master']).read_text()
        (self.D / PATHS['master']).write_text(text.replace(text.split("RENDER_SHA256 = '")[1].split("'")[0], sha(self.D / PATHS['render'])))
        text = (BUNDLE / PATHS['verify']).read_text()
        (self.D / PATHS['verify']).write_text(text.replace(text.split("MASTER_HELPER: '")[1].split("'")[0], sha(self.D / PATHS['master'])))
        self.safe_env = dict(self.env, PATH=f"{self.env['HOME']}/.local/bin:{self.env['HOME']}/.cargo/bin:/usr/bin:/bin")

    def faults(self, faults=None):
        path = self.O / 'fake-config.json'
        data = json.loads(path.read_text())
        data['faults'] = faults or {}
        path.write_text(json.dumps(data))

    def go(self, kind, *args):
        self.runs[kind] += 1
        run = f'{self.runs[kind]:02}'
        done = subprocess.run(['bash', str(self.D / f'start-final-{kind}01.sh'), run, *args], env=self.env, capture_output=True, text=True)
        return done.returncode, text_of(self.stage_dir / f'final-{kind}-controller{run}.log')

    def mode(self, name, *args):
        return subprocess.run([self.python, '-I', '-B', str(self.D / PATHS[name]), *args], cwd=self.R, env=self.safe_env, capture_output=True, text=True)

    def probe(self, code, *args):
        run = subprocess.run([self.python, '-I', '-B', '-c', code, str(self.D / PATHS['render']), *args], cwd=self.R, env=self.safe_env, capture_output=True, text=True)
        return dict(json.loads(line) for line in run.stdout.splitlines()), run

    def joblog(self, name):
        return text_of(self.J / name / 'command.log')

    def listed(self, path):
        return {row['path']: row['sha256'] for row in json.loads(path.read_text())['files']}

    def review(self):
        folder = self.J / STAGE_JOB['pictures'] / 'pictures'
        self.stage_dir.mkdir(exist_ok=True)
        review = {'reviewer': 'fixture reviewer', 'review_scope': 'source-author', 'pictures_viewed': True, 'chapters': list(Q.CHAPTERS), 'unresolved_visual_defects': [],
                  'sheet_sha256': {str(p): sha(p) for p in sorted(folder.glob('*/sheet-*.png'))}, 'extraction_sha256': sha(folder / 'extraction.json'),
                  'preview_summary_sha256': sha(self.J / STAGE_JOB['preview'] / 'preview-summary.json')}
        (self.stage_dir / 'preview-pictures-viewed01.json').write_text(json.dumps(review))

    def captions(self):
        """The kit's exact caption format, one block per real cue, placed on the fixture timeline."""
        script = dict(line.split('\t', 1) for line in (self.C / 'script.tsv').read_text().splitlines() if line.strip() and not line.startswith('#'))
        starts, offset = [], 0
        for chapter in Q.CHAPTERS:
            for line in (self.J / STAGE_JOB['timeline'] / 'timeline-output' / f'{chapter}.timeline.tsv').read_text().splitlines():
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
        (self.O / 'fixture.srt').write_text(srt)
        (self.O / 'fixture.vtt').write_text(vtt)


@contextlib.contextmanager
def held(path):
    with path.open('a') as handle:
        fcntl.flock(handle, fcntl.LOCK_EX)
        yield


def flipped(path):
    data = bytearray(path.read_bytes())
    data[len(data) // 2] ^= 1
    path.write_bytes(data)


def probe_results(t, code, expected, *args):
    got, run = t.probe(code, *args)
    if set(got) != set(expected):
        check('probe-complete', False, f"{sorted(set(expected) - set(got))}: {last(run.stderr)}")
    for name, words in expected.items():
        check('v-' + name, name in got and words in got[name], re.sub(r'/\S+', '<path>', got.get(name, 'not reached'))[:120])


REVIEW_EXPECTED = {
    'valid': 'no refusal', 'no-viewer': 'Review names no viewer', 'no-viewer-field': 'KeyError', 'not-viewed': 'Review names no viewer', 'no-scope': 'KeyError',
    'vague-scope': 'Review scope must be explicit', 'open-defect': 'AssertionError', 'chapters-short': 'AssertionError', 'sheet-dropped': 'AssertionError',
    'sheet-hash': 'AssertionError', 'sheet-extra': 'AssertionError', 'extraction-hash': 'AssertionError', 'summary-hash': 'AssertionError',
    'preview-movie-changed': 'AssertionError', 'n2-take-changed': 'N2 differs', 'qualification-changed': 'Build summary differs', 'sheet-gone-from-both': 'Sheets differ from the extraction',
    'other-voice': 'Preview is another film', 'other-binary': 'Preview is another film', 'preview-no-x264-record': 'No x264 record of CRF 18', 'valid-again': 'no refusal',
    'chain-names-hold': 'no refusal', 'chain-name-missing': 'The pinned chain lacks', 'chain-stage-missing': 'The pinned chain lacks',
    'derive-summary-names-another': 'Derive summary names another folder', 'source-matches': 'no refusal', 'source-other-hash': 'Narration provenance differs', 'source-absent': 'Narration provenance differs', 'receipt-reread': 'True 3',
    'receipt-never-settles': 'The guard receipt never settled', 'receipt-missing-not-retried': 'FileNotFoundError'}
GATE_EXPECTED = {
    'gate-holds': 'no refusal', 'gate-not-admitted': 'AssertionError', 'gate-already-exited': 'AssertionError', 'gate-other-command': 'AssertionError',
    'gate-other-policy': 'Guard policy changed', 'gate-row-changed': 'AssertionError', 'gate-tool-unbound': 'Child resolves a ffprobe outside the manifest'}


def real_controls(t):
    """Read-only, on the real logarithm stage09 files this host keeps: real kit output, not the fixture's."""
    lane = REAL / Q.B
    work = lane / 'logarithm-stage09-render-work02'
    chapter = '05_complex_turns'
    frames = int(next(line.split('\t')[1] for line in (work / f'{chapter}.timeline.tsv').read_text().splitlines() if line.startswith('frames\t')))
    code = PROBE_HEAD + r'''
work, chapter, frames = Path(sys.argv[2]), sys.argv[3], int(sys.argv[4])
say('movie', F.movie_facts(work / (chapter + '.mp4'), frames))
say('capture', json.dumps(F.captured_soundtrack(work, chapter, frames)))
'''
    got, run = t.probe(code, str(work), chapter, str(frames))
    settings = got.get('movie', '')
    capture = json.loads(got.get('capture', '{}'))
    check('x-real-x264-record', settings.startswith('x264 - core') and ' rc=crf' in settings and ' crf=16.0' in settings and '\0' not in settings, last(run.stderr, "CRF 16 read from a real 4K60 kit chapter's own stream"))
    check('x-real-capture', capture.get('samples') == frames * 800, 'the capture check reads a real capture at 48 kHz, 16-bit stereo, frames x 800 samples')
    log = (lane / 'logarithm-stage09-master02/command.log').read_text(errors='replace')
    lines = [line for line in log.splitlines() if "replay = fmn's soundtrack, bit for bit" in line]
    keys = sorted(p.name[:-len('.cuelog.tsv')] for p in work.glob('*.cuelog.tsv'))
    check('x-real-proof-lines', [line.split()[0] for line in lines] == keys and len(keys) == 13, "the proof-line parse reads a real master log, 13 of 13 chapters in order")


def before_review(t):
    run = t.mode('render', '--check')
    d = json.loads(run.stdout) if run.returncode == 0 else {}
    check('c-check-before-review', d.get('launched') is False and d.get('review_present') is False and d.get('transferred_files') == 3 and d.get('chapter_jobs') == 15, last(run.stderr, 'read-only'))
    run = t.mode('verify', '--pins')
    check('c-pins-before-master', run.returncode == 0 and '"launched": false' in run.stdout, last(run.stderr, 'read-only'))
    gone = t.R / json.loads((BUNDLE / 'transfer-files01.json').read_text())['files'][1]['path']
    gone.rename(gone.with_suffix('.moved'))
    run = t.mode('render', '--check')
    check('c-transfer-missing', run.returncode != 0 and 'Transfer this file to ts1: ' in run.stderr, last(run.stderr))
    gone.with_suffix('.moved').rename(gone)
    run = t.mode('render', '--check-review')
    check('c-review-missing', run.returncode != 0 and 'Actual narrated preview review is missing' in run.stderr, last(run.stderr))
    run = t.mode('master', '--check')
    check('c-master-check-no-review', run.returncode != 0 and 'Actual narrated preview review is missing' in run.stderr, last(run.stderr))
    code, text = t.go('render')
    check('c-all-refused-without-review', code != 0 and 'Actual narrated preview review is missing' in text and not list(t.J.glob('complex-remote-render-*')) and not list(t.stage_dir.glob('*-inputs.json')),
          'no job name or manifest is spent')


def render_failures(t):
    a, b, c, d, e, f, g, h, i = Q.CHAPTERS[:9]
    job = lambda chapter, tail='': f'complex-remote-render-{chapter}-01{tail}'

    def step(label, faults, retries, name, words):
        t.faults(faults)
        code, text = t.go('render', *[x for retry in retries for x in ('--retry', retry)])
        text += t.joblog(name)
        ok = code != 0 and words in text and not (t.J / name / 'render-summary.json').exists() and (t.J / name).is_dir()
        check('r-' + label, ok, f'{name}: {words[:48]}' if ok else last(text))

    step('render-exit', {a: 'render-exit'}, [], job(a), 'CalledProcessError')
    check('r-failed-stops-all', not (t.J / job(b)).exists(), 'the next chapter never started')
    t.faults()
    code, text = t.go('render')
    check('r-no-retry-without-name', code != 0 and f'pass --retry {a}=r2' in text and not (t.J / job(a, '-r2')).exists(), last(text))
    code, text = t.go('render', '--retry', f'{a}=r3')
    check('r-wrong-retry-name', code != 0 and f'pass --retry {a}=r2' in text and not (t.J / job(a, '-r3')).exists(), last(text))
    code, text = t.go('render', '--retry', 'nonsense=r2')
    check('r-unknown-retry-chapter', code != 0 and 'AssertionError' in text, last(text))
    sbh = Path(t.env['HOME']) / '.local/bin/sbh'
    saved = sbh.read_text()
    sbh.write_text('#!/bin/sh\necho refused\nexit 1\n')
    code, text = t.go('render', '--retry', f'{a}=r2')
    sbh.write_text(saved)
    check('r-admission-refused-burns-no-name', code != 0 and 'sbh refuses admission' in text and not (t.J / job(a, '-r2')).exists(), last(text))
    step('timeline-differs', {a: 'timeline-differs'}, [f'{a}=r2'], job(a, '-r2'), 'Final table differs')
    lock = t.O / 'expanded-native-lane.lock'
    with held(lock):
        code, text = t.go('render', '--retry', f'{a}=r3')
    check('r-busy-lane-moves-nothing', code != 0 and 'The lane is busy' in text and (t.OUT / f'{a}.mp4').exists() and not (t.J / job(a, '-r3')).exists(), last(text))
    with held(t.stage_dir / 'render.lock'):
        code, text = t.go('render', '--retry', f'{a}=r3')
    check('r-second-driver-refused', code != 0 and 'Another render driver holds this lane' in text, last(text))
    step('cues-differs', {b: 'cues-differs'}, [f'{a}=r3'], job(b), 'Final table differs')
    moved = json.loads((t.ASIDE / job(a, '-r2') / 'moved.json').read_text())
    held_copy = t.ASIDE / job(a, '-r2') / f'{a}.mp4'
    row = next(r for r in moved if r['from'].endswith('.mp4'))
    check('r-retry-preserves-attempt', sha(held_copy) == row['sha256'] and (t.ASIDE / job(a, '-r2') / a).is_dir() and len(moved) == 6, f'{len(moved)} paths with hashes, nothing deleted')
    step('cuelog-differs', {b: 'cuelog-differs'}, [f'{b}=r2'], job(b, '-r2'), 'Preview score rows differ')
    step('layout-missing', {c: 'layout-missing'}, [f'{b}=r3'], job(c), 'Rendered lint report missing')
    step('res-wrong', {d: 'res-wrong'}, [f'{c}=r2'], job(d), WRONG_SIZE)
    step('crf-wrong', {e: 'crf-wrong'}, [f'{d}=r2'], job(e), 'No x264 record of CRF 16')
    step('capture-missing', {f: 'capture-missing'}, [f'{e}=r2'], job(f), 'missing soundtrack capture')
    step('capture-short', {g: 'capture-short'}, [f'{f}=r2'], job(g), 'assert (wav.getframerate(), wav.getnchannels(), wav.getsampwidth(), wav.getnframes())')
    transfer = t.D / 'transfer-files01.json'
    saved = transfer.read_bytes()
    t.faults({h: 'input-changed'})
    code, text = t.go('render', '--retry', f'{g}=r2')
    d_receipt = json.loads((t.J / job(h) / 'resource-receipt.json').read_text())
    transfer.write_bytes(saved)
    check('r-input-changed', code != 0 and 'inputs_sha256' in d_receipt and d_receipt['inputs_sha256'] != d_receipt.get('final_inputs_sha256'), "the real guard's final hash differs, so the job failed")
    (t.OUT / f'{i}.mp4').write_text('keep')
    step('existing-movie', {}, [f'{h}=r2'], job(i), 'Existing render preserved')
    check('r-existing-movie-kept', (t.OUT / f'{i}.mp4').read_text() == 'keep', 'the bytes were not touched')
    code, text = t.go('render', '--retry', f'{i}=r2')
    check('r-all-pass-after-retries', code == 0 and text.strip().splitlines()[-1].startswith('RENDER PASS: 15 chapters'), last(text))
    check('r-retry-moved-planted-movie', text_of(t.ASIDE / job(i) / f'{i}.mp4') == 'keep', 'the planted movie is in the aside folder')


def render_records(t):
    last_job = 'complex-remote-render-15_closing-01'
    jobs_before = sorted(p.name for p in t.J.iterdir())
    code, text = t.go('render')
    check('e-resume-runs-nothing', code == 0 and text.count('DONE ') == 15 and sorted(p.name for p in t.J.iterdir()) == jobs_before, '15 chapters skipped after their hashes held')
    code, text = t.go('render', '--retry', f'{Q.CHAPTERS[-1]}=r2')
    check('e-retry-of-passed-refused', code != 0 and 'there is nothing to retry' in text, last(text))
    probe_results(t, PROBE_GATE, GATE_EXPECTED, str(t.base / 'gate-a.json'), str(t.base / 'gate-b.json'))
    summary = json.loads((t.J / last_job / 'render-summary.json').read_text())
    receipt = json.loads((t.J / last_job / 'resource-receipt.json').read_text())
    check('e-summary', summary['frames'] == FRAMES and summary['fps'] == 60 and summary['soundtrack']['samples'] == FRAMES * 800 and ' crf=16.0' in summary['encoder'] and len(summary['files']) == 4 and summary['narration'] == str(t.n2) and summary['derived_from'] == str(t.N),
          'the x264 record, capture, N2 path and its source are in the summary')
    check('e-receipt', receipt['exit'] == 0 and receipt['policy'] == Q.POLICY and receipt['inputs_sha256'] == receipt['final_inputs_sha256'] and receipt['command'][1:3] == ['-I', '-B'] and receipt['command'][3] == str(t.D / PATHS['render']),
          'the real guard admitted and closed the job, on the logical path')
    rows = t.listed(t.stage_dir / f'{last_job}-inputs.json')
    wanted = [t.D / PATHS['render'], t.D / 'start-final-render01.sh', t.D / 'transfer-files01.json', t.stage_dir / 'preview-pictures-viewed01.json', t.n2 / 'report.json', t.N / 'report.json', t.script,
              t.O / 'guard01.py', t.O / PICTURES_INPUTS, t.J / STAGE_JOB['pictures'] / 'pictures/extraction.json', t.J / BUILD_SUMMARY,
              t.O / 'complex_analysis_visual-release02', *(t.R / row['path'] for row in json.loads((BUNDLE / 'transfer-files01.json').read_text())['files'])]
    missing = [str(p) for p in wanted if str(p) not in rows]
    check('e-manifest', not missing and not [p for p in rows if p.startswith(str(t.physical))], 'review, N, N2, binary, guard, chain, transfers and this helper, by logical paths' + (f' MISSING {missing}' if missing else ''))
    movie_path = t.OUT / f'{Q.CHAPTERS[4]}.mp4'
    flipped(movie_path)
    code, text = t.go('render')
    flipped(movie_path)
    check('e-changed-movie-refused', code != 0 and str(movie_path) in text, last(text))
    log = t.J / f'complex-remote-render-{Q.CHAPTERS[-1]}-01/command.log'
    saved = log.read_bytes()
    check('e-render-last-line', log.read_text().strip().splitlines()[-1].startswith('RENDER PASS: 4K60 chapter'), last(log.read_text()))
    with log.open('a') as handle:
        handle.write('something after\n')
    code, text = t.go('render')
    log.write_bytes(saved)
    check('e-success-line-required', code != 0 and 'No success line' in text, last(text))


def master_failures(t):
    def step(label, fault, retries, name, words):
        t.faults({'master': fault})
        code, text = t.go('master', *[x for retry in retries for x in ('--retry', retry)])
        text += t.joblog(name)
        ok = code != 0 and words in text and not (t.J / name / 'master-summary.json').exists() and (t.J / name).is_dir()
        check('m-' + label, ok, f'{name}: {words[:48]}' if ok else last(text))

    code, text = t.go('verify')
    check('w-no-master', code != 0 and 'Stable complex master required' in text and not list(t.J.glob(VERIFY + '*')) and not list(t.stage_dir.glob(VERIFY + '*')), last(text))
    run = t.mode('master', '--check')
    d = json.loads(run.stdout) if run.returncode == 0 else {}
    check('m-check-prints-growth', d.get('guard_growth_cap_gib') == 4 and d.get('over_cap') is False, f"estimate {d.get('estimated_growth_gib')} GiB on the fixture; decides nothing")
    step('master-exit', 'master-exit', [], MASTER, 'CalledProcessError')
    t.faults()
    code, text = t.go('master')
    check('m-no-retry-without-name', code != 0 and 'pass --retry master=r2' in text and not (t.J / (MASTER + '-r2')).exists(), last(text))
    step('proof-missing', 'proof-missing', ['master=r2'], MASTER + '-r2', 'Missing native soundtrack replay proof')
    step('captions-exit', 'captions-exit', ['master=r3'], MASTER + '-r3', 'CalledProcessError')
    check('m-retry-moved-kit-work', (t.ASIDE / (MASTER + '-r2') / 'master/01.video.mp4').is_file() and (t.ASIDE / (MASTER + '-r2') / 'complex_analysis_visual_master.mp4').is_file(), 'the failed master and the kit folder are preserved')
    step('master-wrong', 'master-wrong', ['master=r4'], MASTER + '-r4', WRONG_SIZE)
    step('srt-empty', 'srt-empty', ['master=r5'], MASTER + '-r5', "assert OUTPUTS['srt'].stat().st_size and OUTPUTS['vtt']")


def tool_control(t):
    """The fake tables pass the real delivery tool's master check; one dropped cue must fail it."""
    folder = t.base / 'control-tables'
    folder.mkdir()
    for path in t.OUT.glob('*.tsv'):
        shutil.copy(path, folder)
    keys = [x for chapter in Q.CHAPTERS for x in ('--chapter-key', chapter)]
    call = [t.python, '-I', '-B', str(t.workspace / 'tools/narration_delivery.py'), '--topic', str(t.C), '--narration', str(t.n2), '--consumer', 'master', '--gain-db', '0',
            *keys, '--timelines', str(folder), '--score', str(t.J / STAGE_JOB['score'] / 'scored/stems')]
    good = subprocess.run(call, cwd=t.R, env=t.safe_env, capture_output=True, text=True)
    path = folder / f'{Q.CHAPTERS[2]}.cuelog.tsv'
    lines = path.read_text().splitlines(keepends=True)
    path.write_text(''.join(lines[:1] + lines[2:]))
    bad = subprocess.run(call, cwd=t.R, env=t.safe_env, capture_output=True, text=True)
    check('m-real-tool-control', good.returncode == 0 and bad.returncode != 0 and 'cue log omits verified narration' in bad.stdout + bad.stderr, last(bad.stderr + bad.stdout))


def master_pass(t):
    name = MASTER + '-r6'
    t.faults()
    code, text = t.go('master', '--retry', 'master=r6')
    log = t.joblog(name)
    check('e-master', code == 0 and (t.J / name / 'master-summary.json').exists(), 'real ffprobe at 3840x2160, 60/1 and 540 frames' if code == 0 else last(text + log))
    check('e-master-last-line', log.strip().splitlines()[-1] == 'MASTER WRITTEN: candidate master and captions; strict verify, pictures, ASR, loudness and listening remain.', last(log))
    s = json.loads((t.J / name / 'master-summary.json').read_text())
    check('e-master-summary', [line.split()[0] for line in s['native_bit_identical_replays']] == list(Q.CHAPTERS) and set(s['outputs']) == {'master', 'srt', 'vtt'} and s['narration'] == str(t.n2)
          and s['full_baseline_equivalence_established'] is False and 'Full decode' in s['scope'], 'all 15 proof lines; master, SRT and VTT hash-bound; no acceptance claimed')
    rows = t.listed(t.stage_dir / f'{name}-inputs.json')
    chapter = Q.CHAPTERS[9]  # one that needed no retry, so its job keeps the first name
    wanted = [t.D / PATHS['render'], t.D / PATHS['master'], t.D / 'start-final-master01.sh', t.OUT / 'soundtrack' / chapter / 'soundtrack_0.wav', t.OUT / f'{chapter}.mp4',
              t.stage_dir / f'complex-remote-render-{chapter}-01-inputs.json', t.stage_dir / 'preview-pictures-viewed01.json']
    missing = [str(p) for p in wanted if str(p) not in rows]
    check('e-master-manifest', not missing and not [p for p in rows if p.startswith(str(t.physical))], 'both helpers, the review and every chapter file, by logical paths' + (f' MISSING {missing}' if missing else ''))
    code, text = t.go('master')
    check('m-duplicate-refused', code != 0 and 'Duplicate master refused' in text, last(text))
    code, text = t.go('master', '--retry', 'master=r7')
    check('m-retry-of-passed-refused', code != 0 and 'there is nothing to retry' in text, last(text))
    return name


def verify_refusals(t, master):
    path = t.J / 'complex-remote-render-work01/complex_analysis_visual_master.mp4'
    flipped(path)
    code, text = t.go('verify')
    flipped(path)
    check('w-master-changed', code != 0 and not list(t.J.glob(VERIFY + '*')), last(text))
    log = t.J / master / 'command.log'
    saved = log.read_bytes()
    with log.open('a') as handle:
        handle.write('something after\n')
    code, text = t.go('verify')
    log.write_bytes(saved)
    check('w-master-success-line', code != 0 and 'No success line' in text and not list(t.J.glob(VERIFY + '*')), last(text))
    run = t.mode('verify', '--check')
    check('w-check-read-only', run.returncode == 0 and not list(t.J.glob(VERIFY + '*')) and not list(t.stage_dir.glob(VERIFY + '*')), last(run.stderr, 'plan holds; nothing written'))


def verify_pass(t, master):
    code, text = t.go('verify')
    out = t.J / VERIFY
    log = t.joblog(VERIFY)
    check('e-verify', code == 0 and (out / 'verification.json').exists(), 'finished on the fixture master' if code == 0 else last(text + log))
    v = json.loads((out / 'verification.json').read_text())
    picture_python = str(t.J / 'complex-remote-picture-env01/env/bin/python')
    receipt = json.loads((out / 'resource-receipt.json').read_text())
    check('e-verify-picture-python', receipt['command'][0] == picture_python and receipt['phase'] == 'native', 'the guard ran the child under the picture env, for Pillow')
    check('e-verify-last-line', log.strip().splitlines()[-1] == 'FINAL VERIFY PASS: strict video check, frame count, A/V length and exact captions passed; every sheet still needs actual viewing.', last(log))
    strict = (out / 'strict-video-verification.log').read_text().splitlines()
    check('e-strict-real', strict[-1] == 'VERIFY PASS' and v['media']['video_frames'] == FRAMES * 15 and abs(float(v['encoded_loudness']['integrated_lufs']) + 16) <= 1, 'real pinned strict check and ebur128 on the fixture master')
    check('e-captions-real', v['captions']['caption_blocks'] == 105 and v['captions']['text_equals_script'] == 'whole and per cue' and v['caption_screen']['caption_count'] == 105, '105 real cue texts, SRT equals VTT, real caption screen ran')
    joined = ' | '.join(reason for p in v['pictures'] for reason in p['reasons'])
    missing = [word for word in ('last frame', ' start', 'authored hold', 'still there at 3 later looks', 'stands 0.20 from the edge') if word not in joined]
    sources = v['layout_sources'][Q.CHAPTERS[0]]
    check('e-samples', not missing and sources[:2] == [str(t.OUT / (Q.CHAPTERS[0] + '.layout.tsv')), str(t.J / STAGE_JOB['preview'] / 'preview' / (Q.CHAPTERS[0] + '.layout.tsv'))],
          f"{len(v['pictures'])} final pictures; the final lint and the preview lint both add samples" + (f' MISSING {missing}' if missing else ''))
    derive = t.J / N.DERIVE / 'derive-summary.json'
    s = json.loads((t.J / master / 'master-summary.json').read_text())
    check('e-provenance', all(d['narration'] == str(t.n2) and d['derived_from'] == str(t.N) and d['derive_job'] == N.DERIVE and d['derive_summary_sha256'] == sha(derive) for d in (v, s)),
          'master and verification name N2, the folder it came from, and the derive summary by hash')
    check('e-no-picture-claim', v['pictures_viewed'] is False and 'does not carry over' in v['scope'] and v['preview_review']['review_scope'] == 'source-author' and v['full_baseline_equivalence_established'] is False and v['voice'] == 'ElevenLabs Frederick Surrey',
          'pictures_viewed is false; the preview review does not carry over')
    rows = t.listed(t.stage_dir / f'{VERIFY}-inputs.json')
    wanted = [t.D / PATHS['render'], t.D / PATHS['master'], t.D / PATHS['verify'], t.D / 'start-final-verify01.sh', t.stage_dir / f'{master}-inputs.json', t.J / master / 'master-summary.json',
              t.J / master / 'resource-receipt.json', t.OUT / 'complex_analysis_visual_master.mp4', t.OUT / 'complex_analysis_visual.en.srt', t.OUT / 'complex_analysis_visual.en.vtt']
    missing = [str(p) for p in wanted if str(p) not in rows]
    check('e-verify-manifest', not missing and not [p for p in rows if p.startswith(str(t.physical))], 'all three helpers, the shared checks, master and captions, by logical paths' + (f' MISSING {missing}' if missing else ''))
    record = t.stage_dir / f'{VERIFY}-postcheck.json'
    r = json.loads(record.read_text())
    check('e-postcheck', list(r) == ['job', 'master_job', 'master_sha256', 'srt_sha256', 'vtt_sha256', 'derived_from', 'derive_summary_sha256', 'verification_sha256', 'frames', 'pictures', 'sheets', 'pictures_viewed', 'scope']
          and r['job'] == VERIFY and r['master_job'] == master and r['master_sha256'] == sha(t.OUT / 'complex_analysis_visual_master.mp4') and r['verification_sha256'] == sha(out / 'verification.json') and r['pictures_viewed'] is False,
          f"{r['pictures']} pictures, {r['sheets']} sheets, written once by the launch")
    saved = record.read_bytes()
    run = t.mode('verify', '--postcheck', VERIFY)
    check('e-postcheck-repeats', run.returncode == 0 and record.read_bytes() == saved, last(run.stderr, 'read-only repeat; identical bytes accepted'))
    for label, path, words in (('sheet', Path(v['sheets'][0]), 'A sheet changed'), ('picture', Path(v['pictures'][0]['path']), 'A picture changed'), ('strict-log', out / 'strict-video-verification.log', 'Strict log differs'),
                               ('success-line', out / 'command.log', 'No success line'), ('master', t.OUT / 'complex_analysis_visual_master.mp4', 'complex_analysis_visual_master.mp4'), ('other-record', record, 'exists with other bytes')):
        data = path.read_bytes()
        path.write_bytes(data + b'\nx\n')
        run = t.mode('verify', '--postcheck', VERIFY)
        path.write_bytes(data)
        check('p-' + label, run.returncode != 0 and words in run.stderr, last(run.stderr))
    for label, file_name, key, value, words in (('claim', 'verification.json', 'pictures_viewed', True, 'A claim changed'), ('provenance', 'verification.json', 'derived_from', 'another folder', 'Narration provenance differs'), ('screen', 'caption-screen.json', 'srt_sha256', '0' * 64, 'Caption screen read another SRT')):
        path = out / file_name
        data = path.read_bytes()
        edited = json.loads(data)
        edited[key] = value
        path.write_text(json.dumps(edited))
        run = t.mode('verify', '--postcheck', VERIFY)
        path.write_bytes(data)
        check('p-' + label, run.returncode != 0 and words in run.stderr, last(run.stderr))
    code, text = t.go('verify')
    check('w-duplicate', code != 0 and 'Duplicate final verification refused' in text, last(text))


def shelve(t, label):
    """Moves a finished case aside, so its names can be used again in this fixture only."""
    shelf = t.base / 'shelf' / label
    shelf.mkdir(parents=True)
    for path in (*t.J.glob(MASTER + '*'), *t.J.glob(VERIFY + '*'), *t.stage_dir.glob(MASTER + '*'), *t.stage_dir.glob(VERIFY + '*'), *(t.OUT / name for name in ('master', 'complex_analysis_visual_master.mp4', 'complex_analysis_visual.en.srt', 'complex_analysis_visual.en.vtt'))):
        if path.exists():
            path.rename(shelf / path.name)


def bad_masters(t):
    shelve(t, 'pass')
    for fault, words in (('caption-text', "the captions' text differs from the script"), ('caption-differs', 'SRT and VTT differ')):
        t.faults({'master': fault})
        code, text = t.go('master')
        assert code == 0, text[-600:]
        code, text = t.go('verify')
        check('w-' + fault, code != 0 and words in text and not list(t.J.glob(VERIFY + '*')), last(text))
        shelve(t, fault)
    t.faults({'master': 'master-quiet'})
    code, text = t.go('master')
    assert code == 0, text[-600:]
    code, text = t.go('verify')
    strict = (t.J / VERIFY / 'strict-video-verification.log').read_text().splitlines()
    check('w-master-quiet', code != 0 and 'CalledProcessError' in t.joblog(VERIFY) and strict[-1] == 'VERIFY FAIL' and not (t.J / VERIFY / 'verification.json').exists(), strict[-1])
    code, text = t.go('verify')
    check('w-no-retry-without-name', code != 0 and 'pass --retry verify=r2' in text and not (t.J / (VERIFY + '-r2')).exists(), last(text))
    code, text = t.go('verify', '--retry', 'verify=r2')
    again = (t.J / (VERIFY + '-r2') / 'strict-video-verification.log').read_text().splitlines()
    check('w-named-retry-runs', code != 0 and again[-1] == 'VERIFY FAIL' and (t.J / VERIFY / 'strict-video-verification.log').exists(), 'the second attempt ran in its own folder; the first is preserved')
    shelve(t, 'quiet')


def start_scripts(t):
    for name in HELPERS:
        script = t.D / f'start-final-{name}01.sh'
        before = sorted(p.name for p in t.stage_dir.iterdir())
        run = subprocess.run(['bash', str(script)], env=t.env, capture_output=True, text=True)
        wrong = subprocess.run(['bash', str(script), '1'], env=t.env, capture_output=True, text=True)
        check(f's-{name}-usage', run.returncode == wrong.returncode == 2 and 'usage' in run.stderr and sorted(p.name for p in t.stage_dir.iterdir()) == before, 'a run number is required; nothing was written')
        log = t.stage_dir / f'final-{name}-controller01.log'
        exit_file = t.stage_dir / f'final-{name}-controller01.exit'
        saved = log.read_bytes(), exit_file.read_bytes()
        run = subprocess.run(['bash', str(script), '01'], env=t.env, capture_output=True, text=True)
        check(f's-{name}-repeat', run.returncode == 1 and 'refused' in run.stderr and (log.read_bytes(), exit_file.read_bytes()) == saved, "a repeated run number leaves that run's log and exit file alone")


def main():
    static_checks()
    root = Path(tempfile.mkdtemp(prefix='final-')).resolve()
    Q.FIXTURE[0] = str(root)
    repin_checks(root)
    shared = root / 'shared'
    shared.mkdir()
    movie(shared / 'tiny.mp4', FRAMES, '960x540', crf=18)
    base = root / 'tree'
    base.mkdir()
    t = Final(base.resolve(), 'baseline', shared)
    t.finish()
    t.bundle()
    run = t.mode('render', '--check')
    check('c-check-before-chain', run.returncode == 0 and '"launched": false' in run.stdout, last(run.stderr, 'the read-only check needs no finished stage'))
    run = t.mode('verify', '--pins')
    check('c-pins-before-chain', run.returncode == 0 and '"launched": false' in run.stdout, last(run.stderr, 'the read-only pins need no finished stage'))
    t.qualify()
    run = t.stage('timeline')
    check('f-sealed-refusal', run.returncode != 0, 'chain01 stops as on ts1, so the next chain may begin')
    for stage in ('derive', 'timeline', 'score', 'preview', 'pictures'):
        run = t.start2(stage)
        job = N.DERIVE if stage == 'derive' else STAGE_JOB[stage]
        d = json.loads((t.J / job / 'resource-receipt.json').read_text())
        check('f-stage-' + stage, run.returncode == 0 and d['exit'] == 0, last(t.text2(stage)) if run.returncode else 'real guard, sealed chain, real tools')
    movie(t.O / 'fake-final.mp4', FRAMES)
    movie(t.O / 'fake-crf23.mp4', FRAMES, crf=23)
    movie(t.O / 'fake-master.mp4', FRAMES * 15)
    movie(t.O / 'fake-quiet.mp4', FRAMES * 15, level=-30)
    t.captions()
    real_controls(t)
    before_review(t)
    t.review()
    run = t.mode('render', '--check-review')
    check('c-review-binds', run.returncode == 0, last(run.stderr, 'a complete hash-bound review passes'))
    probe_results(t, PROBE_REVIEW, REVIEW_EXPECTED)
    render_failures(t)
    render_records(t)
    master_failures(t)
    tool_control(t)
    master = master_pass(t)
    verify_refusals(t, master)
    verify_pass(t, master)
    bad_masters(t)
    start_scripts(t)
    real = REAL / Q.EC / 'remote-ts1-01'
    check('z-repository-clean', not (real / 'jobs').exists() and not (real / 'final-production01').exists(), 'no job or stage folder appeared on the local lane')
    print(f"RESULT {'PASS' if all(C.results) else 'FAIL'}, {C.results.count(False)} failed of {len(C.results)}")
    return 0 if all(C.results) else 1


if __name__ == '__main__':
    sys.exit(main())
