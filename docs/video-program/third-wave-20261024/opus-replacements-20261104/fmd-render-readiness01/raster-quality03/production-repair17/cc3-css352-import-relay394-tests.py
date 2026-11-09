"""Tiny local tests for cc3-css352-import-relay394.py: fake transports, no network, a few KB of temp files.
argv: Q temp-parent. Reads root's manifests 392 and 396 in Q read-only; writes only under a new folder in temp-parent."""
import contextlib, hashlib, importlib.util, io, json, os, re, shlex, sys, tempfile, types
from pathlib import Path

Q = Path(sys.argv[1]).resolve()
TMP = Path(tempfile.mkdtemp(prefix='relay394-test-', dir=sys.argv[2]))
RELAY = 'cc3-css352-import-relay394.py'
M392, SHA392 = 'css352-import371-01-the-bump-manifest392.json', '633c06d0542f6669eeb42993a3b1b91a1403a752ec259757ef5ab7297a24ac4b'
M396, SHA396 = 'css352-import371-04-reading-color-manifest396.json', '04d1cd99e0114e63abfeaefa23aab7d3ce6e56092d77d8a4459fa3cb2da6896c'


def load():
    spec = importlib.util.spec_from_file_location('relay394', Q/RELAY)
    r = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(r)
    r.time = types.SimpleNamespace(monotonic=__import__('time').monotonic, sleep=lambda s: None)
    return r


R = load()
H = R.load_helper()
M = json.loads((Q/M392).read_bytes())
PIN = M['chapters']['01_the_bump']
M4 = json.loads((Q/M396).read_bytes())
PIN4 = M4['chapters']['04_reading_color']
results = []


def ok(name, cond, note=''):
    results.append((name, bool(cond), note))


def quiet(fn, *a):
    with contextlib.redirect_stdout(io.StringIO()) as out, contextlib.redirect_stderr(io.StringIO()) as err:
        code = fn(*a)
    return code, out.getvalue(), err.getvalue()


def args(**kw):
    a = dict(mode='run', chapter='01_the_bump', attempt=1, manifest=M392, manifest_sha=SHA392, prepared=False,
             lane_wait_minutes=0, record_dir=TMP)
    a.update(kw)
    return types.SimpleNamespace(**a)


job = H.job_name('01_the_bump', 1)
ROWS = R.expected_rows(H, '01_the_bump', 1)
GUARD = H.guard_command(H.Q/R.HELPER, '01_the_bump', 1, H.Q/M392)
RECEIVE = [str(c) for c in H.receive_command(H.Q/R.HELPER, '01_the_bump', 1, H.Q/M392)]
WANT = R.expected_inputs(H, M, PIN, args())
CHECK_OK = dict(problems=[], job=job, job_exists=False)
PREPARE_OK = dict(job=job, inputs=str(H.inputs_path(job)), manifest_sha256=SHA392, guard_command=GUARD, rows350=ROWS, chapter_pin=PIN)
ADMITTED = dict(exists=True, receipt=True, admitted=True, command=RECEIVE, pid=123, finished=None, exit=None)
POST_OK = dict(problems=[], rows350=ROWS, job=job, job_exists=True, terminal_success=True, import_result=True)
ROWS_OK = dict(terminal=True, inputs_sha256='i'*64, receipt_inputs_sha256='i'*64, rows=WANT, distinct=WANT, changed=[], changed_count=0,
               result_manifest_sha256=SHA392, result_rows350=ROWS)


def facts(css=None, csd=None):
    c = dict(helper=R.HELPER_SHA256, manifest=SHA392, receipt=PIN['receipt_sha256'], summary=PIN['summary_sha256'], terminal=True)
    d = dict(helper=R.HELPER_SHA256, manifest=SHA392, imports={'01-the-bump': []}, inputs=[], lane=dict(holders=[], waiters=[]), running=[])
    c.update(css or {})
    d.update(csd or {})
    return c, d


