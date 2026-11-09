"""Root-review relay 485: one guarded build-host import of the CSS 352 final of 08_loop_sums, with frozen
import371. Prepared only; nothing here has run. Relay 394 (371f5d1f, preserved) with 08 added after
14_continuation and before 05_gallery, the route of draft 476 (3e7b59d0) and controller 476, and relay
472's three changes: c binds the exact receive command and helper hash, g the exact path:hash set, and
the stream-loss stop names verify.

Only 08_loop_sums goes through this relay. Root runs it on the controller, from Q. The relay order is
01_the_bump, 04_reading_color, 12_residues, 14_continuation, 08_loop_sums, 05_gallery: 08 is refused
until each of 01, 04, 12 and 14 has a completed 371 import on CSD. It does not wait for 05, which
imports through 448 (relay 472). Each run:
  1. checks the frozen helper (csd-css352-import371.py, ba337aca) here, on CSS and on CSD;
  2. checks root's exported manifest. Root names it and gives its sha; it must sit in Q here,
     in CQ on CSS and in Q on CSD, with that sha on all three. The relay never exports and never
     stages: root's own staging places these files. The manifest's 08 pin must be draft 476's row 08
     (job, worker, worker sha, inputs, permit) with root's permit 474 at 92119d3a, and the attempt's
     import names must be draft 476's: attempt 1, the only attempt controller 476 accepts;
  3. checks on CSS that the chapter's render receipt is a terminal success, and that the receipt
     and summary are the bytes the manifest pins, so the manifest is the export of the finished
     job;
  4. refuses an attempt number at or below any existing 371 attempt of the chapter. Root chooses
     --attempt; the relay never increments it (316 L1: a job without a finished receipt is
     investigated first);
  5. waits until CSD's native lane has no holder and no waiter (/proc/locks on
     D/expanded-native-lane.lock, read only), then runs 371 check and 371 prepare. prepare writes
     the guard inputs with mode 'x'. With --prepared, the relay instead checks the inputs that an
     earlier stopped run of the same attempt wrote;
  6. checks the lane again, then starts the guarded receive on CSD with an empty pipe on its stdin.
     The guard takes the lane lock before it admits the job;
  7. only when the job's receipt shows admission, this exact receive command and a started child,
     starts the CSS send into that pipe. No byte flows while another job holds the lane;
  8. waits for both ends, then runs the post-import validation (validate(), also the read-only
     `verify` mode). All eight conditions must hold, each named in the record when it fails:
       a. 371 check for this attempt exits 0;
       b. it reports this job, and the job folder exists;
       c. the job's guard receipt is a terminal success (371 check and a direct read agree), it
          records this exact 371 receive command, and 371 check reports the pinned ba337aca;
       d. import-result.json exists and names this manifest's sha;
       e. 371 check found every manifest row (foreign, shared, system) at its size and hash through
          its original path, as 350 reads it (problems []);
       f. the guard inputs file is the one the guard admitted (its sha equals the receipt's
          inputs_sha256);
       g. every row of that inputs file re-hashes, as 350's imported() re-hashes it, and the
          inputs hold exactly the shared + system + 4 frozen rows that 371 prepare writes, path for
          path at the manifest's and the pins' hashes (the guard's row at the hash prepare read):
          the set --prepared checks;
       h. rows350 from 371 check and from import-result.json both equal the expected names.
     Only then does it print rows350. The master controller must not use 08's rows350 without a
     passing validation report for that attempt and manifest sha.
Admission is never retried and a receive is started at most once per run. Any transport failure
(ssh exit 255 or no answer in time) stops the run at once and names what is known. A stop after
the receive started leaves that attempt to root's investigation; `verify` then answers whether
it imported, without starting anything.

Writes: one new folder here per run, relay485-08-loop-sums-a<N>-run<K>/ (the first unused K; mkdir
refuses an existing one), holding record.json and the two ends' logs, all opened with 'x'. A run
refused before it started anything leaves only its record, and the attempt number stays free. On CSD, only what 371 prepare and the
guarded job write. On CSS, nothing.
Exit: 0 imported (or status ready, or verify passed); 1 refused before anything started;
2 transport stop; 3 the import was started and did not complete or validate (verify: not
validated); 4 the lane stayed busy until the deadline.
"""
import argparse, datetime, hashlib, importlib.util, json, shlex, subprocess, sys, time
from pathlib import Path

