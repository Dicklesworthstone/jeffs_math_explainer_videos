"""CC0 controller 468: after a stable successful apportionment preview458, run helper 462's picture extraction
once, under the original guard. Prepared only; root starts it and nothing here has run.

  - Helper 462 is pinned (23575514) and so is the original guard chain: guard_owner80g_target20g.py (4f34e0d1),
    guard_owner56g_target20g.py (1c19a9b8), the base runner (819746fb). The guard's policy is read from the guard
    itself and must be 10 GiB native admission, 8 GiB stop, 4 GiB native growth, 80 GiB work cap, 20 GiB target
    growth, with its job folder under the five-film WORK that 462 reads.
  - It runs only on the Homebrew Python 3.15.0, and every child (462's checks run in this process, the guard and the
    extract child) uses exactly this sys.executable.
  - It refuses at start, while waiting and before each step if 462's input manifest or job folder exists, if another
    run of this controller is alive, or if a 462 inputs or extract process is alive.
  - It waits for timeline444, score445 and preview458. Any of them finishing without success stops it at once
    (nothing launched, no retry). preview458's receipt has to read successful twice, STABLE_SECONDS apart, with
    identical bytes. At the deadline it stops with exit 3, launching nothing.
  - Then, in this order and once: 462's own build() (all of 462's prerequisite checks, its sampling plan), 462's own
    manifest rows plus this controller's file, written once to 462's INPUTS path; then 462's guard command (the
    original guard, phase native, job apportionment-frederick-pictures462, child `-I -B <462> extract`) in the
    foreground. Its exit is returned; nothing is retried.

Modes (python3 -I -B cc0-apportionment-pictures-after-preview468.py MODE), from any folder:
  check                 read-only: pins, guard policy, Python, absent outputs, duplicates, the three receipts.
                        Exit 0 when preview458 is stably successful, 3 while pending, 1 on a failure or refusal.
  run --wait-hours N    root only: waits at most N hours (0 < N <= 24), then extracts as above.

It accepts no picture, motion, sound, fine-sync or AV result; its last line reports the guard's exit and the job
receipt's top-level fields exactly as read.
"""
import argparse
import hashlib
import importlib.util
import json
import os
import subprocess
import sys
import time
from pathlib import Path

VERSION = '468'
SELF = Path(__file__).resolve()
HERE = SELF.parent
HELPER = HERE/'cc0-apportionment-preview-review462.py'
HELPER_SHA = '235755141df29ee093daf0af2a91a2ed6a6d0dfc5db74586c13193e78128bf37'
PYTHON_HOME = Path('/opt/homebrew/Cellar/python@3.15/3.15.0/Frameworks/Python.framework/Versions/3.15')
POLICY = dict(admission_native_gib=10, stop_free_gib=8, phase_native_gib=4, work_cap_gib=80, target_growth_cap_gib=20)
POLL_SECONDS = 15
STABLE_SECONDS = 15
MAX_WAIT_HOURS = 24


def sha(path):
    with Path(path).open('rb') as f:
        return hashlib.file_digest(f, 'sha256').hexdigest()


def say(**fields):
    print(json.dumps(fields), flush=True)


def python_ok(executable=None, version=None):
    executable, version = executable or sys.executable, version or tuple(sys.version_info[:3])
    assert Path(executable).resolve().is_relative_to(PYTHON_HOME) and tuple(version) == (3, 15, 0), \
        'run with the Homebrew Python 3.15.0, not %s %s' % (executable, version)


def load_helper():
    assert sha(HELPER) == HELPER_SHA, 'helper 462 changed'
    spec = importlib.util.spec_from_file_location('helper462', HELPER)
    h = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(h)
    assert h.VERSION == '462' and h.HELPER == HELPER
    return h


def load_guard(h):
    """The original guard's own policy, read from the guard chain at its pins."""
    for path, digest in h.GUARD_CHAIN.items():
        assert sha(path) == digest, 'guard chain changed: %s' % path
    spec = importlib.util.spec_from_file_location('guard80g_for468', h.GUARD)
    wrapper = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(wrapper)
    guard = wrapper.load()
    got = dict(admission_native_gib=guard.ADMISSION_GIB['native'], stop_free_gib=guard.STOP_FREE_GIB,
               phase_native_gib=guard.CAPS['native'], work_cap_gib=guard.WORK_CAP_GIB,
               target_growth_cap_gib=guard.TARGET_GROWTH_CAP_GIB)
    assert got == POLICY, 'guard policy %s, expected %s' % (got, POLICY)
    assert Path(guard.WORK) == h.WORK, 'the guard writes jobs elsewhere than 462 reads'
    return got


def absent(h, inputs_too=True):
    assert not h.JOB.exists(), 'the 462 job folder already exists: '+str(h.JOB)
    if inputs_too:
        assert not h.INPUTS.exists(), 'the 462 input manifest already exists: '+str(h.INPUTS)


def no_duplicates():
    """No other run of this controller, and no 462 inputs or extract process (this process's ancestry excepted)."""
    listing = subprocess.run(['/bin/ps', '-axo', 'pid=,ppid=,command='], capture_output=True, text=True, check=True,
                             timeout=60).stdout
    procs = {}
    for line in listing.splitlines():
        parts = line.split(None, 2)
        if len(parts) == 3 and parts[0].isdigit() and parts[1].isdigit():
            procs[int(parts[0])] = (int(parts[1]), parts[2])
    mine, pid = set(), os.getpid()
    while pid in procs and pid not in mine:
        mine.add(pid)
        pid = procs[pid][0]
    mine.add(os.getpid())
    found = []
    for pid, (_, command) in procs.items():
        if pid in mine:
            continue
        words = command.split()
        if any(w.endswith(SELF.name) for w in words) and 'run' in words:
            found.append('%d controller468 run' % pid)
        if any(w.endswith(HELPER.name) for w in words) and {'inputs', 'extract'} & set(words):
            found.append('%d helper462 %s' % (pid, ' '.join(sorted({'inputs', 'extract'} & set(words)))))
    assert not found, 'already running: '+', '.join(found)


