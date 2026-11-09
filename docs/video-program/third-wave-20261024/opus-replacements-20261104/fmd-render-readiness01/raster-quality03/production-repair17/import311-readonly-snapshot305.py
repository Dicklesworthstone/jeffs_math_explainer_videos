"""Guarded CSD import of the four retained finals, version 311. Prepared only; nothing here has run.

Chapters 02_turning_numbers and 03_two_planes were rendered on TRJ, 13_real_integral on CSS and
15_closing on HZ2. cc0-combined-master293.py may retain them only after a completed guarded CSD import
places each final's whole closure at its original absolute path on CSD, on the D/jobs filesystem
(293 imported() and chapter_gate(); cc2-master-review307.md B1). This helper is that import.

One import job per source host (trj: 02 and 03; css: 13; hz2: 15). Each job:
  - is admitted by guard_csd02.py under the current policy (10 GiB admission, 8 GiB stop, 4 GiB
    growth per job, 56 GiB work, 20 GiB target growth) for exactly this receive command;
  - binds this helper, the closure manifest, the guard and every shared D file in its guard inputs;
  - checks every shared D file and every destination before it reads a byte;
  - reads a header line naming the manifest hash, the file count and the byte count;
  - stages every new file inside the job, checks each hash and the end of the stream, then checks the
    closure again from the staged bytes;
  - only then gives the files their names under D/work/imported-<host>-root311 (a second hard link to
    each staged file, so no path is ever replaced, the staged name is kept and both names share the
    same disk blocks) and creates the foreign root path as a symlink to that folder. An identical existing file is kept; a different one, a
    foreign root that is a real directory, or a symlink to anywhere else stops the job before the
    first write;
  - checks every file again through its original path and writes import-result.json once.

The closure of a chapter is the rule of css-final13-export97.py (the render inputs and each of
their rows, the receipt, the render summary and each of its files, everything in the job folder)
plus every file 293 and the retained worker's check() read: the worker, the permit and its evidence,
the receipts check() reads, the inherited delivery inputs, the narration folder, the timeline21
tables, score21 and binary 19. Files under D are never streamed or written. They must already be on
CSD at the hashes the source host recorded.

Modes (root runs each on the host named; none starts a job by itself):
  export   source host, read-only: compute the closure manifest and print it.
  send     source host, read-only: check the manifest's foreign files again, then stream them.
  prepare  CSD: check everything that can be checked before the stream, write one job's guard inputs
           (mode 'x'), print the guard command and the two 293 row fields.
  receive  CSD, inside that guarded job only: the import.
  check    CSD, read-only: what prepare would refuse, or the state of a finished import.
"""
import argparse, datetime, hashlib, json, os, re, stat, sys
from pathlib import Path

VERSION = '311'
MAGIC = b'CSD-RETAINED-IMPORT311'
D = Path('/data/projects/jmev-stroke-probe-20261106')        # CSD
Q = D/'qualification01/production-repair17'
GUARD = D/'guard_csd02.py'
PYTHON = '/usr/bin/python3'
TRJ = Path('/data/projects/jmev-render-worker-20261107')
CSS = Path('/data/projects/jmev-css-render-worker-20261107')
HZ2 = Path('/data/projects/jmev-hz2-render-worker-20261107')
BINARY_19 = 'f4836eeeb4f2465c0d27ca0497f123f0af6bc0db7bc65096098bb20c5abb0f22'
BIN19 = D/'work/bin/production19'
TIMELINE21 = D/'jobs/csd-production-timeline21'
SCORE21 = D/'jobs/csd-production-score21'
STEMS = SCORE21/'score/stems'
BUILD19 = D/'jobs/csd-production-repair-build19'
# CSD files every retained worker's check() reads (trj-final-chapter19-02.py check()).
SHARED_READS = (BIN19, TIMELINE21/'resource-receipt.json', SCORE21/'resource-receipt.json',
                BUILD19/'qualification.json', SCORE21/'score21-summary.json')
KINDS = ('timeline', 'cues', 'cuelog')
SUFFIXES = ('.mp4', '.timeline.tsv', '.cues.tsv', '.cuelog.tsv', '.layout.tsv')
NARRATION_WAVS = 105
POLICY = {'admission_gib': 10, 'stop_free_gib': 8, 'phase_growth_gib': 4, 'work_cap_gib': 56,
          'target_growth_cap_gib': 20}
GIB = 1 << 30
# Every new file keeps two names: the staged one in the job (D/jobs) and the placed one (D/work).
# The disk holds its bytes once, but the guard sums D/jobs and D/work separately, deduplicating hard
# links only within each tree, so the job's measured growth is twice the foreign bytes. The manifest
# check keeps that doubled figure half a GiB under the 4 GiB growth cap.
GROWTH_LIMIT = 4*GIB - GIB//2
FREE_RESERVE = 9*GIB                                          # the 8 GiB stop plus 1 GiB, after the import
HEADER_MAX = 4096
# A stalled sender must not hold CSD's single native lane: the receive ends after four hours
# (about 1.4 GiB per job; root's CSD jobs use the same /usr/bin/timeout wrapper).
TIMEOUT = ('/usr/bin/timeout', '14400')
HEX64 = re.compile('[0-9a-f]{64}')

