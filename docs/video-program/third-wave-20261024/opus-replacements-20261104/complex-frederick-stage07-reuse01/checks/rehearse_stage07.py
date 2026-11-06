#!/usr/bin/env python3
"""Rehearse stage07 audio in a throwaway root that links to the real repository.

Real: root's two takes and three receipts, the sealed stage05 and stage06 lanes, root's stage06 decisions, the sealed
repair, ffmpeg, the assembler and the three stage07 tools, run as they will be run.
Stand-ins: fw and the guard. fw never saw these takes: each record's words are the request text spread evenly over the
take's own four spoken stretches, and its times are invented. The guard here writes a receipt and runs the command.
So this proves the plumbing and the refusals. It says nothing about the takes, and it selects none: take01 is an
arbitrary fixture choice. The fixture's root is not the repository, so sealed lanes spell paths outside it; the fixture's
copy of the bridge differs from the bundle's by one line, in rel(), for that reason alone.

Run from the repository root:  python3 -I -B <bundle>/checks/rehearse_stage07.py SCRATCH_FOLDER
"""
import array
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import wave

HERE = Path(__file__).resolve()
BUNDLE = HERE.parents[1]
REAL = BUNDLE.parents[4]
PREL = 'docs/video-program/third-wave-20261024/opus-replacements-20261104'
EC = 'narration_lab/third-wave-20261024/root/elevenlabs-complex-rebuild-20261104'
WORK = 'narration_lab/third-wave-20261024/root/five-film-release-20261027'
STAGE07 = f'{PREL}/{BUNDLE.name}'
GUARD = f'{PREL}/engine-secondary-validation01/guard_owner56g_target20g.py'
TAKES = REAL / EC / 'remote-ts1-01/cauchy-cue-repair01'
MODEL = Path.home() / '.cache/franken_whisper/models/whisper/whisper-large-v3-turbo-f16-v1/ggml-large-v3-turbo.bin'
OPEN = [f'{EC}/remote-ts1-01', WORK, STAGE07, f'{PREL}/engine-secondary-validation01']
failed = 0


def report(ok, what):
    global failed
    failed += not ok
    print(('PASS ' if ok else 'FAIL ') + what, flush=True)


def sha(path):
    with Path(path).open('rb') as handle:
        return hashlib.file_digest(handle, 'sha256').hexdigest()


def read(path):
    return json.loads(Path(path).read_text())


def write(path, data):
    Path(path).write_text(json.dumps(data, indent=1))


def graft(root):
    """Real folders along the paths that change; a link for everything else."""
    opened = {Path(*Path(rel).parts[:n]) for rel in OPEN for n in range(1, len(Path(rel).parts) + 1)}
    def walk(folder):
        target = root / folder
        target.mkdir(exist_ok=True)
        for child in sorted((REAL / folder).iterdir()):
            name = folder / child.name
            if name in opened:
                walk(name)
            elif child.name not in ('.git', '.ee', '__pycache__', 'target'):
                (root / name).symlink_to(child)
    walk(Path('.'))
    folder = root / STAGE07
    for path in folder.iterdir():
        path.unlink()
    for name in ('frederick_intake_stage07.py', 'frederick_timing_bridge_stage07.py', 'audio_stage07.py'):
        shutil.copyfile(BUNDLE / name, folder / name)
    bridge = folder / 'frederick_timing_bridge_stage07.py'
    old = 'if path.is_relative_to(REPO.resolve()) else str(path)\n'
    text = bridge.read_text()
    assert text.count(old) == 1
    bridge.write_text(text.replace(old, f'if path.is_relative_to(REPO.resolve()) else str(path).removeprefix({str(REAL) + "/"!r})\n'))


STUB_GUARD = r'''import hashlib, json, subprocess, sys
from pathlib import Path
args = sys.argv[1:]
job, inputs = args[args.index('--job') + 1], args[args.index('--inputs') + 1]
command = args[args.index('--') + 1:]
folder = Path('narration_lab/third-wave-20261024/root/five-film-release-20261027') / job
folder.mkdir()
digest = lambda p: hashlib.sha256(Path(p).read_bytes()).hexdigest()
receipt = {'phase': 'native', 'command': command, 'admitted': True, 'inputs_sha256': digest(inputs)}
(folder / 'resource-receipt.json').write_text(json.dumps(receipt))
with (folder / 'command.log').open('w') as log:
    code = subprocess.call(command, stdout=log, stderr=subprocess.STDOUT)
receipt.update(exit=code, final_inputs_sha256=digest(inputs), finished='rehearsal')
(folder / 'resource-receipt.json').write_text(json.dumps(receipt))
sys.exit(code)
'''


