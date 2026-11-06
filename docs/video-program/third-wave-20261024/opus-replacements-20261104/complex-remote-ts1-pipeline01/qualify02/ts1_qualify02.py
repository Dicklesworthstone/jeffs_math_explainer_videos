"""Build and qualify the stage07 complex binary on ts1, in one guarded job.

Root's build01.sh steps in order, on the stage07 crate, plus what that job never reached: the oracle under its isolated
env, both facts comparisons and the chapter registry. A clean run here needs no separate qualifier job, and the failed
build01 evidence is left alone and is not read. The rebuild overwrites O/target's binary, so it waits for chain02's
pictures job to end. complex_analysis_visual-release01 stays, and this job's binary is copied to its own
complex_analysis_visual-release02 before anything judges it.

    --check   read-only, any time: stage07 tree, preserved old binary, env, guard policy, unused names; launches nothing
    --launch  the same checks, then waits for chain02 to end, repeats them, freezes the manifest and runs ts1's guard
              (start-build02.sh does this)
    --run     the guard's child; never call it by hand

Paths are logical, as in qualify01: R comes from the frozen manifest, samefile proves it is this repository, and
everything else is built from R. The engine folder comes from the same manifest.
"""
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import sys
import time

EC = 'narration_lab/third-wave-20261024/root/elevenlabs-complex-rebuild-20261104'
PHYSICAL = Path(__file__).resolve().parents[6]
OLD_BIN_SHA256 = '3c90fb5464fc8736214228d62f1c322629ede230cd58cb11ddb947e5d9e9c168'  # root, Mail 5348
RELEASE_FACTS_SHA256 = '7d528509752e532c07edb3fda7c12cd8251ac4ba77022355ae42ad725add4417'
TREE_FILES = {'script.tsv': '946a9c07a462eb066fc8d20c6a092d5b7564d38752d26177bdbbcf15c6027cdc',
              'narration_units.json': 'c937fca1f0d7507117462600f0f478472bd2b8dab93075aba6494bd99ce81141',
              'docs/storyboard.md': 'ad1ccd44893116b9acf01999e2a919a28233c57c775368b927d2b6387f3880bc',
              'docs/narration-direction-map-frederick-stage07.json': '4db4c4d5a433f3212e6af739d71625dd0931ac86bc68c4cfc859212925faff77',
              'src/chapters/boundary.rs': '1ebe0331ba47144b0968dc553fc2237139d10f2d8f26a207cd0b12f3c5dfdca7'}  # sealed repair, Mail 5612
TESTS = 207
NUMPY, SCIPY = '2.5.4', '1.18.1'
TOOLCHAIN = 'nightly-2026-08-31'
BUILD_SECONDS = 7200
WAIT_POLLS = 30240  # 7 days at 20 s, as the chain's wait
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
START = ME.parent / 'start-build02.sh'
O = R / EC / 'remote-ts1-01'
JOBS = O / 'jobs'
BUILD = JOB = 'complex-remote-build02'
ENV_JOB = 'complex-remote-oracle-env01'
ENV_PYTHON = JOBS / ENV_JOB / 'env/bin/python'
GUARD = O / 'guard01.py'
PRESERVED_GUARD = R / 'docs/video-program/third-wave-20261024/opus-replacements-20261104/engine-secondary-validation01/guard_owner56g_target20g.py'
LOCK_TOOL = R / 'docs/video-program/third-wave-20261024/opus-replacements-20261104/engine-geometry-validation02/verify_patched_lock.py'
INPUTS = O / 'inputs01.json'
MANIFEST = O / 'build-inputs02.json'
BIN = O / 'complex_analysis_visual-release02'
OLD_BIN = O / 'complex_analysis_visual-release01'
TARGET = O / 'target'
OLD_CHAIN_END = O / 'chain-pictures-controller02.exit'
CONTROLLER_EXIT = O / 'build-controller02.exit'
TREE = R / EC / 'native-stage07-root01'
OLD_TREE = R / EC / 'native-stage06-root03'
C = TREE / 'workspace/explainer_corpus/complex_analysis_visual'
ORACLE = C / 'oracle.py'
RELEASE_FACTS = R / 'narration_lab/third-wave-20261024/root/five-film-release-20261027/complex-stage06-optimized-build01/release-facts.tsv'