# The retained workers' host constants: ROOT, NARRATION, the receipts check() reads and the
# inherited delivery inputs run() binds (trj-final-chapter19-02.py, css-final-chapter78.py,
# hz2-final-chapter53.py; the three differ only in these).
HOSTS = {
    'trj': dict(root=TRJ, chapters=('02_turning_numbers', '03_two_planes'),
                checks=(TRJ/'jobs/trj-production19-delivery02/resource-receipt.json',),
                narration=TRJ/'jobs/trj-production19-delivery02/narration',
                delivery_inputs=TRJ/'qualification01/production19/delivery02-inputs.json'),
    'css': dict(root=CSS, chapters=('13_real_integral',),
                checks=(CSS/'jobs/css-runtime-transfer75/resource-receipt.json',
                        CSS/'jobs/css-encoder-stop79/resource-receipt.json',
                        CSS/'jobs/css-production19-delivery76/resource-receipt.json'),
                narration=CSS/'jobs/css-production19-delivery76/narration',
                delivery_inputs=CSS/'qualification01/delivery76-inputs.json'),
    'hz2': dict(root=HZ2, chapters=('15_closing',),
                checks=(HZ2/'jobs/hz2-runtime-transfer47/resource-receipt.json',
                        HZ2/'jobs/hz2-encoder-stop03/resource-receipt.json',
                        HZ2/'jobs/hz2-production19-delivery49/resource-receipt.json'),
                narration=HZ2/'jobs/hz2-production19-delivery49/narration',
                delivery_inputs=HZ2/'qualification01/production19/delivery49-inputs.json'),
}
# worker, job, inputs, permit, permit/movie hashes and frames: 293 RETAINED (the test compares them).
# receipt, summary and inputs hashes: each final's terminal records (final02-terminal167,
# final03-terminal115, final13-terminal200, final15-terminal81; export97 and export56 rows agree).
# evidence: the number of evidence items in each permit.
CHAPTERS = {
    '02_turning_numbers': dict(
        host='trj', worker=TRJ/'qualification01/production19/trj-final-chapter19-02.py',
        worker_sha256='788f75666d687140c6ab67e8f517b6e713f0e885508c258f055c44e8dc09c1c8',
        job='trj-production19-final-02-turning-numbers32',
        inputs=TRJ/'qualification01/production19/final02-32-inputs.json',
        inputs_sha256='668f9ed7b8dfb6eb5d861ea8c88d66efe108406c236d03304fb6919b69056e25',
        permit=TRJ/'qualification01/production19/review02-32/final-render-permit32.json',
        permit_sha256='ccdff19cd315f5e4893b59fdc11e84486e8db15ab6f9fd63ae73535bd7dc5ac2',
        receipt_sha256='fc60ac86e58581a9aec368659436f625ccefdd3080e63d7ae22fb59566898f5d',
        summary_sha256='df3de467042cbb1c638f1413fd5584e76c097707c8516dc67f9854c5addb57ed',
        movie_sha256='bf0891a409f589bd4fc125c038e03dd027ec334436e25cdc129463c659984f07',
        frames=5372, evidence=9),
    '03_two_planes': dict(
        host='trj', worker=TRJ/'qualification01/production19/trj-final-chapter19-02.py',
        worker_sha256='788f75666d687140c6ab67e8f517b6e713f0e885508c258f055c44e8dc09c1c8',
        job='trj-production19-final-03-two-planes28',
        inputs=TRJ/'qualification01/production19/final03-28-inputs.json',
        inputs_sha256='f89c95b127b0724facc9ec0c64e67aa28afa578c4c34f5c4bc42b0142b71d487',
        permit=TRJ/'qualification01/production19/review03-28/final-render-permit28.json',
        permit_sha256='107aebb15239c67d66569c5e1217c1343c7490901431ef935ce044962e7c64ca',
        receipt_sha256='6f79313524606f33c71467263594cbf15bb4a8192e88369d09c6ed07d03b6e60',
        summary_sha256='04cb015c059aaf1da21536eae104fe591408bfad47c630d8fd49be4019417af8',
        movie_sha256='20353d01ee564ef101ecad80bdfa8ec1db87ac2188b2e4642969b371f2e82cf3',
        frames=7249, evidence=5),
    '13_real_integral': dict(
        host='css', worker=CSS/'qualification01/css-final-chapter78.py',
        worker_sha256='8edccab96593344242386d4ae08399d9cff96da588b00bd3c7f63b0fd1b4accb',
        job='css-production19-final-13-real-integral96',
        inputs=CSS/'qualification01/final13-96-inputs.json',
        inputs_sha256='4681e535235a2648af57be80829f54f7057f090c3d91fe1288b32e83e9e143fe',
        permit=CSS/'qualification01/review13-96/final-render-permit96.json',
        permit_sha256='e86b5d886b6adf15ff2e682308b19b2284f0c00a6229e4dced6994b70290f8e2',
        receipt_sha256='d67279d1de3599af8f2471d104991d987a04c9e24ed5f8bc017ed3278ce942bc',
        summary_sha256='392ebd1ce9e5bc2d36c958fc8cd4ae78261f9f6cf422b4137d981a5931062fdc',
        movie_sha256='49934034d4611b81155925e667dab463072b21ba0240467447b9ddfb52d32f51',
        frames=10641, evidence=6),
    '15_closing': dict(
        host='hz2', worker=HZ2/'qualification01/production19/hz2-final-chapter53.py',
        worker_sha256='20f5c3a94ddada0afd5150c2ac81a028c789bf3dcd33a7f3608a5a8d4f618fac',
        job='hz2-production19-final-15-closing53',
        inputs=HZ2/'qualification01/production19/final15-53-inputs.json',
        inputs_sha256='416a2c8e11f1b6b2560a27740cde9d04e0d3f1e7d53de2a26b3704e1059449d3',
        permit=HZ2/'qualification01/production19/review15-53/final-render-permit53.json',
        permit_sha256='ec74b68375331470ac18c2c7e5db4fc60bb82a247e2eeead844233c453fecd20',
        receipt_sha256='41ce29dbca82b739389b38b6fa1e7d1be3930bcc40d7dc6c7e12a0e9d0b3c6cb',
        summary_sha256='c767d8ed446e77aa99a5c34d03ab68384eda38771075338cbab022bc32c0ac82',
        movie_sha256='3e2440a8785319cf9477e73303e6391c8a4165c7fbfd3fa6f4ad97e81f874fd4',
        frames=4379, evidence=12),
}
ROW_KEYS = {'path', 'sha256', 'bytes', 'mode', 'kind'}


