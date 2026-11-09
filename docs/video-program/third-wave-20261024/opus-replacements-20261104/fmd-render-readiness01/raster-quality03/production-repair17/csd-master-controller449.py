"""Master launch controller, version 393. Prepared only; nothing here has run.

It launches the combined master exactly once, through frozen cc0-combined-master350.py's own `launch` mode and so
under 350's own guard (guard_csd02.py), with the 15 chapter rows of master350-manifest-draft384.json. It first
waits until every receipt those rows rest on is finished and successful, in 350's own success() sense, read twice
STABLE_SECONDS apart with identical bytes. It changes none of 350's gates: 350's launch still runs gates(),
preflight() and its free-space check before it spends anything.

Rules:
  - 350 (60575944), draft 384 (ddbe3f2b) and import 371 (ba337aca) are pinned. The guard and this file are hashed at
    start and must not change before the launch.
  - The CSS rows' import names must be exactly what frozen 371's own job_name(chapter, 1), inputs_path() and
    css_inputs() give (CC3: attempt a1). The retained rows' must be the import311 a1 jobs 384 names.
  - The master job folder D/jobs/csd-combined-master350 and Q/combined-master350-inputs.json must not exist, at
    start, at every poll and just before the launch. No other run of this controller and no 350 launch or master
    process may be running.
  - A finished receipt that fails 350's success(), or one that is not admitted or is preflight-only, stops the
    controller at once: nothing is launched and nothing is retried. A later attempt needs a new manifest and a new
    controller version.
  - A receipt that is missing, unfinished, or half-written (the guard rewrites it in place) is pending. A CSS render
    that fails on CSS never reaches CSD, so it shows here only as an import that never arrives: at the deadline the
    controller stops, launching nothing.
  - It writes one file of its own, Q/master350-launch-manifest393.json: draft 384's 15 rows unchanged, and a block
    that claims no result and no acceptance. Its bytes are fixed, so a later run accepts it only byte for byte.
  - It records no completion of its own. Its last line reports 350 launch's exit and the master receipt's top-level
    exit and finished exactly as read.

Modes (python3 -I -B csd-master-controller393.py MODE), on CSD, from Q:
  check                 read-only: one pass, a JSON line per receipt and a summary. Exit 0 when every receipt is
                        successful, 3 while any is pending and none failed, 1 on a failure or a refused gate.
  run --wait-hours N    root only: polls every POLL_SECONDS for at most N hours (0 < N <= 96), then launches 350
                        (`launch --manifest Q/master350-launch-manifest393.json --wait-hours 0`) in the foreground
                        and returns its exit. Exit 1 on a failure or refused gate, 3 at the deadline, launching
                        nothing in either case.
It accepts no picture, motion, sound, fine-sync or AV result, and authorises no publication.
"""
import argparse
import contextlib
import datetime
import hashlib
import importlib.util
import io
import json
import os
import subprocess
import sys
import time
from pathlib import Path
from types import SimpleNamespace

VERSION = '449'
HERE = Path(__file__).resolve().parent
MASTER350 = HERE/'combined-master447.py'
MASTER350_SHA = '2b01ecaab2b409268d86fe13b83e97a7e374dd61f1c657680a32444fdb6aa524'
DRAFT = HERE/'master447-manifest-draft449.json'
DRAFT_SHA = '0108537698028e0b7389533f85c552c652e9b7aef72f86b6028ae209870be5aa'
IMPORT371 = HERE/'csd-css352-import371.py'
IMPORT371_SHA = 'ba337aca143316748561c4300d5774386387690b49052e72d29a04b103dd244f'
IMPORT448 = HERE/'csd-gallery-import448.py'
IMPORT448_SHA = 'b6653757c5c114029a94832a0dd4cb9597c42808614fbb959ef6448a5d994f6e'
LAUNCH_MANIFEST = 'master447-launch-manifest449.json'
# root preflight385 (a5b019ae): the registry, in order
REGISTRY = ('01_the_bump', '02_turning_numbers', '03_two_planes', '04_reading_color', '05_gallery', '06_one_derivative',
            '07_one_point', '08_loop_sums', '09_boundary', '10_rigidity', '11_counting', '12_residues',
            '13_real_integral', '14_continuation', '15_closing')
