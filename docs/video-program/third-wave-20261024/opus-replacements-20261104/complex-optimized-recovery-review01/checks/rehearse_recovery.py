#!/usr/bin/env python3
"""Rehearse the helper's --run with a fake binary and oracle in a temporary folder.

It runs no real render, no guard and no release binary, and writes nothing in the repository. The
one real step is the helper's read-only --check. Narration verify is stubbed here because only
the job's own run needs it. Run from the repository root:

    python3 -I -B <bundle>/checks/rehearse_recovery.py
"""
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile

HERE = Path(__file__).resolve()
HELPER = HERE.parents[1] / 'optimized_timeline_recovery01.py'
# case: what the fake does, and the words the failure must contain (None means it must pass)
CASES = {'pass': None, 'table': 'differ from the failed run', 'missing': '15_closing.timeline.tsv',
         'order': 'Cue placements differ', 'facts': 'Native facts differ from oracle', 'input': '/fake',
         'render-exit': 'CalledProcessError', 'existing-output': 'Existing timeline output retained',
         'policy': 'Guard policy changed'}
FAKE = r'''#!PYTHON -I
import json, shutil, sys
from pathlib import Path
c = json.loads((Path(sys.argv[0]).parent / 'config.json').read_text())
if sys.argv[1] == 'chapters':
    print('\n'.join(c['chapters']))
elif sys.argv[1] == 'facts':
    sys.stdout.buffer.write(Path(c['facts']).read_bytes() + (b'extra\t1\n' if c['case'] == 'facts' else b''))
else:
    assert sys.argv[1:6] == ['render', 'all', '--timeline-only', '--fps', '60']
    if c['case'] == 'render-exit':
        sys.exit(3)
    out = Path(sys.argv[sys.argv.index('--out') + 1])
    narration = sys.argv[sys.argv.index('--narration') + 1]
    out.mkdir()
    for chapter in c['chapters'][:-1 if c['case'] == 'missing' else None]:
        if chapter in c['prior']:
            for table in ('timeline', 'cues', 'cuelog'):
                shutil.copy(Path(c['prior_out']) / f'{chapter}.{table}.tsv', out)
            if c['case'] == 'table' and chapter == '02_turning_numbers':
                path = out / f'{chapter}.timeline.tsv'
                path.write_text(path.read_text().replace('frames\t5372', 'frames\t5373'))
        else:
            ids = [i for i in c['script'] if i.startswith(chapter[:2] + '_')]
            if c['case'] == 'order' and chapter == c['chapters'][-1]:
                ids.reverse()
            rows = [f'chapter\t{chapter}', 'fps\t60', 'frames\t9000']
            rows += [f'line\t{cue}\t{10 * k}\t0\t{narration}/{cue}.wav' for k, cue in enumerate(ids)]
            (out / f'{chapter}.timeline.tsv').write_text('\n'.join(rows) + '\n')
'''

results = []


def check(name, ok, detail):
    results.append(ok)
    print(f"{'PASS' if ok else 'FAIL'} {name}: {detail}", flush=True)


def load():
    spec = importlib.util.spec_from_file_location('recovery', HELPER)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def tree(root):
    return {str(p.relative_to(root)): hashlib.sha256(p.read_bytes()).hexdigest()
            for p in sorted(root.rglob('*')) if p.is_file()}


def child(case, tmp):
    H = load()
    H.BIN, H.ORACLE, H.MANIFEST = tmp / 'fake', tmp / 'oracle', tmp / 'inputs.json'
    H.B = H.T.B = tmp / 'B'
    H.verify_narration = lambda: None
    H.run()


