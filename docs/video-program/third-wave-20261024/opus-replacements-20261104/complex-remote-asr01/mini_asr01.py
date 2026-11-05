"""Step 2 of 2 on mac-mini-max: recognize the transferred audio of the final complex master, preserving discrepancies.

asr02.py from `command = [fw, ...` to the end, by AST: the same engine, model, settings, timeouts, guard phase, cue-edge
windows and token rule. Only the input differs: the WAV that ts1_asr_audio01.py decoded on ts1, named by the hash of its
audio-summary.json, because the master itself never reaches this host. Start through start-asr01.sh.

    mini_asr01.py --check AUDIO_JOB SUMMARY_SHA256 NN    read-only: every gate, no job name used
    mini_asr01.py AUDIO_JOB SUMMARY_SHA256 NN            freeze the manifest and run this lane's guard
    mini_asr01.py --run AUDIO_JOB SUMMARY_SHA256 NN      the guard's child; never call it by hand

To move recognition to a host with its own fw, only LANE and the two pins below would change.
ASR gives leads for triage. It isn't listening, fine sync or AV acceptance.
"""
import difflib
import hashlib
import json
import math
from pathlib import Path
import re
import subprocess
import sys
import time

HERE = Path(__file__).resolve().parent
R = HERE.parents[4]
LANE = 'narration_lab/third-wave-20261024/root/elevenlabs-logarithm-rebuild-20261104/remote-mac-mini-max01'
P = R / LANE
B = P / 'jobs'
GUARD = P / 'guard01.py'
SOURCES = P / 'asr-source-bindings01.json'
SOURCES_SHA256 = '71e140bbc3bdd218b9c11e407d9e572118fca9e5d25d2bf5d1c82b0beb556a5e'  # root's, confirmed remotely in Mail 5503
TOOLS = P / 'tools01.json'
TOOLS_SHA256 = '188b76509f7b3024fe4577d8d01ff7ca396375ca831a2294f7c4dc3bb5497c2c'
EC = R / 'narration_lab/third-wave-20261024/root/elevenlabs-complex-rebuild-20261104'
AUDIO_JOBS = EC / 'remote-ts1-01/jobs'
SCRIPT = EC / 'native-stage06-root03/workspace/explainer_corpus/complex_analysis_visual/script.tsv'
SCRIPT_SHA256 = '0bae7acb15f2f0453c8a25ba7110fedd65cf9156eba932143e08db7fdc8bfeee'
AUDIO_NAME = r'complex-remote-asr-audio\d\d'
CHAPTERS = ('01_the_bump', '02_turning_numbers', '03_two_planes', '04_reading_color',
            '05_gallery', '06_one_derivative', '07_one_point', '08_loop_sums', '09_boundary',
            '10_rigidity', '11_counting', '12_residues', '13_real_integral', '14_continuation', '15_closing')
VOICE = 'ElevenLabs Frederick Surrey'


def select(audio_job, summary_sha256, attempt):
    """The explicit selection, set once. Root carries the summary hash from ts1's last log line."""
    global AUDIO_JOB, SUMMARY_SHA256, ATTEMPT, AUDIO, WAV, JOB, OUT, MANIFEST
    assert re.fullmatch(AUDIO_NAME, audio_job), f'bad audio job {audio_job}'
    assert re.fullmatch(r'[0-9a-f]{64}', summary_sha256), f'bad summary hash {summary_sha256}'
    assert re.fullmatch(r'\d\d', attempt) and int(attempt) >= 1, f'bad attempt {attempt}'
    AUDIO_JOB, SUMMARY_SHA256, ATTEMPT = audio_job, summary_sha256, attempt
    AUDIO = AUDIO_JOBS / audio_job
    WAV = AUDIO / 'encoded-master-16k-mono.wav'
    JOB = f'complex-remote-asr{attempt}'
    OUT = B / JOB
    MANIFEST = P / f'complex-asr-inputs{attempt}.json'


def child_command():
    return [sys.executable, '-I', '-B', str(Path(__file__).resolve()), '--run', AUDIO_JOB, SUMMARY_SHA256, ATTEMPT]