IMPORT311 = {'02_turning_numbers': 'trj', '03_two_planes': 'trj', '13_real_integral': 'css', '15_closing': 'hz2'}
POLL_SECONDS = 60
STABLE_SECONDS = 60
MAX_WAIT_HOURS = 96


def sha(path):
    with Path(path).open('rb') as stream:
        return hashlib.file_digest(stream, 'sha256').hexdigest()


def load(path, digest, name):
    assert sha(path) == digest, 'not the pinned bytes: '+str(path)
    spec = importlib.util.spec_from_file_location(name, path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)        # constants and functions only; both keep main() behind __name__
    return module


def utc():
    return datetime.datetime.now(datetime.timezone.utc).isoformat(timespec='seconds')


def say(**fields):
    print(json.dumps(dict(utc=utc(), **fields)), flush=True)


def rows_and_receipts(m, i):
    """Draft 384's rows, checked against 350's row shapes and 371's names, and the receipts each rests on, in the
    order a row needs them (an import before the render receipt it places)."""
    rows = json.loads(DRAFT.read_text())['chapters']
    assert tuple(r['chapter'] for r in rows) == REGISTRY, 'draft rows are not the registry, in order'
    assert len({r['job'] for r in rows}) == 15
    receipts = []
    for r in rows:
        key, kind = r['chapter'], r['kind']
        want = 'boundary313' if key == m.BOUNDARY_CHAPTER else 'combined' if key in m.CHANGED else 'retained'
        assert kind == want, key
        if kind == 'retained':
            pin = m.RETAINED[key]
            assert set(r) == m.ROW_KEYS | m.IMPORT_KEYS | {'retention_decision'}, key
            assert (r['worker'], r['job'], r['inputs'], r['permit']) == \
                (str(pin['worker']), pin['job'], str(pin['inputs']), str(pin['permit'])), key
            job = 'csd-retained-import311-%s-a1' % IMPORT311[key]
            assert (r['import_receipt'], r['import_inputs']) == \
                (str(m.D/'jobs'/job/'resource-receipt.json'), str(m.Q/(job+'-inputs.json'))), key
            receipts += [(key, 'import311', Path(r['import_receipt'])),
                         (key, 'retained render', pin['host']/'jobs'/pin['job']/'resource-receipt.json')]
        elif kind == 'boundary313':
            assert set(r) == m.ROW_KEYS | {'worker_review'} and (r['worker'], r['job']) == (str(m.WORKER344), m.JOB344)
            receipts.append((key, 'render', m.D/'jobs'/m.JOB344/'resource-receipt.json'))
        elif r['worker'] == str(m.WORKER352):
            assert set(r) == m.ROW_KEYS and r['job'] == 'csd-reviewed-final-%s352' % key.replace('_', '-'), key
            receipts.append((key, 'render', m.D/'jobs'/r['job']/'resource-receipt.json'))
        elif r['worker'] == str(m.WORKER442):
            recovery = load(IMPORT448, IMPORT448_SHA, 'import448')
            assert key == '05_gallery' and set(r) == m.ROW_KEYS | m.IMPORT_KEYS
            assert r['job'] == recovery.css_job(key)
            job = recovery.job_name(key, 1)
            assert (r['import_receipt'], r['import_inputs'], r['inputs']) == (str(recovery.D/'jobs'/job/'resource-receipt.json'), str(recovery.inputs_path(job)), str(recovery.css_inputs(key)))
            receipts += [(key, 'import371', Path(r['import_receipt'])), (key, 'css render', m.SSD442/'jobs'/r['job']/'resource-receipt.json')]
        else:
            assert r['worker'] == str(m.WORKER352_CSS) and set(r) == m.ROW_KEYS | m.IMPORT_KEYS, key
            assert r['job'] == i.css_job(key) == 'css-reviewed-final-%s352' % key.replace('_', '-'), key
            job = i.job_name(key, 1)                       # 371: csd-css352-import371-<dashed>-a1
            assert (r['import_receipt'], r['import_inputs'], r['inputs']) == \
                (str(i.D/'jobs'/job/'resource-receipt.json'), str(i.inputs_path(job)), str(i.css_inputs(key))), \
                key+': not the names frozen 371 gives for attempt 1'
            receipts += [(key, 'import371', Path(r['import_receipt'])),
                         (key, 'css render', m.CSS/'jobs'/r['job']/'resource-receipt.json')]
    return rows, receipts


