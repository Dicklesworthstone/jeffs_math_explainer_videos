#!/usr/bin/env python3
"""Rehearse the stage07 build in a tiny temporary tree whose repository path is a symlink, as on ts1.

Real: the helper, its start script, ts1's guard01.py logic and the preserved guard, numpy and scipy, and the five stage07
files from the sealed repair. Fake: rustup and cargo, the lock tool, the binary, sbh and the old chain's exit file. In
the fixture copy only, the hostname and the hash pins of the old binary and macOS facts change. Nothing compiled, and
nothing is written in the repository.

    python3 -I -B <bundle>/checks/rehearse_build02.py
"""
import importlib.util
import json
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile

HERE = Path(__file__).resolve()
BUNDLE = HERE.parents[1]
PIPE = BUNDLE.parent
REAL = HERE.parents[7]
SPEC = importlib.util.spec_from_file_location('rehearse_qualify', PIPE / 'qualify01/checks/rehearse_qualify.py')
Q = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(Q)
sha, write = Q.sha, Q.write
FOLDER = 'docs/video-program/third-wave-20261024/opus-replacements-20261104/complex-remote-ts1-pipeline01/qualify02'
LOCK = 'docs/video-program/third-wave-20261024/opus-replacements-20261104/engine-geometry-validation02/verify_patched_lock.py'
REPAIR = 'docs/video-program/third-wave-20261024/opus-replacements-20261104/complex-cauchy-wording-repair01/staged/explainer_corpus/complex_analysis_visual'
TREE = Q.EC + '/native-stage07-root01'
CRATE = TREE + '/workspace/explainer_corpus/complex_analysis_visual'
NAMES = ('script.tsv', 'narration_units.json', 'docs/storyboard.md', 'docs/narration-direction-map-frederick-stage07.json', 'src/chapters/boundary.rs')
FAKE_RUSTUP = r'''#!PYTHON -I
import json, os, shutil, sys
from pathlib import Path
config = json.loads(Path(os.environ['CARGO_TARGET_DIR']).parent.joinpath('fake-cargo.json').read_text())
case = config['case']
assert sys.argv[1:4] == ['run', 'nightly-2026-08-31', 'cargo'], sys.argv
action = sys.argv[4]
finished = '    Finished `release` profile [optimized] target(s) in 1.00s'
if action == 'fmt':
    sys.exit(1 if case == 'fmt-fails' else 0)
if action == 'test':
    print(finished)
    tests = 206 if case == 'test-missing' else 207
    for k in range(tests):
        print(f'test chapters::t{k} ... {"FAILED" if case == "test-fails" and k == 9 else "ok"}')
    print(f"test result: {'FAILED' if case == 'test-fails' else 'ok'}. {tests} passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.00s")
    sys.exit(101 if case == 'test-fails' else 0)
target = Path(os.environ['CARGO_TARGET_DIR']) / 'release/complex_analysis_visual'
old = Path(config['old_binary'])
if action == 'build':
    print('   Compiling complex_analysis_visual v0.1.0')
    print(finished)
    text = old.read_text()
    if case == 'facts-differ':
        text = text.replace(config['facts_probe'], config['facts_other'])
    target.unlink()
    if case == 'same-binary':
        shutil.copy2(old, target)
    else:
        target.write_text(text + '\n# stage07 source\n')
        target.chmod(0o755)
    sys.exit(0)
if action == 'clippy':
    print('    Checking complex_analysis_visual v0.1.0')
    if case == 'clippy-warns':
        print('error: this could be rewritten')
        sys.exit(101)
    if case == 'target-moves':
        target.write_text(target.read_text() + '\n# moved\n')
    print(finished)
    sys.exit(0)
sys.exit(f'unexpected action {action}')
'''
results = []


def check(name, ok, detail=''):
    results.append(bool(ok))
    detail = detail.replace(Q.FIXTURE[0], '<fixture>') if Q.FIXTURE[0] else detail
    print(f"{'PASS' if ok else 'FAIL'} {name}{': ' + detail if detail else ''}", flush=True)


