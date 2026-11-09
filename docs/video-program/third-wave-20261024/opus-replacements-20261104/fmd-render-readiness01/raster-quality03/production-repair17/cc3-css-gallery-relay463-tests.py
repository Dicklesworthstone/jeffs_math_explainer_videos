"""CC3 tests for cc3-css-gallery-relay463.py: the 457 fixture battery replayed on the actual 463, plus read-only verify cases.

Fake transports only: no ssh, no network, no remote file, no transfer. Root's files in Q are read only; a synthetic
version-448 05_gallery manifest and every record go under one new temp folder.
  A  root's css-gallery-relay453.py (c8f5539d), kept as the comparison;
  B  the actual cc3-css-gallery-relay463.py.
Section 0 proves 463 is relay394 371f5d1f plus exactly the edits listed in DELTA (top-level statement by statement).
argv: Q temp-parent. Exit 0 when every expectation holds.
"""
import ast, contextlib, hashlib, importlib.util, io, json, shlex, sys, tempfile, types
from pathlib import Path

Q = Path(sys.argv[1]).resolve()
TMP = Path(tempfile.mkdtemp(prefix='relay463-tests-', dir=sys.argv[2]))
PINNED = {'css-gallery-relay453.py': 'c8f5539d2290416543e4a9bee50d7ef5bc2af82b61cf687ea73f30198881ec0d',
          'csd-gallery-import448.py': 'b6653757c5c114029a94832a0dd4cb9597c42808614fbb959ef6448a5d994f6e',
          'cc3-css352-import-relay394.py': '371f5d1ff415cc1309beebc7feba0f59bd4ba9b139153453d94326be16e64c8c',
          'cc3-css-gallery-relay463.py': 'd1822a5157895ae19c7128a1de374dbafa7de204aeba8c99b3895fabe4d5b892'}
results = []


def ok(name, cond, note=''):
    results.append((name, bool(cond), note))


def sha_file(p):
    return hashlib.sha256(Path(p).read_bytes()).hexdigest()


for name, digest in PINNED.items():
    assert sha_file(Q/name) == digest, 'not the tested bytes: '+name

