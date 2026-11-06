"""Derive, timeline, score, preview and pictures for the stage07 complex binary on ts1.

One file, from the sealed chain01 and chain02-n2 with every check kept. What changes: the stage07 narration and tree, the
new binary from qualify02, new job names, and the prior comparison. timeline02, the stage06 wording on the old binary,
is now the baseline: 14 chapters must equal it under one exact path mapping, and 09_boundary must differ from it only as
compare_boundary says. Nothing synthesizes, and viewing stays a separate human step.

    --check STAGE      read-only; for derive also a dry N2, both unmodified tools and the binary call
    --launch STAGE     wait for upstream exit files, freeze, run ts1's guard (start-chain03.sh)
    --run STAGE        the guard's child; never call it by hand
    --compare CHAPTER  read-only: one chapter's tables against timeline02, under the same rules
"""
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import re
import shutil
import struct
import subprocess
import sys
import tempfile
import time
import wave

HERE = Path(__file__).resolve().parent
QUALIFIER = HERE.parent / 'qualify02/ts1_qualify02.py'
QUALIFIER_SHA256 = '604ef33814feb1fd6f79166b29593642f1bb44fc83e12ea92dd13c7a1e4460ca'
SPEC = importlib.util.spec_from_file_location('ts1_qualify', QUALIFIER)
Q = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(Q)
R, O, JOBS, C, BIN, CHAPTERS, POLICY, read, sha = Q.R, Q.O, Q.JOBS, Q.C, Q.BIN, Q.CHAPTERS, Q.POLICY, Q.read, Q.sha

SCRIPT_SHA256 = '946a9c07a462eb066fc8d20c6a092d5b7564d38752d26177bdbbcf15c6027cdc'
BRIDGE_SHA256 = 'b50479b62953fba7da2c2a8af0653030f926a5fcbacb5fccabe968aed8d0427a'  # the tool that wrote the rows file
COHERENT_SHA256 = '95615cfadb42fbfdddf828340b74ec66d89c975a1f399593e212bdd9840e65d5'
BASE_BIN_SHA256 = '3c90fb5464fc8736214228d62f1c322629ede230cd58cb11ddb947e5d9e9c168'  # timeline02's binary
CUES, ROWS, FILES = 105, 414, 110
AFFECTED, REPAIRED = '09_boundary', '09_5'
REFUSAL = 'coherent-narration: assembly sentence/source spans differ from the supplied script and generation receipts'
PILLOW = '12.1.1'
TIMELINE_SECONDS = PREVIEW_SECONDS = 172800  # a bound, not an estimate; see INTEGRATION.md
SCORE_SECONDS = 7200
WAIT_POLLS = 30240  # 7 days at 20 s, because each upstream limit is 48 h

ME = R / Path(__file__).resolve().relative_to(Q.PHYSICAL)
START = ME.parent / 'start-chain03.sh'
EC = R / Q.EC
NATIVE = EC / 'native-stage07-root01'
TOOLS = NATIVE / 'workspace/tools'
N1 = EC / 'production-audio-stage07-01/native-narration01'
AUDIO_ROWS = O / 'stage07-rows01.json'  # hash frozen into every manifest; see INTEGRATION.md
BASE_JOB = 'complex-remote-timeline02'
BASE_OUT = JOBS / BASE_JOB / 'timeline-output'
BASE_N2 = JOBS / 'complex-remote-narration-derive01/narration-n2'
BASE_MANIFEST = O / 'chain-timeline-inputs02.json'
BASE_EXIT = O / 'chain-timeline-controller02.exit'
UNAFFECTED = tuple(chapter for chapter in CHAPTERS if chapter != AFFECTED)
TABLES = ('timeline', 'cues', 'cuelog')
PICTURE_ENV = 'complex-remote-picture-env01'
PICTURE_PYTHON = JOBS / PICTURE_ENV / 'env/bin/python'
DERIVE = 'complex-remote-narration-derive02'
NAMES = {'derive': DERIVE, 'timeline': 'complex-remote-timeline03', 'score': 'complex-remote-score03',
         'preview': 'complex-remote-preview03', 'pictures': 'complex-remote-pictures03'}
UPSTREAM = {'derive': (), 'timeline': ('derive',), 'score': ('derive', 'timeline'), 'preview': ('derive', 'timeline', 'score'),
            'pictures': ('derive', 'timeline', 'score', 'preview')}
N2 = N = JOBS / DERIVE / 'narration-n2'
TIMELINE_OUT = JOBS / NAMES['timeline'] / 'timeline-output'
SCORE_OUT = JOBS / NAMES['score'] / 'scored'
PREVIEW_OUT = JOBS / NAMES['preview'] / 'preview'
PICTURES_OUT = JOBS / NAMES['pictures'] / 'pictures'


def job(stage):
    return JOBS / NAMES[stage]


def exit_file(stage):
    return O / f'chain-{stage}-controller03.exit'


def manifest(stage):
    return O / f'chain-{stage}-inputs03.json'


def child_command(stage):
    python = str(PICTURE_PYTHON) if stage == 'pictures' else sys.executable
    return [python, '-I', '-B', str(ME), '--run', stage]


def guard_command(stage):
    return [sys.executable, '-I', '-B', str(Q.GUARD), '--phase', 'native', '--job', NAMES[stage],
            '--inputs', str(manifest(stage)), '--', *child_command(stage)]


def tool(name):
    found = shutil.which(name)
    assert found, 'Missing tool: ' + name
    return Path(found).resolve()


def normalized(path):
    return [line for line in path.read_text().splitlines() if line and not line.startswith('#')]


