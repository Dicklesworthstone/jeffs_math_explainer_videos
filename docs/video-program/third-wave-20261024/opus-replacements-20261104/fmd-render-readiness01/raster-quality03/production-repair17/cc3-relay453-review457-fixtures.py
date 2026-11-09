"""CC3 review 457 fixtures: css-gallery-relay453.py against import448 and relay394. Fake transports only.

No ssh, no network, no remote file, no transfer. Root's files in Q are read only. A synthetic version-448
05_gallery manifest (six rows) and every record are written under one new temp folder.
Two relays run the same expectations:
  A  root's css-gallery-relay453.py as it stands (c8f5539d);
  B  a temp-only candidate: root's five 453 hunks applied to relay394 371f5d1f. Each hunk's old text must occur
     once in 394, and its new text once in 453, so B differs from A only by what 394 gained after 93f677ff.
Expectations marked FINDING pass when the finding reproduces (they describe A, not a wish).
argv: Q temp-parent. Exit 0 when every expectation holds.
"""
import ast, contextlib, hashlib, importlib.util, io, json, shlex, sys, tempfile, types
from pathlib import Path

Q = Path(sys.argv[1]).resolve()
TMP = Path(tempfile.mkdtemp(prefix='relay453-review457-', dir=sys.argv[2]))
PINNED = {'css-gallery-relay453.py': 'c8f5539d2290416543e4a9bee50d7ef5bc2af82b61cf687ea73f30198881ec0d',
          'csd-gallery-import448.py': 'b6653757c5c114029a94832a0dd4cb9597c42808614fbb959ef6448a5d994f6e',
          'cc3-css352-import-relay394.py': '371f5d1ff415cc1309beebc7feba0f59bd4ba9b139153453d94326be16e64c8c'}
results = []


def ok(name, cond, note=''):
    results.append((name, bool(cond), note))


def sha_file(p):
    return hashlib.sha256(Path(p).read_bytes()).hexdigest()


for name, digest in PINNED.items():
    assert sha_file(Q/name) == digest, 'not the reviewed bytes: '+name

HUNKS = (  # (old text in 394, new text in 453)
    ("VERSION = '394'", "VERSION = '453'"),
    ("HELPER = 'csd-css352-import371.py'\nHELPER_SHA256 = 'ba337aca143316748561c4300d5774386387690b49052e72d29a04b103dd244f'\n"
     "ORDER = ('01_the_bump', '04_reading_color', '12_residues', '14_continuation', '05_gallery')\n",
     "HELPER = 'csd-gallery-import448.py'\nHELPER_SHA256 = 'b6653757c5c114029a94832a0dd4cb9597c42808614fbb959ef6448a5d994f6e'\n"
     "ORDER = ('05_gallery',)\nEARLIER371 = ('01_the_bump', '04_reading_color', '12_residues', '14_continuation')\n"),
    ("    for j in sorted(jobs.glob(A['prefix']+d+'-a*')):\n        tail = j.name[len(A['prefix']+d+'-a'):]\n",
     "    prefix = A.get('prefixes', {}).get(d, A['prefix'])\n    for j in sorted(jobs.glob(prefix+d+'-a*')):\n        tail = j.name[len(prefix+d+'-a'):]\n"),
    ("dict(helper=str(h.CQ/HELPER), manifest=str(h.CQ/args.manifest),\n                receipt=str(h.CSS/'jobs'/pin['job']/'resource-receipt.json'), "
     "summary=str(h.CSS/'jobs'/pin['job']/'render-summary.json')))",
     "dict(helper=str(h.RECOVERY/HELPER), manifest=str(h.RECOVERY/args.manifest),\n                receipt=str(h.RECOVERY/'jobs'/pin['job']/'resource-receipt.json'), "
     "summary=str(h.RECOVERY/'jobs'/pin['job']/'render-summary.json')))"),
    ("    earlier = ORDER[:ORDER.index(args.chapter)]\n", "    earlier = EARLIER371\n"),
    ("chapters=[h.dashed(c) for c in earlier+(args.chapter,)], lock=str(h.D/LANE_LOCK), running=True))",
     "chapters=[h.dashed(c) for c in earlier+(args.chapter,)], prefixes={h.dashed(c): 'csd-css352-import371-' for c in earlier}, lock=str(h.D/LANE_LOCK), running=True))"),
    ("str(h.CQ/HELPER), 'send', '--chapter', args.chapter,\n                   '--manifest', str(h.CQ/args.manifest), '--manifest-sha'",
     "str(h.RECOVERY/HELPER), 'send', '--chapter', args.chapter,\n                   '--manifest', str(h.RECOVERY/args.manifest), '--manifest-sha'"))
