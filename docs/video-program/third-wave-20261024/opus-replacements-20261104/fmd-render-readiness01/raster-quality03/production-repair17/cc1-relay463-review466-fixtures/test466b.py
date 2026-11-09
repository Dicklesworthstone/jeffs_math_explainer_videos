"""task466 trial b (CC1): the cases trial a left out, on the same sim466 world.
F1r  receive binding again, with refusal texts cleaned before truncation (trial a cut paths mid-way).
F3   each of a-h failing on its own, on top of one completed import made by a 463 run in the world
     (verify on the actual 463 with the 448 twin's check and 463's own POST_PROBE).
F4   448 SSD routing on the CSS side (helper copy, manifest copy, 442 receipt bytes, terminal receipt) and
     on the CSD side (RECOVERY alias missing or pointing elsewhere), plus the 371-prefix serial gate.
Fixture state only; no actual import. Usage: python3 -I -B test466b.py Q OUTDIR
"""
import contextlib
import io
import json
import os
import shutil
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import sim466 as S  # noqa: E402

Q0, OUT = Path(sys.argv[1]).resolve(), Path(sys.argv[2]).resolve()
assert not OUT.exists()
OUT.mkdir()
res = {'F1r': {}, 'F3': {}, 'F4': {}}


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


def write_job(w, attempt, rows):
    h = w.h
    job = w.job(attempt)
    inputs = h.inputs_path(job.name)
    S.put(inputs, json.dumps({'files': rows}, indent=2))
    command = [str(c) for c in h.receive_command(w.helper, S.CH, attempt, h.Q/S.MANIFEST)]
    r = S.terminal_receipt(command, S.sha(inputs))
    for k in ('exit', 'final_inputs_sha256', 'finished'):
        del r[k]
    S.put(job/'resource-receipt.json', json.dumps(r, indent=2))
    return job


# F1r: receive binding, cleaned texts
w = S.World(OUT/'F1r', Q0)
assert w.run448('prepare', 1).returncode == 0
exact = json.loads(w.h.inputs_path(w.h.job_name(S.CH, 1)).read_text())['files']
targets = {'shared row': next(x['path'] for x in w.m['files'] if x['kind'] == 'shared'),
           'system row': next(x['path'] for x in w.m['files'] if x['kind'] == 'system'),
           'Q permit copy': str(w.h.q_permit(w.pin)), 'helper': str(w.helper), 'guard': str(w.h.GUARD), 'manifest': str(w.h.Q/S.MANIFEST),
           'none (exact rows)': None}
for n, (name, path) in enumerate(targets.items(), start=2):
    rows = exact if path is None else sorted([x for x in exact if x['path'] != path] + [dict(path=str(w.spare), sha256=S.sha(w.spare))],
                                             key=lambda x: x['path'])
    job = write_job(w, n, rows)
    r = w.run448('receive', n)
    last = [x for x in r.stderr.decode().replace(str(w.S), 'W').splitlines() if x.strip()][-1]
    res['F1r'][name] = dict(rows=len(rows), exit=r.returncode, refusal=last, incoming=(job/'incoming').exists(),
                            import_result=(job/'import-result.json').exists())


# F3: one completed import (463 run, scripted CSD end), then single-condition breaks, each verified
def completed(name):
    w = S.World(OUT/name, Q0)
    relay = S.load463(w, Q0, outcome=('complete', 0))
    code, out, err = call(w, relay.run, S.args(w))
    assert code == 0, (code, err)
    relay.subprocess.faults = {}
    return w, relay


def verify(w, relay, **kw):
    code, out, err = call(w, relay.verify, S.args(w, **kw))
    rep = json.loads(out) if out.strip().startswith('{') else {}
    return dict(code=code, failures=[f[:2] for f in rep.get('failures', [])], stop=err or None,
                check_problems=rep.get('check_problems'), inputs_changed_count=rep.get('inputs_changed_count'))


def placed(w):
    x = next(x for x in w.m['files'] if x['kind'] == 'foreign' and x['path'].endswith('.mp4'))
    return w.h.physical(x['path'])