def table_value(rows, key):
    found = [row[1] for row in rows if row[0] == key]
    assert len(found) == 1, key
    return found[0]


def script_ids():
    return [line.split('\t', 1)[0] for line in (C / 'script.tsv').read_text().splitlines()
            if line.strip() and not line.startswith('#')]


def check_pins():
    assert sha(QUALIFIER) == QUALIFIER_SHA256, 'Sealed qualifier changed'
    rows = read(AUDIO_ROWS)
    assert rows['tool_sha256'] == BRIDGE_SHA256, 'Audio rows came from another tool'
    assert rows['script_sha256'] == SCRIPT_SHA256 == sha(C / 'script.tsv'), 'Audio rows bind another script'
    comparison = rows['comparison']
    assert (comparison['cues_identical'], comparison['cue_changed'], comparison['report_rows']) == (CUES - 1, REPAIRED, ROWS), 'Audio rows differ'


def check_qualified():
    """Downstream needs the clean build, with its inputs, log, facts and both binaries."""
    Q.check_build()


def check_baseline():
    """timeline02 stays what it is: the stage06 wording on the old binary. Here it only supplies expected values."""
    d = read(JOBS / BASE_JOB / 'resource-receipt.json')
    Q.success(BASE_JOB)
    assert d.get('inputs_sha256') == sha(BASE_MANIFEST), 'timeline02 inputs changed'
    assert BASE_EXIT.read_text().strip() == '0', 'timeline02 controller changed'
    s = read(JOBS / BASE_JOB / 'timeline-summary.json')
    assert s['binary_sha256'] == BASE_BIN_SHA256 and s['cues'] == CUES, 'timeline02 summary differs'
    assert [row['chapter'] for row in s['chapters']] == list(CHAPTERS), 'timeline02 chapters differ'
    for row in s['chapters']:
        assert sha(BASE_OUT / (row['chapter'] + '.timeline.tsv')) == row['timeline_sha256'], 'timeline02 table changed: ' + row['chapter']
    assert sorted(p.name for p in BASE_OUT.glob('*.tsv')) == sorted(f'{chapter}.{table}.tsv' for chapter in CHAPTERS for table in TABLES)


def base_rows():
    """The build manifest's rows and the rows file's. A file added to N1 or the workspace is refused as firmly as a changed one."""
    rows = [*read(Q.MANIFEST)['files'], *({'path': str(R / row['path']), 'sha256': row['sha256']} for row in read(AUDIO_ROWS)['files'])]
    files = {}
    for row in rows:
        assert files.setdefault(row['path'], row['sha256']) == row['sha256'], row['path']
    for root in (N1, NATIVE / 'workspace'):
        listed = {path for path in files if path.startswith(str(root) + '/')}
        different = sorted(listed ^ {str(p) for p in root.rglob('*') if p.is_file()})
        assert not different, f'File set changed: {len(different)} paths, first {different[:3]}'
    for path, digest in files.items():
        assert sha(path) == digest, path
    return files


def stage_files(stage):
    if stage == 'derive':
        files = base_rows()
        rows = read(Q.MANIFEST)['files']
        paths = [Q.MANIFEST, AUDIO_ROWS, ME, START, tool('ffmpeg'), Q.BIN, *Q.job_files(Q.JOB), BASE_MANIFEST, BASE_EXIT, *Q.job_files(BASE_JOB)]
    else:
        prior = UPSTREAM[stage][-1]
        files = {}
        rows = read(manifest(prior))['files']
        paths = [manifest(prior), ME, START, *Q.job_files(NAMES[prior])]
    if stage == 'preview':
        paths.append(tool('ffprobe'))
    if stage == 'pictures':
        paths += [tool('ffmpeg'), *Q.job_files(PICTURE_ENV)]
    for row in rows:
        assert files.setdefault(row['path'], row['sha256']) == row['sha256'], row['path']
        assert sha(row['path']) == row['sha256'], row['path']
    for path in paths:
        files[str(path)] = sha(path)
    return files


def child_gate(stage):
    """The guard rewrites the live receipt just after Popen, so a read can catch it half written. Only that parse failure
    is retried, briefly."""
    for _ in range(100):
        try:
            d = read(job(stage) / 'resource-receipt.json')
            break
        except json.JSONDecodeError:
            time.sleep(0.05)
    else:
        d = read(job(stage) / 'resource-receipt.json')
    assert d.get('admitted') is True and 'exit' not in d
    assert d['command'] == child_command(stage)
    assert d['policy'] == POLICY, 'Guard policy changed'
    rows = read(manifest(stage))['files']
    for row in rows:
        assert sha(row['path']) == row['sha256'], row['path']
    assert str(tool('ffmpeg')) in {row['path'] for row in rows}, 'Child resolves an ffmpeg outside the manifest'


def check_facts(folder):
    """The macOS checks, with the oracle under its isolated env. Both outputs are kept first."""
    facts = {}
    for name, argv in [('native', [str(BIN), 'facts']), ('oracle', [str(Q.ENV_PYTHON), '-I', '-B', str(Q.ORACLE)])]:
        result = subprocess.run(argv, cwd=C, capture_output=True, timeout=900)
        for stream, data in [('stdout', result.stdout), ('stderr', result.stderr)]:
            with (folder / f'facts-{name}.{stream}').open('xb') as handle:
                handle.write(data)
        assert result.returncode == 0, (name, result.returncode)
        facts[name] = result.stdout
    assert facts['native'] and facts['native'] == facts['oracle'], 'Native facts differ from oracle; both outputs retained'
    assert facts['native'] == (JOBS / Q.BUILD / 'facts-native.tsv').read_bytes() == Q.RELEASE_FACTS.read_bytes(), 'ts1 facts differ from the build or macOS'
    keys = [line.split(b'\t', 1)[0] for line in facts['native'].splitlines()]
    assert len(keys) == len(set(keys))
    return len(keys)


