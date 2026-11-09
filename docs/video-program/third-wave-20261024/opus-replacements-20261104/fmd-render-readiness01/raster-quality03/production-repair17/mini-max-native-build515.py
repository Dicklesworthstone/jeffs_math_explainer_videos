"""Build 515 (CC3): build 268's native contract on mac-mini-max (Apple arm64), as the child of root's guard.

Successor of 507 (8e5db00a), which is unchanged. The one fix: 507 ran each step with subprocess.run(timeout=),
which kills cargo only. Its rustc or test grandchildren could outlive the helper, and the guard's stop then
saw a dead child and stopped nothing. 515 starts each step as the leader of its own process group and owns
that group. On a timeout, an exception, or a stop signal (the guard's SIGTERM to this helper's group, SIGINT
or SIGHUP), it sends SIGTERM to the whole step group, waits 6 s, then SIGKILL, waits 4 s, and reaps cargo.
That is about 10 s, inside the guard's 20 s before its own SIGKILL. It writes <step>.cleanup.json and
exits non-zero: 128 + the signal number after a stop. A step that exits but leaves processes in its group
is also stopped and fails. Every source, guard, toolchain, test, facts and chapter binding is 507's.

507's description, unchanged except for the output names:

It keeps review-repair-build268.py's flags, steps and pass rules:
  --release --locked --offline --config SOURCE/engine-budget.toml --manifest-path CRATE/Cargo.toml
  --bin complex_analysis_visual, run from the crate folder so that SOURCE/.cargo/config.toml applies;
  test: one result line, 233 passed, 0 failed, 1 ignored; plate_fade_probe --ignored: 1 passed;
  clippy --no-deps -D warnings; rustc -C metadata=jmev_review_repair268; facts sha256 7d528509...;
  chapters equal to the 15 names; CARGO_INCREMENTAL=0, CARGO_BUILD_JOBS=2, RCH_SHIM_LOCAL_IDE=1;
  no RUSTFLAGS, CARGO_ENCODED_RUSTFLAGS, RUSTC or rustc wrappers; PATH = toolchain bin, /usr/bin, /bin.

What changes for this host, and why:
  - No copy step and no fmt step. CC0's 506 tree is already the 268 scene source, placed at ROOT/source.
    Its listing must equal 770e5239... (4,197 files) before the first Cargo step and after the last.
  - The toolchain is nightly-2026-08-31-aarch64-apple-darwin, found by rustup like 268, but never through
    a shim. RCH can replace the toolchain's bin/cargo with a script and keep the real one as cargo-rch-real.
    The helper runs cargo-rch-real when it is present, otherwise bin/cargo. It refuses any of cargo, rustc,
    cargo-clippy or clippy-driver that is not a thin arm64 Mach-O binary. The toolchain is pinned by
    version and commit (the same nightly as 268), not by the Linux file hashes. The four hashes are recorded.
  - Clippy is run as toolchain cargo-clippy with CARGO set to that cargo. This is what `cargo clippy`
    does, but it cannot pick up a rustup proxy from CARGO_HOME/bin first.
  - macOS has no sched_setaffinity. RUST_TEST_THREADS=4 stands in for 268's four-CPU affinity.
    RAYON_NUM_THREADS=4 comes from the guard.
  - TMPDIR = ROOT/tmp and XDG_CACHE_HOME = ROOT/cache, as 268 used its work tmp and cache. CARGO_TARGET_DIR
    is the guard's ROOT/target. CARGO_HOME is inherited: CC0's 501 found its caches cover all of 268's lock.
    Every folder Cargo writes to, and the toolchain, must lie on APFS; ExFAT and FAT are refused.
  - Cargo merges config from CARGO_HOME and from every ancestor of the crate folder. The only configs
    allowed are SOURCE/.cargo/config.toml (bound by the listing) and the CARGO_HOME config. That one may not
    set flags, targets, profiles, patches, sources, env or unstable options.

Guard binding: it runs only inside a fresh job of root's guard, with this lane's ROOT: the job folder
ROOT/jobs/JOB holds just resource-receipt.json and command.log. The receipt is admitted, not finished,
for the native phase, with the policy 20/8/4/80/20/10. CARGO_TARGET_DIR is ROOT/target.

Usage (root, under the guard): python3 -I -B mini-max-native-build515.py --job JOB
Outputs, all new files in ROOT/jobs/JOB: preflight515.json, <step>.stdout/.stderr, any <step>.cleanup.json,
complex_analysis_visual-515 and qualification.json. Outside the job folder it writes nothing itself: it
creates ROOT/tmp and ROOT/cache if they are missing, and Cargo writes TARGET, TMPDIR and its own caches.
Prepared only: not run on any host. It authorizes no render. It accepts no picture, motion, audio or
release. The binary cannot equal 268's Linux binary 983954c5.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import platform
import re
import shutil
import signal
import subprocess
import sys
import time
import tomllib

ROOT = Path.home()/'scratch'/'jmev-complex-native-20261109'
SOURCE = ROOT/'source'
CRATE = SOURCE/'workspace/explainer_corpus/complex_analysis_visual'
WORK = ROOT/'jobs'
TARGET = ROOT/'target'
TMP = ROOT/'tmp'
CACHE = ROOT/'cache'
LISTING_506 = '770e5239a547a6b81b079400c7a11d47a401b2c76a8e90f95746da55d3211cea'
FILES_506 = 4197
POLICY = dict(admission_gib=20, stop_free_gib=8, phase_growth_gib=4, work_cap_gib=80, target_growth_cap_gib=20,
              sample_seconds=10)
TOOLCHAIN = 'nightly-2026-08-31'
TOOLCHAIN_DIR = 'nightly-2026-08-31-aarch64-apple-darwin'
CARGO_VERSION = 'cargo 1.100.0-nightly (e8cb624d5 2026-08-22)'
RUSTC_LINES = ('release: 1.100.0-nightly', 'commit-hash: 90850177249efe0321573c569aec5d12b257f8d6',
               'commit-date: 2026-08-30', 'host: aarch64-apple-darwin')
FACTS_SHA256 = '7d528509752e532c07edb3fda7c12cd8251ac4ba77022355ae42ad725add4417'
CHAPTERS = ['01_the_bump', '02_turning_numbers', '03_two_planes', '04_reading_color', '05_gallery',
            '06_one_derivative', '07_one_point', '08_loop_sums', '09_boundary', '10_rigidity', '11_counting',
            '12_residues', '13_real_integral', '14_continuation', '15_closing']
METADATA = 'metadata=jmev_review_repair268'
FORBIDDEN_ENV = ('RUSTFLAGS', 'CARGO_ENCODED_RUSTFLAGS', 'RUSTC', 'RUSTC_WRAPPER', 'RUSTC_WORKSPACE_WRAPPER',
                 'RUSTDOC', 'RUSTDOCFLAGS', 'CARGO', 'CARGO_BUILD_TARGET')
FORBIDDEN_ENV_PREFIXES = ('CARGO_BUILD_RUSTC', 'CARGO_BUILD_RUSTFLAGS', 'CARGO_PROFILE_', 'CARGO_UNSTABLE_',
                          'CARGO_TARGET_', 'CARGO_ENV_', 'CARGO_PATCH', 'CARGO_SOURCE')
FORBIDDEN_TABLES = ('target', 'profile', 'patch', 'source', 'env', 'unstable', 'replace')
FORBIDDEN_BUILD_KEYS = ('rustflags', 'rustdocflags', 'rustc', 'rustc-wrapper', 'rustc-workspace-wrapper',
                        'rustdoc', 'target', 'incremental')
WRITABLE_FS = ('apfs',)
ARM64_MACHO = b'\xcf\xfa\xed\xfe\x0c\x00\x00\x01'      # MH_MAGIC_64, CPU_TYPE_ARM64, little-endian
STEP_SECONDS = 7200
COLD_SECONDS = 14400                                   # the first Cargo step builds a cold target
TERM_GRACE = 6                                         # the guard waits 20 s after its SIGTERM, then SIGKILL
KILL_GRACE = 4
STOP_SIGNALS = (signal.SIGTERM, signal.SIGINT, signal.SIGHUP)


def sha(path):
    with Path(path).open('rb') as f:
        return hashlib.file_digest(f, 'sha256').hexdigest()


def listing(top):
    """CC0's 492/506 rule: sha256 of sorted "<sha256>  <path>\\n" over every file; no links, no specials."""
    rows = []
    for parent, dirs, files in os.walk(top, followlinks=False):
        for name in dirs + files:
            path = Path(parent, name)
            if path.is_symlink():
                raise SystemExit('515 refused: symlink in the source tree: %s' % path)
        for name in files:
            path = Path(parent, name)
            if not path.is_file():
                raise SystemExit('515 refused: not a regular file: %s' % path)
            rows.append((path.relative_to(top).as_posix(), sha(path)))
    rows.sort()
    text = ''.join('%s  %s\n' % (digest, rel) for rel, digest in rows)
    return len(rows), hashlib.sha256(text.encode()).hexdigest()