text394, text453 = (Q/'cc3-css352-import-relay394.py').read_text(), (Q/'css-gallery-relay453.py').read_text()
candidate = text394
for old, new in HUNKS:
    assert text394.count(old) == 1 and text453.count(new) == 1, (old[:60], text394.count(old), text453.count(new))
    candidate = candidate.replace(old, new)
(TMP/'candidate-B-relay453-on-394.py').open('x').write(candidate)


def load(path, name):
    spec = importlib.util.spec_from_file_location(name, path)
    r = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(r)
    r.time = types.SimpleNamespace(monotonic=__import__('time').monotonic, sleep=lambda s: None)
    return r


A = load(Q/'css-gallery-relay453.py', 'relay453')
H = A.load_helper()                                         # 448 at its pinned sha, from Q
B = load(TMP/'candidate-B-relay453-on-394.py', 'relay453b')
for R in (A, B):
    R.load_helper = lambda: H
    R.HERE = TMP                                            # the synthetic manifest lives in TMP, never in Q

# A synthetic 05_gallery manifest that 448's own manifest_problems() accepts.
CH, JOB442 = '05_gallery', H.css_job('05_gallery')
PIN = dict(job=JOB442, worker=str(H.WORKER), worker_sha256=H.WORKER_SHA256, inputs=str(H.css_inputs(CH)), inputs_sha256='1'*64,
           permit=str(H.PERMITS/'05_gallery-css-permit375.json'), permit_sha256='2'*64, receipt_sha256='3'*64, summary_sha256='4'*64,
           movie_sha256='5'*64, frames=9879)
ROWS = [H.RECOVERY/'jobs'/JOB442/'resource-receipt.json', H.RECOVERY/'jobs'/JOB442/'render-summary.json', H.css_inputs(CH),
        H.CSS/'frozen-fixture.bin', H.D/'work/bin/fixture-shared', Path('/usr/lib/fixture-system.so')]
M = H.manifest_from_rows({CH: PIN}, [dict(path=str(p), sha256='%064x' % (i+7), bytes=10+i, mode=0o644) for i, p in enumerate(ROWS)])
assert H.manifest_problems(M) == [] and H.chapter_of(M) == CH, H.manifest_problems(M)
MNAME = 'css352-import448-05-gallery-manifest-fixture457.json'
data = json.dumps(M, indent=1).encode()
(TMP/MNAME).open('xb').write(data)
MSHA = hashlib.sha256(data).hexdigest()
ok('synthetic manifest: 448 manifest_problems() empty; foreign rows are RECOVERY and CSS paths',
   [x['kind'] for x in M['files']].count('foreign') == 4 and M['shared_files'] == 1 and M['system_files'] == 1)

