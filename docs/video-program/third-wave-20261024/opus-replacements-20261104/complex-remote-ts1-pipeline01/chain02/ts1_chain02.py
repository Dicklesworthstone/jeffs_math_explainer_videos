"""The sealed ts1 chain under new job names, with the narration tools run through the alias adapter.

complex-remote-timeline01 failed in standalone verify because report.json spells 414 take paths
through R and the sealed verifier spells them through the real root (Mails 5426, 5428). The binary's
own narration call would fail the same way. chain01 stays sealed and is loaded here by hash; every
stage check in it runs unchanged. What changes is listed in INTEGRATION.md.

    --check STAGE    read-only: chain01's check; for timeline also the identity diagnostic, the
                     sealed verifier through the adapter, and the real binary's first narrated call
    --launch STAGE   wait for upstream exit files, freeze, run ts1's guard (start-chain02.sh)
    --run STAGE      the guard's child; never call it by hand
"""
import importlib.util
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile

HERE = Path(__file__).resolve().parent
CHAIN01 = HERE.parent / 'chain01/ts1_chain01.py'
CHAIN01_SHA256 = '2210245e28694688d9d1f40ad5f7e4fbc5cda4dc1d7f5dc7ba4e7f5cd7a3b0aa'
ALIAS_UNITS, ALIAS_ROWS = 105, 414
REFUSAL = 'coherent-narration: assembly sentence/source spans differ from the supplied script and generation receipts'
SPEC = importlib.util.spec_from_file_location('ts1_chain01', CHAIN01)
M = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(M)
Q = M.Q
R, O, JOBS, C, N, BIN, NATIVE, read, sha = M.R, M.O, M.JOBS, M.C, M.N, M.BIN, M.NATIVE, M.read, M.sha

ME = R / Path(__file__).resolve().relative_to(Q.PHYSICAL)
START = ME.parent / 'start-chain02.sh'
ADAPTER = ME.parent / 'narration_alias01.py'
SHIM = ME.parent / 'shim/python3'
TOOLS = NATIVE / 'workspace/tools'
OLD_JOB = 'complex-remote-timeline01'
OLD_MANIFEST = O / 'chain-timeline-inputs01.json'
OLD_EXIT, OLD_LOG = O / 'chain-timeline-controller01.exit', O / 'chain-timeline-controller01.log'
STAGES = {'timeline': M.run_timeline, 'score': M.run_score, 'preview': M.run_preview, 'pictures': M.run_pictures}
NARRATED = {'timeline': ('coherent_narration.py', 'narration_delivery.py'), 'score': ('narration_delivery.py',),
            'preview': ('narration_delivery.py',)}
sealed_check_qualified, sealed_stage_files = M.check_qualified, M.stage_files


def check_timeline01():
    """The failed job stays what it is: the exact refusal, before any render, nothing relabelled."""
    d = read(JOBS / OLD_JOB / 'resource-receipt.json')
    assert d.get('finished') and d.get('admitted') is True and d.get('exit') == 1, 'timeline01 receipt changed'
    assert d.get('inputs_sha256') == d.get('final_inputs_sha256') == sha(OLD_MANIFEST), 'timeline01 inputs changed'
    assert not d.get('stopped_for_resource_limit') and 'error' not in d, 'timeline01 was not this failure'
    assert d.get('policy') == M.POLICY, 'Guard policy changed: ' + OLD_JOB
    assert (JOBS / OLD_JOB / 'command.log').read_text().count(REFUSAL) == 1, 'timeline01 log lacks the exact refusal'
    assert not (JOBS / OLD_JOB / 'timeline-output').exists(), 'timeline01 rendered; this recovery does not fit it'
    assert not (JOBS / OLD_JOB / 'timeline-summary.json').exists(), 'timeline01 rendered; this recovery does not fit it'
    assert OLD_EXIT.read_text().strip() == '1', 'timeline01 controller changed'


def check_bundle():
    assert sha(CHAIN01) == CHAIN01_SHA256, 'Sealed chain01 changed'
    assert os.access(SHIM, os.X_OK), 'shim/python3 lost its executable bit'


