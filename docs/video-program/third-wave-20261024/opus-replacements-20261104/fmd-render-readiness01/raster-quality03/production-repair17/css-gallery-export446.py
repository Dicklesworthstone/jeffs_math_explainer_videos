"""Read-only SSD442 gallery closure exporter derived from371. All import/mutation modes disabled.
Requires actual successful stable render, exact source/score/narration/evidence closure.
The historical371 implementation documentation follows; its import modes are NOT authorized here.
"""
"""Guarded CSD import of one CSS 352 final, version 371. Prepared only; nothing here has run.

cc0-combined-master350.py takes a 352 chapter rendered on CSS (css-final-repair352.py, 1074051b)
only through a completed guarded CSD import. imported() reads the import's receipt and guard inputs.
chapter_gate() reads the CSS job's receipt, inputs, permit, summary and outputs, and re-hashes every
row of its frozen inputs at the original path strings, through the CSS alias on CSD.
csd-retained-import311.py cannot be that import (cc3-css352-import371-notes.md, section 1):
  - it pins 13_real_integral and production19's closure;
  - it refuses a second css job once css-a1 has imported;
  - it refuses the 2,269 CSD runtime rows outside CSS and D that a 352 job binds;
  - it counts every foreign byte against the growth limit, including the 1.4 GB css-a1 placed.
This helper is 311 with those parts changed and the rest kept. It also never creates the CSS
alias: 311 creates it when absent, while 371 refuses unless css-a1's alias and folder are in place.

One import job per chapter (csd-css352-import371-<chapter>-a<N>). Each job:
  - is admitted by guard_csd02.py under the current policy (10 GiB admission, 8 GiB stop, 4 GiB
    growth per job, 56 GiB work, 20 GiB target growth) for exactly this receive command;
  - binds in its guard inputs: this helper, the closure manifest, the guard, root's Q copy of the
    permit, every shared D file and every CSD system file;
  - checks every shared and system file and every destination before it reads a byte, and refuses
    while any csd-reviewed-final-<chapter>* job exists on CSD;
  - reads a header line naming the manifest hash, the file count and the byte count;
  - stages every new file inside the job, checks each hash and the end of the stream, then checks the
    closure again from the staged bytes;
  - only then gives the new files their names under D/work/imported-css-root311. css-a1 created that
    folder, and the CSS alias already names it. Each name is a second hard link to the staged file,
    so no path is ever replaced, the staged name is kept and both names share the same disk blocks.
    An identical existing file is kept. A different one stops the job before the first write. This
    job never creates, replaces or changes the alias or the folder; if either is missing or
    different, the job stops;
  - checks every file again through its original path and writes import-result.json once.

The closure of a chapter is the rule of css-final13-export97.py applied to the 352 job: its
admitted inputs and each of their rows, the receipt, the render summary and each of its files, and
everything in the job folder. The admitted inputs are CSS/qualification01/final-<chapter>-inputs352.json,
the file whose hash its receipt names. 352's launch() froze into those inputs every file its
render() and check() read. Among them are the files 350 reads: the permit's evidence, source
binding and independent files, the delivery309 inputs and narration, the timeline269 tables, score21
and binary 268. Rows are of three kinds:
  foreign  under CSS: streamed, staged, placed.
  shared   under D. On CSS, D is the alias into FROZEN; on CSD these are CSD's own files. Never
           streamed or written: each must already be on CSD, symlink-free, at the bound hash.
  system   outside CSS, D and /data: CSD's runtime under /usr. 352's carried_rows() binds only rows
           of CSD's own timeline269, preview243 and preview274 manifests, at their hashes. Never
           streamed or written: each must be a row of one of those manifests at the same hash, and
           on CSD at that hash.

Modes (root runs each on the host named; none starts a job by itself):
  export   CSS, read-only: compute one chapter's closure manifest and print it. Refuses unless the
           chapter's render receipt is a terminal success.
  send     CSS, read-only: check the manifest's foreign files again, then stream them.
  prepare  CSD: check everything that can be checked before the stream, write one job's guard inputs
           (mode 'x'), print the guard command and the two 350 row fields.
  receive  CSD, inside that guarded job only: the import.
  check    CSD, read-only: what prepare would refuse, or the state of a finished import.
"""
import argparse, datetime, hashlib, json, os, re, stat, sys
from pathlib import Path

