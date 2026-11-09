"""Delivery 540 (root, adapted from interrupted CC3 scratch 536): sections 3 and 4 of handoff 523 on mac-mini-max, as the child of root's guard 508.

It makes the rebind269 Frederick narration verifiable on the mini without changing any sample, take,
sentence row or hash. Only path strings change, and only in new copies:
  1. Binds the build: 521's qualification.json (63533e3c), its terminal guard receipt and binary 5b77a1c8.
  2. Binds the inputs: handoff manifest 523 (5be46fe5); every one of its 428 rows is re-hashed at its
     ROOT destination, together with the three fetched rebind269 JSONs and the review269 direction map.
  3. Topic step (handoff section 4), as rebind269 did on the build host after build 268:
     narration_units.before-review269.json (new; the current bytes), the review269 map (new), and
     narration_units.json replaced by the fetched external-units.json bytes. The whole source tree is
     bound before and after by 521's listing rule: 770e5239 (4,197 files) -> be161e50 (4,199 files).
     A tree already at be161e50 is accepted unchanged; any other state is refused.
  4. Rebinding (handoff section 3) into the fresh guard job JOB, new files only:
       JOB/narration/                   the 109 files; 107 copied byte for byte (105 WAVs among them);
       JOB/narration/report.json        row `source` and disposition `evidence.path` rebound;
       JOB/narration/assembly.json      08_5/09_1 unit `wav` and `generation_receipt`, disposition
                                        `evidence.path` rebound; `report_sha256` recomputed;
       JOB/mapped-receipt-mini540.json  audio17/mapped-receipt.json with only records' wav/source/request
                                        rebound (the original stays as it is).
     Build-host prefixes map to the mini: work/production-repo07/ -> ROOT/repo/;
     qualification01/production-repair17/ -> ROOT/repo/<this folder's repository path>/;
     work/review-repair268/workspace/ -> ROOT/source/workspace/. Every rebound file is checked against
     the hash already recorded beside it. A leaf-by-leaf diff must contain exactly those fields
     (report 461 + 23, assembly 2 + 2 + 23 + report_sha256, receipt 6) and nothing else.
  5. Delivery verification: tools/narration_delivery.py with the python3 that PATH gives (the one the
     binary will call), consumer render, gain 0, first all 15 chapters, then 10_rigidity alone (the
     render's own argv). Both must verify all 105 cues, external-ui plan, no chapter scope.
  6. Re-hashes the 428 inputs and the source tree, and writes JOB/delivery540.json.

Guard binding: a fresh 508 job (only resource-receipt.json and command.log; admitted, native, not
preflight, policy 20/8/4/80/20/10; CARGO_TARGET_DIR and RUSTUP_TOOLCHAIN set by the guard). The working
directory must be ROOT/repo, the guard child's, because the 103 kept units resolve from it. The guard's
--inputs must not include ROOT/source: the topic step changes three files there; this helper binds that
tree itself.

Usage (root, under guard 508): python3 -I -B mini-delivery540.py --job JOB --manifest PATH-TO-MANIFEST-523
It runs no binary, render or SSH. No listening, picture, timing or AV acceptance: sample reconstruction
and source binding only. Timeline equality and new-picture reads remain before any final use.
"""
import argparse
import copy
import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import sys
import time

ROOT = Path.home()/'scratch'/'jmev-complex-native-20261109'
REPO = ROOT/'repo'
WORK = ROOT/'jobs'
TARGET = ROOT/'target'
SOURCE = ROOT/'source'
W = SOURCE/'workspace'
TOPIC = W/'explainer_corpus/complex_analysis_visual'
INPUTS = ROOT/'inputs523'
REPLICA = INPUTS/'rebind269-narration'
TOPIC269 = INPUTS/'topic269'
BUILD = WORK/'native-build521'
QREL = ('docs/video-program/third-wave-20261024/opus-replacements-20261104/fmd-render-readiness01/'
        'raster-quality03/production-repair17')