def live_guard(m, path, receipt):
    """A CSD receipt whose guard pid is still a live process naming this job: not yet stable. (A foreign
    receipt's pid is another host's.)"""
    pid = receipt.get('pid')
    if type(pid) is not int or path.parent.parent != m.D/'jobs':
        return False
    try:
        words = Path('/proc', str(pid), 'cmdline').read_bytes().split(b'\0')
    except OSError:
        return False
    return any(path.parent.name.encode() in w for w in words)


def state(m, path, after=None):
    """('pending'|'successful'|'failed', sha256 or None, detail). after: the import receipt this render receipt
    is placed by; while it is not successful, the render receipt is pending."""
    if after is not None and after != 'successful':
        return 'pending', None, 'waits for its import'
    try:
        data = path.read_bytes()
    except FileNotFoundError:
        if after == 'successful':
            return 'failed', None, 'import finished, but this render receipt is not at its original path'
        return 'pending', None, 'no receipt yet'
    try:
        receipt = json.loads(data)
    except ValueError:
        return 'pending', None, 'receipt being rewritten'
    digest = hashlib.sha256(data).hexdigest()
    if not isinstance(receipt, dict) or not receipt.get('finished'):
        return 'pending', digest, 'not finished (top-level finished absent)'
    if receipt.get('admitted') is False or receipt.get('preflight_only') is True:
        return 'failed', digest, 'finished without an admitted run'
    try:
        m.success(path)                                    # 350's own rule, on the file as it is now
    except (AssertionError, KeyError, TypeError) as error:
        return 'failed', digest, 'finished, fails 350 success(): %s exit %r' % (type(error).__name__, receipt.get('exit'))
    except ValueError:
        return 'pending', None, 'receipt being rewritten'
    if hashlib.sha256(path.read_bytes()).hexdigest() != digest or live_guard(m, path, receipt):
        return 'pending', digest, 'finished, still being written'
    return 'successful', digest, 'finished, exit 0, inputs unchanged'


def one_pass(m, receipts, show):
    results, imports = [], {}
    for key, role, path in receipts:
        after = imports.get(key, 'pending') if role in ('retained render', 'css render') else None
        s, digest, detail = state(m, path, after)
        if role in ('import311', 'import371'):
            imports[key] = s
        results.append((key, role, path, s, digest))
        if show:
            say(chapter=key, role=role, receipt=str(path), state=s, sha256=digest, detail=detail)
    return results


def master_absent(m):
    assert not (m.D/'jobs'/m.JOB).exists(), 'the master job folder already exists: '+str(m.D/'jobs'/m.JOB)
    assert not m.INPUTS.exists(), 'the master inputs already exist: '+str(m.INPUTS)