VERSION = '446'
MAGIC = b'CSD-CSS352-IMPORT446'
D = Path('/data/projects/jmev-stroke-probe-20261106')        # CSD
Q = D/'qualification01/production-repair17'
GUARD = D/'guard_csd02.py'
PYTHON = '/usr/bin/python3'
CSS = Path('/data/projects/jmev-css-render-worker-20261107')
CQ = CSS/'qualification01'
DATA = Path('/data')
# The CSS alias on CSD and the folder it names. csd-retained-import311-css-a1 created both
# (import-result alias_created true). 371 places files into that folder; it never creates,
# replaces or changes either.
TOP = D/'work/imported-css-root311'
ROOTS311 = tuple(D/'work'/('imported-%s-root311' % h) for h in ('trj', 'css', 'hz2'))
OWN_PREFIXES = ('csd-retained-import311', 'csd-css352-import'+VERSION)
# css-final-repair352.py's own constants (350 WORKERS352 binds this worker at this hash).
RECOVERY = Path('/data/ssd/work/jmev-gallery-recovery440')
WORKER = RECOVERY/'css-gallery-ssd442.py'
WORKER_SHA256 = 'ce9baf6a922682ea5792ccbdb3758b68383e7e8e23e2e968a83d9136f9448429'
PERMITS = CQ/'repair352-permits'
GUARD_CSS = RECOVERY/'guard_css_ssd441.py'
BIN = D/'work/bin/review-repair268'
BINARY_268 = '983954c508e42b90d443ad6ca09f2baba7565a4f609d09d2ea821640534d6ec7'
TIMELINES = D/'jobs/csd-review-repair-timeline269/timeline-output'
FMD_NARRATION = D/'jobs/csd-review-repair-rebind269/narration'
SCORE = D/'jobs/csd-production-score21'
STEMS = SCORE/'score/stems'
BUILD268 = D/'jobs/csd-review-repair-build268'
TRANSFER75 = CSS/'jobs/css-runtime-transfer75'
MANIFEST75 = CQ/'css-runtime-manifest75.json'
ENCODER_STOP = CSS/'jobs/css-encoder-stop79'
TRANSFER = CSS/'jobs/css-repair-transfer309'
DELIVERY = CSS/'jobs/css-repair-delivery309'
NARRATION = DELIVERY/'narration'
DELIVERY_INPUTS = CQ/'css-repair-delivery309-inputs.json'
# The receipts launch() binds and the CSD manifests carried_rows() reads (352 CSD_MANIFESTS).
CHECKS = tuple(p/'resource-receipt.json' for p in (ENCODER_STOP, TRANSFER75, TRANSFER, DELIVERY))
CSD_MANIFESTS = (Q/'review-repair-timeline269-inputs.json', Q/'review-repair-preview243-inputs.json',
                 Q/'review-repair-preview274-inputs.json')
CHAPTERS = ('01_the_bump', '04_reading_color', '05_gallery', '06_one_derivative', '07_one_point', '08_loop_sums',
            '10_rigidity', '11_counting', '12_residues', '14_continuation')
KINDS = ('timeline', 'cues', 'cuelog')
SUFFIXES = ('.mp4', '.timeline.tsv', '.cues.tsv', '.cuelog.tsv', '.layout.tsv')
NARRATION_WAVS = 105
POLICY = {'admission_gib': 10, 'stop_free_gib': 8, 'phase_growth_gib': 4, 'work_cap_gib': 56,
          'target_growth_cap_gib': 20}
GIB = 1 << 30
# Every new file keeps two names: the staged one in the job (D/jobs) and the placed one (D/work).
# The disk holds its bytes once, but the guard sums D/jobs and D/work separately, deduplicating hard
# links only within each tree, so the job's measured growth is twice the new bytes. An identical
# existing file is read from the stream and discarded, so it adds nothing. The CSD check keeps that
# doubled figure half a GiB under the 4 GiB growth cap.
GROWTH_LIMIT = 4*GIB - GIB//2
FREE_RESERVE = 9*GIB                                          # the 8 GiB stop plus 1 GiB, after the import
# The guard's whole-tree limits (admission and stop): D/jobs under 56 GiB, D/work growth since its
# allocation baseline at most 20 GiB. prepare and check keep each half a GiB clear after the import.
WORK_CAP = 56*GIB
TARGET_CAP = 20*GIB
BASELINE = D/'jobs/allocation-baseline.json'
CAP_MARGIN = GIB//2
HEADER_MAX = 4096
# A stalled sender must not hold CSD's single native lane: the receive ends after four hours.
# A first 352 chapter streams about 2.2 GB, 1.4 GB of it already placed by css-a1 and discarded.
TIMEOUT = ('/usr/bin/timeout', '14400')
HEX64 = re.compile('[0-9a-f]{64}')
ROW_KEYS = {'path', 'sha256', 'bytes', 'mode', 'kind'}
PIN_KEYS = {'job', 'worker', 'worker_sha256', 'inputs', 'inputs_sha256', 'permit', 'permit_sha256',
            'receipt_sha256', 'summary_sha256', 'movie_sha256', 'frames'}
COUNTED = ('foreign', 'shared', 'system')


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
    """350 success(): finished, exit 0, inputs unchanged, no error or resource stop."""
    return (isinstance(r, dict) and bool(r.get('finished')) and r.get('exit') == 0
            and r.get('inputs_sha256') is not None and r.get('inputs_sha256') == r.get('final_inputs_sha256')
            and not any(r.get(k) for k in ('error', 'resource_stop', 'stopped_for_resource_limit')))


def flag(command, name):
    values = [command[i+1] for i, item in enumerate(command[:-1]) if item == name]
    return values[0] if len(values) == 1 else None


def dashed(chapter):
    return chapter.replace('_', '-')


def css_job(chapter):
    """352 job_name(): the CSS render job of one chapter."""
    assert chapter == '05_gallery'
    return 'css-reviewed-final-05-gallery352-ssd442'


def css_inputs(chapter):
    """352 launch(): the frozen inputs of that job (the chapter key keeps its underscores)."""
    assert chapter == '05_gallery'
    return RECOVERY/'gallery-inputs442.json'


def job_name(chapter, attempt):
    return 'csd-css352-import%s-%s-a%d' % (VERSION, dashed(chapter), attempt)


