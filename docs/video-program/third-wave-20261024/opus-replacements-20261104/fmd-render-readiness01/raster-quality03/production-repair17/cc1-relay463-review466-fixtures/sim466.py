"""task466 (CC1): a two-host scratch world for relay 463 and import 448. Fixture only; never an actual import.

- 448 runs as a "twin": its pinned bytes (b6653757) sit at the CSD Q path, and launch448_466.py executes that
  text with only its four root literals (D, CSS, RECOVERY, DATA) rewritten into this world's CSD namespace and
  __file__ set to the genuine-bytes path. Reversing the rewrite reproduces b6653757; nothing else changes.
- 463 runs unchanged (d1822a51) in-process. Only module attributes are rebound: HERE (the controller's Q),
  load_helper (returns the in-process 448 twin) and subprocess (a shim for ssh). The shim runs each probe with
  463's own probe text on this machine. On 'css' it maps the CSD namespace to the CSS namespace in ARGS; on
  'csd' it maps /proc/locks to a scratch file. It runs 448 check/prepare through the launcher. Popen of the
  guarded receive emulates the guard's admission (job folder plus admitted receipt), then starts
  receiver466.py, which writes a scripted outcome. Popen of the CSS send starts a process that exits 0.
- Hosts: S/csd/... is CSD (CSS and RECOVERY are exact alias symlinks into D/work, as 448 requires). S/css/...
  is CSS (RECOVERY is a real folder holding the 442 job, the 448 copy and the manifest copy).
"""
import argparse
import datetime
import hashlib
import json
import os
import shlex
import subprocess as _sp
import sys
import types
from pathlib import Path

H448 = 'b6653757c5c114029a94832a0dd4cb9597c42808614fbb959ef6448a5d994f6e'
H463 = 'd1822a5157895ae19c7128a1de374dbafa7de204aeba8c99b3895fabe4d5b892'
LIT = {'D': "Path('/data/projects/jmev-stroke-probe-20261106')", 'CSS': "Path('/data/projects/jmev-css-render-worker-20261107')",
       'RECOVERY': "Path('/data/ssd/work/jmev-gallery-recovery440')", 'DATA': "Path('/data')"}
HERE = Path(__file__).resolve().parent
LAUNCHER, RECEIVER = HERE/'launch448_466.py', HERE/'receiver466.py'
CH, J442 = '05_gallery', 'css-reviewed-final-05-gallery352-ssd442'
MANIFEST = 'gallery-manifest466.json'
EARLIER = ('01-the-bump', '04-reading-color', '12-residues', '14-continuation')


def sha(p):
    with open(p, 'rb') as f:
        return hashlib.file_digest(f, 'sha256').hexdigest()


def utc():
    return datetime.datetime.now(datetime.timezone.utc).isoformat()


def put(p, data, mode=0o644):
    p = Path(p)
    p.parent.mkdir(parents=True, exist_ok=True)
    with open(p, 'xb') as f:
        f.write(data if isinstance(data, bytes) else data.encode())
    os.chmod(p, mode)
    return p


def terminal_receipt(command, inputs_sha='0'*64, exit_code=0):
    return dict(started=utc(), phase='native', command=command, admitted=True, inputs_sha256=inputs_sha, pid=4242,
                policy={'admission_gib': 10, 'stop_free_gib': 8, 'phase_growth_gib': 4, 'work_cap_gib': 56, 'target_growth_cap_gib': 20},
                exit=exit_code, final_inputs_sha256=inputs_sha, finished=utc())