class Unavailable(Exception):
    """A reader has no bytes for this path. Only the source-level tests use it; export and receive
    read every closure file and treat any unavailable one as a refusal."""


def sha(path):
    with Path(path).open('rb') as f:
        return hashlib.file_digest(f, 'sha256').hexdigest()


def sha_bytes(data):
    return hashlib.sha256(data).hexdigest()


def read(path):
    return json.loads(Path(path).read_text())


def utc():
    return datetime.datetime.now(datetime.timezone.utc).isoformat()


def terminal(r):
    """293 success(): finished, exit 0, inputs unchanged, no error or resource stop."""
    return (isinstance(r, dict) and bool(r.get('finished')) and r.get('exit') == 0
            and r.get('inputs_sha256') is not None and r.get('inputs_sha256') == r.get('final_inputs_sha256')
            and not any(r.get(k) for k in ('error', 'resource_stop', 'stopped_for_resource_limit')))


def flag(command, name):
    values = [command[i+1] for i, item in enumerate(command[:-1]) if item == name]
    return values[0] if len(values) == 1 else None


def physical_root(host):
    return D/'work'/('imported-%s-root%s' % (host, VERSION))


def job_name(host, attempt):
    return 'csd-retained-import%s-%s-a%d' % (VERSION, host, attempt)


def inputs_path(job):
    return Q/(job+'-inputs.json')


def physical(path, host):
    return physical_root(host)/Path(path).relative_to(HOSTS[host]['root'])


def receive_command(helper, host, attempt, manifest):
    return list(TIMEOUT)+[PYTHON, '-I', '-B', str(helper), 'receive', '--host', host, '--attempt', str(attempt),
                          '--manifest', str(manifest)]


def guard_command(helper, host, attempt, manifest):
    job = job_name(host, attempt)
    return [PYTHON, '-I', '-B', str(GUARD), '--phase', 'native', '--job', job, '--inputs',
            str(inputs_path(job)), '--'] + receive_command(helper, host, attempt, manifest)


def pins(host):
    return {c: {k: str(v) if isinstance(v, Path) else v for k, v in CHAPTERS[c].items()}
            for c in HOSTS[host]['chapters']}


def kind_of(path, host):
    p = Path(path)
    if p.is_relative_to(D):
        return 'shared'
    if p.is_relative_to(HOSTS[host]['root']):
        return 'foreign'
    return None


def clean_path(s):
    """An absolute path string in normal form: no '..', '.', empty part or control character."""
    return (isinstance(s, str) and s.startswith('/') and str(Path(s)) == s and '//' not in s
            and '..' not in Path(s).parts and not any(ord(c) < 32 for c in s))


def own_paths(p):
    """Paths this version creates: its physical roots, its import jobs and their inputs."""
    if any(p.is_relative_to(physical_root(h)) for h in HOSTS):
        return True
    if p.is_relative_to(D/'jobs') and p != D/'jobs' and p.relative_to(D/'jobs').parts[0].startswith('csd-retained-import'+VERSION):
        return True
    return p.parent == Q and p.name.startswith('csd-retained-import'+VERSION)


def manifest_from_rows(host, rows):
    files = [dict(path=x['path'], sha256=x['sha256'], bytes=x['bytes'], mode=x['mode'], kind=kind_of(x['path'], host))
             for x in rows]
    foreign = [x for x in files if x['kind'] == 'foreign']
    shared = [x for x in files if x['kind'] == 'shared']
    return dict(version=VERSION, host=host, root=str(HOSTS[host]['root']), chapters=pins(host), files=files,
                foreign_files=len(foreign), foreign_bytes=sum(x['bytes'] for x in foreign),
                shared_files=len(shared), shared_bytes=sum(x['bytes'] for x in shared))


# ---------------------------------------------------------------- checks shared by every mode

def manifest_problems(m):
    """Host-independent shape of a 311 manifest. An empty list means the manifest may be used."""
    out = []

    def need(ok, message):
        if not ok:
            out.append(message)
        return ok
    if not need(isinstance(m, dict) and m.get('version') == VERSION, 'not a version-311 manifest'):
        return out
    host = m.get('host')
    if not need(host in HOSTS, 'unknown host %r' % (host,)):
        return out
    root = HOSTS[host]['root']
    need(m.get('root') == str(root), 'root is not '+str(root))
    need(m.get('chapters') == pins(host), 'chapter pins differ from the 293 retained pins')
    files = m.get('files')
    if not need(isinstance(files, list) and files, 'no files'):
        return out
    seen, counts = set(), {'foreign': [0, 0], 'shared': [0, 0]}
    for x in files:
        if not need(isinstance(x, dict) and set(x) == ROW_KEYS, 'malformed row %r' % (x,)):
            continue
        p = x['path']
        if not need(clean_path(p), 'not a clean absolute path: %r' % (p,)):
            continue
        need(p not in seen, 'duplicate row '+p)
        seen.add(p)
        kind = kind_of(p, host)
        need(kind is not None, 'row outside %s and D: %s' % (root, p))
        need(Path(p) not in (root, D), 'row names a root itself: '+p)
        # A D row is never streamed or written: it must be marked shared.
        need(x['kind'] == kind, 'row kind %r is not %r: %s' % (x['kind'], kind, p))
        need(not own_paths(Path(p)), 'row inside a 311 import: '+p)
        need(isinstance(x['sha256'], str) and HEX64.fullmatch(x['sha256']), 'bad sha256: '+p)
        need(type(x['bytes']) is int and x['bytes'] >= 0, 'bad size: '+p)
        need(type(x['mode']) is int and 0 <= x['mode'] <= 0o777 and x['mode'] & 0o400, 'bad mode: '+p)
        if x['kind'] in counts and type(x['bytes']) is int:
            counts[x['kind']][0] += 1
            counts[x['kind']][1] += x['bytes']
    parents = {str(q) for p in seen for q in Path(p).parents}
    for p in sorted(seen & parents):
        out.append('a row is also a directory of another row: '+p)
    need((m.get('foreign_files'), m.get('foreign_bytes'), m.get('shared_files'), m.get('shared_bytes')) ==
         (counts['foreign'][0], counts['foreign'][1], counts['shared'][0], counts['shared'][1]), 'totals differ from the rows')
    need(counts['foreign'][0] > 0, 'nothing to import')
    need(2*counts['foreign'][1] <= GROWTH_LIMIT,
         'import of %d bytes (measured growth twice that) exceeds the per-job growth limit of %d' % (counts['foreign'][1], GROWTH_LIMIT))
    return out