def verify_css_imports(m, i, rows):
    """Recheck actual imported bytes, not just successful transfer receipts.

    The original guard binds the export manifest; 371 check rehashes every
    imported file. This runs immediately before master launch, independently
    of whether the relay has already performed its own post-import check.
    """
    for row in rows:
        if row['worker'] not in (str(m.WORKER352_CSS), str(m.WORKER442)):
            continue
        checker = load(IMPORT448, IMPORT448_SHA, 'import448_check') if row['worker'] == str(m.WORKER442) else i
        receipt = Path(row['import_receipt'])
        m.success(receipt)
        top = json.loads(receipt.read_text())
        inputs = Path(row['import_inputs'])
        assert sha(inputs) == top['inputs_sha256'], 'changed import bindings'
        result = json.loads((receipt.parent/'import-result.json').read_text())
        manifest = i.q_manifest(Path(result['manifest']))
        digest = sha(manifest)
        bound = json.loads(inputs.read_text())['files']
        assert sum(x['path'] == str(manifest) and x['sha256'] == digest for x in bound) == 1
        assert result['manifest_sha256'] == digest
        assert checker.chapter_of(json.loads(manifest.read_text())) == row['chapter']
        expected = {row['chapter']: {k: row[k] for k in ('import_receipt', 'import_inputs')}}
        assert result['rows350'] == expected, 'import result names different master rows'
        output = io.StringIO()
        with contextlib.redirect_stdout(output):
            code = checker.check(SimpleNamespace(chapter=row['chapter'], attempt=1, manifest=manifest))
        report = json.loads(output.getvalue())
        assert code == 0 and report.get('terminal_success') is True
        assert report.get('import_result') is True and report.get('problems') == []
        assert report.get('rows350') == expected
        say(chapter=row['chapter'], post_import_rehash=True, manifest_sha256=digest)


def ancestors():
    out, pid = set(), os.getpid()
    while pid > 1:
        out.add(pid)
        try:
            pid = int(Path('/proc', str(pid), 'stat').read_text().rsplit(')', 1)[1].split()[1])
        except (OSError, ValueError, IndexError):
            break
    return out


def no_duplicates():
    """No other run of this controller (outside this process's own ancestry) and no 350 launch or master process."""
    mine, found = ancestors(), []
    for entry in os.listdir('/proc'):
        if not entry.isdigit() or int(entry) in mine:
            continue
        try:
            words = [w.decode(errors='replace') for w in Path('/proc', entry, 'cmdline').read_bytes().split(b'\0') if w]
        except OSError:
            continue
        if any(w.endswith(('csd-master-controller393.py', 'csd-master-controller449.py')) for w in words) and 'run' in words:
            found.append(entry+' controller393 run')
        if any(w.endswith(('cc0-combined-master350.py', 'combined-master447.py')) for w in words) and {'launch', 'master'} & set(words):
            found.append(entry+' 350 '+' '.join(sorted({'launch', 'master'} & set(words))))
    assert not found, 'already running: '+', '.join(found)


def manifest_bytes(m, rows):
    body = dict(chapters=rows, launch393=dict(
        source=dict(path=str(m.Q/DRAFT.name), sha256=DRAFT_SHA),
        rows='The 15 chapter rows of the source, unchanged. Frozen 350 reads only "chapters".',
        gate='csd-master-controller393.py writes this file only after its prerequisite pass; 350 launch then runs '
             'its own gates before it spends anything.',
        claims='None: no chapter, import or master result, and no picture, motion, sound, sync or AV acceptance.'))
    return (json.dumps(body, indent=2)+'\n').encode()


def check(args, m, i):
    _, receipts = rows_and_receipts(m, i)
    master_absent(m)
    results = one_pass(m, receipts, show=True)
    states = [r[3] for r in results]
    say(summary=dict(receipts=len(states), successful=states.count('successful'), pending=states.count('pending'),
                     failed=states.count('failed')), launched=False)
    return 1 if 'failed' in states else 3 if 'pending' in states else 0