def mounts():
    out = subprocess.run(['/sbin/mount'], capture_output=True, text=True, timeout=30, check=True).stdout
    rows = []
    for line in out.splitlines():
        if ' on ' in line and ' (' in line:
            point, rest = line.split(' on ', 1)[1].rsplit(' (', 1)
            rows.append((point, rest.split(',')[0].strip(' )').lower()))
    return rows


def fstype(path, rows):
    real = os.path.realpath(path)
    under = [(len(p), kind) for p, kind in rows if real == p or real.startswith(p.rstrip('/') + '/')]
    return max(under)[1] if under else None


def arm64_binary(path):
    with Path(path).open('rb') as f:
        return f.read(8) == ARM64_MACHO


def text(argv, env):
    return subprocess.run(list(map(str, argv)), capture_output=True, text=True, timeout=120, env=env,
                          check=True).stdout.strip()


def read_receipt(job):
    for _ in range(10):                                # the guard rewrites it every 10 s while we run
        try:
            return json.loads((job/'resource-receipt.json').read_text())
        except json.JSONDecodeError:
            time.sleep(0.5)
    raise SystemExit('515 refused: the guard receipt could not be read')


def guard_binding(job):
    assert job.parent == WORK and job.is_dir() and not job.is_symlink(), job
    present = sorted(p.name for p in job.iterdir())
    assert present == ['command.log', 'resource-receipt.json'], ('the job folder is not fresh', present)
    receipt = read_receipt(job)
    assert receipt['admitted'] is True and 'exit' not in receipt and 'finished' not in receipt, 'receipt state'
    assert receipt['phase'] == 'native' and receipt['preflight_only'] is False, 'receipt phase'
    assert receipt['policy'] == POLICY, ('policy', receipt['policy'])
    assert os.environ.get('CARGO_TARGET_DIR') == str(TARGET), 'CARGO_TARGET_DIR is not the guard TARGET'
    assert os.environ.get('RUSTUP_TOOLCHAIN') == TOOLCHAIN, 'RUSTUP_TOOLCHAIN is not the guard value'
    return dict(job=job.name, command=receipt['command'], policy=receipt['policy'],
                admission=receipt['admission'], inputs_sha256=receipt.get('inputs_sha256'))