# 0. 463 = 394 371f5d1f + DELTA, statement by statement (module docstring rewritten, compared separately)
I448, I371 = 'b6653757c5c114029a94832a0dd4cb9597c42808614fbb959ef6448a5d994f6e', 'ba337aca143316748561c4300d5774386387690b49052e72d29a04b103dd244f'
DELTA = {
    'VERSION': [("'394'", "'463'")],
    'HELPER': [("'csd-css352-import371.py'", "'csd-gallery-import448.py'")],
    'HELPER_SHA256': [(I371, I448)],
    'ORDER': [("('01_the_bump', '04_reading_color', '12_residues', '14_continuation', '05_gallery')", "('05_gallery',)")],
    'load_helper': [("'local %s is not the frozen 371 (%s)' % (HELPER, HELPER_SHA256[:12])",
                     "'local %s is not the pinned %s (%s)' % (HELPER, HELPER_LABEL, HELPER_SHA256[:12])"),
                    ("spec_from_file_location('import371', path)", "spec_from_file_location(HELPER_LABEL, path)")],
    'CSD_PROBE': [("    for j in sorted(jobs.glob(A['prefix']+d+'-a*')):\n        tail = j.name[len(A['prefix']+d+'-a'):]\n",
                   "    prefix = A.get('prefixes', {}).get(d, A['prefix'])\n    for j in sorted(jobs.glob(prefix+d+'-a*')):\n        tail = j.name[len(prefix+d+'-a'):]\n")],
    'local_manifest': [('accepted by 371 for this chapter', 'accepted by import448 for this chapter'),
                       ("'manifest refused by 371: '+", "'manifest refused by %s: ' % HELPER_LABEL+")],
    'gather': [("dict(helper=str(h.CQ/HELPER), manifest=str(h.CQ/args.manifest),\n                receipt=str(h.CSS/'jobs'/pin['job']/'resource-receipt.json'), "
                "summary=str(h.CSS/'jobs'/pin['job']/'render-summary.json')))",
                "dict(helper=str(h.RECOVERY/HELPER), manifest=str(h.RECOVERY/args.manifest),\n                receipt=str(h.RECOVERY/'jobs'/pin['job']/'resource-receipt.json'), "
                "summary=str(h.RECOVERY/'jobs'/pin['job']/'render-summary.json')))"),
               ("'CSS helper copy is not ba337aca: %s' % css['helper']", "'CSS RECOVERY copy of %s is not %s: %s' % (HELPER, HELPER_SHA256[:8], css['helper'])"),
               ("'CSS manifest copy is missing or differs: %s'", "'CSS RECOVERY manifest copy is missing or differs: %s'"),
               ('    earlier = ORDER[:ORDER.index(args.chapter)]\n', '    earlier = EARLIER371\n'),
               ("chapters=[h.dashed(c) for c in earlier+(args.chapter,)], lock=str(h.D/LANE_LOCK), running=True))",
                "chapters=[h.dashed(c) for c in earlier+(args.chapter,)],\n                prefixes={h.dashed(c): 'csd-css352-import371-' for c in earlier}, "
                "lock=str(h.D/LANE_LOCK), running=True))"),
               ("'CSD helper copy is not ba337aca: %s' % csd['helper']", "'CSD Q copy of %s is not %s: %s' % (HELPER, HELPER_SHA256[:8], csd['helper'])")],
    'expected_inputs': [('The rows 371 prepare freezes', "The rows 448 prepare (371's, unchanged) freezes")],
    'validate': [("run_371(h, 'check', args)", "run_helper(h, 'check', args)"),
                 ("('a. 371 check exits 0', code == 0)", "('a. %s check exits 0' % HELPER_LABEL, code == 0)"),
                 ("('b. 371 check reports %s and the job folder exists' % job,", "('b. %s check reports %s and the job folder exists' % (HELPER_LABEL, job),"),
                 ("('e. every manifest row is at its size and hash through its original path (371 check problems [])', shown.get('problems') == [])",
                  "('e. every manifest row is at its size and hash through its original path (%s check problems [])' % HELPER_LABEL,\n         shown.get('problems') == [])"),
                 ("('h. rows350 from 371 check and import-result.json equal the expected names',",
                  "('h. rows350 from %s check and import-result.json equal the expected names' % HELPER_LABEL,")],
    'run_371': [('def run_371(h, mode, args):\n', 'def run_helper(h, mode, args):\n    """One mode of the pinned helper on CSD, from Q: check or prepare only."""\n')],
    'check_prepared': [("exactly 371 prepare's set", "exactly 448 prepare's set"),
                       ("'--prepared: the existing inputs are not the set 371 prepare writes for this manifest')",
                        "'--prepared: the existing inputs are not the set %s prepare writes for this manifest' % HELPER_LABEL)")],
    'run': [("run_371(h, 'check', args)", "run_helper(h, 'check', args)"),
            ("'371 check refused: %s' % ((shown or {}).get('problems') or err[-400:])", "'%s check refused: %s' % (HELPER_LABEL, (shown or {}).get('problems') or err[-400:])"),
            ("run_371(h, 'prepare', args)", "run_helper(h, 'prepare', args)"),
            ("'371 prepare refused: '+err[-400:]", "'%s prepare refused: %s' % (HELPER_LABEL, err[-400:])"),
            ("'371 prepare printed another job, inputs, command, rows or pin; nothing started')",
             "'%s prepare printed another job, inputs, command, rows or pin; nothing started' % HELPER_LABEL)")],
    'start': [("str(h.CQ/HELPER), 'send'", "str(h.RECOVERY/HELPER), 'send'"), ("str(h.CQ/args.manifest), '--manifest-sha'", "str(h.RECOVERY/args.manifest), '--manifest-sha'")],
    'status': [('plus 371 check.', 'plus import448 check.'), ("run_371(h, 'check', args)", "run_helper(h, 'check', args)"),
               ("'371 check: %s' % ((shown or {}).get('problems') or err[-400:])", "'%s check: %s' % (HELPER_LABEL, (shown or {}).get('problems') or err[-400:])")]}