def spoken_stretches(path, count):
    """The take's own `count` stretches of speech, split at its longest quiet runs, in seconds."""
    with wave.open(str(path)) as handle:
        rate, data = handle.getframerate(), array.array('h', handle.readframes(handle.getnframes()))
    step = rate // 100
    loud = [max(map(abs, data[i:i + step])) > 300 for i in range(0, len(data) - step, step)]
    runs, start = [], None
    for i, value in enumerate([*loud, True]):
        if not value and start is None:
            start = i
        elif value and start is not None:
            runs.append((i - start, start, i))
            start = None
    first, last = loud.index(True), len(loud) - 1 - loud[::-1].index(True)
    inner = sorted(sorted(r for r in runs if r[1] > first and r[2] < last)[-(count - 1):], key=lambda r: r[1])
    edges = [first, *[x for run in inner for x in run[1:]], last]
    return [(edges[i] / 100, edges[i + 1] / 100) for i in range(0, len(edges), 2)]


def stand_in_job(folder, inputs_path, helper):
    """A finished two-take job in the shape of the sealed helper's, with invented recognition."""
    request = read(TAKES / 'request01.json')
    folder.mkdir(parents=True)
    takes = read(TAKES / 'download-completion01.json')['takes']
    sentences = [s.strip() + '.' for s in request['text'].split('.') if s.strip()]
    compared = []
    for number, take in enumerate(takes, 1):
        name = Path(take['path']).name
        segments = []
        for sentence, (a, b) in zip(sentences, spoken_stretches(TAKES / name, len(sentences))):
            words = sentence.split()
            for i, word in enumerate(words):
                segments.append({'start_sec': a + (b - a) * i / len(words), 'end_sec': a + (b - a) * (i + 1) / len(words),
                                 'text': word, 'speaker': None, 'confidence': 0.5})
        write(folder / f'take{number:02d}-full-large.json', {
            'input_path': str(TAKES / name),
            'request': {'input': {'kind': 'file', 'path': str(TAKES / name)}, 'backend': 'whisper_cpp', 'model': str(MODEL),
                        'language': 'en', 'diarize': False,
                        'backend_params': {'split_on_word': True, 'decoding': {'max_segment_length': 1}}},
            'result': {'backend': 'whisper_cpp', 'transcript': request['text'], 'language': 'en', 'segments': segments,
                       'raw_output': {'model_version_tag': 'fw-native-v1+sha256:1fc70f774d38', 'word_timestamps': True}}})
        compared.append({'take': name, 'sha256': take['sha256']})
    write(folder / 'take-comparison.json', {
        'cue': '09_5', 'text_sha256': request['text_sha256'], 'request_sha256': sha(TAKES / 'request01.json'),
        'download_completion_sha256': sha(TAKES / 'download-completion01.json'),
        'fw_sha256': '7b570cf44d1f413ead01340c45e4070ec71009b7c2482d71a2aebf49e8fda47f',
        'model_sha256': '1fc70f774d38eb169993ac391eea357ef47c88757ef72ee5943879b7e8e2bc69', 'takes': compared,
        'selected_take': None, 'listening': False, 'fine_sync': False})
    (folder / 'command.log').write_text('Take recognition complete; stand-in.\n')
    rows = [{'path': f'/stand-in/{helper}', 'sha256': 'edd34a1ae9852d9f18c66d23424bbd6583ad2c21d91004de61b72566e463c091'},
            {'path': '/stand-in/fw-0.11.0', 'sha256': '7b570cf44d1f413ead01340c45e4070ec71009b7c2482d71a2aebf49e8fda47f'},
            {'path': str(MODEL), 'sha256': '1fc70f774d38eb169993ac391eea357ef47c88757ef72ee5943879b7e8e2bc69'}]
    rows += [{'path': str(TAKES / name), 'sha256': sha(TAKES / name)}
             for name in ('request01.json', 'submission01.json', 'download-completion01.json', '09_5-take01.wav', '09_5-take02.wav')]
    write(inputs_path, {'files': rows})
    write(folder / 'resource-receipt.json', {
        'phase': 'asr', 'admitted': True, 'exit': 0, 'finished': 'rehearsal', 'inputs_sha256': sha(inputs_path),
        'final_inputs_sha256': sha(inputs_path), 'command': ['python', '-I', '-B', f'/stand-in/{helper}', '--run', '01']})