JOB = H.job_name(CH, 1)
EXPECTED = {CH: dict(import_receipt=str(H.D/'jobs'/JOB/'resource-receipt.json'), import_inputs=str(H.inputs_path(JOB)))}
GUARD = H.guard_command(H.Q/'csd-gallery-import448.py', CH, 1, H.Q/MNAME)
RECEIVE = [str(c) for c in H.receive_command(H.Q/'csd-gallery-import448.py', CH, 1, H.Q/MNAME)]
EARLY = ('01-the-bump', '04-reading-color', '12-residues', '14-continuation')
DONE = dict(job='csd-css352-import371-x-a1', attempt=1, finished=True, imported=True)
CHECK_OK = dict(problems=[], job=JOB, job_exists=False)
PREPARE_OK = dict(job=JOB, inputs=str(H.inputs_path(JOB)), manifest_sha256=MSHA, guard_command=GUARD, rows350=EXPECTED, chapter_pin=PIN)
ADMITTED = dict(exists=True, receipt=True, admitted=True, command=RECEIVE, pid=7, finished=None, exit=None)
POST_OK = dict(problems=[], rows350=EXPECTED, job=JOB, job_exists=True, terminal_success=True, import_result=True)
ROWS_OK = dict(terminal=True, inputs_sha256='i'*64, receipt_inputs_sha256='i'*64, rows=6, distinct=6, changed=[], changed_count=0,
               result_manifest_sha256=MSHA, result_rows350=EXPECTED)


class World:
    def __init__(self, R, css=None, csd=None, lanes=None, check=None, prepare=None, post=None, rows=None, admission=None, exits=(0, 0)):
        self.R = R
        self.css = dict(helper=R.HELPER_SHA256, manifest=MSHA, receipt=PIN['receipt_sha256'], summary=PIN['summary_sha256'], terminal=True)
        if isinstance(css, Exception):
            self.css = css
        else:
            self.css.update(css or {})
        self.csd = dict(helper=R.HELPER_SHA256, manifest=MSHA, imports=dict({d: [DONE] for d in EARLY}, **{'05-gallery': []}), inputs=[],
                        lane=dict(holders=[], waiters=[]), running=[])
        self.csd.update(csd or {})
        self.lanes, self.admission = list(lanes or []), list(admission or [])
        self.check, self.prepare, self.post, self.rows, self.exits = check, prepare, post, ROWS_OK if rows is None else rows, exits
        self.spawned, self.checks, self.calls, self.post_probes = [], 0, [], 0
        R.probe, R.remote, R.spawn = self.probe, self.remote, self.spawn

    def probe(self, host, script, a, timeout=None):
        R = self.R
        self.calls.append((host, script, a))
        if script is R.CSS_PROBE:
            if isinstance(self.css, Exception):
                raise self.css
            return self.css
        if script is R.CSD_PROBE:
            d = dict(self.csd)
            if not a['running'] and self.lanes:
                d['lane'] = self.lanes.pop(0)
            return d
        if script is R.JOB_PROBE:
            if isinstance(self.admission, Exception):
                raise self.admission
            return self.admission.pop(0) if self.admission else dict(exists=False, receipt=False, admitted=False, command=[], pid=None, finished=None, exit=None)
        if script is getattr(R, 'POST_PROBE', None):
            self.post_probes += 1
            return self.rows
        if script is R.INPUTS_PROBE:
            return self.inputs_rows
        raise AssertionError(script[:40])

    def remote(self, host, command, stdin=b'', timeout=0):
        if ' check ' in command:
            self.checks += 1
            shown = self.check if self.checks == 1 else self.post
        else:
            shown = self.prepare
        if isinstance(shown, Exception):
            raise shown
        return types.SimpleNamespace(returncode=0 if shown and not shown.get('problems') else 1, stdout=json.dumps(shown).encode(), stderr=b'')

    def spawn(self, host, command, **kw):
        world = self

        class P:
            def __init__(self):
                self.closed, self.returncode, self.host = False, None, host
                self.stdin = types.SimpleNamespace(close=self.close)

            def close(self):
                self.closed = True

            def poll(self):
                return self.returncode

            def wait(self, timeout=None):
                self.returncode = world.exits[0 if self.host == world.R.HOST_CSS else 1]
                return self.returncode
        p = P()
        self.spawned.append((host, command, p))
        return p


