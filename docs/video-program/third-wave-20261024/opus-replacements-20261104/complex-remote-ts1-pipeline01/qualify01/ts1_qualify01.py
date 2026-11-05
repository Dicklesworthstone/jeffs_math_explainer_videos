"""Qualify the retained ts1 complex binary after its build stopped on one missing module.

complex-remote-build01 passed its lock check, fmt, tests, release build and clippy, then stopped
at its oracle line because the job's python3 has no scipy. This helper compiles nothing and never
relabels that job. It proves the failure was exactly that, binds the original evidence, then runs
the steps build01.sh never reached: the oracle with the isolated env, both facts comparisons and
the chapter registry. Downstream stages need this success and the original receipt together.

    --check   read-only: original evidence, every frozen file, env, guard policy; launches nothing
    --launch  writes the new manifest and runs ts1's guard (start-qualify01.sh does this)
    --run     the guard's child; never call it by hand

Paths are logical. On ts1 the repository path is a symlink, so Path.resolve() names a different
string for the same file. R comes from the frozen manifest, samefile proves it is this repository,
and everything else is built from R, so strings stay equal to the manifest's and to the binary's.
"""
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import re
import subprocess
import sys

EC = 'narration_lab/third-wave-20261024/root/elevenlabs-complex-rebuild-20261104'
PHYSICAL = Path(__file__).resolve().parents[6]
BIN_SHA256 = '3c90fb5464fc8736214228d62f1c322629ede230cd58cb11ddb947e5d9e9c168'  # root, Mail 5348
BUILD_INPUTS_SHA256 = '29c03e8e9c5a74e21ff46f457cbb6ebb54f7111df8f99f8a6c87f6c2c740d338'
RELEASE_FACTS_SHA256 = '7d528509752e532c07edb3fda7c12cd8251ac4ba77022355ae42ad725add4417'
TESTS = 207
NUMPY, SCIPY = '2.5.4', '1.18.1'
FIXTURE_NOISE = 'error: unclosed table, expected `]`'  # a dependency's malformed-file test, in healthy logs too
CHAPTERS = ('01_the_bump', '02_turning_numbers', '03_two_planes', '04_reading_color',
            '05_gallery', '06_one_derivative', '07_one_point', '08_loop_sums', '09_boundary',
            '10_rigidity', '11_counting', '12_residues', '13_real_integral', '14_continuation', '15_closing')
POLICY = {'admission_gib': 10, 'stop_free_gib': 8, 'phase_growth_gib': 4, 'work_cap_gib': 56,
          'target_growth_cap_gib': 20, 'sample_seconds': 10}


def read(path):
    return json.loads(path.read_text())


def sha(path):
    with open(path, 'rb') as handle:
        return hashlib.file_digest(handle, 'sha256').hexdigest()


def logical_root():
    suffix = f'/{EC}/remote-ts1-01/guard01.py'
    found = [row['path'] for row in read(PHYSICAL / EC / 'remote-ts1-01/inputs01.json')['files'] if row['path'].endswith(suffix)]
    assert len(found) == 1, 'Manifest does not name this lane once'
    root = Path(found[0].removesuffix(suffix))
    assert os.path.samefile(root, PHYSICAL), 'Manifest root is not this repository'
    return root


R = logical_root()
ME = R / Path(__file__).resolve().relative_to(PHYSICAL)
START = ME.parent / 'start-qualify01.sh'
O = R / EC / 'remote-ts1-01'
JOBS = O / 'jobs'
JOB = 'complex-remote-qualify01'
BUILD = 'complex-remote-build01'
ENV_JOB = 'complex-remote-oracle-env01'
ENV_PYTHON = JOBS / ENV_JOB / 'env/bin/python'
GUARD = O / 'guard01.py'
PRESERVED_GUARD = R / 'docs/video-program/third-wave-20261024/opus-replacements-20261104/engine-secondary-validation01/guard_owner56g_target20g.py'
INPUTS = O / 'inputs01.json'
MANIFEST = O / 'qualify-inputs01.json'
BIN = O / 'complex_analysis_visual-release01'
TARGET_BIN = O / 'target/release/complex_analysis_visual'
C = R / EC / 'native-stage06-root03/workspace/explainer_corpus/complex_analysis_visual'
ORACLE = C / 'oracle.py'
RELEASE_FACTS = R / 'narration_lab/third-wave-20261024/root/five-film-release-20261027/complex-stage06-optimized-build01/release-facts.tsv'