breaks = {
    'control (no change)': lambda w: None,
    'a+e: a placed SSD file changed after import': lambda w: (os.chmod(placed(w), 0o644), placed(w).write_bytes(b'changed after import\n')),
    'a+e: RECOVERY alias on CSD removed': lambda w: os.rename(w.h.RECOVERY, str(w.h.RECOVERY)+'.moved466'),
    'a+e: RECOVERY alias points at another folder': lambda w: (os.rename(w.h.RECOVERY, str(w.h.RECOVERY)+'.moved466'),
                                                               shutil.copytree(w.h.D/'work/imported-css-ssd442', w.S/'csd/other-ssd466'),
                                                               os.symlink(str(w.S/'csd/other-ssd466'), w.h.RECOVERY),
                                                               (w.S/'csd/other-ssd466/jobs'/S.J442/'render/05_gallery.mp4').write_bytes(b'other\n')),
    'd: import-result names another manifest': lambda w: (lambda p: p.write_text(json.dumps(dict(json.loads(p.read_text()), manifest_sha256='0'*64))))(w.job(1)/'import-result.json'),
    'f: inputs file rewritten after admission (same rows)': lambda w: (lambda p: p.write_text(p.read_text()+'\n'))(w.h.inputs_path(w.job(1).name)),
    'g+e: a shared row changed after import': lambda w: (lambda p: p.write_bytes(b'changed shared row\n'))(Path(next(x['path'] for x in w.m['files'] if x['kind'] == 'shared'))),
    'c: receipt rewritten with exit 1': lambda w: (lambda p: p.write_text(json.dumps(dict(json.loads(p.read_text()), exit=1))))(w.job(1)/'resource-receipt.json'),
    'h: import-result rows350 names attempt 2': lambda w: (lambda p: p.write_text(json.dumps(dict(json.loads(p.read_text()), rows350=w.import_result(2)['rows350']))))(w.job(1)/'import-result.json'),
}
for name, mutate in breaks.items():
    w, relay = completed('F3_%02d' % len(res['F3']))
    mutate(w)
    res['F3'][name] = verify(w, relay)
w, relay = completed('F3_b')
res['F3']['b: verify --attempt 2 (no such job)'] = verify(w, relay, attempt=2)


# F4: routing and gate refusals before anything starts
def refused(name, mutate, world_kw=None):
    w = S.World(OUT/name, Q0, **(world_kw or {}))
    relay = S.load463(w, Q0)
    mutate(w)
    code, out, err = call(w, relay.run, S.args(w))
    res['F4'][name] = dict(code=code, stop=err[:400], transport=[k for _, k in relay.subprocess.log],
                           csd_inputs=sorted(p.name for p in w.h.Q.glob('csd-css352-import448-*-inputs.json')),
                           csd_job=w.job(1).exists())


refused('css_helper_copy_differs', lambda w: (lambda p: (p.unlink(), S.put(p, b'not 448\n')))(w.css_rec/'csd-gallery-import448.py'))
refused('css_manifest_copy_missing', lambda w: os.rename(w.css_rec/S.MANIFEST, str(w.css_rec/S.MANIFEST)+'.moved466'))
refused('css_442_receipt_not_terminal', lambda w: (lambda p: p.write_text(json.dumps(dict(json.loads(p.read_text()), exit=1))))(w.css_rec/'jobs'/S.J442/'resource-receipt.json'))
refused('css_442_job_only_under_css_root', lambda w: (shutil.copytree(w.css_rec/'jobs', w.css_data/'projects/jmev-css-render-worker-20261107/jobs'),
                                                      os.rename(w.css_rec/'jobs', str(w.css_rec/'jobs')+'.moved466')))
refused('csd_helper_copy_differs', lambda w: (lambda p: (p.unlink(), S.put(p, b'not 448\n')))(w.helper))
refused('csd_recovery_alias_missing', lambda w: os.rename(w.h.RECOVERY, str(w.h.RECOVERY)+'.moved466'))


def earlier_14_as_448(w):
    old = w.h.D/'jobs/csd-css352-import371-14-continuation-a1'
    os.rename(old, w.h.D/'jobs/csd-css352-import448-14-continuation-a1')


refused('serial_gate_14_under_448_prefix', earlier_14_as_448)
with open(OUT/'test466b.out.json', 'x') as f:
    json.dump(res, f, indent=1)
print(json.dumps(res, indent=1))
