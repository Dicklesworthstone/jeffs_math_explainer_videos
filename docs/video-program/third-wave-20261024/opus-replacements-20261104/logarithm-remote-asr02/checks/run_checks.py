"""Focused checks: one good proof, then the real failures from the sealed review.

Usage, from the repository root, with a fresh scratch path: python3 -I -B CHECKS/run_checks.py SCRATCH

The delivered asr02.py and start_asr02.sh run as real subprocesses, copied into a tiny scratch repository at their real
depth. Stand-ins: guard, verifier, ffmpeg, fw, the tools file and every record. Nothing real is launched, no media
exists and no ASR runs. A copy differs from the delivered file in two places only: the wrapper's verifier pin names the
stand-in, and the one retired-master case rewrites that literal, since no file can be made with the retired hash.
"""
import ast
import difflib
import hashlib
import json
from pathlib import Path
import subprocess
import sys
import time

HERE = Path(__file__).resolve().parents[1]
R = HERE.parents[4]
REL_P = 'docs/video-program/third-wave-20261024/opus-replacements-20261104'
REL_O = 'narration_lab/third-wave-20261024/root/elevenlabs-logarithm-rebuild-20261104/remote-mac-mini-max01'
PIN = '361fe7334b1a3c332e4d7b19a8bba01457e1bf85f62459dc407474069d12c20d'
RETIRED = '00236d1a6915ff968235a30ab59769b02125ece0fee482e4fa3154c265ac7c24'
V, M = 'logarithm-remote-verify01', 'logarithm-remote-master01'
sys.path.insert(0, str(HERE))
import asr02  # noqa: E402  constants only; select() is never called here

GUARD = r'''"""Stand-in for O/guard01.py: the real guard's arguments and receipt keys, with no resource policy."""
import hashlib, json, os, shutil, subprocess, sys
from pathlib import Path
O = Path(__file__).resolve().parent
sha = lambda p: hashlib.sha256(Path(p).read_bytes()).hexdigest()
a = sys.argv[1:]
cut = a.index('--') if '--' in a else len(a)
flags, command = a[:cut], a[cut + 1:]
preflight = '--preflight-only' in flags
with (O.parents[5] / 'events.jsonl').open('a') as h:
    h.write(json.dumps({'kind': 'preflight' if preflight else 'run', 'flags': flags, 'command': command, 'cwd': os.getcwd(),
                        'path': os.environ['PATH'], 'sbh': shutil.which('sbh')}) + '\n')
job = O / 'jobs' / flags[flags.index('--job') + 1]
job.mkdir()
r = {'phase': flags[flags.index('--phase') + 1], 'command': command, 'preflight_only': preflight}
save = lambda: (job / 'resource-receipt.json').write_text(json.dumps(r))
r['admitted'] = not (O.parents[5] / 'guard-refuses').exists()
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
VERIFIER = r'''"""Stand-in for verify_remote_final.py verify-job: the real key names, from the proof on disk."""
import hashlib, json, os, sys
from pathlib import Path
fixture = Path(__file__).resolve().parents[6]
a = sys.argv[1:]
with (fixture / 'events.jsonl').open('a') as h:
    h.write(json.dumps({'kind': 'verifier', 'flags': a, 'cwd': os.getcwd()}) + '\n')
if (fixture / 'verifier-refuses').exists():
    print('REFUSED: the stand-in verifier refuses')
    sys.exit(1)
job = 'logarithm-remote-verify' + a[a.index('--verify') + 1]
path = Path(a[a.index('--lane') + 1]) / job / 'verification.json'
v = json.loads(path.read_text())
print(json.dumps({'verify_job': 'PASS', 'job': job, 'verification_sha256': hashlib.sha256(path.read_bytes()).hexdigest(),
                  'master_sha256': v['sha256'], 'frames': v['frames'], 'pictures_viewed': False}, indent=2))