class World:
    """Fake hosts: probes answer by script, remote answers 371 check/prepare, spawn records each start."""

    def __init__(self, css=None, csd=None, lanes=None, check=None, prepare=None, post=None, rows=None, admission=None, exits=(0, 0)):
        self.css, self.csd = facts(css, csd)
        self.lanes = list(lanes or [])
        self.check, self.prepare, self.post = check, prepare, post
        self.rows = ROWS_OK if rows is None else rows
        self.admission = list(admission or [])
        self.exits = exits
        self.spawned, self.checks, self.calls = [], 0, []
        R.probe, R.remote, R.spawn = self.probe, self.remote, self.spawn

    def probe(self, host, script, a, timeout=None):
        self.calls.append((host, script[-60:], timeout))
        if script is R.CSS_PROBE:
            return self.css
        if script is R.CSD_PROBE:
            d = dict(self.csd)
            if not a['running'] and self.lanes:
                d['lane'] = self.lanes.pop(0)
            return d
        if script is R.JOB_PROBE:
            return self.admission.pop(0) if self.admission else dict(exists=False, receipt=False, admitted=False, command=[], pid=None, finished=None, exit=None)
        if script is R.POST_PROBE:
            if isinstance(self.rows, Exception):
                raise self.rows
            return self.rows
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
                self.closed = False
                self.stdin = types.SimpleNamespace(close=self.close)
                self.returncode = None
                self.host = host

            def close(self):
                self.closed = True

            def poll(self):
                return self.returncode

            def wait(self, timeout=None):
                self.returncode = world.exits[0 if self.host == R.HOST_CSS else 1]
                return self.returncode
        p = P()
        self.spawned.append((host, command, p))
        return p


def last_record(dashed='01-the-bump'):
    runs = sorted(TMP.glob('relay394-%s-a1-run*' % dashed), key=lambda p: int(p.name.rsplit('run', 1)[1]))
    return json.loads((runs[-1]/'record.json').read_text()), runs


def run(a=None):
    return quiet(R.run, a or args())[0]


# 1. success: lane free, check, prepare, admission, send, explicit post-import validation
w = World(check=CHECK_OK, prepare=PREPARE_OK, post=POST_OK, admission=[dict(ADMITTED, exists=False, admitted=False, pid=None), ADMITTED])
code, out, _ = quiet(R.run, args())
rec, runs = last_record()
hosts = [h for h, _, _ in w.spawned]
ok('success: imported and validated', code == 0 and rec['status']['message'] == 'imported and validated' and rec['status']['rows350'] == ROWS, code)
ok('success: all eight conditions a-h passed', len(rec['status']['validation_passed']) == 8 and [p[0] for p in rec['status']['validation_passed']] == list('abcdefgh'))
ok('success: printed report carries rows350 and the validator name', json.loads(out)['validation']['rows350'] == ROWS and 'validate a-h' in out)
ok('expected inputs rows = shared + system + 4 frozen files', WANT == M['shared_files'] + M['system_files'] + 4, (WANT, M['shared_files'], M['system_files']))
ok('rows probe gets the long timeout', any(t == R.CHECK_SECONDS for _, s, t in w.calls if 'result_rows350' in s))
ok('receive before send, once each', hosts == [R.HOST_CSD, R.HOST_CSS], hosts)
ok('receive command is 371 guard_command', w.spawned[0][1] == shlex.join(GUARD))
ok('send command names the CQ helper, chapter, CQ manifest and sha',
   w.spawned[1][1] == shlex.join(['/usr/bin/python3', '-I', '-B', str(H.CQ/R.HELPER), 'send', '--chapter', '01_the_bump',
                                  '--manifest', str(H.CQ/M392), '--manifest-sha', SHA392]))
steps = [s['step'] for s in rec['steps']]
ok('send only after an admission poll shows this command', steps.index('send started') > max(i for i, s in enumerate(steps) if s == 'admission poll'), steps)