def closure_problems(m, reader):
    """Every file 293 and the retained workers read for this host's finals is in the manifest, at
    the hash the finals' own records give. reader(path) returns the bytes of a closure file.
    Returns (problems, unavailable paths)."""
    host = m['host']
    spec = HOSTS[host]
    root = spec['root']
    rows = {x['path']: x for x in m['files']}
    problems, unavailable = [], []

    def need(ok, message):
        if not ok:
            problems.append(message)
        return ok

    def held(path, digest=None, what='file'):
        x = rows.get(str(path))
        if x is None:
            problems.append('closure lacks %s: %s' % (what, path))
            return False
        if digest is not None and x['sha256'] != digest:
            problems.append('%s is %s in the manifest but %s in its record: %s' % (what, x['sha256'][:12], digest[:12], path))
            return False
        return True

    def load(path, what):
        if not held(path, what=what):
            return None
        try:
            data = reader(Path(path))
        except Unavailable:
            unavailable.append(str(path))
            return None
        if sha_bytes(data) != rows[str(path)]['sha256']:
            problems.append('bytes of %s differ from the manifest: %s' % (what, path))
            return None
        return data

    def record(path, what):
        data = load(path, what)
        if data is None:
            return None
        try:
            return json.loads(data)
        except ValueError:
            problems.append(what+' is not JSON: '+str(path))
            return None

    # CSD files every retained worker's check() reads; they stay CSD's own.
    for path in SHARED_READS:
        held(path, what='shared check() read')
    if str(BIN19) in rows:
        need(rows[str(BIN19)]['sha256'] == BINARY_19, 'binary 19 row is not '+BINARY_19[:12])
    score = record(SCORE21/'score21-summary.json', 'shared check() read')
    if score is not None:
        for relative, digest in score['files'].items():
            held(SCORE21/'score'/relative, digest, 'score21 file')
    for path in (TIMELINE21/'resource-receipt.json', SCORE21/'resource-receipt.json'):
        state = record(path, 'shared check() read')
        need(state is None or terminal(state), 'not a terminal success: '+str(path))
    # Host files check() and run() read.
    states = {}
    for path in spec['checks']:
        states[path] = record(path, 'check() receipt')
        need(states[path] is None or terminal(states[path]), 'not a terminal success: '+str(path))
    delivery = states[spec['checks'][-1]]
    if held(spec['delivery_inputs'], what='inherited delivery inputs') and delivery is not None:
        need(delivery['inputs_sha256'] == rows[str(spec['delivery_inputs'])]['sha256'],
             'delivery receipt does not bind its inputs: '+str(spec['delivery_inputs']))
    inherited = record(spec['delivery_inputs'], 'inherited delivery inputs')
    if inherited is not None:
        for x in inherited['files']:
            held(x['path'], x['sha256'], 'inherited delivery input')
    wavs = sorted(p for p in rows if Path(p).parent == spec['narration'] and p.endswith('.wav'))
    need(len(wavs) == NARRATION_WAVS, 'narration folder holds %d WAVs, not %d: %s' % (len(wavs), NARRATION_WAVS, spec['narration']))
    for ch in spec['chapters']:
        pin = CHAPTERS[ch]
        job = root/'jobs'/pin['job']
        render = job/'render'
        held(pin['worker'], pin['worker_sha256'], 'worker')
        held(pin['permit'], pin['permit_sha256'], 'permit')
        held(pin['inputs'], pin['inputs_sha256'], 'render inputs')
        held(job/'resource-receipt.json', pin['receipt_sha256'], 'render receipt')
        held(job/'render-summary.json', pin['summary_sha256'], 'render summary')
        for kind in KINDS:
            held(TIMELINE21/'timeline-output'/(ch+'.'+kind+'.tsv'), what='timeline21 table')
        state = record(job/'resource-receipt.json', 'render receipt')
        if state is not None:
            command = [str(c) for c in state.get('command', [])]
            need(terminal(state) and state['inputs_sha256'] == pin['inputs_sha256'], 'render receipt is not the pinned terminal success: '+ch)
            need((state.get('policy') or {}).get('stop_free_gib') == 8 and (state.get('policy') or {}).get('phase_growth_gib') == 4,
                 'render receipt policy is not 8/4: '+ch)
            need(str(pin['worker']) in command and (flag(command, '--chapter'), flag(command, '--job'), flag(command, '--inputs'),
                 flag(command, '--permit')) == (ch, pin['job'], str(pin['inputs']), str(pin['permit'])),
                 'render command does not name the pinned worker, job, inputs and permit: '+ch)
        frozen = record(pin['inputs'], 'render inputs')
        if frozen is not None:
            listed = frozen['files']
            bound = {x['path']: x['sha256'] for x in listed}
            need(len(bound) == len(listed), 'duplicate render input: '+ch)
            for path, digest in bound.items():
                held(path, digest, 'render input')
            for path, digest in ((pin['worker'], pin['worker_sha256']), (pin['permit'], pin['permit_sha256'])):
                need(bound.get(str(path)) == digest, 'render inputs do not bind %s: %s' % (path, ch))
            for path in list(spec['checks']) + [spec['delivery_inputs'], BIN19] + wavs:
                need(str(path) in bound, 'render inputs do not bind %s: %s' % (path, ch))
        summary = record(job/'render-summary.json', 'render summary')
        if summary is not None:
            need((summary.get('chapter'), summary.get('frames'), summary.get('binary_sha256'), summary.get('permit_sha256')) ==
                 (ch, pin['frames'], BINARY_19, pin['permit_sha256']), 'render summary does not match the pins: '+ch)
            outputs = {x['path']: x['sha256'] for x in summary['files']}
            for path, digest in outputs.items():
                need(Path(path).is_relative_to(render), 'render summary file outside render/: '+path)
                held(path, digest, 'render output')
            need(outputs.get(str(render/(ch+'.mp4'))) == pin['movie_sha256'], 'movie is not the pinned final: '+ch)
            for suffix in SUFFIXES:
                need(str(render/(ch+suffix)) in outputs, 'render summary lacks '+ch+suffix)
            need(any(Path(p).parent == render/'soundtrack'/ch and Path(p).name.startswith('soundtrack_') for p in outputs),
                 'render summary lacks a soundtrack capture: '+ch)
        review = record(pin['permit'], 'permit')
        if review is not None:
            need((review.get('chapter'), review.get('frames'), review.get('binary_sha256')) == (ch, pin['frames'], BINARY_19),
                 'permit does not match the pins: '+ch)
            table = rows.get(str(TIMELINE21/'timeline-output'/(ch+'.timeline.tsv')))
            need(table is not None and review.get('timeline_sha256') == table['sha256'], 'permit timeline is not timeline21: '+ch)
            evidence = review.get('evidence') or []
            need(len(evidence) == pin['evidence'], 'permit lists %d evidence items, not %d: %s' % (len(evidence), pin['evidence'], ch))
            for item in evidence:
                need(Path(item['path']).is_relative_to(root), 'permit evidence outside %s: %s' % (root, item['path']))
                held(item['path'], item['sha256'], 'permit evidence')
        timeline = load(render/(ch+'.timeline.tsv'), 'render timeline')
        if timeline is not None:
            text = timeline.decode()
            lines = [row.split('\t') for row in text.splitlines() if row.startswith('line\t')]
            need(lines, 'render timeline has no narration lines: '+ch)
            for row in lines:
                if need(len(row) > 4 and Path(row[4]).parent == spec['narration'], 'timeline line outside the worker narration: '+ch):
                    held(row[4], what='narration WAV')
            top = dict(row.split('\t', 1) for row in text.splitlines() if row.startswith(('chapter\t', 'fps\t', 'frames\t')))
            need(top == {'chapter': ch, 'fps': '60', 'frames': str(pin['frames'])}, 'render timeline header: '+ch)
        cuelog = load(render/(ch+'.cuelog.tsv'), 'render cuelog')
        if cuelog is not None:
            for line in cuelog.decode().splitlines():
                if not line.strip() or line.startswith('#'):
                    continue
                row = line.split('\t')
                if need(len(row) > 1 and Path(row[0]).parent in (spec['narration'], STEMS), 'cuelog row outside narration and stems: '+row[0]):
                    held(row[0], row[1], 'cuelog source')
    return list(dict.fromkeys(problems)), unavailable


