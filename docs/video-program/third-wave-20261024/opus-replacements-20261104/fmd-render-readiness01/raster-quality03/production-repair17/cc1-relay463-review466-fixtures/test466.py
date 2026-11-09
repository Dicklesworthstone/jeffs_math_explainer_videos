"""task466 (CC1): targeted fixtures for relay 463 (d1822a51) with the 448 twin (b6653757 bytes, roots rewritten).
F1  root's g question: 448 prepare's actual row set; same-count substituted inputs against 448 receive's own
    binding asserts; 463 validate() on fabricated completed states (exact, substituted, short, long); the
    --prepared exact-set check.
F2  463 run()/verify()/status() through interruptions and guards: stream lost after the remote completed or
    failed, validation interrupted, admission poll lost, guard refusal, lane busy, attempt reuse, --prepared,
    serial gate, CSS transport. Each scenario has a fresh world. All fixture state; no actual import.
Usage: python3 -I -B test466.py Q OUTDIR
"""
import contextlib
import io
import json
import os
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import sim466 as S  # noqa: E402

Q0, OUT = Path(sys.argv[1]).resolve(), Path(sys.argv[2]).resolve()
assert not OUT.exists()
OUT.mkdir()
res = {'F1': {}, 'F2': {}}


def tail(text, n=1):
    lines = [x for x in text.splitlines() if x.strip()]
    return lines[-n:] if lines else []


def clean(x, w):
    return json.loads(json.dumps(x).replace(str(w.S), 'W'))


def call(fn, *a):
    out, err = io.StringIO(), io.StringIO()
    with contextlib.redirect_stdout(out), contextlib.redirect_stderr(err):
        try:
            code = fn(*a)
        except Exception as error:                       # 463 re-raises unexpected errors after recording them
            code = 'raised %s: %s' % (type(error).__name__, error)
    return code, out.getvalue(), err.getvalue()


def write_job(w, attempt, rows, receipt='admitted', result=False):
    h = w.h
    job = w.job(attempt)
    inputs = h.inputs_path(job.name)
    S.put(inputs, json.dumps({'files': rows}, indent=2))
    command = [str(c) for c in h.receive_command(w.helper, S.CH, attempt, h.Q/S.MANIFEST)]
    if receipt is not None:
        r = S.terminal_receipt(command, S.sha(inputs))
        if receipt == 'admitted':
            for k in ('exit', 'final_inputs_sha256', 'finished'):
                del r[k]
        S.put(job/'resource-receipt.json', json.dumps(r, indent=2))
    if result:
        S.put(job/'import-result.json', json.dumps(w.import_result(attempt), indent=1))
    return job, inputs


def substitute(rows, path, spare):
    out = [dict(x) for x in rows if x['path'] != path] + [dict(path=str(spare), sha256=S.sha(spare))]
    assert len(out) == len(rows)
    return sorted(out, key=lambda x: x['path'])


# ------------------------------------------------------------------ F1
w = S.World(OUT/'F1', Q0)
relay = S.load463(w, Q0)
h = w.h
r = w.run448('prepare', 1)
exact = json.loads(h.inputs_path(h.job_name(S.CH, 1)).read_text())['files']
prepared = json.loads(r.stdout) if r.returncode == 0 else None
res['F1']['prepare'] = dict(exit=r.returncode, stderr=tail(r.stderr.decode(), 3), rows=len(exact),
                            relay_expected_inputs=relay.expected_inputs(h, w.m, w.pin, S.args(w)),
                            path_set_equals_463_expression=({x['path'] for x in exact} == w.expected_paths()),
                            helper_row_sha=[x['sha256'] for x in exact if x['path'] == str(w.helper)] == [S.H448],
                            guard_command_matches_463=prepared is not None and prepared['guard_command'] ==
                            [str(c) for c in h.guard_command(h.Q/'csd-gallery-import448.py', S.CH, 1, h.Q/S.MANIFEST)])
shared0 = next(x['path'] for x in w.m['files'] if x['kind'] == 'shared')
system0 = next(x['path'] for x in w.m['files'] if x['kind'] == 'system')
subs = {'shared row': shared0, 'system row': system0, "Q permit copy": str(h.q_permit(w.pin)), 'helper': str(w.helper),
        'guard': str(h.GUARD), 'manifest': str(h.Q/S.MANIFEST)}
receive = {}
for n, (name, path) in enumerate(list(subs.items()) + [('none (exact rows)', None)], start=2):
    rows = exact if path is None else substitute(exact, path, w.spare)
    job, inputs = write_job(w, n, rows)
    rr = w.run448('receive', n, stdin=b'')
    receive[name] = dict(attempt=n, rows=len(rows), exit=rr.returncode, refusal=tail(rr.stderr.decode())[0][:160] if rr.returncode else None,
                         byte_read_or_staged=(job/'incoming').exists(), import_result=(job/'import-result.json').exists())
res['F1']['receive_binding'] = receive
w.place_foreign()
validate = {}
cases = {'exact rows': exact, 'substituted shared row (same count)': substitute(exact, shared0, w.spare),
         'substituted Q permit copy (same count)': substitute(exact, str(h.q_permit(w.pin)), w.spare),
         'one row dropped': exact[1:], 'one extra row': sorted(exact + [dict(path=str(w.spare), sha256=S.sha(w.spare))], key=lambda x: x['path'])}
for n, (name, rows) in enumerate(cases.items(), start=20):
    write_job(w, n, rows, receipt='terminal', result=True)
    code, out, err = call(relay.validate, h, S.args(w, attempt=n), w.m, w.pin)
    rep = code if isinstance(code, dict) else {}
    validate[name] = dict(attempt=n, rows=len(rows), failures=rep.get('failures', code), passed=len(rep.get('passed', [])))
