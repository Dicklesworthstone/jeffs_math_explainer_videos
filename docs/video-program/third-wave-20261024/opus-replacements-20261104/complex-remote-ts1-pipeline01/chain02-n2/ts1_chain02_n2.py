"""Derive a ts1 narration folder, then run the sealed chain on it.

timeline01 refused because report.json spells 414 take paths through R, while coherent_narration
spells them through the real root (Mails 5426, 5428, 5453). The tools, N and the binary stay frozen,
so the one input the chain can change is --narration (Mails 5456, 5460, 5461).

    derive      writes N2 from N: report.source and assembly.report_sha256 change, nothing else
    timeline, score, preview, pictures    sealed chain01 on N2, loaded by hash, under new job names

    --check STAGE      read-only; for derive also a dry N2, both unmodified tools and the binary call
    --launch STAGE     wait for upstream exit files, freeze, run ts1's guard (start-chain02-n2.sh)
    --run STAGE        the guard's child; never call it by hand
    --compare CHAPTER  read-only: one chapter's tables against macOS, under the same exact mapping
"""
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import time

HERE = Path(__file__).resolve().parent
CHAIN01 = HERE.parent / 'chain01/ts1_chain01.py'
CHAIN01_SHA256 = '2210245e28694688924f0fd47b512479c21c4e95e39e7b8c7f44c5e5278a9074'
COHERENT_SHA256 = '95615cfadb42fbfdddf828340b74ec66d89c975a1f399593e212bdd9840e65d5'
CUES, ROWS, FILES, PRIOR_PATH_CELLS = 105, 414, 110, 52
REFUSAL = 'coherent-narration: assembly sentence/source spans differ from the supplied script and generation receipts'
assert hashlib.sha256(CHAIN01.read_bytes()).hexdigest() == CHAIN01_SHA256, 'Sealed chain01 changed'
SPEC = importlib.util.spec_from_file_location('ts1_chain01', CHAIN01)
M = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(M)
Q = M.Q
R, O, JOBS, C, BIN, read, sha = M.R, M.O, M.JOBS, M.C, M.BIN, M.read, M.sha

N1 = M.N
ME = R / Path(__file__).resolve().relative_to(Q.PHYSICAL)
START = ME.parent / 'start-chain02-n2.sh'
CHAIN01_ROW = ME.parent.parent / 'chain01/ts1_chain01.py'
TOOLS = M.NATIVE / 'workspace/tools'
DERIVE = 'complex-remote-narration-derive01'
N2 = JOBS / DERIVE / 'narration-n2'
OLD_JOB = 'complex-remote-timeline01'
OLD_MANIFEST = O / 'chain-timeline-inputs01.json'
OLD_EXIT, OLD_LOG = O / 'chain-timeline-controller01.exit', O / 'chain-timeline-controller01.log'
sealed_check_qualified, sealed_check_upstream = M.check_qualified, M.check_upstream
sealed_stage_files, sealed_base_rows, sealed_child_gate = M.stage_files, M.base_rows, M.child_gate


def digest(data):
    return hashlib.sha256(data).hexdigest()


def check_timeline01():
    """The failed job stays what it is: the exact refusal, before any render, nothing relabelled."""
    d = read(JOBS / OLD_JOB / 'resource-receipt.json')
    assert d.get('finished') and d.get('admitted') is True and d.get('exit') == 1, 'timeline01 receipt changed'
    assert d.get('inputs_sha256') == d.get('final_inputs_sha256') == sha(OLD_MANIFEST), 'timeline01 inputs changed'
    assert not d.get('stopped_for_resource_limit') and 'error' not in d, 'timeline01 was another failure'
    assert d.get('policy') == M.POLICY, 'Guard policy changed: ' + OLD_JOB
    assert (JOBS / OLD_JOB / 'command.log').read_text().count(REFUSAL) == 1, 'timeline01 log lacks the exact refusal'
    assert not (JOBS / OLD_JOB / 'timeline-output').exists(), 'timeline01 rendered, so this recovery does not fit'
    assert OLD_EXIT.read_text().strip() == '1', 'timeline01 controller changed'