def stage(H, case, tmp):
    """One fresh fake job folder per case, laid out as the guard leaves it."""
    tmp.mkdir()
    (tmp / 'B' / H.JOB).mkdir(parents=True)
    policy = dict(H.POLICY, work_cap_gib=48) if case == 'policy' else H.POLICY
    (tmp / 'B' / H.JOB / 'resource-receipt.json').write_text(
        json.dumps({'admitted': True, 'policy': policy, 'command': H.child_command()}))
    if case == 'existing-output':
        (tmp / 'B' / H.JOB / 'timeline-output').mkdir()
    (tmp / 'config.json').write_text(json.dumps({
        'case': case, 'chapters': list(H.CHAPTERS), 'prior': list(H.PRIOR_CHAPTERS), 'script': H.script_ids(),
        'prior_out': str(H.PRIOR_OUT), 'facts': str(H.FAILED / 'facts-native.stdout')}))
    for name, body in [('fake', FAKE.replace('PYTHON', sys.executable)),
                       ('oracle', f"import sys\nsys.stdout.buffer.write(open({str(H.FAILED / 'facts-native.stdout')!r}, 'rb').read())\n")]:
        (tmp / name).write_text(body)
        (tmp / name).chmod(0o755)
    digest = hashlib.sha256((tmp / 'fake').read_bytes()).hexdigest()
    (tmp / 'inputs.json').write_text(json.dumps({'files': [
        {'path': str(tmp / 'fake'), 'sha256': '0' * 64 if case == 'input' else digest}]}))


def main():
    H = load()
    before = tree(H.FAILED)
    run = subprocess.run([sys.executable, '-I', '-B', str(HELPER), '--check'], capture_output=True, text=True)
    report = json.loads(run.stdout) if run.returncode == 0 else {}
    check('c1-real-check', run.returncode == 0 and report.get('launched') is False and report['prior_tables'] == 12,
          f"the helper's read-only --check passes on the real tree: {report.get('frozen_files')} frozen files, "
          f"12 prior tables, nothing launched; {run.stderr[-200:]}")
    expected = ['-I', '-B', 'guard_owner56g_target20g.py', '--phase', 'native', '--job', H.JOB, '--inputs',
                'inputs01.json', '--', sys.executable, '-I', '-B', 'optimized_timeline_recovery01.py', '--run']
    shown = [Path(x).name if x.startswith('/') and x != sys.executable else x for x in H.guard_command()[1:]]
    check('c2-guard-command', shown == expected and H.PINS[H.GUARD].startswith('1c19a9b8'),
          f"--launch would run the pinned guard (not run here) as: guard --phase native --job {H.JOB}, with the "
          f"new manifest, 56/20/10/8/4 asserted, in the same lane lock")
    with tempfile.TemporaryDirectory() as folder:
        for case, wanted in CASES.items():
            tmp = Path(folder) / case
            stage(H, case, tmp)
            job = tmp / 'B' / H.JOB
            with (job / 'command.log').open('x') as log:
                code = subprocess.run([sys.executable, '-I', '-B', str(HERE), '--child', case, str(tmp)],
                                      stdout=log, stderr=subprocess.STDOUT).returncode
            text = (job / 'command.log').read_text()
            summary = job / 'timeline-summary.json'
            if wanted is None and not summary.exists():
                check(f'r-{case}', False, f"no summary; log ends {text.strip().splitlines()[-3:]}")
            elif wanted is None:
                d = json.loads(summary.read_text())
                compared = json.loads((job / 'prior-table-comparison.json').read_text())['tables']
                check(f'r-{case}', code == 0 and d['cues'] == 105 and d['prior_tables_compared'] == 12
                      and len(d['chapters_without_prior_table']) == 11
                      and d['full_baseline_equivalence_established'] is False
                      and all(row['normalized_equal'] for row in compared) and len(d['chapters']) == 15,
                      'all checks pass and the summary keeps the old keys; 12 tables equal; 11 chapters listed '
                      'without a prior table; full baseline equivalence stays false')
            else:
                check(f'r-{case}', code != 0 and not summary.exists() and wanted in text,
                      f'refused with no success summary ({wanted})')
    check('p1-failed-run-preserved', tree(H.FAILED) == before,
          f'all {len(before)} files of the failed job are byte-identical after every case')
    print(f"RESULT {'PASS' if all(results) else 'FAIL'}, {results.count(False)} failed")
    return 0 if all(results) else 1


if __name__ == '__main__':
    if sys.argv[1:2] == ['--child']:
        child(sys.argv[2], Path(sys.argv[3]))
    else:
        sys.exit(main())
