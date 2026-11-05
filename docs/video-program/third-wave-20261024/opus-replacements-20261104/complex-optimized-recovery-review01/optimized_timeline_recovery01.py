"""Recover the complex stage06 timeline with root's already-built release binary.

This is timeline-after-audio04.py's run with three changes: the binary, the job and output
names, and the render timeout. Every check it made stays. It adds a line-exact comparison of the
12 tables the failed run finished (chapters 01-04). The failed job is hashed into the new manifest
and never written. Chapters 05-15 have no prior table, so this cannot show full baseline
equivalence, and the summary says so.

    --check   read-only: pins, failed run, every frozen file, guard policy; launches nothing
    --launch  writes the new manifest and runs the unchanged guard (start01.sh does this)
    --run     the guard's child; never call it by hand

The module also has the names downstream helpers read from timeline-after-audio04.py, so a new
copy of one can import this file instead.
"""
import importlib.util
import json
from pathlib import Path
import re
import subprocess
import sys

R = Path(__file__).resolve().parents[5]
E = R / 'narration_lab/third-wave-20261024/root/elevenlabs-complex-rebuild-20261104'
NATIVE = E / 'native-stage06-root03'
OPT = E / 'optimized-native-stage06-01'
SPEC = importlib.util.spec_from_file_location('complex_timeline', NATIVE / 'timeline-after-audio04.py')
T = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(T)
A, B, C, N, ORACLE, CHAPTERS, read, sha, success = T.A, T.B, T.C, T.N, T.ORACLE, T.CHAPTERS, T.read, T.sha, T.success

JOB = 'complex-stage06-optimized-timeline01'
BIN = OPT / 'complex_analysis_visual-release01'
STAGE = E / 'optimized-timeline-recovery01'
MANIFEST = STAGE / 'inputs01.json'
START = Path(__file__).resolve().parent / 'start01.sh'
BUILD_JOB = 'complex-stage06-optimized-build01'
FAILED = B / T.JOB
PRIOR_OUT = FAILED / 'timeline-output'
PRIOR_CHAPTERS = CHAPTERS[:4]  # the failed run finished these and died inside 05_gallery
TABLES = ('timeline', 'cues', 'cuelog')
RELEASE_FACTS = B / BUILD_JOB / 'release-facts.tsv'
GUARD = R / 'docs/video-program/third-wave-20261024/opus-replacements-20261104/engine-secondary-validation01/guard_owner56g_target20g.py'
GUARD_SOURCE = R / 'narration_lab/production-20261020/second-wave-central/duration-and-storage-20261020/root/run_bounded_phase_oliver_owner48g_target14g_20261102.py'
# The 14400 s that expired was this Python call's, not the guard's. This is a bound, not an estimate:
# it is the single-render limit root already uses in final-render01.py.
RENDER_TIMEOUT_SECONDS = 43200
POLICY = {'admission_gib': 10, 'stop_free_gib': 8, 'phase_growth_gib': 4, 'work_cap_gib': 56,
          'target_growth_cap_gib': 20, 'sample_seconds': 10}
BIN_SHA256 = 'b37c8f667748c900c7f890b5e471f41c59a769fbdcdf35a1bcecf09608e3431c'
FAILED_BIN_SHA256 = 'f2e4f79ebc798319b4bf1eba1627fd2645bd58cf2dad43b7716f82e2a955e970'
PINS = {
    BIN: BIN_SHA256,
    T.BIN: FAILED_BIN_SHA256,
    T.MANIFEST: '7d6aeaf7d1f8b83a55fb7818f48ff01c2c6fa62ba135d84ea63c8a087307072c',
    OPT / 'inputs01.json': 'e04bff50bfbde4b46a9f4f869176e333b3728e19e2580ee58b0d22165a1055a4',
    NATIVE / 'timeline-after-audio04.py': 'e3ab6203a565b020d988faa6008f926abc5c08aa2840c915b759f32e831051ae',
    C / 'script.tsv': '0bae7acb15f2f0453c8a25ba7110fedd65cf9156eba932143e08db7fdc8bfeee',
    FAILED / 'facts-native.stdout': '7d528509752e532c07edb3fda7c12cd8251ac4ba77022355ae42ad725add4417',
    RELEASE_FACTS: '7d528509752e532c07edb3fda7c12cd8251ac4ba77022355ae42ad725add4417',
    B / BUILD_JOB / 'build-summary.json': '566cbbc4ff1387debf5b837763460339ba33759e3361b45912598a79bb683176',
    GUARD: '1c19a9b8f817c3bca06e18fd0785ac21c3cb935be715d67ecc41e4e39d1f5097',
    GUARD_SOURCE: '819746fb33110d6c133e184812c80a00562d848816f623078ad86f0fb1a316da',
}