res['F1']['validate_fabricated_completion'] = validate
prep = {}
for n, (name, rows) in enumerate({'exact rows': exact, 'substituted shared row (same count)': substitute(exact, shared0, w.spare)}.items(), start=40):
    write_job(w, n, rows, receipt=None)
    try:
        prep[name] = dict(outcome='accepted', **{k: v for k, v in relay.check_prepared(h, w.m, w.pin, S.args(w, attempt=n, prepared=True)).items()
                                                 if k == 'guard_sha256'})
    except relay.Stop as stop:
        prep[name] = dict(outcome='refused', code=stop.code, message=str(stop))
res['F1']['check_prepared'] = prep
res['F1'] = clean(res['F1'], w)


# ------------------------------------------------------------------ F2
def scenario(name, outcome=('complete', 0), faults=None, before=None, world_kw=None, run_kw=None, after=()):
    w = S.World(OUT/name, Q0, **(world_kw or {}))
    relay = S.load463(w, Q0, faults, outcome)
    if before:
        before(w, relay)
    pre_inputs = sorted(p.name for p in w.h.Q.glob('csd-css352-import448-*-inputs.json'))
    code, out, err = call(relay.run, S.args(w, **(run_kw or {})))
    for p in w.children:
        p.wait(timeout=120)
    folders = sorted(w.ctl.glob('relay463-05-gallery-a*-run*'))
    record = json.loads((folders[-1]/'record.json').read_text()) if folders else None
    job = w.job((run_kw or {}).get('attempt', 1))
    receipt = json.loads((job/'resource-receipt.json').read_text()) if (job/'resource-receipt.json').exists() else None
    o = dict(run=code, stop=tail(err)[0][:300] if tail(err) else None, record_status=(record or {}).get('status'),
             steps=[s['step'] for s in (record or {}).get('steps', [])], transport=[k for _, k in relay.subprocess.log],
             send_started=['css', 'send'] in relay.subprocess.log, inputs_before=pre_inputs,
             inputs_after=sorted(p.name for p in w.h.Q.glob('csd-css352-import448-*-inputs.json')),
             job_exists=job.exists(), receipt=None if receipt is None else {k: receipt.get(k) for k in ('admitted', 'exit', 'finished')},
             import_result=(job/'import-result.json').exists())
    if o['receipt'] and o['receipt']['finished']:
        o['receipt']['finished'] = True
    for label, kw, fn in after:
        relay.subprocess.faults = {}
        before_jobs = sorted(p.name for p in (w.h.D/'jobs').iterdir())
        before_q = sorted(p.name for p in w.h.Q.iterdir())
        c, out2, err2 = call(getattr(relay, fn), S.args(w, **kw))
        entry = dict(code=c, stop=tail(err2)[0][:300] if tail(err2) else None,
                     csd_jobs_unchanged=sorted(p.name for p in (w.h.D/'jobs').iterdir()) == before_jobs,
                     csd_q_unchanged=sorted(p.name for p in w.h.Q.iterdir()) == before_q)
        if fn == 'verify' and out2.strip():
            rep = json.loads(out2)
            entry.update(failures=rep['failures'], passed=len(rep['passed']))
        if fn == 'status' and out2.strip():
            entry.update(problems=json.loads(out2)['problems'])
        o[label] = entry
    res['F2'][name] = clean(o, w)


VERIFY = ('verify', {}, 'verify')
RERUN = ('rerun same attempt', {}, 'run')
scenario('S01_happy')
scenario('S02_stream_lost_after_remote_completed', outcome=('complete', 255),
         after=[VERIFY, RERUN, ('run attempt 2', {'attempt': 2}, 'run'), ('status', {}, 'status')])
scenario('S03_stream_lost_remote_failed', outcome=('fail', 255),
         after=[VERIFY, RERUN, ('rerun --prepared', {'prepared': True}, 'run')])
scenario('S04_validation_post_probe_timeout', faults={'POST_PROBE': ['timeout']}, after=[VERIFY])
scenario('S05_validation_check_255', faults={'helper check': [None, '255']}, after=[VERIFY])
scenario('S06_admission_poll_timeout', outcome=('fail', 1), faults={'JOB_PROBE': ['timeout']}, after=[VERIFY, RERUN])
scenario('S07_guard_refused_admission', outcome=('refused', 75), after=[VERIFY, RERUN])
LOCK_HOLDER = None


def busy(w, relay):
    st = os.stat(w.h.D/'expanded-native-lane.lock')
    w.locks.write_text('1: FLOCK  ADVISORY  WRITE 31337 %02x:%02x:%d 0 EOF\n' % (os.major(st.st_dev), os.minor(st.st_dev), st.st_ino))


scenario('S08_lane_busy', before=busy)


def stopped_prepare(w, relay):
    assert w.run448('prepare', 1).returncode == 0


def substituted_prepare(w, relay):
    shared0 = next(x['path'] for x in w.m['files'] if x['kind'] == 'shared')
    good = [dict(path=p, sha256=S.sha(p)) for p in sorted(w.expected_paths())]
    write_job(w, 1, substitute(good, shared0, w.spare), receipt=None)
    assert not w.job(1).exists()


scenario('S09_stopped_prepare_without_flag', before=stopped_prepare)
scenario('S10_stopped_prepare_with_flag', before=stopped_prepare, run_kw={'prepared': True})
scenario('S11_substituted_inputs_with_flag', before=substituted_prepare, run_kw={'prepared': True})
scenario('S12_serial_gate_14_missing', world_kw={'earlier': S.EARLIER[:3]})
scenario('S13_css_probe_255', faults={'CSS_PROBE': ['255']})
with open(OUT/'test466.out.json', 'x') as f:
    json.dump(res, f, indent=1)
print(json.dumps(res, indent=1))
