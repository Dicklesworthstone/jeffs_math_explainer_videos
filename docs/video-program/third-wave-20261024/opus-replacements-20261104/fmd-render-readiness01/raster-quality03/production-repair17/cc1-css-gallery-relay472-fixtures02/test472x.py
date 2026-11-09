"""relay472 (CC1): the targeted refusals, 463 and 472 side by side on the same fixture state.
X1 substituted rows: fabricated completed states (terminal receipt, import-result, foreign files placed) whose
   inputs swap one frozen row (shared, system, Q permit copy, helper, guard, manifest) for another valid file at
   the same count, plus the exact rows; validate() of each relay on each state.
X2 helper hash: one import completed by a 472 run (scripted CSD end); then 448 check's stdout is rewritten to
   report another helper_sha256 (standing for check run from a Q copy that is not the pin); verify of each relay,
   with and without the rewrite.
Command rewrites are trial c's F5, rerun on 472 by run472.py. Fixture only; no actual import.
Usage: python3 -I -B test472x.py Q FIX466 H472 OUTDIR"""
import contextlib
import io
import json
import shlex
import sys
from pathlib import Path

Q0, FIX, H472, OUT = Path(sys.argv[1]).resolve(), Path(sys.argv[2]).resolve(), sys.argv[3], Path(sys.argv[4]).resolve()
assert not OUT.exists()
OUT.mkdir()
sys.path.insert(0, str(FIX))
sys.path.insert(0, str(Path(__file__).resolve().parent))
import sim466 as S  # noqa: E402
import sim472 as L  # noqa: E402

res = {'X1': {}, 'X2': {}}


def letters(names):
    return [x.split('.')[0] for x in names]


def call(w, fn, *a):
    out, err = io.StringIO(), io.StringIO()
    with contextlib.redirect_stdout(out), contextlib.redirect_stderr(err):
        code = fn(*a)
    for p in w.children:
        p.wait(timeout=120)
    return code, out.getvalue(), err.getvalue().replace(str(w.S), 'W')


def write_job(w, attempt, rows):
    h, job = w.h, w.job(attempt)
    inputs = h.inputs_path(job.name)
    S.put(inputs, json.dumps({'files': rows}, indent=2))
    command = [str(c) for c in h.receive_command(w.helper, S.CH, attempt, h.Q/S.MANIFEST)]
    S.put(job/'resource-receipt.json', json.dumps(S.terminal_receipt(command, S.sha(inputs)), indent=2))
    S.put(job/'import-result.json', json.dumps(w.import_result(attempt), indent=1))


# X1
w = S.World(OUT/'X1', Q0)
relays = {'463': S.load463(w, Q0), '472': L.load472(w, Q0, H472)}
assert w.run448('prepare', 1).returncode == 0
exact = json.loads(w.h.inputs_path(w.h.job_name(S.CH, 1)).read_text())['files']
w.place_foreign()
targets = {'exact rows (control)': None,
           'shared row': next(x['path'] for x in w.m['files'] if x['kind'] == 'shared'),
           'system row': next(x['path'] for x in w.m['files'] if x['kind'] == 'system'),
           'Q permit copy': str(w.h.q_permit(w.pin)), 'helper': str(w.helper), 'guard': str(w.h.GUARD), 'manifest': str(w.h.Q/S.MANIFEST)}
for n, (name, path) in enumerate(targets.items(), start=20):
    rows = exact if path is None else sorted([x for x in exact if x['path'] != path] + [dict(path=str(w.spare), sha256=S.sha(w.spare))],
                                             key=lambda x: x['path'])
    assert len(rows) == len(exact)
    write_job(w, n, rows)
    res['X1'][name] = {}
    for label, relay in relays.items():
        rep, out, err = call(w, relay.validate, w.h, S.args(w, attempt=n), w.m, w.pin)
        res['X1'][name][label] = dict(rows=len(rows), passed=len(rep['passed']), failures=letters(rep['failures']))


# X2
def forge(relay):
    shim = relay.subprocess
    real = shim.run

    def run(argv, input=b'', capture_output=True, timeout=None):
        r = real(argv, input=input, capture_output=capture_output, timeout=timeout)
        host, command = shim.split(argv)
        if command != '/usr/bin/python3 -I -B -' and shlex.split(command)[4] == 'check' and r.returncode == 0:
            d = json.loads(r.stdout)
            d['helper_sha256'] = '0'*64
            r = S._sp.CompletedProcess(argv, r.returncode, json.dumps(d).encode(), r.stderr)
        return r
    shim.run = run


w = S.World(OUT/'X2', Q0)
run472 = L.load472(w, Q0, H472, outcome=('complete', 0))
code, out, err = call(w, run472.run, S.args(w))
assert code == 0, (code, err)
for forged in (False, True):
    for label, relay in (('463', S.load463(w, Q0)), ('472', L.load472(w, Q0, H472))):
        if forged:
            forge(relay)
        code, out, err = call(w, relay.verify, S.args(w))
        rep = json.loads(out) if out.strip().startswith('{') else {}
        res['X2'][('check reports helper_sha256 0000... (forged)' if forged else 'check unchanged (control)')+' / '+label] = dict(
            code=code, passed=len(rep.get('passed', [])), failures=letters(rep.get('failures', [])), stop=err.strip() or None)
with open(OUT/'test472x.out.json', 'x') as f:
    json.dump(res, f, indent=1)
print(json.dumps(res, indent=1))