def normalized(path):
    """preview-after-score01.py's own table normalization, unchanged."""
    return [line for line in path.read_text().splitlines() if line and not line.startswith('#')]


def table_value(rows, key):
    found = [row[1] for row in rows if row[0] == key]
    assert len(found) == 1, key
    return found[0]


def check_failed_run():
    """The failed job is what its receipts say: a timeout, with exactly four finished chapters."""
    d = read(FAILED / 'resource-receipt.json')
    assert d.get('admitted') is True and d.get('exit') == 1 and d.get('finished'), 'Failed receipt changed'
    assert d.get('inputs_sha256') == d.get('final_inputs_sha256') == PINS[T.MANIFEST]
    assert not d.get('stopped_for_resource_limit') and 'error' not in d
    assert d.get('policy') == POLICY, 'Failed run used another policy'
    log = (FAILED / 'command.log').read_text()
    assert 'timed out after 14400 seconds' in log
    assert not (FAILED / 'timeline-summary.json').exists(), 'No full baseline may exist'
    assert sorted(p.name for p in PRIOR_OUT.glob('*.tsv')) == sorted(
        f'{chapter}.{table}.tsv' for chapter in PRIOR_CHAPTERS for table in TABLES)
    for chapter in PRIOR_CHAPTERS:
        rows = [line.split('\t') for line in (PRIOR_OUT / f'{chapter}.timeline.tsv').read_text().splitlines()]
        assert re.search(rf'^{chapter}\s+{table_value(rows, "frames")} frames\s', log, re.M), chapter


def frozen_files():
    """Both frozen manifests, then everything they could not list; a file added to the audio or
    workspace folders is refused as firmly as a changed one."""
    files = {}
    for manifest in (T.MANIFEST, OPT / 'inputs01.json'):
        for row in read(manifest)['files']:
            assert files.setdefault(row['path'], row['sha256']) == row['sha256'], row['path']
    for root in (N, NATIVE / 'workspace'):
        listed = {path for path in files if path.startswith(str(root) + '/')}
        assert listed == {str(p) for p in root.rglob('*') if p.is_file()}, f'File set changed: {root}'
    paths = [T.MANIFEST, OPT / 'inputs01.json', BIN, Path(__file__).resolve(), START]
    paths += [p for job in (B / BUILD_JOB, FAILED) for p in job.rglob('*') if p.is_file()]
    for path in paths:
        files.setdefault(str(path), sha(path))
    for path, digest in files.items():
        assert sha(Path(path)) == digest, path
    return files


def guard_policy():
    """The guard as it will run: same files, same lane lock, the owner's caps. Reads only."""
    spec = importlib.util.spec_from_file_location('guard_owner56g_target20g', GUARD)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    guard = module.load()
    assert (guard.WORK_CAP_GIB, guard.TARGET_GROWTH_CAP_GIB, guard.ADMISSION_GIB['native'],
            guard.STOP_FREE_GIB, guard.CAPS['native']) == (56, 20, 10, 8, 4)
    return guard.sample()


def check():
    assert T.R == R and T.JOB == 'complex-stage06-timeline01'
    for path, digest in PINS.items():
        assert sha(path) == digest, path
    assert all(success(job) for job in (*T.PREREQUISITES, BUILD_JOB))
    build = read(B / BUILD_JOB / 'build-summary.json')
    assert build['candidate_sha256'] == BIN_SHA256 and build['baseline_sha256'] == FAILED_BIN_SHA256
    assert build['profile'] == 'release' and build['native_facts_equal_baseline_and_oracle'] is True
    assert build['chapters'] == list(CHAPTERS)
    check_failed_run()
    return frozen_files()


def child_command():
    return [sys.executable, '-I', '-B', str(Path(__file__).resolve()), '--run']


def guard_command():
    return [sys.executable, '-I', '-B', str(GUARD), '--phase', 'native', '--job', JOB,
            '--inputs', str(MANIFEST), '--', *child_command()]


def script_ids():
    return [line.split('\t', 1)[0] for line in (C / 'script.tsv').read_text().splitlines()
            if line.strip() and not line.startswith('#')]


def check_facts():
    """The failed run's facts check, plus byte equality with both earlier release-side records."""
    facts = {}
    for name, argv in [('native', [str(BIN), 'facts']),
                       ('oracle', [sys.executable, '-I', '-B', str(ORACLE)])]:
        result = subprocess.run(argv, cwd=C, capture_output=True, timeout=900)
        for stream, data in [('stdout', result.stdout), ('stderr', result.stderr)]:
            with (B / JOB / ('facts-' + name + '.' + stream)).open('xb') as handle:
                handle.write(data)
        assert result.returncode == 0, (name, result.returncode)
        facts[name] = result.stdout
    assert facts['native'] and facts['native'] == facts['oracle'], 'Native facts differ from oracle; both outputs retained'
    assert facts['native'] == (FAILED / 'facts-native.stdout').read_bytes() == RELEASE_FACTS.read_bytes()
    keys = [line.split(b'\t', 1)[0] for line in facts['native'].splitlines()]
    assert len(keys) == len(set(keys))
    return len(keys)