def env_problems():
    bad = [k for k in os.environ if k in FORBIDDEN_ENV or (k.startswith(FORBIDDEN_ENV_PREFIXES) and k != 'CARGO_TARGET_DIR')]
    return ['forbidden environment: %s' % ', '.join(sorted(bad))] if bad else []


def cargo_home():
    home = Path(os.environ.get('CARGO_HOME') or Path.home()/'.cargo')
    assert home.is_absolute(), home
    return home


def config_problems(home):
    """Only SOURCE/.cargo/config.toml and a harmless CARGO_HOME config may reach Cargo."""
    out, seen = [], {}
    for name in ('config', 'config.toml'):
        path = home/name
        if path.exists():
            data = tomllib.loads(path.read_text())
            seen[str(path)] = sha(path)
            out += ['%s sets [%s]' % (path, t) for t in FORBIDDEN_TABLES if t in data]
            out += ['%s sets build.%s' % (path, k) for k in FORBIDDEN_BUILD_KEYS if k in data.get('build', {})]
    for parent in CRATE.parents:
        if parent == SOURCE or SOURCE in parent.parents:
            continue                                   # inside the bound source listing
        for name in ('config', 'config.toml'):
            path = parent/'.cargo'/name
            if path.exists() and path.parent != home:
                out.append('an unbound Cargo config is on the crate path: %s' % path)
    return out, seen


def toolchain():
    rustup = Path.home()/'.cargo/bin/rustup'
    rustc = Path(subprocess.check_output([str(rustup), 'which', '--toolchain', TOOLCHAIN, 'rustc'], text=True,
                                         timeout=120).strip())
    bin_dir = rustc.parent
    assert bin_dir.parent.name == TOOLCHAIN_DIR, ('not the arm64 Apple toolchain', bin_dir)
    real = bin_dir/'cargo-rch-real'
    cargo = real if real.is_file() else bin_dir/'cargo'
    tools = dict(cargo=cargo, rustc=rustc, cargo_clippy=bin_dir/'cargo-clippy', clippy_driver=bin_dir/'clippy-driver')
    for name, path in tools.items():
        assert path.is_file() and arm64_binary(path), ('%s is not a thin arm64 Mach-O binary (a shim?)' % name, path)
    env = dict(PATH=str(bin_dir) + ':/usr/bin:/bin', HOME=str(Path.home()))
    version = text([cargo, '--version'], env)
    assert version == CARGO_VERSION, ('cargo version', version)
    verbose = text([rustc, '-vV'], env).splitlines()
    assert all(line in verbose for line in RUSTC_LINES), ('rustc identity', verbose)
    clippy = text([tools['clippy_driver'], '--version'], env)
    record = dict(bin_dir=str(bin_dir), cargo_is_rch_real=cargo == real, cargo_version=version, rustc_vv=verbose,
                  clippy_version=clippy, sha256={k: sha(p) for k, p in tools.items()},
                  rch_wrapper_sha256=sha(bin_dir/'cargo') if cargo == real else None)
    return tools, record