QUALIFICATION_SHA256 = '63533e3c4ede63060a1801ddf1aa0ff3775f9d6c84851ad5b4ec32accc4fbb60'
BINARY_SHA256 = '5b77a1c8a33a41bab3c3987c9d4f40797bb6a6f1a76bbb57f1ab562ae9960312'
BUILD_INPUTS_SHA256 = '01698c8ab02394969bb7f22f45374b650a4520492f7d91df71c4a295f7bf5ddd'
FACTS_SHA256 = '7d528509752e532c07edb3fda7c12cd8251ac4ba77022355ae42ad725add4417'
MANIFEST523_SHA256 = '5be46fe5fa39de86d6f9169233cc5093912d94caef49a560d9cfc7c32af35e02'
LISTING_506, FILES_506 = '770e5239a547a6b81b079400c7a11d47a401b2c76a8e90f95746da55d3211cea', 4197
LISTING_269, FILES_269 = 'be161e50591708bd16a485b5c8e8ccfdb4a8abefefdebe89c8b873420d41d00c', 4199
PLAN_506 = ('995e25c7788943deff65c4d84050d8a17edc3c2c3498be0d665d7ac5b661357a', 39643)
FETCHED = {'report.json': ('b4a4c91e8e8fd26dbad621d118b762660e60532a2bbcf535ad88b87a45406bdd', 365593),
           'assembly.json': ('f0366ea40b3c23fd17c00a0222003b9041368bea995b3c1bac3c9d34f4743864', 106672),
           'external-units.json': ('fd90880fbe7c1737786a9695e3c627dad0e4bfbaad0efba706e052e901b71968', 39639)}
MAP_NAME = 'narration-direction-map-frederick-review269.json'
MAP269 = ('7eb0e40d6366950169eb10727b3386fa1d290eaeb60078d609d5e57a0ef344e3', 32108)
RECEIPT = ('8a4b1d2aa0576dc422474b9441d8f0d805f9715e0bfaeb1532b2da66474eaa94', 8221)
POLICY = dict(admission_gib=20, stop_free_gib=8, phase_growth_gib=4, work_cap_gib=80, target_growth_cap_gib=20,
              sample_seconds=10)
TOOLCHAIN = 'nightly-2026-08-31'
CHAPTERS = ['01_the_bump', '02_turning_numbers', '03_two_planes', '04_reading_color', '05_gallery',
            '06_one_derivative', '07_one_point', '08_loop_sums', '09_boundary', '10_rigidity', '11_counting',
            '12_residues', '13_real_integral', '14_continuation', '15_closing']
D = '/data/projects/jmev-stroke-probe-20261106/'             # build-host root (constant D of css-final-repair352.py)
BUILD_HOST_NARRATION = D + 'jobs/csd-review-repair-rebind269/narration'   # S in the timeline equality rule
REPLACED = {'oliver_08_5': '08_5', 'oliver_09_1': '09_1'}
EXPECTED_CHANGES = {'report': 461 + 23, 'assembly': 2 + 2 + 23 + 1, 'receipt': 6}
ALLOWED = {
    'report': (re.compile(r'/lines/[0-9]{2}_[0-9]+/[0-9]+/source'),
               re.compile(r'/meta/direction_contract/dispositions/[0-9]+/evidence/path')),
    'assembly': (re.compile(r'/manifest/units/[0-9]+/(wav|generation_receipt)'),
                 re.compile(r'/manifest/direction_dispositions/[0-9]+/evidence/path'),
                 re.compile(r'/report_sha256')),
    'receipt': (re.compile(r'/records/[0-9]+/(wav|source|request)'),),
}


def require(ok, message):
    if not ok:
        raise SystemExit('540 refused: ' + message)


def sha(path):
    with Path(path).open('rb') as f:
        return hashlib.file_digest(f, 'sha256').hexdigest()