ADDED = {'HELPER_LABEL': "HELPER_LABEL = 'import448'",
         'EARLIER371': "EARLIER371 = ('01_the_bump', '04_reading_color', '12_residues', '14_continuation')"}
RENAMED = {'run_371': 'run_helper'}


def top_level(text):
    tree = ast.parse(text)
    out = {}
    for node in tree.body:
        if isinstance(node, (ast.FunctionDef, ast.ClassDef)):
            out[node.name] = ast.get_source_segment(text, node)
        elif isinstance(node, ast.Assign):
            out[','.join(ast.unparse(t) for t in node.targets)] = ast.get_source_segment(text, node)
    return out


text394, text453, text463 = ((Q/n).read_text() for n in ('cc3-css352-import-relay394.py', 'css-gallery-relay453.py', 'cc3-css-gallery-relay463.py'))
t394, t463 = top_level(text394), top_level(text463)
bad = []
for name, src in t394.items():
    for old, new in DELTA.get(name, []):
        if src.count(old) != 1:
            bad.append('%s: %d x %r' % (name, src.count(old), old[:50]))
        src = src.replace(old, new)
    if t463.get(RENAMED.get(name, name)) != src:
        bad.append('differs beyond DELTA: '+name)
for name, src in ADDED.items():
    if t463.get(name) != src:
        bad.append('added statement differs: '+name)
extra = set(t463) - {RENAMED.get(n, n) for n in t394} - set(ADDED)
ok('0. 463 is 394 371f5d1f plus exactly DELTA: %d statements edited, 2 added, 1 renamed, %d byte-identical'
   % (len(DELTA), len(t394)-len(DELTA)), not bad and not extra, (bad, extra))
ok('0. byte-identical to 394: finish, verify, wait_lane, remote, probe, spawn, expected_rows, POST_PROBE, CSS/JOB/INPUTS probes, timeouts',
   all(t394[n] == t463[n] for n in ('finish', 'verify', 'wait_lane', 'remote', 'probe', 'spawn', 'expected_rows', 'POST_PROBE', 'CSS_PROBE',
                                    'JOB_PROBE', 'INPUTS_PROBE', 'PROBE_COMMON', 'lane_free', 'Stop', 'SSH', 'PROBE_SECONDS', 'CHECK_SECONDS',
                                    'ADMISSION_SECONDS', 'STREAM_SECONDS', 'POLL_SECONDS', 'LANE_LOCK', 'main')))
doc = ast.get_docstring(ast.parse(text463))
ok('0. 463 docstring names relay 463, import448 b6653757, RECOVERY, the four 371 chapters and a-h; no 394 run-folder name',
   doc.startswith('Root-review relay 463') and 'b6653757' in doc and 'RECOVERY' in doc and 'relay463-05-gallery-a<N>-run<K>' in doc
   and 'relay394-<chapter>' not in doc and all(c in doc for c in ('01_the_bump', '04_reading_color', '12_residues', '14_continuation'))
   and all('%s. ' % c in doc for c in 'abcdefgh'))


def load(path, name):
    spec = importlib.util.spec_from_file_location(name, path)
    r = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(r)
    r.time = types.SimpleNamespace(monotonic=__import__('time').monotonic, sleep=lambda s: None)
    return r


A = load(Q/'css-gallery-relay453.py', 'relay453')
B = load(Q/'cc3-css-gallery-relay463.py', 'relay463')
H = B.load_helper()                                         # 448 at its pinned sha, from Q, through 463's own loader
ok('463 load_helper loads 448 under its label', H.VERSION == '448' and H.__name__ == 'import448')
for R in (A, B):
    R.load_helper = lambda: H
    R.HERE = TMP                                            # the synthetic manifest lives in TMP, never in Q

CH, JOB442 = '05_gallery', H.css_job('05_gallery')
PIN = dict(job=JOB442, worker=str(H.WORKER), worker_sha256=H.WORKER_SHA256, inputs=str(H.css_inputs(CH)), inputs_sha256='1'*64,
           permit=str(H.PERMITS/'05_gallery-css-permit375.json'), permit_sha256='2'*64, receipt_sha256='3'*64, summary_sha256='4'*64,
           movie_sha256='5'*64, frames=9879)
