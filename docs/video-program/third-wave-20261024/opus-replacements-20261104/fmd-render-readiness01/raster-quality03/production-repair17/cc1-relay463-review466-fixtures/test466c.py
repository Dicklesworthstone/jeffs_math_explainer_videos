"""task466 trial c (CC1): does a-h bind the completed job to the pinned receive command?
F5  on one completed import made by a 463 run (scripted CSD end), the guard receipt's recorded command is
    rewritten (another helper path; another attempt), then 463 verify runs. Control: unchanged.
F5r the receive side of the same question: real 448 receive (twin) on a job whose admitted receipt names
    another command, to show where the command binding lives.
Fixture state only; no actual import. Usage: python3 -I -B test466c.py Q OUTDIR
"""
import contextlib
import io
import json
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import sim466 as S  # noqa: E402

Q0, OUT = Path(sys.argv[1]).resolve(), Path(sys.argv[2]).resolve()
assert not OUT.exists()
OUT.mkdir()
res = {'F5': {}, 'F5r': {}}


def call(w, fn, *a):
    out, err = io.StringIO(), io.StringIO()
    with contextlib.redirect_stdout(out), contextlib.redirect_stderr(err):
        try:
            code = fn(*a)
        except Exception as error:
            code = 'raised %s: %s' % (type(error).__name__, error)
    for p in w.children:
        p.wait(timeout=120)
    return code, out.getvalue().replace(str(w.S), 'W'), ' | '.join(x for x in err.getvalue().replace(str(w.S), 'W').splitlines() if x.strip())


def completed(name):
    w = S.World(OUT/name, Q0)
    relay = S.load463(w, Q0, outcome=('complete', 0))
    code, out, err = call(w, relay.run, S.args(w))
    assert code == 0, (code, err)
    relay.subprocess.faults = {}
    return w, relay


def other_helper(command, w):
    return [str(w.h.Q/'some-other-import-helper.py') if c == str(w.helper) else c for c in command]


def other_attempt(command, w):
    i = command.index('--attempt')
    return command[:i+1] + ['7'] + command[i+2:]


edits = {'control (receipt unchanged)': None, 'receipt command names another helper path': other_helper,
         'receipt command names attempt 7': other_attempt}
for n, (name, edit) in enumerate(edits.items()):
    w, relay = completed('F5_%02d' % n)
    p = w.job(1)/'resource-receipt.json'
    r = json.loads(p.read_text())
    before = list(r['command'])
    if edit:
        r['command'] = edit(before, w)
        p.write_text(json.dumps(r, indent=2))
    code, out, err = call(w, relay.verify, S.args(w))
    rep = json.loads(out) if out.strip().startswith('{') else {}
    res['F5'][name] = dict(code=code, passed=len(rep.get('passed', [])), failures=rep.get('failures'), stop=err or None,
                           command_changed=r['command'] != before,
                           command_equals_receive_command=r['command'] == [str(c) for c in w.h.receive_command(w.helper, S.CH, 1, w.h.Q/S.MANIFEST)])

# F5r: 448's own receive refuses a job admitted for another command
w = S.World(OUT/'F5r', Q0)
assert w.run448('prepare', 2).returncode == 0
job = w.job(2)
inputs = w.h.inputs_path(job.name)
for n, (name, edit) in enumerate([('admitted receipt names another helper path', other_helper),
                                  ('admitted receipt names attempt 7', other_attempt),
                                  ('admitted receipt names the exact receive command', None)]):
    if n:
        (job/'resource-receipt.json').rename(job/('resource-receipt.json.trial%d' % (n - 1)))
    command = [str(c) for c in w.h.receive_command(w.helper, S.CH, 2, w.h.Q/S.MANIFEST)]
    r = S.terminal_receipt(edit(command, w) if edit else command, S.sha(inputs))
    for k in ('exit', 'final_inputs_sha256', 'finished'):
        del r[k]
    S.put(job/'resource-receipt.json', json.dumps(r, indent=2))
    rr = w.run448('receive', 2)
    last = [x for x in rr.stderr.decode().replace(str(w.S), 'W').splitlines() if x.strip()]
    res['F5r'][name] = dict(exit=rr.returncode, refusal=last[-1] if last else None, incoming=(job/'incoming').exists(),
                            import_result=(job/'import-result.json').exists())
with open(OUT/'test466c.out.json', 'x') as f:
    json.dump(res, f, indent=1)
print(json.dumps(res, indent=1))