# 2. refusals before anything starts: nothing spawned, no 371 call, attempt number left free
for name, kw, needle in (
        ('helper copy on CSS differs', dict(css=dict(helper='0'*64)), 'CSS helper copy'),
        ('manifest copy on CSD missing', dict(csd=dict(manifest=None)), 'CSD manifest copy'),
        ('CSS render not terminal', dict(css=dict(terminal=False)), 'not a terminal success'),
        ('stale export', dict(css=dict(receipt='1'*64)), 'not the export of the finished job'),
        ('already imported', dict(csd=dict(imports={'01-the-bump': [dict(job=job, attempt=1, finished=True, imported=True)]})), 'already imported'),
        ('existing attempt 1 job', dict(csd=dict(imports={'01-the-bump': [dict(job=job, attempt=1, finished=False, imported=False)]})), 'never re-admits'),
        ('existing attempt 1 inputs', dict(csd=dict(inputs=[1])), 'never re-admits'),
        ('unparsed attempt name', dict(csd=dict(inputs=[None])), 'never re-admits')):
    w = World(check=CHECK_OK, prepare=PREPARE_OK, post=POST_OK, **kw)
    code = run()
    rec, _ = last_record()
    ok('refused: '+name, code == 1 and not w.spawned and w.checks == 0 and needle in rec['status']['message'], rec['status']['message'][:90])

# 3. manifest and serial order
w = World()
code = run(args(chapter='04_reading_color', manifest=M392))
rec, _ = last_record('04-reading-color')
ok('04 with the 01 manifest: refused by 371 manifest rules', code == 1 and 'manifest refused by 371' in rec['status']['message'], rec['status']['message'][:80])
code = run(args(manifest_sha='0'*64))
rec, _ = last_record()
ok('manifest sha other than root gave: refused', code == 1 and 'not at the sha root gave' in rec['status']['message'])
four = dict(css=dict(manifest=SHA396, receipt=PIN4['receipt_sha256'], summary=PIN4['summary_sha256']),
            csd=dict(manifest=SHA396, imports={'01-the-bump': [], '04-reading-color': []}))
w = World(check=CHECK_OK, **four)
code = run(args(chapter='04_reading_color', manifest=M396, manifest_sha=SHA396))
rec, _ = last_record('04-reading-color')
ok('serial order: 04 refused while 01 has no completed import', code == 1 and 'serial order: 01_the_bump' in rec['status']['message'] and not w.spawned and w.checks == 0,
   rec['status']['message'][:90])
four['csd']['imports'] = {'01-the-bump': [dict(job=job, attempt=1, finished=True, imported=False)], '04-reading-color': []}
w = World(check=CHECK_OK, **four)
code = run(args(chapter='04_reading_color', manifest=M396, manifest_sha=SHA396))
rec, _ = last_record('04-reading-color')
ok('serial order: a finished but not imported 01 does not count', code == 1 and 'serial order' in rec['status']['message'])
four['csd']['imports'] = {'01-the-bump': [dict(job=job, attempt=1, finished=True, imported=True)], '04-reading-color': []}
w = World(**four)
_, _, _, problems = R.gather(H, args(chapter='04_reading_color', manifest=M396, manifest_sha=SHA396))
ok('serial order: 04 accepted after 01 imported', problems == [], problems)

# 4. lane busy, no wait: stop 4 before 371 check; nothing spawned
w = World(check=CHECK_OK, prepare=PREPARE_OK, lanes=[dict(holders=[999], waiters=[])])
code = run()
ok('busy lane stops before check', code == 4 and w.checks == 0 and not w.spawned, code)
w = World(check=CHECK_OK, prepare=PREPARE_OK, lanes=[dict(holders=[], waiters=[]), dict(holders=[], waiters=[998])])
code = run()
rec, _ = last_record()
ok('lane taken during prepare: stop 4 after prepare, nothing spawned', code == 4 and w.checks == 1 and not w.spawned and rec['steps'][-1]['step'] == 'prepare', code)

# 5. transport failure: before start, during the admission poll, during the stream
w = World(check=R.Stop(2, 'csd: ssh exit 255 (transport)'))
ok('transport stop at 371 check: stop 2, nothing spawned', run() == 2 and not w.spawned)


