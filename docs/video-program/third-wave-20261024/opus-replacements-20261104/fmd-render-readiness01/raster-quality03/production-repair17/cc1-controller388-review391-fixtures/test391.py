"""task391 (CC1): runs the frozen 388 against the 377 stand-in and root's guard runner in fresh scratch trees.
Every scenario has its own tree under runs/ and its own log; nothing is reused or deleted.
Usage: python3 -I -B test391.py REAL388 RUNNER819746fb
"""
import datetime
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import threading
import time

T = Path(__file__).resolve().parent
RUNS = T / 'runs'
REAL388, RUNNER = Path(sys.argv[1]).resolve(), Path(sys.argv[2]).resolve()
FAKE377, FAKEGUARD, RUN388 = T / 'fake377-391c.py', T / 'fakeguard391.py', T / 'run388_391.py'
QREL = 'qualification01/production-repair17'
MASTER_REL = 'jobs/csd-combined-master350'
PATHS = {'verify_job': 'jobs/csd-final-verify377', 'verify_inputs': QREL + '/final-verify377-inputs.json',
         'overview_job': 'jobs/csd-final-pictures377-overview', 'overview_inputs': QREL + '/final-pictures377-overview-inputs.json',
         'native_job': 'jobs/csd-final-pictures377-native', 'native_inputs': QREL + '/final-pictures377-native-inputs.json',
         'ledger': QREL + '/final-pictures377-overview-ledger388.json'}


def sha(p):
    with open(p, 'rb') as f:
        return hashlib.file_digest(f, 'sha256').hexdigest()


def utc():
    return datetime.datetime.now(datetime.timezone.utc).isoformat()


def master_receipt(D, kind):
    inputs = D / QREL / 'combined-master350-inputs.json'
    h = sha(inputs)
    base = {'started': utc(), 'phase': 'native', 'command': ['/usr/bin/timeout', '86400', '/usr/bin/python3', '-I', '-B', 'stub-350', 'master'],
            'preflight_only': False, 'samples': [], 'sbh': {'exit': 0}, 'admission': {'free_bytes': 1}, 'policy': {'stop_free_gib': 8},
            'admitted': True, 'inputs_sha256': h, 'pid': 1}
    if kind == 'success':
        base.update(exit=0, final_inputs_sha256=h, finished=utc())
    elif kind == 'exit1':
        base.update(exit=1, final_inputs_sha256=h, finished=utc())
    elif kind == 'stopped':
        base.update(stopped_for_resource_limit=True, exit=-15, final_inputs_sha256=h, finished=utc())
    elif kind == 'refused':
        base = {k: base[k] for k in ('started', 'phase', 'command', 'preflight_only', 'samples', 'sbh', 'admission', 'policy')}
        base.update(admitted=False, finished=utc())
    elif kind == 'running':
        pass
    return json.dumps(base, indent=2) + '\n'


def setup(name, master='success'):
    D = RUNS / name
    D.mkdir(parents=True)
    for sub in ('jobs', 'work', QREL):
        (D / sub).mkdir(parents=True, exist_ok=sub == QREL)
    Q = D / QREL
    shutil.copyfile(FAKE377, Q / 'csd-final-verify377.py')
    shutil.copyfile(FAKEGUARD, D / 'guard_csd02.py')
    (Q / 'cc0-combined-master350.py').write_text('# stub 350 helper (task391)\n')
    if master is not None:
        mj = D / MASTER_REL
        (mj / 'final').mkdir(parents=True)
        (mj / 'final/master.mp4').write_bytes(b'stub master (task391)')
        rows = [dict(path=str(p), sha256=sha(p)) for p in (mj / 'final/master.mp4', Q / 'cc0-combined-master350.py')]
        (Q / 'combined-master350-inputs.json').write_text(json.dumps({'files': rows}, indent=2))
        if master == 'partial':
            (mj / 'resource-receipt.json').write_text(master_receipt(D, 'success')[:60])
        else:
            (mj / 'resource-receipt.json').write_text(master_receipt(D, master))
    return D


def env(fault377='', faultguard='', D=None):
    e = dict(os.environ, FAKE377_D=str(D), FAKEGUARD_RUNNER=str(RUNNER))
    if fault377:
        e['FAKE377_FAULT'] = fault377
    if faultguard:
        e['FAKEGUARD_FAULT'] = faultguard
    return e