ROWS = [H.RECOVERY/'jobs'/JOB442/'resource-receipt.json', H.RECOVERY/'jobs'/JOB442/'render-summary.json', H.css_inputs(CH),
        H.CSS/'frozen-fixture.bin', H.D/'work/bin/fixture-shared', Path('/usr/lib/fixture-system.so')]
M = H.manifest_from_rows({CH: PIN}, [dict(path=str(p), sha256='%064x' % (i+7), bytes=10+i, mode=0o644) for i, p in enumerate(ROWS)])
assert H.manifest_problems(M) == [] and H.chapter_of(M) == CH, H.manifest_problems(M)
MNAME = 'css352-import448-05-gallery-manifest-fixture463.json'
data = json.dumps(M, indent=1).encode()
with (TMP/MNAME).open('xb') as f:
    f.write(data)
MSHA = hashlib.sha256(data).hexdigest()

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
        self.spawned, self.checks, self.calls, self.commands, self.post_probes = [], 0, [], [], 0
        R.probe, R.remote, R.spawn = self.probe, self.remote, self.spawn

    def probe(self, host, script, a, timeout=None):
        R = self.R
        self.calls.append((host, script, a, timeout))
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
            if isinstance(self.rows, Exception):
                raise self.rows
            return self.rows
        raise AssertionError(script[:40])

    def remote(self, host, command, stdin=b'', timeout=0):
        self.commands.append((host, command, timeout))
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
    runs = sorted(TMP.glob('relay%s-05-gallery-a1-run*' % R.VERSION), key=lambda p: int(p.name.rsplit('run', 1)[1]))
    return json.loads((runs[-1]/'record.json').read_text())


NEEDLES = {A: dict(css_helper='CSS helper copy', css_manifest='CSS manifest copy', csd_helper='CSD helper copy'),
           B: dict(css_helper='CSS RECOVERY copy of csd-gallery-import448.py is not b6653757', css_manifest='CSS RECOVERY manifest copy',
                   csd_helper='CSD Q copy of csd-gallery-import448.py is not b6653757')}

for R, tag in ((A, 'A453'), (B, 'B463')):
    # 1. routing: SSD sender, original receiving guard and lane, 448 names
    w = World(R, check=CHECK_OK, prepare=PREPARE_OK, post=POST_OK, admission=[dict(ADMITTED, admitted=False, pid=None), ADMITTED])
    code = run(R)
    css_args = [a for h, s, a, t in w.calls if s is R.CSS_PROBE][0]
    csd_args = [a for h, s, a, t in w.calls if s is R.CSD_PROBE and a['running']][0]
    lane_args = [a for h, s, a, t in w.calls if s is R.CSD_PROBE and not a['running']]
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
    ok(tag+' rows350 names = 449/461 expectation (import448 a1)', last(R)['status']['rows350'] == EXPECTED
       and JOB == 'csd-css352-import448-05-gallery-a1', last(R)['status'])

    # 2. refusals before anything starts
    n = NEEDLES[R]
    for name, kw, needle in (('RECOVERY helper copy differs', dict(css=dict(helper='0'*64)), n['css_helper']),
                             ('RECOVERY manifest copy differs', dict(css=dict(manifest='0'*64)), n['css_manifest']),
                             ('442 receipt not terminal', dict(css=dict(terminal=False)), 'not a terminal success'),
                             ('stale gallery export', dict(css=dict(summary='0'*64)), 'not the export of the finished job'),
                             ('CSD 448 copy differs', dict(csd=dict(helper='0'*64)), n['csd_helper']),
                             ('gallery attempt 1 inputs exist', dict(csd=dict(inputs=[1])), 'never re-admits'),
                             ('gallery already imported', dict(csd=dict(imports=dict({d: [DONE] for d in EARLY}, **{'05-gallery': [dict(DONE, attempt=1)]}))), 'already imported')):
        w = World(R, check=CHECK_OK, prepare=PREPARE_OK, **kw)
        code = run(R)
        ok('%s refused: %s' % (tag, name), code == 1 and not w.spawned and w.checks == 0 and needle in last(R)['status']['message'], last(R)['status']['message'][:100])
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
        with (j/'resource-receipt.json').open('x') as f:
            f.write(json.dumps(dict(finished='t', exit=0 if terminal_ok else 1, inputs_sha256='k', final_inputs_sha256='k')))
        with (j/'import-result.json').open('x') as f:
            f.write('{}')
    with (root/'q'/'csd-css352-import448-05-gallery-a3-inputs.json').open('x') as f:
        f.write('{}')
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

    # 5. a version-371 manifest is refused cleanly (448's own wording)
    bad = dict(M, version='371')
    with (TMP/('v371-'+tag+'.json')).open('x') as f:
        f.write(json.dumps(bad))
    vsha = sha_file(TMP/('v371-'+tag+'.json'))
    w = World(R)
    run(R, args(manifest='v371-'+tag+'.json', manifest_sha=vsha))
    ok(tag+' a version-371 manifest is refused cleanly (stop 1, 448 text "not a version-371 manifest")',
       last(R)['status']['code'] == 1 and 'not a version-371 manifest' in last(R)['status']['message'], last(R)['status'])