def change_json(name, change):
    def apply(job, inputs):
        path = (inputs if name == 'inputs' else job / name)
        data = read(path)
        change(data)
        write(path, data)
        if name == 'inputs':
            receipt = read(job / 'resource-receipt.json')
            receipt['inputs_sha256'] = receipt['final_inputs_sha256'] = sha(path)
            write(job / 'resource-receipt.json', receipt)
    return apply


def set_key(key, value):
    return lambda data: data.__setitem__(key, value)


def other_tag(data):
    data['result']['raw_output']['model_version_tag'] = 'fw-native-v1+sha256:000000000000'


def other_input(data):
    data['input_path'] = '/stand-in/09_5-take02.wav'


def moved_fw(data):
    next(row for row in data['files'] if row['path'].endswith('fw-0.11.0'))['sha256'] = '0' * 64


def moved_input_hash(data):
    data['inputs_sha256'] = '0' * 64


PLANTS = {
    'a take selected in the comparison': change_json('take-comparison.json', set_key('selected_take', '09_5-take01.wav')),
    'listening claimed in the comparison': change_json('take-comparison.json', set_key('listening', True)),
    'another model tag in a record': change_json('take01-full-large.json', other_tag),
    'a record of the other take': change_json('take01-full-large.json', other_input),
    'a failed job': change_json('resource-receipt.json', set_key('exit', 1)),
    'a receipt for another manifest': change_json('resource-receipt.json', moved_input_hash),
    'a manifest holding another fw': change_json('inputs', moved_fw),
}


