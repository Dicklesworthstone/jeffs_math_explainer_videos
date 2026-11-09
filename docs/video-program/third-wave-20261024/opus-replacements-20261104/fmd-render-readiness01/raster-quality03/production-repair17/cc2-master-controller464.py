"""Controller 464: master 447 from draft 460, with every CSS import pinned to root's named export.

Derived from csd-master-controller461.py (6f5fa078), which stays preserved, as do 393, 449, 459 and draft 460.
Every 461 gate is kept, in 461's order: stable receipts read twice, pin rechecks, master_absent, no_duplicates,
the post-import recheck before the launch manifest is written, then one 447 launch with --wait-hours 0.

Added for CC3 review 457, gap 1 (named-export pins):
  - The four 371 exports are pinned here by file name and full sha256 (EXPORTS371: root's 392, 396, 397, 424).
  - The gallery export has no pin in this file: it does not exist yet. run requires
        --gallery-manifest NAME --gallery-sha SHA
    which root gives only after a successful 448 export. NAME is a plain .json file name directly in Q (no folder,
    no leading dot); SHA is its full lowercase sha256. Missing, malformed or non-Q values are refused.
  - Before waiting, each of the five manifests must be a regular, symlink-free file in Q at its sha, carry its
    importer's version, name its row's chapter, pass that importer's manifest_problems(), and bind its draft row's
    job, worker, worker sha, inputs and permit.
  - The five manifests join the pins rechecked while waiting, before the post-import check and after it.
  - The post-import check requires each import result to name exactly its pinned file, at its pinned sha.
  - The launch manifest is master447-launch-manifest464.json. It records the five pins and the gallery arguments,
    so a run with other arguments refuses an existing launch manifest rather than adopting it.
  - no_duplicates also refuses a running 464 (and 393/449/459/461) run.

Modes (python3 -I -B cc2-master-controller464.py MODE), on CSD, from Q:
  check [--gallery-manifest NAME --gallery-sha SHA]
        read-only: the pins (the gallery's only when both are given), then one pass over the receipts.
        Exit 0 all successful, 3 pending, 1 a failed receipt or a refused pin or gate.
  run --wait-hours N --gallery-manifest NAME --gallery-sha SHA
        root only: as 461, launching 447 once. Exit 1 refused or failed, 3 deadline; nothing launched in either.
Prepared only; nothing here has run on any host. It claims no master result and no picture, motion, audio, sync
or release acceptance, and authorises no publication.
"""
import argparse
import contextlib
import datetime
import hashlib
import importlib.util
import io
import json
import os
import re
import subprocess
import sys
import time
from pathlib import Path
from types import SimpleNamespace

VERSION = '464'
HERE = Path(__file__).resolve().parent
SELF_NAME = 'cc2-master-controller464.py'
MASTER350 = HERE/'combined-master447.py'
MASTER350_SHA = '2b01ecaab2b409268d86fe13b83e97a7e374dd61f1c657680a32444fdb6aa524'
DRAFT = HERE/'master447-manifest-draft460.json'
DRAFT_SHA = 'fcf0e7d738c5bbe142f3a938a7526f7847cdb16fe006eed2ba53596f5a6525b3'
IMPORT371 = HERE/'csd-css352-import371.py'
IMPORT371_SHA = 'ba337aca143316748561c4300d5774386387690b49052e72d29a04b103dd244f'
IMPORT448 = HERE/'csd-gallery-import448.py'
IMPORT448_SHA = 'b6653757c5c114029a94832a0dd4cb9597c42808614fbb959ef6448a5d994f6e'
LAUNCH_MANIFEST = 'master447-launch-manifest464.json'
# Root's named 371 exports, by file name in Q and full sha256 (CC3 457 gap 1).
EXPORTS371 = {
    '01_the_bump': ('css352-import371-01-the-bump-manifest392.json',
                    '633c06d0542f6669eeb42993a3b1b91a1403a752ec259757ef5ab7297a24ac4b'),
    '04_reading_color': ('css352-import371-04-reading-color-manifest396.json',
                         '04d1cd99e0114e63abfeaefa23aab7d3ce6e56092d77d8a4459fa3cb2da6896c'),
    '12_residues': ('css352-import371-12-residues-manifest397.json',
                    '73c62b8d3c92ced7b4f48a9316ce12440b72424ae29ddc6d8b9a8346451a5359'),
    '14_continuation': ('css352-import371-14-continuation-manifest424.json',
                        '79fc3bf9e5c9ffa86f5f30df19a22431acb2b51d0ac620ac259706344caa5206')}
GALLERY = '05_gallery'                                     # pinned only by root's --gallery-manifest/--gallery-sha
GALLERY_NAME = re.compile(r'[A-Za-z0-9][A-Za-z0-9._-]{0,199}\.json')
HEX64 = re.compile(r'[0-9a-f]{64}')
ROW_PIN = ('job', 'worker', 'worker_sha256', 'inputs', 'permit')
CONTROLLERS = ('csd-master-controller393.py', 'csd-master-controller449.py', 'csd-master-controller459.py',
               'csd-master-controller461.py', SELF_NAME)
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
    """Draft 460's rows, checked against 447's row shapes and the 371/448 a1 names, and the receipts each rests on,
    in the order a row needs them (an import before the render receipt it places)."""
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