VERSION = '485'
HERE = Path(__file__).resolve().parent
HELPER = 'csd-css352-import371.py'
HELPER_SHA256 = 'ba337aca143316748561c4300d5774386387690b49052e72d29a04b103dd244f'
ORDER = ('01_the_bump', '04_reading_color', '12_residues', '14_continuation', '08_loop_sums', '05_gallery')
CHAPTER = '08_loop_sums'                                  # the only chapter this relay imports
DRAFT476 = 'cc1-master447-manifest-draft476.json'         # its row 08 is the route this relay imports
DRAFT476_SHA = '3e7b59d0cfb843cd3cd9741f72746139ee39359975d6bd99bb1c69e50cd3b457'
PERMIT474 = '08_loop_sums-css-permit474.json'             # root's host-only permit, as the CSS 08 job rendered it
PERMIT474_SHA = '92119d3a65fd5b3637f839ca83fb60eb455661d296337a1e1dc02cf777d0e59a'
ROW_PIN = ('job', 'worker', 'worker_sha256', 'inputs', 'permit')
HOST_CSD, HOST_CSS = 'csd', 'css'                         # ssh aliases on the controller
SSH = ('ssh', '-o', 'BatchMode=yes', '-o', 'ConnectTimeout=30', '-o', 'ServerAliveInterval=30', '-o', 'ServerAliveCountMax=10')
PROBE_SECONDS = 300                                       # small read-only probes
CHECK_SECONDS = 3600                                      # 371 check/prepare hash a few GB on CSD
ADMISSION_SECONDS = 1800                                  # the guard hashes the inputs before it saves the receipt
STREAM_SECONDS = 14400 + 600                              # 371's receive timeout plus a margin
POLL_SECONDS = 60
LANE_LOCK = 'expanded-native-lane.lock'                   # guard_csd02's lane: ROOT/expanded-native-lane.lock


class Stop(Exception):
    def __init__(self, code, message):
        super().__init__(message)
        self.code = code


def utc():
    return datetime.datetime.now(datetime.timezone.utc).isoformat()


def sha(path):
    with Path(path).open('rb') as f:
        return hashlib.file_digest(f, 'sha256').hexdigest()