def bound_tools():
    """fw, the model and the interpreter are root's verified worker tools, by hash."""
    assert sha(SOURCES) == SOURCES_SHA256, 'not root\'s verified ASR bindings'
    assert sha(TOOLS) == TOOLS_SHA256, 'not the worker tools file'
    tools = read(TOOLS)
    sources = read(SOURCES)['files']
    assert sources[3]['sha256'] == tools['ffmpeg']['sha256'], 'ASR ffmpeg is not the pinned tool'
    python = tools['python3']
    assert Path(sys.executable).resolve() == Path(python['path']).resolve(), 'this is not the pinned interpreter'
    assert sha(sys.executable) == python['sha256'], 'the pinned interpreter changed'
    check_files(sources[1:])
    return sources


def bound_proof():
    """ts1's checks end in one file. Its hash is the whole link between that host and this one."""
    summary = AUDIO / 'audio-summary.json'
    assert summary.is_file(), f'no transferred summary: {summary.name}'
    assert sha(summary) == SUMMARY_SHA256, 'this is not the summary ts1 printed'
    proof = read(summary)
    assert proof['job'] == AUDIO_JOB and proof['host'] == 'ts1', 'the summary is for another job'
    assert proof['voice'] == VOICE and proof['pictures_viewed'] is False, 'the summary is for another film'
    assert sha(SCRIPT) == SCRIPT_SHA256 == proof['script_sha256'], 'script.tsv is not the frozen Frederick script'
    assert proof['fps'] == 60 and isinstance(proof['frames'], int) and proof['frames'] > 0, 'the summary has another length'
    assert tuple(x['chapter'] for x in proof['chapters']) == CHAPTERS, 'not the 15 registered chapters'
    assert sum(x['frames'] for x in proof['chapters']) == proof['frames'], 'the summary has another length'
    assert proof['wav'].endswith('/' + str(WAV.relative_to(R))), 'the summary names another WAV'
    assert sha(WAV) == proof['wav_sha256'], 'the WAV changed in transfer'
    return proof


def read(p):
    return json.loads(p.read_text())


def sha(p):
    with Path(p).open('rb') as handle:
        return hashlib.file_digest(handle, 'sha256').hexdigest()


def live_receipt():
    """This job's receipt while the guard owns it. The guard rewrites it in place just after starting the job, so a
    read can land inside that write. A receipt that never parses refuses."""
    for _ in range(100):
        try:
            return read(OUT / 'resource-receipt.json')
        except json.JSONDecodeError:
            time.sleep(0.1)
    raise RuntimeError('The guard receipt never settled')


def check_files(rows):
    for row in rows:
        assert sha(row['path']) == row['sha256'], row['path']


def script_rows(path):
    rows = [line.split('\t', 1) for line in path.read_text().splitlines()
            if line.strip() and not line.startswith('#')]
    assert len(rows) == 105 and len({x[0] for x in rows}) == 105
    assert all(re.fullmatch(r'\d\d_\d+', x[0]) and x[1].strip() for x in rows)
    return rows


def tokens(text):
    # Preserve mathematical signs; no automatic acceptance via normalization.
    return re.findall(r"[a-z]+(?:'[a-z]+)?|\d+(?:\.\d+)?|[-+/%^π]",
                      text.lower().replace('’', "'").replace('−', '-'))


def boundary_review(proof, rows, segments):
    """Time-window leads at every authored cue edge, not a forced alignment."""
    placements = proof['placements']
    assert [p['cue'] for p in placements] == [cue for cue, text in rows]
    total = proof['frames'] / proof['fps']
    for segment in segments:
        a, b = segment['start_sec'], segment['end_sec']
        assert math.isfinite(a) and math.isfinite(b) and 0 <= a <= b <= total + 1
    result = []
    for i, (placement, (cue, text)) in enumerate(zip(placements, rows)):
        start = placement['start_frame'] / proof['fps']
        end = start + placement['duration_s']
        assert 0 <= start < end <= total
        windows = []
        for edge, left, right in [('opening', max(0, start-.5), min(end, start+5)),
                                  ('closing', max(start, end-5), min(total, end+.5))]:
            matches = [s for s in segments if s['end_sec'] >= left and s['start_sec'] <= right]
            windows.append({'edge': edge, 'window_s': [left, right],
                            'recognized_segments': matches, 'no_recognition_in_window': not matches})
        words = tokens(text)
        result.append({'cue': cue, 'chapter': placement['chapter'], 'start_s': start, 'end_s': end,
                       'expected_first_words': words[:12], 'expected_last_words': words[-12:],
                       'previous_cue': placements[i-1]['cue'] if i else None,
                       'next_cue': placements[i+1]['cue'] if i+1 < len(placements) else None,
                       'windows': windows})
    return result