def inputs_path(job):
    return Q/(job+'-inputs.json')


def physical(path):
    return TOP/Path(path).relative_to(CSS)


def q_permit(pin):
    """Root's CSD copy of the permit the CSS job rendered from: same name, directly in Q."""
    return Q/Path(pin['permit']).name


def receive_command(helper, chapter, attempt, manifest):
    return list(TIMEOUT)+[PYTHON, '-I', '-B', str(helper), 'receive', '--chapter', chapter, '--attempt', str(attempt),
                          '--manifest', str(manifest)]


def guard_command(helper, chapter, attempt, manifest):
    job = job_name(chapter, attempt)
    return [PYTHON, '-I', '-B', str(GUARD), '--phase', 'native', '--job', job, '--inputs',
            str(inputs_path(job)), '--'] + receive_command(helper, chapter, attempt, manifest)


def kind_of(path):
    p = Path(path)
    if p.is_relative_to(D):
        return 'shared'
    if p.is_relative_to(CSS) or p.is_relative_to(RECOVERY):
        return 'foreign'
    if p.is_relative_to(DATA):
        return None                                           # another project tree: never a closure row
    return 'system'


def clean_path(s):
    """An absolute path string in normal form: no '..', '.', empty part or control character."""
    return (isinstance(s, str) and s.startswith('/') and str(Path(s)) == s and '//' not in s
            and '..' not in Path(s).parts and not any(ord(c) < 32 for c in s))


def own_paths(p):
    """Paths the 311 and 371 imports create: the physical roots, the import jobs and their inputs."""
    if any(p.is_relative_to(r) for r in ROOTS311):
        return True
    if p.is_relative_to(D/'jobs') and p != D/'jobs' and p.relative_to(D/'jobs').parts[0].startswith(OWN_PREFIXES):
        return True
    return p.parent == Q and p.name.startswith(OWN_PREFIXES)


def manifest_from_rows(chapters, rows):
    files = [dict(path=x['path'], sha256=x['sha256'], bytes=x['bytes'], mode=x['mode'], kind=kind_of(x['path']))
             for x in rows]
    out = dict(version=VERSION, host='css', root=str(CSS), chapters=chapters, files=files)
    for kind in COUNTED:
        chosen = [x for x in files if x['kind'] == kind]
        out.update({kind+'_files': len(chosen), kind+'_bytes': sum(x['bytes'] for x in chosen)})
    return out


# ---------------------------------------------------------------- checks shared by every mode

def pin_problems(chapters):
    """The one chapter of a 371 manifest and the names 352 gives its job, inputs, worker and permit."""
    if not (isinstance(chapters, dict) and len(chapters) == 1):
        return ['a 371 manifest names exactly one chapter']
    (ch, pin), = chapters.items()
    if ch not in CHAPTERS:
        return ['not a 352 chapter: %r' % (ch,)]
    if not (isinstance(pin, dict) and set(pin) == PIN_KEYS):
        return ['chapter pin keys are not %s' % sorted(PIN_KEYS)]
    out = []
    expected = dict(job=css_job(ch), worker=str(WORKER), worker_sha256=WORKER_SHA256, inputs=str(css_inputs(ch)))
    for k, v in expected.items():
        if pin[k] != v:
            out.append('chapter pin %s is %r, not %r' % (k, pin[k], v))
    permit = pin['permit']
    if not (clean_path(permit) and Path(permit).parent == PERMITS and Path(permit).name.startswith(ch+'-')
            and permit.endswith('.json')):
        out.append('permit is not a %s file directly in %s: %r' % (ch, PERMITS, permit))
    for k in ('inputs_sha256', 'permit_sha256', 'receipt_sha256', 'summary_sha256', 'movie_sha256'):
        if not (isinstance(pin[k], str) and HEX64.fullmatch(pin[k])):
            out.append('bad %s in the chapter pin' % k)
    if not (type(pin['frames']) is int and pin['frames'] > 0):
        out.append('bad frames in the chapter pin')
    return out


def manifest_problems(m):
    """Host-independent shape of a 371 manifest. An empty list means the manifest may be used."""
    out = []

    def need(ok, message):
        if not ok:
            out.append(message)
        return ok
    if not need(isinstance(m, dict) and m.get('version') == VERSION, 'not a version-371 manifest'):
        return out
    need(m.get('host') == 'css', 'host is not css')
    need(m.get('root') == str(CSS), 'root is not '+str(CSS))
    pins = pin_problems(m.get('chapters'))
    out += pins
    if pins:
        return out
    files = m.get('files')
    if not need(isinstance(files, list) and files, 'no files'):
        return out
    seen, counts = set(), {k: [0, 0] for k in COUNTED}
    for x in files:
        if not need(isinstance(x, dict) and set(x) == ROW_KEYS, 'malformed row %r' % (x,)):
            continue
        p = x['path']
        if not need(clean_path(p), 'not a clean absolute path: %r' % (p,)):
            continue
        need(p not in seen, 'duplicate row '+p)
        seen.add(p)
        kind = kind_of(p)
        need(kind is not None, 'row in another project tree under %s: %s' % (DATA, p))
        need(Path(p) not in (CSS, D), 'row names a root itself: '+p)
        # Only a CSS row is ever streamed or written: a D or system row must say so.
        need(x['kind'] == kind, 'row kind %r is not %r: %s' % (x['kind'], kind, p))
        need(not own_paths(Path(p)), 'row inside a 311 or 371 import: '+p)
        need(isinstance(x['sha256'], str) and HEX64.fullmatch(x['sha256']), 'bad sha256: '+p)
        need(type(x['bytes']) is int and x['bytes'] >= 0, 'bad size: '+p)
        need(type(x['mode']) is int and 0 <= x['mode'] <= 0o777 and x['mode'] & 0o400, 'bad mode: '+p)
        if x['kind'] in counts and type(x['bytes']) is int:
            counts[x['kind']][0] += 1
            counts[x['kind']][1] += x['bytes']
    parents = {str(q) for p in seen for q in Path(p).parents}
    for p in sorted(seen & parents):
        out.append('a row is also a directory of another row: '+p)
    need(all((m.get(k+'_files'), m.get(k+'_bytes')) == tuple(counts[k]) for k in COUNTED), 'totals differ from the rows')
    need(counts['foreign'][0] > 0, 'nothing to import')
    return out


