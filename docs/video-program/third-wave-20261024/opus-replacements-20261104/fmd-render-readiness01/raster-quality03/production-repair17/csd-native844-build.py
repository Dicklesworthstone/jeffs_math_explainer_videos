"""Linux qualification of exact native760 source, preserving full narration and geometry.

Run only as the direct child of guard_csd02. No render or acceptance is implied.
The immutable Mac source manifest is compared to existing Linux dependencies;
only absolute dependency paths are normalized. All new output is exclusive.
"""
import argparse
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import platform
import re
import shutil
import signal
import subprocess
import tomllib

D = Path('/data/projects/jmev-stroke-probe-20261106')
Q = D/'qualification01/production-repair17'
BASE = Q/'csd-native844-base761.py'
BASE_SHA = '32bf6d9e08c1f21f2445debfd46ba090e0374007e4da5516310cb122774099e5'
OLD = D/'work/review-repair313/workspace'
SOURCE = D/'work/native-boundary844'
CRATE = SOURCE/'workspace/explainer_corpus/complex_analysis_visual'
MAC_SOURCE = '/Users/jemanuel/scratch/jmev-complex-native-20261109/source'
POLICY = dict(admission_gib=10, stop_free_gib=8, phase_growth_gib=4,
              work_cap_gib=56, target_growth_cap_gib=20, sample_seconds=10)

