"""Focused checks: both takes through the real helper and wrapper, then the real failures.

Usage, from the repository root, with a fresh scratch path: python3 -I -B CHECKS/run_checks.py SCRATCH

Root's real records and both real WAVs run in a tiny scratch repository at their real depth. Stand-ins: the guard, fw, the
model, sbh, the bindings and the tools file. Nothing real is launched and no recognition runs: the stand-in fw prints a planted
result. A copy of mini_take_asr01.py differs from the delivered file only in its two pin literals, because the fixture's
bindings and tools files are stand-ins.
"""
import ast
import hashlib
import json
from pathlib import Path
import re
import shutil
import subprocess
import sys
import wave

HERE = Path(__file__).resolve().parents[1]
R = HERE.parents[4]
REL_P = 'docs/video-program/third-wave-20261024/opus-replacements-20261104'
REL_LANE = 'narration_lab/third-wave-20261024/root/elevenlabs-logarithm-rebuild-20261104/remote-mac-mini-max01'
REL_TAKES = 'narration_lab/third-wave-20261024/root/elevenlabs-complex-rebuild-20261104/remote-ts1-01/cauchy-cue-repair01'
ASR02_SEAL = 'f63501eae4cd2097b062c16ca6c2ace0f87e1b1daabf4e5b0e77a4d8a0528189'
INTAKE_SEAL = '48ae1dff4e230a164c21bcaac46a3cea7c864b4c5f705a6fb84c53104885565e'
WHITE_SEAL = 'eed378da77060ca922f0d4746b45ee4b43de0515c0e5d6072cee01b8387ce359'
MAIL_5610 = {'09_5-take01.wav': ('f7c07ffafb2b0becbae44d0a879c7f59b3dd1f404a67643ae8e8bbd1aae80dd7', 864000),
             '09_5-take02.wav': ('16f16da361e30e1f91c2dc82eb4807d27a70f6cdd740b15b276a354ceb63b06c', 875520)}
TEXT_SHA256 = 'c84c83bd91b69948de77e1d21089e71c70abe71fcde6c49510c81568be104bcb'  # White's sealed bindings.json
TAKE_FILES = ('09_5-take01.wav', '09_5-take02.wav', 'request01.json', 'submission01.json', 'download-completion01.json')

GUARD = r'''"""Stand-in for the lane's guard01.py: the real guard's arguments and receipt keys, with no resource policy."""
import hashlib, json, os, shutil, subprocess, sys
from pathlib import Path
O = Path(__file__).resolve().parent
fixture = O.parents[5]
sha = lambda p: hashlib.sha256(Path(p).read_bytes()).hexdigest()
a = sys.argv[1:]
cut = a.index('--') if '--' in a else len(a)
flags, command = a[:cut], a[cut + 1:]
preflight = '--preflight-only' in flags
with (fixture / 'events.jsonl').open('a') as h:
    h.write(json.dumps({'kind': 'preflight' if preflight else 'run', 'flags': flags, 'command': command, 'cwd': os.getcwd()}) + '\n')
job = O / 'jobs' / flags[flags.index('--job') + 1]
job.mkdir(parents=True)
r = {'phase': flags[flags.index('--phase') + 1], 'command': command, 'preflight_only': preflight,
     'admitted': not (fixture / 'guard-refuses').exists()}
save = lambda: (job / 'resource-receipt.json').write_text(json.dumps(r))
if preflight or not r['admitted']:
    r['finished'] = 'stand-in'
    save()
    sys.exit(0 if r['admitted'] else 75)
manifest = Path(flags[flags.index('--inputs') + 1])
for row in json.loads(manifest.read_text())['files']:
    assert sha(row['path']) == row['sha256'], row['path']
r['inputs_sha256'] = sha(manifest)
save()
with (job / 'command.log').open('x') as log:
    code = subprocess.run(command, cwd=O.parents[4], stdout=log, stderr=subprocess.STDOUT).returncode
r.update(exit=code, final_inputs_sha256=sha(manifest), finished='stand-in')
save()
sys.exit(code)
'''
FW = f'''#!{sys.executable} -I
import json, sys
from pathlib import Path
here = Path(__file__).resolve().parent
with (here / 'fw.argv').open('a') as h:
    h.write(json.dumps(sys.argv[1:]) + '\\n')
take = Path(sys.argv[sys.argv.index('--input') + 1]).name
plan = json.loads((here / 'fw.plan.json').read_text())[take]
if plan == 'fail':
    sys.exit(3)
print(json.dumps({{'request': {{'backend': 'whisper_cpp'}}, 'result': plan}}))
'''