class Stopped(BaseException):
    """A stop signal reached this helper; raised inside the running step so that its group is stopped."""
    def __init__(self, signum):
        super().__init__(signum)
        self.signum = signum


def on_stop(signum, frame):
    raise Stopped(signum)


def group_alive(pgid):
    try:
        os.killpg(pgid, 0)
        return True
    except ProcessLookupError:
        return False


def stop_group(child, reason):
    """SIGTERM the step's whole group, then SIGKILL; reap cargo; report what is left."""
    for sig in STOP_SIGNALS:
        signal.signal(sig, signal.SIG_IGN)            # already stopping: a second signal must not cut this short
    sent = []
    for sig, grace in ((signal.SIGTERM, TERM_GRACE), (signal.SIGKILL, KILL_GRACE)):
        if child.poll() is not None and not group_alive(child.pid):
            break
        try:
            os.killpg(child.pid, sig)
            sent.append(sig.name)
        except ProcessLookupError:
            pass
        deadline = time.monotonic() + grace
        while time.monotonic() < deadline and (child.poll() is None or group_alive(child.pid)):
            time.sleep(0.1)
    return dict(reason=reason, pgid=child.pid, signals=sent, child_exit=child.poll(),
                group_left=group_alive(child.pid))


def run_step(job, name, argv, seconds, extra):
    """507's run(), with the step as the leader of its own process group, which is stopped before any exit."""
    with (job/(name + '.stdout')).open('xb') as o, (job/(name + '.stderr')).open('xb') as e:
        child = subprocess.Popen(list(map(str, argv)), cwd=CRATE, stdout=o, stderr=e, process_group=0,
                                 env=dict(os.environ, **(extra or {})))
        try:
            code = child.wait(timeout=seconds)
            if group_alive(child.pid):
                raise RuntimeError('%s exited but left processes in its group' % name)
        except BaseException as error:
            record = stop_group(child, repr(error))
            with (job/(name + '.cleanup.json')).open('x') as f:
                json.dump(record, f, indent=2)
            raise
    assert code == 0, (name, code)
    return (job/(name + '.stdout')).read_bytes()