def closure_problems(m, reader):
    """Every file 350 and the 352 worker read for this chapter is in the manifest, at the hash the
    job's own records give. reader(path) returns the bytes of a closure file.
    Returns (problems, unavailable paths)."""
    (ch, pin), = m['chapters'].items()
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

    job = RECOVERY/'jobs'/pin['job']
    render = job/'render'
    held(WORKER, WORKER_SHA256, 'worker')
    held(pin['permit'], pin['permit_sha256'], 'permit')
    held(pin['inputs'], pin['inputs_sha256'], 'render inputs')
    held(job/'resource-receipt.json', pin['receipt_sha256'], 'render receipt')
    held(job/'render-summary.json', pin['summary_sha256'], 'render summary')
    held(BIN, BINARY_268, 'binary 268')
    for kind in KINDS:
        held(TIMELINES/(ch+'.'+kind+'.tsv'), what='timeline269 table')
    # The render receipt: 350 launches on a foreign row once its import receipt exists, so the CSS
    # render must already be a terminal success here.
    state = record(job/'resource-receipt.json', 'render receipt')
    if state is not None:
        command = [str(c) for c in state.get('command', [])]
        need(terminal(state) and state['inputs_sha256'] == pin['inputs_sha256'],
             'render receipt is not a terminal success binding the pinned inputs: '+ch)
        need({k: (state.get('policy') or {}).get(k) for k in POLICY} == POLICY, 'render receipt policy is not 10/8/4/56/20: '+ch)
        need(command[:5] == [PYTHON, '-I', '-B', str(WORKER), 'render'] and (flag(command, '--chapter'), flag(command, '--job'),
             flag(command, '--inputs'), flag(command, '--permit')) == (ch, pin['job'], pin['inputs'], pin['permit']),
             'render command does not name the pinned worker, job, inputs and permit: '+ch)
    # The 97 rule: every row of the admitted inputs, at its frozen hash.
    frozen = record(pin['inputs'], 'render inputs')
    bound = {}
    if frozen is not None:
        listed = frozen['files']
        bound = {x['path']: x['sha256'] for x in listed}
        need(len(bound) == len(listed), 'duplicate render input: '+ch)
        for path, digest in bound.items():
            held(path, digest, 'render input')
        for path, digest in ((WORKER, WORKER_SHA256), (pin['permit'], pin['permit_sha256']), (BIN, BINARY_268)):
            need(bound.get(str(path)) == digest, 'render inputs do not bind %s: %s' % (path, ch))
        for path in (GUARD_CSS, DELIVERY_INPUTS, DELIVERY/'delivery-result.json', TRANSFER/'transfer-result.json', MANIFEST75) \
                + CHECKS + CSD_MANIFESTS:
            need(str(path) in bound, 'render inputs do not bind %s: %s' % (path, ch))
    wavs = sorted(p for p in rows if Path(p).parent == NARRATION and p.endswith('.wav'))
    need(len(wavs) == NARRATION_WAVS, 'narration folder holds %d WAVs, not %d: %s' % (len(wavs), NARRATION_WAVS, NARRATION))
    need(all(p in bound for p in wavs), 'render inputs do not bind every narration WAV: '+ch)
    # The receipts launch() binds, and the delivery309 inputs and result.
    states = {p: record(p, 'bound receipt') for p in CHECKS}
    for p, s in states.items():
        need(s is None or terminal(s), 'not a terminal success: '+str(p))
    delivery = states[DELIVERY/'resource-receipt.json']
    if held(DELIVERY_INPUTS, what='delivery309 inputs') and delivery is not None:
        need(delivery['inputs_sha256'] == rows[str(DELIVERY_INPUTS)]['sha256'], 'delivery receipt does not bind its inputs: '+str(DELIVERY_INPUTS))
    inherited = record(DELIVERY_INPUTS, 'delivery309 inputs')
    if inherited is not None:
        for x in inherited['files']:
            held(x['path'], x['sha256'], 'delivery309 input')
    result = record(DELIVERY/'delivery-result.json', 'delivery309 result')
    if result is not None:
        need((result.get('binary_sha256'), (result.get('delivery') or {}).get('verified_cues'), result.get('source')) ==
             (BINARY_268, NARRATION_WAVS, str(FMD_NARRATION)), 'delivery309 result is not binary 268, 105 cues, rebind269: '+ch)
    # System rows: rows of CSD's own manifests, at the same hash (352 carried_rows()).
    carried = {}
    for path in CSD_MANIFESTS:
        manifest = record(path, 'CSD manifest')
        for x in (manifest or {}).get('files', []):
            carried.setdefault(x['path'], set()).add(x['sha256'])
    for x in m['files']:
        if x['kind'] == 'system':
            need(x['sha256'] in carried.get(x['path'], ()), 'system row is not a CSD manifest row at this hash: '+x['path'])
    summary = record(job/'render-summary.json', 'render summary')
    if summary is not None:
        need((summary.get('host'), summary.get('chapter'), summary.get('frames'), summary.get('binary_sha256'), summary.get('permit_sha256'),
              summary.get('worker352_sha256'), summary.get('narration')) ==
             ('css', ch, pin['frames'], BINARY_268, pin['permit_sha256'], WORKER_SHA256, str(NARRATION)),
             'render summary does not match the pins: '+ch)
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
        need((review.get('chapter'), review.get('host'), review.get('frames'), review.get('binary_sha256')) == (ch, 'css', pin['frames'], BINARY_268),
             'permit does not match the pins: '+ch)
        table = rows.get(str(TIMELINES/(ch+'.timeline.tsv')))
        need(table is not None and review.get('timeline_sha256') == table['sha256'], 'permit timeline is not timeline269: '+ch)
        for key in ('evidence', 'source_binding_evidence', 'independent_review'):
            items = review.get(key) or []
            need(items or key == 'evidence', 'permit lists no %s: %s' % (key, ch))
            for item in items:
                held(item['path'], item['sha256'], 'permit '+key)
        for item in review.get('independent_review') or []:
            need(Path(item['path']).parent == Q, 'independent review file not directly in Q (350 independent_files): '+item['path'])
    timeline = load(render/(ch+'.timeline.tsv'), 'render timeline')
    if timeline is not None:
        text = timeline.decode()
        lines = [row.split('\t') for row in text.splitlines() if row.startswith('line\t')]
        need(lines, 'render timeline has no narration lines: '+ch)
        for row in lines:
            if need(len(row) > 4 and Path(row[4]).parent == NARRATION, 'timeline line outside the delivery309 narration: '+ch):
                held(row[4], what='narration WAV')
        top = dict(row.split('\t', 1) for row in text.splitlines() if row.startswith(('chapter\t', 'fps\t', 'frames\t')))
        need(top == {'chapter': ch, 'fps': '60', 'frames': str(pin['frames'])}, 'render timeline header: '+ch)
    cuelog = load(render/(ch+'.cuelog.tsv'), 'render cuelog')
    if cuelog is not None:
        for line in cuelog.decode().splitlines():
            if not line.strip() or line.startswith('#'):
                continue
            row = line.split('\t')
            if need(len(row) > 1 and Path(row[0]).parent in (NARRATION, STEMS), 'cuelog row outside narration and stems: '+row[0]):
                held(row[0], row[1], 'cuelog source')
    return list(dict.fromkeys(problems)), unavailable