class Failing(World):
    def probe(self, host, script, a, timeout=None):
        if script is R.JOB_PROBE:
            raise R.Stop(2, 'csd: ssh exit 255 (transport)')
        return super().probe(host, script, a, timeout)


w = Failing(check=CHECK_OK, prepare=PREPARE_OK)
code = run()
rec, _ = last_record()
ok('transport stop after receive: stdin closed, no send, named', code == 2 and len(w.spawned) == 1 and w.spawned[0][2].closed
   and 'admission unknown' in rec['status']['message'], code)

# 6. never admitted (guard refused or another command): stdin closed, no send, stop 3
w = World(check=CHECK_OK, prepare=PREPARE_OK, admission=[dict(ADMITTED, admitted=False, finished='t', exit=None, pid=None)])
code = run()
ok('admission refused: no send', code == 3 and len(w.spawned) == 1 and w.spawned[0][2].closed, code)
w = World(check=CHECK_OK, prepare=PREPARE_OK, admission=[dict(ADMITTED, command=RECEIVE[:-1]+['other'])]*3)
R.ADMISSION_SECONDS, saved = -1, R.ADMISSION_SECONDS
code = run()
R.ADMISSION_SECONDS = saved
ok('admitted for another command: no send', code == 3 and len(w.spawned) == 1, code)

# 7. prepare printing other names; stream failures
w = World(check=CHECK_OK, prepare=dict(PREPARE_OK, rows350={'01_the_bump': dict(import_receipt='x', import_inputs='y')}))
ok('prepare with other rows350: no start', run() == 1 and not w.spawned)
w = World(check=CHECK_OK, prepare=PREPARE_OK, post=POST_OK, admission=[ADMITTED], exits=(0, 75))
ok('guard resource stop: stop 3, no validation', run() == 3 and w.checks == 1)
w = World(check=CHECK_OK, prepare=PREPARE_OK, post=POST_OK, admission=[ADMITTED], exits=(255, 1))
ok('ssh 255 in the stream: stop 2', run() == 2)

# 8. post-import validation: each condition fails on its own, by name
cases = (('a', dict(post=dict(POST_OK, problems=['not at its hash on CSD: x']))),
         ('b', dict(post=dict(POST_OK, job='other'))),
         ('c', dict(post=dict(POST_OK, terminal_success=False))),
         ('c', dict(rows=dict(ROWS_OK, terminal=False))),
         ('d', dict(post=dict(POST_OK, import_result=False))),
         ('d', dict(rows=dict(ROWS_OK, result_manifest_sha256='0'*64))),
         ('f', dict(rows=dict(ROWS_OK, inputs_sha256='j'*64))),
         ('f', dict(rows=dict(ROWS_OK, inputs_sha256=None, receipt_inputs_sha256=None))),
         ('g', dict(rows=dict(ROWS_OK, changed=['/usr/lib/x'], changed_count=1))),
         ('g', dict(rows=dict(ROWS_OK, rows=WANT-1, distinct=WANT-1))),
         ('g', dict(rows=dict(ROWS_OK, distinct=WANT-1))),
         ('h', dict(post=dict(POST_OK, rows350={}))),
         ('h', dict(rows=dict(ROWS_OK, result_rows350={}))))
for letter, kw in cases:
    w = World(check=CHECK_OK, prepare=PREPARE_OK, admission=[ADMITTED], **dict(dict(post=POST_OK), **kw))
    code = run()
    rec, _ = last_record()
    failures = [s for s in rec['steps'] if s['step'] == 'post-import validation'][-1]['failures']
    expected = {letter} | ({'a', 'e'} if kw.get('post', {}).get('problems') else set())
    ok('validation %s fails by name: %s' % (letter, sorted(kw)), code == 3 and {f[0] for f in failures} == expected
       and 'post-import validation failed' in rec['status']['message'], failures)
w = World(check=CHECK_OK, prepare=PREPARE_OK, post=POST_OK, admission=[ADMITTED], rows=R.Stop(2, 'csd: no answer'))
code = run()
rec, _ = last_record()
ok('transport during validation: stop 2, tells root to run verify', code == 2 and 'verify' in rec['status']['message'], rec['status']['message'][:120])