def start(D, logname, hours='0.01', poll='0.2', hook='', fault377='', faultguard=''):
    log = open(RUNS / (logname + '.log'), 'x')
    argv = [sys.executable, '-I', '-B', str(RUN388), str(REAL388), str(D), hours, poll] + ([hook] if hook else [])
    return subprocess.Popen(argv, stdout=log, stderr=subprocess.STDOUT, env=env(fault377, faultguard, D), cwd=str(T)), log


def finish(proc, log, logname):
    rc = proc.wait(timeout=180)
    log.close()
    text = (RUNS / (logname + '.log')).read_text(errors='replace')
    lines = [x for x in text.splitlines() if x.strip()]
    return dict(exit=rc, result=next((x for x in reversed(lines) if x.startswith('RESULT ')), None),
                runs=[x[len('388 runs: '):].split()[-1] for x in lines if x.startswith('388 runs: ')],
                json_lines=[sorted(json.loads(x)) for x in lines if x.startswith('{')][:8],
                ledger_line=next((x for x in lines if x.startswith('388 ledger')), None) is not None,
                hook=[x for x in lines if x.startswith('HOOK')], log_lines=len(lines))


def tree(D):
    out = {}
    for parent, dirs, files in os.walk(D):
        dirs.sort()
        for n in sorted(files):
            p = Path(parent) / n
            if n == 'expanded-native-lane.lock':
                continue
            out[str(p.relative_to(D))] = sha(p)
    return out


def state(D):
    s = {k: (D / v).exists() for k, v in PATHS.items()}
    jobs = {}
    for n in sorted(os.listdir(D / 'jobs')):
        j = D / 'jobs' / n
        if not j.is_dir() or n == 'csd-combined-master350':
            continue
        row = {}
        r = j / 'resource-receipt.json'
        if r.is_file():
            rec = json.loads(r.read_text())
            row.update(admitted=rec.get('admitted'), exit=rec.get('exit'), finished=bool(rec.get('finished')),
                       stopped=rec.get('stopped_for_resource_limit', False), error=bool(rec.get('error')),
                       started=rec.get('started'), finished_utc=rec.get('finished'))
        else:
            row['receipt'] = False
        log = j / 'command.log'
        if log.is_file():
            lines = [x for x in log.read_text(errors='replace').splitlines() if x.strip()]
            row['command_log_last'] = lines[-1][:160] if lines else None
            row['command_log_lines'] = len(lines)
        jobs[n] = row
    s['jobs'] = jobs
    led = D / PATHS['ledger']
    if led.is_file():
        rec = json.loads(led.read_text())
        s['ledger_keys'] = sorted(rec)
        s['ledger_value_types'] = sorted({type(v).__name__ for v in rec.values()})
        s['ledger_written_utc'] = rec.get('written_utc')
        s['ledger_sha256'] = sha(led)
        s['ledger_overview_sampling_sha256'] = rec.get('overview_sampling_sha256')
        sp = D / 'jobs/csd-final-pictures377-overview/sampling.json'
        s['ledger_equals_sampling_now'] = sp.is_file() and sha(sp) == rec.get('overview_sampling_sha256')
    ni = D / PATHS['native_inputs']
    if ni.is_file() and led.is_file():
        bound = {x['path']: x['sha256'] for x in json.loads(ni.read_text())['files']}
        s['native_inputs_row_equals_ledger'] = bound.get(str(D / 'jobs/csd-final-pictures377-overview/sampling.json')) == json.loads(led.read_text())['overview_sampling_sha256']
    ns = D / 'jobs/csd-final-pictures377-native/sampling.json'
    if ns.is_file() and led.is_file():
        s['native_union_equals_ledger'] = json.loads(ns.read_text())['union']['overview_sampling_sha256'] == json.loads(led.read_text())['overview_sampling_sha256']
    return s


def later(delay, fn):
    t = threading.Timer(delay, fn)
    t.start()
    return t


results = {'started': utc(), 'real388': sha(REAL388), 'runner': sha(RUNNER), 'fake377': sha(FAKE377),
           'fakeguard': sha(FAKEGUARD), 'run388': sha(RUN388), 'python': sys.version.split()[0], 'scenarios': {}}
assert not RUNS.exists()
RUNS.mkdir()