def plan_problems(m):
    """Every CSD check made before the first write: shared D files, the alias and each destination.
    Returns (plan, problems); plan holds (row, physical destination, new) for the foreign rows in
    stream order."""
    host = m['host']
    root, top = HOSTS[host]['root'], physical_root(host)
    plan, problems = [], []

    def need(ok, message):
        if not ok:
            problems.append(message)
        return ok
    device = os.stat(D/'jobs').st_dev
    work = D/'work'
    need(work.is_dir() and not work.is_symlink() and os.stat(work).st_dev == device,
         'D/work must be a real directory on the D/jobs filesystem')
    if root.is_symlink():
        need(os.readlink(root) == str(top) and top.is_dir(), 'foreign root is a symlink to somewhere else; never replaced: '+str(root))
    elif root.exists():
        problems.append('foreign root exists as a real path; never replaced: '+str(root))
    else:
        need(root.parent.is_dir() and os.access(root.parent, os.W_OK), 'cannot create the alias '+str(root))
    if top.is_symlink() or (top.exists() and not top.is_dir()):
        problems.append('physical root is not a directory: '+str(top))
    elif top.exists():
        need(top.resolve() == top and os.stat(top).st_dev == device, 'physical root is not a real directory on D/jobs\'s filesystem')
    for x in m['files']:
        p = Path(x['path'])
        if x['kind'] == 'shared':
            ok = (p.is_file() and not p.is_symlink() and p.resolve() == p and p.stat().st_size == x['bytes']
                  and sha(p) == x['sha256'])
            need(ok, 'shared CSD file missing or different: '+str(p))
            continue
        dest = physical(p, host)
        bad = [d for d in reversed(dest.parents) if d.is_relative_to(top) and (d.is_symlink() or (d.exists() and not d.is_dir()))]
        if not need(not bad, 'not a real directory: '+str(bad[0]) if bad else ''):
            continue
        if dest.is_symlink() or (dest.exists() and not dest.is_file()):
            problems.append('destination is not a regular file: '+str(dest))
            continue
        if dest.exists():
            need(dest.stat().st_size == x['bytes'] and sha(dest) == x['sha256'], 'existing destination differs; never overwritten: '+str(dest))
            plan.append((x, dest, False))
        else:
            plan.append((x, dest, True))
    return plan, problems