def gallery_arguments(args, required):
    """Root's (name, sha) for the gallery export, or None when check is given neither."""
    name, digest = getattr(args, 'gallery_manifest', None), getattr(args, 'gallery_sha', None)
    if name is None and digest is None and not required:
        return None
    assert name is not None and digest is not None, \
        'give both --gallery-manifest and --gallery-sha (root, after a successful 448 export)'
    assert isinstance(name, str) and GALLERY_NAME.fullmatch(name) and Path(name).name == name, \
        'the gallery manifest must be a plain .json file name directly in Q: %r' % (name,)
    assert isinstance(digest, str) and HEX64.fullmatch(digest), \
        'the gallery sha must be the full sha256, 64 lowercase hex digits: %r' % (digest,)
    assert name not in {n for n, _ in EXPORTS371.values()} | {DRAFT.name, LAUNCH_MANIFEST}, \
        'the gallery manifest name is a 371 export, the draft or the launch manifest: '+name
    assert digest not in {d for _, d in EXPORTS371.values()}, 'the gallery sha is a 371 export sha'
    return name, digest


def pinned_exports(m, i, rows, gallery):
    """{chapter: (path in Q, sha256)} for every CSS-imported draft row (the gallery's only when given), each read and
    checked now: a symlink-free file in Q at its pinned sha, its importer's version, chapter and manifest_problems(),
    and the job, worker, worker sha, inputs and permit of its draft row."""
    importer448 = load(IMPORT448, IMPORT448_SHA, 'import448_pins')
    css = [r for r in rows if r['worker'] in (str(m.WORKER352_CSS), str(m.WORKER442))]
    assert sorted(r['chapter'] for r in css) == sorted(set(EXPORTS371) | {GALLERY}), \
        'the CSS-imported rows are not 01/04/12/14 (371) and 05 (448)'
    want = dict(EXPORTS371)
    if gallery is not None:
        want[GALLERY] = gallery
    out = {}
    for r in css:
        key = r['chapter']
        if key not in want:
            continue
        gallery_row = r['worker'] == str(m.WORKER442)
        assert gallery_row == (key == GALLERY), key
        checker = importer448 if gallery_row else i
        name, digest = want[key]
        path = m.Q/name
        assert path.is_file() and not path.is_symlink() and path.resolve() == path, \
            'pinned export is not a symlink-free file directly in Q: '+str(path)
        data = path.read_bytes()
        assert hashlib.sha256(data).hexdigest() == digest, 'pinned export is not at its sha %s: %s' % (digest, path)
        try:
            manifest = json.loads(data)
        except ValueError:
            raise AssertionError('pinned export is not JSON: '+str(path)) from None
        assert isinstance(manifest, dict) and manifest.get('version') == checker.VERSION, \
            'pinned export is not a version-%s manifest: %s' % (checker.VERSION, path)
        assert checker.chapter_of(manifest) == key, 'pinned export is not for %s: %s' % (key, path)
        problems = checker.manifest_problems(manifest)
        assert not problems, 'pinned export refused by %s: %s' % (checker.VERSION, '; '.join(problems[:3]))
        pin = manifest['chapters'][key]
        assert {k: pin[k] for k in ROW_PIN} == {k: r[k] for k in ROW_PIN}, \
            'pinned export binds another job, worker, inputs or permit than draft row '+key
        out[key] = (path, digest)
    return out


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