def args(**kw):
    a = dict(mode='run', chapter=CH, attempt=1, manifest=MNAME, manifest_sha=MSHA, prepared=False, lane_wait_minutes=0, record_dir=TMP)
    a.update(kw)
    return types.SimpleNamespace(**a)


def run(R, a=None, raises=False):
    with contextlib.redirect_stdout(io.StringIO()), contextlib.redirect_stderr(io.StringIO()):
        try:
            return R.run(a or args())
        except Exception as error:
            if raises:
                return error
            raise


def last(R):
    runs = sorted(TMP.glob('relay453-05-gallery-a1-run*'), key=lambda p: int(p.name.rsplit('run', 1)[1]))
    return json.loads((runs[-1]/'record.json').read_text())


for R, tag in ((A, 'A'), (B, 'B')):
    # 1. routing: SSD sender, original receiving guard and lane, 448 names
    w = World(R, check=CHECK_OK, prepare=PREPARE_OK, post=POST_OK, admission=[dict(ADMITTED, admitted=False, pid=None), ADMITTED])
    code = run(R)
    css_args = [a for h, s, a in w.calls if s is R.CSS_PROBE][0]
    csd_args = [a for h, s, a in w.calls if s is R.CSD_PROBE and a['running']][0]
    lane_args = [a for h, s, a in w.calls if s is R.CSD_PROBE and not a['running']]
    ok(tag+' success path exits 0', code == 0, code)
    ok(tag+' CSS probe reads helper, manifest, receipt and summary under RECOVERY',
       css_args == dict(helper=str(H.RECOVERY/'csd-gallery-import448.py'), manifest=str(H.RECOVERY/MNAME),
                        receipt=str(H.RECOVERY/'jobs'/JOB442/'resource-receipt.json'), summary=str(H.RECOVERY/'jobs'/JOB442/'render-summary.json')), css_args)
    ok(tag+' CSD probe: 448 prefix for gallery, 371 prefix for the four earlier chapters, the native lane lock',
       csd_args['prefix'] == 'csd-css352-import448-' and csd_args['prefixes'] == {d: 'csd-css352-import371-' for d in EARLY}
       and csd_args['chapters'] == list(EARLY)+['05-gallery'] and csd_args['lock'] == str(H.D/'expanded-native-lane.lock'), csd_args)
    ok(tag+' lane is polled before 448 check and again before the receive', len(lane_args) == 2)
    ok(tag+' receive first, send second, once each', [h for h, _, _ in w.spawned] == [R.HOST_CSD, R.HOST_CSS])
    ok(tag+' receive = 448 guard_command under guard_csd02 (original receiving guard)',
       w.spawned[0][1] == shlex.join(GUARD) and GUARD[3] == str(H.D/'guard_csd02.py') and GUARD[GUARD.index('--')+6] == str(H.Q/'csd-gallery-import448.py'))
    ok(tag+' send = RECOVERY 448 helper with the RECOVERY manifest copy and its sha',
       w.spawned[1][1] == shlex.join(['/usr/bin/python3', '-I', '-B', str(H.RECOVERY/'csd-gallery-import448.py'), 'send', '--chapter', CH,
                                      '--manifest', str(H.RECOVERY/MNAME), '--manifest-sha', MSHA]))
    ok(tag+' rows350 names = 449 expectation (import448 a1)', last(R)['status']['rows350'] == EXPECTED
       and JOB == 'csd-css352-import448-05-gallery-a1', last(R)['status'])

    # 2. refusals before anything starts
    for name, kw, needle in (('RECOVERY helper copy differs', dict(css=dict(helper='0'*64)), 'CSS helper copy'),
                             ('RECOVERY manifest copy differs', dict(css=dict(manifest='0'*64)), 'CSS manifest copy'),
                             ('442 receipt not terminal', dict(css=dict(terminal=False)), 'not a terminal success'),
                             ('stale gallery export', dict(css=dict(summary='0'*64)), 'not the export of the finished job'),
                             ('CSD 448 copy differs', dict(csd=dict(helper='0'*64)), 'CSD helper copy'),
                             ('gallery attempt 1 inputs exist', dict(csd=dict(inputs=[1])), 'never re-admits'),
                             ('gallery already imported', dict(csd=dict(imports=dict({d: [DONE] for d in EARLY}, **{'05-gallery': [dict(DONE, attempt=1)]}))), 'already imported')):
        w = World(R, check=CHECK_OK, prepare=PREPARE_OK, **kw)
        code = run(R)
        ok('%s refused: %s' % (tag, name), code == 1 and not w.spawned and w.checks == 0 and needle in last(R)['status']['message'], last(R)['status']['message'][:80])
    for d in EARLY:
        imports = dict({e: [DONE] for e in EARLY}, **{'05-gallery': []})
        imports[d] = [dict(DONE, imported=False)]
        w = World(R, csd=dict(imports=imports))
        code = run(R)
        ok('%s refused: %s has no completed 371 import' % (tag, d), code == 1 and not w.spawned and 'serial order: '+d.replace('-', '_') in last(R)['status']['message'])
    with contextlib.redirect_stderr(io.StringIO()):
        try:
            sys.argv = ['relay', 'run', '--chapter', '01_the_bump', '--attempt', '1', '--manifest', MNAME, '--manifest-sha', MSHA]
            R.main()
            ok(tag+' argparse refuses a 371 chapter', False)
        except SystemExit as e:
            ok(tag+' argparse refuses a 371 chapter (only gallery uses 448)', e.code == 2)

    # 3. lane, transport, admission
    w = World(R, check=CHECK_OK, prepare=PREPARE_OK, lanes=[dict(holders=[11], waiters=[])])
    ok(tag+' busy lane: stop 4 before 448 check, nothing spawned', run(R) == 4 and w.checks == 0 and not w.spawned)
    w = World(R, css=R.Stop(2, 'css: ssh exit 255'))
    ok(tag+' second render host unreachable at the probe: stop 2, nothing started', run(R) == 2 and not w.spawned and w.checks == 0)
    w = World(R, check=R.Stop(2, 'csd: no answer'))
    ok(tag+' build host silent at 448 check: stop 2, nothing spawned', run(R) == 2 and not w.spawned)
    w = World(R, check=CHECK_OK, prepare=PREPARE_OK)
    w.admission = R.Stop(2, 'csd: ssh exit 255')
    code = run(R)
    ok(tag+' transport stop at the admission poll: stdin closed, no send, attempt left to root',
       code == 2 and len(w.spawned) == 1 and w.spawned[0][2].closed and 'admission unknown' in last(R)['status']['message'], code)
    w = World(R, check=CHECK_OK, prepare=PREPARE_OK, admission=[dict(ADMITTED, admitted=False, finished='t', pid=None)])
    ok(tag+' guard refused admission: stop 3, no send', run(R) == 3 and len(w.spawned) == 1 and w.spawned[0][2].closed)
    w = World(R, check=CHECK_OK, prepare=PREPARE_OK, post=POST_OK, admission=[ADMITTED], exits=(255, 1))
    ok(tag+' ssh 255 in the stream: stop 2', run(R) == 2)

    # 4. the CSD probe itself on a synthetic jobs tree: which jobs count
    root = TMP/('csd-'+tag)
    (root/'jobs').mkdir(parents=True)
    (root/'q').mkdir()
    lock = root/'lane.lock'
    lock.open('xb').close()
    for name, terminal_ok in (('csd-css352-import371-01-the-bump-a1', True), ('csd-css352-import448-04-reading-color-a1', True),
                              ('csd-css352-import371-05-gallery-a1', True), ('csd-css352-import448-05-gallery-a2', False)):
        j = root/'jobs'/name
        j.mkdir()
        (j/'resource-receipt.json').open('x').write(json.dumps(dict(finished='t', exit=0 if terminal_ok else 1, inputs_sha256='k', final_inputs_sha256='k')))
        (j/'import-result.json').open('x').write('{}')
    (root/'q'/'csd-css352-import448-05-gallery-a3-inputs.json').open('x').write('{}')
    script = 'ARGS = %r\n' % json.dumps(dict(helper=str(lock), manifest=str(lock), jobs=str(root/'jobs'), q=str(root/'q'), prefix='csd-css352-import448-',
                                             chapters=list(EARLY)+['05-gallery'], prefixes={d: 'csd-css352-import371-' for d in EARLY},
                                             lock=str(lock), running=False)) + R.CSD_PROBE.replace("open('/proc/locks')", "iter(())")
    out = io.StringIO()
    with contextlib.redirect_stdout(out):
        exec(compile(script, 'csdprobe', 'exec'), {})
    seen = json.loads(out.getvalue())
    ok(tag+' CSD probe: a 371 01 import counts; a 448-named 04 job does not; gallery sees only 448 names',
       [j['imported'] for j in seen['imports']['01-the-bump']] == [True] and seen['imports']['04-reading-color'] == []
       and [(j['attempt'], j['imported']) for j in seen['imports']['05-gallery']] == [(2, False)] and seen['inputs'] == [3], seen)

    # 5. stale text a stopping root reads
    w = World(R, css=dict(helper='0'*64))
    run(R)
    ok('FINDING L1 %s: helper-mismatch stop still names ba337aca (371) while the pin is b6653757 (448)' % tag,
       'not ba337aca' in last(R)['status']['message'] and R.HELPER_SHA256.startswith('b6653757'))
    bad = dict(M, version='371')
    (TMP/('v371-'+tag+'.json')).open('x').write(json.dumps(bad))
    vsha = sha_file(TMP/('v371-'+tag+'.json'))
    w = World(R)
    run(R, args(manifest='v371-'+tag+'.json', manifest_sha=vsha))
    ok('FINDING L1 %s: a 371 manifest is refused cleanly, worded "not a version-371 manifest" (448 text)' % tag,
       last(R)['status']['code'] == 1 and 'not a version-371 manifest' in last(R)['status']['message'], last(R)['status'])

