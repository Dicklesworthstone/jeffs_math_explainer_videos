"""Focused checks: one good film through both steps, then the real failures.

Usage, from the repository root, with a fresh scratch path: python3 -I -B CHECKS/run_checks.py SCRATCH

The delivered scripts and wrappers, the sealed qualifier and the real 105-cue script.tsv run in a tiny scratch repository
at their real depth, behind a symlink that plays ts1's /Users alias. Stand-ins: both guards, ffmpeg, fw, sbh, the bindings,
the tools file and every record. Nothing real is launched, no media exists and no ASR runs. A copy of mini_asr01.py differs
from the delivered file only in its two pin literals, because the fixture's bindings and tools files are stand-ins.
"""
import ast
import hashlib
import json
from pathlib import Path
import re
import subprocess
import sys
import wave

HERE = Path(__file__).resolve().parents[1]
R = HERE.parents[4]
REL_P = 'docs/video-program/third-wave-20261024/opus-replacements-20261104'
REL_EC = 'narration_lab/third-wave-20261024/root/elevenlabs-complex-rebuild-20261104'
REL_O = REL_EC + '/remote-ts1-01'
REL_LOG = 'narration_lab/third-wave-20261024/root/elevenlabs-logarithm-rebuild-20261104/remote-mac-mini-max01'
REL_SCRIPT = REL_EC + '/native-stage07-root01/workspace/explainer_corpus/complex_analysis_visual/script.tsv'
REL_Q = REL_P + '/complex-remote-ts1-pipeline01/qualify02/ts1_qualify02.py'
REL_PRESERVED = REL_P + '/engine-secondary-validation01/guard_owner56g_target20g.py'
ASR02_SEAL = 'f63501eae4cd2097b062c16ca6c2ace0f87e1b1daabf4e5b0e77a4d8a0528189'
ROOT_COMPLEX = '6ac60883507c84b11b7dcbf2f11416a586ee2d4fd28ebd3562ebbd7fbee86460'
CHAIN_SEAL = 'bbf00548ba5f804fcecefe31854ffa9af60494ba71003cfc2ccd491fa871daf9'
QUALIFY_SEAL = 'f6878e685a7202c9cc32865314bfff0686068ce4f678502a06b31305b52cd860'
BIN = 'b' * 64  # a stand-in: qualify02 reads the real hash from build02's binary.sha256 at import
V, M, A = 'complex-remote-final-verify01', 'complex-remote-master01', 'complex-remote-asr-audio01'
VOICE = 'ElevenLabs Frederick Surrey'
NATIVE = {'admission_gib': 10, 'stop_free_gib': 8, 'phase_growth_gib': 4, 'work_cap_gib': 56, 'target_growth_cap_gib': 20, 'sample_seconds': 10}
CHAPTERS = ('01_the_bump', '02_turning_numbers', '03_two_planes', '04_reading_color', '05_gallery', '06_one_derivative', '07_one_point',
            '08_loop_sums', '09_boundary', '10_rigidity', '11_counting', '12_residues', '13_real_integral', '14_continuation', '15_closing')
FRAMES_PER_CUE = 12  # tiny: 21 s of film, one small WAV