def state(h, receipt):
    try:
        top = json.loads(receipt.read_text())
    except (FileNotFoundError, json.JSONDecodeError):
        return 'pending', None
    if not top.get('finished'):
        return 'pending', None
    try:
        h.success(receipt)
    except (AssertionError, KeyError) as error:
        return 'failed', '%s: %s' % (type(error).__name__, error)
    return 'successful', None


def receipts(h):
    return dict(timeline444=h.T444/'resource-receipt.json', score445=h.S445/'resource-receipt.json',
                preview458=h.P458/'resource-receipt.json')


def check_pins(pins, when):
    for path, digest in pins.items():
        assert sha(path) == digest, 'changed %s: %s' % (when, path)


def check(args):
    python_ok()
    h = load_helper()
    policy = load_guard(h)
    absent(h)
    no_duplicates()
    states = {k: state(h, p) for k, p in receipts(h).items()}
    say(version=VERSION, python=sys.executable, policy=policy, receipts={k: v[0] for k, v in states.items()},
        failed={k: v[1] for k, v in states.items() if v[0] == 'failed'}, launched=False)
    if any(v[0] == 'failed' for v in states.values()):
        return 1
    return 0 if states['preview458'][0] == 'successful' else 3


def run(args):
    assert 0 < args.wait_hours <= MAX_WAIT_HOURS, 'wait hours must be in (0, %d]' % MAX_WAIT_HOURS
    python_ok()
    h = load_helper()
    policy = load_guard(h)
    pins = {SELF: sha(SELF), HELPER: HELPER_SHA, **h.GUARD_CHAIN, **h.PREVIEW_HELPERS}
    for path, digest in h.PREVIEW_HELPERS.items():
        assert sha(path) == digest, 'preview helper changed: %s' % path
    absent(h)
    no_duplicates()
    tracked = receipts(h)
    say(start=True, version=VERSION, python=sys.executable, policy=policy, wait_hours=args.wait_hours,
        pins={str(p): d for p, d in pins.items()}, launched=False)
    deadline, last = time.monotonic() + args.wait_hours*3600, None
    while True:
        states = {k: state(h, p) for k, p in tracked.items()}
        failed = {k: v[1] for k, v in states.items() if v[0] == 'failed'}
        if failed:
            say(stop='a prerequisite finished without success; nothing launched, no retry', failed=failed, launched=False)
            return 1
        if states['preview458'][0] == 'successful':
            first = sha(tracked['preview458'])
            time.sleep(STABLE_SECONDS)
            if sha(tracked['preview458']) == first and state(h, tracked['preview458'])[0] == 'successful':
                break
            continue
        now = {k: v[0] for k, v in states.items()}
        if now != last:
            say(waiting=now, launched=False)
            last = now
        if time.monotonic() >= deadline:
            say(stop='deadline reached; nothing launched', waiting=now, launched=False)
            return 3
        check_pins(pins, 'while waiting')
        absent(h)
        time.sleep(POLL_SECONDS)
    check_pins(pins, 'before the plan')
    absent(h)
    no_duplicates()
    plans, read = h.build()
    rows = h.manifest_rows(read)
    assert str(SELF) not in {r['path'] for r in rows}
    rows.append(dict(path=str(SELF), sha256=pins[SELF]))
    check_pins(pins, 'before the input manifest')
    absent(h)
    no_duplicates()
    with h.INPUTS.open('x') as f:
        json.dump(dict(files=rows), f, indent=2)
        f.write('\n')
    command = h.guard_command()
    assert command == [sys.executable, str(h.GUARD), '--phase', 'native', '--job', 'apportionment-frederick-pictures462',
                       '--inputs', str(h.INPUTS), '--', sys.executable, '-I', '-B', str(HELPER), 'extract'], command
    check_pins(pins, 'before the launch')
    absent(h, inputs_too=False)
    no_duplicates()
    say(inputs=str(h.INPUTS), inputs_sha256=sha(h.INPUTS), rows=len(rows),
        samples=sum(len(p['samples']) for p in plans.values()), launch=command, launched=True)
    code = subprocess.run(command, cwd=h.REPO, check=False).returncode
    receipt = h.JOB/'resource-receipt.json'
    try:
        top = json.loads(receipt.read_text())
        seen = {k: top.get(k, 'absent') for k in ('admitted', 'exit', 'finished', 'stopped_for_resource_limit', 'error')}
    except (OSError, ValueError) as error:
        seen = dict(receipt='unreadable: '+type(error).__name__)
    summary = h.JOB/'pictures462-summary.json'
    say(guard_exit=code, job_receipt=str(receipt), job_receipt_top_level=seen,
        summary_sha256=sha(summary) if summary.is_file() else None,
        note='extraction only; no picture, motion, sound, fine-sync or AV acceptance')
    return 0 if code == 0 else 1


def main():
    parser = argparse.ArgumentParser()
    modes = parser.add_subparsers(dest='mode', required=True)
    modes.add_parser('check')
    one = modes.add_parser('run')
    one.add_argument('--wait-hours', type=float, required=True)
    args = parser.parse_args()
    try:
        return {'check': check, 'run': run}[args.mode](args)
    except AssertionError as error:
        say(refused=str(error), launched=False)
        return 1


if __name__ == '__main__':
    raise SystemExit(main())