def scenario(name, master='success', hours='0.01', poll='0.2', hook='', fault377='', faultguard='', before=None, during=None):
    D = setup(name, master)
    if before:
        before(D)
    t0 = tree(D)
    proc, log = start(D, name, hours, poll, hook, fault377, faultguard)
    timer = later(during[0], lambda: during[1](D)) if during else None
    out = finish(proc, log, name)
    if timer:
        timer.join()
    out['state'] = state(D)
    out['tree_unchanged'] = tree(D) == t0
    results['scenarios'][name] = out
    return D, out


# S01 happy path; S02 a second 388 on the finished tree
D1, _ = scenario('S01_happy')
t1 = tree(D1)
proc, log = start(D1, 'S02_rerun_after_happy')
o = finish(proc, log, 'S02_rerun_after_happy')
o['tree_unchanged'] = tree(D1) == t1
results['scenarios']['S02_rerun_after_happy'] = o

# master wait and terminal failures
scenario('S03_master_absent_timeout', master=None, hours='0.0004')
scenario('S04_master_partial_then_success', master='partial',
         during=(1.5, lambda D: (D / MASTER_REL / 'resource-receipt.json').write_text(master_receipt(D, 'success'))))
scenario('S05_master_exit1', master='exit1')
scenario('S06_master_refused_admission', master='refused')
scenario('S07_master_resource_stop', master='stopped')
scenario('S08_master_receipt_rewritten_after_finished', poll='2.5',
         during=(1.0, lambda D: (D / MASTER_REL / 'resource-receipt.json').write_text(master_receipt(D, 'success'))))
scenario('S09_pin_377_changes_during_wait', master='running',
         during=(1.0, lambda D: open(D / QREL / 'csd-final-verify377.py', 'a').write('# changed\n')))
scenario('S09b_master_running_timeout', master='running', hours='0.0004')

# refusals before waiting: each of the seven 377/388 paths already present
for key, rel in PATHS.items():
    def make(D, rel=rel):
        p = D / rel
        if rel.startswith('jobs/'):
            p.mkdir()
        else:
            p.write_text('{}\n')
    scenario('S10_preexisting_' + key, before=make)

# 377 check refuses (a master row changed after the master finished)
scenario('S11_check_refuses', before=lambda D: (D / MASTER_REL / 'final/master.mp4').write_bytes(b'changed'))
# verify job failures
scenario('S12_verify_job_exit1', fault377='verify_fail')
scenario('S13_verify_guard_refused_admission', faultguard='refuse_verify')
D14, _ = scenario('S14_377_free_space_refusal', fault377='need_huge')
t14 = tree(D14)
proc, log = start(D14, 'S14b_rerun_after_free_space_refusal', fault377='need_huge')
o = finish(proc, log, 'S14b_rerun_after_free_space_refusal')
o['tree_unchanged'] = tree(D14) == t14
o['state'] = state(D14)
results['scenarios']['S14b_rerun_after_free_space_refusal'] = o
# overview failures and the ledger gates
scenario('S15_overview_resource_stop', faultguard='stop_overview')
scenario('S16_overview_extra_last_line', fault377='overview_extra_line')
scenario('S17_overview_sampling_changed_after_print', fault377='overview_sampling_changed')
scenario('S18_sampling_changed_after_ledger', hook='mutate_sampling_after_ledger')
scenario('S19_native_job_exit1', fault377='native_fail')
# two controllers started together on one tree
D20 = setup('S20_two_controllers')
pa, la = start(D20, 'S20_two_controllers_A')
pb, lb = start(D20, 'S20_two_controllers_B')
oa, ob = finish(pa, la, 'S20_two_controllers_A'), finish(pb, lb, 'S20_two_controllers_B')
results['scenarios']['S20_two_controllers'] = dict(A=oa, B=ob, state=state(D20))
# argument bounds
for hours in ('49', '-1', 'nan'):
    scenario('S21_wait_hours_' + hours.replace('-', 'minus'), hours=hours)
results['finished'] = utc()
with open(T / 'test391.out.json', 'x') as f:
    json.dump(results, f, indent=1)
for name, o in results['scenarios'].items():
    if 'A' in o:
        print(name, 'A', o['A']['result'], '| B', o['B']['result'])
    else:
        print(name, o['exit'], o['result'], 'runs', o['runs'], 'unchanged' if o.get('tree_unchanged') else 'changed')