# 6. post-import validation: A keeps the old implicit check, B has a-h
for name, rows in (('guard inputs file changed after admission (f)', dict(ROWS_OK, inputs_sha256='j'*64)),
                   ("a frozen non-manifest row changed, e.g. Q's permit copy or the guard (g)", dict(ROWS_OK, changed=['q-permit'], changed_count=1)),
                   ('inputs hold fewer rows than 448 prepare writes (g)', dict(ROWS_OK, rows=5, distinct=5))):
    wa = World(A, check=CHECK_OK, prepare=PREPARE_OK, post=POST_OK, rows=rows, admission=[ADMITTED])
    ca = run(A)
    wb = World(B, check=CHECK_OK, prepare=PREPARE_OK, post=POST_OK, rows=rows, admission=[ADMITTED])
    cb = run(B)
    ok('FINDING M1: A exits 0 "imported" and never reads the guard inputs when %s; B stops 3 naming it' % name,
       ca == 0 and wa.post_probes == 0 and cb == 3 and 'post-import validation failed' in last(B)['status']['message'], (ca, cb))
codes = []
for R in (A, B):
    World(R, check=CHECK_OK, prepare=PREPARE_OK, post=dict(POST_OK, problems=['not at its hash on CSD: x']), admission=[ADMITTED])
    codes.append(run(R))
