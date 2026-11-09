"""Final-verification launch controller 452 for the complex master 447 (CC2). Prepared for root's review; not run.

It waits for the actual master job csd-combined-master447 (cc0-combined-master447.py, launched by root from CC0's
manifest 384) to finish successfully, then runs the frozen verifier 451 (csd-final-verify451.py, hashed before any
of its code runs) unchanged, in the run order of its notes (451 notes section 4, 363 notes section 5):
  1. check            read-only; the plan_sha256 it prints is the plan both picture jobs must be cut from;
  2. launch-verify    451's guarded launch of csd-final-verify451;
  3. launch-pictures  451 starts csd-final-pictures451-overview;
  4. the external ledger (review 339 L1/N1): at overview success, the sampling.json sha256 from that job's last line
     is confirmed read-only against the file and copied into Q/final-pictures451-overview-ledger452.json, outside the
     job folder, written once and never rewritten; the ledger's own sha256 is printed;
  5. immediately before the native launch, sampling.json and the ledger are re-hashed against it;
  6. launch-pictures  451 starts csd-final-pictures451-native; the ledger must equal the printed
     overview_sampling_sha256, the native inputs row and union.overview_sampling_sha256;
  7. before viewing, every picture and sheet both jobs recorded is re-hashed (451's own part_done()).
Every launch is 451's own, so its gates, never-duplicate checks, free-space and CPU refusals, bound inputs and guard
argv are unchanged. 452 adds only refusals: any 451 job, inputs file or the ledger existing before it starts; any
pinned file changing; a finished failure of the master job or of any step, which stops the chain at once with
nothing retried. It writes the ledger and nothing else, and records no review or acceptance. Every sheet and native
frame still needs root's viewing; picture, motion, listening, fine sync, caption playback and independent AV review
stay open, and nothing here authorizes publication.

Run on CSD:  /usr/bin/python3 -I -B csd-final-controller452.py --wait-hours H   (0 <= H <= 48)
"""
import argparse
from datetime import datetime, timezone
import hashlib
import importlib.util
import json
from pathlib import Path
import re
import subprocess
import time

D = Path('/data/projects/jmev-stroke-probe-20261106')            # CSD
Q = D/'qualification01/production-repair17'
VERIFIER = Q/'csd-final-verify451.py'
VERIFIER_SHA = 'c876caf664124e9becc1d8698608ae456978badbfca114316f93459a18186d03'   # 451 root-bound successor; review378 covers predecessor377 only
LEDGER = Q/'final-pictures451-overview-ledger452.json'
POLL = 30
# 451 pictures(): the overview job's last line, written into its own command.log by the guard
LAST_LINE = re.compile(r"^(\d+) pictures in (\d+) sheets await root's viewing; the native job is next\. "
                       r"sampling\.json sha256 ([0-9a-f]{64})$")


def digest(path):
    with Path(path).open('rb') as stream:
        return hashlib.file_digest(stream, 'sha256').hexdigest()


def load():
    """The frozen 451, hashed before it is loaded (CC3 297 F2, as csd-launch-final352.py loads its worker). Loading it
    runs no I/O: its top level is constants and functions only."""
    path = VERIFIER.resolve(strict=True)
    assert digest(path) == VERIFIER_SHA, 'not the frozen 451: '+str(path)
    spec = importlib.util.spec_from_file_location('verifier451', path)
    v = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(v)
    assert (v.VERSION, v.D, v.Q, v.PYTHON, v.PICTURE_PARTS) == ('451', D, Q, '/usr/bin/python3', ('overview', 'native'))
    assert (v.MASTER, v.MASTER_INPUTS) == (D/'jobs/csd-combined-master447', Q/'combined-master447-inputs.json')
    return path, v


def unchanged(pins):
    for path, value in pins.items():
        assert digest(path) == value, 'changed: '+str(path)


def master_finished(v, pins, hours):
    """Wait, as 447's own launch() waits on its rows, for the master receipt to be written finished. A receipt not
    yet there, or read while the guard rewrites it in place (every 10 s), is pending. A finished failure (refused
    admission, non-zero exit, inputs changed, error or resource stop) stops here: no 451 job, no retry. The finished
    receipt must then read the same one poll later."""
    receipt = v.MASTER/'resource-receipt.json'
    deadline = time.monotonic() + hours*3600
    while True:
        unchanged(pins)
        try:
            done = receipt.exists() and bool(v.read(receipt).get('finished'))
        except ValueError:
            done = False
        if done:
            break
        assert time.monotonic() < deadline, 'master447 not finished; no 451 job launched'
        time.sleep(POLL)
    v.success(receipt)
    first = digest(receipt)
    time.sleep(POLL)
    assert digest(receipt) == first, 'master447 receipt changed after it finished; no 451 job launched'
    return first


def step(path, v, pins, *mode):
    """One mode of the frozen 451, exactly as root would type it. Its output is echoed; its JSON lines are returned.
    A non-zero exit (a 451 refusal, the guard's refused admission, a failed job) stops the chain."""
    unchanged(pins)
    argv = [v.PYTHON, '-I', '-B', str(path), *mode]
    print('452 runs: '+' '.join(argv), flush=True)
    child = subprocess.Popen(argv, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True)
    lines = []
    for line in child.stdout:
        print(line, end='', flush=True)
        lines.append(line)
    code = child.wait()
    assert code == 0, '451 %s exited %d; nothing further launched' % (' '.join(mode), code)
    return [json.loads(x) for x in lines if x.startswith('{')]