def built():
    path = JOBS / BUILD / 'binary.sha256'
    return path.read_text().strip() if path.is_file() else None


BIN_SHA256 = built()


def success(job):
    d = read(JOBS / job / 'resource-receipt.json')
    assert d.get('finished') and d.get('admitted') is True and d.get('exit') == 0, job
    assert d.get('inputs_sha256') == d.get('final_inputs_sha256'), job
    assert not d.get('stopped_for_resource_limit') and 'error' not in d, job
    assert d.get('policy') == POLICY, 'Guard policy changed: ' + job
    return True


def engine():
    """The folder build01.sh calls W, spelled by the frozen manifest."""
    found = [Path(row['path']) for row in read(INPUTS)['files'] if row['path'].endswith('/Cargo.lock.pinned')]
    assert len(found) == 1, 'Manifest does not name one pinned lock'
    return found[0].parent


def check_log(text):
    """Every step ran, in order, and none reported an error."""
    lines = [line for line in text.splitlines() if line.strip()]
    assert 'Traceback (most recent call last):' not in text, 'The job raised'
    assert [line for line in lines if line.startswith('error') and line != FIXTURE_NOISE] == [], 'Build log has an error'
    assert 'PASS: the 19 packages are path copies, every other record is identical; patch.unused entries: 0' in text, 'Lock check missing'
    results = re.findall(r'^test result: (.+?); finished in ', text, re.M)
    assert results == [f'ok. {TESTS} passed; 0 failed; 0 ignored; 0 measured; 0 filtered out'], 'Test result differs'
    outcomes = re.findall(r'^test \S+ \.\.\. (\w+)$', text, re.M)
    assert len(outcomes) == TESTS and set(outcomes) == {'ok'}, 'Test lines differ'
    done = [n for n, line in enumerate(lines) if line.strip().startswith('Finished `release` profile')]
    assert len(done) == 3, 'Expected test, build and clippy to finish'
    index = {key: next(n for n, line in enumerate(lines) if line.startswith(key) or line.strip().startswith(key))
             for key in ('test result:', 'Checking complex_analysis_visual')}
    assert done[0] < index['test result:'] < done[1] < index['Checking complex_analysis_visual'] < done[2], 'Build steps out of order'


def check_outputs():
    """Everything the job left, without the controller's exit file, which only exists once it returns."""
    job = JOBS / BUILD
    success(BUILD)
    d = read(job / 'resource-receipt.json')
    assert d.get('inputs_sha256') == sha(MANIFEST), 'Build inputs moved'
    assert d.get('command', [])[1:] == child_command()[1:], 'Build ran another command'  # pictures reads this under its own python
    check_log((job / 'command.log').read_text(errors='replace'))
    native = (job / 'facts-native.tsv').read_bytes()
    assert native and native == (job / 'facts-oracle.tsv').read_bytes() == RELEASE_FACTS.read_bytes(), 'Facts differ'
    assert (job / 'chapters.txt').read_text().splitlines() == list(CHAPTERS), 'Chapter registry differs'
    assert built() and sha(BIN) == built(), 'Binary differs from its recorded hash'
    assert built() != OLD_BIN_SHA256, 'The stage07 script compiles into the binary, so this cannot be the old one'
    assert sha(OLD_BIN) == OLD_BIN_SHA256, 'The stage06 binary copy changed'
    s = read(job / 'build-summary.json')
    assert s['binary_sha256'] == built() and s['chapters'] == list(CHAPTERS) and s['tests_passed'] == TESTS, 'Build summary differs'


def check_build():
    check_outputs()
    assert CONTROLLER_EXIT.read_text().strip() == '0', 'Controller exit differs'


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


