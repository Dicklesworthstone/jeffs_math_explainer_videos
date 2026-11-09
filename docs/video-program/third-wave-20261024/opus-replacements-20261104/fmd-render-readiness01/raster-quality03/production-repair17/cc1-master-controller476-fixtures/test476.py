"""controller476 (CC1): route-delta checks only, local and read-only. Usage: python3 -I -B test476.py Q OUTDIR

Each case says what it shows. Nothing contacts a host; the /data paths named by the draft do not exist on this
machine, so every receipt is pending. The 08 export is a labelled synthetic fixture (fixture476-08-manifest.json):
it is not a 371 export and proves nothing about the CSS 08 job. Its digests are dummies except permit_sha256.
OUTDIR must be new and outside Q: it receives hard links (not copies) of the four pinned 371 exports beside the
synthetic 08 file, so pinned_exports() can be run on a Q that holds all five.
447's Q is the CSD path, which does not exist here. rows_and_receipts() needs that path (it compares the draft's
/data names); pinned_exports() reads files under it. So pinned_exports() is given a stand-in module whose only
change is Q (this checkout's Q, or OUTDIR/q), and check() is run with pinned_exports() redirected the same way."""
import contextlib
import hashlib
import importlib.util
import io
import json
import os
import subprocess
import sys
from pathlib import Path
from types import SimpleNamespace

Q, OUT = Path(sys.argv[1]).resolve(), Path(sys.argv[2]).resolve()
assert not OUT.exists() and not OUT.is_relative_to(Q)
CH = '08_loop_sums'
PERMIT474_SHA = '92119d3a65fd5b3637f839ca83fb60eb455661d296337a1e1dc02cf777d0e59a'
results = []


def case(name, shows, ok, **detail):
    results.append(dict(case=name, shows=shows, ok=bool(ok), **detail))


