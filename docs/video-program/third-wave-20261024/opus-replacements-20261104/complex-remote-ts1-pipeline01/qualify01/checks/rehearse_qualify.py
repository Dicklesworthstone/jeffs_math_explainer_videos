#!/usr/bin/env python3
"""Rehearse the qualifier in a tiny temporary tree whose repository path is a symlink, as on ts1.

Real: the helper, its start script, ts1's guard01.py logic and the preserved guard, numpy and scipy
through a fake env, and the macOS build log as the base of the fixture log. Fake: the binary, the
receipts, sbh, and four test values. In the fixture copy only, the hostname, the two hash pins and
the repository-level file names change. Nothing is written in the repository.

    python3 -I -B <bundle>/checks/rehearse_qualify.py
"""
import hashlib
import json
import os
from pathlib import Path
import shutil
import socket
import subprocess
import sys
import tempfile

HERE = Path(__file__).resolve()
BUNDLE = HERE.parents[1]
REAL = HERE.parents[7]
PIPE = 'docs/video-program/third-wave-20261024/opus-replacements-20261104/complex-remote-ts1-pipeline01/qualify01'
EC = 'narration_lab/third-wave-20261024/root/elevenlabs-complex-rebuild-20261104'
B = 'narration_lab/third-wave-20261024/root/five-film-release-20261027'
GUARD = 'docs/video-program/third-wave-20261024/opus-replacements-20261104/engine-secondary-validation01/guard_owner56g_target20g.py'
SOURCE = 'narration_lab/production-20261020/second-wave-central/duration-and-storage-20261020/root/run_bounded_phase_oliver_owner48g_target14g_20261102.py'
CRATE = EC + '/native-stage06-root03/workspace/explainer_corpus/complex_analysis_visual'
POLICY = {'admission_gib': 10, 'stop_free_gib': 8, 'phase_growth_gib': 4, 'work_cap_gib': 56, 'target_growth_cap_gib': 20, 'sample_seconds': 10}
CHAPTERS = ['01_the_bump', '02_turning_numbers', '03_two_planes', '04_reading_color', '05_gallery', '06_one_derivative', '07_one_point',
            '08_loop_sums', '09_boundary', '10_rigidity', '11_counting', '12_residues', '13_real_integral', '14_continuation', '15_closing']
FACTS = 'a\t1\nb\t2\n'
TRACE = ('Traceback (most recent call last):\n  File "{o}", line 39, in <module>\n    import scipy.special as sp\n'
         "ModuleNotFoundError: No module named 'scipy'\n")
results = []
FIXTURE = ['']


def check(name, ok, detail=''):
    results.append(bool(ok))
    detail = detail.replace(FIXTURE[0], '<fixture>') if FIXTURE[0] else detail
    print(f"{'PASS' if ok else 'FAIL'} {name}{': ' + detail if detail else ''}", flush=True)


def last_line(text, empty):
    return text.strip().splitlines()[-1].replace(FIXTURE[0], '<fixture>')[:90] if text.strip() else empty