def extend_rows(t):
    """The two engine files build01.sh names, in inputs01 as on ts1."""
    engine = t.R / 'engine'
    write(engine / 'Cargo.lock.pinned', 'pinned\n')
    write(engine / 'engine-budget.toml', 'budget\n')
    path = t.O / 'inputs01.json'
    data = json.loads(path.read_text())
    data['files'] += [{'path': str(p), 'sha256': sha(p)} for p in (engine / 'Cargo.lock.pinned', engine / 'engine-budget.toml')]
    path.write_text(json.dumps(data))


def tree_files(t):
    """The five sealed files and a fake oracle, for fixtures that need no real crate."""
    crate = t.R / CRATE
    for name in NAMES:
        crate.joinpath(name).parent.mkdir(parents=True, exist_ok=True)
        shutil.copy(REAL / REPAIR / name, crate / name)
    shutil.copy(t.C / 'oracle.py', crate / 'oracle.py')
    rows = [{'path': str(p.relative_to(t.R)), 'sha256': sha(p)} for p in sorted(crate.rglob('*')) if p.is_file()]
    write(t.R / TREE / 'tree-receipt.json', json.dumps({'files': rows}))


def add_build(t, case='ok'):
    """Everything start-build02.sh needs in a fixture that holds qualify01's tree. tree_files or its like comes first."""
    write(t.R / LOCK, "print('PASS: the 19 packages are path copies, every other record is identical; patch.unused entries: 0')\n")
    folder = t.R / FOLDER
    folder.mkdir(parents=True, exist_ok=True)
    shutil.copy(BUNDLE / 'start-build02.sh', folder)
    text = (BUNDLE / 'ts1_qualify02.py').read_text()
    for name, value in (('OLD_BIN_SHA256', sha(t.O / 'complex_analysis_visual-release01')),
                        ('RELEASE_FACTS_SHA256', sha(t.R / Q.B / 'complex-stage06-optimized-build01/release-facts.tsv'))):
        text = text.replace(text.split(f"{name} = '")[1].split("'")[0], value)
    t.qualify02 = folder / 'ts1_qualify02.py'
    t.qualify02.write_text(text)
    write(Path(t.env['HOME']) / '.local/bin/rustup', FAKE_RUSTUP.replace('PYTHON', sys.executable), 0o755)
    cargo_case(t, case)


def cargo_case(t, case):
    write(t.O / 'fake-cargo.json', json.dumps({'case': case, 'old_binary': str(t.O / 'complex_analysis_visual-release01'),
                                               'facts_probe': 'a\\t1', 'facts_other': 'a\\t9'}))


def start_build(t):
    return subprocess.run(['bash', str(t.qualify02.parent / 'start-build02.sh')], env=t.env, capture_output=True, text=True)


def mode(t, flag):
    return subprocess.run([t.python, '-I', '-B', str(t.qualify02), flag], cwd=t.R, env=t.env, capture_output=True, text=True)


class Build(Q.Tree):
    def __init__(self, base, case='ok', old_chain_ended=True):
        super().__init__(base)
        tree_files(self)
        add_build(self, case)
        if old_chain_ended:
            write(self.O / 'chain-pictures-controller02.exit', '0\n')

    def rows(self):
        super().rows()
        extend_rows(self)

    def controller(self):
        return (self.O / 'build-controller02.log').read_text()

    def job(self):
        path = self.J / 'complex-remote-build02/command.log'
        return path.read_text() if path.exists() else ''


def fresh(root, name, *args, **kwargs):
    base = root / name
    base.mkdir()
    return Build(base.resolve(), *args, **kwargs)


def last(text):
    return Q.last_line(text, 'no output')


def replace_in(path, old, new):
    path.write_text(path.read_text().replace(old, new))


def edited_tree(t):
    (t.R / CRATE / 'script.tsv').write_text('changed\n')


def changed_old_binary(t):
    with (t.O / 'complex_analysis_visual-release01').open('a') as handle:
        handle.write('# changed\n')


def stray_file(t):
    write(t.R / CRATE / 'stray.rs', 'x\n')


CHECK_CASES = {
    'tree-file-changed': (edited_tree, 'Stage07 tree differs: script.tsv'),
    'tree-extra-file': (stray_file, 'Tree file set differs'),
    'old-binary-changed': (changed_old_binary, 'The stage06 binary copy changed'),
}
FAILURES = {
    'fmt-fails': 'CalledProcessError',
    'test-fails': 'CalledProcessError',
    'clippy-warns': 'CalledProcessError',
    'test-missing': 'Test result differs',
    'same-binary': 'cannot be the old one',
    'facts-differ': 'Native facts differ from the oracle',
    'target-moves': 'Target binary moved during clippy',
}