'''
FFMPEG = '#!/bin/sh\nprintf \'%s\\n\' "$@" > "$(dirname "$0")/ffmpeg.argv"\nfor last; do :; done\nprintf RIFF > "$last"\n'
FW = ('#!/bin/sh\nprintf \'%s\\n\' "$@" > "$(dirname "$0")/fw.argv"\n'
      'printf \'{"result":{"segments":[{"start_sec":0.0,"end_sec":1.0,"text":"word one"}]}}\\n\'\n')


def sha(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def write(path, text, mode=0o644):
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(text)
    path.chmod(mode)
    return path


def js(path, data):
    return write(path, json.dumps(data, indent=2) + '\n')


def edit(path, change):
    data = json.loads(path.read_text())
    change(data)
    js(path, data)


def row(path):
    return {'path': str(path), 'sha256': sha(path)}


def build(base):
    """A complete good film under base, built whole for each case: every record holds its own absolute paths."""
    fx = type('Fixture', (), {})()
    fx.base, fx.repo = base, base / 'repo'
    fx.O, fx.bundle = fx.repo / REL_O, fx.repo / REL_P / 'logarithm-remote-asr02'
    fx.jobs, fx.stand = fx.O / 'jobs', fx.O / 'stand-ins'
    write(base / 'home/.local/bin/sbh', '#!/bin/sh\n', 0o755)
    ffmpeg, fw = write(fx.stand / 'ffmpeg', FFMPEG, 0o755), write(fx.stand / 'fw', FW, 0o755)
    ffprobe, model = write(fx.stand / 'ffprobe', '#!/bin/sh\n', 0o755), write(fx.stand / 'model.bin', 'model\n')
    fx.tools = {'schema': 'logarithm-remote-tools-v1', 'host': 'mac-mini-max',
                'python3': {'path': sys.executable, 'sha256': sha(sys.executable), 'version': 'stand-in', 'pillow': 'stand-in'},
                'ffmpeg': {**row(ffmpeg), 'version': 'stand-in'}, 'ffprobe': {**row(ffprobe), 'version': 'stand-in'}}
    js(fx.O / 'tools01.json', fx.tools)
    fx.cues = [f'{i % 13:02d}_{i}' for i in range(80)]
    script = write(fx.stand / 'script.tsv', ''.join(f'{cue}\tword number {i}\n' for i, cue in enumerate(fx.cues)))
    js(fx.O / 'asr-source-bindings01.json', {'files': [row(script), row(fw), row(model), row(ffmpeg)]})
    js(fx.O / 'guard-pins01.json', {'files': [row(write(fx.stand / 'pin1', 'a\n')), row(write(fx.stand / 'pin2', 'b\n'))]})
    write(fx.O / 'guard01.py', GUARD)
    verifier = write(fx.repo / REL_P / 'logarithm-remote-final-verifier02/verify_remote_final.py', VERIFIER)
    write(fx.bundle / 'asr02.py', (HERE / 'asr02.py').read_text())
    wrapper = (HERE / 'start_asr02.sh').read_text()
    assert wrapper.count(PIN) == 1
    write(fx.bundle / 'start_asr02.sh', wrapper.replace(PIN, sha(verifier)), 0o755)
    fx.names = [f'{i:02d}_chapter' for i in range(13)]
    folder = fx.jobs / 'logarithm-remote-timeline01'
    frames = [3692] * 12 + [3696]
    fx.chapters = [{'chapter': c, 'frames': n, 'timeline_sha256': sha(write(folder / f'timeline-output/{c}.timeline.tsv', c + '\n'))}
                   for c, n in zip(fx.names, frames)]
    js(folder / 'timeline-summary.json', {'total_frames': 48000, 'fps': 60, 'chapters': fx.chapters})
    film(fx, V, M)
    return fx


def job(fx, name, scripts, success):
    script = write(scripts / f'{name}.py', f'# stand-in {name}\n')
    inputs = js(scripts / f'{name}-inputs.json', {'files': [row(script)]})
    folder = fx.jobs / name
    js(folder / 'resource-receipt.json', {'finished': 'stand-in', 'admitted': True, 'exit': 0, 'phase': 'native',
                                          'command': [sys.executable, '-I', '-B', str(script)],
                                          'inputs_sha256': sha(inputs), 'final_inputs_sha256': sha(inputs)})
    write(folder / 'command.log', f'work\n{success}\n')
    return folder


def film(fx, verify, master):
    work = fx.jobs / 'logarithm-remote-render-work01'
    movie = write(work / 'logarithm_mysteries_master.mp4', f'stand-in master of {master}\n')
    srt, vtt = write(work / 'logarithm_mysteries.en.srt', 'srt\n'), write(work / 'logarithm_mysteries.en.vtt', 'vtt\n')
    tools = sha(fx.O / 'tools01.json')
    js(job(fx, master, fx.O / 'render-jobs01', asr02.MASTER_SUCCESS) / 'master-summary.json',
       {'master': row(movie), 'srt': row(srt), 'vtt': row(vtt), 'tools_sha256': tools, 'binding_sha256': 'b' * 64})
    placements = [{'cue': c, 'chapter': fx.names[i % 13], 'start_frame': 600 * i, 'duration_s': 5.0} for i, c in enumerate(fx.cues)]
    js(job(fx, verify, fx.O / 'job-scripts', asr02.VERIFY_SUCCESS) / 'verification.json',
       {'video': str(movie), 'sha256': sha(movie), 'srt_sha256': sha(srt), 'vtt_sha256': sha(vtt), 'master_job': master, 'frames': 48000,
        'fps': 60, 'chapters': fx.chapters, 'placements': placements, 'strict_video_check': 'passed', 'pictures_viewed': False,
        'tools_sha256': tools, 'binding_sha256': 'b' * 64,
        'worker': {'host': 'mac-mini-max', **{k: fx.tools[k] for k in ('python3', 'ffmpeg', 'ffprobe')}}})


def start(fx, *jobs, path=None):
    """The wrapper as root would start it, with only HOME in its environment."""
    jobs = jobs or (V, M, '02')
    done = subprocess.run(['/bin/bash', str(path or fx.bundle / 'start_asr02.sh'), *jobs], env={'HOME': str(fx.base / 'home')},
                          capture_output=True, text=True, timeout=60)
    log = fx.O / f'asr{jobs[-1]}.log'
    return done.returncode, done.stdout + done.stderr + (log.read_text() if log.exists() else '')


def helper(fx, *args):
    done = subprocess.run([sys.executable, '-I', '-B', str(fx.bundle / 'asr02.py'), *(args or (V, M, '02'))],
                          capture_output=True, text=True, timeout=60)
    return done.returncode, done.stdout + done.stderr


def events(fx):
    path = fx.base / 'events.jsonl'
    return [json.loads(x) for x in path.read_text().splitlines()] if path.exists() else []


def check(ok, label):
    if not ok:
        raise SystemExit(f'FAIL {label}')


def postcheck(fx):
    js(fx.O / 'asr-postcheck02.json', {'verify_job': 'PASS', 'job': V, 'verification_sha256': sha(fx.jobs / V / 'verification.json'),
                                       'master_sha256': json.loads((fx.jobs / V / 'verification.json').read_text())['sha256'],
                                       'frames': 48000, 'pictures_viewed': False})


def good_film(base):
    """Started through a symlinked path, so physical paths must come from the wrapper rather than the caller."""
    fx = build(base)
    (base / 'link').symlink_to(fx.repo)
    code, text = start(fx, path=base / 'link' / REL_P / 'logarithm-remote-asr02/start_asr02.sh')
    check(code == 0, 'good film: ' + text[-400:])
    seen = events(fx)
    check([x['kind'] for x in seen] == ['verifier', 'preflight', 'run'], 'order: post-check, preflight, launch')
    probe, launched = seen[1], seen[2]
    check(seen[0]['cwd'] == str(fx.repo) and seen[0]['flags'][seen[0]['flags'].index('--lane') + 1] == str(fx.jobs), 'physical cwd and lane')
    check(seen[0]['flags'][seen[0]['flags'].index('--master-job') + 1] == M, 'selected master reaches verify-job')
    home = fx.base / 'home'
    want_path = f'{home}/.local/bin:{home}/.cargo/bin:/opt/homebrew/bin:/usr/bin:/bin:/usr/sbin:/sbin'
    check(probe['path'] == launched['path'] == want_path and Path(probe['sbh']) == home / '.local/bin/sbh', 'wrapper PATH reaches the guard')
    check(probe['flags'] == ['--phase', 'asr', '--job', 'logarithm-remote-asr02-preflight', '--preflight-only'], 'preflight line')
    check(launched['flags'] == ['--phase', 'asr', '--job', 'logarithm-remote-asr02', '--inputs', str(fx.O / 'asr-inputs02.json')], 'guard line')
    run = [sys.executable, '-I', '-B', str(fx.bundle / 'asr02.py'), '--run', V, M, '02']
    check(launched['command'] == run, 'guard command carries the selection')
    out, movie = fx.jobs / 'logarithm-remote-asr02', fx.jobs / 'logarithm-remote-render-work01/logarithm_mysteries_master.mp4'
    wav = out / 'encoded-master-16k-mono.wav'
    model = fx.stand / 'model.bin'
    check((fx.stand / 'ffmpeg.argv').read_text().splitlines() == ['-nostdin', '-n', '-v', 'error', '-xerror', '-i', str(movie), '-map', '0:a:0',
          '-ac', '1', '-ar', '16000', '-c:a', 'pcm_s16le', str(wav)], 'decode arguments')
    check((fx.stand / 'fw.argv').read_text().splitlines() == ['transcribe', '--input', str(wav), '--backend', 'whisper-cpp', '--model', str(model),
          '--language', 'en', '--no-diarize', '--no-persist', '--split-on-word', '--max-segment-length', '1', '--threads', '4',
          '--timeout', '7200', '--json'], 'fw arguments')
    comparison = json.loads((out / 'comparison.json').read_text())
    check(comparison['master_sha256'] == sha(movie) and comparison['cue_count'] == 80 and (out / 'cue-boundaries.json').is_file(), 'ASR outputs')
    bound = {x['path'] for x in json.loads((fx.O / 'asr-inputs02.json').read_text())['files']}
    for path in (fx.O / 'asr-postcheck02.json', fx.jobs / M / 'master-summary.json', fx.jobs / V / 'command.log', fx.O / 'tools01.json'):
        check(str(path) in bound, f'manifest binds {path.name}')
    check((fx.O / 'asr02.exit').read_text() == '0\n', 'exit file')
    return f'post-check, preflight, launch in order; PATH, physical paths, selection and every ASR argument exact; {len(bound)} rows bound'


def good_retry(base):
    """F3: both retries reach ASR, with the failed attempts left beside them."""
    fx = build(base)
    for name in (V, M):
        edit(fx.jobs / name / 'resource-receipt.json', lambda d: d.update(exit=1))
    film(fx, 'logarithm-remote-verify02', 'logarithm-remote-master01-r2')
    code, text = start(fx, 'logarithm-remote-verify02', 'logarithm-remote-master01-r2', '03')
    check(code == 0 and (fx.jobs / 'logarithm-remote-asr03/comparison.json').is_file(), 'retry: ' + text[-400:])
    return 'verify02 of master01-r2 reaches ASR as attempt 03 beside failed verify01 and master01'


def proof(fx):
    return fx.jobs / V / 'verification.json'


def receipt(fx, name):
    return fx.jobs / name / 'resource-receipt.json'


def retire(fx):
    path = fx.bundle / 'asr02.py'
    text = path.read_text()
    assert text.count(RETIRED) == 1
    path.write_text(text.replace(RETIRED, json.loads(proof(fx).read_text())['sha256']))


def other_ffmpeg(fx):
    other = write(fx.stand / 'ffmpeg-other', '#!/bin/sh\n', 0o755)
    edit(fx.O / 'asr-source-bindings01.json', lambda d: d['files'].__setitem__(3, row(other)))


def key(path, *names, value):
    def change(d):
        for name in names[:-1]:
            d = d[name]
        d[names[-1]] = value
    edit(path, change)


def no_sbh(fx):
    path = fx.bundle / 'start_asr02.sh'
    lines = [x if not x.startswith('export PATH=') else 'export PATH="/usr/bin:/bin:/usr/sbin:/sbin"' for x in path.read_text().splitlines()]
    path.write_text('\n'.join(lines) + '\n')


def later(fx, name):
    return (fx.jobs / name).mkdir()


def unfinished(fx):
    edit(receipt(fx, V), lambda d: d.pop('finished'))


# Findings from the sealed review. Each plant is the failure itself; every record it does not name stays good.
WRAPPER_CASES = [
    ('F1', 'retired master', retire, 'retired stage09 master'),
    ('F1', 'summary of another master', lambda f: key(f.jobs / M / 'master-summary.json', 'master', 'sha256', value='0' * 64), 'another master'),
    ('F1', 'summary names another file', lambda f: key(f.jobs / M / 'master-summary.json', 'master', 'path', value='/other.mp4'), 'another master file'),
    ('F1', 'proof of another master job', lambda f: key(proof(f), 'master_job', value='logarithm-stage09-master02'), 'the proof is for'),
    ('F2', 'verify job failed', lambda f: key(receipt(f, V), 'exit', value=1), 'did not pass'),
    ('F2', 'verify ran in another phase', lambda f: key(receipt(f, V), 'phase', value='asr'), 'ran in phase'),
    ('F2', 'verify ran another command', lambda f: key(receipt(f, V), 'command', value=[sys.executable, '-I', '-B', 'other.py']), 'another command'),
    ('F2', 'verify inputs moved', lambda f: js(f.O / f'job-scripts/{V}-inputs.json', {'files': []}), 'frozen inputs moved'),
    ('F2', 'verify log ends on a near-match', lambda f: write(f.jobs / V / 'command.log', 'FINAL VERIFY PASS: but not the line\n'), 'success line'),
    ('F2', 'strict check not passed', lambda f: key(proof(f), 'strict_video_check', value='failed'), 'strict video check'),
    ('F2', 'proof claims a viewing', lambda f: key(proof(f), 'pictures_viewed', value=True), 'picture viewing'),
    ('F2', 'post-check refuses', lambda f: write(f.base / 'verifier-refuses', ''), 'the stand-in verifier refuses'),
    ('M', 'master job failed', lambda f: key(receipt(f, M), 'exit', value=75), 'did not pass'),
    ('M', 'master ran another command', lambda f: key(receipt(f, M), 'command', value=['python3', '-I', '-B', 'other.py']), 'another command'),
    ('M', 'master log lacks its success line', lambda f: write(f.jobs / M / 'command.log', 'work\n'), 'success line'),
    ('M', 'master replaced after verify', lambda f: write(f.jobs / 'logarithm-remote-render-work01/logarithm_mysteries_master.mp4', 'other\n'), 'changed after verification'),
    ('F3', 'newer master attempt ignored', lambda f: later(f, M + '-r2'), 'not the newest'),
    ('F3', 'newer verify attempt ignored', lambda f: later(f, 'logarithm-remote-verify02'), 'not the newest'),
    ('F6', 'bindings ffmpeg is not the pinned tool', other_ffmpeg, 'ASR ffmpeg is not the pinned tool'),
    ('F6', 'tools file replaced after verify', lambda f: key(f.O / 'tools01.json', 'ffprobe', 'version', value='other'), 'tools'),
    ('F6', 'proof names another worker ffmpeg', lambda f: key(proof(f), 'worker', 'ffmpeg', 'sha256', value='0' * 64), 'other tools'),
    ('F4', 'preflight not admitted', lambda f: write(f.base / 'guard-refuses', ''), None),
    ('F4', 'sbh off PATH', no_sbh, 'sbh is not on PATH'),
    ('pin', 'verifier is not the sealed helper', lambda f: write(f.repo / REL_P / 'logarithm-remote-final-verifier02/verify_remote_final.py', '# other\n'), 'not the sealed verifier02'),
]
HELPER_CASES = [  # asr02.py started without the wrapper
    ('F2', 'no post-check record', None, 'no verifier post-check record'),
    ('F2', 'proof changed after the post-check', lambda f: (postcheck(f), key(proof(f), 'closing', value='forged')), 'read another proof'),
    ('F6', 'interpreter is not the pinned one', lambda f: (postcheck(f), key(f.O / 'tools01.json', 'python3', 'path', value='/usr/bin/true')), 'pinned interpreter'),
]


def refusals(base):
    n = 0
    for mode, cases in (('wrapper', WRAPPER_CASES), ('helper', HELPER_CASES)):
        for finding, name, plant, words in cases:
            n += 1
            fx = build(base / f'case{n:02d}')
            if plant:
                plant(fx)
            code, text = start(fx) if mode == 'wrapper' else helper(fx)
            check(code != 0, f'{name}: accepted')
            check(words is None or words in text, f'{name}: {text[-400:]}')
            check(not (fx.jobs / 'logarithm-remote-asr02').exists() and not (fx.O / 'asr-inputs02.json').exists(), f'{name}: job name or manifest used')
            said = 'guard exit 75' if words is None else [x for x in text.splitlines() if x.strip()][-1]
            print(f'PASS {finding:4}{name}: {said.replace(str(fx.base), "SCRATCH")}')
    fx = build(base / 'unfinished')  # No waiter: an unfinished job is refused at once.
    unfinished(fx)
    began = time.monotonic()
    code, text = helper(fx)
    took = time.monotonic() - began
    check(code != 0 and 'did not pass' in text and took < 5, f'unfinished verify: {took} {text[-300:]}')
    print(f'PASS F3  verify still running: refused at once, under 5 s, no wait')
    fx = build(base / 'used')  # F4: a used name refuses before any step.
    later(fx, 'logarithm-remote-asr02')
    code, text = start(fx)
    check(code == 2 and 'use the next attempt number' in text and not events(fx), 'used attempt')
    print('PASS F4  attempt already used: refused before any step')
    fx = build(base / 'order')  # Each step stops the next.
    write(fx.base / 'verifier-refuses', '')
    start(fx)
    check([x['kind'] for x in events(fx)] == ['verifier'] and (fx.O / 'asr02.exit').read_text() == '1\n', 'post-check stops the rest')
    fx = build(base / 'order2')
    write(fx.base / 'guard-refuses', '')
    start(fx)
    check([x['kind'] for x in events(fx)] == ['verifier', 'preflight'] and (fx.O / 'asr02.exit').read_text() == '75\n', 'preflight stops launch')
    print('PASS F4  a refused post-check reaches no guard; a refused preflight reaches no launch; both leave their exit codes')
    for jobs in ((V, 'logarithm-stage09-master02', '02'), ('logarithm-remote-verify1', M, '02'), (V, M, '01')):
        fx = build(base / f'name{jobs[0][-1]}{jobs[2]}{len(jobs[1])}')
        for code, text in (start(fx, *jobs), helper(fx, *jobs)):
            check(code != 0 and not events(fx) and not (fx.O / 'asr-inputs02.json').exists(), f'bad name {jobs}')
    print('PASS F3  bad master name, bad verify name and attempt 01: wrapper and helper both refuse')
    fx = build(base / 'run')  # The ASR step itself refuses anything but its own guarded job.
    code, text = helper(fx, '--run', V, M, '02')
    check(code != 0 and 'FileNotFoundError' in text and not (fx.stand / 'ffmpeg.argv').exists(), '--run outside the guard')
    js(fx.jobs / 'logarithm-remote-asr02/resource-receipt.json', {'admitted': True, 'phase': 'asr', 'inputs_sha256': 'x', 'command': []})
    write(fx.O / 'asr-inputs02.json', '{"files": []}\n')
    code, text = helper(fx, '--run', V, M, '02')
    check(code != 0 and 'not this guarded ASR job' in text and not (fx.stand / 'ffmpeg.argv').exists(), '--run with a foreign receipt')
    print('PASS run  --run outside the guard or under a foreign receipt refuses before any decode')


def nodes(path):
    return {n.name: n for n in ast.parse(path.read_text()).body if isinstance(n, ast.FunctionDef)}


def constants(path, *names):
    found = {}
    for n in ast.parse(path.read_text()).body:
        if isinstance(n, ast.Assign) and isinstance(n.targets[0], ast.Name) and n.targets[0].id in names:
            found[n.targets[0].id] = ast.literal_eval(n.value)
    return found


def tail(function):
    at = [i for i, s in enumerate(function.body) if ast.unparse(s).startswith("wav = OUT /")]
    assert len(at) == 1
    return [ast.dump(s) for s in function.body[at[0]:]]


def sources():
    O = R / REL_O
    root = O / 'asr01.py'
    check(sha(root) == '203f2ad3b333e8618ab252f380e1d629f321121bb23eacd49c67959a827fb03c', 'O/asr01.py changed')
    check(sha(O / 'asr-source-bindings01.json')[:16] == '71e140bbc3bdd218', 'bindings changed')
    old, new = nodes(root), nodes(HERE / 'asr02.py')
    kept = ('read', 'sha', 'check_files', 'script_rows', 'tokens', 'boundary_review')
    check(all(ast.dump(old[n]) == ast.dump(new[n]) for n in kept), 'kept functions')
    check(tail(old['run']) == tail(new['run']), 'run() from the decode on')
    print(f'PASS keep O/asr01.py and its bindings unchanged; {", ".join(kept)} and all of run() from the decode on equal root\'s by AST')
    text = (HERE / 'asr02.py').read_text()
    check('while ' not in text and 'sleep' not in text and 'import time' not in text and 'range(8640)' not in text and '.exit' not in text, 'waiter')
    print('PASS F5  no wait loop, no sleep, no exit-file watching')
    want = ''.join(difflib.unified_diff(root.read_text().splitlines(True), text.splitlines(True), 'O/asr01.py', 'asr02.py'))
    check((HERE / 'docs/asr01-to-asr02.diff').read_text() == want, 'diff document')
    print('PASS diff docs/asr01-to-asr02.diff is the whole change to asr01.py')
    verifier = R / REL_P / 'logarithm-remote-final-verifier02/verify_remote_final.py'
    check(sha(verifier) == PIN, 'verifier pin')
    v = constants(verifier, 'RETIRED_MASTER', 'VERIFY_SUCCESS', 'TOOLS_SCHEMA', 'HOST')
    check(all(getattr(asr02, k) == x for k, x in v.items()) and len(v) == 4, 'verifier constants')
    body = verifier.read_text()
    keys = ('master_job', 'video', 'srt_sha256', 'vtt_sha256', 'strict_video_check', 'pictures_viewed', 'tools_sha256', 'binding_sha256',
            'worker', 'verification_sha256', 'master_sha256')
    check(all(f'"{k}"' in body for k in keys) and '{"verify_job": "PASS"' in body, 'verifier keys')
    print(f'PASS peer  the sealed verifier {PIN[:12]}: pin, 4 constants and {len(keys)} keys agree (source text, not run)')
    lavender = R / REL_P / 'logarithm-remote-final-production01'
    driver, worker = (lavender / 'render_master_remote.py'), (lavender / 'job_worker.py')
    d, w = driver.read_text(), worker.read_text()
    check(constants(driver, 'MASTER_SUCCESS')['MASTER_SUCCESS'] == asr02.MASTER_SUCCESS, 'master success')
    check('SCRIPTS, PRIOR_SCRIPTS = O / "render-jobs01", O / "job-scripts"' in d and 'SCRIPTS / f"{job}-inputs.json"' in d
          and 'python, "-I", "-B", str(script)]' in d and 'summary("master-summary.json", data)' in w, 'master forms')
    print(f'PASS peer  Lavender\'s UNSEALED driver {sha(driver)[:12]} and worker {sha(worker)[:12]}: success line, paths, command form agree (source text, not run)')
    check(subprocess.run(['bash', '-n', str(HERE / 'start_asr02.sh')]).returncode == 0, 'bash -n')
    print('PASS sh   bash -n')


def main():
    base = Path(sys.argv[1]).resolve()
    base.mkdir()
    sources()
    print('PASS good ' + good_film(base / 'good'))
    print('PASS good ' + good_retry(base / 'retry'))
    refusals(base)
    print('ALL PASS. Stand-ins only: nothing ran on mac-mini-max, and fw transcribe, the real guard and the real verifier did not run.')


if __name__ == '__main__':
    main()