# 6. L1: helper messages
w = World(A, css=dict(helper='0'*64))
run(A)
msg_a = last(A)['status']['message']
w = World(B, css=dict(helper='0'*64), csd=dict(helper='0'*64))
run(B)
msg_b = last(B)['status']['message']
ok('L1 closed: 463 names csd-gallery-import448.py and b6653757 on both hosts, never ba337aca (453 still says ba337aca)',
   'ba337aca' in msg_a and 'ba337aca' not in msg_b and msg_b.count('csd-gallery-import448.py is not b6653757') == 2, (msg_a[:80], msg_b[:200]))
v = World(B)
run(B, args(manifest='v371-B463.json', manifest_sha=sha_file(TMP/'v371-B463.json')))
ok('L1 closed: 463 refusal names import448, not 371', last(B)['status']['message'].startswith('manifest refused by import448:'), last(B)['status'])
ok('L1 closed: no 371/ba337aca in any 463 message except the four earlier chapters\' 371 imports',
   'ba337aca' not in text463 and 'frozen 371' not in text463 and "'import371'" not in text463 and 'run_371' not in text463)

# 7. M1: post-import validation a-h (453 keeps the implicit check; 463 stops)
for name, rows in (('guard inputs file changed after admission (f)', dict(ROWS_OK, inputs_sha256='j'*64)),
                   ("a frozen non-manifest row changed, e.g. Q's permit copy or the guard (g)", dict(ROWS_OK, changed=['q-permit'], changed_count=1)),
                   ('inputs hold fewer rows than 448 prepare writes (g)', dict(ROWS_OK, rows=5, distinct=5))):
    wa = World(A, check=CHECK_OK, prepare=PREPARE_OK, post=POST_OK, rows=rows, admission=[ADMITTED])
    ca = run(A)
    wb = World(B, check=CHECK_OK, prepare=PREPARE_OK, post=POST_OK, rows=rows, admission=[ADMITTED])
    cb = run(B)
    ok('M1 closed: %s: 453 exits 0 without reading the inputs; 463 stops 3 naming it' % name,
       ca == 0 and wa.post_probes == 0 and cb == 3 and 'post-import validation failed' in last(B)['status']['message'], (ca, cb))
codes = []
for R in (A, B):
    World(R, check=CHECK_OK, prepare=PREPARE_OK, post=dict(POST_OK, problems=['not at its hash on CSD: x']), admission=[ADMITTED])
    codes.append(run(R))
ok('453 and 463 both stop 3 when 448 check reports a row off its hash (e)', codes == [3, 3], codes)
w = World(B, check=CHECK_OK, prepare=PREPARE_OK, post=POST_OK, admission=[ADMITTED])
code = run(B)
ok('463 success: "imported and validated", a-h all passed, labels name import448',
   code == 0 and last(B)['status']['message'] == 'imported and validated' and [p[0] for p in last(B)['status']['validation_passed']] == list('abcdefgh')
   and last(B)['status']['validation_passed'][0] == 'a. import448 check exits 0')