def check_tree():
    assert sha(RELEASE_FACTS) == RELEASE_FACTS_SHA256, 'macOS facts changed'
    assert sha(OLD_BIN) == OLD_BIN_SHA256, 'The stage06 binary copy changed'
    for name, digest in TREE_FILES.items():
        assert sha(C / name) == digest, 'Stage07 tree differs: ' + name
    rows = {str(R / row['path']): row['sha256'] for row in read(TREE / 'tree-receipt.json')['files']}
    found = {str(p) for p in (TREE / 'workspace').rglob('*') if p.is_file()}
    assert found == set(rows), f'Tree file set differs: {sorted(found ^ set(rows))[:3]}'
    return rows


def frozen_files():
    """inputs01's rows with the stage07 tree in place of the stage06 tree."""
    old = str(OLD_TREE) + '/'
    files = {row['path']: row['sha256'] for row in read(INPUTS)['files'] if not row['path'].startswith(old)}
    files.update(check_tree())
    for path, digest in files.items():
        assert sha(path) == digest, path
    for path in (INPUTS, TREE / 'tree-receipt.json', LOCK_TOOL, RELEASE_FACTS, OLD_BIN, ME, START, *job_files(ENV_JOB)):
        files.setdefault(str(path), sha(path))
    return files


def child_command():
    return [sys.executable, '-I', '-B', str(ME), '--run']


def guard_command():
    return [sys.executable, '-I', '-B', str(GUARD), '--phase', 'native', '--job', BUILD, '--inputs', str(MANIFEST), '--', *child_command()]


def cargo(*argv):
    return [str(shutil.which('rustup')), 'run', TOOLCHAIN, 'cargo', *argv]


def build_flags():
    return ['--release', '--locked', '--offline', '--config', str(engine() / 'engine-budget.toml'), '--manifest-path', str(C / 'Cargo.toml'),
            '--bin', 'complex_analysis_visual']


def step(argv):
    subprocess.run(argv, cwd=R, check=True, timeout=BUILD_SECONDS)


def capture(name, argv):
    """Keep both streams before judging, so a mismatch leaves its evidence."""
    result = subprocess.run(argv, cwd=C, capture_output=True, timeout=900)
    for stream, data in (('stdout', result.stdout), ('stderr', result.stderr)):
        with (JOBS / JOB / f'{name}.{stream}').open('xb') as handle:
            handle.write(data)
    assert result.returncode == 0, (name, result.returncode)
    return result.stdout


def no_cargo_config():
    assert not (Path.home() / '.cargo/config.toml').exists() and not (Path.home() / '.cargo/config').exists(), 'A home cargo config exists'


def live_receipt():
    """The guard rewrites the live receipt just after Popen, so a read can catch it half written. Only that parse failure
    is retried, briefly."""
    for _ in range(100):
        try:
            return read(JOBS / JOB / 'resource-receipt.json')
        except json.JSONDecodeError:
            time.sleep(0.05)
    return read(JOBS / JOB / 'resource-receipt.json')