def verify_narration():
    subprocess.run([sys.executable, '-I', '-B', str(NATIVE / 'workspace/tools/coherent_narration.py'),
                    'verify', str(N), '--script', str(C / 'script.tsv'), '--units',
                    str(N / 'external-units.json'), '--require-directions'], cwd=R, check=True, timeout=900)


def compare_prior(out):
    """Write the comparison before judging it, so a mismatch leaves its evidence."""
    rows = []
    for chapter in PRIOR_CHAPTERS:
        for table in TABLES:
            name = f'{chapter}.{table}.tsv'
            old, new = normalized(PRIOR_OUT / name), normalized(out / name)
            rows.append({'table': name, 'lines': len(old), 'normalized_equal': old == new,
                         'prior_sha256': sha(PRIOR_OUT / name), 'new_sha256': sha(out / name)})
    with (B / JOB / 'prior-table-comparison.json').open('x') as handle:
        json.dump({'prior_job': T.JOB, 'prior_binary_sha256': FAILED_BIN_SHA256, 'new_binary_sha256': BIN_SHA256,
                   'tables': rows}, handle, indent=2)
    different = [row['table'] for row in rows if not row['normalized_equal']]
    assert not different, f'Release tables differ from the failed run: {different}'
    return rows


def check_tables(out, script):
    """timeline-after-audio04.py's table checks, unchanged."""
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


def run():
    receipt = read(B / JOB / 'resource-receipt.json')
    assert receipt.get('admitted') is True and 'exit' not in receipt
    assert receipt['command'] == child_command()
    assert receipt['policy'] == POLICY, 'Guard policy changed'
    for row in read(MANIFEST)['files']:
        assert sha(Path(row['path'])) == row['sha256'], row['path']
    listed = subprocess.check_output([str(BIN), 'chapters'], text=True).splitlines()
    assert listed == list(CHAPTERS), listed
    script = script_ids()
    assert len(script) == len(set(script)) == 105
    fact_rows = check_facts()
    verify_narration()
    out = B / JOB / 'timeline-output'
    assert not out.exists(), 'Existing timeline output retained'
    subprocess.run([str(BIN), 'render', 'all', '--timeline-only', '--fps', '60', '--narration',
                    str(N), '--out', str(out)], cwd=R, check=True, timeout=RENDER_TIMEOUT_SECONDS)
    compared = compare_prior(out)
    chapters, seen = check_tables(out, script)
    with (B / JOB / 'timeline-summary.json').open('x') as handle:
        json.dump({'chapters': chapters, 'cues': len(seen), 'fps': 60,
                   'native_oracle_fact_rows_matched': fact_rows,
                   'total_frames': sum(row['frames'] for row in chapters),
                   'binary_sha256': BIN_SHA256, 'profile': 'release', 'failed_job': T.JOB,
                   'prior_chapters_compared': list(PRIOR_CHAPTERS), 'prior_tables_compared': len(compared),
                   'chapters_without_prior_table': list(CHAPTERS[len(PRIOR_CHAPTERS):]),
                   'full_baseline_equivalence_established': False,
                   'scope': 'Release native timeline only. Tables for chapters 01-04 equal the failed debug run line for line; chapters 05-15 have no prior table. No speedup, frame, PCM, listening or AV equivalence established.'}, handle, indent=2)
    print('TIMELINE PASS: 15 chapters, 105 cues, 12 prior tables equal; no full baseline equivalence.', flush=True)


def launch():
    files = check()
    guard_policy()
    assert not (B / JOB).exists(), 'Duplicate recovery refused'
    assert STAGE.is_dir() and not MANIFEST.exists(), 'Existing manifest retained'
    with MANIFEST.open('x') as handle:
        json.dump({'files': [{'path': path, 'sha256': digest} for path, digest in sorted(files.items())]}, handle, indent=2)
    subprocess.run(guard_command(), cwd=R, check=True)
    assert success(JOB)
    assert read(B / JOB / 'resource-receipt.json')['policy'] == POLICY


def preflight():
    files = check()
    now = guard_policy()
    assert not (B / JOB).exists() and not STAGE.exists(), 'New names already used'
    print(json.dumps({'frozen_files': len(files), 'prior_tables': len(PRIOR_CHAPTERS) * len(TABLES),
                      'resources': now, 'launched': False}))


if __name__ == '__main__':
    assert sys.argv[1:] in (['--check'], ['--launch'], ['--run']), __doc__
    {'--check': preflight, '--launch': launch, '--run': run}[sys.argv[1]]()