ok('463 validate expects shared + system + 4 frozen inputs rows (448 prepare unchanged from 371)', B.expected_inputs(H, M, PIN, args()) == 6)
w = World(B, check=CHECK_OK, prepare=PREPARE_OK, post=POST_OK, admission=[ADMITTED], rows=B.Stop(2, 'csd: no answer'))
code = run(B)
ok('463: transport during the post-import validation: stop 2, tells root to run verify', code == 2 and 'verify' in last(B)['status']['message'])

# 8. L2: --prepared parse and unexpected errors
World(A, csd=dict(inputs=[1, None]))
error = None
try:
    A.gather(H, args(prepared=True))
except TypeError as e:
    error = e
World(B, csd=dict(inputs=[1, None]))
problems_b = B.gather(H, args(prepared=True))[3]
ok('L2 closed: --prepared with an unparsed attempt: 453 raises TypeError, 463 refuses by name',
   isinstance(error, TypeError) and any('--prepared needs attempt 1' in p for p in problems_b), (error, problems_b))
statuses = []
for R in (A, B):
    w = World(R)
    del w.csd['inputs']
    error = run(R, raises=True)
    statuses.append((type(error).__name__, last(R)['status']))
ok('L2 closed: an unexpected error: 453 record status null, 463 records it; both re-raise',
   statuses[0] == ('KeyError', None) and statuses[1][0] == 'KeyError' and statuses[1][1]['message'].startswith('unexpected KeyError'), statuses)

# 9. read-only verify on the actual 463: every failure case, transport, and what it never does
before = sorted(p.name for p in TMP.iterdir())
CHECK_CMD = shlex.join(['/usr/bin/python3', '-I', '-B', str(H.Q/'csd-gallery-import448.py'), 'check', '--chapter', CH, '--attempt', '1', '--manifest', str(H.Q/MNAME)])


def verify(world_kw, a=None):
    w = World(B, **world_kw)
    with contextlib.redirect_stdout(io.StringIO()) as out, contextlib.redirect_stderr(io.StringIO()) as err:
        code = B.verify(a or args(mode='verify'))
    report = json.loads(out.getvalue()) if out.getvalue().strip() else None
    return w, code, report, err.getvalue()


w, code, report, _ = verify(dict(check=POST_OK))
post_calls = [(a, t) for h, s, a, t in w.calls if s is B.POST_PROBE]
ok('verify pass: exit 0, a-h, rows350, the exact 448 check command, POST_PROBE on the a1 job and inputs with the long timeout',
   code == 0 and report['failures'] == [] and len(report['passed']) == 8 and report['rows350'] == EXPECTED
   and [c for _, c, _ in w.commands] == [CHECK_CMD] and w.commands[0][2] == B.CHECK_SECONDS
   and post_calls == [(dict(job=str(H.D/'jobs'/JOB), inputs=str(H.inputs_path(JOB))), B.CHECK_SECONDS)], (code, report and report['failures']))
for letter, kw in (('a', dict(check=dict(POST_OK, problems=['not at its hash on CSD: x']))),
                   ('b', dict(check=dict(POST_OK, job='csd-css352-import448-05-gallery-a2'))),
                   ('b', dict(check=dict(POST_OK, job_exists=False))),
                   ('c', dict(check=dict(POST_OK, terminal_success=False))),
                   ('c', dict(check=POST_OK, rows=dict(ROWS_OK, terminal=False))),
                   ('d', dict(check=dict(POST_OK, import_result=False))),
                   ('d', dict(check=POST_OK, rows=dict(ROWS_OK, result_manifest_sha256='0'*64))),
                   ('f', dict(check=POST_OK, rows=dict(ROWS_OK, inputs_sha256='j'*64))),
                   ('f', dict(check=POST_OK, rows=dict(ROWS_OK, inputs_sha256=None, receipt_inputs_sha256=None))),
                   ('g', dict(check=POST_OK, rows=dict(ROWS_OK, changed=['/usr/lib/x'], changed_count=1))),
                   ('g', dict(check=POST_OK, rows=dict(ROWS_OK, rows=7, distinct=7))),
                   ('g', dict(check=POST_OK, rows=dict(ROWS_OK, distinct=5))),
                   ('h', dict(check=dict(POST_OK, rows350={}))),
                   ('h', dict(check=POST_OK, rows=dict(ROWS_OK, result_rows350={CH: dict(import_receipt='x', import_inputs='y')}))),
                   ('c', dict(check=dict(problems=['earlier attempt still running or unrecorded: x'], job=JOB, job_exists=True, finished=False),
                              rows=dict(ROWS_OK, terminal=False, result_manifest_sha256=None, result_rows350=None)))):
    w, code, report, _ = verify(kw)
    letters = {f[0] for f in report['failures']}
    ok('verify fails %s: exit 3, failure named, nothing spawned, no prepare: %s' % (letter, json.dumps(kw)[:70]),
       code == 3 and letter in letters and not w.spawned and all(' prepare ' not in c for _, c, _ in w.commands), (code, sorted(letters)))
