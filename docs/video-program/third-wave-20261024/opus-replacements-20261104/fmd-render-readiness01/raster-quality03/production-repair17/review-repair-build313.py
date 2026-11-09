"""Guarded build313: build268's exact compiled tree with one substitution, chapters/boundary.rs
from boundary-guide-repair310.rs. Prepared by cc0 for root; cc0 has not run it.

Adapted from review-repair-build268.py: same admission check, toolchain pins, environment,
test, probe, Clippy, rustc and facts gates. What changes is listed in cc0-repair313-notes.md:
  - the base is build268's own source-copy.json (70 files), rebuilt file by file from the live
    268 workspace. rebind269 changed that workspace after the build (narration_units.json was
    rewritten, its backup and the review269 map were added), so narration_units.json is taken
    from its byte-exact backup and the two additions are left out;
  - only boundary.rs is substituted, and rustfmt runs as --check, so the compiled boundary.rs
    is exactly bda7d931 (no mutating format pass).
It approves nothing: build, tests and facts only. Pictures, motion, 4K and audio stay open (no audio heard or checked).
"""
from pathlib import Path
import hashlib
import json
import os
import re
import shutil
import subprocess

R = Path('/data/projects/jmev-stroke-probe-20261106')
W = R / 'work'
Q = R / 'qualification01/production-repair17'
J = R / 'jobs/csd-review-repair-build313'
BUILD268 = R / 'jobs/csd-review-repair-build268'
REBIND269 = R / 'jobs/csd-review-repair-rebind269'
TREE268 = W / 'review-repair268/workspace'
B = W / 'review-repair313/workspace'
C = B / 'explainer_corpus/complex_analysis_visual'
BINARY268 = W / 'bin/review-repair268'
BINARY = W / 'bin/review-repair313'

CRATE = 'explainer_corpus/complex_analysis_visual/'
BOUNDARY = CRATE + 'src/chapters/boundary.rs'
UNITS = CRATE + 'narration_units.json'
UNITS_BACKUP269 = CRATE + 'narration_units.before-review269.json'
MAP269 = CRATE + 'docs/narration-direction-map-frederick-review269.json'

SOURCE_COPY268_SHA256 = '81d28272a7a23b63199e79e6aa1086f01f131f0ddd685d4ae859d04b1de1934c'
BINARY268_SHA256 = '983954c508e42b90d443ad6ca09f2baba7565a4f609d09d2ea821640534d6ec7'
OLD_BOUNDARY_SHA256 = 'dc80c541799209f10d02701d18c7fab5a14327000719929e5e2fad0c6fd62f9f'
CANDIDATE = 'boundary-guide-repair310.rs'
CANDIDATE_SHA256 = 'bda7d931498b23bd3203cd0c6106de69dc0a8c5e160e738d698376322ff39e53'
FACTS_SHA256 = '7d528509752e532c07edb3fda7c12cd8251ac4ba77022355ae42ad725add4417'
RECIPE = 'review-repair-recipe313.json'
RECIPE_SCOPE = 'chapter 09 zoom-line repair'


def sha(p):
    with p.open('rb') as f:
        return hashlib.file_digest(f, 'sha256').hexdigest()


def snapshot(root):
    return {str(p.relative_to(root)): sha(p) for p in root.rglob('*') if p.is_file()}


def success(job):
    r = json.loads((job / 'resource-receipt.json').read_text())
    assert r.get('finished') and r.get('exit') == 0 and r.get('admitted'), job
    assert r['inputs_sha256'] == r['final_inputs_sha256'], job
    assert not any(r.get(k) for k in ('error', 'resource_stop', 'stopped_for_resource_limit')), job
    return r


def run(n, argv, cwd=None):
    with (J / (n + '.stdout')).open('xb') as o, (J / (n + '.stderr')).open('xb') as e:
        r = subprocess.run(list(map(str, argv)), cwd=cwd or C, stdout=o, stderr=e, timeout=7200)
    assert r.returncode == 0, (n, r.returncode)
    return (J / (n + '.stdout')).read_bytes()


def check_recipe():
    """Root's own permit for this build; cc0 does not write it."""
    recipe = json.loads((Q / RECIPE).read_text())
    assert recipe['reviewed_by_root'] is True and recipe['scope'] == RECIPE_SCOPE
    assert recipe['files'] == {'boundary.rs': {'candidate': CANDIDATE, 'sha256': CANDIDATE_SHA256}}
    for key in ('source_review', 'helper_review'):
        review = Q / recipe[key]
        assert review.parent == Q and review.is_file() and sha(review) == recipe[key + '_sha256'], key
    return recipe


def check_base():
    """build268's exact compiled tree, and the 268 workspace exactly as rebind269 left it."""
    success(BUILD268)
    success(REBIND269)
    assert sha(BUILD268 / 'source-copy.json') == SOURCE_COPY268_SHA256
    copied = json.loads((BUILD268 / 'source-copy.json').read_text())
    source = copied['files']
    assert len(source) == 70 and source[BOUNDARY] == OLD_BOUNDARY_SHA256
    assert UNITS_BACKUP269 not in source and MAP269 not in source
    q = json.loads((BUILD268 / 'qualification.json').read_text())
    assert q['binary_sha256'] == BINARY268_SHA256 == sha(BINARY268)
    assert q['tests_passed'] == 233 and q['explicit_native_plate_fade_probe_passed'] and q['clippy_passed']
    assert q['facts_sha256'] == FACTS_SHA256 and len(q['chapters']) == 15
    rebind = json.loads((REBIND269 / 'rebind-summary.json').read_text())
    live = snapshot(TREE268)
    assert set(live) == set(source) | {UNITS_BACKUP269, MAP269}, sorted(set(live) ^ set(source))
    for name, digest in source.items():
        if name != UNITS:
            assert live[name] == digest, name
    assert live[UNITS] == rebind['canonical_sha256']
    assert live[UNITS_BACKUP269] == source[UNITS]
    assert live[MAP269] == rebind['map_sha256']
    return source, q