def run():
    receipt = live_receipt()
    assert receipt.get('admitted') is True and 'exit' not in receipt
    assert receipt.get('phase') == 'asr' and receipt.get('inputs_sha256') == sha(MANIFEST), 'not this guarded ASR job'
    assert receipt['command'] == child_command(), 'the guard ran another command'
    check_files(read(MANIFEST)['files'])
    fw, model = [row['path'] for row in bound_tools()[1:3]]
    script = str(SCRIPT)
    rows = script_rows(Path(script))
    proof = bound_proof()
    wav = WAV
    command = [fw, 'transcribe', '--input', str(wav), '--backend', 'whisper-cpp',
               '--model', model, '--language', 'en', '--no-diarize', '--no-persist',
               '--split-on-word', '--max-segment-length', '1', '--threads', '4',
               '--timeout', '7200', '--json']
    with (OUT / 'full-large.json').open('x') as stdout, (OUT / 'full-large.stderr.log').open('x') as stderr:
        subprocess.run(command, stdout=stdout, stderr=stderr, check=True, timeout=7260)
    result = read(OUT / 'full-large.json')
    result = result.get('result', result)
    assert result['segments'], 'No recognized speech'
    boundaries = boundary_review(proof, rows, result['segments'])
    with (OUT / 'cue-boundaries.json').open('x') as handle:
        json.dump({'master_sha256': proof['sha256'], 'cue_count': len(boundaries),
                   'cues': boundaries, 'scope': 'All cue edge windows, including chapter joins; provisional ASR timestamps, not listening or forced alignment. Every window requires triage.'}, handle, indent=2)
    expected = [{'word': t, 'cue': cue} for cue, text in rows for t in tokens(text)]
    recognized = [{'word': t, 'start_s': s['start_sec'], 'end_s': s['end_sec']}
                  for s in result['segments'] for t in tokens(s['text'])]
    changes = []
    for op, i, k, j, l in difflib.SequenceMatcher(None, [x['word'] for x in expected],
                                                [x['word'] for x in recognized], autojunk=False).get_opcodes():
        if op != 'equal':
            changes.append({'operation': op, 'expected': expected[i:k], 'recognized': recognized[j:l],
                            'expected_context': expected[max(0, i-3):min(len(expected), k+3)],
                            'recognized_context': recognized[max(0, j-3):min(len(recognized), l+3)]})
    with (OUT / 'comparison.json').open('x') as handle:
        json.dump({'master_sha256': proof['sha256'], 'script_sha256': sha(script),
                   'wav_sha256': sha(wav), 'cue_count': len(rows), 'expected_words': len(expected),
                   'recognized_words': len(recognized), 'differences': changes,
                   'scope': 'ASR leads awaiting triage, including numeric formatting. No speech correctness, listening, fine sync or independent AV acceptance.'}, handle, indent=2)
    print(f'Encoded-master ASR complete; {len(changes)} differences await triage.', flush=True)


def check():
    assert not OUT.exists() and not MANIFEST.exists(), 'Duplicate ASR refused'
    bound_tools()
    script_rows(SCRIPT)
    proof = bound_proof()
    print(json.dumps({'job': JOB, 'master_sha256': proof['sha256'], 'wav_sha256': proof['wav_sha256'], 'launched': False}))


def launch():
    assert not OUT.exists() and not MANIFEST.exists(), 'Duplicate ASR refused'
    sources = bound_tools()
    script_rows(SCRIPT)
    bound_proof()
    paths = [Path(x['path']) for x in sources[1:]] + [Path(__file__).resolve(), HERE / 'start-asr01.sh', SOURCES, TOOLS, GUARD, P / 'guard-pins01.json',
             SCRIPT, AUDIO / 'audio-summary.json', WAV]
    paths += [Path(x['path']) for x in read(P / 'guard-pins01.json')['files']]
    rows = [{'path': str(p), 'sha256': sha(p)} for p in paths]
    with MANIFEST.open('x') as handle:
        json.dump({'files': rows}, handle, indent=2)
    subprocess.run([sys.executable, '-I', '-B', str(GUARD), '--phase', 'asr', '--job', JOB,
                    '--inputs', str(MANIFEST), '--', *child_command()],
                   cwd=R, check=True)


if __name__ == '__main__':
    if not __debug__:
        raise SystemExit('mini_asr01.py checks with assertions: launch without -O')
    args = sys.argv[1:]
    mode = args.pop(0) if args[:1] in (['--run'], ['--check']) else None
    select(*args)
    {None: launch, '--check': check, '--run': run}[mode]()