ok('A and B both stop 3 when 448 check reports a row off its hash (e)', codes == [3, 3], codes)
ok('FINDING M1: A has no read-only verify mode; B does', not hasattr(A, 'verify') and hasattr(B, 'verify') and hasattr(B, 'POST_PROBE'))
w = World(B, check=POST_OK)
with contextlib.redirect_stdout(io.StringIO()) as out:
    vcode = B.verify(args(mode='verify'))
ok('B verify: exit 0 with a-h passed and the 449 row names, nothing spawned',
   vcode == 0 and json.loads(out.getvalue())['rows350'] == EXPECTED and len(json.loads(out.getvalue())['passed']) == 8 and not w.spawned)
ok('B validate expects shared + system + 4 frozen inputs rows (448 prepare unchanged from 371)', B.expected_inputs(H, M, PIN, args()) == 6)

# 7. --prepared with an unparsed inputs name
w = World(A, csd=dict(inputs=[1, None]))
error = None
try:
    A.gather(H, args(prepared=True))
except TypeError as e:
    error = e
World(B, csd=dict(inputs=[1, None]))
problems_b = B.gather(H, args(prepared=True))[3]
ok('FINDING L2: A gather --prepared raises TypeError on an unparsed attempt name; B refuses by name',
   isinstance(error, TypeError) and any('--prepared needs attempt 1' in p for p in problems_b), (error, problems_b))