def reconstruct(source):
    """B = build268's compiled tree, file by file, then the one substitution."""
    assert not B.parent.exists()
    for name in sorted(source):
        origin = TREE268 / (UNITS_BACKUP269 if name == UNITS else name)
        target = B / name
        target.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(origin, target)
        assert sha(target) == source[name], name
    assert snapshot(B) == source
    candidate = Q / CANDIDATE
    assert candidate.parent == Q and sha(candidate) == CANDIDATE_SHA256
    shutil.copy2(candidate, B / BOUNDARY)
    after = snapshot(B)
    changed = sorted(n for n in after if after[n] != source.get(n))
    assert set(after) == set(source) and changed == [BOUNDARY] and after[BOUNDARY] == CANDIDATE_SHA256
    return after, changed


def main():
    a = json.loads((J / 'resource-receipt.json').read_text())
    assert a['admitted'] and 'exit' not in a
    assert a['policy'] == dict(admission_gib=10, stop_free_gib=8, phase_growth_gib=4, work_cap_gib=56,
                               target_growth_cap_gib=20, sample_seconds=10)
    os.sched_setaffinity(0, {0, 1, 2, 3})
    check_recipe()
    source268, q268 = check_base()
    source, changed = reconstruct(source268)

    tool = Path(subprocess.check_output([str(Path.home() / '.cargo/bin/rustup'), 'which', '--toolchain',
                                         'nightly-2026-08-31', 'cargo'], text=True).strip())
    assert sha(tool) == 'b854c9424332416075760b8b5fc8a43254e8eea8f38f14a57deea6126810f1b9'
    assert sha(tool.parent / 'rustc') == '49f9fa06f3e2b93aba85273433fef67b984ddb4cacb4500de56e85f70ec2e339'
    for k in ['RUSTFLAGS', 'CARGO_ENCODED_RUSTFLAGS', 'RUSTC', 'RUSTC_WRAPPER', 'RUSTC_WORKSPACE_WRAPPER']:
        assert k not in os.environ
    os.environ.update(PATH=str(tool.parent) + ':/usr/bin:/bin', CARGO_INCREMENTAL='0', CARGO_BUILD_JOBS='2',
                      RCH_SHIM_LOCAL_IDE='1', CARGO_TARGET_DIR=str(W / 'target'), TMPDIR=str(W / 'tmp'),
                      XDG_CACHE_HOME=str(W / 'cache'))
    # format check only: the compiled bytes stay exactly the reviewed candidate's
    run('fmt-check', [tool, 'fmt', '--check', '--manifest-path', C / 'Cargo.toml', '--', '--config-path',
                      W / 'tree/repo-view/rustfmt.toml'])
    assert snapshot(B) == source
    with (J / 'source-copy.json').open('x') as f:
        json.dump(dict(files=source, changed=changed, base_source_copy_sha256=SOURCE_COPY268_SHA256,
                       base=str(BUILD268 / 'source-copy.json'), substituted={BOUNDARY: CANDIDATE}), f, indent=2)
    f = ['--release', '--locked', '--offline', '--config', str(W / 'tree/engine-budget.toml'),
         '--manifest-path', str(C / 'Cargo.toml'), '--bin', 'complex_analysis_visual']
    tests = run('tests', [tool, 'test', *f]).decode()
    counts = re.findall(r'^test result: ok\. (\d+) passed; 0 failed; 1 ignored; 0 measured; 0 filtered out;',
                        tests, re.M)
    assert len(counts) == 1 and int(counts[0]) == 233, tests[-2000:]
    probe = run('plate-fade-probe', [tool, 'test', *f, 'plate_fade_probe', '--', '--ignored', '--nocapture']).decode()
    assert re.search(r'^test result: ok\. 1 passed; 0 failed; 0 ignored; 0 measured; \d+ filtered out;',
                     probe, re.M), probe[-3000:]
    run('clippy', [tool, 'clippy', *f, '--no-deps', '--', '-D', 'warnings'])
    run('build', [tool, 'rustc', *f, '--', '-C', 'metadata=jmev_review_repair313'])
    assert not BINARY.exists()
    shutil.copy2(W / 'target/release/complex_analysis_visual', BINARY)
    assert sha(BINARY) != BINARY268_SHA256
    facts = run('facts', [BINARY, 'facts'])
    assert hashlib.sha256(facts).hexdigest() == FACTS_SHA256
    chapters = run('chapters', [BINARY, 'chapters']).decode().splitlines()
    assert chapters == q268['chapters']
    assert snapshot(B) == source
    with (J / 'qualification.json').open('x') as out:
        json.dump(dict(binary_sha256=sha(BINARY), tests_passed=int(counts[0]),
                       explicit_native_plate_fade_probe_passed=True, clippy_passed=True, fmt_check_passed=True,
                       facts_sha256=hashlib.sha256(facts).hexdigest(), chapters=chapters,
                       base_binary_sha256=BINARY268_SHA256, changed=changed,
                       scope='Native build of build268 plus boundary-guide-repair310 only; automated gates only. '
                             'No picture, motion, audio, fine-sync or release acceptance; no audio heard or checked.'), out, indent=2)
    print('Build313 passed fmt check, 233 tests, plate-fade probe, Clippy and facts; '
          'rebind, 09 timeline equality and the repaired pictures remain required.')


if __name__ == '__main__':
    main()