def success(job):
    d = read(JOBS / job / 'resource-receipt.json')
    assert d.get('finished') and d.get('admitted') is True and d.get('exit') == 0, job
    assert d.get('inputs_sha256') == d.get('final_inputs_sha256'), job
    assert not d.get('stopped_for_resource_limit') and 'error' not in d, job
    assert d.get('policy') == POLICY, 'Guard policy changed: ' + job
    return True


def check_log(text):
    """build01.sh runs under set -e, so reaching the oracle line means every earlier step exited 0.
    The log must show those steps in order and end in exactly the one expected failure."""
    lines = [line for line in text.splitlines() if line.strip()]
    assert lines[-1] == "ModuleNotFoundError: No module named 'scipy'", 'Build failed some other way'
    assert text.count('Traceback (most recent call last):') == 1, 'More than one traceback'
    tail = lines[-6:]
    assert any(re.search(r'oracle\.py", line 39, in <module>$', line) for line in tail), 'Failure is not at the oracle import'
    assert any(line.strip() == 'import scipy.special as sp' for line in tail), 'Failure is not at the oracle import'
    assert [line for line in lines if line.startswith('error') and line != FIXTURE_NOISE] == [], 'Build log has another error'
    assert 'PASS: the 19 packages are path copies, every other record is identical; patch.unused entries: 0' in text, 'Lock check missing'
    results = re.findall(r'^test result: (.+?); finished in ', text, re.M)
    assert results == [f'ok. {TESTS} passed; 0 failed; 0 ignored; 0 measured; 0 filtered out'], 'Test result differs'
    outcomes = re.findall(r'^test \S+ \.\.\. (\w+)$', text, re.M)
    assert len(outcomes) == TESTS and set(outcomes) == {'ok'}, 'Test lines differ'
    done = [n for n, line in enumerate(lines) if line.strip().startswith('Finished `release` profile')]
    assert len(done) == 3, 'Expected test, build and clippy to finish'
    index = {key: next(n for n, line in enumerate(lines) if line.startswith(key) or line.strip().startswith(key))
             for key in ('test result:', 'Checking complex_analysis_visual', 'Traceback')}
    assert done[0] < index['test result:'] < done[1] < index['Checking complex_analysis_visual'] < done[2] < index['Traceback'], 'Build steps out of order'


def check_build():
    job = JOBS / BUILD
    d = read(job / 'resource-receipt.json')
    assert d.get('finished') and d.get('admitted') is True, 'Build is not terminal'
    assert d.get('exit') == 1, 'Qualifier applies only to the original exit 1'
    assert d.get('inputs_sha256') == d.get('final_inputs_sha256') == BUILD_INPUTS_SHA256 == sha(INPUTS), 'Build inputs moved'
    assert not d.get('stopped_for_resource_limit') and 'error' not in d, 'Build stopped by the guard'
    assert d.get('policy') == POLICY, 'Guard policy changed'
    assert d.get('command') == ['/bin/bash', str(O / 'build01.sh')], 'Build ran another command'
    assert (O / 'build-controller01.exit').read_text().strip() == '1', 'Controller exit differs'
    text = (job / 'command.log').read_text(errors='replace')
    check_log(text)
    assert (job / 'facts-native.tsv').stat().st_size > 0, 'Native facts missing'
    assert (job / 'facts-oracle.tsv').stat().st_size == 0, 'Oracle wrote output'
    assert not (job / 'chapters.txt').exists(), 'Build went past its oracle line'
    assert sha(BIN) == BIN_SHA256 == sha(TARGET_BIN), 'Binary differs from the pinned target'


def env_versions():
    return subprocess.run([str(ENV_PYTHON), '-I', '-B', '-c', 'import numpy, scipy, scipy.special; print(numpy.__version__, scipy.__version__)'],
                          cwd=R, capture_output=True, text=True, timeout=120, check=True).stdout.split()


def check_env():
    success(ENV_JOB)
    assert env_versions() == [NUMPY, SCIPY], 'Oracle env versions differ'


def guard_policy():
    spec = importlib.util.spec_from_file_location('preserved_guard', PRESERVED_GUARD)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    guard = module.load()
    assert (guard.ADMISSION_GIB['native'], guard.STOP_FREE_GIB, guard.CAPS['native'], guard.WORK_CAP_GIB,
            guard.TARGET_GROWTH_CAP_GIB) == (10, 8, 4, 56, 20), 'Guard caps changed'


def job_files(job):
    """Every file the job kept, minus uv's download cache, which nothing reads after install."""
    return [p for p in (JOBS / job).rglob('*') if p.is_file() and 'uv-cache' not in p.parts]