def regular(path, digest, size, what):
    path = Path(path)
    require(path.is_file() and not path.is_symlink(), '%s is not a regular file: %s' % (what, path))
    require(path.stat().st_size == size and sha(path) == digest, '%s has other bytes: %s' % (what, path))


def listing(top):
    """521's rule (CC0 492/506): sha256 of sorted "<sha256>  <path>\\n" over every file; no links, no specials."""
    rows = []
    for parent, dirs, files in os.walk(top, followlinks=False):
        for name in dirs + files:
            require(not Path(parent, name).is_symlink(), 'symlink in the source tree: %s' % Path(parent, name))
        for name in files:
            path = Path(parent, name)
            require(path.is_file(), 'not a regular file in the source tree: %s' % path)
            rows.append((path.relative_to(top).as_posix(), sha(path)))
    rows.sort()
    return len(rows), hashlib.sha256(''.join('%s  %s\n' % (d, r) for r, d in rows).encode()).hexdigest()


def read_receipt(job):
    for _ in range(10):                                # the guard rewrites it every 10 s while we run
        try:
            return json.loads((job/'resource-receipt.json').read_text())
        except json.JSONDecodeError:
            time.sleep(0.5)
    raise SystemExit('540 refused: the guard receipt could not be read')


def guard_binding(job):
    require(job.parent == WORK and job.is_dir() and not job.is_symlink(), 'job is not a folder in WORK')
    present = sorted(p.name for p in job.iterdir())
    require(present == ['command.log', 'resource-receipt.json'], 'the job folder is not fresh: %s' % present)
    receipt = read_receipt(job)
    require(receipt['admitted'] is True and 'exit' not in receipt and 'finished' not in receipt, 'receipt state')
    require(receipt['phase'] == 'native' and receipt['preflight_only'] is False, 'receipt phase')
    require(receipt['policy'] == POLICY, 'policy %s' % receipt['policy'])
    require(os.environ.get('CARGO_TARGET_DIR') == str(TARGET), 'CARGO_TARGET_DIR is not the guard TARGET')
    require(os.environ.get('RUSTUP_TOOLCHAIN') == TOOLCHAIN, 'RUSTUP_TOOLCHAIN is not the guard value')
    require(Path.cwd() == REPO and REPO.resolve() == REPO, 'the working directory is not the real ROOT/repo')
    return dict(job=job.name, command=receipt['command'], admission=receipt['admission'],
                inputs_sha256=receipt.get('inputs_sha256'))


def build_binding():
    """521 finished with exit 0 on its own inputs, and its qualification is the one root folded in (529)."""
    regular(BUILD/'qualification.json', QUALIFICATION_SHA256, (BUILD/'qualification.json').stat().st_size, '521 qualification')
    q = json.loads((BUILD/'qualification.json').read_text())
    require(q['schema'] == 'mini-max-native-build521' and q['binary_sha256'] == q['release_binary_sha256'] == BINARY_SHA256,
            'qualification binary')
    require(q['facts_sha256'] == FACTS_SHA256 and q['chapters'] == CHAPTERS and q['source_listing'] == LISTING_506,
            'qualification facts, chapters or listing')
    r = json.loads((BUILD/'resource-receipt.json').read_text())
    require(r.get('exit') == 0 and r.get('finished') and not r.get('resource_stop')
            and r['inputs_sha256'] == r['final_inputs_sha256'] == BUILD_INPUTS_SHA256, '521 guard receipt is not a clean exit 0')
    return q


def binary_binding(q):
    binary = BUILD/'complex_analysis_visual-521'
    require(q['binary'] == str(binary), 'qualification names another binary path')
    regular(binary, BINARY_SHA256, binary.stat().st_size, '521 binary')
    return str(binary)


def destination(row):
    require(row['mini'].startswith('ROOT/') and '..' not in row['mini'].split('/'), 'manifest destination %s' % row['mini'])
    return ROOT/row['mini'][len('ROOT/'):]