def derived_bytes():
    """N's report and assembly with the two permitted changes, from the sealed unit_spans.

    Exact quoted strings are substituted, so every other byte stays by construction. The parsed
    result must then equal what the unmodified verifier will compute."""
    os.chdir(R)
    assert sha(TOOLS / 'coherent_narration.py') == COHERENT_SHA256, 'Sealed coherent_narration.py changed'
    spec = importlib.util.spec_from_file_location('coherent', TOOLS / 'coherent_narration.py')
    coherent = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(coherent)
    report_bytes, assembly_bytes = (N1 / 'report.json').read_bytes(), (N1 / 'assembly.json').read_bytes()
    report, assembly = json.loads(report_bytes), json.loads(assembly_bytes)
    assert assembly['report_sha256'] == digest(report_bytes), 'Assembly does not bind this report'
    records = assembly['manifest']['units']
    script, units = coherent.read_units(C / 'script.tsv', N1 / 'external-units.json', True)
    assert [row['id'] for row in records] == [unit['id'] for unit in units], 'Assembly units differ from the plan'
    fresh = {}
    for unit, record in zip(units, records):
        for cue, row, _ in coherent.unit_spans(unit, script, record):
            fresh.setdefault(cue, []).append(row)
    assert set(fresh) == set(report['lines']) and len(fresh) == CUES, 'Cue coverage differs'
    renames, uses = {}, {}
    for cue, stored in report['lines'].items():
        assert len(stored) == len(fresh[cue]), 'Row count differs: ' + cue
        for old, new in zip(stored, fresh[cue]):
            assert set(old) == set(new), 'Row keys differ: ' + cue
            assert [key for key in old if old[key] != new[key]] == ['source'], 'More than source differs: ' + cue
            assert os.path.samefile(old['source'], new['source']), 'Not the same file: ' + cue
            assert sha(new['source']) == old['source_sha256'], 'Take hash differs: ' + cue
            assert renames.setdefault(old['source'], new['source']) == new['source'], 'Source maps twice: ' + cue
            uses[old['source']] = uses.get(old['source'], 0) + 1
    assert sum(uses.values()) == ROWS and len(renames) == len(set(renames.values())) == CUES, 'Row or take count differs'
    text = report_bytes
    for old, new in renames.items():
        quoted = json.dumps(old).encode()
        assert text.count(quoted) == uses[old], 'Source string appears outside its rows'
        text = text.replace(quoted, json.dumps(new).encode())
    derived = json.loads(text)
    assert derived['lines'] == fresh and {k: v for k, v in derived.items() if k != 'lines'} == {k: v for k, v in report.items() if k != 'lines'}, 'Derived report differs'
    quoted = json.dumps(assembly['report_sha256']).encode()
    assert assembly_bytes.count(quoted) == 1, 'Report hash appears more than once'
    assembly_out = assembly_bytes.replace(quoted, json.dumps(digest(text)).encode())
    assert json.loads(assembly_out) == {**assembly, 'report_sha256': digest(text)}, 'Derived assembly differs'
    tally = {'cues': len(fresh), 'rows': sum(uses.values()), 'source_fields_changed': sum(uses.values()), 'other_fields_changed': 0,
             'samefile': sum(uses.values()), 'take_hash_equal': sum(uses.values()), 'takes': len(renames)}
    return text, assembly_out, renames, tally


def names():
    found = sorted(path.name for path in N1.iterdir())
    assert len(found) == FILES and all((N1 / name).is_file() for name in found), 'N file set differs'
    return found


def write_folder(out, report, assembly, place):
    out.mkdir()
    for name in names():
        data = {'report.json': report, 'assembly.json': assembly}.get(name)
        if data is None:
            place(N1 / name, out / name)
            continue
        with (out / name).open('xb') as handle:
            handle.write(data)


def check_folder(out, report, assembly):
    """N2 is exactly the derivation: two rebuilt files, and 108 byte for byte."""
    assert sorted(path.name for path in out.iterdir()) == names(), 'N2 file set differs'
    changed = {'report.json': digest(report), 'assembly.json': digest(assembly)}
    for name in names():
        assert sha(out / name) == changed.get(name, sha(N1 / name)), 'N2 differs: ' + name


def tool_argv(folder):
    return [sys.executable, '-I', '-B', str(TOOLS / 'coherent_narration.py'), 'verify', str(folder), '--script', str(C / 'script.tsv'),
            '--units', str(folder / 'external-units.json'), '--require-directions']


def binary_call(folder):
    """The real binary's first narrated call. check_delivery_contract comes before any output, so a
    refusing python3 stops it there. The stub records the call and nothing else."""
    probe = Path(tempfile.mkdtemp(prefix='chain02-n2-probe-'))
    stub = probe / 'python3'
    stub.write_text('#!/bin/sh\nfor a; do printf "%s\\n" "$a"; done > "$0.argv"\nexit 3\n')
    stub.chmod(0o755)
    out = probe / 'out'
    run = subprocess.run([str(BIN), 'render', 'all', '--timeline-only', '--fps', '60', '--narration', str(folder), '--out', str(out)],
                         cwd=R, env={**os.environ, 'PATH': f"{probe}:{os.environ['PATH']}"}, capture_output=True, text=True, timeout=300)
    assert run.returncode != 0 and not out.exists(), 'Binary went past its narration gate'
    argv = (probe / 'python3.argv').read_text().splitlines()
    assert argv[:2] == ['-I', '-B'] and Path(argv[2]).samefile(TOOLS / 'narration_delivery.py'), f'Binary call differs: {argv[:3]}'
    assert argv[3] == '--topic' and Path(argv[4]).samefile(C) and argv[5:7] == ['--narration', str(folder)], f'Binary call differs: {argv[3:7]}'
    assert argv[7:9] == ['--consumer', 'render'] and argv[9] == '--gain-db', f'Binary call differs: {argv[7:10]}'
    assert argv[11::2] == ['--chapter-key'] * 15 and argv[12::2] == list(M.CHAPTERS), 'Binary chapter keys differ'
    return argv


