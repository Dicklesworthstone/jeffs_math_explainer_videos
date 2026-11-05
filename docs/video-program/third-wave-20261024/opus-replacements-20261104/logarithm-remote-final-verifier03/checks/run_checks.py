"""Narrow checks for the verifier03 delta. From the repository root, with a fresh scratch path:
python3 -I -B CHECKS/run_checks.py SCRATCH

Real: both helpers' run_job, called on tiny receipt files. Nothing is launched, no media exists, and verifier02's full
check suite isn't rerun, because the only code it covered that changes is listed in EXPECTED below.
"""
import contextlib
import difflib
import hashlib
import importlib.util
import io
import json
from pathlib import Path
import sys
import threading
import time

HERE = Path(__file__).resolve().parents[1]
OLD = HERE.parent / 'logarithm-remote-final-verifier02'
SEAL_02 = 'ed88be4568c6150219cfa2678bfe2bc7266efd94156a206015ee8cc92bf455da'
HELPER_02 = '361fe7334b1a3c332e4d7b19a8bba01457e1bf85f62459dc407474069d12c20d'
PYTHON = '/opt/homebrew/opt/python@3.14/bin/python3.14'
SCRIPT = '/lane/job-scripts/logarithm-remote-verify01.py'
GOOD = {'admitted': True, 'phase': 'native', 'command': [PYTHON, '-I', '-B', SCRIPT]}
PARTIAL = '{"started": "2026-11-05T23:00:00Z", "phase": "nat'
PAST_THE_GATE = 'the remote verifier runs only on'  # run_job's next gate after the receipt: proof this one passed
GATE = "does not show this script running"
EXPECTED = {  # every changed line of code; the docstring's three new lines are the only others
    '-': ['ANCHORS = HERE.parent / "anchors.json"', 'anchors = resolve_anchors(load(ANCHORS)["rows"], place, total)',
          'started = json.loads(receipt.read_text())'],
    '+': ['import time', 'ANCHORS = HERE.parent.parent / "logarithm-remote-final-verifier02/anchors.json"  # sealed there; never copied',
          'ANCHORS_SHA256 = "15dd516477629a8addb1282b4ab5880cb89d32019d0d1e0df889f01a43940c71"',
          'def live_receipt(path):', 'for _ in range(100):', 'try:', 'return json.loads(Path(path).read_text())',
          'except json.JSONDecodeError:', 'time.sleep(0.1)', 'raise Refused(f"the guard\'s receipt never settled: {path}")',
          'def load_anchors():', 'need(sha(ANCHORS) == ANCHORS_SHA256, f"{ANCHORS} is not the sealed anchors.json")',
          'return load(ANCHORS)', 'anchors = resolve_anchors(load_anchors()["rows"], place, total)', 'started = live_receipt(receipt)'],
}