def frozen_files():
    files = {row['path']: row['sha256'] for row in read(INPUTS)['files']}
    for path, digest in files.items():
        assert sha(path) == digest, path
    paths = [INPUTS, O / 'build-controller01.exit', O / 'build-controller01.log', BIN, TARGET_BIN, ME, START]
    paths += job_files(BUILD) + job_files(ENV_JOB)
    for path in paths:
        files.setdefault(str(path), sha(path))
    return files


def check():
    assert sha(RELEASE_FACTS) == RELEASE_FACTS_SHA256, 'macOS facts changed'
    check_build()
    check_env()
    guard_policy()
    return frozen_files()


def child_command():
    return [sys.executable, '-I', '-B', str(ME), '--run']


def guard_command():
    return [sys.executable, '-I', '-B', str(GUARD), '--phase', 'native', '--job', JOB, '--inputs', str(MANIFEST), '--', *child_command()]


def capture(name, argv):
    """Keep both streams before judging, so a mismatch leaves its evidence."""
    result = subprocess.run(argv, cwd=C, capture_output=True, timeout=900)
    for stream, data in (('stdout', result.stdout), ('stderr', result.stderr)):
        with (JOBS / JOB / f'{name}.{stream}').open('xb') as handle:
            handle.write(data)
    assert result.returncode == 0, (name, result.returncode)
    return result.stdout


def run():
    d = read(JOBS / JOB / 'resource-receipt.json')
    assert d.get('admitted') is True and 'exit' not in d
    assert d['command'] == child_command()
    assert d['policy'] == POLICY, 'Guard policy changed'
    for row in read(MANIFEST)['files']:
        assert sha(row['path']) == row['sha256'], row['path']
    check_build()
    check_env()
    native = capture('facts-native', [str(BIN), 'facts'])
    oracle = capture('facts-oracle', [str(ENV_PYTHON), '-I', '-B', str(ORACLE)])
    assert native and native == oracle, 'Native facts differ from the oracle; both retained'
    assert native == (JOBS / BUILD / 'facts-native.tsv').read_bytes(), 'Native facts differ from the original build'
    assert native == RELEASE_FACTS.read_bytes(), 'ts1 facts differ from macOS; both retained'
    keys = [line.split(b'\t', 1)[0] for line in native.splitlines()]
    assert len(keys) == len(set(keys))
    listed = capture('chapters', [str(BIN), 'chapters']).decode().splitlines()
    assert listed == list(CHAPTERS), 'Chapter registry differs'
    with (JOBS / JOB / 'qualification-summary.json').open('x') as handle:
        json.dump({'original_build_job': BUILD, 'original_build_exit': 1, 'original_build_relabelled': False,
                   'original_receipt_sha256': sha(JOBS / BUILD / 'resource-receipt.json'),
                   'original_log_sha256': sha(JOBS / BUILD / 'command.log'),
                   'original_inputs_sha256': BUILD_INPUTS_SHA256, 'tests_passed': TESTS, 'binary_sha256': BIN_SHA256,
                   'fact_rows': len(keys), 'facts_sha256': hashlib.sha256(native).hexdigest(), 'chapters': listed,
                   'oracle_python': str(ENV_PYTHON), 'numpy': NUMPY, 'scipy': SCIPY,
                   'scope': 'Binary qualified for downstream use alongside the original exit 1 receipt. Not a rebuild, and not timeline, picture, listening or baseline equivalence.'}, handle, indent=2)
    print(f'QUALIFY PASS: {len(keys)} fact rows equal oracle, original build and macOS; 15 chapters; original exit 1 preserved.', flush=True)


def launch():
    files = check()
    assert not (JOBS / JOB).exists() and not MANIFEST.exists(), 'Duplicate qualification refused'
    with MANIFEST.open('x') as handle:
        json.dump({'files': [{'path': path, 'sha256': digest} for path, digest in sorted(files.items())]}, handle, indent=2)
    subprocess.run(guard_command(), cwd=R, check=True)
    success(JOB)


def preflight():
    files = check()
    assert not (JOBS / JOB).exists() and not MANIFEST.exists(), 'Duplicate qualification refused'
    print(json.dumps({'root': str(R), 'frozen_files': len(files), 'binary_sha256': BIN_SHA256, 'launched': False}))


if __name__ == '__main__':
    assert sys.argv[1:] in (['--check'], ['--launch'], ['--run']), __doc__
    {'--check': preflight, '--launch': launch, '--run': run}[sys.argv[1]]()