def alias_problems():
    """The CSS alias and its folder exactly as css-a1 left them, on the D/jobs filesystem."""
    device = os.stat(D/'jobs').st_dev
    out = []
    if not (CSS.is_symlink() and os.readlink(CSS) == str(TOP)):
        out.append('the CSS alias is not the symlink to %s that css-a1 created; never created or replaced here: %s' % (TOP, CSS))
    if not (TOP.is_dir() and not TOP.is_symlink() and TOP.resolve() == TOP and os.stat(TOP).st_dev == device):
        out.append('physical root is not a real directory on the D/jobs filesystem; never created here: '+str(TOP))
    return out


def plan_problems(m):
    """Every CSD check made before the first write: the alias, one host per chapter, root's permit
    copy, shared and system files and each destination. Returns (plan, problems); plan holds
    (row, physical destination, new) for the foreign rows in stream order."""
    (ch, pin), = m['chapters'].items()
    plan, problems = [], []

    def need(ok, message):
        if not ok:
            problems.append(message)
        return ok
    device = os.stat(D/'jobs').st_dev
    work = D/'work'
    need(work.is_dir() and not work.is_symlink() and os.stat(work).st_dev == device,
         'D/work must be a real directory on the D/jobs filesystem')
    problems += alias_problems()
    if problems:
        return plan, problems
    # One host per chapter (352 permit_sources, on CSD's real D/jobs).
    others = sorted(p.name for p in (D/'jobs').glob('csd-reviewed-final-'+dashed(ch)+'*'))
    need(not others, 'chapter already has a CSD final job: '+', '.join(others))
    copy = q_permit(pin)
    need(copy.is_file() and not copy.is_symlink() and copy.resolve() == copy and sha(copy) == pin['permit_sha256'],
         "root's Q copy of the permit is missing or differs: "+str(copy))
    for x in m['files']:
        p = Path(x['path'])
        if x['kind'] == 'shared':
            ok = (p.is_file() and not p.is_symlink() and p.resolve() == p and p.stat().st_size == x['bytes']
                  and sha(p) == x['sha256'])
            need(ok, 'shared CSD file missing or different: '+str(p))
            continue
        if x['kind'] == 'system':
            ok = p.is_file() and p.stat().st_size == x['bytes'] and sha(p) == x['sha256']
            need(ok, 'system CSD file missing or different: '+str(p))
            continue
        dest = physical(p)
        bad = [d for d in reversed(dest.parents) if d.is_relative_to(TOP) and (d.is_symlink() or (d.exists() and not d.is_dir()))]
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
    new_bytes = sum(x['bytes'] for x, _, new in plan if new)
    need(2*new_bytes <= GROWTH_LIMIT,
         'import of %d new bytes (measured growth twice that) exceeds the per-job growth limit of %d' % (new_bytes, GROWTH_LIMIT))
    return plan, problems


