"""Guarded native qualification of retained four repairs plus root chapter11 candidate.
Independent source/picture review remains open. No narration or render acceptance."""
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
MANIFEST = Q/'csd-native844-source760.json'
MANIFEST_SHA = '5afe200ee13dec271e924fb60386554ebc28af4cbacfeb0dea99e4418bf741c0'
PLAN = Q/'csd-native844-narration_units.json'
CARGO_MAC = Q/'csd-native844-Cargo.toml'
BUNDLE = Q/'csd-counting02-bundle03-manifest.json'
BUNDLE_SHA = 'a5210650e0f8dc18860dfca9b2d06857a67588d6715fb04d990b7b8a39b1ee21'
BUNDLE_NAME = 'counting-repair-bundle03'
COMBINED_LISTING = 'bab62da4ed84f554754caa1a56c8fe7a20568abfbdd82136b0b2948850642771'
OLD = D/'work/review-repair313/workspace'
SOURCE = D/'work/native-counting02'
CRATE = SOURCE/'workspace/explainer_corpus/complex_analysis_visual'
JOB_NAME = 'csd-native-counting-build02'
BINARY_NAME = 'complex_analysis_visual-counting02'
METADATA = 'metadata=jmev_counting_repair02'
MAC_SOURCE = '/Users/jemanuel/scratch/jmev-complex-native-20261109/source'
POLICY = dict(admission_gib=10, stop_free_gib=8, phase_growth_gib=4,
              work_cap_gib=56, target_growth_cap_gib=20, sample_seconds=10)
CAV = 'workspace/explainer_corpus/complex_analysis_visual/'
# Workspace row: (file staged in Q, 760 base row, bundle02 replacement).
SUBST = {
    CAV+'src/chapters/counting.rs': (Q/'csd-counting02-counting.rs',
        'ecff48f7ca1aea98b3ac8eac65ffe8f4a74d5366fd30ce25546def3b627339d1',
        '21c5043a3339cfadb10024a2e8227a62a0b06674139eace1a0d9029a4e1aa0db'),
    CAV+'src/chapters/the_bump.rs': (Q/'csd-counting02-the_bump.rs',
        '8e93012e5b7598b9a99ffcdb07e436341f052d1175455f5581e108ee86e6ddb3',
        '90b51c2b30877911bbb5f5e14ee810268e14efb24e7c4e2a55f7f7e128c06ee7'),
    CAV+'src/chapters/loop_sums.rs': (Q/'csd-counting02-loop_sums.rs',
        '8a13e8a190cd502e7b58efbcb7c40872cdb0e6e702feb69e1b789e451a7a42b5',
        'a8d62f560321ea6ebea3d3859acd9a3ce3e5dde4375ec0ab42c1a1ba114d61ef'),
    CAV+'src/chapters/mod.rs': (Q/'csd-counting02-mod.rs',
        'e1f4d1b87f4b36f979e7800f7b2048117b9a4809b2372f385350698f5c3e7cbd',
        '4de1e4a56fde5c11ac3d6a65e772d09ec33db4f5757d817aff370799d3423578'),
    CAV+'src/chapters/boundary.rs': (Q/'csd-counting02-boundary.rs',
        'cd1900aadd46090083729d9aa768e59e1551c42ab94e5bd055f520cb4eff5e6e',
        '4b689791748132bd14c3929d191bb78ae7752bf9c3332beb22f774931873e932'),
}
TESTS_PASSED = 250            # build844's 235 plus 1 + 4 + 7 new tests (static count; this run decides)
NEW_TESTS = {
    'counting': ['the_first_need_names_the_input_loop_it_is_true_of', 'the_moebius_images_stay_circles_with_their_true_winding', 'the_cards_of_the_needs_fit_the_output_window'],
    'the_bump': ['the_preview_arrow_reaches_the_loop_head_first'],
    'loop_sums': ['the_kept_square_is_a_bisection_quarter_that_holds_c',
                  'the_name_of_c_is_joined_to_its_dot_by_a_leader_from_the_rim',
                  'the_repaired_square_picture_leaves_before_the_triangle',
                  'the_walk_rests_off_the_diagonal_and_stays_counterclockwise'],
    'boundary': ['the_closing_bars_keep_clear_of_dz', 'the_growing_disks_never_draw_through_a',
                 'the_growing_disks_pass_under_the_cards_and_round_a', 'the_leader_runs_from_the_hole_up_to_a',
                 'the_shrinking_circle_never_crosses_a_or_rho', 'the_shrinking_names_start_where_the_shrink_puts_them',
                 'the_shrunk_circle_keeps_apart_from_the_hole_ring'],
}
PREFLIGHT844 = D/'jobs/csd-native-build844/preflight844.json'
PREFLIGHT844_SHA = '37a28c9c492f27a5c491a01e877589fc1202a628546ecc0e08cc40492cdf7a02'
TOOL_SHA = {
    'cargo-rch-real': '19008157a0089896bc835c5ceb0b5bfbe26e70cdd5579c32bd09538fb68c0b76',
    'rustc': '49f9fa06f3e2b93aba85273433fef67b984ddb4cacb4500de56e85f70ec2e339',
    'cargo-clippy': '92708a2623b9186bd8d40e6f3ddac686b6fd541bd052584fb4343dfcfe92dcd8',
    'clippy-driver': '2cd2fc266f83e81a1fc9bd46528c5be8db6ec7d940f67872123feef4a993a427',
    'rustfmt': '016e3712bb170692190f20ce3356646d6d2c59343b81516de66af7c5fd2e5e7c',
}
TEMPLATE = dict(helper='csd-native844-build.py',
                helper_sha256='7932fe7803f311274ae850bc1c79a0d36df39db37aa579deda95e1e93ab11c9a',
                guard_inputs='csd-native844-inputs.json',
                guard_inputs_sha256='3b54974babc3d17763d89040719c014a4999fb0542af3fa43d83636538e9979f')