def verify_narration():
    subprocess.run([sys.executable, '-I', '-B', str(TOOLS / 'coherent_narration.py'),
                    'verify', str(N), '--script', str(C / 'script.tsv'), '--units',
                    str(N / 'external-units.json'), '--require-directions'], cwd=R, check=True, timeout=900)


def digest(data):
    return hashlib.sha256(data).hexdigest()


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
    probe = Path(tempfile.mkdtemp(prefix='chain03-probe-'))
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
    assert argv[11::2] == ['--chapter-key'] * 15 and argv[12::2] == list(CHAPTERS), 'Binary chapter keys differ'
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


def run_derive():
    child_gate('derive')
    check_qualified()
    check_baseline()
    assert not N2.exists(), 'Existing N2 retained'
    report, assembly, renames, tally = derived_bytes()
    write_folder(N2, report, assembly, shutil.copyfile)
    check_folder(N2, report, assembly)
    proof = prove(N2)
    with (job('derive') / 'derive-summary.json').open('x') as handle:
        json.dump({**tally, 'files_total': FILES, 'files_changed': ['assembly.json', 'report.json'], 'files_identical': FILES - 2,
                   'narration': str(N2), 'derived_from': str(N1), 'report_sha256': digest(report), 'source_report_sha256': sha(N1 / 'report.json'),
                   'assembly_sha256': digest(assembly), 'source_assembly_sha256': sha(N1 / 'assembly.json'), 'tools': proof, 'source_map': renames,
                   'scope': 'Path spelling of report.source and the assembly hash of that report only. Takes, timing, text and receipts are unchanged. No listening, picture, fine sync or AV claim.'}, handle, indent=2)
    print('DERIVE PASS: 105 cues, 414 rows, 2 of 110 files changed; both unmodified tools pass N2.', flush=True)


def check_derived():
    Q.success(DERIVE)
    report, assembly, renames, tally = derived_bytes()
    check_folder(N2, report, assembly)
    summary = read(job('derive') / 'derive-summary.json')
    assert summary['source_map'] == renames and (summary['cues'], summary['rows'], summary['files_identical']) == (CUES, ROWS, FILES - 2), 'Derive summary differs'
    assert summary['report_sha256'] == digest(report) and summary['assembly_sha256'] == digest(assembly), 'Derive summary differs'
    return tally


def mapped(path, table):
    """The new table in timeline02's spelling. Only timeline `line` column 4 and cue-log column 0 can change, and only from
    exactly N2/<cue>.wav to BASE_N2/<cue>.wav. Anything else stays and fails."""
    lines, cells = [], 0
    for line in normalized(path):
        row = line.split('\t')
        at = 4 if table == 'timeline' and row[0] == 'line' else 0 if table == 'cuelog' else None
        if at is not None:
            cue = row[1] if at == 4 else Path(row[0]).stem
            if row[at] == f'{N2}/{cue}.wav':
                row[at] = f'{BASE_N2}/{cue}.wav'
                cells += 1
        lines.append('\t'.join(row))
    return lines, cells


def compare_table(chapter, table, out):
    name = f'{chapter}.{table}.tsv'
    old = normalized(BASE_OUT / name)
    new, cells = mapped(out / name, table)
    return {'table': name, 'lines': len(old), 'normalized_equal': old == new, 'path_cells_mapped': cells,
            'baseline_sha256': sha(BASE_OUT / name), 'new_sha256': sha(out / name)}


def split_rows(lines):
    return [line.split('\t') for line in lines]