def header_fields(m, digest):
    return dict(version=VERSION, host=m['host'], manifest_sha256=digest, files=m['foreign_files'], bytes=m['foreign_bytes'])


def header_line(m, digest):
    return MAGIC+b' '+json.dumps(header_fields(m, digest), sort_keys=True, separators=(',', ':')).encode()+b'\n'


def read_header(stream, expected):
    line = stream.readline(HEADER_MAX+1)
    assert line.endswith(b'\n') and len(line) <= HEADER_MAX, 'stream does not start with a header line'
    magic, _, body = line[:-1].partition(b' ')
    assert magic == MAGIC, 'not a 311 import stream'
    assert json.loads(body) == expected, 'stream header names another manifest, host, count or size'


def stage(stream, plan, incoming):
    """Phase 1: every foreign file from the stream into this job. Nothing outside the job is written."""
    sources = {}
    for i, (x, dest, new) in enumerate(plan):
        target = incoming/('%05d' % i)
        remaining, h = x['bytes'], hashlib.sha256()
        with (target.open('xb') if new else open(os.devnull, 'wb')) as f:
            while remaining:
                b = stream.read(min(remaining, 1 << 20))
                assert b, 'truncated stream at '+x['path']
                f.write(b)
                h.update(b)
                remaining -= len(b)
        assert h.hexdigest() == x['sha256'], 'stream bytes differ from the manifest: '+x['path']
        sources[x['path']] = target if new else dest
    assert not stream.read(1), 'unexpected data after the last file'
    return sources


def make_dirs(path, top):
    for d in reversed([path]+list(path.parents)):
        if not d.is_relative_to(top) or d == top:
            continue
        assert not d.is_symlink(), 'symlink inside the physical root: '+str(d)
        if not d.exists():
            d.mkdir(mode=0o700)
        assert d.is_dir(), str(d)


def place(plan, sources, host):
    """Phase 2: give every staged file its destination name as a second hard link. os.link refuses an
    existing path, so nothing is replaced; the staged name stays in the job, so nothing is removed."""
    top = physical_root(host)
    device = os.stat(D/'jobs').st_dev
    if not top.exists():
        top.mkdir(mode=0o700)
    assert top.is_dir() and not top.is_symlink() and os.stat(top).st_dev == device
    rows = []
    for x, dest, new in plan:
        if new:
            make_dirs(dest.parent, top)
            os.link(sources[x['path']], dest)
            os.chmod(dest, x['mode'])
            assert os.path.samefile(sources[x['path']], dest), str(dest)
        st = os.lstat(dest)
        assert stat.S_ISREG(st.st_mode) and st.st_size == x['bytes'] and st.st_dev == device, str(dest)
        rows.append(dict(path=x['path'], physical=str(dest), staged=str(sources[x['path']]) if new else None,
                         sha256=x['sha256'], bytes=x['bytes'], mode=x['mode'], written=new))
    return rows


def make_alias(host):
    root, top = HOSTS[host]['root'], physical_root(host)
    if root.is_symlink():
        assert os.readlink(root) == str(top), 'foreign root is a symlink to somewhere else; never replaced'
        return False
    assert not root.exists(), 'foreign root appeared during the import; never replaced'
    root.symlink_to(top, target_is_directory=True)
    assert root.resolve(strict=True) == top
    return True


def earlier_attempts(host, attempt):
    """Refuse a duplicate: an earlier attempt for this host that is running or already imported."""
    problems = []
    for n in range(1, attempt):
        job = D/'jobs'/job_name(host, n)
        if not job.exists():
            continue
        receipt = job/'resource-receipt.json'
        state = read(receipt) if receipt.exists() else {}
        if not state.get('finished'):
            problems.append('earlier attempt still running or unrecorded: '+str(job))
        elif terminal(state) and (job/'import-result.json').exists():
            problems.append('already imported by '+str(job))
    return problems


def csd_side():
    helper = Path(__file__).resolve()
    assert D.is_dir() and not D.is_symlink(), 'CSD modes run on CSD only'
    assert helper.parent == Q, 'the helper must run from '+str(Q)
    return helper


def q_manifest(path):
    path = Path(path)
    assert path.is_absolute() and path.parent == Q and path == path.resolve(strict=True) and path.is_file(), \
        'the manifest must be a symlink-free file directly in '+str(Q)
    return path


# ---------------------------------------------------------------- source host (read-only)