def load_helper():
    path = HERE/HELPER
    if sha(path) != HELPER_SHA256:
        raise Stop(1, 'local %s is not the frozen 371 (%s)' % (HELPER, HELPER_SHA256[:12]))
    spec = importlib.util.spec_from_file_location('import371', path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def remote(host, command, stdin=b'', timeout=PROBE_SECONDS):
    """One remote command. ssh's own failure (exit 255) or no answer in time is a transport stop."""
    try:
        r = subprocess.run(list(SSH)+[host, command], input=stdin, capture_output=True, timeout=timeout)
    except subprocess.TimeoutExpired:
        raise Stop(2, '%s: no answer within %d s; the remote outcome is unknown (it may still run); stopped, nothing retried: %s'
                   % (host, timeout, command[:100]))
    if r.returncode == 255:
        raise Stop(2, '%s: ssh exit 255 (transport); stopped, nothing retried: %s' % (host, r.stderr.decode(errors='replace')[-300:]))
    return r


def spawn(host, command, **kw):
    return subprocess.Popen(list(SSH)+[host, command], **kw)


def probe(host, script, args, timeout=PROBE_SECONDS):
    """A read-only Python probe: the script arrives on stdin, prints one JSON line."""
    r = remote(host, '/usr/bin/python3 -I -B -', ('ARGS = %r\n' % json.dumps(args) + script).encode(), timeout=timeout)
    if r.returncode != 0:
        raise Stop(1, '%s probe failed (exit %d): %s' % (host, r.returncode, r.stderr.decode(errors='replace')[-400:]))
    return json.loads(r.stdout)


PROBE_COMMON = r'''
import hashlib, json, os
from pathlib import Path
A = json.loads(ARGS)
def sha(p):
    try:
        with open(p, 'rb') as f:
            return hashlib.file_digest(f, 'sha256').hexdigest()
    except FileNotFoundError:
        return None
def state(p):
    try:
        return json.loads(Path(p).read_text())
    except (FileNotFoundError, ValueError):
        return None
def terminal(r):
    return (isinstance(r, dict) and bool(r.get('finished')) and r.get('exit') == 0
            and r.get('inputs_sha256') is not None and r.get('inputs_sha256') == r.get('final_inputs_sha256')
            and not any(r.get(k) for k in ('error', 'resource_stop', 'stopped_for_resource_limit')))
'''

CSS_PROBE = PROBE_COMMON + r'''
r = state(A['receipt'])
print(json.dumps(dict(helper=sha(A['helper']), manifest=sha(A['manifest']), receipt=sha(A['receipt']),
                      summary=sha(A['summary']), terminal=terminal(r))))
'''

CSD_PROBE = PROBE_COMMON + r'''
jobs = Path(A['jobs'])
imports = {}
for d in A['chapters']:
    rows = []
    for j in sorted(jobs.glob(A['prefix']+d+'-a*')):
        tail = j.name[len(A['prefix']+d+'-a'):]
        r = state(j/'resource-receipt.json')
        rows.append(dict(job=j.name, attempt=int(tail) if tail.isdigit() else None, finished=bool((r or {}).get('finished')),
                         imported=terminal(r) and (j/'import-result.json').is_file()))
    imports[d] = rows
head = A['prefix']+A['chapters'][-1]+'-a'
inputs = []
for p in sorted(Path(A['q']).glob(head+'*-inputs.json')):
    tail = p.name[len(head):-len('-inputs.json')]
    inputs.append(int(tail) if tail.isdigit() else None)
st = os.stat(A['lock'])
key = (os.major(st.st_dev), os.minor(st.st_dev), st.st_ino)
holders, waiters = [], []
for line in open('/proc/locks'):
    parts = line.split()
    blocked = len(parts) > 1 and parts[1] == '->'
    f = parts[2:] if blocked else parts[1:]
    try:
        maj, mnr, ino = f[4].split(':')
        if (int(maj, 16), int(mnr, 16), int(ino)) == key:
            (waiters if blocked else holders).append(int(f[3]))
    except (IndexError, ValueError):
        continue
running = None
if A['running']:                                          # once per run, not on every lane poll
    running = sorted(j.name for j in jobs.iterdir() if (j/'resource-receipt.json').is_file()
                     and (state(j/'resource-receipt.json') or {}).get('admitted') and not (state(j/'resource-receipt.json') or {}).get('finished'))
print(json.dumps(dict(helper=sha(A['helper']), manifest=sha(A['manifest']), imports=imports, inputs=inputs,
                      lane=dict(holders=holders, waiters=waiters), running=running)))
'''

JOB_PROBE = PROBE_COMMON + r'''
job = Path(A['job'])
r = state(job/'resource-receipt.json')
print(json.dumps(dict(exists=job.is_dir(), receipt=r is not None, admitted=bool((r or {}).get('admitted')),
                      command=[str(c) for c in (r or {}).get('command', [])], pid=(r or {}).get('pid'),
                      finished=(r or {}).get('finished'), exit=(r or {}).get('exit'))))
'''

INPUTS_PROBE = PROBE_COMMON + r'''
print(json.dumps(dict(job=Path(A['job']).exists(), rows=(state(A['inputs']) or {}).get('files'))))
'''

POST_PROBE = PROBE_COMMON + r'''
job = Path(A['job'])
r = state(job/'resource-receipt.json')
result = state(job/'import-result.json')
listed = (state(A['inputs']) or {}).get('files') or []
changed = [x.get('path') for x in listed if not isinstance(x.get('path'), str) or sha(x['path']) != x.get('sha256')]
print(json.dumps(dict(terminal=terminal(r), inputs_sha256=sha(A['inputs']), receipt_inputs_sha256=(r or {}).get('inputs_sha256'),
                      receipt_command=[str(c) for c in (r or {}).get('command') or []],
                      rows=len(listed), distinct=len({x.get('path') for x in listed}), changed=changed[:50], changed_count=len(changed),
                      listed_rows=[[x.get('path'), x.get('sha256')] for x in listed],
                      result_manifest_sha256=(result or {}).get('manifest_sha256'), result_rows350=(result or {}).get('rows350'))))
'''


def lane_free(facts):
    return not facts['lane']['holders'] and not facts['lane']['waiters']


def local_manifest(h, args):
    """Root's manifest in Q here, at the sha root gave, accepted by 371 for this chapter, and bound to draft 476's
    08 route (route476). Returns (manifest, pin)."""
    if args.chapter != CHAPTER:
        raise Stop(1, 'relay %s imports only %s, not %s' % (VERSION, CHAPTER, args.chapter))
    if Path(args.manifest).name != args.manifest or not args.manifest.endswith('.json'):
        raise Stop(1, '--manifest is a bare file name in Q')
    data = (HERE/args.manifest).read_bytes()
    if hashlib.sha256(data).hexdigest() != args.manifest_sha:
        raise Stop(1, 'local manifest is not at the sha root gave: '+args.manifest)
    m = json.loads(data)
    shape = h.manifest_problems(m)
    if shape or h.chapter_of(m) != args.chapter:
        raise Stop(1, 'manifest refused by 371: '+'; '.join(shape or ['not for '+args.chapter]))
    pin = m['chapters'][args.chapter]
    unbound = route476(h, args, pin)
    if unbound:
        raise Stop(1, 'not the draft 476 route for %s: %s' % (CHAPTER, '; '.join(unbound)))
    return m, pin


def route476(h, args, pin):
    """What keeps this manifest and attempt from being draft 476's row 08 with root's permit 474 (empty: bound):
    the pin's job, worker, worker sha, inputs and permit are the row's, the permit is 474 at its sha (and so is
    root's copy here), and the attempt's import names are the row's (attempt 1)."""
    draft = HERE/DRAFT476
    if not (draft.is_file() and sha(draft) == DRAFT476_SHA):
        return ['local %s is missing or not at %s' % (DRAFT476, DRAFT476_SHA[:12])]
    row = next(r for r in json.loads(draft.read_text())['chapters'] if r['chapter'] == CHAPTER)
    out = ["the manifest pin's %s is %r, not draft 476 row 08's %r" % (k, pin[k], row[k]) for k in ROW_PIN if pin[k] != row[k]]
    if Path(pin['permit']).name != PERMIT474 or pin['permit_sha256'] != PERMIT474_SHA:
        out.append("the manifest's permit is not root's %s at %s" % (PERMIT474, PERMIT474_SHA[:12]))
    if not ((HERE/PERMIT474).is_file() and sha(HERE/PERMIT474) == PERMIT474_SHA):
        out.append("root's %s is missing here or not at %s" % (PERMIT474, PERMIT474_SHA[:12]))
    if expected_rows(h, args.chapter, args.attempt) != {CHAPTER: {k: row[k] for k in ('import_receipt', 'import_inputs')}}:
        out.append('attempt %d is not the import draft 476 and controller 476 bind (attempt 1); another attempt needs '
                   'a new draft and controller' % args.attempt)
    return out


def gather(h, args):
    """Every read-only check before anything starts. Returns (manifest, pin, facts, problems)."""
    problems = []

    def need(ok, message):
        if not ok:
            problems.append(message)
        return ok
    m, pin = local_manifest(h, args)
    css = probe(HOST_CSS, CSS_PROBE, dict(helper=str(h.CQ/HELPER), manifest=str(h.CQ/args.manifest),
                receipt=str(h.CSS/'jobs'/pin['job']/'resource-receipt.json'), summary=str(h.CSS/'jobs'/pin['job']/'render-summary.json')))
    need(css['helper'] == HELPER_SHA256, 'CSS helper copy is not ba337aca: %s' % css['helper'])
    need(css['manifest'] == args.manifest_sha, 'CSS manifest copy is missing or differs: %s' % css['manifest'])
    need(css['terminal'], 'CSS render receipt is not a terminal success: '+pin['job'])
    need((css['receipt'], css['summary']) == (pin['receipt_sha256'], pin['summary_sha256']),
         'the manifest is not the export of the finished job (receipt or summary bytes differ)')
    earlier = ORDER[:ORDER.index(args.chapter)]
    csd = probe(HOST_CSD, CSD_PROBE, dict(helper=str(h.Q/HELPER), manifest=str(h.Q/args.manifest), jobs=str(h.D/'jobs'), q=str(h.Q),
                prefix='csd-css352-import%s-' % h.VERSION, chapters=[h.dashed(c) for c in earlier+(args.chapter,)], lock=str(h.D/LANE_LOCK), running=True))
    need(csd['helper'] == HELPER_SHA256, 'CSD helper copy is not ba337aca: %s' % csd['helper'])
    need(csd['manifest'] == args.manifest_sha, 'CSD manifest copy is missing or differs: %s' % csd['manifest'])
    for c in earlier:
        need(any(j['imported'] for j in csd['imports'][h.dashed(c)]), 'serial order: %s has no completed 371 import yet' % c)
    mine = csd['imports'][h.dashed(args.chapter)]
    done = [j['job'] for j in mine if j['imported']]
    need(not done, 'already imported by %s; nothing to do' % ', '.join(done))
    attempts = [j['attempt'] for j in mine] + csd['inputs']
    if args.prepared:
        need(None not in attempts and args.attempt in attempts and not any(j['attempt'] == args.attempt for j in mine)
             and max(attempts) == args.attempt,
             '--prepared needs attempt %d to be the latest, with inputs and no job: %s' % (args.attempt, attempts))
    else:
        need(all(a is not None and a < args.attempt for a in attempts),
             'attempt %d is not above every existing attempt %s; the relay never re-admits or picks a number' % (args.attempt, sorted(map(str, attempts))))
    return m, pin, dict(css=css, csd=csd), problems


def expected_rows(h, chapter, attempt):
    job = h.job_name(chapter, attempt)
    return {chapter: dict(import_receipt=str(h.D/'jobs'/job/'resource-receipt.json'), import_inputs=str(h.inputs_path(job)))}


def expected_inputs(h, m, pin, args):
    """The rows 371 prepare freezes: every shared and system row, the helper, the manifest, the guard and Q's permit copy."""
    return len({x['path'] for x in m['files'] if x['kind'] in ('shared', 'system')}
               | {str(h.Q/HELPER), str(h.Q/args.manifest), str(h.GUARD), str(h.q_permit(pin))})


def frozen_rows(h, m, pin, args):
    """371 prepare's rows besides the guard's own, as {path: sha256}: every shared and system row at its manifest hash, and the
    helper, the manifest and Q's permit copy at their pins. Both --prepared and validate() g compare against it."""
    want = {x['path']: x['sha256'] for x in m['files'] if x['kind'] in ('shared', 'system')}
    want.update({str(h.Q/HELPER): HELPER_SHA256, str(h.Q/args.manifest): args.manifest_sha, str(h.q_permit(pin)): pin['permit_sha256']})
    return want


def is_frozen_set(rows, h, m, pin, args):
    """rows ({path: sha256}) are exactly 371 prepare's set: frozen_rows() plus the guard's row, at the hash prepare read."""
    return rows.get(str(h.GUARD)) is not None and {k: v for k, v in rows.items() if k != str(h.GUARD)} == frozen_rows(h, m, pin, args)


def pinned_receive(h, args):
    """The exact 371 receive command the guard runs for this attempt (the tail of 371's guard command)."""
    return [str(c) for c in h.receive_command(h.Q/HELPER, args.chapter, args.attempt, h.Q/args.manifest)]


def validate(h, args, m, pin):
    """Post-import validation (docstring step 8, a-h). Read-only. Returns a report; report['failures'] empty means imported."""
    job = h.job_name(args.chapter, args.attempt)
    rows350 = expected_rows(h, args.chapter, args.attempt)
    code, shown, out, err = run_371(h, 'check', args)
    post = probe(HOST_CSD, POST_PROBE, dict(job=str(h.D/'jobs'/job), inputs=str(h.inputs_path(job))), timeout=CHECK_SECONDS)
    shown = shown or {}
    want = expected_inputs(h, m, pin, args)
    listed = {p: s for p, s in post['listed_rows'] if isinstance(p, str)}
    conditions = (
        ('a. 371 check exits 0', code == 0),
        ('b. 371 check reports %s and the job folder exists' % job, shown.get('job') == job and shown.get('job_exists') is True),
        ('c. the guard receipt is a terminal success of the exact 371 receive command, and 371 check reports the pinned %s'
         % HELPER_SHA256[:8],
         shown.get('terminal_success') is True and post['terminal'] is True and post['receipt_command'] == pinned_receive(h, args)
         and shown.get('helper_sha256') == HELPER_SHA256),
        ('d. import-result.json exists and names manifest %s' % args.manifest_sha[:12],
         shown.get('import_result') is True and post['result_manifest_sha256'] == args.manifest_sha),
        ('e. every manifest row is at its size and hash through its original path (371 check problems [])', shown.get('problems') == []),
        ('f. the guard inputs file is the one the guard admitted',
         post['inputs_sha256'] is not None and post['inputs_sha256'] == post['receipt_inputs_sha256']),
        ('g. the guard inputs hold exactly the %d frozen rows at their pinned hashes (the --prepared set) and every row re-hashes' % want,
         post['rows'] == post['distinct'] == want and post['changed_count'] == 0 and is_frozen_set(listed, h, m, pin, args)),
        ('h. rows350 from 371 check and import-result.json equal the expected names',
         shown.get('rows350') == rows350 and post['result_rows350'] == rows350))
    problems = shown.get('problems')
    return dict(validator='relay%s validate a-h' % VERSION, chapter=args.chapter, attempt=args.attempt, job=job, manifest=args.manifest,
                manifest_sha256=args.manifest_sha, manifest_rows=len(m['files']), inputs_rows=post['rows'], inputs_rows_expected=want,
                inputs_changed=post['changed'], inputs_changed_count=post['changed_count'], check_exit=code,
                check_problems=problems[:50] if isinstance(problems, list) else problems,
                check_problem_count=len(problems) if isinstance(problems, list) else None, check_stderr=err[-2000:],
                passed=[name for name, ok in conditions if ok], failures=[name for name, ok in conditions if not ok],
                rows350=rows350, utc=utc())


def run_371(h, mode, args):
    command = shlex.join(['/usr/bin/python3', '-I', '-B', str(h.Q/HELPER), mode, '--chapter', args.chapter,
                          '--attempt', str(args.attempt), '--manifest', str(h.Q/args.manifest)])
    r = remote(HOST_CSD, command, timeout=CHECK_SECONDS)
    out = r.stdout.decode(errors='replace')
    try:
        shown = json.loads(out)
    except ValueError:
        shown = None
    return r.returncode, shown, out, r.stderr.decode(errors='replace')


def check_prepared(h, m, pin, args):
    """--prepared: the inputs an earlier stopped run of this attempt wrote are exactly 371 prepare's set."""
    job = h.job_name(args.chapter, args.attempt)
    got = probe(HOST_CSD, INPUTS_PROBE, dict(job=str(h.D/'jobs'/job), inputs=str(h.inputs_path(job))))
    if got['job'] or not got['rows']:
        raise Stop(1, '--prepared: the job exists or the inputs are unreadable: '+job)
    rows = {x['path']: x['sha256'] for x in got['rows']}
    if not is_frozen_set(rows, h, m, pin, args):
        raise Stop(1, '--prepared: the existing inputs are not the set 371 prepare writes for this manifest')
    return dict(job=job, inputs=str(h.inputs_path(job)), guard_sha256=rows[str(h.GUARD)])


def wait_lane(h, args, deadline, record):
    while True:
        facts = probe(HOST_CSD, CSD_PROBE, dict(helper=str(h.Q/HELPER), manifest=str(h.Q/args.manifest), jobs=str(h.D/'jobs'), q=str(h.Q),
                      prefix='csd-css352-import%s-' % h.VERSION, chapters=[h.dashed(args.chapter)], lock=str(h.D/LANE_LOCK), running=False))
        record['lane'].append(dict(utc=utc(), lane=facts['lane']))
        if lane_free(facts):
            return
        if time.monotonic() > deadline:
            raise Stop(4, 'the native lane stayed busy until the deadline: %s; nothing started' % facts['lane'])
        time.sleep(POLL_SECONDS)


def run(args):
    h = load_helper()
    base = (args.record_dir or HERE)/('relay%s-%s-a%d-run' % (VERSION, h.dashed(args.chapter), args.attempt))
    folder = Path(str(base)+str(next(k for k in range(1, 1000) if not Path(str(base)+str(k)).exists())))
    folder.mkdir()                                       # refuses an existing folder; earlier runs' records are kept
    record = dict(version=VERSION, chapter=args.chapter, attempt=args.attempt, manifest=args.manifest, manifest_sha256=args.manifest_sha,
                  started=utc(), steps=[], lane=[], status=None)
    try:
        m, pin, facts, problems = gather(h, args)
        record.update(chapter_pin=pin, facts=facts)
        if problems:
            raise Stop(1, '; '.join(problems))
        deadline = time.monotonic() + args.lane_wait_minutes*60
        wait_lane(h, args, deadline, record)
        rows350 = expected_rows(h, args.chapter, args.attempt)
        guard_command = h.guard_command(h.Q/HELPER, args.chapter, args.attempt, h.Q/args.manifest)
        code, shown, out, err = run_371(h, 'check', args)
        record['steps'].append(dict(step='check', utc=utc(), exit=code, report=shown, stderr=err[-2000:]))
        if code != 0 or not shown or shown.get('problems') != [] or shown.get('job_exists'):
            raise Stop(1, '371 check refused: %s' % ((shown or {}).get('problems') or err[-400:]))
        if args.prepared:
            record['steps'].append(dict(step='prepared', utc=utc(), **check_prepared(h, m, pin, args)))
        else:
            code, shown, out, err = run_371(h, 'prepare', args)
            record['steps'].append(dict(step='prepare', utc=utc(), exit=code, report=shown, stderr=err[-2000:]))
            if code != 0 or not shown:
                raise Stop(1, '371 prepare refused: '+err[-400:])
            if (shown['job'], shown['inputs'], shown['manifest_sha256'], shown['guard_command'], shown['rows350'], shown['chapter_pin']) != \
                    (h.job_name(args.chapter, args.attempt), str(h.inputs_path(h.job_name(args.chapter, args.attempt))), args.manifest_sha,
                     guard_command, rows350, pin):
                raise Stop(1, '371 prepare printed another job, inputs, command, rows or pin; nothing started')
        wait_lane(h, args, deadline, record)
        started = start(h, args, guard_command, folder, record)
        record['status'] = finish(h, args, m, pin, started, record)
        return 0
    except Stop as stop:
        record['status'] = dict(code=stop.code, message=str(stop))
        print('STOP %d: %s' % (stop.code, stop), file=sys.stderr)
        return stop.code
    except Exception as error:                           # recorded, then the traceback; nothing is retried
        record['status'] = dict(code=None, message='unexpected %s: %s' % (type(error).__name__, error))
        raise
    finally:
        record['ended'] = utc()
        with (folder/'record.json').open('x') as f:
            json.dump(record, f, indent=1)


def start(h, args, guard_command, folder, record):
    """The receive first, with an empty pipe; the send only after this job's admission."""
    job = h.job_name(args.chapter, args.attempt)
    logs = {name: (folder/name).open('xb') for name in ('receive.stdout', 'receive.stderr', 'send.stderr')}
    receiver = spawn(HOST_CSD, shlex.join(guard_command), stdin=subprocess.PIPE, stdout=logs['receive.stdout'], stderr=logs['receive.stderr'])
    record['steps'].append(dict(step='receive started', utc=utc(), command=guard_command))
    receive = pinned_receive(h, args)
    deadline = time.monotonic() + ADMISSION_SECONDS
    while True:
        try:
            state = probe(HOST_CSD, JOB_PROBE, dict(job=str(h.D/'jobs'/job)))
        except Stop as stop:
            receiver.stdin.close()                      # no byte was sent; the job, if admitted, reads EOF and stops
            raise Stop(stop.code, 'receive started, admission unknown: %s; no byte sent; attempt %d is root\'s to investigate' % (stop, args.attempt))
        record['steps'].append(dict(step='admission poll', utc=utc(), **{k: state[k] for k in ('exists', 'admitted', 'pid', 'finished', 'exit')}))
        if state['admitted'] and state['pid'] and not state['finished'] and state['command'] == receive:
            break
        if state['finished'] or receiver.poll() is not None or time.monotonic() > deadline:
            receiver.stdin.close()
            try:
                receiver.wait(timeout=600)
            except subprocess.TimeoutExpired:
                pass
            raise Stop(3, 'the job was not admitted for this receive (finished %s, guard exit %s); no byte sent; attempt %d is root\'s to investigate'
                       % (state['finished'], receiver.poll(), args.attempt))
        time.sleep(15)
    sender = spawn(HOST_CSS, shlex.join(['/usr/bin/python3', '-I', '-B', str(h.CQ/HELPER), 'send', '--chapter', args.chapter,
                   '--manifest', str(h.CQ/args.manifest), '--manifest-sha', args.manifest_sha]), stdin=subprocess.DEVNULL,
                   stdout=receiver.stdin, stderr=logs['send.stderr'])
    receiver.stdin.close()                              # the sender's copy carries the stream; EOF when it ends
    record['steps'].append(dict(step='send started', utc=utc()))
    return dict(receiver=receiver, sender=sender, logs=logs)


def finish(h, args, m, pin, started, record):
    receiver, sender = started['receiver'], started['sender']
    try:
        sender.wait(timeout=STREAM_SECONDS)
        receiver.wait(timeout=STREAM_SECONDS)
    except subprocess.TimeoutExpired:
        raise Stop(3, 'the stream did not end within %d s; nothing killed here; attempt %d is root\'s to investigate' % (STREAM_SECONDS, args.attempt))
    for f in started['logs'].values():
        f.close()
    record['steps'].append(dict(step='stream ended', utc=utc(), send_exit=sender.returncode, receive_exit=receiver.returncode))
    if 255 in (sender.returncode, receiver.returncode):
        raise Stop(2, 'transport failure during the stream (send %d, receive %d); attempt %d is root\'s to investigate; '
                   'run verify --attempt %d when both hosts answer' % (sender.returncode, receiver.returncode, args.attempt, args.attempt))
    if (sender.returncode, receiver.returncode) != (0, 0):
        raise Stop(3, 'the import did not complete (send %d, receive/guard %d; 75 = resource stop); attempt %d is root\'s to investigate'
                   % (sender.returncode, receiver.returncode, args.attempt))
    try:
        report = validate(h, args, m, pin)
    except Stop as stop:
        raise Stop(stop.code, 'both ends exited 0, but the post-import validation did not complete: %s; nothing is used until '
                   '`verify` for attempt %d passes' % (stop, args.attempt))
    record['steps'].append(dict(step='post-import validation', **report))
    if report['failures']:
        raise Stop(3, 'post-import validation failed: %s; attempt %d is root\'s to investigate' % ('; '.join(report['failures']), args.attempt))
    print(json.dumps(dict(imported=args.chapter, validation=report), indent=1))
    return dict(code=0, message='imported and validated', rows350=report['rows350'], validation_passed=report['passed'])


def status(args):
    """Read-only: everything run checks before it starts, plus 371 check. Writes nothing anywhere."""
    h = load_helper()
    try:
        m, pin, facts, problems = gather(h, args)
        code, shown, out, err = run_371(h, 'check', args)
    except Stop as stop:
        print('STOP %d: %s' % (stop.code, stop), file=sys.stderr)
        return stop.code
    if code != 0 or not shown or shown.get('problems'):
        problems.append('371 check: %s' % ((shown or {}).get('problems') or err[-400:]))
    print(json.dumps(dict(chapter=args.chapter, attempt=args.attempt, problems=problems, lane=facts['csd']['lane'],
                          lane_free=lane_free(facts['csd']), running=facts['csd']['running'], imports=facts['csd']['imports'],
                          check=shown, rows350=expected_rows(h, args.chapter, args.attempt)), indent=1))
    return 0 if not problems else 1


def verify(args):
    """Read-only post-import validation of one attempt (a-h), for controller 476 and after any stop. Writes nothing anywhere."""
    h = load_helper()
    try:
        m, pin = local_manifest(h, args)
        report = validate(h, args, m, pin)
    except Stop as stop:
        print('STOP %d: %s' % (stop.code, stop), file=sys.stderr)
        return stop.code
    print(json.dumps(report, indent=1))
    return 0 if not report['failures'] else 3


def main():
    p = argparse.ArgumentParser(description=__doc__.split('\n')[0])
    p.add_argument('mode', choices=('status', 'run', 'verify'))
    p.add_argument('--chapter', choices=(CHAPTER,), required=True)
    p.add_argument('--attempt', type=int, required=True, choices=range(1, 100), metavar='N')
    p.add_argument('--manifest', required=True, help="root's exported manifest, a bare file name in Q")
    p.add_argument('--manifest-sha', required=True)
    p.add_argument('--prepared', action='store_true', help='attempt N was prepared by an earlier stopped run')
    p.add_argument('--lane-wait-minutes', type=int, default=0, help='how long to wait for a free lane (default: do not wait)')
    p.add_argument('--record-dir', type=Path, help='where the new relay folder goes (default: Q)')
    args = p.parse_args()
    return {'status': status, 'run': run, 'verify': verify}[args.mode](args)


if __name__ == '__main__':
    raise SystemExit(main())