def compare_boundary(out):
    """The one expected change, as rules. Timeline tables list rows by kind, so a row's frame says which side it is on.

    At or before 09_5's baseline frame: every row equal under the mapping, except 09_5 itself.
    09_5 is new: its start frame must equal the baseline's. Its hash, size, length and sample count may change, and its
    hash, size and samples must be this N2's.
    After that frame: one whole-frame shift, the same for every row, and every other cell equal. Cue seconds are three
    decimals, so each is recomputed from its shifted frame, and never by subtraction."""
    problems = []
    old_lines = split_rows(normalized(BASE_OUT / f'{AFFECTED}.timeline.tsv'))
    new_lines, timeline_cells = mapped(out / f'{AFFECTED}.timeline.tsv', 'timeline')
    new_lines = split_rows(new_lines)
    starts = {row[1]: int(row[2]) for row in old_lines if row[0] == 'line'}
    found = {row[1]: int(row[2]) for row in new_lines if row[0] == 'line'}
    if list(starts) != list(found):
        problems.append('09_boundary cues differ')
        return {'problems': problems, 'path_cells_mapped': timeline_cells, 'frame_shift': None}
    cues = list(starts)
    pivot = starts[REPAIRED]
    if found[REPAIRED] != pivot:
        problems.append('09_5 itself moved, so something before it changed')
    following = cues[cues.index(REPAIRED) + 1]
    shift = found[following] - starts[following]
    frame_column = {'line': 2, 'mark': 2, 'breath': 1, 'listen': 1}

    def moved(table, old, new, column):
        expect = list(old)
        expect[column] = str(int(old[column]) + shift)
        if new != expect:
            problems.append(f'{table}: {new} is not {expect}')

    if len(old_lines) != len(new_lines):
        problems.append('timeline rows differ in number')
    for old, new in zip(old_lines, new_lines):
        kind = old[0]
        if kind == 'frames':
            moved('timeline', old, new, 1)
        elif kind == 'line' and old[1] == REPAIRED:
            if new[:2] != old[:2] or new[3:] != old[3:]:
                problems.append(f'timeline: {new} is not 09_5 with a new frame only')
        elif kind in frame_column:
            if int(old[frame_column[kind]]) > pivot:
                moved('timeline', old, new, frame_column[kind])
            elif new != old:
                problems.append(f'timeline: {new} moved; {old} comes before 09_5')
        elif new != old:
            problems.append(f'timeline: {new} is not {old}')
    old_rows, new_rows = split_rows(normalized(BASE_OUT / f'{AFFECTED}.cues.tsv')), split_rows(normalized(out / f'{AFFECTED}.cues.tsv'))
    if [row[0] for row in old_rows] != [row[0] for row in new_rows]:
        problems.append('cues rows differ')
    for old, new in zip(old_rows, new_rows):
        seconds = f'{found[old[0]] / 60:.3f}'
        if old[0] == REPAIRED:
            if new[1] != seconds or float(new[2]) <= 0:
                problems.append(f'cues: {new} does not start at frame {found[REPAIRED]}')
        elif starts[old[0]] > pivot:
            if found[old[0]] != starts[old[0]] + shift or new != [old[0], seconds, old[2]]:
                problems.append(f'cues: {new} is not {old[0]} at {seconds} with length {old[2]}')
        elif new != old:
            problems.append(f'cues: {new} moved; {old} comes before 09_5')
    old_log = split_rows(normalized(BASE_OUT / f'{AFFECTED}.cuelog.tsv'))
    log_lines, log_cells = mapped(out / f'{AFFECTED}.cuelog.tsv', 'cuelog')
    new_log = split_rows(log_lines)
    if [Path(row[0]).stem for row in old_log] != [Path(row[0]).stem for row in new_log]:
        problems.append('cue log rows differ')
    wav = N / f'{REPAIRED}.wav'
    with wave.open(str(wav)) as handle:
        samples = handle.getnframes()
    for old, new in zip(old_log, new_log):
        frame = int(old[3])
        if Path(old[0]).stem == REPAIRED:
            expect = [old[0], sha(wav), str(wav.stat().st_size), str(found[REPAIRED]), *old[4:11], str(samples),
                      str(int(old[12]) + (found[REPAIRED] - frame) * 800)]
        elif frame > pivot:
            expect = [*old[:3], str(frame + shift), *old[4:12], str(int(old[12]) + shift * 800)]
        else:
            expect = old
        if new != expect:
            problems.append(f'cuelog: {new} is not {expect}')
    return {'problems': problems, 'frame_shift': shift, 'repaired_frame': {'baseline': pivot, 'new': found[REPAIRED]},
            'cues': len(cues), 'path_cells_mapped': timeline_cells + log_cells}


def expected_cells():
    """Timeline `line` rows and cue-log rows of the unaffected chapters, counted in the baseline's own tables."""
    count = 0
    for chapter in UNAFFECTED:
        count += sum(1 for row in split_rows(normalized(BASE_OUT / f'{chapter}.timeline.tsv')) if row[0] == 'line')
        count += len(normalized(BASE_OUT / f'{chapter}.cuelog.tsv'))
    return count


def compare_baseline(out):
    """Written before it is judged, so a mismatch leaves its evidence."""
    rows = [compare_table(chapter, table, out) for chapter in UNAFFECTED for table in TABLES]
    boundary = compare_boundary(out)
    expected = expected_cells()
    with (job('timeline') / 'baseline-table-comparison.json').open('x') as handle:
        json.dump({'baseline_job': BASE_JOB, 'baseline_binary_sha256': BASE_BIN_SHA256, 'new_binary_sha256': Q.BIN_SHA256,
                   'mapped_from': str(N2), 'mapped_to': str(BASE_N2), 'mapped_columns': {'timeline': 'line rows, column 4', 'cuelog': 'column 0', 'cues': 'none'},
                   'unaffected_chapters': list(UNAFFECTED), 'expected_path_cells': expected, 'tables': rows, 'chapter09': boundary}, handle, indent=2)
    different = [row['table'] for row in rows if not row['normalized_equal']]
    assert not different, f'ts1 tables differ from timeline02: {different}'
    assert not boundary['problems'], f"09_boundary differs beyond the expected change: {boundary['problems'][:3]}"
    assert sum(row['path_cells_mapped'] for row in rows) == expected == 2 * (CUES - boundary['cues']), 'Mapped path cells differ'
    assert boundary['path_cells_mapped'] == 2 * boundary['cues'], '09_boundary path cells differ'
    return rows, boundary


def check_tables(out, script):
    seen, chapters = [], []
    for chapter in CHAPTERS:
        path = out / (chapter + '.timeline.tsv')
        rows = [line.split('\t') for line in path.read_text().splitlines()]
        assert table_value(rows, 'chapter') == chapter and table_value(rows, 'fps') == '60'
        frames = int(table_value(rows, 'frames'))
        assert frames > 0
        lines = [row for row in rows if row[0] == 'line']
        starts = [int(row[2]) for row in lines]
        assert starts == sorted(starts) and all(0 <= frame < frames for frame in starts)
        for row in lines:
            assert Path(row[4]).resolve() == (N / (row[1] + '.wav')).resolve()
        seen.extend(row[1] for row in lines)
        chapters.append({'chapter': chapter, 'frames': frames, 'cues': [row[1] for row in lines],
                         'timeline_sha256': sha(path)})
    assert seen == script, 'Cue placements differ from the full ordered script'
    return chapters, seen