# 9. verify mode: read-only, exit 0 only when a-h pass
w = World(check=POST_OK)
code, out, _ = quiet(R.verify, args(mode='verify'))
ok('verify passes on a validated import, starts nothing', code == 0 and not w.spawned and json.loads(out)['failures'] == [])
w = World(check=dict(POST_OK, terminal_success=False))
code, out, _ = quiet(R.verify, args(mode='verify'))
ok('verify fails (3) with the named condition', code == 3 and json.loads(out)['failures'][0].startswith('c.'))
w = World(check=dict(problems=['earlier attempt still running or unrecorded: x'], job=job, job_exists=True, finished=False))
code, out, _ = quiet(R.verify, args(mode='verify'))
ok('verify of an unfinished job fails', code == 3 and not w.spawned)

# 10. --prepared: attempt must be the latest with inputs and no job; inputs set must equal prepare's
w = World(csd=dict(inputs=[1]))
want = {x['path']: x['sha256'] for x in M['files'] if x['kind'] in ('shared', 'system')}
want.update({str(H.Q/R.HELPER): R.HELPER_SHA256, str(H.Q/M392): SHA392, str(H.q_permit(PIN)): PIN['permit_sha256'], str(H.GUARD): 'g'*64})
rows = [dict(path=p, sha256=s) for p, s in sorted(want.items())]
orig = w.probe
w.probe = lambda host, script, a, timeout=None: dict(job=False, rows=rows) if script is R.INPUTS_PROBE else orig(host, script, a, timeout)
R.probe = w.probe
ok('prepared inputs accepted', R.check_prepared(H, M, PIN, args(prepared=True))['guard_sha256'] == 'g'*64)
bad = rows + [dict(path='/usr/bin/extra', sha256='e'*64)]
w.probe = lambda host, script, a, timeout=None: dict(job=False, rows=bad) if script is R.INPUTS_PROBE else orig(host, script, a, timeout)
R.probe = w.probe
try:
    R.check_prepared(H, M, PIN, args(prepared=True))
    ok('prepared inputs with an extra row refused', False)
except R.Stop as e:
    ok('prepared inputs with an extra row refused', e.code == 1)
_, _, _, problems = R.gather(H, args(prepared=True, attempt=2))
ok('--prepared for a number that has no inputs refused', any('--prepared needs attempt 2' in p for p in problems), problems)
w = World(csd=dict(inputs=[1, None]))
_, _, _, problems = R.gather(H, args(prepared=True, attempt=1))
ok('--prepared with an unparsed attempt name refused, no crash', any('--prepared needs attempt 1' in p for p in problems), problems)

# 11. lane parsing in the CSD probe on a synthetic /proc/locks; POST_PROBE on synthetic files
lock = TMP/'lane.lock'
lock.write_bytes(b'')
st = os.stat(lock)
dev = '%02x:%02x:%d' % (os.major(st.st_dev), os.minor(st.st_dev), st.st_ino)
locks = TMP/'locks.txt'
locks.write_text('1: FLOCK  ADVISORY  WRITE 4242 %s 0 EOF\n1: -> FLOCK  ADVISORY  WRITE 4343 %s 0 EOF\n2: POSIX  ADVISORY  READ 7 00:32:1 0 EOF\n3: FLOCK  ADVISORY  WRITE 9 %s:%d 0 EOF\n'
                 % (dev, dev, dev.rsplit(':', 1)[0], st.st_ino+1))
(TMP/'jobs').mkdir()


def exec_probe(script, a, replace=()):
    text = 'ARGS = %r\n' % json.dumps(a) + script
    for old, new in replace:
        text = text.replace(old, new)
    out = io.StringIO()
    with contextlib.redirect_stdout(out):
        exec(compile(text, 'probe', 'exec'), {})
    return json.loads(out.getvalue())