def sha(p):
    with Path(p).open('rb') as f:
        return hashlib.file_digest(f, 'sha256').hexdigest()

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--job', required=True)
    args = parser.parse_args()
    assert args.job == 'csd-native-build844'
    assert platform.system() == 'Linux' and platform.machine() == 'x86_64'
    assert sha(BASE) == BASE_SHA
    spec = importlib.util.spec_from_file_location('owned_native761', BASE)
    base = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(base)
    base.own_group()
    job = D/'jobs'/args.job
    assert sorted(p.name for p in job.iterdir()) == ['command.log', 'resource-receipt.json']
    receipt = json.loads((job/'resource-receipt.json').read_text())
    assert receipt['admitted'] is True and 'finished' not in receipt and 'exit' not in receipt
    assert receipt['policy'] == POLICY and receipt['phase'] == 'native'
    assert os.environ['CARGO_TARGET_DIR'] == str(D/'work'), 'guard accounting root changed'
    base.STATE['job'] = job
    for sig in base.STOP_SIGNALS:
        signal.signal(sig, base.on_stop)
    base.CRATE = CRATE
    base.SOURCE = SOURCE
    manifest = json.loads((Q/'csd-native844-source760.json').read_text())
    assert manifest['listing_sha256'] == base.LISTING_506
    rows = manifest['files']
    boundary_rel = 'workspace/explainer_corpus/complex_analysis_visual/src/chapters/boundary.rs'
    cargo_rel = 'workspace/explainer_corpus/complex_analysis_visual/Cargo.toml'
    plan_rel = 'workspace/explainer_corpus/complex_analysis_visual/narration_units.json'
    def dependency_check():
        checked = {}
        for rel, digest in rows.items():
            if rel.startswith(('engine/', 'repo-view/')):
                path = D/'work/tree'/rel
                assert path.is_file() and not path.is_symlink() and sha(path) == digest, str(path)
                checked[rel] = digest
        assert checked
        return checked
    dependencies = dependency_check()
    assert not SOURCE.exists(), 'preserve existing attempt; use a new successor'
    old_files = {str(p.relative_to(OLD)): p for p in OLD.rglob('*') if p.is_file()}
    expected = {k.removeprefix('workspace/'): v for k,v in rows.items() if k.startswith('workspace/')}
    assert set(expected) <= set(old_files)
    assert set(old_files)-set(expected) == {
        'explainer_corpus/complex_analysis_visual/docs/narration-direction-map-frederick-review313.json',
        'explainer_corpus/complex_analysis_visual/narration_units.before-review313.json'}
    for rel in expected:
        path = old_files[rel]
        assert not path.is_symlink(), str(path)
        if 'workspace/'+rel not in (boundary_rel, cargo_rel, plan_rel):
            assert sha(path) == expected[rel], rel
    boundary = Q/'csd-native844-boundary.rs'
    assert sha(boundary) == rows[boundary_rel]
    plan = Q/'csd-native844-narration_units.json'
    assert sha(plan) == rows[plan_rel]
    cargo_mac = (Q/'csd-native844-Cargo.toml').read_text()
    assert hashlib.sha256(cargo_mac.encode()).hexdigest() == rows[cargo_rel]
    cargo_linux = cargo_mac.replace(MAC_SOURCE, str(D/'work/tree'))
    assert cargo_linux == old_files[cargo_rel.removeprefix('workspace/')].read_text()
    for rel in sorted(expected):
        src = boundary if 'workspace/'+rel == boundary_rel else plan if 'workspace/'+rel == plan_rel else old_files[rel]
        dest = SOURCE/'workspace'/rel
        dest.parent.mkdir(parents=True, exist_ok=True)
        with dest.open('xb') as f:
            f.write(src.read_bytes())
    for rel in ('.cargo/config.toml', 'engine-budget.toml'):
        src = Q/('csd-native844-'+Path(rel).name)
        raw = src.read_text()
        assert hashlib.sha256(raw.encode()).hexdigest() == rows[rel]
        dest = SOURCE/rel
        dest.parent.mkdir(parents=True, exist_ok=True)
        with dest.open('x') as f:
            f.write(raw.replace(MAC_SOURCE, str(D/'work/tree')))
    source_listing = base.listing(SOURCE)
    assert not base.env_problems(), base.env_problems()
    home = base.cargo_home()
    assert not (home/'config').exists(), 'unexpected alternate Cargo configuration'
    home_config = home/'config.toml'
    # Only dev/test debug settings (inactive under --release), plus a target
    # location explicitly overridden below. Do not allow flags/source patches.
    assert tomllib.loads(home_config.read_text()) == {
        'profile': {'dev': {'debug':'line-tables-only','incremental':False},
                    'test': {'debug':'line-tables-only'}},
        'build': {'target-dir':'/data/ssd/build/cargo-target'}}
    work_config = D/'work/.cargo/config.toml'
    assert sha(work_config) == rows['.cargo/config.toml']
    allowed = {home_config, work_config, SOURCE/'.cargo/config.toml'}
    configs = {str(p):sha(p) for p in allowed}
    for parent in (CRATE, *CRATE.parents):
        for name in ('config','config.toml'):
            path = parent/'.cargo'/name
            assert not path.exists() or path in allowed, ('unexpected Cargo configuration',str(path))
    tool = Path(subprocess.check_output([str(Path.home()/'.cargo/bin/rustup'), 'which',
        '--toolchain', 'nightly-2026-08-31', 'rustc'], text=True, timeout=60).strip()).parent
    cargo = tool/'cargo-rch-real' if (tool/'cargo-rch-real').is_file() else tool/'cargo'
    tools = [cargo, tool/'rustc', tool/'cargo-clippy', tool/'clippy-driver', tool/'rustfmt']
    for p in tools:
        with p.open('rb') as f:
            assert f.read(4) == b'\x7fELF', ('not native Linux tool', str(p))
    verbose = subprocess.check_output([str(tool/'rustc'), '-vV'], text=True, timeout=60)
    assert subprocess.check_output([str(cargo),'--version'],text=True,timeout=60).strip() == base.CARGO_VERSION
    for value in ('release: 1.100.0-nightly', 'commit-hash: 90850177249efe0321573c569aec5d12b257f8d6', 'host: x86_64-unknown-linux-gnu'):
        assert value in verbose
    os.sched_setaffinity(0, sorted(os.sched_getaffinity(0))[:4])
    os.environ.update(PATH=str(tool)+':/usr/bin:/bin', CARGO_INCREMENTAL='0', CARGO_BUILD_JOBS='2',
        RCH_SHIM_LOCAL_IDE='1', CARGO_TARGET_DIR=str(D/'work/target'), TMPDIR=str(D/'work/tmp'),
        XDG_CACHE_HOME=str(D/'work/cache'), RUST_TEST_THREADS='4')
    with (job/'preflight844.json').open('x') as f:
        json.dump(dict(source_listing=source_listing, tool_hashes={str(p):sha(p) for p in tools},
                       rustc=verbose, policy=POLICY, dependencies=len(dependencies),cargo_configs=configs), f, indent=2)
    def run(name, argv, seconds=7200, extra=None):
        return base.run_step(job, name, argv, seconds, extra)
    run('boundary-fmt-check', [tool/'rustfmt', '--check', '--edition', '2024', CRATE/'src/chapters/boundary.rs',
                              '--config-path', D/'work/tree/repo-view/rustfmt.toml'])
    flags = ['--release','--locked','--offline','--config',str(SOURCE/'engine-budget.toml'),
             '--manifest-path',str(CRATE/'Cargo.toml'),'--bin','complex_analysis_visual']
    tests = run('tests', [cargo,'test',*flags], 14400).decode()
    assert re.findall(r'^test result: ok\. (\d+) passed; 0 failed; 1 ignored; 0 measured; 0 filtered out;', tests, re.M) == ['235']
    probe = run('plate-fade-probe', [cargo,'test',*flags,'plate_fade_probe','--','--ignored','--nocapture']).decode()
    assert re.search(r'^test result: ok\. 1 passed; 0 failed; 0 ignored;', probe, re.M)
    run('clippy',[tool/'cargo-clippy','clippy',*flags,'--no-deps','--','-D','warnings'],extra=dict(CARGO=str(cargo)))
    run('build',[cargo,'rustc',*flags,'--','-C','metadata=jmev_review_repair752'])
    binary = job/'complex_analysis_visual-844'
    assert not binary.exists()
    shutil.copy2(D/'work/target/release/complex_analysis_visual',binary)
    facts = run('facts',[binary,'facts'])
    assert hashlib.sha256(facts).hexdigest() == base.FACTS_SHA256
    assert run('chapters',[binary,'chapters']).decode().splitlines() == base.CHAPTERS
    assert base.listing(SOURCE) == source_listing and dependency_check() == dependencies
    assert all(sha(path) == digest for path,digest in configs.items())
    with (job/'qualification.json').open('x') as f:
        json.dump(dict(binary=str(binary),binary_sha256=sha(binary),tests_passed=235,
            explicit_native_plate_fade_probe_passed=True,clippy_passed=True,
            facts_sha256=base.FACTS_SHA256,chapters=base.CHAPTERS,source_listing=source_listing,
            scope='Linux source760 qualification only. No timeline, pictures, listening, fine-sync or publication acceptance.'),f,indent=2)

if __name__ == '__main__':
    main()