def run_timeline():
    child_gate('timeline')
    check_qualified()
    check_baseline()
    listed = subprocess.check_output([str(BIN), 'chapters'], text=True).splitlines()
    assert listed == list(CHAPTERS), listed
    script = script_ids()
    assert len(script) == len(set(script)) == CUES
    fact_rows = check_facts(job('timeline'))
    verify_narration()
    assert not TIMELINE_OUT.exists(), 'Existing timeline output retained'
    subprocess.run([str(BIN), 'render', 'all', '--timeline-only', '--fps', '60', '--narration', str(N),
                    '--out', str(TIMELINE_OUT)], cwd=R, check=True, timeout=TIMELINE_SECONDS)
    compared, boundary = compare_baseline(TIMELINE_OUT)
    chapters, seen = check_tables(TIMELINE_OUT, script)
    with (job('timeline') / 'timeline-summary.json').open('x') as handle:
        json.dump({'chapters': chapters, 'cues': len(seen), 'fps': 60, 'native_oracle_fact_rows_matched': fact_rows,
                   'total_frames': sum(row['frames'] for row in chapters), 'binary_sha256': Q.BIN_SHA256,
                   'profile': 'release', 'host': 'ts1', 'qualifier_job': Q.JOB, 'baseline_job': BASE_JOB,
                   'baseline_binary_sha256': BASE_BIN_SHA256, 'unaffected_chapters': list(UNAFFECTED), 'baseline_tables_compared': len(compared),
                   'chapter09_frame_shift': boundary['frame_shift'], 'chapter09_repaired_frame': boundary['repaired_frame'],
                   'full_baseline_equivalence_established': False,
                   'scope': 'ts1 release native timeline of stage07. Tables for 14 chapters equal timeline02 under one exact path mapping. In 09_boundary rows up to 09_5 are equal, 09_5 is new, and later rows moved by one whole-frame shift. No speedup, frame, PCM, listening or AV equivalence established.'}, handle, indent=2)
    print(f"TIMELINE PASS: 15 chapters, 105 cues; {len(compared)} baseline tables equal; 09_boundary later rows moved {boundary['frame_shift']} frames.", flush=True)


def check_timeline():
    Q.success(NAMES['timeline'])
    summary = read(job('timeline') / 'timeline-summary.json')
    assert summary['binary_sha256'] == Q.BIN_SHA256 == sha(BIN) and summary['profile'] == 'release', 'Timeline binary differs'
    assert summary['baseline_job'] == BASE_JOB, 'Timeline names another baseline'
    assert summary['full_baseline_equivalence_established'] is False, 'Full baseline equivalence cannot be claimed'
    assert summary['unaffected_chapters'] == list(UNAFFECTED)
    assert [row['chapter'] for row in summary['chapters']] == list(CHAPTERS)
    assert summary['cues'] == CUES and summary['fps'] == 60
    assert summary['native_oracle_fact_rows_matched'] > 0
    folder = TIMELINE_OUT
    assert {p.stem.removesuffix('.timeline') for p in folder.glob('*.timeline.tsv')} == set(CHAPTERS)
    assert all((folder / f'{chapter}.{table}.tsv').is_file() for chapter in CHAPTERS for table in TABLES)
    compared = read(job('timeline') / 'baseline-table-comparison.json')
    assert compared['baseline_job'] == BASE_JOB and compared['new_binary_sha256'] == Q.BIN_SHA256
    assert len(compared['tables']) == summary['baseline_tables_compared'] == 3 * len(UNAFFECTED)
    assert compared['chapter09']['problems'] == [] and compared['chapter09']['frame_shift'] == summary['chapter09_frame_shift']
    for row in compared['tables']:
        assert row['normalized_equal'] is True, 'Baseline table differs: ' + row['table']
        assert sha(folder / row['table']) == row['new_sha256'], 'New table changed: ' + row['table']
        assert sha(BASE_OUT / row['table']) == row['baseline_sha256'], 'Baseline table changed: ' + row['table']
    seen = []
    total = 0
    for item in summary['chapters']:
        path = folder / (item['chapter'] + '.timeline.tsv')
        assert sha(path) == item['timeline_sha256']
        rows = [line.split('\t') for line in path.read_text().splitlines()]
        assert table_value(rows, 'chapter') == item['chapter'] and table_value(rows, 'fps') == '60'
        frames = int(table_value(rows, 'frames'))
        assert frames == item['frames'] and frames > 0
        cues = [row for row in rows if row[0] == 'line']
        assert [row[1] for row in cues] == item['cues']
        starts = [int(row[2]) for row in cues]
        assert starts == sorted(starts) and all(0 <= n < frames for n in starts)
        assert all(Path(row[4]).resolve() == (N / (row[1] + '.wav')).resolve() for row in cues)
        seen.extend(row[1] for row in cues)
        total += frames
    ordered = script_ids()
    assert seen == ordered and len(seen) == len(set(seen)) == CUES
    assert total == summary['total_frames']
    return folder