def check_qualified():
    sealed_check_qualified()
    check_timeline01()


def stage_files(stage):
    files = sealed_stage_files(stage)
    if stage == 'timeline':
        for path in (CHAIN01, ADAPTER, SHIM, OLD_MANIFEST, OLD_EXIT, OLD_LOG, *Q.job_files(OLD_JOB)):
            files[str(path)] = sha(path)
    return files


def alias_env(log):
    return {'NARRATION_ALIAS_PYTHON': sys.executable, 'NARRATION_ALIAS_ADAPTER': str(ADAPTER),
            'NARRATION_ALIAS_ROOT': str(R), 'NARRATION_ALIAS_LOG': str(log)}


def identity():
    """The sealed rows against report.json, field by field, with the sealed code unwrapped."""
    spec = importlib.util.spec_from_file_location('coherent', TOOLS / 'coherent_narration.py')
    coherent = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(coherent)
    report = read(N / 'report.json')['lines']
    records = read(N / 'assembly.json')['manifest']['units']
    script, units = coherent.read_units(C / 'script.tsv', N / 'external-units.json', True)
    assert [row['id'] for row in records] == [unit['id'] for unit in units], 'Assembly units differ'
    built = {}
    for unit, record in zip(units, records):
        for cue, row, _ in coherent.unit_spans(unit, script, record):
            built.setdefault(cue, []).append(row)
    assert list(built) == list(report), 'Cue order differs'
    tally = {'cues': len(report), 'rows': 0, 'source_differs': 0, 'other_fields_differ': 0, 'samefile': 0}
    for cue, rows in report.items():
        assert len(rows) == len(built[cue]), 'Row count differs: ' + cue
        for old, new in zip(rows, built[cue]):
            assert set(old) == set(new), 'Row keys differ: ' + cue
            tally['rows'] += 1
            tally['source_differs'] += old['source'] != new['source']
            tally['other_fields_differ'] += sum(old[key] != new[key] for key in old if key != 'source')
            tally['samefile'] += os.path.samefile(old['source'], new['source'])
    return tally


def check_identity(tally):
    wanted = {'cues': ALIAS_UNITS, 'rows': ALIAS_ROWS, 'source_differs': ALIAS_ROWS, 'other_fields_differ': 0, 'samefile': ALIAS_ROWS}
    assert tally == wanted, f'Alias identity differs: {tally}'


def run_adapter(argv, log):
    return subprocess.run([sys.executable, '-I', '-B', str(ADAPTER), *argv], cwd=R, env={**os.environ, **alias_env(log)},
                          capture_output=True, text=True, timeout=1800)


def standalone_argv():
    return ['-I', '-B', str(TOOLS / 'coherent_narration.py'), 'verify', str(N), '--script', str(C / 'script.tsv'),
            '--units', str(N / 'external-units.json'), '--require-directions']


def verify_narration():
    """chain01's standalone verify, through the adapter, with its evidence written first."""
    tally = identity()
    with (M.job('timeline') / 'alias-identity.json').open('x') as handle:
        json.dump(tally, handle, indent=2)
    check_identity(tally)
    run = run_adapter(standalone_argv(), M.job('timeline') / 'alias-calls.jsonl')
    print(run.stdout, run.stderr, flush=True)
    assert run.returncode == 0, 'Standalone verify refused through the adapter'


def check_calls(stage, log):
    """Prove the calls went through the adapter and covered the whole film."""
    calls = [json.loads(line) for line in log.read_text().splitlines()]
    assert sorted(call['tool'] for call in calls) == sorted(NARRATED[stage]), f'Adapter calls differ: {stage}'
    for call in calls:
        assert (call['units'], call['rows'], call['renamed']) == (ALIAS_UNITS, ALIAS_ROWS, ALIAS_ROWS), f'Adapter call differs: {call}'