def header_fields(m, digest):
    (ch,) = m['chapters']
    return dict(version=VERSION, chapter=ch, manifest_sha256=digest, files=m['foreign_files'], bytes=m['foreign_bytes'])


def header_line(m, digest):
    return MAGIC+b' '+json.dumps(header_fields(m, digest), sort_keys=True, separators=(',', ':')).encode()+b'\n'


def read_header(stream, expected):
    line = stream.readline(HEADER_MAX+1)
    assert line.endswith(b'\n') and len(line) <= HEADER_MAX, 'stream does not start with a header line'
    magic, _, body = line[:-1].partition(b' ')
    assert magic == MAGIC, 'not a 371 import stream'
    assert json.loads(body) == expected, 'stream header names another manifest, chapter, count or size'


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


def place(plan, sources):
    """Phase 2: give every staged file its destination name as a second hard link. os.link refuses an
    existing path, so nothing is replaced; the staged name stays in the job, so nothing is removed.
    The physical root must already exist (css-a1's); it is never created here."""
    device = os.stat(D/'jobs').st_dev
    assert TOP.is_dir() and not TOP.is_symlink() and os.stat(TOP).st_dev == device, 'physical root missing: '+str(TOP)
    rows = []
    for x, dest, new in plan:
        if new:
            make_dirs(dest.parent, TOP)
            os.link(sources[x['path']], dest)
            os.chmod(dest, x['mode'])
            assert os.path.samefile(sources[x['path']], dest), str(dest)
        st = os.lstat(dest)
        assert stat.S_ISREG(st.st_mode) and st.st_size == x['bytes'] and st.st_dev == device, str(dest)
        rows.append(dict(path=x['path'], physical=str(dest), staged=str(sources[x['path']]) if new else None,
                         sha256=x['sha256'], bytes=x['bytes'], mode=x['mode'], written=new))
    return rows


def earlier_attempts(chapter, attempt):
    """Refuse a duplicate: an earlier attempt for this chapter that is running or already imported."""
    problems = []
    for n in range(1, attempt):
        job = D/'jobs'/job_name(chapter, n)
        if not job.exists():
            continue
        receipt = job/'resource-receipt.json'
        state = read(receipt) if receipt.exists() else {}
        if not state.get('finished'):
            problems.append('earlier attempt still running or unrecorded: '+str(job))
        elif terminal(state) and (job/'import-result.json').exists():
            problems.append('already imported by '+str(job))
    return problems


def allocated(root):
    """The guard's allocated(): allocated bytes, deduplicating hard links; never follow symlinks."""
    seen = set()
    total = 0
    for parent, dirs, files in os.walk(root, followlinks=False):
        dirs[:] = [d for d in dirs if not (Path(parent) / d).is_symlink()]
        for name in files:
            path = Path(parent) / name
            try:
                stat = path.lstat()
            except FileNotFoundError:
                continue  # Cargo may atomically replace a temporary file.
            if path.is_symlink() or (stat.st_dev, stat.st_ino) in seen:
                continue
            seen.add((stat.st_dev, stat.st_ino))
            total += stat.st_blocks * 512
    return total


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


def chapter_of(m):
    chapters = m.get('chapters') if isinstance(m, dict) else None
    return next(iter(chapters)) if isinstance(chapters, dict) and len(chapters) == 1 else None


# ---------------------------------------------------------------- CSS (read-only)