def sha(p):
    with Path(p).open('rb') as f:
        return hashlib.file_digest(f, 'sha256').hexdigest()


def listing_of(rows):
    """CC0's listing rule: sha256 of the lines "<sha256>  <path>\\n" over every row, sorted by path."""
    return hashlib.sha256(''.join('%s  %s\n' % (rows[k], k) for k in sorted(rows)).encode()).hexdigest()


def bundle_check(rows, bundle, subst=SUBST):
    """The bundle is the 760 tree plus exactly the four SUBST rows; returns the combined rows."""
    assert bundle['name'] == BUNDLE_NAME
    assert bundle['base_tree']['listing_sha256'] == listing_of(rows) and bundle['base_tree']['files'] == len(rows)
    assert sorted(bundle['combined_tree']['changed_rows']) == sorted(subst)
    by_row = {r['workspace_row']: r for r in bundle['rows']}
    assert sorted(by_row) == sorted(subst) and len(bundle['rows']) == len(subst)
    combined = dict(rows)
    for rel, (_, base_row, new) in subst.items():
        r = by_row[rel]
        assert rows[rel] == base_row == r['base_sha256'], rel
        assert r['replacement_sha256'] == new and r['diff']['forward_from_base_gives_replacement'] is True, rel
        combined[rel] = new
    assert listing_of(combined) == bundle['combined_tree']['listing_sha256'] == COMBINED_LISTING
    assert bundle['combined_tree']['files'] == len(combined)
    return combined