def main():
    scratch = Path(sys.argv[1]).resolve()
    root = scratch / 'root'
    assert not root.exists(), 'Use an empty scratch folder'
    root.mkdir(parents=True)
    graft(root)
    os.chdir(root)
    python = [sys.executable, '-I', '-B']
    intake, bridge, audio = [python + [f'{STAGE07}/{name}'] for name in
                             ('frederick_intake_stage07.py', 'frederick_timing_bridge_stage07.py', 'audio_stage07.py')]
    run = lambda *argv: subprocess.run([*argv], capture_output=True, text=True)
    lane, tree = f'{EC}/recognition-stage07-root01', f'{EC}/native-stage07-root01'
    crate = f'{tree}/workspace/explainer_corpus/complex_analysis_visual'
    decisions, jobs = f'{EC}/stage07-timing-decisions01', str(root / WORK)

    out = run(*intake, 'tree', '--out', tree)
    report(out.returncode == 0 and json.loads(out.stdout)['files'] == 60, 'tree writes the 60-file stage07 workspace')
    old_rows = {str(p.relative_to(REAL / EC / 'native-stage06-root03/workspace')): sha(p)
                for p in (REAL / EC / 'native-stage06-root03/workspace').rglob('*') if p.is_file() and '.ee' not in p.parts}
    new_rows = {str(p.relative_to(root / tree / 'workspace')): sha(p) for p in (root / tree / 'workspace').rglob('*') if p.is_file()}
    changed = sorted(k for k in new_rows if old_rows.get(k) != new_rows[k])
    report(len(changed) == 5 and all(k.startswith('explainer_corpus/complex_analysis_visual/') for k in changed),
           'the tree differs from stage06 in exactly the five repair files')
    report(run(*intake, 'tree', '--out', tree).returncode == 2, 'a second tree over the same folder is refused')

    job, inputs = scratch / 'mini/complex-cauchy-take-asr01', scratch / 'mini/inputs.json'
    stand_in_job(job, inputs, 'mini_take_asr01.py')
    for what, plant in PLANTS.items():
        bad = scratch / 'plant/complex-cauchy-take-asr01'
        shutil.copytree(job, bad)
        bad_inputs = scratch / 'plant/inputs.json'
        shutil.copyfile(inputs, bad_inputs)
        plant(bad, bad_inputs)
        out = run(*intake, 'adopt', '--lane', lane, '--job', str(bad), '--job-inputs', str(bad_inputs))
        report(out.returncode == 2 and not (root / lane).exists(), f'adopt refuses {what}, and writes nothing')
        shutil.move(str(scratch / 'plant'), str(scratch / f'plant-used-{len(what)}-{what[:12].replace(" ", "-")}'))
    out = run(*intake, 'adopt', '--lane', lane, '--job', str(job), '--job-inputs', str(inputs))
    report(out.returncode == 0 and json.loads(out.stdout)['processed'] == 2, 'adopt reads both takes from the job')
    plan = read(root / lane / 'plan-01/plan.json')
    held = {take['name']: take['generation'] for take in plan['takes']}
    report(sorted(held) == ['09_5-take01.wav', '09_5-take02.wav'] and 'adopted_job' in plan['recognition']
           and 'fw_command' not in plan['recognition'], 'the plan names the adopted job and has nothing to launch')
    statement = held['09_5-take01.wav']
    report(statement['script_at_generation'].startswith('0bae7acb') and 'script_sha256' not in read(TAKES / 'request01.json'),
           "each take's generation record names the stage06 script; root's request is unchanged")
    out = run(*intake, 'adopt', '--lane', lane, '--job', str(job), '--job-inputs', str(inputs))
    report(out.returncode == 2, 'a second adopt on the lane is refused')

    common = ['--crate', crate, '--lane', lane, '--jobs', jobs]
    out = run(*bridge, 'carry', *common, '--take', '09_5-download01.wav', '--by', 'x', '--basis', 'x', '--out', f'{scratch}/refused')
    report(out.returncode == 2, 'carry refuses the old stage06 take')
    out = run(*bridge, 'carry', *common, '--take', '09_5-take01.wav', '--by', 'rehearsal fixture', '--basis', 'rehearsal fixture; selects nothing',
              '--out', decisions)
    data = json.loads(out.stdout) if out.returncode == 0 else {}
    report(out.returncode == 0 and data['still_open'] == [], f'carry leaves no timing question open {data.get("still_open", out.stderr[-400:])}')
    old = read(REAL / EC / 'stage06-timing-decisions01/selection01.json')['takes']
    new = read(root / decisions / 'selection.json')['takes']
    report([c for c in new if new[c] != old[c]] == ['09_5'], 'the selection differs from stage06 at 09_5 only')

    (root / GUARD).unlink()
    (root / GUARD).write_text(STUB_GUARD)
    out = run(*audio, '--check')
    report(out.returncode == 0, f'the audio controller check passes {out.stderr[-300:]}')
    out = run(*audio, '--launch')
    report(out.returncode == 0, f'the audio controller runs its six jobs in order {out.stderr[-600:]}')
    if out.returncode:
        print(out.stdout[-1500:])
        for name in sorted((root / WORK).glob('complex-stage07-*')):
            print(name.name, (name / 'command.log').read_text()[-1200:])
        raise SystemExit(1)
    names = ('audio-decode01', 'audio-prepare01', 'audio-assemble01', 'audio-timings01', 'native-audio-install01', 'audio-rows01')
    report(all(read(root / WORK / f'complex-stage07-{n}/resource-receipt.json')['exit'] == 0 for n in names), 'all six job receipts end with exit 0')
    rows = read(root / EC / 'remote-ts1-01/stage07-rows01.json')
    comparison = rows['comparison']
    report(comparison['cues_identical'] == 104 and comparison['cue_changed'] == '09_5' and comparison['report_rows'] == 414,
           'rows: 104 cues equal stage06, only 09_5 differs, 414 report rows')
    report(all(sha(path if Path(path).is_absolute() else Path(path)) == value
               for path, value in ((row['path'], row['sha256']) for row in rows['files'])), 'every rows file hashes as listed')
    names = {Path(row['path']).name for row in rows['files']}
    report({'request01.json', 'submission01.json', 'download-completion01.json', '09_5-take01.wav', '09_5-take02.wav', '09_5.wav', 'decode-receipt.json'} <= names,
           "the rows carry root's receipts, both takes, the decoded WAV and the decode receipt")
    done = read(root / EC / 'production-audio-stage07-01/native-narration01/native-install-receipt.json')
    report(done['cues'] == 105 and not done['listening_acceptance'] and not done['fine_sync_acceptance'],
           'the install receipt claims no listening or fine sync')
    out = run(*audio, '--check')
    report(out.returncode != 0, 'the controller refuses to run a second time')
    print(f'RESULT {"PASS" if not failed else "FAIL"}, {failed} failed')
    raise SystemExit(1 if failed else 0)


if __name__ == '__main__':
    main()