w, code, report, err = verify(dict(check=B.Stop(2, 'csd: ssh exit 255 (transport)')))
ok('verify: build host transport failure at 448 check: exit 2, no report, nothing spawned', code == 2 and report is None and 'STOP 2' in err and not w.spawned)
w, code, report, err = verify(dict(check=POST_OK, rows=B.Stop(2, 'csd: no answer within 3600 s')))
ok('verify: transport failure at the inputs re-hash: exit 2, nothing spawned', code == 2 and report is None and not w.spawned)
w, code, report, err = verify(dict(check=POST_OK), args(mode='verify', manifest_sha='0'*64))
ok('verify: manifest not at the sha root gave: exit 1 before any remote call', code == 1 and not w.commands and not w.calls)
w, code, report, err = verify(dict(check=dict(POST_OK, job=H.job_name(CH, 2), rows350={CH: dict(
    import_receipt=str(H.D/'jobs'/H.job_name(CH, 2)/'resource-receipt.json'), import_inputs=str(H.inputs_path(H.job_name(CH, 2))))}),
    rows=dict(ROWS_OK, result_rows350={CH: dict(import_receipt=str(H.D/'jobs'/H.job_name(CH, 2)/'resource-receipt.json'),
                                                import_inputs=str(H.inputs_path(H.job_name(CH, 2))))})), args(mode='verify', attempt=2))
ok('verify --attempt 2 validates the a2 names (449/461 bind a1 only, so root would need a new controller)',
   code == 0 and report['job'] == 'csd-css352-import448-05-gallery-a2' and ' --attempt 2 ' in w.commands[0][1])
ok('verify wrote nothing: no new file or folder in the temp tree', sorted(p.name for p in TMP.iterdir()) == before)

# 10. source checks on 463
code = text463.split('"""', 2)[2]
banned = ('rmtree', 'rmdir', 'os.remove', 'os.rename', 'os.replace', 'unlink', 'write_text', 'write_bytes', 'shutil', 'symlink',
          "open('w", "open('a", '.kill(', '.terminate(', 'send_signal', "'scp'", "'rsync'", 'h.CQ', "h.CSS/'jobs'")
ok('463: no remove, rename, kill, signal or copy tool, and no CQ or CSS-jobs path', not [b for b in banned if b in code], [b for b in banned if b in code])
ok('463: two spawn sites; helper modes check and prepare (build host) and send (SSD) only',
   code.count('spawn(HOST_') == 2 and "'export'" not in code and "'receive'" not in code
   and "run_helper(h, 'check'" in code and "run_helper(h, 'prepare'" in code and "'send'" in code)
body = code.split('def verify', 1)[1].split('def main', 1)[0]
ok('463: verify never spawns or prepares', 'spawn' not in body and 'prepare' not in body)

failed = [r for r in results if not r[1]]
for name, good, note in results:
    print('PASS' if good else 'FAIL', name, '' if good else '| %s' % (note,))
print('%d/%d passed; temp %s (%d KB)' % (len(results)-len(failed), len(results), TMP.name,
      sum(p.stat().st_size for p in TMP.rglob('*') if p.is_file())//1024))
sys.exit(1 if failed else 0)