def run(args, m, i):
    assert 0 < args.wait_hours <= MAX_WAIT_HOURS, 'wait hours must be in (0, %d]' % MAX_WAIT_HOURS
    assert HERE == m.Q and Path(__file__).resolve() == m.Q/'csd-master-controller449.py', 'run only from CSD Q'
    rows, receipts = rows_and_receipts(m, i)
    pins = {p: sha(p) for p in (Path(__file__).resolve(), MASTER350, DRAFT, IMPORT371, IMPORT448, m.GUARD)}
    assert pins[IMPORT448] == IMPORT448_SHA
    assert (pins[MASTER350], pins[DRAFT], pins[IMPORT371]) == (MASTER350_SHA, DRAFT_SHA, IMPORT371_SHA)
    master_absent(m)
    no_duplicates()
    say(start=True, version=VERSION, receipts=len(receipts), wait_hours=args.wait_hours,
        pins={str(p): d for p, d in pins.items()})
    deadline, last = time.monotonic() + args.wait_hours*3600, None
    while True:
        first = one_pass(m, receipts, show=False)
        failed = [r for r in first if r[3] == 'failed']
        if failed:
            one_pass(m, receipts, show=True)
            say(stop='a prerequisite receipt finished without success; nothing launched, no retry', launched=False)
            return 1
        if all(r[3] == 'successful' for r in first):
            time.sleep(STABLE_SECONDS)
            second = one_pass(m, receipts, show=False)
            if [(r[2], r[3], r[4]) for r in second] == [(r[2], 'successful', r[4]) for r in first]:
                break
            if any(r[3] == 'failed' for r in second):
                continue                                   # the failure branch above reports it and stops
        pending = sorted('%s %s' % (r[0], r[1]) for r in first if r[3] != 'successful')
        if pending != last:
            say(pending=pending, launched=False)
            last = pending
        if time.monotonic() >= deadline:
            say(stop='deadline reached with receipts pending; nothing launched', pending=pending, launched=False)
            return 3
        for path, digest in pins.items():
            assert sha(path) == digest, 'changed while waiting: '+str(path)
        master_absent(m)
        time.sleep(POLL_SECONDS)
    one_pass(m, receipts, show=True)
    for path, digest in pins.items():
        assert sha(path) == digest, 'changed before the launch: '+str(path)
    master_absent(m)
    no_duplicates()
    manifest, body = m.Q/LAUNCH_MANIFEST, manifest_bytes(m, rows)
    if manifest.exists():
        assert manifest.read_bytes() == body, 'a different launch manifest already exists: '+str(manifest)
    else:
        with manifest.open('xb') as stream:
            stream.write(body)
    assert json.loads(manifest.read_text())['chapters'] == json.loads(DRAFT.read_text())['chapters']
    verify_css_imports(m, i, rows)
    for path, digest in pins.items():
        assert sha(path) == digest, 'changed during post-import checks: '+str(path)
    master_absent(m)
    no_duplicates()
    command = [m.PYTHON, '-I', '-B', str(MASTER350), 'launch', '--manifest', str(manifest), '--wait-hours', '0']
    say(launch=command, manifest_sha256=sha(manifest), launched=True)
    code = subprocess.run(command).returncode
    receipt = m.D/'jobs'/m.JOB/'resource-receipt.json'
    try:
        top = json.loads(receipt.read_text())
        seen = {k: top.get(k, 'absent') for k in ('exit', 'finished')}
    except (OSError, ValueError) as error:
        seen = dict(receipt='unreadable: '+type(error).__name__)
    say(launch_exit=code, master_receipt=str(receipt), master_receipt_top_level=seen,
        note='350 launch returns 0 only after its own success() on the master receipt; nothing else is claimed here')
    return 0 if code == 0 else 1


def main():
    parser = argparse.ArgumentParser()
    modes = parser.add_subparsers(dest='mode', required=True)
    modes.add_parser('check')
    one = modes.add_parser('run')
    one.add_argument('--wait-hours', type=float, required=True)
    args = parser.parse_args()
    m = load(MASTER350, MASTER350_SHA, 'master350')
    i = load(IMPORT371, IMPORT371_SHA, 'import371')
    assert sha(DRAFT) == DRAFT_SHA, 'not draft 384'
    try:
        return {'check': check, 'run': run}[args.mode](args, m, i)
    except AssertionError as error:
        say(refused=str(error), launched=False)
        return 1


if __name__ == '__main__':
    raise SystemExit(main())