def prove(folder):
    """Both tools, unmodified, called as their owners call them. N must still be refused."""
    refused = subprocess.run(tool_argv(N1), cwd=R, capture_output=True, text=True, timeout=900)
    assert refused.returncode != 0 and REFUSAL in refused.stderr, f'Frozen N is no longer refused: {refused.stderr.strip()[-200:]}'
    run = subprocess.run(tool_argv(folder), cwd=R, capture_output=True, text=True, timeout=900)
    assert run.returncode == 0, f'coherent_narration refused N2: {run.stderr.strip()[-300:]}'
    call = binary_call(folder)
    delivery = subprocess.run([sys.executable, *call], cwd=R, capture_output=True, text=True, timeout=900)
    assert delivery.returncode == 0, f'narration_delivery refused N2: {delivery.stderr.strip()[-300:]}'
    result = json.loads(delivery.stdout)
    assert (result['consumer'], result['verified_cues'], result['selected_cues'], result['plan_mode']) == ('render', CUES, CUES, 'external-ui'), f'Delivery differs: {result}'
    return {'standalone_refuses_N': True, 'standalone_passes': run.stdout.strip(), 'binary_tool': call[2], 'binary_topic': call[4],
            'binary_gain_db': call[10], 'delivery_consumer': result['consumer'], 'delivery_cues': result['verified_cues'], 'plan_mode': result['plan_mode']}


def child_gate(stage):
    """The guard rewrites the live receipt just after Popen, so a read can catch it half written.
    Only that parse failure is retried, briefly. Every gate in the sealed function still runs."""
    for _ in range(100):
        try:
            return sealed_child_gate(stage)
        except json.JSONDecodeError:
            time.sleep(0.05)
    return sealed_child_gate(stage)


def run_derive():
    child_gate('derive')
    check_qualified()
    assert not N2.exists(), 'Existing N2 retained'
    report, assembly, renames, tally = derived_bytes()
    write_folder(N2, report, assembly, shutil.copyfile)
    check_folder(N2, report, assembly)
    proof = prove(N2)
    with (M.job('derive') / 'derive-summary.json').open('x') as handle:
        json.dump({**tally, 'files_total': FILES, 'files_changed': ['assembly.json', 'report.json'], 'files_identical': FILES - 2,
                   'narration': str(N2), 'derived_from': str(N1), 'report_sha256': digest(report), 'source_report_sha256': sha(N1 / 'report.json'),
                   'assembly_sha256': digest(assembly), 'source_assembly_sha256': sha(N1 / 'assembly.json'), 'tools': proof, 'source_map': renames,
                   'scope': 'Path spelling of report.source and the assembly hash of that report only. Takes, timing, text and receipts are unchanged. No listening, picture, fine sync or AV claim.'}, handle, indent=2)
    print('DERIVE PASS: 105 cues, 414 rows, 2 of 110 files changed; both unmodified tools pass N2.', flush=True)


def check_derived():
    Q.success(DERIVE)
    report, assembly, renames, tally = derived_bytes()
    check_folder(N2, report, assembly)
    summary = read(M.job('derive') / 'derive-summary.json')
    assert summary['source_map'] == renames and (summary['cues'], summary['rows'], summary['files_identical']) == (CUES, ROWS, FILES - 2), 'Derive summary differs'
    assert summary['report_sha256'] == digest(report) and summary['assembly_sha256'] == digest(assembly), 'Derive summary differs'
    return tally


def mapped(path, table):
    """The new table in the macOS spelling. Only timeline `line` column 4 and cue-log column 0 can
    change, and only from exactly N2/<cue>.wav to N/<cue>.wav. Anything else stays and fails."""
    lines, cells = [], 0
    for line in M.normalized(path):
        row = line.split('\t')
        at = 4 if table == 'timeline' and row[0] == 'line' else 0 if table == 'cuelog' else None
        if at is not None:
            cue = row[1] if at == 4 else Path(row[0]).stem
            if row[at] == f'{N2}/{cue}.wav':
                row[at] = f'{N1}/{cue}.wav'
                cells += 1
        lines.append('\t'.join(row))
    return lines, cells