def main():
    p = argparse.ArgumentParser(description=__doc__.split('\n', 1)[0])
    p.add_argument('--job', required=True)
    args = p.parse_args()
    for sig in STOP_SIGNALS:
        signal.signal(sig, on_stop)
    assert re.fullmatch(r'[a-z0-9_-]+', args.job), 'job must be a simple lowercase name'
    assert sys.version_info >= (3, 11) and platform.system() == 'Darwin' and platform.machine() == 'arm64'
    job = WORK/args.job
    guard = guard_binding(job)
    for k in ('RUSTFLAGS', 'CARGO_ENCODED_RUSTFLAGS', 'RUSTC', 'RUSTC_WRAPPER', 'RUSTC_WORKSPACE_WRAPPER'):
        assert k not in os.environ, k                  # 268's own check, kept word for word
    for folder in (TMP, CACHE):
        folder.mkdir(mode=0o700, exist_ok=True)
        assert folder.is_dir() and not folder.is_symlink(), folder
    assert TARGET.is_dir() and not TARGET.is_symlink() and CRATE.is_dir(), 'ROOT layout'
    home = cargo_home()
    tools, tool_record = toolchain()
    rows = mounts()
    places = dict(source=SOURCE, target=TARGET, tmp=TMP, cache=CACHE, job=job, cargo_home=home,
                  cargo_registry=home/'registry', cargo_git=home/'git', toolchain=Path(tool_record['bin_dir']).parent)
    kinds = {name: fstype(path, rows) for name, path in places.items()}
    problems = ['%s (%s) is missing' % (n, places[n]) for n, path in places.items() if not path.exists()]
    problems += ['%s (%s) is on %s, not APFS' % (n, os.path.realpath(places[n]), k) for n, k in kinds.items()
                 if k not in WRITABLE_FS and places[n].exists()]
    config_issues, configs = config_problems(home)
    problems += env_problems() + config_issues
    if problems:
        raise SystemExit('515 refused before Cargo: ' + '; '.join(problems))
    count, digest = listing(SOURCE)
    assert (count, digest) == (FILES_506, LISTING_506), ('source is not the 506 tree', count, digest)
    os.environ.update(PATH=str(tools['cargo'].parent) + ':/usr/bin:/bin', CARGO_INCREMENTAL='0', CARGO_BUILD_JOBS='2',
                      RCH_SHIM_LOCAL_IDE='1', CARGO_TARGET_DIR=str(TARGET), TMPDIR=str(TMP),
                      XDG_CACHE_HOME=str(CACHE), RUST_TEST_THREADS='4')
    keep = ('PATH', 'CARGO_HOME', 'CARGO_INCREMENTAL', 'CARGO_BUILD_JOBS', 'RCH_SHIM_LOCAL_IDE', 'CARGO_TARGET_DIR',
            'TMPDIR', 'XDG_CACHE_HOME', 'RUST_TEST_THREADS', 'RAYON_NUM_THREADS', 'RUSTUP_TOOLCHAIN')
    preflight = dict(schema='mini-max-native-build515', guard=guard, toolchain=tool_record, cargo_home=str(home),
                     cargo_configs=configs, file_systems=kinds, source_listing=digest, source_files=count,
                     python=platform.python_version(), macos=platform.mac_ver()[0], node=platform.node(),
                     env={k: os.environ.get(k) for k in keep})
    with (job/'preflight515.json').open('x') as f:
        json.dump(preflight, f, indent=2)

    def run(name, argv, seconds=STEP_SECONDS, extra=None):
        return run_step(job, name, argv, seconds, extra)

    cargo = tools['cargo']
    f = ['--release', '--locked', '--offline', '--config', str(SOURCE/'engine-budget.toml'),
         '--manifest-path', str(CRATE/'Cargo.toml'), '--bin', 'complex_analysis_visual']
    tests = run('tests', [cargo, 'test', *f], COLD_SECONDS).decode()
    counts = re.findall(r'^test result: ok\. (\d+) passed; 0 failed; 1 ignored; 0 measured; 0 filtered out;', tests, re.M)
    assert len(counts) == 1 and int(counts[0]) == 233, tests[-2000:]
    probe = run('plate-fade-probe', [cargo, 'test', *f, 'plate_fade_probe', '--', '--ignored', '--nocapture']).decode()
    assert re.search(r'^test result: ok\. 1 passed; 0 failed; 0 ignored; 0 measured; \d+ filtered out;', probe, re.M), probe[-3000:]
    run('clippy', [tools['cargo_clippy'], 'clippy', *f, '--no-deps', '--', '-D', 'warnings'], extra=dict(CARGO=str(cargo)))
    run('build', [cargo, 'rustc', *f, '--', '-C', METADATA])
    binary = job/'complex_analysis_visual-515'
    assert not binary.exists()
    shutil.copy2(TARGET/'release/complex_analysis_visual', binary)
    assert arm64_binary(binary)
    facts = run('facts', [binary, 'facts'])
    assert hashlib.sha256(facts).hexdigest() == FACTS_SHA256, 'facts differ from 268: compare facts.stdout'
    chapters = run('chapters', [binary, 'chapters']).decode().splitlines()
    assert chapters == CHAPTERS, chapters
    assert listing(SOURCE) == (FILES_506, LISTING_506), 'the source tree changed during the build'
    with (job/'qualification.json').open('x') as out:
        json.dump(dict(schema='mini-max-native-build515', binary=str(binary), binary_sha256=sha(binary),
                       release_binary_sha256=sha(TARGET/'release/complex_analysis_visual'),
                       tests_passed=int(counts[0]), explicit_native_plate_fade_probe_passed=True, clippy_passed=True,
                       facts_sha256=hashlib.sha256(facts).hexdigest(), chapters=chapters, source_listing=LISTING_506,
                       preflight_sha256=sha(job/'preflight515.json'), toolchain_sha256=tool_record['sha256'],
                       linux_268_binary_sha256='983954c508e42b90d443ad6ca09f2baba7565a4f609d09d2ea821640534d6ec7',
                       scope='Native mac-mini-max build of the 268 source and its automated checks only; the binary '
                             'differs from the Linux 268 binary; no timeline, frame, picture, motion, listening, '
                             'AV or release acceptance'), out, indent=2)
    print(json.dumps(dict(passed=True, job=job.name, binary_sha256=sha(binary), facts_sha256=FACTS_SHA256)))
    return 0


if __name__ == '__main__':
    try:
        code = main()
    except Stopped as stop:
        code = 128 + stop.signum
    raise SystemExit(code)