def sha(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def write(path, text, mode=None):
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(text)
    if mode:
        path.chmod(mode)


def receipt(job, command, inputs, exit_code):
    return {'started': 't', 'finished': 't', 'admitted': True, 'exit': exit_code, 'command': command, 'policy': POLICY,
            'inputs_sha256': inputs, 'final_inputs_sha256': inputs, 'samples': []}


class Tree:
    def __init__(self, base):
        self.base = base
        self.physical = base / 'data/repo'
        self.physical.mkdir(parents=True)
        (base / 'logical').symlink_to(base / 'data')
        self.R = base / 'logical/repo'
        self.O = self.R / EC / 'remote-ts1-01'
        self.J = self.O / 'jobs'
        self.helper = self.R / PIPE / 'ts1_qualify01.py'
        self.C = self.R / CRATE
        self.env = dict(os.environ, HOME=str(base / 'home'))
        home = base / 'home/.local/bin'
        write(home / 'sbh', '#!/bin/sh\nexit 0\n', 0o755)
        (home / 'python3').symlink_to(sys.executable)
        self.python = str(home / 'python3')
        (self.R / PIPE).mkdir(parents=True)
        for name in ('ts1_qualify01.py', 'start-qualify01.sh'):
            shutil.copy(BUNDLE / name, self.R / PIPE / name)
        for rel in (GUARD, SOURCE):
            (self.R / rel).parent.mkdir(parents=True, exist_ok=True)
            shutil.copy(REAL / rel, self.R / rel)
        guard = (REAL / EC / 'remote-ts1-01/guard01.py').read_text()
        write(self.O / 'guard01.py', guard.replace('thinkstation1', socket.gethostname().split('.')[0].lower()))
        write(self.O / 'guard-pins01.json', json.dumps({'files': [{'path': str(self.R / rel), 'sha256': sha(self.R / rel)} for rel in (GUARD, SOURCE)]}))
        write(self.O / 'build01.sh', '#!/bin/bash\n')
        write(self.C / 'oracle.py', f'import sys\nsys.stdout.write({FACTS!r})\n')
        write(self.R / B / 'complex-stage06-optimized-build01/release-facts.tsv', FACTS)
        write(self.R / 'source-row.txt', 'frozen\n')
        self.rows()
        self.binary(FACTS, CHAPTERS)
        build = self.J / 'complex-remote-build01'
        write(build / 'facts-native.tsv', FACTS)
        write(build / 'facts-oracle.tsv', '')
        log = (REAL / B / 'complex-stage06-optimized-build01/command.log').read_text(errors='replace')
        keep = log[:log.index('    Finished `release` profile', log.index('test result:'))]
        keep += '    Finished `release` profile [optimized] target(s) in 10.28s\n'
        keep = 'PASS: the 19 packages are path copies, every other record is identical; patch.unused entries: 0\n' + keep
        write(build / 'command.log', keep + '    Checking complex_analysis_visual v0.1.0\n    Finished `release` profile [optimized] target(s) in 44s\n' + TRACE.format(o=self.C / 'oracle.py'))
        write(self.O / 'build-controller01.exit', '1\n')
        write(self.O / 'build-controller01.log', 'controller\n')
        self.receipt = receipt('build', ['/bin/bash', str(self.O / 'build01.sh')], sha(self.O / 'inputs01.json'), 1)
        self.save()
        env = self.J / 'complex-remote-oracle-env01'
        (env / 'env/bin').mkdir(parents=True)
        (env / 'env/bin/python').symlink_to(sys.executable)
        write(env / 'versions.txt', '2.5.4 1.18.1\n')
        write(env / 'uv-cache/skipped.bin', 'never bound\n')
        write(env / 'resource-receipt.json', json.dumps(receipt('env', [], 'x', 0)))
        self.pin()

    def rows(self):
        paths = [self.O / 'guard01.py', self.O / 'guard-pins01.json', self.O / 'build01.sh', self.C / 'oracle.py', self.R / 'source-row.txt']
        write(self.O / 'inputs01.json', json.dumps({'files': [{'path': str(p), 'sha256': sha(p)} for p in paths]}))

    def binary(self, facts, chapters):
        escaped = facts.replace('\t', '\\t').replace('\n', '\\n')
        text = f"#!/bin/sh\ncase \"$1\" in\nfacts) printf '{escaped}' ;;\nchapters) printf '%s\\n' {' '.join(chapters)} ;;\nesac\n"
        write(self.O / 'complex_analysis_visual-release01', text, 0o755)
        write(self.O / 'target/release/complex_analysis_visual', text, 0o755)

    def save(self):
        write(self.J / 'complex-remote-build01/resource-receipt.json', json.dumps(self.receipt))

    def pin(self):
        text = (BUNDLE / 'ts1_qualify01.py').read_text()
        real_bin = text.split("BIN_SHA256 = '")[1].split("'")[0]
        real_inputs = text.split("BUILD_INPUTS_SHA256 = '")[1].split("'")[0]
        text = text.replace(real_bin, sha(self.O / 'complex_analysis_visual-release01')).replace(real_inputs, sha(self.O / 'inputs01.json'))
        text = text.replace("RELEASE_FACTS_SHA256 = '7d528509752e532c07edb3fda7c12cd8251ac4ba77022355ae42ad725add4417'",
                            f"RELEASE_FACTS_SHA256 = '{sha(self.R / B / 'complex-stage06-optimized-build01/release-facts.tsv')}'")
        self.helper.write_text(text)

    def mode(self, flag):
        return subprocess.run([self.python, '-I', '-B', str(self.helper), flag], cwd=self.R, env=self.env, capture_output=True, text=True)

    def start(self):
        return subprocess.run(['bash', str(self.R / PIPE / 'start-qualify01.sh')], env=self.env, capture_output=True, text=True)

    def log(self):
        path = self.J / 'complex-remote-build01/command.log'
        return path, path.read_text()


def edit_log(old, new):
    def apply(t):
        path, text = t.log()
        assert old in text
        path.write_text(text.replace(old, new))
    return apply


def set_receipt(key, value):
    def apply(t):
        t.receipt[key] = value
        t.save()
    return apply


def drop_receipt(key):
    def apply(t):
        del t.receipt[key]
        t.save()
    return apply


def pinned(change):
    def apply(t):
        change(t)
        t.pin()
    return apply


def new_oracle(t):
    write(t.C / 'oracle.py', "print('z')\n")
    t.rows()
    digest = sha(t.O / 'inputs01.json')
    t.receipt.update(inputs_sha256=digest, final_inputs_sha256=digest)
    t.save()
    t.pin()


def foreign_root(t):
    other = t.base / 'elsewhere'
    other.mkdir()
    path = t.physical / EC / 'remote-ts1-01/inputs01.json'
    path.write_text(path.read_text().replace(str(t.R), str(other)))


CHECK_CASES = {
    'build-exit-0': (set_receipt('exit', 0), 'applies only to the original exit 1'),
    'build-exit-101': (set_receipt('exit', 101), 'applies only to the original exit 1'),
    'build-unfinished': (drop_receipt('finished'), 'Build is not terminal'),
    'build-resource-stop': (set_receipt('stopped_for_resource_limit', True), 'Build stopped by the guard'),
    'build-guard-error': (set_receipt('error', 'x'), 'Build stopped by the guard'),
    'build-inputs-moved': (set_receipt('final_inputs_sha256', 'other'), 'Build inputs moved'),
    'build-policy': (set_receipt('policy', {}), 'Guard policy changed'),
    'build-command': (set_receipt('command', ['/bin/bash', 'other']), 'Build ran another command'),
    'controller-exit': (lambda t: write(t.O / 'build-controller01.exit', '0\n'), 'Controller exit differs'),
    'other-module': (edit_log("No module named 'scipy'", "No module named 'numpy'"), 'failed some other way'),
    'other-line': (edit_log('", line 39,', '", line 38,'), 'not at the oracle import'),
    'other-import': (edit_log('import scipy.special as sp', 'import numpy as np'), 'not at the oracle import'),
    'second-traceback': (edit_log('    Checking complex', 'Traceback (most recent call last):\n    Checking complex'), 'More than one traceback'),
    'clippy-error': (edit_log('    Checking complex', 'error: could not compile\n    Checking complex'), 'another error'),
    'no-lock-pass': (edit_log('PASS: the 19 packages', 'PASS: the 18 packages'), 'Lock check missing'),
    'tests-one-fewer': (edit_log('ok. 207 passed', 'ok. 206 passed'), 'Test result differs'),
    'tests-one-failed': (edit_log('207 passed; 0 failed', '206 passed; 1 failed'), 'Test result differs'),
    'test-line-failed': (lambda t: (lambda p, x: p.write_text(x.replace(' ... ok\n', ' ... FAILED\n', 1)))(*t.log()), 'Test lines differ'),
    'no-clippy-finish': (edit_log('    Finished `release` profile [optimized] target(s) in 44s\n', ''), 'Expected test, build and clippy'),
    'steps-out-of-order': (lambda t: (lambda p, x: p.write_text(x.replace('    Checking complex_analysis_visual v0.1.0\n', '').replace('test result:', '    Checking complex_analysis_visual v0.1.0\ntest result:', 1)))(*t.log()), 'Build steps out of order'),
    'no-native-facts': (lambda t: write(t.J / 'complex-remote-build01/facts-native.tsv', ''), 'Native facts missing'),
    'oracle-wrote': (lambda t: write(t.J / 'complex-remote-build01/facts-oracle.tsv', 'x'), 'Oracle wrote output'),
    'chapters-present': (lambda t: write(t.J / 'complex-remote-build01/chapters.txt', 'x'), 'went past its oracle line'),
    'target-differs': (lambda t: write(t.O / 'target/release/complex_analysis_visual', 'other'), 'differs from the pinned target'),
    'binary-differs': (lambda t: write(t.O / 'complex_analysis_visual-release01', 'other'), 'differs from the pinned target'),
    'source-row-changed': (lambda t: write(t.R / 'source-row.txt', 'edited\n'), 'source-row.txt'),
    'env-failed': (lambda t: write(t.J / 'complex-remote-oracle-env01/resource-receipt.json', json.dumps(receipt('env', [], 'x', 1))), 'AssertionError'),
    'env-missing-python': (lambda t: (t.J / 'complex-remote-oracle-env01/env/bin/python').unlink(), 'FileNotFoundError'),
    'foreign-root': (foreign_root, 'Manifest root is not this repository'),
    'macos-facts-changed': (lambda t: write(t.R / B / 'complex-stage06-optimized-build01/release-facts.tsv', 'c\t3\n'), 'macOS facts changed'),
    'duplicate-job': (lambda t: (t.J / 'complex-remote-qualify01').mkdir(), 'Duplicate qualification refused'),
}
RUN_CASES = {
    'oracle-differs': (new_oracle, 'Native facts differ from the oracle'),
    'original-facts-differ': (lambda t: (write(t.J / 'complex-remote-build01/facts-native.tsv', 'c\t3\n')), 'differ from the original build'),
    'chapter-registry': (pinned(lambda t: t.binary(FACTS, CHAPTERS[:14])), 'Chapter registry differs'),
    'binary-facts-fail': (pinned(lambda t: (lambda text: [write(p, text, 0o755) for p in (t.O / 'complex_analysis_visual-release01', t.O / 'target/release/complex_analysis_visual')])("#!/bin/sh\nexit 3\n")), "('facts-native', 3)"),
}


def fresh(root, name):
    base = root / name
    base.mkdir()
    return Tree(base.resolve())


def main():
    log = (REAL / B / 'complex-stage06-optimized-build01/command.log').read_text(errors='replace')
    with tempfile.TemporaryDirectory(prefix='qualify-') as folder:
        root = Path(folder).resolve()
        FIXTURE[0] = str(root)
        t = fresh(root, 'baseline')
        check('fixture-alias', t.R.is_symlink() is False and (t.base / 'logical').is_symlink() and t.helper.resolve() != t.helper, 'the fixture repository sits behind a symlink, as on ts1')
        run = t.mode('--check')
        check('check-mode', run.returncode == 0 and '"launched": false' in run.stdout and not (t.J / 'complex-remote-qualify01').exists(), run.stderr[-200:])
        run = t.start()
        job = t.J / 'complex-remote-qualify01'
        check('end-to-end', run.returncode == 0 and (t.O / 'qualify-controller01.exit').read_text().strip() == '0', 'the start script ran the job through the real guard')
        d = json.loads((job / 'resource-receipt.json').read_text())
        check('real-guard-receipt', d['exit'] == 0 and d['policy'] == POLICY and d['inputs_sha256'] == d['final_inputs_sha256'] and d['command'][-1] == '--run', 'the real guard admitted and closed the job')
        s = json.loads((job / 'qualification-summary.json').read_text())
        check('summary', s['original_build_exit'] == 1 and s['original_build_relabelled'] is False and s['tests_passed'] == 207 and s['chapters'] == CHAPTERS and s['fact_rows'] == 2, 'the original exit 1 stays in the record')
        before = json.loads((t.J / 'complex-remote-build01/resource-receipt.json').read_text())
        check('original-untouched', before['exit'] == 1 and sha(t.J / 'complex-remote-build01/resource-receipt.json') == s['original_receipt_sha256'], 'the original receipt is unchanged')
        rows = {row['path'] for row in json.loads((t.O / 'qualify-inputs01.json').read_text())['files']}
        wanted = [t.J / 'complex-remote-build01/resource-receipt.json', t.J / 'complex-remote-build01/command.log', t.O / 'inputs01.json', t.O / 'complex_analysis_visual-release01',
                  t.O / 'target/release/complex_analysis_visual', t.J / 'complex-remote-oracle-env01/versions.txt', t.R / 'source-row.txt', t.helper]
        check('manifest-binds-evidence', all(str(p) in rows for p in wanted) and not [p for p in rows if 'uv-cache' in p or str(t.physical) in p], 'all rows are logical, and the evidence is frozen')
        check('logical-strings', d['command'][-2].startswith(str(t.R)), 'the guard ran the logical path')
        run = t.start()
        check('second-start-refused', run.returncode == 1 and 'refused' in run.stderr and (t.O / 'qualify-controller01.exit').read_text().strip() == '0')
        for name, (damage, words) in CHECK_CASES.items():
            t = fresh(root, name)
            damage(t)
            run = t.mode('--check')
            check('refuse-' + name, run.returncode != 0 and words in run.stderr, last_line(run.stderr, 'no refusal'))
        for name, (damage, words) in RUN_CASES.items():
            t = fresh(root, name)
            damage(t)
            run = t.start()
            text = (t.J / 'complex-remote-qualify01/command.log').read_text()
            check('stop-' + name, run.returncode != 0 and words in text and not (t.J / 'complex-remote-qualify01/qualification-summary.json').exists(), last_line(text, 'no log'))
    print(f"RESULT {'PASS' if all(results) else 'FAIL'}, {results.count(False)} failed")
    return 0 if all(results) else 1


if __name__ == '__main__':
    sys.exit(main())