def run():
    d = live_receipt()
    assert d.get('admitted') is True and 'exit' not in d
    assert d['command'] == child_command()
    assert d['policy'] == POLICY, 'Guard policy changed'
    for row in read(MANIFEST)['files']:
        assert sha(row['path']) == row['sha256'], row['path']
    assert os.path.samefile(os.environ['CARGO_TARGET_DIR'], TARGET), 'Guard target differs'
    assert not BIN.exists(), 'Existing release02 retained'
    no_cargo_config()
    os.environ.update(CARGO_INCREMENTAL='0', RCH_SHIM_LOCAL_IDE='1')
    step([sys.executable, '-I', '-B', str(LOCK_TOOL), str(engine() / 'Cargo.lock.pinned'), str(C / 'Cargo.lock')])
    step(cargo('fmt', '--manifest-path', str(C / 'Cargo.toml'), '--', '--check'))
    step(cargo('test', *build_flags()))
    step(cargo('build', *build_flags()))
    target_bin = Path(os.environ['CARGO_TARGET_DIR']) / 'release/complex_analysis_visual'
    shutil.copy2(target_bin, BIN)
    step(cargo('clippy', *build_flags(), '--no-deps', '--', '-D', 'warnings'))
    digest = sha(BIN)
    assert sha(target_bin) == digest, 'Target binary moved during clippy'
    assert digest != OLD_BIN_SHA256, 'The stage07 script compiles into the binary, so this cannot be the old one'
    check_log((JOBS / JOB / 'command.log').read_text(errors='replace'))
    native = capture('facts-native', [str(BIN), 'facts'])
    oracle = capture('facts-oracle', [str(ENV_PYTHON), '-I', '-B', str(ORACLE)])
    for name, data in (('facts-native.tsv', native), ('facts-oracle.tsv', oracle)):
        with (JOBS / JOB / name).open('xb') as handle:
            handle.write(data)
    assert native and native == oracle, 'Native facts differ from the oracle; both retained'
    assert native == RELEASE_FACTS.read_bytes(), 'ts1 facts differ from macOS; both retained'
    keys = [line.split(b'\t', 1)[0] for line in native.splitlines()]
    assert len(keys) == len(set(keys))
    listed = capture('chapters', [str(BIN), 'chapters'])
    with (JOBS / JOB / 'chapters.txt').open('xb') as handle:
        handle.write(listed)
    assert listed.decode().splitlines() == list(CHAPTERS), 'Chapter registry differs'
    no_cargo_config()
    with (JOBS / JOB / 'binary.sha256').open('x') as handle:
        handle.write(digest + '\n')
    with (JOBS / JOB / 'build-summary.json').open('x') as handle:
        json.dump({'tests_passed': TESTS, 'binary_sha256': digest, 'old_binary_sha256': OLD_BIN_SHA256, 'fact_rows': len(keys),
                   'facts_sha256': hashlib.sha256(native).hexdigest(), 'chapters': listed.decode().splitlines(),
                   'oracle_python': str(ENV_PYTHON), 'numpy': NUMPY, 'scipy': SCIPY, 'script_sha256': TREE_FILES['script.tsv'],
                   'scope': 'Stage07 source built, tested, linted and fact-checked on ts1. No timeline, picture, listening or baseline equivalence.'}, handle, indent=2)
    print(f'BUILD PASS: {TESTS} tests, {len(keys)} fact rows equal oracle and macOS; 15 chapters; binary {digest}.', flush=True)


def wait_for_old_chain():
    """Mail 5610: nothing rebuilds until chain02's last job has ended, however it ended."""
    for _ in range(WAIT_POLLS):
        if OLD_CHAIN_END.exists() and OLD_CHAIN_END.read_text().strip():
            return
        time.sleep(20)
    raise RuntimeError('Wait expired; nothing launched')


def check():
    files = frozen_files()
    check_env()
    guard_policy()
    assert TARGET.is_dir(), 'The preserved target folder is missing'
    assert not (JOBS / JOB).exists() and not MANIFEST.exists() and not BIN.exists(), 'Duplicate build refused'
    assert not CONTROLLER_EXIT.exists(), 'Duplicate build refused'
    return files


def launch():
    check()  # a bad transfer stops here, not days later
    wait_for_old_chain()
    files = check()
    files[str(OLD_CHAIN_END)] = sha(OLD_CHAIN_END)
    with MANIFEST.open('x') as handle:
        json.dump({'files': [{'path': path, 'sha256': digest} for path, digest in sorted(files.items())]}, handle, indent=2)
    subprocess.run(guard_command(), cwd=R, check=True)
    check_outputs()


def preflight():
    files = check()
    print(json.dumps({'root': str(R), 'frozen_files': len(files), 'engine': str(engine()), 'chain02_ended': OLD_CHAIN_END.exists(), 'launched': False}))


if __name__ == '__main__':
    assert sys.argv[1:] in (['--check'], ['--launch'], ['--run']), __doc__
    {'--check': preflight, '--launch': launch, '--run': run}[sys.argv[1]]()