def main():
    with tempfile.TemporaryDirectory(prefix='build02-') as folder:
        root = Path(folder).resolve()
        Q.FIXTURE[0] = str(root)
        t = fresh(root, 'early', old_chain_ended=False)
        run = mode(t, '--check')
        check('check-runs-before-chain02-ends', run.returncode == 0 and '"chain02_ended": false' in run.stdout, last(run.stderr) if run.returncode else 'and says so')
        edited_tree(t)
        run = start_build(t)
        check('launch-refuses-a-bad-tree-before-the-wait', run.returncode != 0 and 'Stage07 tree differs' in t.controller(), last(t.controller()))
        t = fresh(root, 'baseline')
        target_before = sha(t.O / 'target/release/complex_analysis_visual')
        run = mode(t, '--check')
        check('check-passes', run.returncode == 0 and '"launched": false' in run.stdout, last(run.stderr) if run.returncode else 'nothing launched')
        check('check-spends-no-name', not (t.J / 'complex-remote-build02').exists() and not (t.O / 'build-inputs02.json').exists())
        run = start_build(t)
        check('build-through-start-script', run.returncode == 0 and 'BUILD PASS: 207 tests' in t.job(), last(t.controller() + t.job()))
        d = json.loads((t.J / 'complex-remote-build02/resource-receipt.json').read_text())
        check('guard-closed-the-job', d['exit'] == 0 and d['inputs_sha256'] == d['final_inputs_sha256'] == sha(t.O / 'build-inputs02.json') and d['policy'] == Q.POLICY)
        digest = (t.J / 'complex-remote-build02/binary.sha256').read_text().strip()
        release = t.O / 'complex_analysis_visual-release02'
        check('distinct-frozen-copy', release.is_file() and sha(release) == digest != sha(t.O / 'complex_analysis_visual-release01') and not release.samefile(t.O / 'target/release/complex_analysis_visual'))
        check('target-binary-was-replaced', sha(t.O / 'target/release/complex_analysis_visual') == digest != target_before, 'which is why the build waits for chain02')
        rows = {row['path']: row['sha256'] for row in json.loads((t.O / 'build-inputs02.json').read_text())['files']}
        check('manifest-swaps-the-tree', not [p for p in rows if '/native-stage06-root03/' in p] and all(str(t.R / CRATE / name) in rows for name in NAMES)
              and not [p for p in rows if p.startswith(str(t.physical))], 'stage07 rows in, stage06 tree rows out, all logical')
        probe = (f"import importlib.util\nspec = importlib.util.spec_from_file_location('q', {str(t.qualify02)!r})\nm = importlib.util.module_from_spec(spec)\nspec.loader.exec_module(m)\n"
                 "m.check_build()\nprint(m.BIN_SHA256)\n")
        run = subprocess.run([t.python, '-I', '-B', '-c', probe], cwd=t.R, env=t.env, capture_output=True, text=True)
        check('check-build-after-the-job', run.returncode == 0 and run.stdout.strip() == digest, last(run.stderr) if run.returncode else 'BIN_SHA256 reads the job')
        run = start_build(t)
        check('second-start-refused', run.returncode == 1 and 'refused' in run.stderr)
        for name, (damage, words) in CHECK_CASES.items():
            t = fresh(root, name)
            damage(t)
            run = mode(t, '--check')
            check('refuse-' + name, run.returncode != 0 and words in run.stderr, last(run.stderr))
        for case, words in FAILURES.items():
            t = fresh(root, case, case)
            old = sha(t.O / 'complex_analysis_visual-release01')
            run = start_build(t)
            text = t.controller() + t.job()
            check('stop-' + case, run.returncode != 0 and words in text and not (t.J / 'complex-remote-build02/binary.sha256').exists()
                  and sha(t.O / 'complex_analysis_visual-release01') == old, last(text))
    print(f"RESULT {'PASS' if all(results) else 'FAIL'}, {results.count(False)} failed")
    return 0 if all(results) else 1


if __name__ == '__main__':
    sys.exit(main())