statuses = []
for R in (A, B):
    w = World(R)
    del w.csd['inputs']                                     # a malformed probe answer: KeyError inside gather
    error = run(R, raises=True)
    statuses.append((type(error).__name__, last(R)['status']))
ok('FINDING L2: an unexpected error leaves A\'s record with status null; B records it, both re-raise',
   statuses[0] == ('KeyError', None) and statuses[1][0] == 'KeyError' and statuses[1][1]['message'].startswith('unexpected KeyError'), statuses)

# 8. source checks on A
code = text453.split('Historical394 documentation follows.\n"""', 1)[1]
banned = ('rmtree', 'rmdir', 'os.remove', 'os.rename', 'os.replace', 'unlink', 'write_text', 'write_bytes', 'shutil', 'symlink',
          "open('w", "open('a", '.kill(', '.terminate(', "'scp'", "'rsync'", 'h.CQ', "h.CSS/'jobs'")
ok('A: no remove, rename, kill or copy tool, and no CQ or CSS-jobs path left', not [b for b in banned if b in code], [b for b in banned if b in code])
ok('A: two spawn sites; 448 modes check, prepare (CSD) and send (SSD) only',
   code.count('spawn(HOST_') == 2 and "'export'" not in code and "'receive'" not in code)


def top_level(text):
    """name -> source of each top-level def, class and assignment (docstrings left out)."""
    tree = ast.parse(text)
    out = {}
    for node in tree.body:
        if isinstance(node, (ast.FunctionDef, ast.ClassDef)):
            out[node.name] = ast.get_source_segment(text, node)
        elif isinstance(node, ast.Assign):
            out[','.join(ast.unparse(t) for t in node.targets)] = ast.get_source_segment(text, node)
    return out


ta, tb = top_level(text453), top_level(candidate)
differ = sorted(k for k in set(ta) | set(tb) if ta.get(k) != tb.get(k))
ok('A vs B differ only where 394 changed after 93f677ff (remote wording, probe timeout, POST_PROBE, local_manifest, gather, '
   'expected_inputs, validate, run, finish, verify, main); start, wait_lane, check_prepared and every probe but POST_PROBE are equal',
   differ == sorted(['remote', 'probe', 'POST_PROBE', 'local_manifest', 'gather', 'expected_inputs', 'validate', 'run', 'finish', 'verify', 'main']), differ)

failed = [r for r in results if not r[1]]
for name, good, note in results:
    print('PASS' if good else 'FAIL', name, '' if good else '| %s' % (note,))
print('%d/%d passed; temp %s (%d KB)' % (len(results)-len(failed), len(results), TMP.name,
      sum(p.stat().st_size for p in TMP.rglob('*') if p.is_file())//1024))
sys.exit(1 if failed else 0)