def sha(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def check(ok, label):
    if not ok:
        raise SystemExit(f'FAIL {label}')


def module(name, path):
    spec = importlib.util.spec_from_file_location(name, path)
    found = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(found)
    return found


def delta():
    check(sha(OLD / 'SHA256SUMS') == SEAL_02 and sha(OLD / 'verify_remote_final.py') == HELPER_02, 'verifier02 changed')
    rows = [x.split('  ', 1) for x in (OLD / 'SHA256SUMS').read_text().splitlines()]
    check(all(sha(OLD / name) == digest for digest, name in rows), 'a verifier02 file changed')
    old, new = (OLD / 'verify_remote_final.py').read_text(), (HERE / 'verify_remote_final.py').read_text()
    lines = list(difflib.unified_diff(old.splitlines(True), new.splitlines(True), 'verifier02/verify_remote_final.py', 'verify_remote_final.py'))
    check((HERE / 'docs/remote02-to-remote03.diff').read_text() == ''.join(lines), 'diff document')
    body = [x for x in lines[2:] if x[0] in '+-']
    gone = [x[1:].strip() for x in body if x[0] == '-']
    added = [x[1:].strip() for x in body if x[0] == '+' and x[1:].strip()]
    doc = ['Revision 03 changes only', 'the guard\'s live receipt now', 'pinned by hash. docs/remote02-to-remote03.diff', '"""This job\'s receipt while',
           'can land inside', '"""Outside this folder\'s seal']
    code = [x for x in added if not any(x.startswith(d) for d in doc)]
    check(gone == EXPECTED['-'] and code == EXPECTED['+'], f'unexpected change: {gone} {code}')
    print(f'PASS keep verifier02 unchanged, {len(rows)} sealed files; the diff to it is exactly {len(gone)} lines out and {len(code)} code lines in, listed in EXPECTED')
    check((HERE / 'anchors.json').exists() is False, 'anchors copied')


def run(helper, base, text=None, later=None, delay=0.3):
    """helper's real run_job on one job folder, with a receipt that may arrive whole only after a delay."""
    job = base / 'logarithm-remote-verify01'
    job.mkdir(parents=True)
    path = job / 'resource-receipt.json'
    if text is not None:
        path.write_text(text)
    writer = None
    if later is not None:
        writer = threading.Timer(delay, lambda: path.write_text(json.dumps(later)))
        writer.start()
    out = io.StringIO()
    began = time.monotonic()
    with contextlib.redirect_stdout(out):
        try:
            code = helper.run_job({'lane': str(base), 'job': 'logarithm-remote-verify01', 'script': SCRIPT})
        except json.JSONDecodeError:
            code = 'JSONDecodeError'
    if writer:
        writer.join()
    return code, out.getvalue().strip(), time.monotonic() - began


def behaviour(base):
    old = module('verifier02_as_sealed', OLD / 'verify_remote_final.py')
    new = module('verifier03', HERE / 'verify_remote_final.py')
    check(new.ANCHORS.resolve() == (OLD / 'anchors.json').resolve() and new.load_anchors() == old.load(old.ANCHORS), 'anchors')
    other = base / 'anchors-other.json'
    other.parent.mkdir(exist_ok=True)
    other.write_text('{"rows": []}\n')
    new.ANCHORS, kept = other, new.ANCHORS
    try:
        new.load_anchors()
        check(False, 'other anchors accepted')
    except new.Refused as refusal:
        check('is not the sealed anchors.json' in str(refusal), 'anchors refusal')
    new.ANCHORS = kept
    print('PASS anchors load from verifier02\'s sealed file, and any other bytes there refuse')
    check(new.VERIFY_SUCCESS == old.VERIFY_SUCCESS, 'success line')
    code, said, _ = run(old, base / 'old', PARTIAL, later=GOOD)
    check(code == 'JSONDecodeError', f'verifier02 should fail here: {code} {said}')
    print('PASS control verifier02 dies on the partial receipt with an uncaught JSONDecodeError: the defect is real here')
    code, said, took = run(new, base / 'whole', json.dumps(GOOD))
    check(code == 1 and PAST_THE_GATE in said and took < 1, f'whole: {code} {said}')
    print('PASS whole receipt: no delay, and it passes this gate; the next gate then refuses this host, as it must here')
    code, said, took = run(new, base / 'partial', PARTIAL, later=GOOD)
    check(code == 1 and PAST_THE_GATE in said and 0.2 < took < 3, f'partial then valid: {code} {said} {took}')
    print(f'PASS malformed then valid: rereads, then passes this gate after about {took:.1f} s')
    code, said, _ = run(new, base / 'empty', '', later=GOOD)
    check(code == 1 and PAST_THE_GATE in said, f'empty then valid: {code} {said}')
    print('PASS empty then valid: the same')
    faults = (('another script', {**GOOD, 'command': [PYTHON, '-I', '-B', '/other.py']}), ('not admitted', {**GOOD, 'admitted': False}),
              ('already exited', {**GOOD, 'exit': 0}), ('another interpreter form', {**GOOD, 'command': ['bash', '-I', '-B', SCRIPT]}))
    for name, receipt in faults:
        code, said, _ = run(new, base / name.replace(' ', '-'), PARTIAL, later=receipt)
        check(code == 1 and GATE in said, f'{name}: {code} {said}')
        print(f'PASS malformed then wrong, {name}: the semantic gate still refuses')
    code, said, took = run(new, base / 'forever', PARTIAL)
    check(code == 1 and "the guard's receipt never settled" in said and 9 < took < 30, f'forever: {code} {said} {took}')
    print(f'PASS permanently malformed: REFUSED after {took:.0f} s, the bound, with exit 1')
    code, said, took = run(new, base / 'missing')
    check(code == 1 and 'launch only through the guard' in said and took < 1, f'missing: {code} {said}')
    print('PASS missing receipt: refused at once, not reread')


def main():
    base = Path(sys.argv[1]).resolve()
    base.mkdir()
    delta()
    behaviour(base)
    print('ALL PASS on this host, with tiny receipt files. No job, guard or worker tool ran.')


if __name__ == '__main__':
    main()