class World:
    def __init__(self, base, Q0, earlier=EARLIER, locks=''):
        base.mkdir(parents=True)
        self.S = base.resolve()
        self.csd_data, self.css_data = self.S/'csd/data', self.S/'css/data'
        self.ctl = self.S/'controller'
        self.ctl.mkdir()
        # the twin text: four literals rewritten, checked to reverse to the pinned bytes
        text = (Q0/'csd-gallery-import448.py').read_bytes()
        assert hashlib.sha256(text).hexdigest() == H448
        text = text.decode()
        new = {'D': self.csd_data/'projects/jmev-stroke-probe-20261106', 'CSS': self.csd_data/'projects/jmev-css-render-worker-20261107',
               'RECOVERY': self.csd_data/'ssd/work/jmev-gallery-recovery440', 'DATA': self.csd_data}
        for k, lit in LIT.items():
            assert text.count(lit) == 1, k
            text = text.replace(lit, 'Path(%r)' % str(new[k]))
        back = text
        for k, lit in LIT.items():
            back = back.replace('Path(%r)' % str(new[k]), lit)
        assert hashlib.sha256(back.encode()).hexdigest() == H448
        self.twin = put(self.S/'twin/csd-gallery-import448-twin466.py', text)
        D = new['D']
        Q = D/'qualification01/production-repair17'
        self.helper = put(Q/'csd-gallery-import448.py', (Q0/'csd-gallery-import448.py').read_bytes())
        h = types.ModuleType('import448_twin466')
        h.__file__ = str(self.helper)
        exec(compile(text, str(self.helper), 'exec'), h.__dict__)
        assert (h.D, h.Q, h.RECOVERY) == (D, Q, new['RECOVERY'])
        self.h = h
        # CSD: aliases, lane lock, guard stand-in, shared and system files, earlier 371 imports
        top, top_ssd = D/'work/imported-css-root311', D/'work/imported-css-ssd442'
        top.mkdir(parents=True)
        top_ssd.mkdir(parents=True)
        h.CSS.parent.mkdir(parents=True, exist_ok=True)
        h.RECOVERY.parent.mkdir(parents=True, exist_ok=True)
        os.symlink(str(top), h.CSS)
        os.symlink(str(top_ssd), h.RECOVERY)
        (D/'jobs').mkdir()
        put(D/'expanded-native-lane.lock', b'')
        self.locks = put(self.S/'csd/proc-locks466', locks)
        put(h.GUARD, b'# stand-in for guard_csd02.py (task466); never executed\n')
        shared = [D/'jobs/csd-production-score21/score/stems/stem-a466.wav', D/'jobs/csd-review-repair-build268/qualification466.json',
                  D/'work/bin/review-repair268-stand-in466']
        system = [self.S/'csd/usr/lib/libstand-in466.so']
        for i, p in enumerate(shared + system):
            put(p, 'local row %d (task466)\n' % i)
        self.spare = put(D/'jobs/csd-unrelated466/valid-other-file.bin', b'a valid CSD file that is no manifest row (task466)\n')
        for d in earlier:
            job = D/'jobs'/('csd-css352-import371-%s-a1' % d)
            put(job/'resource-receipt.json', json.dumps(terminal_receipt(['stub-371'])))
            put(job/'import-result.json', '{}')
        # CSS: the 442 job, the 448 copy and the manifest copy under the real RECOVERY folder
        self.css_rec = self.css_data/'ssd/work/jmev-gallery-recovery440'
        (self.css_data/'projects/jmev-css-render-worker-20261107').mkdir(parents=True)
        put(self.css_rec/'csd-gallery-import448.py', (Q0/'csd-gallery-import448.py').read_bytes())
        cj = self.css_rec/'jobs'/J442
        receipt = put(cj/'resource-receipt.json', json.dumps(terminal_receipt(['stub-442'])))
        summary = put(cj/'render-summary.json', json.dumps(dict(frames=12)))
        movie = put(cj/'render/05_gallery.mp4', b'stub gallery movie (task466)\n')
        inputs442 = put(self.css_rec/'gallery-inputs442.json', json.dumps(dict(files=[])))
        permit_bytes = b'{"permit": "stand-in (task466)"}\n'
        pin = dict(job=J442, worker=str(h.WORKER), worker_sha256=h.WORKER_SHA256, inputs=str(h.css_inputs(CH)), inputs_sha256=sha(inputs442),
                   permit=str(h.PERMITS/'05_gallery-css-permit466.json'), permit_sha256=hashlib.sha256(permit_bytes).hexdigest(),
                   receipt_sha256=sha(receipt), summary_sha256=sha(summary), movie_sha256=sha(movie), frames=12)
        rows = []
        for p in (receipt, summary, movie, inputs442):          # foreign: CSS-host files named by their CSD-namespace paths
            st = p.stat()
            rows.append(dict(path=str(h.RECOVERY/p.relative_to(self.css_rec)), sha256=sha(p), bytes=st.st_size, mode=st.st_mode & 0o777))
        for p in shared + system:
            st = p.stat()
            rows.append(dict(path=str(p), sha256=sha(p), bytes=st.st_size, mode=st.st_mode & 0o777))
        m = h.manifest_from_rows({CH: pin}, sorted(rows, key=lambda x: x['path']))
        assert h.manifest_problems(m) == [], h.manifest_problems(m)
        data = json.dumps(m, indent=1).encode()
        self.m, self.pin, self.manifest_sha = m, pin, hashlib.sha256(data).hexdigest()
        for where in (self.ctl, Q, self.css_rec):
            put(where/MANIFEST, data)
        put(h.q_permit(pin), permit_bytes)
        self.foreign = {x['path']: self.css_rec/Path(x['path']).relative_to(h.RECOVERY) for x in m['files'] if x['kind'] == 'foreign'}
        self.children = []

    # ---- 448 on CSD, through the launcher (exact text, rewritten roots)
    def run448(self, mode, attempt=1, stdin=b''):
        argv = [sys.executable, '-I', '-B', str(LAUNCHER), str(self.twin), str(self.helper), mode, '--chapter', CH,
                '--attempt', str(attempt), '--manifest', str(self.h.Q/MANIFEST)]
        return _sp.run(argv, input=stdin, capture_output=True, timeout=600)

    def job(self, attempt):
        return self.h.D/'jobs'/self.h.job_name(CH, attempt)

    def expected_paths(self):
        """463 expected_inputs()'s own set expression (it returns only the size of this set)."""
        h, m, pin = self.h, self.m, self.pin
        return {x['path'] for x in m['files'] if x['kind'] in ('shared', 'system')} | \
            {str(h.Q/'csd-gallery-import448.py'), str(h.Q/MANIFEST), str(h.GUARD), str(h.q_permit(pin))}

    def place_foreign(self):
        """What a completed 448 receive leaves at the physical SSD root (fixture: copies, not the receive)."""
        for path, src in self.foreign.items():
            dst = self.h.physical(path)
            if not dst.exists():
                put(dst, src.read_bytes())

    def import_result(self, attempt):
        h, job = self.h, self.job(attempt)
        return dict(version='448', chapter=CH, job=job.name, manifest=str(h.Q/MANIFEST), manifest_sha256=self.manifest_sha,
                    rows350={CH: dict(import_receipt=str(job/'resource-receipt.json'), import_inputs=str(h.inputs_path(job.name)))},
                    scope='fixture (task466): written by the test, not by 448 receive')