def verify_css_imports(m, i, rows, exports):
    """Recheck actual imported bytes, not just successful transfer receipts, against root's named exports.

    The original guard binds the export manifest; 371/448 check rehashes every imported file. Each import result
    must name exactly its pinned manifest in Q, at its pinned sha. This runs immediately before the launch manifest
    is written, independently of whether the relay has already performed its own post-import check.
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
        pinned_path, pinned_sha = exports[row['chapter']]
        assert str(manifest) == str(pinned_path), \
            'import result names %s, not the pinned export %s' % (manifest, pinned_path)
        digest = sha(manifest)
        assert digest == pinned_sha, 'imported manifest is not at its pinned sha %s: %s' % (pinned_sha, manifest)
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
        say(chapter=row['chapter'], post_import_rehash=True, manifest=str(manifest), manifest_sha256=digest)


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
    """No other run of this or an earlier master controller (outside this process's own ancestry) and no 447 or 350
    launch or master process."""
    mine, found = ancestors(), []
    for entry in os.listdir('/proc'):
        if not entry.isdigit() or int(entry) in mine:
            continue
        try:
            words = [w.decode(errors='replace') for w in Path('/proc', entry, 'cmdline').read_bytes().split(b'\0') if w]
        except OSError:
            continue
        if any(w.endswith(CONTROLLERS) for w in words) and 'run' in words:
            found.append(entry+' master-controller run')
        if any(w.endswith(('cc0-combined-master350.py', 'combined-master447.py')) for w in words) and {'launch', 'master'} & set(words):
            found.append(entry+' master '+' '.join(sorted({'launch', 'master'} & set(words))))
    assert not found, 'already running: '+', '.join(found)


def manifest_bytes(m, rows, exports, gallery):
    body = dict(chapters=rows, launch464=dict(
        source=dict(path=str(m.Q/DRAFT.name), sha256=DRAFT_SHA),
        imports={key: dict(manifest=str(path), sha256=digest) for key, (path, digest) in sorted(exports.items())},
        gallery_arguments={'--gallery-manifest': gallery[0], '--gallery-sha': gallery[1]},
        rows='The 15 chapter rows of the source, unchanged. Frozen 447 reads only "chapters".',
        gate=SELF_NAME+' writes this file only after stable receipts, the named-export pins above and actual '
             'post-import rechecks; 447 launch then runs its own gates before it spends anything.',
        claims='None: no chapter, import or master result, and no picture, motion, sound, sync or AV acceptance.'))
    return (json.dumps(body, indent=2)+'\n').encode()


def check(args, m, i):
    gallery = gallery_arguments(args, required=False)
    rows, receipts = rows_and_receipts(m, i)
    refused = None
    try:
        exports = pinned_exports(m, i, rows, gallery)
        say(pinned_exports={k: dict(manifest=str(p), sha256=d) for k, (p, d) in sorted(exports.items())},
            gallery='given' if gallery else 'not given; run refuses without --gallery-manifest and --gallery-sha')
    except AssertionError as error:
        refused = str(error)
        say(pinned_exports_refused=refused)
    master_absent(m)
    results = one_pass(m, receipts, show=True)
    states = [r[3] for r in results]
    say(summary=dict(receipts=len(states), successful=states.count('successful'), pending=states.count('pending'),
                     failed=states.count('failed'), pins='refused' if refused else 'ok'), launched=False)
    return 1 if refused or 'failed' in states else 3 if 'pending' in states else 0


def run(args, m, i):
    assert 0 < args.wait_hours <= MAX_WAIT_HOURS, 'wait hours must be in (0, %d]' % MAX_WAIT_HOURS
    assert HERE == m.Q and Path(__file__).resolve() == m.Q/SELF_NAME, 'run only from CSD Q'
    gallery = gallery_arguments(args, required=True)
    rows, receipts = rows_and_receipts(m, i)
    exports = pinned_exports(m, i, rows, gallery)
    pins = {p: sha(p) for p in (Path(__file__).resolve(), MASTER350, DRAFT, IMPORT371, IMPORT448, m.GUARD)}
    assert pins[IMPORT448] == IMPORT448_SHA
    assert (pins[MASTER350], pins[DRAFT], pins[IMPORT371]) == (MASTER350_SHA, DRAFT_SHA, IMPORT371_SHA)
    for path, digest in exports.values():
        pins[path] = digest
    master_absent(m)
    no_duplicates()
    say(start=True, version=VERSION, receipts=len(receipts), wait_hours=args.wait_hours,
        gallery_arguments={'--gallery-manifest': gallery[0], '--gallery-sha': gallery[1]},
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
    verify_css_imports(m, i, rows, exports)
    for path, digest in pins.items():
        assert sha(path) == digest, 'changed during post-import checks: '+str(path)
    master_absent(m)
    no_duplicates()
    manifest, body = m.Q/LAUNCH_MANIFEST, manifest_bytes(m, rows, exports, gallery)
    if manifest.exists():
        assert manifest.read_bytes() == body, 'a different launch manifest already exists: '+str(manifest)
    else:
        with manifest.open('xb') as stream:
            stream.write(body)
    assert json.loads(manifest.read_text())['chapters'] == json.loads(DRAFT.read_text())['chapters']
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
        note='447 launch returns 0 only after its own success() on the master receipt; nothing else is claimed here')
    return 0 if code == 0 else 1


def main():
    parser = argparse.ArgumentParser()
    modes = parser.add_subparsers(dest='mode', required=True)
    for name in ('check', 'run'):
        one = modes.add_parser(name)
        if name == 'run':
            one.add_argument('--wait-hours', type=float, required=True)
        one.add_argument('--gallery-manifest', default=None)
        one.add_argument('--gallery-sha', default=None)
    args = parser.parse_args()
    try:
        m = load(MASTER350, MASTER350_SHA, 'master447')
        i = load(IMPORT371, IMPORT371_SHA, 'import371')
        assert sha(DRAFT) == DRAFT_SHA, 'not draft 460'
        return {'check': check, 'run': run}[args.mode](args, m, i)
    except AssertionError as error:
        say(refused=str(error), launched=False)
        return 1
    except (OSError, ValueError, KeyError) as error:       # review 395 F2: a refusal line, not a bare traceback
        say(refused='%s: %s' % (type(error).__name__, error), launched=False)
        return 1


if __name__ == '__main__':
    raise SystemExit(main())