def export(args):
    host = args.host
    spec = HOSTS[host]
    root = spec['root']
    assert root.is_dir() and not root.is_symlink() and root == root.resolve(), 'export runs on the source host itself: '+str(root)

    def walk(folder):
        found = []
        for parent, dirs, files in os.walk(folder, followlinks=False):
            for name in dirs+files:
                p = Path(parent)/name
                assert not p.is_symlink(), 'symlink in the closure: '+str(p)
                if name in files:
                    found.append(p)
        return found
    paths = set(SHARED_READS) | set(spec['checks']) | {spec['delivery_inputs']}
    rule97 = set()
    paths.update(Path(x['path']) for x in read(spec['delivery_inputs'])['files'])
    paths.update(walk(spec['narration']))
    paths.update(SCORE21/'score'/r for r in read(SCORE21/'score21-summary.json')['files'])
    for ch in spec['chapters']:
        pin = CHAPTERS[ch]
        job = root/'jobs'/pin['job']
        rule = {pin['inputs'], job/'resource-receipt.json', job/'render-summary.json'}
        rule.update(Path(x['path']) for x in read(pin['inputs'])['files'])
        rule.update(Path(x['path']) for x in read(job/'render-summary.json')['files'])
        rule.update(walk(job))
        rule97 |= rule
        paths |= rule | {pin['worker'], pin['permit']}
        paths.update(Path(x['path']) for x in read(pin['permit'])['evidence'])
        paths.update(TIMELINE21/'timeline-output'/(ch+'.'+k+'.tsv') for k in KINDS)
    rows = []
    for p in sorted(paths):
        assert clean_path(str(p)) and kind_of(p, host), 'outside %s and D: %s' % (root, p)
        assert p.is_file() and not p.is_symlink(), 'not a regular file: '+str(p)
        if kind_of(p, host) == 'foreign':
            assert p.resolve() == p, 'symlink on the path of '+str(p)
        st = p.stat()
        rows.append(dict(path=str(p), sha256=sha(p), bytes=st.st_size, mode=st.st_mode & 0o777))
    m = manifest_from_rows(host, rows)
    problems = manifest_problems(m)
    more, unavailable = closure_problems(m, lambda p: Path(p).read_bytes())
    problems += more + ['unavailable: '+p for p in unavailable]
    if problems:
        print('\n'.join(['export refused:']+problems), file=sys.stderr)
        return 1
    m.update(utc=utc(), beyond_rule97=sorted(str(p) for p in paths - rule97),
             scope='Closure manifest only; no import has happened. D rows stay CSD files and are never streamed.')
    print(json.dumps(m, indent=1))
    return 0


def send(args):
    data = Path(args.manifest).read_bytes()
    assert sha_bytes(data) == args.manifest_sha, 'manifest is not the one named'
    m = json.loads(data)
    problems = manifest_problems(m)
    assert not problems, '\n'.join(problems)
    assert m['host'] == args.host
    root = HOSTS[args.host]['root']
    assert root.is_dir() and not root.is_symlink(), 'send runs on the source host itself'
    foreign = [x for x in m['files'] if x['kind'] == 'foreign']
    for x in foreign:                                     # every file, before the first byte
        p = Path(x['path'])
        assert p.is_file() and not p.is_symlink() and p.resolve() == p, str(p)
        assert p.stat().st_size == x['bytes'] and sha(p) == x['sha256'], 'changed since export: '+str(p)
    out = sys.stdout.buffer
    out.write(header_line(m, args.manifest_sha))
    for x in foreign:
        h, n = hashlib.sha256(), 0
        with Path(x['path']).open('rb') as f:
            while b := f.read(1 << 20):
                h.update(b)
                n += len(b)
                out.write(b)
        assert n == x['bytes'] and h.hexdigest() == x['sha256'], 'changed while sending: '+x['path']
    out.flush()
    return 0


# ---------------------------------------------------------------- CSD

def preflight(host, attempt, manifest):
    """Everything prepare and check refuse on. Returns (m, digest, plan, problems)."""
    job = job_name(host, attempt)
    problems = earlier_attempts(host, attempt)
    data = manifest.read_bytes()
    digest = sha_bytes(data)
    m = json.loads(data)
    problems += manifest_problems(m)
    if not (isinstance(m, dict) and m.get('host') == host):
        problems.append('manifest is not for host '+host)
    if problems:
        return m, digest, [], problems
    plan, more = plan_problems(m)
    problems += more
    s = os.statvfs(D)
    free = s.f_bavail*s.f_frsize
    new_bytes = sum(x['bytes'] for x, _, new in plan if new)
    if free < POLICY['admission_gib']*GIB or free-new_bytes < FREE_RESERVE:
        problems.append('free space %d leaves under %d after %d new bytes' % (free, FREE_RESERVE, new_bytes))
    return m, digest, plan, problems


def prepare(args):
    helper = csd_side()
    manifest = q_manifest(args.manifest)
    job = job_name(args.host, args.attempt)
    inputs = inputs_path(job)
    problems = []
    if (D/'jobs'/job).exists() or (D/'jobs'/job).is_symlink():
        problems.append('job exists: '+job)
    if inputs.exists() or inputs.is_symlink():
        problems.append('inputs exist: '+str(inputs))
    m, digest, plan, more = preflight(args.host, args.attempt, manifest)
    problems += more
    if problems:
        print('\n'.join(['prepare refused:']+problems), file=sys.stderr)
        return 1
    shared = sorted(x['path'] for x in m['files'] if x['kind'] == 'shared')
    files = [str(helper), str(manifest), str(GUARD)]+shared
    rows = [dict(path=p, sha256=sha(p)) for p in sorted(set(files))]
    with inputs.open('x') as f:
        json.dump({'files': rows}, f, indent=2)
    print(json.dumps(dict(job=job, inputs=str(inputs), inputs_sha256=sha(inputs), manifest_sha256=digest,
        new_files=sum(new for _, _, new in plan), new_bytes=sum(x['bytes'] for x, _, new in plan if new),
        existing_identical=sum(not new for _, _, new in plan), shared_files=len(shared),
        guard_command=guard_command(helper, args.host, args.attempt, manifest),
        rows293={ch: dict(import_receipt=str(D/'jobs'/job/'resource-receipt.json'), import_inputs=str(inputs))
                 for ch in HOSTS[args.host]['chapters']}), indent=1))
    return 0