def launched(rows, job, **want):
    """451 launch() prints one line naming its job before the guard starts it; the guard's own line names none."""
    (row,) = [r for r in rows if 'job' in r]
    assert row['job'] == job.name and all(row[k] == x for k, x in want.items()), row
    return row


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--wait-hours', type=float, required=True, help='0-48: how long to wait for master447')
    args = parser.parse_args()
    assert 0 <= args.wait_hours <= 48
    path, v = load()
    (overview, overview_inputs), (native, native_inputs) = (v.part_paths(p) for p in v.PICTURE_PARTS)
    # never duplicate or overwrite: 451 repeats each job and inputs check at its own launch
    for p in (v.VERIFY, v.VERIFY_INPUTS, overview, overview_inputs, native, native_inputs, LEDGER):
        assert not p.exists(), 'already exists; nothing launched: '+str(p)
    pins = {p: digest(p) for p in (Path(__file__).resolve(), path, v.GUARD)}
    master_receipt = master_finished(v, pins, args.wait_hours)

    # 1. check (read-only). Its first gate is 451's stable() on the master job and every row it bound.
    (c,) = [r for r in step(path, v, pins, 'check') if r.get('check') == 'ok']
    assert c['frames'] == v.TOTAL_FRAMES and [x['job'] for x in c['picture_jobs']] == [overview.name, native.name]
    plan = c['plan_sha256']

    # 2. the verification job
    launched(step(path, v, pins, 'launch-verify'), v.VERIFY)
    v.success(v.VERIFY/'resource-receipt.json')

    # 3. the overview picture job: 451 starts the first part with no job folder
    assert not overview.exists() and not native.exists()
    launched(step(path, v, pins, 'launch-pictures'), overview, part='overview', plan_sha256=plan)
    v.success(overview/'resource-receipt.json')

    # 4. the external ledger, before anything else touches the overview folder
    sampling = overview/'sampling.json'
    last = [x for x in (overview/'command.log').read_text(errors='replace').splitlines() if x.strip()][-1]
    m = LAST_LINE.match(last)
    assert m, 'the overview job did not print its sampling.json sha256 last'
    held = digest(sampling)
    assert held == m.group(3), 'overview sampling.json is not the one its job printed'
    record = v.read(sampling)
    assert (record['part'], record['plan_sha256'], len(record['samples']), len(record['sheets'])) == \
        ('overview', plan, int(m.group(1)), int(m.group(2))), 'overview sampling.json differs from its printed line'
    with LEDGER.open('x') as stream:
        json.dump(dict(controller='452', written_utc=datetime.now(timezone.utc).isoformat(), verifier_sha256=VERIFIER_SHA,
                       master_receipt_sha256=master_receipt, verify_receipt_sha256=digest(v.VERIFY/'resource-receipt.json'),
                       overview_job=overview.name, overview_receipt_sha256=digest(overview/'resource-receipt.json'),
                       overview_last_line=last, sampling_json=str(sampling), overview_sampling_sha256=held,
                       plan_sha256=plan), stream, indent=2)
    ledger = digest(LEDGER)
    print('452 ledger %s sha256 %s holds overview sampling.json sha256 %s' % (LEDGER, ledger, held), flush=True)

    # 5. immediately before the native launch
    assert digest(LEDGER) == ledger and digest(sampling) == held, 'overview sampling.json or the ledger changed; no native job'
    assert not native.exists() and not native_inputs.exists()

    # 6. the native picture job, then the ledger against all three
    launched(step(path, v, pins, 'launch-pictures'), native, part='native', plan_sha256=plan, overview_sampling_sha256=held)
    v.success(native/'resource-receipt.json')
    bound = {x['path']: x['sha256'] for x in v.read(native_inputs)['files']}
    assert bound.get(str(sampling)) == held, 'the native job bound another overview sampling.json'
    union = v.read(native/'sampling.json')['union']
    assert (union['overview_sampling_sha256'], union['plan_sha256']) == (held, plan), 'native union differs from the ledger'

    # 7. before viewing: every recorded picture and sheet at its recorded sha256, none missing or added
    counts = {}
    for p in v.PICTURE_PARTS:
        record, _ = v.part_done(p, plan)
        counts[p] = dict(pictures=len(record['samples'] if p == 'overview' else record['natives']), sheets=len(record['sheets']),
                         sampling_sha256=digest(v.part_paths(p)[0]/'sampling.json'))
    assert digest(LEDGER) == ledger and counts['overview']['sampling_sha256'] == held
    print(json.dumps(dict(controller='452', plan_sha256=plan, ledger=str(LEDGER), ledger_sha256=ledger, parts=counts,
                          scope='Launch order and hashes only. Every sheet and native frame awaits root\'s viewing; '
                          'no review or acceptance is recorded.')), flush=True)
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
