#!/usr/bin/env python3
"""Rehearse stage07 audio in a throwaway root that links to the real repository.

Real: root's two takes and three receipts, the two-take recognition job root fetched and its manifest, the sealed stage05
and stage06 lanes, root's stage06 decisions, the sealed repair, ffmpeg, the assembler and the three stage07 tools, run as
they will be run. Stand-in: the guard, which writes a receipt and runs the command. It ran on no real admission.
take01 is root's provisional choice (Mail 5671), and selection here selects nothing: the fixture's `by` and `basis` say so.
The fixture's root is not the repository, so sealed lanes spell paths outside it. For that reason alone the fixture's
copy of the bridge differs from the bundle's by one line, in rel(). A plant changes the job's bytes, which the result
pins refuse at once, so plants reach the later checks through a copy of the intake with those pins set to the planted
bytes. A separate check proves the pins themselves. The job is copied into scratch; the real folder is only read.

Run from the repository root:  python3 -I -B <bundle>/checks/rehearse_stage07.py SCRATCH_FOLDER
"""
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys

HERE = Path(__file__).resolve()
BUNDLE = HERE.parents[1]
REAL = BUNDLE.parents[4]
PREL = 'docs/video-program/third-wave-20261024/opus-replacements-20261104'
EC = 'narration_lab/third-wave-20261024/root/elevenlabs-complex-rebuild-20261104'
WORK = 'narration_lab/third-wave-20261024/root/five-film-release-20261027'
STAGE07 = f'{PREL}/{BUNDLE.name}'
GUARD = f'{PREL}/engine-secondary-validation01/guard_owner56g_target20g.py'
TAKES = REAL / EC / 'remote-ts1-01/cauchy-cue-repair01'
MINI = REAL / 'narration_lab/third-wave-20261024/root/elevenlabs-logarithm-rebuild-20261104/remote-mac-mini-max01'
JOB = 'complex-cauchy-take-asr01'
MANIFEST = 'complex-cauchy-take-asr-inputs01.json'
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
if '--preflight-only' in args:
    sys.exit(0)
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

    job, inputs = scratch / f'mini/jobs/{JOB}', scratch / f'mini/{MANIFEST}'
    shutil.copytree(MINI / 'jobs' / JOB, job)
    shutil.copyfile(MINI / MANIFEST, inputs)
    text = (root / STAGE07 / 'frederick_intake_stage07.py').read_text()
    names = ('resource-receipt.json', 'take-comparison.json', 'take01-full-large.json', 'take02-full-large.json')
    for number, (what, plant) in enumerate(PLANTS.items()):
        folder = scratch / f'plant-{number}'
        bad, bad_inputs = folder / JOB, folder / MANIFEST
        shutil.copytree(job, bad)
        shutil.copyfile(inputs, bad_inputs)
        plant(bad, bad_inputs)
        out = run(*intake, 'adopt', '--lane', lane, '--job', str(bad), '--job-inputs', str(bad_inputs))
        report(out.returncode == 2 and 'not the evidence root fetched' in out.stderr and not (root / lane).exists(), f'the pins refuse {what}')
        changed = text.replace(sha(inputs), sha(bad_inputs))
        for name in names:
            changed = changed.replace(sha(job / name), sha(bad / name))
        (folder / 'intake-pins-moved.py').write_text(changed)
        out = run(*python, str(folder / 'intake-pins-moved.py'), 'adopt', '--lane', lane, '--job', str(bad), '--job-inputs', str(bad_inputs))
        report(out.returncode == 2 and 'not the evidence root fetched' not in out.stderr and not (root / lane).exists(), f'past the pins, adopt still refuses {what}')
    out = run(*intake, 'adopt', '--lane', lane, '--job', str(job), '--job-inputs', str(inputs))
    report(out.returncode == 0 and json.loads(out.stdout)['processed'] == 2, 'adopt reads both takes from the real job')
    ingest = read(root / lane / 'ingest-01/ingest.json')
    report(ingest['non_json_log_lines'] == 1 and ingest['job_inputs_sha256'] == sha(inputs) and len(ingest['records']) == 2,
           'the ingest counts the one log line that is not JSON and binds the manifest')
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
    for path, label in ((job / 'take-comparison.json', 'comparison'), (inputs, 'manifest'), (job / 'take01-full-large.json', 'take01 result')):
        saved = path.read_bytes()
        path.write_bytes(saved + b' ')
        out = run(*bridge, 'report', *common, '--out', f'{scratch}/report-{path.stem}')
        path.write_bytes(saved)
        report(out.returncode == 2, f'the bridge refuses a {label} changed after adoption')
    out = run(*bridge, 'carry', *common, '--take', '09_5-download01.wav', '--by', 'x', '--basis', 'x', '--out', f'{scratch}/refused')
    report(out.returncode == 2, 'carry refuses the old stage06 take')
    out = run(*bridge, 'carry', *common, '--take', '09_5-take01.wav', '--by', 'rehearsal fixture', '--basis', 'fixture copy of the provisional take in Mail 5671; selects nothing',
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