GUARD = r'''"""Stand-in for either lane's guard01.py: the real guard's arguments and receipt keys, with no resource policy.
With guard-partial it rewrites the live receipt as the real guard can: unparsable for a moment, or for good."""
import hashlib, json, os, shutil, subprocess, sys, time
from pathlib import Path
O = Path(__file__).resolve().parent
fixture = O.parents[5]
sha = lambda p: hashlib.sha256(Path(p).read_bytes()).hexdigest()
a = sys.argv[1:]
cut = a.index('--') if '--' in a else len(a)
flags, command = a[:cut], a[cut + 1:]
preflight = '--preflight-only' in flags
phase = flags[flags.index('--phase') + 1]
with (fixture / 'events.jsonl').open('a') as h:
    h.write(json.dumps({'kind': 'preflight' if preflight else 'run', 'lane': O.name, 'flags': flags, 'command': command, 'cwd': os.getcwd(),
                        'pwd': os.environ.get('PWD'), 'path': os.environ['PATH'], 'sbh': shutil.which('sbh')}) + '\n')
job = O / 'jobs' / flags[flags.index('--job') + 1]
job.mkdir(parents=True)
r = {'phase': phase, 'command': command, 'preflight_only': preflight,
     'policy': {'admission_gib': 12 if phase == 'asr' else 10, 'stop_free_gib': 8, 'phase_growth_gib': 1 if phase == 'asr' else 4,
                'work_cap_gib': 56, 'target_growth_cap_gib': 20, 'sample_seconds': 10}}
if (fixture / 'guard-policy').exists():
    r['policy']['work_cap_gib'] = 48
save = lambda: (job / 'resource-receipt.json').write_text(json.dumps(r))
r['admitted'] = not (fixture / 'guard-refuses').exists()
if preflight or not r['admitted']:
    r['finished'] = 'stand-in'
    save()
    sys.exit(0 if r['admitted'] else 75)
manifest = Path(flags[flags.index('--inputs') + 1])
for row in json.loads(manifest.read_text())['files']:
    assert sha(row['path']) == row['sha256'], row['path']
r['inputs_sha256'] = sha(manifest)
save()
partial = fixture / 'guard-partial'
if partial.exists():
    (job / 'resource-receipt.json').write_text('{"started": ')
with (job / 'command.log').open('x') as log:
    child = subprocess.Popen(command, cwd=O.parents[4], stdout=log, stderr=subprocess.STDOUT)
    if partial.exists() and partial.read_text() == 'moment':
        time.sleep(1)
        save()
    code = child.wait()
r.update(exit=code, final_inputs_sha256=sha(manifest), finished='stand-in')
save()
sys.exit(code)
'''
FFMPEG = f'''#!{sys.executable} -I
import sys, wave
from pathlib import Path
here = Path(__file__).resolve().parent
(here / 'ffmpeg.argv').write_text('\\n'.join(sys.argv[1:]) + '\\n')
rate, channels, samples = map(int, (here / 'ffmpeg.plan').read_text().split())
with wave.open(sys.argv[-1], 'wb') as w:
    w.setnchannels(channels)
    w.setsampwidth(2)
    w.setframerate(rate)
    w.writeframes(b'\\0\\0' * samples * channels)
'''
FW = '#!/bin/sh\nprintf \'%s\\n\' "$@" > "$(dirname "$0")/fw.argv"\ncat "$(dirname "$0")/fw.json"\n'
PRESERVED = '''class G:
    ADMISSION_GIB = {'native': 10, 'asr': 12}
    STOP_FREE_GIB = 8
    CAPS = {'native': 4, 'asr': 1}
    WORK_CAP_GIB = 56
    TARGET_GROWTH_CAP_GIB = 20


def load():
    return G
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


def edit(path, change):
    data = json.loads(path.read_text())
    change(data)
    js(path, data)


def row(path):
    return {'path': str(path), 'sha256': sha(path)}


def key(path, *names, value):
    def change(d):
        for name in names[:-1]:
            d = d[name]
        d[names[-1]] = value
    edit(path, change)


def literal(path, old, new):
    text = path.read_text()
    assert text.count(old) == 1, old
    path.write_text(text.replace(old, new))


def constants(path, *names):
    found = {}
    for n in ast.parse(path.read_text()).body:
        if isinstance(n, ast.Assign) and isinstance(n.targets[0], ast.Name) and n.targets[0].id in names:
            found[n.targets[0].id] = ast.literal_eval(n.value)
    return found


SUCCESS = constants(HERE / 'ts1_asr_audio01.py', 'VERIFY_SUCCESS', 'MASTER_SUCCESS')


def tiny_wav(path):
    path.parent.mkdir(parents=True, exist_ok=True)
    with wave.open(str(path), 'wb') as w:
        w.setnchannels(1)
        w.setsampwidth(2)
        w.setframerate(8000)
        w.writeframes(b'\0\0' * 800)  # 0.1 s


def build(base):
    """A complete good film under base, built whole for each case: every record holds its own absolute paths."""
    fx = type('Fixture', (), {})()
    fx.base, fx.repo, fx.logical = base, base / 'physical', base / 'logical'
    fx.repo.mkdir(parents=True)
    fx.logical.symlink_to(fx.repo)
    fx.home = base / 'home'
    fx.bin = fx.home / '.local/bin'
    fx.O, fx.jobs = fx.logical / REL_O, fx.logical / REL_O / 'jobs'
    fx.stage, fx.work = fx.O / 'final-production01', fx.jobs / 'complex-remote-render-work01'
    fx.final = fx.logical / REL_P / 'complex-remote-final-production01'
    fx.bundle, fx.bundle_logical = fx.repo / REL_P / 'complex-remote-asr01', fx.logical / REL_P / 'complex-remote-asr01'
    fx.lane, fx.lane_jobs = fx.repo / REL_LOG, fx.repo / REL_LOG / 'jobs'
    fx.audio = fx.repo / REL_O / 'jobs' / A
    write(fx.bin / 'python3', f'#!/bin/sh\nexec {sys.executable} "$@"\n', 0o755)
    write(fx.bin / 'sbh', '#!/bin/sh\n', 0o755)
    fx.ffmpeg = write(fx.bin / 'ffmpeg', FFMPEG, 0o755)
    write(fx.repo / REL_Q, (R / REL_Q).read_text())
    fx.script = write(fx.repo / REL_SCRIPT, (R / REL_SCRIPT).read_text())
    for name in ('ts1_asr_audio01.py', 'mini_asr01.py'):
        write(fx.bundle / name, (HERE / name).read_text())
    for name in ('start-asr-audio01.sh', 'start-asr01.sh'):
        write(fx.bundle / name, (HERE / name).read_text(), 0o755)
    preserved = write(fx.logical / REL_PRESERVED, PRESERVED)
    guard = write(fx.O / 'guard01.py', GUARD)
    js(fx.O / 'inputs01.json', {'files': [row(guard)]})
    js(fx.O / 'guard-pins01.json', {'files': [row(preserved), row(write(fx.O / 'pin2', 'b\n'))]})
    fx.rows = [line.split('\t', 1) for line in fx.script.read_text().splitlines() if line.strip() and not line.startswith('#')]
    fx.cues = [cue for cue, _ in fx.rows]
    fx.chapters, fx.placements = [], []
    n2 = fx.jobs / 'complex-remote-narration-derive02/narration-n2'
    write(fx.jobs / 'complex-remote-build02/binary.sha256', BIN + '\n')
    start = 0
    for index, chapter in enumerate(CHAPTERS):
        cues = [c for c in fx.cues if c.startswith(f'{index + 1:02d}_')]
        # The physical spelling: White's chain compares these resolved, so the binary may write either.
        lines = [f'line\t{cue}\t{i * FRAMES_PER_CUE}\t0\t{n2.resolve()}/{cue}.wav' for i, cue in enumerate(cues)]
        frames = len(cues) * FRAMES_PER_CUE
        text = '\n'.join(['# explainer timeline v1', f'chapter\t{chapter}', 'fps\t60', f'frames\t{frames}', *lines, 'breath\t1\t2', '']) + '\n'
        source = write(fx.jobs / 'complex-remote-timeline03/timeline-output' / f'{chapter}.timeline.tsv', text)
        write(fx.work / f'{chapter}.timeline.tsv', text)  # the kit's own comment line may differ; only normalized lines count
        fx.chapters.append({'chapter': chapter, 'frames': frames, 'cues': cues, 'timeline_sha256': sha(source)})
        for i, cue in enumerate(cues):
            tiny_wav(n2 / f'{cue}.wav')
            fx.placements.append({'cue': cue, 'chapter': chapter, 'start_frame': start + i * FRAMES_PER_CUE, 'duration_s': 0.1})
        start += frames
    fx.total = start
    js(fx.jobs / 'complex-remote-timeline03/timeline-summary.json', {'chapters': fx.chapters, 'total_frames': fx.total, 'fps': 60, 'binary_sha256': BIN})
    write(fx.bin / 'ffmpeg.plan', f'16000 1 {fx.total * 16000 // 60}\n')
    film(fx, V, M)
    worker(fx)
    return fx


def worker(fx):
    """The mac-mini-max lane: root's two files, stand-ins here, and the delivered helper re-pinned to them."""
    stand = fx.lane / 'stand-ins'
    fw, model = write(stand / 'fw', FW, 0o755), write(stand / 'model.bin', 'model\n')
    ffmpeg, ffprobe = write(stand / 'ffmpeg', '#!/bin/sh\n', 0o755), write(stand / 'ffprobe', '#!/bin/sh\n', 0o755)
    js(fx.lane / 'tools01.json', {'schema': 'logarithm-remote-tools-v1', 'host': 'mac-mini-max',
                                  'python3': {'path': sys.executable, 'sha256': sha(sys.executable)},
                                  'ffmpeg': row(ffmpeg), 'ffprobe': row(ffprobe)})
    js(fx.lane / 'asr-source-bindings01.json', {'files': [row(write(stand / 'logarithm-script.tsv', 'x\n')), row(fw), row(model), row(ffmpeg)]})
    js(fx.lane / 'guard-pins01.json', {'files': [row(write(stand / 'pin1', 'a\n'))]})
    write(fx.lane / 'guard01.py', GUARD)
    repin(fx)
    segments = [{'start_sec': p['start_frame'] / 60, 'end_sec': p['start_frame'] / 60 + 0.1, 'text': text} for p, (_, text) in zip(fx.placements, fx.rows)]
    js(stand / 'fw.json', {'result': {'segments': segments}})
    fx.stand = stand


def repin(fx):
    path = fx.bundle / 'mini_asr01.py'
    lines = path.read_text().splitlines(True)
    want = {'SOURCES_SHA256 = ': sha(fx.lane / 'asr-source-bindings01.json'), 'TOOLS_SHA256 = ': sha(fx.lane / 'tools01.json')}
    path.write_text(''.join(next((f"{k}'{h}'\n" for k, h in want.items() if line.startswith(k)), line) for line in lines))


def job(fx, name, command, helper, success, extra=()):
    inputs = js(fx.stage / f'{name}-inputs.json', {'files': [row(helper), *(row(x) for x in extra)]})
    folder = fx.jobs / name
    js(folder / 'resource-receipt.json', {'finished': 'stand-in', 'admitted': True, 'exit': 0, 'phase': 'native', 'policy': NATIVE, 'command': command,
                                          'inputs_sha256': sha(inputs), 'final_inputs_sha256': sha(inputs)})
    write(folder / 'command.log', f'work\n{success}\n')
    return folder


def film(fx, verify, master):
    movie = write(fx.work / 'complex_analysis_visual_master.mp4', f'stand-in master of {master}\n')
    srt, vtt = write(fx.work / 'complex_analysis_visual.en.srt', 'srt\n'), write(fx.work / 'complex_analysis_visual.en.vtt', 'WEBVTT\n')
    helper_m = write(fx.final / 'ts1_final_master01.py', '# stand-in\n')
    helper_v = write(fx.final / 'ts1_final_verify01.py', '# stand-in\n')
    folder = job(fx, master, [sys.executable, '-I', '-B', str(helper_m), '--run', master], helper_m, SUCCESS['MASTER_SUCCESS'])
    js(folder / 'master-summary.json', {'chapters': list(CHAPTERS), 'frames': fx.total, 'fps': 60, 'binary_sha256': BIN,
                                        'outputs': {'master': row(movie), 'srt': row(srt), 'vtt': row(vtt)}})
    python = fx.jobs / 'complex-remote-picture-env01/env/bin/python'
    folder = job(fx, verify, [str(python), '-I', '-B', str(helper_v), '--run', verify], helper_v, SUCCESS['VERIFY_SUCCESS'], [fx.ffmpeg])
    write(folder / 'strict-video-verification.log', 'note: stand-in\nVERIFY PASS\n')
    chapters, offset = [], 0
    for chapter in fx.chapters:
        chapters.append({'chapter': chapter['chapter'], 'start_frame': offset, 'frames': chapter['frames']})
        offset += chapter['frames']
    proof = js(folder / 'verification.json', {
        'video': str(movie), 'sha256': sha(movie), 'frames': fx.total, 'fps': 60, 'chapters': chapters, 'placements': fx.placements, 'host': 'ts1',
        'voice': VOICE, 'pictures_viewed': False, 'binary_sha256': BIN, 'timeline_job': 'complex-remote-timeline03', 'master_job': master,
        'narration': str(fx.jobs / 'complex-remote-narration-derive02/narration-n2')})
    js(fx.stage / f'{verify}-postcheck.json', {'job': verify, 'master_job': master, 'master_sha256': sha(movie), 'srt_sha256': sha(srt), 'vtt_sha256': sha(vtt),
                                               'verification_sha256': sha(proof), 'frames': fx.total, 'pictures': 3, 'sheets': 1, 'pictures_viewed': False, 'scope': 'stand-in'})


def run(fx, script, *args):
    """A wrapper as root would start it, with only HOME in its environment."""
    done = subprocess.run(['/bin/bash', str(script), *args], env={'HOME': str(fx.home)}, capture_output=True, text=True, timeout=120)
    return done.returncode, done.stdout + done.stderr


def ts1(fx, *jobs, script=None):
    jobs = jobs or (V, M, '01')
    code, text = run(fx, script or fx.bundle / 'start-asr-audio01.sh', *jobs)
    log = fx.O / f'asr-audio{jobs[-1]}.log'
    return code, text + (log.read_text() if log.exists() else '')


def mini(fx, *args):
    args = args or (A, sha(fx.audio / 'audio-summary.json'), '01')
    code, text = run(fx, fx.bundle / 'start-asr01.sh', *args)
    log = fx.lane / f'complex-asr{args[-1]}.log'
    return code, text + (log.read_text() if log.exists() else '')


def direct(fx, name, *args):
    done = subprocess.run([sys.executable, '-I', '-B', str(fx.bundle / name), *args], capture_output=True, text=True, timeout=60,
                          env={'HOME': str(fx.home), 'PATH': f'{fx.bin}:/usr/bin:/bin'})
    return done.returncode, done.stdout + done.stderr


def events(fx):
    path = fx.base / 'events.jsonl'
    return [json.loads(x) for x in path.read_text().splitlines()] if path.exists() else []


def check(ok, label):
    if not ok:
        raise SystemExit(f'FAIL {label}')


def plant_leads(fx):
    """What the transcript must carry to the reader: a changed word, a real minus sign, and a silent cue."""
    def change(d):
        segments = d['result']['segments']
        segments[10]['text'] = 'nonsense ' + segments[10]['text']
        segments[20]['text'] = 'z − 1 ' + segments[20]['text']
        del segments[30]
    edit(fx.stand / 'fw.json', change)


def good_film(base):
    """Both steps, end to end. ts1's wrapper starts through the logical path, as root starts it there."""
    fx = build(base)
    plant_leads(fx)
    code, text = ts1(fx, script=fx.bundle_logical / 'start-asr-audio01.sh')
    check(code == 0, 'ts1 step: ' + text[-500:])
    seen = events(fx)
    check([x['kind'] for x in seen] == ['preflight', 'run'] and {x['lane'] for x in seen} == {'remote-ts1-01'}, 'ts1 order')
    home = fx.home
    check(seen[0]['path'] == seen[1]['path'] == f'{home}/.local/bin:{home}/.cargo/bin:/usr/bin:/bin' and Path(seen[0]['sbh']) == fx.bin / 'sbh', 'ts1 PATH')
    check(seen[0]['flags'] == ['--phase', 'asr', '--job', A + '-preflight', '--preflight-only'], 'ts1 preflight line')
    check(seen[1]['flags'] == ['--phase', 'asr', '--job', A, '--inputs', str(fx.O / 'asr-audio-inputs01.json')], 'ts1 guard line')
    check(seen[1]['command'] == [sys.executable, '-I', '-B', str(fx.bundle_logical / 'ts1_asr_audio01.py'), '--run', V, M, '01'], 'ts1 logical command')
    wav_logical, movie = fx.jobs / A / 'encoded-master-16k-mono.wav', fx.work / 'complex_analysis_visual_master.mp4'
    check((fx.bin / 'ffmpeg.argv').read_text().splitlines() == ['-nostdin', '-n', '-v', 'error', '-xerror', '-i', str(movie), '-map', '0:a:0',
          '-ac', '1', '-ar', '16000', '-c:a', 'pcm_s16le', str(wav_logical)], 'decode arguments')
    summary = fx.audio / 'audio-summary.json'
    proof = json.loads(summary.read_text())
    check(proof['wav'] == str(wav_logical) and proof['sha256'] == sha(movie) and proof['wav_sha256'] == sha(wav_logical) and len(proof['placements']) == 105, 'summary')
    check(proof['decode_command'] == [str(fx.ffmpeg), *(fx.bin / 'ffmpeg.argv').read_text().splitlines()] and proof['ffmpeg'] == row(fx.ffmpeg), 'summary binds the command run')
    check(proof['wav_samples'] == fx.total * 16000 // 60 and proof['verification_sha256'] == sha(fx.jobs / V / 'verification.json')
          and proof['postcheck_sha256'] == sha(fx.stage / f'{V}-postcheck.json') and proof['script_sha256'] == sha(fx.script), 'summary binds root\'s Mail 5560 list')
    want = f'ASR AUDIO PASS: {A} audio-summary.json sha256 {sha(summary)}; transfer it, the WAV and script.tsv to the ASR host.'
    check(text.splitlines()[-1] == want, 'the line root carries: ' + text.splitlines()[-1])
    bound = {x['path'] for x in json.loads((fx.O / 'asr-audio-inputs01.json').read_text())['files']}
    for path in (movie, fx.stage / f'{V}-postcheck.json', fx.jobs / V / 'verification.json', fx.jobs / M / 'master-summary.json', fx.ffmpeg,
                 fx.work / '15_closing.timeline.tsv', fx.jobs / 'complex-remote-narration-derive02/narration-n2/15_1.wav', fx.logical / REL_SCRIPT, fx.logical / REL_Q):
        check(str(path) in bound, f'ts1 manifest binds {path.name}')
    check((fx.O / 'asr-audio01.exit').read_text() == '0\n', 'ts1 exit file')
    code, text = mini(fx, A, sha(summary), '01')
    check(code == 0, 'mini step: ' + text[-500:])
    seen = events(fx)[2:]
    check([x['kind'] for x in seen] == ['preflight', 'run'] and {x['lane'] for x in seen} == {'remote-mac-mini-max01'}, 'mini order')
    check(seen[1]['cwd'] == seen[1]['pwd'] == str(fx.repo), 'mini physical cwd')
    check(seen[1]['flags'] == ['--phase', 'asr', '--job', 'complex-remote-asr01', '--inputs', str(fx.lane / 'complex-asr-inputs01.json')], 'mini guard line')
    check(seen[1]['command'] == [sys.executable, '-I', '-B', str(fx.bundle / 'mini_asr01.py'), '--run', A, sha(summary), '01'], 'mini command')
    wav = fx.audio / 'encoded-master-16k-mono.wav'
    check((fx.stand / 'fw.argv').read_text().splitlines() == ['transcribe', '--input', str(wav), '--backend', 'whisper-cpp', '--model', str(fx.stand / 'model.bin'),
          '--language', 'en', '--no-diarize', '--no-persist', '--split-on-word', '--max-segment-length', '1', '--threads', '4', '--timeout', '7200', '--json'], 'fw arguments')
    out = fx.lane_jobs / 'complex-remote-asr01'
    comparison = json.loads((out / 'comparison.json').read_text())
    check(comparison['master_sha256'] == sha(movie) and comparison['wav_sha256'] == sha(wav) and comparison['cue_count'] == 105, 'comparison')
    check(comparison['script_sha256'] == sha(fx.script) and comparison['expected_words'] == 4662, 'expected words, real script')
    found = [x['word'] for change in comparison['differences'] for x in change['recognized']]
    check('nonsense' in found and '-' in found, f'word and sign leads: {found[:12]}')
    boundaries = json.loads((out / 'cue-boundaries.json').read_text())
    windows = [w for cue in boundaries['cues'] for w in cue['windows']]
    check(boundaries['cue_count'] == 105 and len(windows) == 210, 'all cue edges')
    joins = [c for a, c in zip(boundaries['cues'], boundaries['cues'][1:]) if a['chapter'] != c['chapter']]
    check(len(joins) == 14 and all(c['previous_cue'] for c in joins), 'chapter joins')
    bound = {x['path'] for x in json.loads((fx.lane / 'complex-asr-inputs01.json').read_text())['files']}
    for path in (wav, summary, fx.script, fx.stand / 'fw', fx.stand / 'model.bin', fx.lane / 'asr-source-bindings01.json', fx.lane / 'tools01.json'):
        check(str(path) in bound, f'mini manifest binds {path.name}')
    return (f'ts1 decodes and prints the summary hash; the mini recognizes from that hash. Every decode and fw argument exact; 105 cues, 210 windows, '
            f'14 chapter joins; a changed word and a minus sign reach the differences; {len(bound)} mini rows bound')


def good_retry(base):
    fx = build(base)
    for name in (V, M):
        key(fx.jobs / name / 'resource-receipt.json', 'exit', value=1)
    film(fx, V + '-r2', M + '-r2')
    code, text = ts1(fx, V + '-r2', M + '-r2', '02')
    check(code == 0 and (fx.jobs / 'complex-remote-asr-audio02/audio-summary.json').is_file(), 'retry: ' + text[-400:])
    summary = fx.repo / REL_O / 'jobs/complex-remote-asr-audio02/audio-summary.json'
    code, text = mini(fx, 'complex-remote-asr-audio02', sha(summary), '02')
    check(code == 0 and (fx.lane_jobs / 'complex-remote-asr02/comparison.json').is_file(), 'retry mini: ' + text[-400:])
    return 'final-verify01-r2 of master01-r2 reaches ASR as attempt 02, beside the failed attempts, reading its own post-check'


def good_race(base):
    """The guard's live receipt reads as partial JSON for a second as each job starts."""
    fx = build(base)
    write(fx.base / 'guard-partial', 'moment')
    code, text = ts1(fx)
    check(code == 0, 'race ts1: ' + text[-400:])
    code, text = mini(fx)
    check(code == 0, 'race mini: ' + text[-400:])
    return 'a partial live receipt for a second, on both hosts: both jobs run'


def rec(f, name):
    return f.jobs / name / 'resource-receipt.json'


def proof(f):
    return f.jobs / V / 'verification.json'


def summary(f):
    return f.jobs / M / 'master-summary.json'


def post(f):
    return f.stage / f'{V}-postcheck.json'


def inputs(f, name):
    return f.stage / f'{name}-inputs.json'


def swap(f):
    edit(proof(f), lambda d: d['placements'].__setitem__(slice(0, 2), d['placements'][1::-1]))


def shift(f, field, value):
    edit(proof(f), lambda d: d['placements'][5].__setitem__(field, value))


def rewrite_table(f, old, new):
    path = f.work / '02_turning_numbers.timeline.tsv'
    text = path.read_text()
    assert old in text, old
    path.write_text(text.replace(old, new))


def fewer_chapters(f):
    edit(f.jobs / 'complex-remote-timeline03/timeline-summary.json', lambda d: d['chapters'].pop())


def other_ffmpeg(f):
    with f.ffmpeg.open('a') as handle:
        handle.write('# other bytes\n')


# Each plant is the failure itself; every record it does not name stays good.
TS1_CASES = [
    ('verify job failed', lambda f: key(rec(f, V), 'exit', value=1), f'AssertionError: {V}'),
    ('verify stopped for resources', lambda f: key(rec(f, V), 'stopped_for_resource_limit', value=True), f'AssertionError: {V}'),
    ('verify under another policy', lambda f: key(rec(f, V), 'policy', 'work_cap_gib', value=48), 'Guard policy changed'),
    ('verify in another phase', lambda f: key(rec(f, V), 'phase', value='asr'), 'ran in phase'),
    ('verify under ts1 python, not the picture env', lambda f: key(rec(f, V), 'command', 0, value=sys.executable), 'ran another command'),
    ('verify inputs moved', lambda f: js(inputs(f, V), {'files': []}), 'frozen inputs moved'),
    ('verify helper moved', lambda f: write(f.final / 'ts1_final_verify01.py', '# other\n'), 'its helper moved'),
    ('verify log ends on a near-match', lambda f: write(f.jobs / V / 'command.log', 'FINAL VERIFY PASS: but not the line\n'), 'success line'),
    ('master job failed', lambda f: key(rec(f, M), 'exit', value=75), f'AssertionError: {M}'),
    ('master ran another command', lambda f: key(rec(f, M), 'command', 4, value='--other'), 'ran another command'),
    ('master log lacks its success line', lambda f: write(f.jobs / M / 'command.log', 'work\n'), 'success line'),
    ('later master attempt ignored', lambda f: (f.jobs / (M + '-r2')).mkdir(), 'not the newest'),
    ('later verify attempt ignored', lambda f: (f.jobs / (V + '-r2')).mkdir(), 'not the newest'),
    ('proof of another master job', lambda f: key(proof(f), 'master_job', value=M + '-r9'), 'the proof is for'),
    ('proof claims a viewing', lambda f: key(proof(f), 'pictures_viewed', value=True), 'picture viewing'),
    ('proof from another host', lambda f: key(proof(f), 'host', value='mac-mini-max'), 'another host or voice'),
    ('proof with another voice', lambda f: key(proof(f), 'voice', value='ElevenLabs Oliver'), 'another host or voice'),
    ('proof from another binary', lambda f: key(proof(f), 'binary_sha256', value='0' * 64), 'another binary'),
    ('build02 has not finished', lambda f: (f.jobs / 'complex-remote-build02/binary.sha256').unlink(), 'another binary'),
    ('proof on the older timeline', lambda f: key(proof(f), 'timeline_job', value='complex-remote-timeline02'), 'another timeline or narration'),
    ('proof on N, not N2', lambda f: key(proof(f), 'narration', value='N'), 'another timeline or narration'),
    ('proof names another master file', lambda f: key(proof(f), 'video', value='/other.mp4'), 'another master file'),
    ('proof with another length', lambda f: key(proof(f), 'frames', value=1), 'another length'),
    ('proof with other chapters', lambda f: edit(proof(f), lambda d: d['chapters'].reverse()), 'other chapters'),
    ('proof with two cues swapped', swap, 'other placements'),
    ('proof with one start moved', lambda f: shift(f, 'start_frame', 61), 'other placements'),
    ('proof with one duration changed', lambda f: shift(f, 'duration_s', 9.0), 'other placements'),
    ('final table differs from the timeline', lambda f: rewrite_table(f, 'line\t02_2\t12', 'line\t02_2\t13'), 'differs from the timeline'),
    ('final and timeline tables name N', lambda f: [rewrite_table(f, 'narration-n2/02_1.wav', 'narration01/02_1.wav'),
                                                    literal(f.jobs / 'complex-remote-timeline03/timeline-output/02_turning_numbers.timeline.tsv', 'narration-n2/02_1.wav', 'narration01/02_1.wav'),
                                                    edit(f.jobs / 'complex-remote-timeline03/timeline-summary.json', lambda d: d['chapters'][1].update(timeline_sha256=hashlib.sha256((f.jobs / 'complex-remote-timeline03/timeline-output/02_turning_numbers.timeline.tsv').read_bytes()).hexdigest()))],
     'another narration file'),
    ('timeline table moved', lambda f: key(f.jobs / 'complex-remote-timeline03/timeline-summary.json', 'chapters', 3, 'timeline_sha256', value='0' * 64), 'timeline table moved'),
    ('only 14 chapters', fewer_chapters, 'not the 15 registered chapters'),
    ('strict check log not passed', lambda f: write(f.jobs / V / 'strict-video-verification.log', 'VERIFY FAIL\n'), 'strict video check'),
    ('master replaced after verify', lambda f: write(f.work / 'complex_analysis_visual_master.mp4', 'other\n'), 'changed after verification'),
    ('summary of another master', lambda f: key(summary(f), 'outputs', 'master', 'sha256', value='0' * 64), 'another master'),
    ('summary names another file', lambda f: key(summary(f), 'outputs', 'master', 'path', value='/other.mp4'), 'another master'),
    ('captions replaced after verify', lambda f: write(f.work / 'complex_analysis_visual.en.srt', 'other\n'), 'other captions'),
    ('web captions replaced after verify', lambda f: write(f.work / 'complex_analysis_visual.en.vtt', 'other\n'), 'other captions'),
    ('summary of another length', lambda f: key(summary(f), 'frames', value=1), 'another film'),
    ('no post-check record', lambda f: post(f).unlink(), 'no verify post-check record'),
    ('post-check of another verify job', lambda f: key(post(f), 'job', value=V + '-r9'), 'for another job'),
    ('post-check of another master job', lambda f: key(post(f), 'master_job', value=M + '-r9'), 'for another job'),
    ('proof changed after the post-check', lambda f: key(proof(f), 'closing', value='forged'), 'read another proof'),
    ('post-check of another master', lambda f: key(post(f), 'master_sha256', value='0' * 64), 'read another master'),
    ('post-check of another length', lambda f: key(post(f), 'frames', value=1), 'read another master'),
    ('post-check of other captions', lambda f: key(post(f), 'srt_sha256', value='0' * 64), 'read other captions'),
    ('post-check of other web captions', lambda f: key(post(f), 'vtt_sha256', value='0' * 64), 'read other captions'),
    ('post-check claims a viewing', lambda f: key(post(f), 'pictures_viewed', value=True), 'claims a picture viewing'),
    ('script one comma different', lambda f: write(f.script, f.script.read_text().replace(',', '', 1)), 'not the frozen Frederick script'),
    ('ffmpeg is not the verify job\'s bytes', other_ffmpeg, "ffmpeg is not the verify job's tool"),
    ('sbh off PATH', lambda f: (f.bin / 'sbh').unlink(), 'sbh is not on PATH'),
    ('ffmpeg off PATH', lambda f: (f.bin / 'ffmpeg').unlink(), 'ffmpeg is not on PATH'),
    ('preflight not admitted', lambda f: write(f.base / 'guard-refuses', ''), None),
]


def forge(f, change):
    """A summary edited after ts1: a fresh hash, so only the field itself can refuse it."""
    edit(f.audio / 'audio-summary.json', change)


def other_binding(f):
    """Bindings ffmpeg differs from tools ffmpeg, under pins that still match."""
    edit(f.lane / 'asr-source-bindings01.json', lambda d: d['files'][3].update(sha256='0' * 64))
    repin(f)


MINI_CASES = [
    ('no transferred summary', lambda f: (f.audio / 'audio-summary.json').unlink(), 'no transferred summary'),
    ('summary is not the one ts1 printed', lambda f: key(f.audio / 'audio-summary.json', 'closing', value='x'), 'not the summary ts1 printed'),
    ('WAV changed in transfer', lambda f: write(f.audio / 'encoded-master-16k-mono.wav', 'other\n'), 'changed in transfer'),
    ('script one comma different', lambda f: write(f.script, f.script.read_text().replace(',', '', 1)), 'not the frozen Frederick script'),
    ('root bindings changed', lambda f: key(f.lane / 'asr-source-bindings01.json', 'fw_version', value='0'), "not root's verified ASR bindings"),
    ('worker tools changed', lambda f: key(f.lane / 'tools01.json', 'host', value='yi-ap'), 'not the worker tools file'),
    ('bindings ffmpeg is not the pinned tool', other_binding, 'ASR ffmpeg is not the pinned tool'),
    ('fw bytes changed', lambda f: write(f.stand / 'fw', '#!/bin/sh\n', 0o755), str(Path('stand-ins/fw'))),
    ('model bytes changed', lambda f: write(f.stand / 'model.bin', 'other\n'), str(Path('stand-ins/model.bin'))),
    ('sbh off PATH', lambda f: [(f.bin / 'sbh').unlink(), literal(f.bundle / 'start-asr01.sh', ':/opt/homebrew/bin:', ':')], 'sbh is not on PATH'),
]
MINI_FORGED = [
    ('summary of another job', lambda d: d.update(job='complex-remote-asr-audio09'), 'for another job'),
    ('summary from another host', lambda d: d.update(host='mac-mini-max'), 'for another job'),
    ('summary with another voice', lambda d: d.update(voice='ElevenLabs Oliver'), 'for another film'),
    ('summary claims a viewing', lambda d: d.update(pictures_viewed=True), 'for another film'),
    ('summary of another script', lambda d: d.update(script_sha256='0' * 64), 'not the frozen Frederick script'),
    ('summary at another rate', lambda d: d.update(fps=30), 'another length'),
    ('summary with 14 chapters', lambda d: d['chapters'].pop(), 'not the 15 registered chapters'),
    ('summary frames disagree', lambda d: d.update(frames=1), 'another length'),
    ('summary names another WAV', lambda d: d.update(wav='/elsewhere/other.wav'), 'another WAV'),
    ('placements swapped', lambda d: d['placements'].__setitem__(slice(0, 2), d['placements'][1::-1]), 'does not place the 105 cues'),
    ('one placement missing', lambda d: d['placements'].pop(), 'does not place the 105 cues'),
]


def no_job(fx, label):
    check(not (fx.jobs / A).exists() and not (fx.O / 'asr-audio-inputs01.json').exists(), f'{label}: ts1 job name or manifest used')


def no_asr(fx, label):
    check(not (fx.lane_jobs / 'complex-remote-asr01').exists() and not (fx.lane / 'complex-asr-inputs01.json').exists(), f'{label}: mini job name or manifest used')


def said(text, fx):
    last = [x for x in text.splitlines() if x.strip()][-1]
    return last.replace(str(fx.base), 'SCRATCH')


def refusals_ts1(base):
    for n, (name, plant, words) in enumerate(TS1_CASES):
        fx = build(base / f'ts1{n:02d}')
        plant(fx)
        code, text = ts1(fx)
        check(code != 0, f'{name}: accepted')
        check(words is None or words in text, f'{name}: {text[-400:]}')
        no_job(fx, name)
        print(f'PASS ts1  {name}: {"guard exit 75" if words is None else said(text, fx)}')
    fx = build(base / 'used')
    (fx.jobs / A).mkdir()
    code, text = ts1(fx)
    check(code == 2 and 'use the next attempt number' in text and not events(fx), 'used attempt')
    fx = build(base / 'order')
    key(rec(fx, V), 'exit', value=1)
    ts1(fx)
    check(not events(fx) and (fx.O / 'asr-audio01.exit').read_text() == '1\n', 'failed proof reaches no guard')
    fx = build(base / 'order2')
    write(fx.base / 'guard-refuses', '')
    ts1(fx)
    check([x['kind'] for x in events(fx)] == ['preflight'] and (fx.O / 'asr-audio01.exit').read_text() == '75\n', 'preflight stops launch')
    print('PASS ts1  a used attempt refuses before any step; a failed gate reaches no guard; a refused preflight reaches no launch')
    for n, jobs in enumerate(((V, 'complex-stage06-optimized-master01', '01'), ('complex-stage06-optimized-final-verify01', M, '01'), (V, M, '00'))):
        fx = build(base / f'name{n}')
        for code, text in (ts1(fx, *jobs), direct(fx, 'ts1_asr_audio01.py', *jobs)):
            check(code != 0 and 'bad ' in text and not events(fx), f'bad name {jobs}')
    print('PASS ts1  bad master name, bad verify name and attempt 00: wrapper and helper both refuse')
    fx = build(base / 'decode-shape')
    write(fx.bin / 'ffmpeg.plan', f'48000 2 {fx.total * 800}\n')
    code, text = ts1(fx)
    log = (fx.jobs / A / 'command.log').read_text()
    check(code != 0 and 'not 16 kHz mono PCM' in log and not (fx.jobs / A / 'audio-summary.json').exists(), 'decode shape')
    fx = build(base / 'decode-short')
    write(fx.bin / 'ffmpeg.plan', f'16000 1 {fx.total * 16000 // 60 - 4000}\n')
    code, text = ts1(fx)
    log = (fx.jobs / A / 'command.log').read_text()
    check(code != 0 and 'the decode lasts' in log and not (fx.jobs / A / 'audio-summary.json').exists(), 'short decode')
    print('PASS ts1  a decode that is not 16 kHz mono PCM, or 0.25 s short, fails its job and writes no summary')
    fx = build(base / 'run')
    args = ('--run', V, M, '01')
    code, text = direct(fx, 'ts1_asr_audio01.py', *args)
    check(code != 0 and 'FileNotFoundError' in text and not (fx.bin / 'ffmpeg.argv').exists(), '--run outside the guard')
    live = fx.jobs / A / 'resource-receipt.json'
    js(live, {'admitted': True, 'phase': 'asr', 'inputs_sha256': 'x', 'command': []})
    js(fx.O / 'asr-audio-inputs01.json', {'files': []})
    code, text = direct(fx, 'ts1_asr_audio01.py', *args)
    check(code != 0 and 'not this guarded job' in text and not (fx.bin / 'ffmpeg.argv').exists(), '--run with a foreign receipt')
    key(live, 'inputs_sha256', value=sha(fx.O / 'asr-audio-inputs01.json'))
    key(live, 'command', value=[sys.executable, '-I', '-B', str(fx.bundle_logical / 'ts1_asr_audio01.py'), '--run', V, M, '02'])
    code, text = direct(fx, 'ts1_asr_audio01.py', *args)
    check(code != 0 and 'the guard ran another command' in text and not (fx.bin / 'ffmpeg.argv').exists(), '--run under another selection')
    fx = build(base / 'policy')
    write(fx.base / 'guard-policy', '')
    code, text = ts1(fx)
    check(code != 0 and 'Guard policy changed' in (fx.jobs / A / 'command.log').read_text() and not (fx.bin / 'ffmpeg.argv').exists(), 'phase policy')
    print('PASS ts1  --run outside the guard, under a foreign receipt, another selection or another policy refuses before any decode')
    fx = build(base / 'unsettled')
    write(fx.base / 'guard-partial', 'forever')
    code, text = ts1(fx)
    check(code != 0 and 'never settled' in (fx.jobs / A / 'command.log').read_text() and not (fx.bin / 'ffmpeg.argv').exists(), 'unsettled')
    print('PASS ts1  a live receipt that never parses refuses after the bounded reread, before any decode')


def after_ts1(base):
    fx = build(base)
    code, text = ts1(fx)
    check(code == 0, 'setup: ' + text[-300:])
    return fx


def refusals_mini(base):
    for n, (name, plant, words) in enumerate(MINI_CASES):
        fx = after_ts1(base / f'mini{n:02d}')
        good = sha(fx.audio / 'audio-summary.json')
        plant(fx)
        code, text = mini(fx, A, good, '01')
        check(code != 0, f'{name}: accepted')
        check(words in text, f'{name}: {text[-400:]}')
        no_asr(fx, name)
        print(f'PASS mini {name}: {said(text, fx)}')
    for n, (name, change, words) in enumerate(MINI_FORGED):
        fx = after_ts1(base / f'forged{n:02d}')
        forge(fx, change)
        code, text = mini(fx)
        check(code != 0 and words in text, f'{name}: {text[-400:]}')
        no_asr(fx, name)
        print(f'PASS mini {name}: {said(text, fx)}')
    fx = after_ts1(base / 'interpreter')
    key(fx.lane / 'tools01.json', 'python3', 'path', value='/usr/bin/true')
    repin(fx)
    code, text = direct(fx, 'mini_asr01.py', A, sha(fx.audio / 'audio-summary.json'), '01')
    check(code != 0 and 'not the pinned interpreter' in text, 'interpreter')
    fx = after_ts1(base / 'interpreter2')
    key(fx.lane / 'tools01.json', 'python3', 'sha256', value='0' * 64)
    repin(fx)
    code, text = direct(fx, 'mini_asr01.py', A, sha(fx.audio / 'audio-summary.json'), '01')
    check(code != 0 and 'pinned interpreter changed' in text, 'interpreter bytes')
    no_asr(fx, 'interpreter')
    print('PASS mini another interpreter, or changed interpreter bytes: refused')
    fx = after_ts1(base / 'used2')
    (fx.lane_jobs / 'complex-remote-asr01').mkdir(parents=True)
    code, text = mini(fx)
    check(code == 2 and 'use the next attempt number' in text, 'used attempt')
    fx = after_ts1(base / 'names')
    good = sha(fx.audio / 'audio-summary.json')
    for args in (('complex-stage06-final-asr01', good, '01'), (A, 'x' * 64, '01'), (A, good, '00')):
        for code, text in (mini(fx, *args), direct(fx, 'mini_asr01.py', *args)):
            check(code != 0 and 'bad ' in text, f'bad name {args}')
    print('PASS mini a used attempt, a bad audio job, a bad hash and attempt 00 refuse')
    fx = after_ts1(base / 'run2')
    good = sha(fx.audio / 'audio-summary.json')
    args = ('--run', A, good, '01')
    code, text = direct(fx, 'mini_asr01.py', *args)
    check(code != 0 and 'FileNotFoundError' in text and not (fx.stand / 'fw.argv').exists(), '--run outside the guard')
    live = fx.lane_jobs / 'complex-remote-asr01/resource-receipt.json'
    js(live, {'admitted': True, 'phase': 'asr', 'inputs_sha256': 'x', 'command': []})
    js(fx.lane / 'complex-asr-inputs01.json', {'files': []})
    code, text = direct(fx, 'mini_asr01.py', *args)
    check(code != 0 and 'not this guarded ASR job' in text and not (fx.stand / 'fw.argv').exists(), '--run with a foreign receipt')
    key(live, 'inputs_sha256', value=sha(fx.lane / 'complex-asr-inputs01.json'))
    key(live, 'command', value=[sys.executable, '-I', '-B', str(fx.bundle / 'mini_asr01.py'), '--run', A, '0' * 64, '01'])
    code, text = direct(fx, 'mini_asr01.py', *args)
    check(code != 0 and 'the guard ran another command' in text and not (fx.stand / 'fw.argv').exists(), '--run under another summary hash')
    print('PASS mini --run outside the guard, under a foreign receipt or under another summary hash refuses before fw')


def nodes(path):
    return {n.name: n for n in ast.parse(path.read_text()).body if isinstance(n, ast.FunctionDef)}


def tail(function):
    at = [i for i, s in enumerate(function.body) if ast.unparse(s).startswith('wav = OUT /')]
    assert len(at) == 1
    return [ast.dump(s) for s in function.body[at[0]:]]


def from_statement(function, prefix, count=None):
    at = [i for i, s in enumerate(function.body) if ast.unparse(s).startswith(prefix)]
    assert len(at) == 1, prefix
    body = function.body[at[0]:]
    return [ast.dump(s) for s in (body[:count] if count else body)]


def sources():
    bundle = R / REL_P / 'logarithm-remote-asr02'
    check(sha(bundle / 'SHA256SUMS') == ASR02_SEAL, 'ASR02 seal')
    root = R / REL_EC / 'final-asr-stage06-01/after-verify01.py'
    check(sha(root) == ROOT_COMPLEX, 'root complex helper')
    old, film_root = nodes(bundle / 'asr02.py'), nodes(root)
    step1, step2 = nodes(HERE / 'ts1_asr_audio01.py'), nodes(HERE / 'mini_asr01.py')
    kept = ('read', 'sha', 'check_files', 'script_rows', 'tokens', 'boundary_review')
    check(all(ast.dump(film_root[n]) == ast.dump(step2[n]) for n in kept), 'kept functions against root')
    same = ('read', 'sha', 'check_files', 'tokens', 'boundary_review', 'live_receipt')
    check(all(ast.dump(old[n]) == ast.dump(step2[n]) for n in same), 'kept functions against ASR02')
    check(ast.dump(old['live_receipt']) == ast.dump(step1['live_receipt']) and ast.dump(old['check_files']) == ast.dump(step1['check_files']), 'step 1 kept functions')
    print(f'PASS keep {", ".join(kept)} equal root\'s complex after-verify01.py by AST, with its 105-cue count; {", ".join(same)} equal sealed asr02.py')
    decode = tail(old['run'])
    first = from_statement(step1['run'], 'wav = OUT /', 2)
    rest = from_statement(step2['run'], 'command = [fw,')
    check(decode[:2] == first, 'decode statements')
    check(decode[2:] == rest, 'fw and comparison statements')
    print(f'PASS keep asr02.py\'s run() from the decode on is exactly step 1\'s {len(first)} statements then step 2\'s {len(rest)}, by AST: decode, fw, timeouts, outputs, scopes')
    text = (R / REL_SCRIPT).read_text()
    rule = r"[a-z]+(?:'[a-z]+)?|\d+(?:\.\d+)?|[-+/%^π]"
    rows = [line.split('\t', 1)[1] for line in text.splitlines() if line.strip() and not line.startswith('#')]
    lost = set(''.join(re.sub(rule, '', x.lower().replace('’', "'").replace('−', '-')) for x in rows).replace(' ', ''))
    check(lost == {'.', ',', '?'}, f'token rule loses {lost}')
    print('PASS sign the real script: the token rule loses only . , and ?, so no mathematical sign')
    peers()
    for name in ('start-asr-audio01.sh', 'start-asr01.sh'):
        check(subprocess.run(['bash', '-n', str(HERE / name)]).returncode == 0, 'bash -n ' + name)
    print('PASS sh   bash -n, both wrappers')


def peers():
    chain = R / REL_P / 'complex-remote-ts1-pipeline01'
    check(sha(chain / 'chain03/SHA256SUMS') == CHAIN_SEAL, 'chain seal')
    check(sha(chain / 'qualify02/SHA256SUMS') == QUALIFY_SEAL, 'qualifier seal')
    body = (chain / 'chain03/ts1_chain03.py').read_text()
    check("DERIVE = 'complex-remote-narration-derive02'" in body and "N = JOBS / DERIVE / 'narration-n2'" in body
          and "'timeline': 'complex-remote-timeline03'" in body, 'chain names')
    check("PICTURE_ENV = 'complex-remote-picture-env01'" in body, 'picture env')
    step1 = (HERE / 'ts1_asr_audio01.py').read_text()
    constant = lambda name: re.search(rf"^{name} = '(.*)'$", step1, re.M).group(1)
    check(constant('QUALIFIER_SHA256') in body and constant('QUALIFIER_SHA256') in (chain / 'qualify02/SHA256SUMS').read_text(), 'chain and seal pin my qualifier')
    print(f'PASS peer White\'s seals {CHAIN_SEAL[:12]} and {QUALIFY_SEAL[:12]}: the qualifier, N2, timeline03 and the picture env agree (source text, not run)')
    final = R / REL_P / 'complex-remote-final-production01'
    master, verify = [(final / name).read_text() for name in ('ts1_final_master01.py', 'ts1_final_verify01.py')]
    render = (final / 'ts1_final_render01.py').read_text()
    for body, spelled in ((master, ["JOB = 'complex-remote-master01'", f"SUCCESS = '{constant('MASTER_SUCCESS')}'",
                                    "return ['-I', '-B', str(ME), '--run', name]", "[sys.executable, *child_tail(name)]",
                                    "('master', '_master.mp4'), ('srt', '.en.srt'), ('vtt', '.en.vtt')",
                                    "F.OUT / ('complex_analysis_visual' + suffix)", "F.STAGE / (name + '-inputs.json')"]),
                          (verify, ["JOB = 'complex-remote-final-verify01'", f"SUCCESS = '{constant('VERIFY_SUCCESS')}'",
                                    "return [str(M.PICTURE_PYTHON), '-I', '-B', str(ME), '--run', name]",
                                    "F.STAGE / (name + '-inputs.json')", "F.STAGE / (name + '-postcheck.json')"]),
                          (render, ["STAGE = O / 'final-production01'", "OUT = JOBS / 'complex-remote-render-work01'"])):
        for text in spelled:
            check(text in body, 'Lavender changed: ' + text)
    ending = 'print(SUCCESS, flush=True)'
    check(all(ast.unparse(nodes(final / name)['run'].body[-1]) == ending for name in ('ts1_final_master01.py', 'ts1_final_verify01.py')), 'success line not last in the child')
    print('PASS peer Lavender\'s helpers: job names, both success lines, child commands, output, manifest and post-check names; each child prints its line last (source text, not run)')


def main():
    base = Path(sys.argv[1]).resolve()
    base.mkdir()
    sources()
    print('PASS good ' + good_film(base / 'good'))
    print('PASS good ' + good_retry(base / 'retry'))
    print('PASS good ' + good_race(base / 'race'))
    refusals_ts1(base)
    refusals_mini(base)
    print('ALL PASS. Stand-ins only: nothing ran on ts1 or mac-mini-max, and fw transcribe, ffmpeg, both real guards and the real final jobs did not run.')


if __name__ == '__main__':
    main()