def input_binding(manifest):
    regular(manifest, MANIFEST523_SHA256, manifest.stat().st_size, 'manifest 523')
    rows = json.loads(manifest.read_text())['files']
    count = {}
    for row in rows:
        regular(destination(row), row['sha256'], row['bytes'], 'manifest row')
        count[row['group']] = count.get(row['group'], 0) + 1
    require(count == {'evidence': 317, 'narration-replica': 106, 'reference': 5}, 'manifest groups %s' % count)
    names = {destination(r).name for r in rows if r['group'] == 'narration-replica'} | set(FETCHED)
    present = {p.name for p in REPLICA.iterdir()}
    require(present == names and len(names) == 109, 'the replica folder is not exactly the 109 rebind269 files')
    for name, (digest, size) in FETCHED.items():
        regular(REPLICA/name, digest, size, 'fetched ' + name)
    regular(TOPIC269/MAP_NAME, *MAP269, 'fetched review269 map')
    regular(REPO/QREL/'audio17/mapped-receipt.json', *RECEIPT, 'audio17 mapped receipt')
    return rows


def topic_step():
    """Handoff section 4. Returns (action, files before, files after)."""
    n, digest = listing(SOURCE)
    plan, backup, new_map = TOPIC/'narration_units.json', TOPIC/'narration_units.before-review269.json', TOPIC/'docs'/MAP_NAME
    if (n, digest) == (FILES_269, LISTING_269):
        return 'already applied', (n, digest), (n, digest)
    require((n, digest) == (FILES_506, LISTING_506),
            'the source tree is neither 506 nor 506+269 (%d files, %s); plan %s, backup %s, map %s' % (
                n, digest, sha(plan), backup.exists(), new_map.exists()))
    regular(plan, *PLAN_506, '506 narration_units.json')
    plan_bytes = plan.read_bytes()
    with backup.open('xb') as f:
        f.write(plan_bytes)
    with new_map.open('xb') as f:
        f.write((TOPIC269/MAP_NAME).read_bytes())
    staged = TOPIC/'.narration_units.json.540'
    with staged.open('xb') as f:
        f.write((REPLICA/'external-units.json').read_bytes())
    os.replace(staged, plan)                             # the 506 bytes are kept in the backup written above
    after = listing(SOURCE)
    require(after == (FILES_269, LISTING_269), 'the tree after the topic step is not 506+269: %s' % (after,))
    return 'applied', (n, digest), after


def local(old):
    """One build-host path -> its mini path. Anything else is refused."""
    for prefix, base in ((D + 'work/production-repo07/', REPO), (D + 'qualification01/production-repair17/', REPO/QREL),
                         (D + 'work/review-repair268/workspace/', W)):
        if old.startswith(prefix):
            rest = old[len(prefix):]
            require('..' not in rest.split('/'), 'traversal in %s' % old)
            return str(base.resolve()/rest)
    raise SystemExit('540 refused: unexpected build-host path ' + old)


def leaves(value, at=''):
    if isinstance(value, dict):
        out = {at + '/{}': None} if not value else {}
        for k, v in value.items():
            out.update(leaves(v, at + '/' + k))
        return out
    if isinstance(value, list):
        out = {at + '/[]': None} if not value else {}
        for i, v in enumerate(value):
            out.update(leaves(v, '%s/%d' % (at, i)))
        return out
    return {at: (type(value).__name__, value)}


def path_only_diff(kind, old, new):
    """Same structure; every differing leaf is an allowed path field. Returns the changed pointers."""
    a, b = leaves(old), leaves(new)
    require(a.keys() == b.keys(), '%s structure changed' % kind)
    changed = sorted(k for k in a if a[k] != b[k])
    for k in changed:
        require(any(p.fullmatch(k) for p in ALLOWED[kind]), '%s field %s changed but is not a path field' % (kind, k))
        require(a[k][0] == b[k][0] == 'str', '%s field %s is not a string' % (kind, k))
    require(len(changed) == EXPECTED_CHANGES[kind], '%s: %d fields changed, expected %d' % (kind, len(changed), EXPECTED_CHANGES[kind]))
    return changed