class Shim:
    """Stands in for the `subprocess` module inside 463: ssh argv in, local processes out."""
    PIPE, DEVNULL, STDOUT = _sp.PIPE, _sp.DEVNULL, _sp.STDOUT
    TimeoutExpired, CompletedProcess = _sp.TimeoutExpired, _sp.CompletedProcess
    MARKS = (('CSS_PROBE', "summary=sha(A['summary'])"), ('CSD_PROBE', "/proc/locks"), ('JOB_PROBE', 'exists=job.is_dir()'),
             ('INPUTS_PROBE', "rows=(state(A['inputs'])"), ('POST_PROBE', 'changed_count'))

    def __init__(self, world, relay, faults=None, outcome=('complete', 0)):
        self.w, self.relay, self.faults, self.outcome, self.log = world, relay, {k: list(v) for k, v in (faults or {}).items()}, outcome, []

    def split(self, argv):
        n = len(self.relay.SSH)
        assert tuple(argv[:n]) == self.relay.SSH and len(argv) == n+2, argv
        return argv[n], argv[n+1]

    def run(self, argv, input=b'', capture_output=True, timeout=None):
        host, command = self.split(argv)
        if command == '/usr/bin/python3 -I -B -':
            text = input.decode()
            kind = next(k for k, mark in self.MARKS if mark in text)
        else:
            parts = shlex.split(command)
            kind = 'helper ' + parts[4]
        self.log.append([host, kind])
        fault = self.faults.get(kind, [None]).pop(0) if self.faults.get(kind) else None
        if fault == 'timeout':
            raise _sp.TimeoutExpired(argv, timeout)
        if fault == '255':
            return _sp.CompletedProcess(argv, 255, b'', b'ssh: connect to host: Operation timed out\n')
        if command == '/usr/bin/python3 -I -B -':
            if host == 'css':
                text = text.replace(str(self.w.csd_data), str(self.w.css_data))
            else:
                assert text.count("open('/proc/locks')") <= 1
                text = text.replace("open('/proc/locks')", 'open(%r)' % str(self.w.locks))
            r = _sp.run([sys.executable, '-I', '-B', '-'], input=text.encode(), capture_output=True, timeout=600)
        else:
            assert host == 'csd' and parts[:4] == ['/usr/bin/python3', '-I', '-B', str(self.w.helper)], parts
            r = _sp.run([sys.executable, '-I', '-B', str(LAUNCHER), str(self.w.twin)] + parts[3:], capture_output=True, timeout=600)
        return _sp.CompletedProcess(argv, r.returncode, r.stdout, r.stderr)

    def Popen(self, argv, stdin=None, stdout=None, stderr=None):
        host, command = self.split(argv)
        parts = shlex.split(command)
        if host == 'css':
            self.log.append([host, 'send'])
            assert parts[:5] == ['/usr/bin/python3', '-I', '-B', str(self.w.h.RECOVERY/'csd-gallery-import448.py'), 'send'], parts
            p = _sp.Popen([sys.executable, '-I', '-B', '-c', 'pass'], stdin=stdin, stdout=stdout, stderr=stderr)
        else:
            self.log.append([host, 'receive'])
            h = self.w.h
            assert parts[:4] == ['/usr/bin/python3', '-I', '-B', str(h.GUARD)], parts
            job = h.D/'jobs'/parts[parts.index('--job')+1]
            inputs = Path(parts[parts.index('--inputs')+1])
            receive = parts[parts.index('--')+1:]
            job.mkdir()                                       # the guard's own mkdir, then its admitted receipt
            outcome, code = self.outcome
            r = terminal_receipt(receive, sha(inputs))
            for k in ('exit', 'final_inputs_sha256', 'finished'):
                del r[k]
            if outcome == 'refused':                          # the guard's own refusal: not admitted, finished
                r = dict(started=utc(), phase='native', command=receive, admitted=False, finished=utc())
            (job/'resource-receipt.json').write_text(json.dumps(r, indent=2))
            params = dict(job=str(job), outcome=outcome, exit=code, result=self.w.import_result(int(parts[parts.index('--job')+1].rsplit('-a', 1)[1])),
                          place=[[str(src), str(h.physical(path))] for path, src in self.w.foreign.items()])
            p = _sp.Popen([sys.executable, '-I', '-B', str(RECEIVER), json.dumps(params)], stdin=stdin, stdout=stdout, stderr=stderr)
        self.w.children.append(p)
        return p


def load463(world, Q0, faults=None, outcome=('complete', 0)):
    src = (Q0/'cc3-css-gallery-relay463.py').read_bytes()
    assert hashlib.sha256(src).hexdigest() == H463
    m = types.ModuleType('relay463_fixture466')
    m.__file__ = str(world.ctl/'cc3-css-gallery-relay463.py')
    exec(compile(src, m.__file__, 'exec'), m.__dict__)
    assert m.HERE == world.ctl
    m.subprocess = Shim(world, m, faults, outcome)
    m.load_helper = lambda: world.h
    return m


def args(world, **kw):
    a = dict(chapter=CH, attempt=1, manifest=MANIFEST, manifest_sha=world.manifest_sha, prepared=False, lane_wait_minutes=0, record_dir=None)
    a.update(kw)
    return argparse.Namespace(**a)