def export(args):
    ch = args.chapter
    assert CSS.is_dir() and not CSS.is_symlink() and CSS == CSS.resolve(), 'export runs on CSS itself: '+str(CSS)
    job = RECOVERY/'jobs'/css_job(ch)
    receipt = job/'resource-receipt.json'
    state = read(receipt)
    assert terminal(state), 'the render receipt is not a terminal success; nothing to import yet: '+str(receipt)
    permit = flag([str(c) for c in state['command']], '--permit')
    assert permit, 'the render command names no single --permit'

    def walk(folder):
        found = []
        for parent, dirs, files in os.walk(folder, followlinks=False):
            for name in dirs+files:
                p = Path(parent)/name
                assert not p.is_symlink(), 'symlink in the closure: '+str(p)
                if name in files:
                    found.append(p)
        return found
    inputs, summary = css_inputs(ch), job/'render-summary.json'
    pin = dict(job=job.name, worker=str(WORKER), worker_sha256=sha(WORKER), inputs=str(inputs), inputs_sha256=sha(inputs),
               permit=permit, permit_sha256=sha(permit), receipt_sha256=sha(receipt), summary_sha256=sha(summary),
               movie_sha256=sha(job/'render'/(ch+'.mp4')), frames=read(summary)['frames'])
    paths = {inputs, receipt, summary}
    paths.update(Path(x['path']) for x in read(inputs)['files'])
    paths.update(Path(x['path']) for x in read(summary)['files'])
    paths.update(walk(job))
    rows = []
    for p in sorted(paths):
        kind = kind_of(p)
        assert clean_path(str(p)) and kind, 'in another project tree: '+str(p)
        assert p.is_file(), 'not a regular file: '+str(p)
        if kind != 'system':                              # /usr/bin/timeout is a symlink on both hosts
            assert not p.is_symlink(), 'symlink in the closure: '+str(p)
        if kind == 'foreign':
            assert p.resolve() == p, 'symlink on the path of '+str(p)
        st = p.stat()
        rows.append(dict(path=str(p), sha256=sha(p), bytes=st.st_size, mode=st.st_mode & 0o777))
    m = manifest_from_rows({ch: pin}, rows)
    problems = manifest_problems(m)
    if not problems:
        more, unavailable = closure_problems(m, lambda p: Path(p).read_bytes())
        problems += more + ['unavailable: '+p for p in unavailable]
    if problems:
        print('\n'.join(['export refused:']+problems), file=sys.stderr)
        return 1
    m.update(utc=utc(), scope='Closure manifest only; no import has happened. D and system rows stay CSD files and are never streamed.')
    print(json.dumps(m, indent=1))
    return 0


def send(args):
    data = Path(args.manifest).read_bytes()
    assert sha_bytes(data) == args.manifest_sha, 'manifest is not the one named'
    m = json.loads(data)
    problems = manifest_problems(m)
    assert not problems, '\n'.join(problems)
    assert chapter_of(m) == args.chapter, 'manifest is not for chapter '+args.chapter
    assert CSS.is_dir() and not CSS.is_symlink(), 'send runs on CSS itself'
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

def preflight(chapter, attempt, manifest):
    """Everything prepare and check refuse on. Returns (m, digest, plan, problems, space)."""
    problems = earlier_attempts(chapter, attempt)
    data = manifest.read_bytes()
    digest = sha_bytes(data)
    m = json.loads(data)
    problems += manifest_problems(m)
    if chapter_of(m) != chapter:
        problems.append('manifest is not for chapter '+chapter)
    if problems:
        return m, digest, [], problems, {}
    plan, more = plan_problems(m)
    problems += more
    s = os.statvfs(D)
    free = s.f_bavail*s.f_frsize
    new_bytes = sum(x['bytes'] for x, _, new in plan if new)
    if free < POLICY['admission_gib']*GIB or free-new_bytes < FREE_RESERVE:
        problems.append('free space %d leaves under %d after %d new bytes' % (free, FREE_RESERVE, new_bytes))
    jobs_now, work_now = allocated(D/'jobs'), allocated(D/'work')
    baseline = read(BASELINE)['target_bytes'] if BASELINE.exists() else work_now
    space = dict(free_bytes=free, new_bytes=new_bytes, jobs_bytes=jobs_now, work_bytes=work_now, work_baseline_bytes=baseline)
    if jobs_now+new_bytes > WORK_CAP-CAP_MARGIN:
        problems.append('D/jobs would hold %d bytes, within half a GiB of the guard\'s 56 GiB work cap' % (jobs_now+new_bytes))
    if work_now+new_bytes-baseline > TARGET_CAP-CAP_MARGIN:
        problems.append('D/work would grow %d bytes past its baseline, within half a GiB of the guard\'s 20 GiB cap' % (work_now+new_bytes-baseline))
    return m, digest, plan, problems, space


def prepare(args):
    helper = csd_side()
    manifest = q_manifest(args.manifest)
    job = job_name(args.chapter, args.attempt)
    inputs = inputs_path(job)
    problems = []
    if (D/'jobs'/job).exists() or (D/'jobs'/job).is_symlink():
        problems.append('job exists: '+job)
    if inputs.exists() or inputs.is_symlink():
        problems.append('inputs exist: '+str(inputs))
    m, digest, plan, more, space = preflight(args.chapter, args.attempt, manifest)
    problems += more
    if problems:
        print('\n'.join(['prepare refused:']+problems), file=sys.stderr)
        return 1
    pin = m['chapters'][args.chapter]
    local = sorted(x['path'] for x in m['files'] if x['kind'] in ('shared', 'system'))
    files = [str(helper), str(manifest), str(GUARD), str(q_permit(pin))]+local
    rows = [dict(path=p, sha256=sha(p)) for p in sorted(set(files))]
    with inputs.open('x') as f:
        json.dump({'files': rows}, f, indent=2)
    print(json.dumps(dict(job=job, inputs=str(inputs), inputs_sha256=sha(inputs), manifest_sha256=digest, chapter_pin=pin,
        new_files=sum(new for _, _, new in plan), existing_identical=sum(not new for _, _, new in plan),
        shared_files=m['shared_files'], system_files=m['system_files'], streamed_bytes=m['foreign_bytes'], space=space,
        guard_command=guard_command(helper, args.chapter, args.attempt, manifest),
        rows350={args.chapter: dict(import_receipt=str(D/'jobs'/job/'resource-receipt.json'), import_inputs=str(inputs))}), indent=1))
    return 0