def new_tests_ok(tests_text):
    """Every new test is reported 'ok' exactly once."""
    lines = tests_text.splitlines()
    missing = []
    for module, names in NEW_TESTS.items():
        for name in names:
            if lines.count(f'test chapters::{module}::tests::{name} ... ok') != 1:
                missing.append(f'{module}::{name}')
    return missing


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--job', required=True)
    args = parser.parse_args()
    assert args.job == JOB_NAME
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
    assert sha(MANIFEST) == MANIFEST_SHA
    manifest = json.loads(MANIFEST.read_text())
    assert manifest['listing_sha256'] == base.LISTING_506
    rows = manifest['files']
    assert sha(BUNDLE) == BUNDLE_SHA
    combined = bundle_check(rows, json.loads(BUNDLE.read_text()))
    for rel, (staged, _, new) in SUBST.items():
        assert staged.parent == Q and staged.is_file() and not staged.is_symlink() and sha(staged) == new, rel
    boundary_rel = CAV+'src/chapters/boundary.rs'
    cargo_rel = CAV+'Cargo.toml'
    plan_rel = CAV+'narration_units.json'
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
            assert sha(path) == expected[rel], rel   # also proves OLD holds the three other 760 base rows
    assert sha(PLAN) == rows[plan_rel]
    cargo_mac = CARGO_MAC.read_text()
    assert hashlib.sha256(cargo_mac.encode()).hexdigest() == rows[cargo_rel]
    cargo_linux = cargo_mac.replace(MAC_SOURCE, str(D/'work/tree'))
    assert cargo_linux == old_files[cargo_rel.removeprefix('workspace/')].read_text()
    for rel in sorted(expected):
        key = 'workspace/'+rel
        src = SUBST[key][0] if key in SUBST else PLAN if key == plan_rel else old_files[rel]
        dest = SOURCE/'workspace'/rel
        dest.parent.mkdir(parents=True, exist_ok=True)
        with dest.open('xb') as f:
            f.write(src.read_bytes())
        if key != cargo_rel:
            assert sha(dest) == combined[key], rel
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
    cargo = tool/'cargo-rch-real'
    assert cargo.is_file(), 'build844 ran cargo-rch-real; refuse the RCH wrapper script'
    tools = [cargo, tool/'rustc', tool/'cargo-clippy', tool/'clippy-driver', tool/'rustfmt']
    for p in tools:
        with p.open('rb') as f:
            assert f.read(4) == b'\x7fELF', ('not native Linux tool', str(p))
        assert sha(p) == TOOL_SHA[p.name], ('toolchain changed since build844', str(p))
    assert sha(PREFLIGHT844) == PREFLIGHT844_SHA
    assert json.loads(PREFLIGHT844.read_text())['tool_hashes'] == {str(p): TOOL_SHA[p.name] for p in tools}
    verbose = subprocess.check_output([str(tool/'rustc'), '-vV'], text=True, timeout=60)
    assert subprocess.check_output([str(cargo),'--version'],text=True,timeout=60).strip() == base.CARGO_VERSION
    for value in ('release: 1.100.0-nightly', 'commit-hash: 90850177249efe0321573c569aec5d12b257f8d6', 'host: x86_64-unknown-linux-gnu'):
        assert value in verbose
    os.sched_setaffinity(0, sorted(os.sched_getaffinity(0))[:4])
    os.environ.update(PATH=str(tool)+':/usr/bin:/bin', CARGO_INCREMENTAL='0', CARGO_BUILD_JOBS='2',
        RCH_SHIM_LOCAL_IDE='1', CARGO_TARGET_DIR=str(D/'work/target'), TMPDIR=str(D/'work/tmp'),
        XDG_CACHE_HOME=str(D/'work/cache'), RUST_TEST_THREADS='4')
    with (job/'preflight-counting02.json').open('x') as f:
        json.dump(dict(source_listing=source_listing, tool_hashes={str(p):sha(p) for p in tools},
                       rustc=verbose, policy=POLICY, dependencies=len(dependencies),cargo_configs=configs,
                       bundle=dict(path=str(BUNDLE), sha256=BUNDLE_SHA, combined_listing=COMBINED_LISTING),
                       substitutions={k: v[2] for k, v in SUBST.items()}, template=TEMPLATE), f, indent=2)
    def run(name, argv, seconds=7200, extra=None):
        return base.run_step(job, name, argv, seconds, extra)
    for rel in sorted(SUBST):
        name = Path(rel).name
        run('fmt-check-'+name.removesuffix('.rs'), [tool/'rustfmt', '--check', '--edition', '2024',
            CRATE/'src/chapters'/name, '--config-path', D/'work/tree/repo-view/rustfmt.toml'])
    flags = ['--release','--locked','--offline','--config',str(SOURCE/'engine-budget.toml'),
             '--manifest-path',str(CRATE/'Cargo.toml'),'--bin','complex_analysis_visual']
    tests = run('tests', [cargo,'test',*flags], 14400).decode()
    assert re.findall(r'^test result: ok\. (\d+) passed; 0 failed; 1 ignored; 0 measured; 0 filtered out;', tests, re.M) == [str(TESTS_PASSED)]
    missing = new_tests_ok(tests)
    assert not missing, ('new tests not reported ok', missing)
    probe = run('plate-fade-probe', [cargo,'test',*flags,'plate_fade_probe','--','--ignored','--nocapture']).decode()
    assert re.search(r'^test result: ok\. 1 passed; 0 failed; 0 ignored;', probe, re.M)
    run('clippy',[tool/'cargo-clippy','clippy',*flags,'--no-deps','--','-D','warnings'],extra=dict(CARGO=str(cargo)))
    run('build',[cargo,'rustc',*flags,'--','-C',METADATA])
    binary = job/BINARY_NAME
    assert not binary.exists()
    shutil.copy2(D/'work/target/release/complex_analysis_visual',binary)
    facts = run('facts',[binary,'facts'])
    assert hashlib.sha256(facts).hexdigest() == base.FACTS_SHA256
    assert run('chapters',[binary,'chapters']).decode().splitlines() == base.CHAPTERS
    assert base.listing(SOURCE) == source_listing and dependency_check() == dependencies
    assert all(sha(path) == digest for path,digest in configs.items())
    with (job/'qualification.json').open('x') as f:
        json.dump(dict(binary=str(binary),binary_sha256=sha(binary),tests_passed=TESTS_PASSED,
            new_tests_passed=sorted(f'{m}::{n}' for m, ns in NEW_TESTS.items() for n in ns),
            explicit_native_plate_fade_probe_passed=True,clippy_passed=True,
            facts_sha256=base.FACTS_SHA256,chapters=base.CHAPTERS,source_listing=source_listing,
            bundle=dict(name=BUNDLE_NAME, sha256=BUNDLE_SHA, combined_listing=COMBINED_LISTING),
            substitutions={k: v[2] for k, v in SUBST.items()}, template=TEMPLATE,
            scope='Linux combined-source qualification only (760 tree plus four bundle rows). No render, '
                  'timeline, narration rebind, pictures, listening, fine-sync or publication acceptance.'),f,indent=2)

if __name__ == '__main__':
    main()