def mod(path, name):
    spec = importlib.util.spec_from_file_location(name, path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def refusal(fn, *a):
    try:
        fn(*a)
    except AssertionError as error:
        return str(error)
    return None


c464 = mod(Q/'cc2-master-controller464.py', 'c464')
c = mod(Q/'cc1-master-controller476.py', 'c476')
m = c.load(c.MASTER350, c.MASTER350_SHA, 'master447')
i = c.load(c.IMPORT371, c.IMPORT371_SHA, 'import371')
raw460, raw476 = (Q/'master447-manifest-draft460.json').read_bytes(), c.DRAFT.read_bytes()
case('D0 pins', 'controller 476 pins draft 476 at the sha it was written with; 464 still pins draft 460',
     hashlib.sha256(raw476).hexdigest() == c.DRAFT_SHA and hashlib.sha256(raw460).hexdigest() == c464.DRAFT_SHA,
     draft476=c.DRAFT_SHA, draft460=c464.DRAFT_SHA)

# D1-D3: the draft itself
d460, d476 = json.loads(raw460), json.loads(raw476)
case('D1 serialisation', 'draft 476 is json.dumps(indent=2) + newline, as draft 460 is',
     (json.dumps(d476, indent=2)+'\n').encode() == raw476 and (json.dumps(d460, indent=2)+'\n').encode() == raw460)


def span(row):                       # a row's exact bytes inside "chapters" (list items sit at indent 4)
    return '\n'.join('    '+line for line in json.dumps(row, indent=2).splitlines()).encode()


others = [r for r in d460['chapters'] if r['chapter'] != CH]
same = [r['chapter'] for r in others if raw460.count(span(r)) == 1 and raw476.count(span(r)) == 1]
old08 = next(r for r in d460['chapters'] if r['chapter'] == CH)
case('D2 fourteen rows', 'each non-08 row of draft 460 occurs byte for byte, once, in draft 476; the order of all 15 '
     'is unchanged; the old 08 row is gone', len(same) == 14
     and [r['chapter'] for r in d476['chapters']] == [r['chapter'] for r in d460['chapters']] == list(c.REGISTRY)
     and [r for r in d476['chapters'] if r['chapter'] != CH] == others and raw476.count(span(old08)) == 0,
     byte_identical=len(same))
new08 = next(r for r in d476['chapters'] if r['chapter'] == CH)
job = i.job_name(CH, 1)
want08 = dict(chapter=CH, kind='combined', worker=str(i.WORKER), worker_sha256=i.WORKER_SHA256, job=i.css_job(CH),
              inputs=str(i.css_inputs(CH)), permit=str(i.PERMITS/'08_loop_sums-css-permit474.json'),
              import_receipt=str(i.D/'jobs'/job/'resource-receipt.json'), import_inputs=str(i.inputs_path(job)))
row01 = d476['chapters'][0]
case('D3 row 08', "row 08 is the planned CSS352 row: frozen 371's names for attempt 1, 447's CSS worker and sha, "
     "447's ROW_KEYS | IMPORT_KEYS in row 01's key order, 08 in 447 CHANGED, and root's Q permit copy at 92119d3a",
     new08 == want08 and list(new08) == list(row01) and set(new08) == m.ROW_KEYS | m.IMPORT_KEYS
     and new08['worker'] == str(m.WORKER352_CSS) and m.WORKERS352[m.WORKER352_CSS] == new08['worker_sha256']
     and CH in m.CHANGED and hashlib.sha256((Q/i.q_permit(new08).name).read_bytes()).hexdigest() == PERMIT474_SHA,
     row08=new08)

# R1: rows_and_receipts, 476 against 464
rows464, rec464 = c464.rows_and_receipts(m, i)
rows476, rec476 = c.rows_and_receipts(m, i)
at = [k for k, _, _ in rec476].index(CH)
case('R1 receipts', "476's receipts are 464's with only 08's replaced, in the same place: 08 waits for its 371 a1 "
     'import receipt, then its CSS render receipt (as 01/04/12/14); the CSD 08 job is no longer read',
     [x for x in rec476 if x[0] != CH] == [x for x in rec464 if x[0] != CH]
     and [(r, str(p)) for k, r, p in rec476 if k == CH] ==
     [('import371', str(i.D/'jobs'/job/'resource-receipt.json')),
      ('css render', str(m.CSS/'jobs'/'css-reviewed-final-08-loop-sums352'/'resource-receipt.json'))]
     and [k for k, _, _ in rec464].index(CH) == at and len(rec476) == len(rec464)+1
     and not any('csd-reviewed-final-08' in str(p) for _, _, p in rec476),
     receipts464=len(rec464), receipts476=len(rec476),
     row08_464=[(r, str(p)) for k, r, p in rec464 if k == CH], row08_476=[(r, str(p)) for k, r, p in rec476 if k == CH])

# A1: loop_sums_arguments
G = ('fixture476-gallery-manifest.json', 'a'*64)
L = ('fixture476-08-manifest.json', None)
args = lambda n, s: SimpleNamespace(loop_sums_manifest=n, loop_sums_sha=s)
good = 'b'*64
bad = {'missing sha': args(L[0], None), 'missing name': args(None, good), 'folder in name': args('x/'+L[0], good),
       'leading dot': args('.'+L[0], good), 'not json': args('fixture476-08.txt', good),
       'upper-case sha': args(L[0], 'B'*64), 'short sha': args(L[0], 'b'*63),
       'a pinned 371 name': args(c.EXPORTS371['14_continuation'][0], good),
       'a pinned 371 sha': args(L[0], c.EXPORTS371['01_the_bump'][1]), 'the draft name': args(c.DRAFT.name, good),
       'the launch manifest name': args(c.LAUNCH_MANIFEST, good), "the gallery's name": args(G[0], good),
       "the gallery's sha": args(L[0], G[1])}
refused = {k: refusal(c.loop_sums_arguments, a, True, G) for k, a in bad.items()}
case('A1 arguments', 'loop_sums_arguments refuses each malformed, missing or reused name or sha (gallery given), '
     'returns None for check given neither, refuses run given neither, and accepts a plain new pair',
     all(refused.values()) and c.loop_sums_arguments(args(None, None), False, None) is None
     and refusal(c.loop_sums_arguments, args(None, None), True, G)
     and c.loop_sums_arguments(args(L[0], good), True, G) == (L[0], good), refused=refused)

# P1-P5: pinned_exports


def stand_in(q):
    s = SimpleNamespace(**{k: getattr(m, k) for k in dir(m) if not k.startswith('__')})
    s.Q = q
    return s


mQ = stand_in(c.HERE)
real = c.pinned_exports(mQ, i, rows476, None, None)
case('P1 real exports', "on this checkout's Q, the four pinned 371 exports still pass 476's widened CSS set; 08 "
     'and the gallery are skipped when not given', sorted(real) == sorted(c.EXPORTS371)
     and all(real[k] == (c.HERE/n, s) for k, (n, s) in c.EXPORTS371.items()))
OUT.mkdir()
SQ = OUT/'q'
SQ.mkdir()
for n, _ in c.EXPORTS371.values():
    os.link(Q/n, SQ/n)                 # same inode: no copy; read only
pin08 = {k: new08[k] for k in c.ROW_PIN}
pin08.update(inputs_sha256='1'*64, permit_sha256=PERMIT474_SHA, receipt_sha256='2'*64, summary_sha256='3'*64,
             movie_sha256='4'*64, frames=14959)
row = dict(path=str(m.CSS/'jobs'/new08['job']/'fixture476-row'), sha256='5'*64, bytes=1, mode=0o644, kind='foreign')
fixture = dict(version='371', host='css', root=str(i.CSS), fixture476='Synthetic test file, not a 371 export. Only '
               'the names and permit_sha256 are real; the other digests and the one row are dummies.',
               chapters={CH: pin08}, files=[row], foreign_files=1, foreign_bytes=1, shared_files=0, shared_bytes=0,
               system_files=0, system_bytes=0)
data = (json.dumps(fixture, indent=2)+'\n').encode()
(OUT/L[0]).write_bytes(data)
(SQ/L[0]).write_bytes(data)
f08 = (L[0], hashlib.sha256(data).hexdigest())
m2 = stand_in(SQ)
got = c.pinned_exports(m2, i, rows476, None, f08)
case('P2 08 export', 'with 08 given, its manifest is checked as a 371 export (version, chapter, manifest_problems) '
     "and must bind row 08's job, worker, worker sha, inputs and permit; the four pins pass beside it",
     i.manifest_problems(fixture) == [] and sorted(got) == sorted(set(c.EXPORTS371) | {CH}) and got[CH] == (SQ/L[0], f08[1]),
     fixture_sha256=f08[1])
case('P3 wrong sha', 'a given 08 sha that is not the file\'s is refused',
     'not at its sha' in (refusal(c.pinned_exports, m2, i, rows476, None, (L[0], 'c'*64)) or ''))
other = json.loads(data)
other['chapters'][CH]['permit'] = str(i.PERMITS/'08_loop_sums-csd-permit375.json')
odata = (json.dumps(other, indent=2)+'\n').encode()
(SQ/'fixture476-08-other-permit.json').write_bytes(odata)
case('P4 other permit', "an 08 manifest that 371 itself accepts but that binds another permit than the draft's "
     'permit474 is refused', i.manifest_problems(other) == [] and 'binds another' in
     (refusal(c.pinned_exports, m2, i, rows476, None, ('fixture476-08-other-permit.json', hashlib.sha256(odata).hexdigest())) or ''))
rows_minus = [r for r in rows476 if r['chapter'] != CH]
case('P5 08 row required', 'a draft without a CSS 08 row is refused by the CSS-set assertion',
     'not 01/04/08/12/14' in (refusal(c.pinned_exports, m, i, rows_minus, None, None) or ''))

# M1: manifest_bytes
gal = ('fixture476-gallery-manifest.json', 'd'*64)
exports = dict(got, **{c.GALLERY: (m.Q/gal[0], gal[1])})
body = json.loads(c.manifest_bytes(m, rows476, exports, gal, f08))
launch = body.get('launch476', {})
case('M1 launch manifest', 'the launch manifest holds the 15 draft-476 rows and a launch476 block with the draft-476 '
     'source, six import pins (08 included) and both argument pairs', set(body) == {'chapters', 'launch476'}
     and body['chapters'] == d476['chapters'] and launch['source'] == dict(path=str(m.Q/c.DRAFT.name), sha256=c.DRAFT_SHA)
     and sorted(launch['imports']) == sorted(set(c.EXPORTS371) | {c.GALLERY, CH})
     and launch['imports'][CH] == dict(manifest=str(SQ/L[0]), sha256=f08[1])
     and launch['gallery_arguments'] == {'--gallery-manifest': gal[0], '--gallery-sha': gal[1]}
     and launch['loop_sums_arguments'] == {'--loop-sums-manifest': L[0], '--loop-sums-sha': f08[1]})

# K1-K3: check() in process with the real 447 module; pinned_exports() reads OUTDIR/q (see the docstring).
# Locally every /data receipt is absent, so each is pending.
original = c.pinned_exports
c.pinned_exports = lambda _m, i_, rows, g, l: original(m2, i_, rows, g, l)
runs = {}
for name, ns in (('K1', SimpleNamespace()), ('K2', SimpleNamespace(loop_sums_manifest=L[0], loop_sums_sha=f08[1])),
                 ('K3', SimpleNamespace(loop_sums_manifest='fixture476-not-in-q.json', loop_sums_sha='e'*64))):
    out = io.StringIO()
    with contextlib.redirect_stdout(out):
        code = c.check(ns, m, i)
    (OUT/(name+'.stdout')).write_text(out.getvalue())
    runs[name] = (code, [json.loads(x) for x in out.getvalue().splitlines()])
c.pinned_exports = original


def summary(lines):
    return next(x['summary'] for x in lines if 'summary' in x)


code, lines = runs['K1']
ex = next((x for x in lines if 'pinned_exports' in x), {})
case('K1 check, no root arguments', 'the four pins pass, 08 and the gallery are "not given", every receipt is pending, '
     'nothing launched, exit 3', code == 3 and summary(lines) == dict(receipts=len(rec476), successful=0,
     pending=len(rec476), failed=0, pins='ok') and sorted(ex.get('pinned_exports', {})) == sorted(c.EXPORTS371)
     and ex['loop_sums'].startswith('not given') and all(x.get('launched') is not True for x in lines),
     exit=code, summary=summary(lines))
code, lines = runs['K2']
ex = next((x for x in lines if 'pinned_exports' in x), {})
case('K2 check, 08 pair given', "08's export is pinned beside the four; receipts pending; exit 3",
     code == 3 and summary(lines)['pins'] == 'ok' and ex.get('loop_sums') == 'given'
     and sorted(ex.get('pinned_exports', {})) == sorted(set(c.EXPORTS371) | {CH}), exit=code)
code, lines = runs['K3']
case('K3 check, 08 export absent', 'an 08 pair naming a file not in Q refuses the pins; exit 1',
     code == 1 and summary(lines)['pins'] == 'refused' and any('pinned_exports_refused' in x for x in lines), exit=code)

# U1: run requires the 08 pair (proxy Q = this Q, so the CSD-only location check passes; it stops before any read)
m3 = stand_in(c.HERE)
ns = SimpleNamespace(wait_hours=1.0, gallery_manifest=gal[0], gallery_sha=gal[1], loop_sums_manifest=None, loop_sums_sha=None)
out = io.StringIO()
with contextlib.redirect_stdout(out):
    why = refusal(c.run, ns, m3, i)
case('U1 run needs 08', 'run given the gallery pair but no 08 pair refuses at loop_sums_arguments, before any receipt '
     'is read or anything printed', (why or '').startswith('give both --loop-sums-manifest') and out.getvalue() == '')

ok = all(r['ok'] for r in results)
report = dict(controller476=hashlib.sha256((Q/'cc1-master-controller476.py').read_bytes()).hexdigest(),
              draft476=c.DRAFT_SHA, python=sys.version.split()[0], ok=ok, passed=sum(r['ok'] for r in results),
              cases=len(results), results=results)
print(json.dumps(report, indent=1))
raise SystemExit(0 if ok else 1)