def receive(args):
    helper = csd_side()
    manifest = q_manifest(args.manifest)
    host = args.host
    job = D/'jobs'/job_name(host, args.attempt)
    inputs = inputs_path(job.name)
    state = read(job/'resource-receipt.json')
    assert state.get('admitted') is True and 'exit' not in state and not state.get('finished'), 'not inside an admitted guard job'
    assert {k: (state.get('policy') or {}).get(k) for k in POLICY} == POLICY, 'guard policy is not the current CSD policy'
    assert [str(c) for c in state['command']] == receive_command(helper, host, args.attempt, manifest), 'job admitted for another command'
    assert sha(inputs) == state['inputs_sha256'], 'guard inputs changed'
    listed = read(inputs)['files']
    bound = {x['path']: x['sha256'] for x in listed}
    assert len(bound) == len(listed)
    for path, digest in bound.items():
        assert sha(path) == digest, 'frozen input changed: '+path
    for path in (helper, manifest, GUARD):
        assert bound.get(str(path)) == sha(path), 'not frozen: '+str(path)
    data = manifest.read_bytes()
    digest = sha_bytes(data)
    m = json.loads(data)
    problems = manifest_problems(m)
    assert not problems and m['host'] == host, '\n'.join(problems)
    shared = {x['path']: x for x in m['files'] if x['kind'] == 'shared'}
    assert all(bound.get(p) == x['sha256'] for p, x in shared.items()), 'shared rows are not frozen at their hashes'
    plan, problems = plan_problems(m)
    assert not problems, '\n'.join(problems)
    read_header(sys.stdin.buffer, header_fields(m, digest))
    incoming = job/'incoming'
    incoming.mkdir(mode=0o700)
    sources = stage(sys.stdin.buffer, plan, incoming)

    def staged(path):
        source = sources.get(str(path)) or (Path(path) if str(path) in shared else None)
        if source is None:
            raise Unavailable(str(path))
        return source.read_bytes()
    problems, unavailable = closure_problems(m, staged)
    assert not problems and not unavailable, '\n'.join(['closure refused from the staged bytes:']+problems+unavailable)
    rows = place(plan, sources, host)
    alias_created = make_alias(host)
    device = os.stat(D/'jobs').st_dev
    for x in m['files']:                                  # through the original paths, as 293 reads them
        p = Path(x['path'])
        assert p.resolve(strict=True) == (physical(p, host) if x['kind'] == 'foreign' else p), str(p)
        assert p.stat().st_size == x['bytes'] and p.stat().st_dev == device and sha(p) == x['sha256'], str(p)
    result = dict(version=VERSION, host=host, job=job.name, manifest=str(manifest), manifest_sha256=digest,
        foreign_root=str(HOSTS[host]['root']), physical_root=str(physical_root(host)), alias_created=alias_created,
        chapters=m['chapters'], written=sum(r['written'] for r in rows), existing_identical=sum(not r['written'] for r in rows),
        foreign_bytes=m['foreign_bytes'], shared_checked=len(shared), files=rows, finished=utc(),
        rows293={ch: dict(import_receipt=str(job/'resource-receipt.json'), import_inputs=str(inputs)) for ch in m['chapters']},
        scope='Exact import only; no render, master, permit, picture, motion, listening or AV acceptance.')
    with (job/'import-result.json').open('x') as f:
        json.dump(result, f, indent=1)
    print('Imported', len(rows), 'files;', result['written'], 'written,', result['existing_identical'], 'already identical;',
          len(shared), 'shared D files checked')
    return 0


def check(args):
    helper = csd_side()
    manifest = q_manifest(args.manifest)
    job = D/'jobs'/job_name(args.host, args.attempt)
    report = dict(helper=str(helper), helper_sha256=sha(helper), job=job.name, job_exists=job.exists())
    receipt = job/'resource-receipt.json'
    if receipt.exists():
        state = read(receipt)
        done = terminal(state) and (job/'import-result.json').exists()
        report.update(finished=bool(state.get('finished')), terminal_success=terminal(state), import_result=(job/'import-result.json').exists())
        if done:
            m = read(manifest)
            result = read(job/'import-result.json')
            problems = [] if result['manifest_sha256'] == sha(manifest) else ['import-result names another manifest']
            for x in m['files']:
                p = Path(x['path'])
                if not (p.is_file() and p.stat().st_size == x['bytes'] and sha(p) == x['sha256']):
                    problems.append('not at its hash on CSD: '+str(p))
            report.update(problems=problems, rows293=result['rows293'])
            print(json.dumps(report, indent=1))
            return 0 if not problems else 1
    m, digest, plan, problems = preflight(args.host, args.attempt, manifest)
    report.update(manifest_sha256=digest, problems=problems, new_files=sum(new for _, _, new in plan),
                  new_bytes=sum(x['bytes'] for x, _, new in plan if new), existing_identical=sum(not new for _, _, new in plan),
                  alias=str(HOSTS[args.host]['root']), alias_state=('symlink to '+os.readlink(HOSTS[args.host]['root'])
                  if HOSTS[args.host]['root'].is_symlink() else 'real path' if HOSTS[args.host]['root'].exists() else 'absent'))
    print(json.dumps(report, indent=1))
    return 0 if not problems else 1


def main():
    p = argparse.ArgumentParser(description=__doc__.split('\n')[0])
    sub = p.add_subparsers(dest='mode', required=True)
    s = sub.add_parser('export')
    s.add_argument('--host', choices=HOSTS, required=True)
    s = sub.add_parser('send')
    s.add_argument('--host', choices=HOSTS, required=True)
    s.add_argument('--manifest', type=Path, required=True)
    s.add_argument('--manifest-sha', required=True)
    for name in ('prepare', 'receive', 'check'):
        s = sub.add_parser(name)
        s.add_argument('--host', choices=HOSTS, required=True)
        s.add_argument('--attempt', type=int, default=1, choices=range(1, 100), metavar='N')
        s.add_argument('--manifest', type=Path, required=True)
    args = p.parse_args()
    return {'export': export, 'send': send, 'prepare': prepare, 'receive': receive, 'check': check}[args.mode](args)


if __name__ == '__main__':
    raise SystemExit(main())