def sha(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def write(path, text, mode=0o644):
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(text)
    path.chmod(mode)
    return path


def js(path, data):
    return write(path, json.dumps(data, indent=2) + '\n')


def row(path):
    return {'path': str(path), 'sha256': sha(path)}


def check(ok, label):
    if not ok:
        raise SystemExit(f'FAIL {label}')


def spoken(text):
    words = text.split()
    return {'segments': [{'start_sec': n * 0.3, 'end_sec': n * 0.3 + 0.25, 'text': word, 'speaker': None, 'confidence': 0.5} for n, word in enumerate(words)],
            'transcript': text, 'backend': 'whisper_cpp', 'raw_output': {'model_version_tag': 'stand-in', 'word_timestamps': 'dtw'}}


def repin(fx):
    path = fx.bundle / 'mini_take_asr01.py'
    lines = path.read_text().splitlines(True)
    want = {'SOURCES_SHA256 = ': sha(fx.lane / 'asr-source-bindings01.json'), 'TOOLS_SHA256 = ': sha(fx.lane / 'tools01.json')}
    path.write_text(''.join(next((f"{k}'{h}'\n" for k, h in want.items() if line.startswith(k)), line) for line in lines))


def build(base):
    """One complete good fixture: the real records and takes, and stand-in tools behind root's two files."""
    fx = type('Fixture', (), {})()
    fx.base, fx.repo, fx.home = base, base / 'repo', base / 'home'
    fx.bundle = fx.repo / REL_P / 'complex-cauchy-take-asr01'
    fx.lane, fx.takes = fx.repo / REL_LANE, fx.repo / REL_TAKES
    fx.stand = fx.lane / 'stand-ins'
    for name in ('mini_take_asr01.py', 'start-take-asr01.sh'):
        write(fx.bundle / name, (HERE / name).read_text(), 0o755)
    fx.takes.mkdir(parents=True)
    for name in TAKE_FILES:
        shutil.copyfile(R / REL_TAKES / name, fx.takes / name)
    write(fx.home / '.local/bin/python3', f'#!/bin/sh\nexec {sys.executable} "$@"\n', 0o755)
    fx.sbh = write(fx.home / '.local/bin/sbh', '#!/bin/sh\n', 0o755)
    fx.fw, fx.model = write(fx.stand / 'fw', FW, 0o755), write(fx.stand / 'model.bin', 'model\n')
    ffmpeg, ffprobe = write(fx.stand / 'ffmpeg', '#!/bin/sh\n', 0o755), write(fx.stand / 'ffprobe', '#!/bin/sh\n', 0o755)
    js(fx.lane / 'tools01.json', {'python3': {'path': sys.executable, 'sha256': sha(sys.executable)}, 'ffmpeg': row(ffmpeg), 'ffprobe': row(ffprobe)})
    js(fx.lane / 'asr-source-bindings01.json', {'files': [row(write(fx.stand / 'logarithm-script.tsv', 'x\n')), row(fx.fw), row(fx.model), row(ffmpeg)]})
    js(fx.lane / 'guard-pins01.json', {'files': [row(write(fx.stand / 'pin1', 'a\n'))]})
    write(fx.lane / 'guard01.py', GUARD)
    repin(fx)
    text = json.loads((fx.takes / 'request01.json').read_text())['text']
    fx.text = text
    misheard = text.replace('minus i', '−i').replace('throughout the disk', 'throughout disk')
    js(fx.stand / 'fw.plan.json', {'09_5-take01.wav': spoken(text), '09_5-take02.wav': spoken(misheard)})
    fx.job = fx.lane / 'jobs/complex-cauchy-take-asr01'
    return fx


def start(fx, *args):
    """The wrapper as root starts it, with only HOME in its environment."""
    args = args or ('01',)
    done = subprocess.run(['/bin/bash', str(fx.bundle / 'start-take-asr01.sh'), *args], env={'HOME': str(fx.home)}, capture_output=True, text=True, timeout=120)
    log = fx.lane / f'complex-cauchy-take-asr{args[0]}.log'
    return done.returncode, done.stdout + done.stderr + (log.read_text() if log.exists() else '')


def direct(fx, *args):
    done = subprocess.run([sys.executable, '-I', '-B', str(fx.bundle / 'mini_take_asr01.py'), *args], capture_output=True, text=True, timeout=60,
                          env={'HOME': str(fx.home), 'PATH': '/usr/bin:/bin'})
    return done.returncode, done.stdout + done.stderr


def events(fx):
    path = fx.base / 'events.jsonl'
    return [json.loads(x) for x in path.read_text().splitlines()] if path.exists() else []


def said(text, fx):
    last = [x for x in text.splitlines() if x.strip()][-1]
    return last.replace(str(fx.base), 'SCRATCH')


def no_job(fx, name):
    check(not fx.job.exists() and not (fx.stand / 'fw.argv').exists() and not (fx.lane / 'complex-cauchy-take-asr-inputs01.json').exists(), f'{name}: a job name was used')


def expected_fw(fx, take):
    return ['transcribe', '--input', str(fx.takes / take), '--backend', 'whisper-cpp', '--model', str(fx.model), '--language', 'en', '--no-diarize',
            '--no-persist', '--split-on-word', '--max-segment-length', '1', '--threads', '4', '--timeout', '180', '--json']


def good(base):
    fx = build(base)
    code, text = start(fx)
    check(code == 0, 'good run: ' + text[-500:])
    check((fx.lane / 'complex-cauchy-take-asr01.exit').read_text().strip() == '0', 'exit file')
    seen = events(fx)
    child = [sys.executable, '-I', '-B', str(fx.bundle.resolve() / 'mini_take_asr01.py'), '--run', '01']
    check([x['kind'] for x in seen] == ['preflight', 'run'] and all('--phase' in x['flags'] and x['flags'][x['flags'].index('--phase') + 1] == 'asr' for x in seen), 'guard phases')
    check(seen[1]['command'] == child and Path(seen[1]['cwd']).resolve() == fx.repo.resolve(), 'guard child and cwd')
    calls = [json.loads(x) for x in (fx.stand / 'fw.argv').read_text().splitlines()]
    check(calls == [expected_fw(fx, name) for name in TAKE_FILES[:2]], 'fw arguments')
    frozen = {row_['path'] for row_ in json.loads((fx.lane / 'complex-cauchy-take-asr-inputs01.json').read_text())['files']}
    want = {str(fx.takes / name) for name in TAKE_FILES} | {str(fx.fw), str(fx.model), str(fx.bundle / 'mini_take_asr01.py'), str(fx.bundle / 'start-take-asr01.sh'), str(fx.lane / 'guard01.py')}
    check(want <= {str(Path(p)) for p in frozen}, f'manifest misses {sorted(want - frozen)[:2]}')
    result = json.loads((fx.job / 'take-comparison.json').read_text())
    check(result['selected_take'] is None and result['listening'] is False and result['fine_sync'] is False, 'a claim appeared')
    check(result['text_sha256'] == TEXT_SHA256, 'text hash')
    first, second = result['takes']
    check(first['differences'] == [] and all(x['matched_words'] == x['expected_words'] for x in first['sentences']) and len(first['sentences']) == 4, 'clean take')
    check(all(w['edge'] and w['recognized_segments'] for w in first['cue_edges']['windows']) and len(first['cue_edges']['windows']) == 2, 'cue edges')
    changed = [(d['operation'], [x['word'] for x in d['expected']], [x['word'] for x in d['recognized']]) for d in second['differences']]
    check(('replace', ['minus'], ['-']) in changed and ('delete', ['the'], []) in changed, f'planted differences {changed}')
    check([x['expected_words'] - x['matched_words'] for x in second['sentences']] == [0, 1, 0, 1], 'sentence rollup')
    check(all(sha(fx.takes / name) == sha(R / REL_TAKES / name) for name in TAKE_FILES), 'a record or take changed')
    lines = [x for x in text.splitlines() if x.strip()]
    check(lines[-1] == f'TAKE ASR PASS: complex-cauchy-take-asr01 take-comparison.json sha256 {sha(fx.job / "take-comparison.json")}; no take is selected.', 'last line')
    return 'both takes run under one guarded job with exact fw arguments; a real minus sign and a dropped word reach the differences and the sentence rollup; no take is selected'


def changed(path):
    with path.open('ab') as handle:
        handle.write(b'\0')


def aside(path):
    path.rename(path.with_name(path.name + '.aside'))


def rebind(fx, name, index, value):
    path = fx.lane / 'asr-source-bindings01.json'
    d = json.loads(path.read_text())
    d['files'][index][name] = value
    js(path, d)
    repin(fx)


def no_sbh(fx):
    aside(fx.sbh)
    path = fx.bundle / 'start-take-asr01.sh'
    text = path.read_text()
    assert text.count(':/opt/homebrew/bin') == 1
    path.write_text(text.replace(':/opt/homebrew/bin', ''))  # a host sbh must not hide this refusal


CASES = [
    ('take01 changed in transfer', lambda f: changed(f.takes / '09_5-take01.wav'), 'changed in transfer'),
    ('take02 not transferred', lambda f: aside(f.takes / '09_5-take02.wav'), 'not transferred: 09_5-take02.wav'),
    ('request changed', lambda f: changed(f.takes / 'request01.json'), "not root's record: request01.json"),
    ('submission changed', lambda f: changed(f.takes / 'submission01.json'), "not root's record: submission01.json"),
    ('completion record changed', lambda f: changed(f.takes / 'download-completion01.json'), "not root's record: download-completion01.json"),
    ('completion record not transferred', lambda f: aside(f.takes / 'download-completion01.json'), 'not transferred: download-completion01.json'),
    ('root bindings changed', lambda f: changed(f.lane / 'asr-source-bindings01.json'), "not root's verified ASR bindings"),
    ('worker tools changed', lambda f: js(f.lane / 'tools01.json', {**json.loads((f.lane / 'tools01.json').read_text()), 'extra': 1}), 'not the worker tools file'),
    ('bindings ffmpeg is not the pinned tool', lambda f: rebind(f, 'sha256', 3, '0' * 64), 'ASR ffmpeg is not the pinned tool'),
    ('fw bytes changed', lambda f: changed(f.fw), 'stand-ins/fw'),
    ('model bytes changed', lambda f: changed(f.model), 'stand-ins/model.bin'),
    ('sbh off PATH', no_sbh, 'sbh is not on PATH'),
]


def refusals(base):
    for n, (name, plant, words) in enumerate(CASES):
        fx = build(base / f'case{n:02d}')
        plant(fx)
        code, text = start(fx)
        check(code != 0 and words in text, f'{name}: {text[-400:]}')
        no_job(fx, name)
        print(f'PASS refuse {name}: {said(text, fx)}')
    fx = build(base / 'interpreter')
    path = fx.lane / 'tools01.json'
    d = json.loads(path.read_text())
    d['python3']['sha256'] = '0' * 64
    js(path, d)
    repin(fx)
    code, text = direct(fx, '--check', '01')
    check(code != 0 and 'pinned interpreter changed' in text, 'interpreter bytes')
    d['python3']['path'] = '/usr/bin/true'
    js(path, d)
    repin(fx)
    code, text = direct(fx, '--check', '01')
    check(code != 0 and 'not the pinned interpreter' in text, 'interpreter path')
    no_job(fx, 'interpreter')
    print('PASS refuse another interpreter, or changed interpreter bytes')
    fx = build(base / 'names')
    for args in (('00',), ('1',), ('abc',)):
        code, text = start(fx, *args)
        check(code != 0 and 'bad attempt' in text, f'bad attempt {args}')
    code, text = direct(fx, '--check', '00')
    check(code != 0 and 'bad attempt' in text, 'direct bad attempt')
    (fx.lane / 'jobs/complex-cauchy-take-asr01').mkdir(parents=True)
    code, text = start(fx)
    check(code == 2 and 'use the next attempt number' in text, 'used attempt')
    print('PASS refuse bad attempts and a used attempt')
    fx = build(base / 'guard')
    write(fx.base / 'guard-refuses', '')
    code, text = start(fx)
    check(code == 75 and not fx.job.exists() and not (fx.stand / 'fw.argv').exists(), f'guard refusal: {code}')
    print('PASS refuse a guard refusal at preflight leaves the job name unused')
    fx = build(base / 'run')
    code, text = direct(fx, '--run', '01')
    check(code != 0 and 'FileNotFoundError' in text and not (fx.stand / 'fw.argv').exists(), '--run outside the guard')
    live = fx.job / 'resource-receipt.json'
    js(live, {'admitted': True, 'phase': 'asr', 'inputs_sha256': 'x', 'command': []})
    manifest = js(fx.lane / 'complex-cauchy-take-asr-inputs01.json', {'files': []})
    code, text = direct(fx, '--run', '01')
    check(code != 0 and 'not this guarded ASR job' in text and not (fx.stand / 'fw.argv').exists(), '--run with a foreign receipt')
    js(live, {'admitted': True, 'phase': 'asr', 'inputs_sha256': sha(manifest), 'command': ['another']})
    code, text = direct(fx, '--run', '01')
    check(code != 0 and 'the guard ran another command' in text and not (fx.stand / 'fw.argv').exists(), '--run with another command')
    print('PASS refuse --run outside the guard, under a foreign receipt or under another command, before fw')
    fx = build(base / 'fail')
    path = fx.stand / 'fw.plan.json'
    d = json.loads(path.read_text())
    d['09_5-take02.wav'] = 'fail'
    js(path, d)
    code, text = start(fx)
    check(code != 0 and (fx.lane / 'complex-cauchy-take-asr01.exit').read_text().strip() != '0', 'fw failure accepted')
    check((fx.job / 'take01-full-large.json').stat().st_size and not (fx.job / 'take-comparison.json').exists(), 'fw failure outputs')
    print('PASS fail fw failing on take02 fails the job, keeps take01 and writes no comparison')
    fx = build(base / 'silent')
    path = fx.stand / 'fw.plan.json'
    d = json.loads(path.read_text())
    d['09_5-take01.wav']['segments'] = []
    js(path, d)
    code, text = start(fx)
    check(code != 0 and 'No recognized speech' in (fx.job / 'command.log').read_text() and not (fx.job / 'take-comparison.json').exists(), 'silence accepted')
    print('PASS fail no recognized speech fails the job and writes no comparison')


def nodes(path):
    return {n.name: n for n in ast.parse(path.read_text()).body if isinstance(n, ast.FunctionDef)}


def statement(function, prefix):
    found = [ast.unparse(n) for n in ast.walk(function) if isinstance(n, (ast.Assign, ast.Expr)) and ast.unparse(n).startswith(prefix)]
    assert len(found) == 1, prefix
    return found[0]


def sources():
    p = R / REL_P
    check(sha(p / 'logarithm-remote-asr02/SHA256SUMS') == ASR02_SEAL, 'ASR02 seal')
    check(sha(p / 'complex-frederick-stage06-reuse-root01/SHA256SUMS') == INTAKE_SEAL, 'intake seal')
    check(sha(p / 'complex-cauchy-wording-repair01/SHA256SUMS') == WHITE_SEAL, "White's seal")
    mine, old = nodes(HERE / 'mini_take_asr01.py'), nodes(p / 'logarithm-remote-asr02/asr02.py')
    kept = ('read', 'sha', 'check_files', 'tokens', 'boundary_review', 'live_receipt')
    check(all(ast.dump(old[n]) == ast.dump(mine[n]) for n in kept), 'kept functions against ASR02')
    print(f'PASS keep {", ".join(kept)} equal sealed asr02.py by AST, so the token rule and cue-edge windows are unchanged')
    command = statement(old['run'], 'command = [fw,')
    check(statement(mine['run'], 'command = [fw,') == command.replace("'7200'", "'180'") and "'7200'" in command, 'fw command')
    check(statement(mine['compare'], 'changes.append(') == statement(old['run'], 'changes.append('), 'difference record')
    print('PASS keep the fw command is asr02.py\'s with one change, --timeout 180; each difference record is asr02.py\'s')
    check(ast.dump(nodes(HERE.parent / 'complex-remote-asr01/mini_asr01.py')['bound_tools']) == ast.dump(mine['bound_tools']), 'bound_tools')
    intake = nodes(p / 'complex-frederick-stage06-reuse-root01/frederick_intake_stage06.py')
    check(ast.dump(intake['sentences']) == ast.dump(mine['sentences']), 'sentences')
    print('PASS keep bound_tools equals the complex ASR helper (unsealed), and sentences equals the sealed stage06 intake, by AST')
    pins = {}
    for n in ast.parse((HERE / 'mini_take_asr01.py').read_text()).body:
        if isinstance(n, ast.Assign) and n.targets[0].id in ('RECORDS', 'SOURCES_SHA256', 'TOOLS_SHA256', 'TAKE_NAMES'):
            pins[n.targets[0].id] = ast.literal_eval(n.value)
    takes = R / REL_TAKES
    check(all(sha(takes / name) == digest for name, digest in pins['RECORDS'].items()), 'record pins')
    check(sha(R / REL_LANE / 'asr-source-bindings01.json') == pins['SOURCES_SHA256'] and sha(R / REL_LANE / 'tools01.json') == pins['TOOLS_SHA256'], 'tool pins')
    done = json.loads((takes / 'download-completion01.json').read_text())
    check([Path(x['path']).name for x in done['takes']] == pins['TAKE_NAMES'] == list(MAIL_5610), 'take names')
    for row_ in done['takes']:
        name = Path(row_['path']).name
        with wave.open(str(takes / name)) as w:
            shape = (w.getframerate(), w.getnchannels(), w.getsampwidth(), w.getnframes())
        check(sha(takes / name) == row_['sha256'] == MAIL_5610[name][0] and shape == (48000, 1, 2, MAIL_5610[name][1]), name)
    print('PASS pin root\'s records, both takes and the Mail 5610 hashes and lengths agree on this host\'s copies')
    text = json.loads((takes / 'request01.json').read_text())['text']
    staged = (p / 'complex-cauchy-wording-repair01/staged/explainer_corpus/complex_analysis_visual/script.tsv').read_text()
    rows = dict(x.split('\t', 1) for x in staged.splitlines() if x.strip() and not x.startswith('#'))
    bound = json.loads((p / 'complex-cauchy-wording-repair01/bindings.json').read_text())
    check(rows['09_5'] == text and hashlib.sha256(text.encode()).hexdigest() == TEXT_SHA256 == bound['new_cue_sha256'], 'cue text')
    print(f'PASS pin the request\'s text is White\'s sealed 09_5 row, sha256 {TEXT_SHA256[:12]}')
    scope = {'re': re}
    exec(ast.unparse(mine['sentences']) + '\n' + ast.unparse(mine['tokens']), scope)
    pieces = scope['sentences'](text)
    check(len(pieces) == 4 and [t for s in pieces for t in scope['tokens'](s)] == scope['tokens'](text) and 'minus' in scope['tokens'](text), 'sentences and tokens')
    lost = set(re.sub(r"[a-z]+(?:'[a-z]+)?|\d+(?:\.\d+)?|[-+/%^π]", '', text.lower()).replace(' ', ''))
    check(lost == {'.', ','}, f'token rule loses {lost}')
    print(f'PASS sign four sentences, {len(scope["tokens"](text))} words; the token rule loses only . and , from this cue, so no mathematical sign')
    check(subprocess.run(['bash', '-n', str(HERE / 'start-take-asr01.sh')]).returncode == 0, 'bash -n')
    print('PASS sh   bash -n, the wrapper')


def main():
    base = Path(sys.argv[1]).resolve()
    base.mkdir()
    sources()
    print('PASS good ' + good(base / 'good'))
    refusals(base)
    print('ALL PASS. Stand-ins only: nothing ran on mac-mini-max, and fw transcribe, the real guard and the model did not run.')


if __name__ == '__main__':
    main()