def strings(value):
    if isinstance(value, dict):
        for v in value.values():
            yield from strings(v)
    elif isinstance(value, list):
        for v in value:
            yield from strings(v)
    elif isinstance(value, str):
        yield value


def write_json(path, value):
    with path.open('x') as f:
        json.dump(value, f, indent=2)
        f.write('\n')


def rebind(job):
    N = job/'narration'
    N.mkdir()
    copied = []
    for src in sorted(REPLICA.iterdir()):
        if src.name in ('report.json', 'assembly.json'):
            continue
        with src.open('rb') as i, (N/src.name).open('xb') as o:
            shutil.copyfileobj(i, o)
        require(sha(N/src.name) == sha(src), 'copy differs: ' + src.name)
        copied.append(src.name)
    require(len(copied) == 107 and sum(n.endswith('.wav') for n in copied) == 105, 'copied files')

    receipt_old = json.loads((REPO/QREL/'audio17/mapped-receipt.json').read_text())
    receipt = copy.deepcopy(receipt_old)
    require([r['cue'] for r in receipt['records']] == sorted(REPLACED.values()), 'mapped receipt cues')
    for r in receipt['records']:
        for key, digest in (('wav', 'wav_sha256'), ('source', 'source_sha256'), ('request', 'request_sha256')):
            r[key] = local(r[key])
            require(sha(r[key]) == r[digest], 'mapped receipt %s %s' % (r['cue'], key))
    receipt_path = job.resolve()/'mapped-receipt-mini540.json'
    write_json(receipt_path, receipt)
    changes = dict(receipt=path_only_diff('receipt', receipt_old, receipt))

    report_old = json.loads((REPLICA/'report.json').read_text())
    report = copy.deepcopy(report_old)
    for rows in report['lines'].values():
        for row in rows:
            row['source'] = local(row['source'])
            require(sha(row['source']) == row['source_sha256'], 'row source ' + row['source'])
    for d in report['meta']['direction_contract']['dispositions']:
        d['evidence']['path'] = local(d['evidence']['path'])
        require(sha(d['evidence']['path']) == d['evidence']['sha256'], 'disposition ' + d['cue'])

    assembly_old = json.loads((REPLICA/'assembly.json').read_text())
    assembly = copy.deepcopy(assembly_old)
    units = assembly['manifest']['units']
    require(len(units) == 105 and {u['id'] for u in units} == {'oliver_' + c for c in report['lines']}, 'units')
    for u in units:
        if u['id'] in REPLACED:
            u['wav'] = local(u['wav'])
            require(u['generation_receipt'] == D + 'qualification01/production-repair17/audio17/mapped-receipt.json', 'receipt path')
            u['generation_receipt'] = str(receipt_path)
        else:
            require(not os.path.isabs(u['wav']) and not os.path.isabs(u['generation_receipt']), 'kept unit ' + u['id'])
        wav = Path(u['wav']).resolve(strict=True)        # what coherent_narration.unit_spans will compute
        require(sha(wav) == u['sha256'], 'unit wav ' + u['id'])
        cue = u['id'][len('oliver_'):]
        require(all(row['source'] == str(wav) for row in report['lines'][cue]), 'report rows of %s are not its unit wav' % cue)
    for d in assembly['manifest']['direction_dispositions']:
        d['evidence']['path'] = local(d['evidence']['path'])
        require(sha(d['evidence']['path']) == d['evidence']['sha256'], 'disposition ' + d['cue'])
    write_json(N/'report.json', report)
    assembly['report_sha256'] = sha(N/'report.json')
    write_json(N/'assembly.json', assembly)
    changes['report'] = path_only_diff('report', report_old, report)
    changes['assembly'] = path_only_diff('assembly', assembly_old, assembly)
    for kind, value in (('report', report), ('assembly', assembly)):
        require(not any(s.startswith(D) for s in strings(value)), kind + ' still names a build-host path')
    require(sorted(p.name for p in N.iterdir()) == sorted(copied + ['report.json', 'assembly.json']), 'narration folder')
    return N, receipt_path, {k: len(v) for k, v in changes.items()}