def run_score():
    child_gate('score')
    check_qualified()
    folder = check_timeline()
    assert not SCORE_OUT.exists(), 'Preserve existing score outputs'
    subprocess.run([str(BIN), 'score', '--narration', str(N), '--timelines', str(folder), '--out', str(SCORE_OUT)],
                   cwd=R, check=True, timeout=SCORE_SECONDS)
    qa_folder = SCORE_OUT / 'stems/qa'
    assert {p.stem for p in qa_folder.glob('*.json')} == set(CHAPTERS)
    qa = {}
    for chapter in CHAPTERS:
        path = qa_folder / (chapter + '.json')
        report = read(path)
        assert report['chapter'] == chapter and report['passed'] is True, chapter
        assert report['metrics'] and all(m.get('pass') is True or m.get('pass') is None for m in report['metrics']), chapter
        qa[chapter] = sha(path)
    with (job('score') / 'score-summary.json').open('x') as handle:
        json.dump({'qa_sha256': qa, 'chapters': 15, 'cues': 105,
                   'scope': 'Native score QA only. Listening, actual pictures, fine sync and independent final AV review remain open.'}, handle, indent=2)
    print('SCORE PASS: all 15 registry chapter QA reports passed.', flush=True)


def check_score():
    Q.success(NAMES['score'])
    timeline = check_timeline()
    summary = read(job('score') / 'score-summary.json')
    assert list(summary['qa_sha256']) == list(CHAPTERS)
    for chapter, digest in summary['qa_sha256'].items():
        path = SCORE_OUT / 'stems/qa' / (chapter + '.json')
        report = read(path)
        assert sha(path) == digest and report['chapter'] == chapter and report['passed'] is True
        assert report['metrics'] and all(m['pass'] is True or m['pass'] is None for m in report['metrics'])
    return timeline


def score_files(chapter, frames):
    """What the kit lays for this chapter, in order: (file, first sample, samples)."""
    stem = SCORE_OUT / 'stems' / (chapter + '.music.f32.wav')
    parts = stem.with_suffix('.parts.tsv')
    if not parts.exists():
        return [(stem, 0, frames * 800)]
    rows = [line.split('\t') for line in parts.read_text().splitlines()[1:]]
    return [(stem.with_name(row[4]), int(row[0]), int(row[1])) for row in rows]


def check_cue_log(chapter, frames, rendered, timeline):
    """A scored render logs the timeline's cues unchanged, then its own score at the last frame.
    At 60 fps a frame is 800 samples at 48 kHz, and a row starts at
    frame * 800 + round(offset * 48000), as narration_delivery.py computes it. The rows must run end
    to end with no gap or overlap and fill the chapter exactly."""
    base, rows = normalized(timeline), normalized(rendered)
    assert rows[:len(base)] == base, 'Preview cue log differs: ' + rendered.name
    laid = [row.split('\t') for row in rows[len(base):]]
    files = score_files(chapter, frames)
    differ = 'Preview score rows differ: ' + rendered.name
    assert len(laid) == len(files), differ
    end = 0
    for row, (path, start, samples) in zip(laid, files):
        offset = float(row[5])
        exact = struct.unpack('>Q', struct.pack('>d', offset))[0]
        assert row[:5] == [str(path), sha(path), str(path.stat().st_size), str(frames), '60'], differ
        assert row[6:] == [f'{exact:#018x}', '0.0', '0x0000000000000000', '48000', '2', str(samples), str(start)], differ
        assert frames * 800 + round(offset * 48000) == start == end, differ
        end += samples
    assert end == frames * 800, differ


def run_preview():
    child_gate('preview')
    check_qualified()
    timeline = check_score()
    probe_tool = tool('ffprobe')
    assert not PREVIEW_OUT.exists(), 'Existing preview must be preserved'
    subprocess.run([str(BIN), 'render', 'all', '--res', '960x540', '--fps', '60', '--narration', str(N),
                    '--score', str(SCORE_OUT / 'stems'), '--out', str(PREVIEW_OUT), '--lint', '--crf', '18',
                    '--preset', 'veryfast'], cwd=R, check=True, timeout=PREVIEW_SECONDS)
    chapters = []
    assert {p.stem for p in PREVIEW_OUT.glob('*.mp4')} == set(CHAPTERS)
    for chapter in CHAPTERS:
        table = PREVIEW_OUT / (chapter + '.timeline.tsv')
        for name in ('timeline', 'cues'):
            rendered = PREVIEW_OUT / f'{chapter}.{name}.tsv'
            assert normalized(rendered) == normalized(timeline / rendered.name), 'Preview table differs: ' + rendered.name
        movie = PREVIEW_OUT / (chapter + '.mp4')
        probe = json.loads(subprocess.check_output([str(probe_tool), '-v', 'error', '-show_streams', '-of', 'json', str(movie)], timeout=120))
        video = [s for s in probe['streams'] if s['codec_type'] == 'video']
        audio = [s for s in probe['streams'] if s['codec_type'] == 'audio']
        assert len(video) == len(audio) == 1
        v = video[0]
        assert (v['width'], v['height'], v['avg_frame_rate']) == (960, 540, '60/1')
        frames = int(next(line.split('\t')[1] for line in normalized(table) if line.startswith('frames\t')))
        assert int(v['nb_frames']) == frames
        check_cue_log(chapter, frames, PREVIEW_OUT / (chapter + '.cuelog.tsv'), timeline / (chapter + '.cuelog.tsv'))
        layout = PREVIEW_OUT / (chapter + '.layout.tsv')
        assert layout.is_file(), 'Rendered lint report missing: ' + layout.name
        chapters.append({'chapter': chapter, 'frames': frames, 'mp4_sha256': sha(movie), 'timeline_sha256': sha(table),
                         'layout_sha256': sha(layout),
                         'layout_findings': len([line for line in layout.read_text().splitlines() if line.strip()])})
    with (job('preview') / 'preview-summary.json').open('x') as handle:
        json.dump({'chapters': chapters, 'res': '960x540', 'fps': 60, 'voice': 'ElevenLabs Frederick Surrey',
                   'binary_sha256': Q.BIN_SHA256, 'timeline_job': NAMES['timeline'], 'tables_equal_to_timeline': ['timeline', 'cues'],
                   'cue_log': 'all timeline rows unchanged, then only this chapter score rows, every field checked',
                   'rendered_lint': True, 'full_baseline_equivalence_established': False,
                   'scope': 'Narrated preview rendered and schedules compared; actual picture review, listening, fine sync and independent AV review remain open. Not a publication master.'}, handle, indent=2)