def receive(args):
    helper = csd_side()
    manifest = q_manifest(args.manifest)
    ch = args.chapter
    job = D/'jobs'/job_name(ch, args.attempt)
    inputs = inputs_path(job.name)
    state = read(job/'resource-receipt.json')
    assert state.get('admitted') is True and 'exit' not in state and not state.get('finished'), 'not inside an admitted guard job'
    assert {k: (state.get('policy') or {}).get(k) for k in POLICY} == POLICY, 'guard policy is not the current CSD policy'
    assert [str(c) for c in state['command']] == receive_command(helper, ch, args.attempt, manifest), 'job admitted for another command'
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
    assert not problems and chapter_of(m) == ch, '\n'.join(problems)
    pin = m['chapters'][ch]
    assert bound.get(str(q_permit(pin))) == pin['permit_sha256'], "root's Q copy of the permit is not frozen at the permit's hash"
    local = {x['path']: x for x in m['files'] if x['kind'] in ('shared', 'system')}
    assert all(bound.get(p) == x['sha256'] for p, x in local.items()), 'shared and system rows are not frozen at their hashes'
    plan, problems = plan_problems(m)
    assert not problems, '\n'.join(problems)
    read_header(sys.stdin.buffer, header_fields(m, digest))
    incoming = job/'incoming'
    incoming.mkdir(mode=0o700)
    sources = stage(sys.stdin.buffer, plan, incoming)

    def staged(path):
        source = sources.get(str(path)) or (Path(path) if str(path) in local else None)
        if source is None:
            raise Unavailable(str(path))
        return source.read_bytes()
    problems, unavailable = closure_problems(m, staged)
    assert not problems and not unavailable, '\n'.join(['closure refused from the staged bytes:']+problems+unavailable)
    assert not alias_problems(), '\n'.join(alias_problems())
    rows = place(plan, sources)
    assert not alias_problems(), '\n'.join(alias_problems())
    device = os.stat(D/'jobs').st_dev
    for x in m['files']:                                  # through the original paths, as 350 reads them
        p = Path(x['path'])
        if x['kind'] == 'system':
            assert p.stat().st_size == x['bytes'] and sha(p) == x['sha256'], str(p)
            continue
        assert p.resolve(strict=True) == (physical(p) if x['kind'] == 'foreign' else p), str(p)
        assert p.stat().st_size == x['bytes'] and p.stat().st_dev == device and sha(p) == x['sha256'], str(p)
    result = dict(version=VERSION, chapter=ch, job=job.name, manifest=str(manifest), manifest_sha256=digest,
        foreign_root=str(CSS), physical_root=str(TOP), alias_created=False, chapters=m['chapters'],
        written=sum(r['written'] for r in rows), existing_identical=sum(not r['written'] for r in rows),
        streamed_bytes=m['foreign_bytes'], shared_checked=m['shared_files'], system_checked=m['system_files'], files=rows, finished=utc(),
        rows350={ch: dict(import_receipt=str(job/'resource-receipt.json'), import_inputs=str(inputs))},
        scope='Exact import only; no render, master, permit, picture, motion, listening or AV acceptance.')
    with (job/'import-result.json').open('x') as f:
        json.dump(result, f, indent=1)
    print('Imported', len(rows), 'files;', result['written'], 'written,', result['existing_identical'], 'already identical;',
          m['shared_files'], 'shared D and', m['system_files'], 'system files checked')
    return 0


def check(args):
    helper = csd_side()
    manifest = q_manifest(args.manifest)
    job = D/'jobs'/job_name(args.chapter, args.attempt)
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
            report.update(problems=problems, rows350=result['rows350'])
            print(json.dumps(report, indent=1))
            return 0 if not problems else 1
    m, digest, plan, problems, space = preflight(args.chapter, args.attempt, manifest)
    report.update(manifest_sha256=digest, problems=problems, new_files=sum(new for _, _, new in plan),
                  existing_identical=sum(not new for _, _, new in plan), space=space,
                  alias=str(CSS), alias_state=('symlink to '+os.readlink(CSS) if CSS.is_symlink() else 'real path' if CSS.exists() else 'absent'))
    print(json.dumps(report, indent=1))
    return 0 if not problems else 1


def main():
    p = argparse.ArgumentParser(description=__doc__.split('\n')[0])
    sub = p.add_subparsers(dest='mode', required=True)
    s = sub.add_parser('export')
    s.add_argument('--chapter', choices=CHAPTERS, required=True)
    s = sub.add_parser('send')
    s.add_argument('--chapter', choices=CHAPTERS, required=True)
    s.add_argument('--manifest', type=Path, required=True)
    s.add_argument('--manifest-sha', required=True)
    for name in ('prepare', 'receive', 'check'):
        s = sub.add_parser(name)
        s.add_argument('--chapter', choices=CHAPTERS, required=True)
        s.add_argument('--attempt', type=int, default=1, choices=range(1, 100), metavar='N')
        s.add_argument('--manifest', type=Path, required=True)
    args = p.parse_args()
    assert args.mode in ('export', 'send'), 'Read-only exporter: import/mutation modes disabled'
    return {'export': export, 'send': send, 'prepare': prepare, 'receive': receive, 'check': check}[args.mode](args)


if __name__ == '__main__':
    raise SystemExit(main())