def deliver(job, N, name, chapters):
    python = shutil.which('python3')
    require(python is not None, 'python3 is not on PATH')
    argv = [python, '-I', '-B', str(W/'tools/narration_delivery.py'), '--topic', str(TOPIC), '--narration', str(N),
            '--consumer', 'render', '--gain-db', '0']
    for c in chapters:
        argv += ['--chapter-key', c]
    with (job/(name + '.stdout')).open('xb') as o, (job/(name + '.stderr')).open('xb') as e:
        code = subprocess.run(argv, cwd=REPO, stdout=o, stderr=e, timeout=1800).returncode
    require(code == 0, '%s exited %d' % (name, code))
    x = json.loads((job/(name + '.stdout')).read_text())
    require(x['verified_cues'] == 105 and x['selected_cues'] == (105 if len(chapters) == 15 else 6)
            and x['plan_mode'] == 'external-ui' and x['chapter_scope'] is None and x['consumer'] == 'render'
            and x['topic'] == 'complex_analysis_visual' and x['listening_acceptance'] is False, '%s result %s' % (name, x))
    return dict(argv=argv, result=x)


def main(argv=None):
    p = argparse.ArgumentParser()
    p.add_argument('--job', required=True)
    p.add_argument('--manifest', type=Path, required=True)
    args = p.parse_args(argv)
    job = WORK/args.job
    guard = guard_binding(job)
    q = build_binding()
    binary = binary_binding(q)
    rows = input_binding(args.manifest)
    action, before, after = topic_step()
    N, receipt_path, changes = rebind(job)
    python = shutil.which('python3')
    version = subprocess.run([python, '-I', '-B', '-c', 'import sys; print(sys.version)'], capture_output=True,
                             text=True, timeout=60, check=True).stdout.strip()
    runs = dict(all_chapters=deliver(job, N, 'delivery-all', CHAPTERS), chapter10=deliver(job, N, 'delivery-10', ['10_rigidity']))
    input_binding(args.manifest)                         # nothing read was changed meanwhile
    require(listing(SOURCE) == (FILES_269, LISTING_269), 'the source tree changed during delivery')
    write_json(job/'delivery540.json', dict(
        schema='mini-delivery540', helper_sha256=sha(__file__), manifest523_sha256=MANIFEST523_SHA256, guard=guard,
        build=dict(qualification_sha256=QUALIFICATION_SHA256, binary=binary, binary_sha256=BINARY_SHA256,
                   facts_sha256=FACTS_SHA256),
        inputs=dict(rows=len(rows), fetched={k: v[0] for k, v in FETCHED.items()}, map=MAP269[0]),
        source=dict(topic_step=action, before=dict(files=before[0], listing=before[1]), after=dict(files=after[0], listing=after[1])),
        narration=dict(path=str(N.resolve()), files=109, report_sha256=sha(N/'report.json'),
                       assembly_sha256=sha(N/'assembly.json'), receipt_copy=str(receipt_path),
                       receipt_copy_sha256=sha(receipt_path), changed_fields=changes),
        timeline_mapping=dict(replace=BUILD_HOST_NARRATION, by=str(N.resolve())),
        python3=dict(path=python, realpath=os.path.realpath(python), version=version),
        delivery=runs,
        scope='Path-only rebinding and sample reconstruction on the mini. No render, picture, timing, '
              'listening or AV acceptance; timeline equality and new-picture reads remain before final use.'))
    print('540: topic %s; %d inputs; changed %s; 105 cues verified twice' % (action, len(rows), changes))
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