def layout_paths(chapter, required_folders, optional_folders=()):
    required = [folder / (chapter + '.layout.tsv') for folder in required_folders]
    for path in required:
        assert path.is_file(), 'Required rendered layout report missing: ' + str(path)
    optional = [folder / (chapter + '.layout.tsv') for folder in optional_folders]
    return required + [path for path in optional if path.is_file()]


def select(chapter, rows, frames, phrases, sentences, layouts):
    selected = {}
    def add(seconds, reason):
        frame = round(seconds * 60)
        if 0 <= frame < frames:
            selected.setdefault(frame, set()).add(reason)
    for sec in range(1, frames // 60, 5):
        add(sec, 'five-second chapter progression')
    add((frames - 1) / 60, 'last frame')
    for row in rows:
        if row[0] == 'line':
            cue, start, wav = row[1], int(row[2]) / 60, Path(row[4])
            assert sha(wav) == phrases[cue]['wav_sha256'], cue
            with wave.open(str(wav)) as handle:
                duration = handle.getnframes() / handle.getframerate()
            for delta, label in [(-.2, 'before'), (.2, 'start'), (duration / 2, 'middle'),
                                 (max(0, duration - .1), 'end')]:
                add(start + delta, cue + ' ' + label)
            for index, sentence in enumerate(sentences[cue]):
                add(start + sentence['start'] + .5, f'{cue} provisional sentence {index+1}')
            for mark in phrases[cue]['marks']:
                for delta in (-.2, .5, 1.5):
                    add(start + mark['seconds'] + delta, f"{cue} provisional phrase byte{mark['byte']} +{delta}")
        elif row[0] == 'breath':
            begin, duration = int(row[1]) / 60, float(row[2])
            for delta in (0, duration / 2, max(0, duration - .1), duration + .5):
                add(begin + delta, 'authored hold +' + str(delta))
    for path in layouts:
        for line in path.read_text().splitlines():
            if not line.strip():
                continue
            match = re.search(r'@([0-9]+(?:\.[0-9]+)?)s', line)
            assert match, (path, line)
            anchor = float(match[1])
            deltas = [-.2, 0, .2, 1, 3, 6, 10]
            persistent = re.search(r'still there at (\d+) later looks', line)
            if persistent:
                count = int(persistent[1])
                deltas += [1 + .2 * count, 2 + .2 * count]
                deltas += list(range(5, max(5, int(frames / 60 - anchor) + 1), 5))
            for delta in deltas:
                add(anchor + delta, f'{path.name} {line} +{delta}s')
    return [{'chapter': chapter, 'frame': frame, 'seconds': frame / 60,
             'reasons': sorted(reasons)} for frame, reasons in sorted(selected.items())]


def run_pictures():
    from PIL import Image, ImageDraw  # only the picture env has Pillow
    child_gate('pictures')
    check_qualified()
    Q.success(NAMES['preview'])
    assert not PICTURES_OUT.exists(), 'Preserve earlier review pictures'
    ffmpeg = tool('ffmpeg')
    PICTURES_OUT.mkdir()
    phrases = read(N / 'phrase-timings.json')['cues']
    sentences = read(N / 'report.json')['lines']
    summary = read(job('preview') / 'preview-summary.json')
    assert [row['chapter'] for row in summary['chapters']] == list(CHAPTERS)
    all_samples = []
    layout_sources = {}
    for chapter_info in summary['chapters']:
        chapter = chapter_info['chapter']
        movie, table = PREVIEW_OUT / (chapter + '.mp4'), PREVIEW_OUT / (chapter + '.timeline.tsv')
        assert sha(movie) == chapter_info['mp4_sha256'] and sha(table) == chapter_info['timeline_sha256']
        rows = [line.split('\t') for line in normalized(table)]
        layouts = layout_paths(chapter, [PREVIEW_OUT])
        layout_sources[chapter] = [str(path) for path in layouts]
        samples = select(chapter, rows, chapter_info['frames'], phrases, sentences, layouts)
        folder = PICTURES_OUT / chapter
        folder.mkdir()
        for index, sample in enumerate(samples):
            dst = folder / f'{index:04}.png'
            subprocess.run([str(ffmpeg), '-v', 'error', '-xerror', '-nostdin', '-threads', '2',
                            '-ss', str(sample['seconds']), '-i', str(movie), '-frames:v', '1',
                            '-threads', '2', '-n', str(dst)], check=True, timeout=90)
            sample.update(picture=str(dst), sha256=sha(dst))
        for first in range(0, len(samples), 6):
            sheet = Image.new('RGB', (1920, 1740), '#0f0f14')
            draw = ImageDraw.Draw(sheet)
            for slot, sample in enumerate(samples[first:first+6]):
                x, y = slot % 2 * 960, slot // 2 * 580
                with Image.open(sample['picture']) as im:
                    assert im.size == (960, 540)
                    sheet.paste(im, (x, y))
                draw.text((x+8, y+545), f"{first+slot:04} {chapter} {sample['seconds']:.3f}s", fill='white')
                draw.text((x+8, y+560), ' | '.join(sample['reasons'])[:150], fill='white')
            sheet.save(folder / f'sheet-{first//6:03}.png')
        all_samples += samples
        print(chapter, len(samples), 'pictures extracted; actual viewing pending', flush=True)
    with (PICTURES_OUT / 'extraction.json').open('x') as handle:
        json.dump({'samples': all_samples, 'pictures_viewed': False, 'layout_sources': layout_sources,
                   'timeline_lint_note': 'Rendered preview lint reports only; timeline-only mode installs no linter.',
                   'scope': 'Preview samples only; all full reason labels retained. Persistence looks can be nonconsecutive. Not exhaustive review, listening, fine sync or independent acceptance.'}, handle, indent=2)


def check_tools(stage):
    tool('python3')
    if stage in ('timeline', 'preview'):  # the binary encodes through ffmpeg in both
        encoders = subprocess.check_output([str(tool('ffmpeg')), '-hide_banner', '-encoders'], text=True, timeout=60)
        assert 'libx264' in encoders, 'ffmpeg has no libx264'
    if stage == 'preview':
        tool('ffprobe')
    if stage == 'pictures':
        tool('ffmpeg')
        Q.success(PICTURE_ENV)
        version = subprocess.check_output([str(PICTURE_PYTHON), '-I', '-B', '-c', 'import PIL; from PIL import Image, ImageDraw; print(PIL.__version__)'],
                                          cwd=R, text=True, timeout=120).strip()
        assert version == PILLOW, 'Picture env differs: ' + version


def check_upstream(stage):
    check_qualified()
    check_baseline()
    if stage == 'derive':
        return
    check_derived()
    if stage == 'score':
        check_timeline()
    elif stage == 'preview':
        check_score()
    elif stage == 'pictures':
        Q.success(NAMES['preview'])
        check_score()


def wait_for(stage):
    """Each controller writes its exit file after its job's receipt is final."""
    exits = [Q.CONTROLLER_EXIT, BASE_EXIT, *(exit_file(name) for name in UPSTREAM[stage])]
    for _ in range(WAIT_POLLS):
        codes = [path.read_text().strip() if path.exists() else '' for path in exits]
        failed = [path.name for path, code in zip(exits, codes) if code not in ('', '0')]
        if failed:
            raise RuntimeError(f'Upstream controller failed; nothing launched: {failed}')
        if all(code == '0' for code in codes):
            return
        time.sleep(20)
    raise RuntimeError('Wait expired; nothing launched')


def launch(stage):
    wait_for(stage)
    check_pins()
    check_upstream(stage)
    check_tools(stage)
    assert not job(stage).exists() and not manifest(stage).exists(), f'Duplicate {stage} refused'
    files = stage_files(stage)
    with manifest(stage).open('x') as handle:
        json.dump({'files': [{'path': path, 'sha256': digest} for path, digest in sorted(files.items())]}, handle, indent=2)
    Q.guard_policy()
    subprocess.run(guard_command(stage), cwd=R, check=True)
    Q.success(NAMES[stage])


def preflight(stage):
    """Read-only. Run it once the build has ended; the stages' own upstream jobs may still be running."""
    check_pins()
    check_baseline()
    check_qualified()
    files = base_rows()
    Q.guard_policy()
    check_tools(stage)
    assert not job(stage).exists() and not manifest(stage).exists(), f'Duplicate {stage} refused'
    print(json.dumps({'stage': stage, 'job': NAMES[stage], 'root': str(R), 'frozen_files': len(files), 'binary_sha256': Q.BIN_SHA256,
                      'audio_rows_sha256': sha(AUDIO_ROWS), 'launched': False}))
    if stage == 'derive':
        assert not N2.exists(), 'Existing N2 retained'
        report, assembly, _, tally = derived_bytes()
        folder = Path(tempfile.mkdtemp(prefix='chain03-dry-')) / 'narration-n2'
        write_folder(folder, report, assembly, os.symlink)
        check_folder(folder, report, assembly)
        print(json.dumps({'derivation': tally, 'tools': prove(folder), 'dry_folder': str(folder)}))


def run(stage):
    if stage == 'derive':
        run_derive()
        return
    check_derived()
    {'timeline': run_timeline, 'score': run_score, 'preview': run_preview, 'pictures': run_pictures}[stage]()


def compare(chapter):
    if chapter == AFFECTED:
        boundary = compare_boundary(TIMELINE_OUT)
        print(json.dumps(boundary))
        return 1 if boundary['problems'] else 0
    rows = [compare_table(chapter, table, TIMELINE_OUT) for table in TABLES]
    for row in rows:
        print(json.dumps(row))
    return 0 if all(row['normalized_equal'] for row in rows) else 1


class Names:
    """Helpers that pin this chain read L.M.name, as they did with chain02-n2 over chain01. Here both are this file."""

    def __getattr__(self, name):
        try:
            return globals()[name]
        except KeyError:
            raise AttributeError(name) from None


M = Names()

if __name__ == '__main__':
    assert len(sys.argv) == 3, __doc__
    mode, name = sys.argv[1:]
    if mode == '--compare':
        assert name in CHAPTERS, __doc__
        sys.exit(compare(name))
    assert mode in ('--check', '--launch', '--run') and name in NAMES, __doc__
    {'--check': preflight, '--launch': launch, '--run': run}[mode](name)