def native_probe(log):
    """Run the real binary to its first narrated call. check_delivery_contract comes before any
    output, so the refusing stub stops it there. Then run that exact call through the adapter."""
    folder = Path(tempfile.mkdtemp(prefix='chain02-probe-'))
    stub = folder / 'python3'
    stub.write_text('#!/bin/sh\nfor a; do printf "%s\\n" "$a"; done > "$0.argv"\nexit 3\n')
    stub.chmod(0o755)
    out = folder / 'out'
    run = subprocess.run([str(BIN), 'render', 'all', '--timeline-only', '--fps', '60', '--narration', str(N), '--out', str(out)],
                         cwd=R, env={**os.environ, 'PATH': f"{folder}:{os.environ['PATH']}"}, capture_output=True, text=True, timeout=300)
    assert run.returncode != 0 and not out.exists(), 'Binary went past its narration gate'
    argv = (folder / 'python3.argv').read_text().splitlines()
    assert argv[:2] == ['-I', '-B'] and Path(argv[2]).samefile(TOOLS / 'narration_delivery.py'), f'Binary call differs: {argv[:3]}'
    fields = dict(zip(argv[3:7:2], argv[4:8:2]))
    assert Path(fields['--topic']).samefile(C) and fields['--narration'] == str(N), f'Binary call differs: {fields}'
    assert argv[7:9] == ['--consumer', 'render'] and argv[9] == '--gain-db', f'Binary call differs: {argv[7:10]}'
    assert argv[11::2] == ['--chapter-key'] * 15 and argv[12::2] == list(M.CHAPTERS), 'Binary chapter keys differ'
    run = run_adapter(argv, log)
    assert run.returncode == 0, f'Binary call refused through the adapter: {run.stderr.strip()[-300:]}'
    result = json.loads(run.stdout)
    assert result['verified_cues'] == ALIAS_UNITS and result['plan_mode'] == 'external-ui', f'Binary call differs: {result}'
    return argv[2], fields['--topic']


def preflight(stage):
    check_bundle()
    check_timeline01()
    M.preflight(stage)
    if stage == 'timeline':
        tally = identity()
        check_identity(tally)
        log = Path(tempfile.mkdtemp(prefix='chain02-calls-')) / 'alias-calls.jsonl'
        run = run_adapter(standalone_argv(), log)
        assert run.returncode == 0, f'Standalone verify refused through the adapter: {run.stderr.strip()[-300:]}'
        tool, topic = native_probe(log)
        calls = read_calls(log)
        print(json.dumps({'identity': tally, 'adapter_calls': calls, 'binary_tool': tool, 'binary_topic': topic}))


def read_calls(log):
    return [{key: call[key] for key in ('tool', 'units', 'rows', 'renamed')} for call in map(json.loads, log.read_text().splitlines())]


def run(stage):
    if stage in NARRATED:
        log = M.job(stage) / 'alias-calls.jsonl'
        os.environ.update(alias_env(log), PATH=f"{SHIM.parent}:{os.environ['PATH']}")
    STAGES[stage]()
    if stage in NARRATED:
        check_calls(stage, log)


M.NAMES.update({stage: f'complex-remote-{stage}02' for stage in M.NAMES})
M.ME, M.START = ME, START
M.TIMELINE_OUT = JOBS / M.NAMES['timeline'] / 'timeline-output'
M.SCORE_OUT = JOBS / M.NAMES['score'] / 'scored'
M.PREVIEW_OUT = JOBS / M.NAMES['preview'] / 'preview'
M.PICTURES_OUT = JOBS / M.NAMES['pictures'] / 'pictures'
M.exit_file = lambda stage: O / f'chain-{stage}-controller02.exit'
M.manifest = lambda stage: O / f'chain-{stage}-inputs02.json'
M.check_qualified, M.stage_files, M.verify_narration = check_qualified, stage_files, verify_narration

if __name__ == '__main__':
    assert len(sys.argv) == 3 and sys.argv[1] in ('--check', '--launch', '--run') and sys.argv[2] in M.NAMES, __doc__
    mode, name = sys.argv[1:]
    check_bundle()
    if mode == '--check':
        preflight(name)
    elif mode == '--launch':
        M.launch(name)
    else:
        run(name)