lane = exec_probe(R.CSD_PROBE, dict(helper=str(lock), manifest=str(lock), jobs=str(TMP/'jobs'), q=str(TMP), prefix='p-', chapters=['01-the-bump'],
                                    lock=str(lock), running=True), [("'/proc/locks'", repr(str(locks)))])['lane']
ok('lane: holder and waiter of this inode only', lane == dict(holders=[4242], waiters=[4343]), lane)
pj = TMP/'jobs'/'p-01-the-bump-a1'
pj.mkdir()
f1, f2 = TMP/'row1', TMP/'row2'
f1.write_bytes(b'one')
f2.write_bytes(b'two')
inputs = TMP/'p-inputs.json'
inputs.write_text(json.dumps({'files': [dict(path=str(f1), sha256=hashlib.sha256(b'one').hexdigest()), dict(path=str(f2), sha256=hashlib.sha256(b'two').hexdigest())]}))
isha = hashlib.sha256(inputs.read_bytes()).hexdigest()
(pj/'resource-receipt.json').write_text(json.dumps(dict(finished='t', exit=0, inputs_sha256=isha, final_inputs_sha256=isha)))
(pj/'import-result.json').write_text(json.dumps(dict(manifest_sha256='m'*64, rows350={'x': 1})))
post = exec_probe(R.POST_PROBE, dict(job=str(pj), inputs=str(inputs)))
ok('POST_PROBE on synthetic files: terminal, rows re-hash, inputs sha read',
   post == dict(terminal=True, inputs_sha256=isha, receipt_inputs_sha256=isha, rows=2, distinct=2, changed=[], changed_count=0,
                result_manifest_sha256='m'*64, result_rows350={'x': 1}), post)
f3 = TMP/'row3'
f3.write_bytes(b'TWO')
inputs2 = TMP/'p-inputs2.json'
inputs2.write_text(json.dumps({'files': [dict(path=str(f1), sha256=hashlib.sha256(b'one').hexdigest()), dict(path=str(f3), sha256=hashlib.sha256(b'two').hexdigest()),
                                         dict(path=str(TMP/'absent'), sha256='0'*64)]}))
post = exec_probe(R.POST_PROBE, dict(job=str(pj), inputs=str(inputs2)))
ok('POST_PROBE: a changed row and an absent row are both counted', post['changed_count'] == 2 and post['inputs_sha256'] != post['receipt_inputs_sha256'], post)

# 12. source scan
code = (Q/RELAY).read_text().split('"""', 2)[2]
banned = ('rmtree', 'rmdir', 'os.remove', 'os.rename', 'os.replace', 'unlink', 'write_text', 'write_bytes', 'shutil', 'symlink', '.truncate(',
          "open('w", "open('a", '.kill(', '.terminate(', 'send_signal', "'scp'", "'rsync'")
found = {b: code.count(b) for b in banned if code.count(b)}
ok('no remove, rename, replace, write_text, kill, signal or copy tool', not found, found)
ok("opens only 'x'/'xb' (plus 'rb' for hashing)", set(re.findall(r"open\('([a-z+]+)'\)", code)) <= {'x', 'xb', 'rb'}, re.findall(r"open\('([a-z+]+)'\)", code))
ok('two spawn sites (one receive, one send)', code.count('spawn(HOST_') == 2)
ok('371 modes used: check, prepare (CSD); send (CSS); never export or receive directly',
   "run_371(h, 'check'" in code and "run_371(h, 'prepare'" in code and "'send'" in code and "'export'" not in code and "'receive'" not in code)
body = code.split('def verify', 1)[1].split('def main', 1)[0]
ok('verify mode never spawns or prepares', 'spawn' not in body and 'prepare' not in body)

failed = [r for r in results if not r[1]]
for name, good, note in results:
    print('PASS' if good else 'FAIL', name, '' if good else '| %s' % (note,))
print('%d/%d passed; temp %s (%d KB)' % (len(results)-len(failed), len(results), TMP.name,
      sum(p.stat().st_size for p in TMP.rglob('*') if p.is_file())//1024))
sys.exit(1 if failed else 0)
