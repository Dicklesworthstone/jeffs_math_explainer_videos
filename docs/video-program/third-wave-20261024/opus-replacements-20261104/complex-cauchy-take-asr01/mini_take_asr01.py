"""Recognize both retained 09_5 takes on mac-mini-max, with that lane's verified fw, model and guard.

Root's two full takes (Mail 5610), one fw run each, as saved: fw normalizes its own input, as in stage06's take recognition.
The fw command is asr02.py's, apart from a 180 s timeout, stage06's value for takes of this length. The comparison is
asr02.py's sign-keeping one, plus each sentence's words found, because all four sentences of this take are newly voiced.
Start through start-take-asr01.sh.

    mini_take_asr01.py --check NN    read-only: every gate, no job name used
    mini_take_asr01.py NN            freeze the manifest, run the guard, recheck, print the hash
    mini_take_asr01.py --run NN      the guard's child; never call it by hand

ASR gives leads for triage. It selects no take, and it isn't listening, fine sync or acceptance.
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
TAKES = R / 'narration_lab/third-wave-20261024/root/elevenlabs-complex-rebuild-20261104/remote-ts1-01/cauchy-cue-repair01'
RECORDS = {'request01.json': 'e9b0aef22537f507fcf968b42065c442f536cd82e0c10a76344c9771e2f9a84a',
           'submission01.json': 'cf0b5c049f17abe6178507b808b1d424f8c7b0f31127e9f8390113ed64b8619d',
           'download-completion01.json': '57ec25437d212f5197db6e106aa4782cf884c5cd9d26ec3ee2ef7bede75d8f9f'}
CUE = '09_5'
TAKE_NAMES = ['09_5-take01.wav', '09_5-take02.wav']


def select(attempt):
    """The explicit selection, set once."""
    global ATTEMPT, JOB, OUT, MANIFEST
    assert re.fullmatch(r'\d\d', attempt) and int(attempt) >= 1, f'bad attempt {attempt}'
    ATTEMPT = attempt
    JOB = f'complex-cauchy-take-asr{attempt}'
    OUT = B / JOB
    MANIFEST = P / f'complex-cauchy-take-asr-inputs{attempt}.json'


def child_command():
    return [sys.executable, '-I', '-B', str(Path(__file__).resolve()), '--run', ATTEMPT]


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


def bound_takes():
    """Root's three records by hash, then each take by the hash its record holds. Those records spell this repository's
    path on the host that wrote them, so only the file names count here."""
    for name, digest in RECORDS.items():
        assert (TAKES / name).is_file(), f'not transferred: {name}'
        assert sha(TAKES / name) == digest, f'not root\'s record: {name}'
    takes = [{**row, 'name': Path(row['path']).name} for row in read(TAKES / 'download-completion01.json')['takes']]
    assert [take['name'] for take in takes] == TAKE_NAMES, 'not the two retained takes'
    for take in takes:
        assert (TAKES / take['name']).is_file(), f'not transferred: {take["name"]}'
        assert sha(TAKES / take['name']) == take['sha256'], f'{take["name"]} changed in transfer'
    return read(TAKES / 'request01.json'), takes


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


def sentences(text):
    return [s.strip() for s in re.findall(r".*?[.?!](?= |$)|.+$", text) if s.strip()]


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


def compare(take, text, result):
    """One take against the cue. Every difference is kept, and nothing here accepts or ranks a take."""
    segments = result['segments']
    placement = {'cue': CUE, 'chapter': '09_boundary', 'start_frame': 0, 'duration_s': take['frames'] / take['sample_rate']}
    edges = boundary_review({'placements': [placement], 'frames': take['frames'], 'fps': take['sample_rate']}, [(CUE, text)], segments)
    expected = [{'word': t, 'cue': CUE} for t in tokens(text)]
    recognized = [{'word': t, 'start_s': s['start_sec'], 'end_s': s['end_sec']}
                  for s in segments for t in tokens(s['text'])]
    changes, found = [], {}
    for op, i, k, j, l in difflib.SequenceMatcher(None, [x['word'] for x in expected],
                                                [x['word'] for x in recognized], autojunk=False).get_opcodes():
        if op != 'equal':
            changes.append({'operation': op, 'expected': expected[i:k], 'recognized': recognized[j:l],
                            'expected_context': expected[max(0, i-3):min(len(expected), k+3)],
                            'recognized_context': recognized[max(0, j-3):min(len(recognized), l+3)]})
        else:
            found.update(zip(range(i, k), recognized[j:l]))
    rollup, at = [], 0
    for number, sentence in enumerate(sentences(text)):
        count = len(tokens(sentence))
        words = [found[n] for n in range(at, at + count) if n in found]
        rollup.append({'sentence': number, 'text': sentence, 'expected_words': count, 'matched_words': len(words),
                       'first_matched_s': words[0]['start_s'] if words else None,
                       'last_matched_s': words[-1]['end_s'] if words else None})
        at += count
    assert at == len(expected), 'sentence words do not add up to the cue words'
    raw = result.get('raw_output') or {}
    return {'take': take['name'], 'sha256': take['sha256'], 'seconds': take['frames'] / take['sample_rate'],
            'transcript': result.get('transcript'), 'expected_words': len(expected), 'recognized_words': len(recognized),
            'matched_words': len(found), 'differences': changes, 'sentences': rollup, 'cue_edges': edges[0],
            'recognizer_reported': {'backend': result.get('backend'), 'model_version_tag': raw.get('model_version_tag'),
                                    'word_timestamps': raw.get('word_timestamps')}}


def run():
    receipt = live_receipt()
    assert receipt.get('admitted') is True and 'exit' not in receipt
    assert receipt.get('phase') == 'asr' and receipt.get('inputs_sha256') == sha(MANIFEST), 'not this guarded ASR job'
    assert receipt['command'] == child_command(), 'the guard ran another command'
    check_files(read(MANIFEST)['files'])
    sources = bound_tools()
    fw, model = [row['path'] for row in sources[1:3]]
    request, takes = bound_takes()
    compared = []
    for number, take in enumerate(takes, 1):
        wav = TAKES / take['name']
        command = [fw, 'transcribe', '--input', str(wav), '--backend', 'whisper-cpp',
                   '--model', model, '--language', 'en', '--no-diarize', '--no-persist',
                   '--split-on-word', '--max-segment-length', '1', '--threads', '4',
                   '--timeout', '180', '--json']
        full = OUT / f'take{number:02d}-full-large.json'
        with full.open('x') as stdout, (OUT / f'take{number:02d}-full-large.stderr.log').open('x') as stderr:
            subprocess.run(command, stdout=stdout, stderr=stderr, check=True, timeout=240)
        result = read(full)
        result = result.get('result', result)
        assert result['segments'], 'No recognized speech'
        compared.append(compare(take, request['text'], result))
    with (OUT / 'take-comparison.json').open('x') as handle:
        json.dump({'cue': CUE, 'text_sha256': request['text_sha256'], 'request_sha256': RECORDS['request01.json'],
                   'download_completion_sha256': RECORDS['download-completion01.json'],
                   'fw_sha256': sources[1]['sha256'], 'model_sha256': sources[2]['sha256'], 'takes': compared,
                   'selected_take': None, 'listening': False, 'fine_sync': False,
                   'scope': 'ASR leads for both retained takes, awaiting triage; recognized times are provisional. No take is selected. No speech correctness, pronunciation, listening, fine sync or acceptance.'}, handle, indent=2)
    print(f'Take recognition complete; {" and ".join(str(len(x["differences"])) for x in compared)} differences await triage.', flush=True)


def check():
    assert not OUT.exists() and not MANIFEST.exists(), 'Duplicate recognition refused'
    bound_tools()
    _, takes = bound_takes()
    print(json.dumps({'job': JOB, 'takes': [take['sha256'] for take in takes], 'launched': False}))


def launch():
    assert not OUT.exists() and not MANIFEST.exists(), 'Duplicate recognition refused'
    sources = bound_tools()
    _, takes = bound_takes()
    paths = [Path(x['path']) for x in sources[1:]] + [Path(__file__).resolve(), HERE / 'start-take-asr01.sh', SOURCES, TOOLS, GUARD, P / 'guard-pins01.json']
    paths += [TAKES / name for name in RECORDS] + [TAKES / take['name'] for take in takes]
    paths += [Path(x['path']) for x in read(P / 'guard-pins01.json')['files']]
    rows = [{'path': str(p), 'sha256': sha(p)} for p in paths]
    with MANIFEST.open('x') as handle:
        json.dump({'files': rows}, handle, indent=2)
    subprocess.run([sys.executable, '-I', '-B', str(GUARD), '--phase', 'asr', '--job', JOB,
                    '--inputs', str(MANIFEST), '--', *child_command()],
                   cwd=R, check=True)
    d = read(OUT / 'resource-receipt.json')
    assert d.get('finished') and d.get('admitted') is True and d.get('exit') == 0, f'{JOB} did not pass'
    assert not d.get('stopped_for_resource_limit') and 'error' not in d, f'{JOB} stopped or raised'
    assert d['inputs_sha256'] == d['final_inputs_sha256'] == sha(MANIFEST), f'{JOB}: its frozen inputs moved'
    print(f'TAKE ASR PASS: {JOB} take-comparison.json sha256 {sha(OUT / "take-comparison.json")}; no take is selected.', flush=True)


if __name__ == '__main__':
    if not __debug__:
        raise SystemExit('mini_take_asr01.py checks with assertions: launch without -O')
    args = sys.argv[1:]
    mode = args.pop(0) if args[:1] in (['--run'], ['--check']) else None
    select(*args)
    {None: launch, '--check': check, '--run': run}[mode]()