def compare_table(chapter, table, out):
    name = f'{chapter}.{table}.tsv'
    old = M.normalized(M.PRIOR_OUT / name)
    new, cells = mapped(out / name, table)
    return {'table': name, 'lines': len(old), 'normalized_equal': old == new, 'path_cells_mapped': cells,
            'prior_sha256': sha(M.PRIOR_OUT / name), 'new_sha256': sha(out / name)}


def compare_prior(out):
    """Written before it is judged, so a mismatch leaves its evidence."""
    rows = [compare_table(chapter, table, out) for chapter in M.PRIOR_CHAPTERS for table in M.TABLES]
    with (M.job('timeline') / 'prior-table-comparison.json').open('x') as handle:
        json.dump({'prior_job': M.FAILED.name, 'new_binary_sha256': Q.BIN_SHA256, 'mapped_from': str(N2), 'mapped_to': str(N1),
                   'mapped_columns': {'timeline': 'line rows, column 4', 'cuelog': 'column 0', 'cues': 'none'}, 'tables': rows}, handle, indent=2)
    different = [row['table'] for row in rows if not row['normalized_equal']]
    assert not different, f'ts1 tables differ from the macOS run: {different}'
    assert sum(row['path_cells_mapped'] for row in rows) == PRIOR_PATH_CELLS, 'Mapped path cells differ'
    return rows


def check_qualified():
    sealed_check_qualified()
    check_timeline01()


def check_upstream(stage):
    if stage == 'derive':
        check_qualified()
        M.check_failed_run()
    else:
        check_derived()
        sealed_check_upstream(stage)


def base_rows():
    """N stays frozen in every manifest, whichever folder the stages read."""
    M.N = N1
    try:
        return sealed_base_rows()
    finally:
        M.N = N2


def stage_files(stage):
    if stage == 'derive':
        files = sealed_stage_files('timeline')
        for path in (CHAIN01_ROW, OLD_MANIFEST, OLD_EXIT, OLD_LOG, *Q.job_files(OLD_JOB)):
            files[str(path)] = sha(path)
        return files
    if stage != 'timeline':
        return sealed_stage_files(stage)
    files = {row['path']: row['sha256'] for row in read(M.manifest('derive'))['files']}
    for path, value in files.items():
        assert sha(path) == value, path
    derived = [N2 / name for name in names()]
    for path in (M.manifest('derive'), ME, START, *derived, M.job('derive') / 'derive-summary.json', *Q.job_files(DERIVE)):
        files[str(path)] = sha(path)
    return files


def preflight(stage):
    check_timeline01()
    M.preflight(stage)
    if stage == 'derive':
        assert not N2.exists(), 'Existing N2 retained'
        report, assembly, _, tally = derived_bytes()
        folder = Path(tempfile.mkdtemp(prefix='chain02-n2-dry-')) / 'narration-n2'
        write_folder(folder, report, assembly, os.symlink)
        check_folder(folder, report, assembly)
        print(json.dumps({'derivation': tally, 'tools': prove(folder), 'dry_folder': str(folder)}))


def run(stage):
    if stage == 'derive':
        run_derive()
        return
    check_derived()
    {'timeline': M.run_timeline, 'score': M.run_score, 'preview': M.run_preview, 'pictures': M.run_pictures}[stage]()


M.NAMES.update({stage: f'complex-remote-{stage}02' for stage in M.NAMES}, derive=DERIVE)
M.UPSTREAM.update(derive=(), timeline=('derive',), score=('derive', 'timeline'), preview=('derive', 'timeline', 'score'),
                  pictures=('derive', 'timeline', 'score', 'preview'))
M.ME, M.START, M.N = ME, START, N2
M.TIMELINE_OUT = JOBS / M.NAMES['timeline'] / 'timeline-output'
M.SCORE_OUT = JOBS / M.NAMES['score'] / 'scored'
M.PREVIEW_OUT = JOBS / M.NAMES['preview'] / 'preview'
M.PICTURES_OUT = JOBS / M.NAMES['pictures'] / 'pictures'
M.exit_file = lambda stage: O / f'chain-{stage}-controller02.exit'
M.manifest = lambda stage: O / f'chain-{stage}-inputs02.json'
M.check_qualified, M.check_upstream, M.base_rows, M.child_gate = check_qualified, check_upstream, base_rows, child_gate
M.stage_files, M.compare_prior = stage_files, compare_prior

if __name__ == '__main__':
    assert len(sys.argv) == 3, __doc__
    mode, name = sys.argv[1:]
    if mode == '--compare':
        assert name in M.CHAPTERS, __doc__
        for table_name in M.TABLES:
            print(json.dumps(compare_table(name, table_name, M.TIMELINE_OUT)))
    else:
        assert mode in ('--check', '--launch', '--run') and name in M.NAMES, __doc__
        {'--check': preflight, '--launch': M.launch, '--run': run}[mode](name)
