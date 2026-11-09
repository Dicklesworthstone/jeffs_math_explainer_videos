"""Combined-repair complex master on CSD, version 293. Prepared only; nothing here has run.

Derived from trj-final-master184.py, which supplies the master body:
  - every chapter staged by hard link;
  - `master` and then `captions all` run with the narration and score;
  - all 15 soundtrack replays must be bit-identical;
  - ffprobe gates for 4K60 video and 48 kHz stereo AAC;
  - every staged and bound file is re-hashed.
The launcher replaces the waiters trj-master-after-imports184.py and
trj-master-after-chapters184.py: wait for terminal receipts, freeze the inputs, start one
guarded job.

What changes against 184:
  - Host. CSD, with binary review-repair268, the timeline269 schedules, the score21 stems and the
    rebind269 narration (ElevenLabs Frederick: 105 WAVs, byte-identical to assembly19's).
  - Chapter list. Root supplies it as a manifest: one row per registry chapter, in registry order.
  - The 11 changed chapters must be combined-repair renders on binary 268.
  - 02/03/13/15 may instead be the existing production19 finals pinned below. That needs source,
    schedule and audio evidence, which this file checks, plus a named root retention decision.

Modes:
  check   read-only: run every gate and print one JSON line per chapter. Writes nothing.
  launch  Optionally wait for the rows' terminal receipts, then re-run every gate. Freeze the inputs
          with an exclusive create and start the one guarded master job. No retry.
  master  Only inside that guard: the 184 body.

This file creates no permit, review, import, render, verification or upload.
"""
import argparse
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import re
import subprocess
import time
import wave

VERSION = '293'
D = Path('/data/projects/jmev-stroke-probe-20261106')        # CSD
Q = D/'qualification01/production-repair17'
TRJ = Path('/data/projects/jmev-render-worker-20261107')
CSS = Path('/data/projects/jmev-css-render-worker-20261107')
HZ2 = Path('/data/projects/jmev-hz2-render-worker-20261107')
JOB = 'csd-combined-master293'
INPUTS = Q/'combined-master293-inputs.json'
GUARD = D/'guard_csd02.py'
PYTHON = '/usr/bin/python3'
BIN = D/'work/bin/review-repair268'
BINARY_268 = '983954c508e42b90d443ad6ca09f2baba7565a4f609d09d2ea821640534d6ec7'
BINARY_19 = 'f4836eeeb4f2465c0d27ca0497f123f0af6bc0db7bc65096098bb20c5abb0f22'
BUILD268 = D/'jobs/csd-review-repair-build268'
BUILD19 = D/'jobs/csd-production-repair-build19'
SOURCE_COPY_268_SHA = '81d28272a7a23b63199e79e6aa1086f01f131f0ddd685d4ae859d04b1de1934c'
# Binary 268 reads the script and runs the delivery contract from its CARGO_MANIFEST_DIR
# (production19 main.rs:48), the crate inside this workspace. Build19 compiled WORKSPACE19.
WORKSPACE268 = D/'work/review-repair268/workspace'
WORKSPACE19 = D/'work/production-repair19/workspace'
CONTRACT = ('explainer_corpus/complex_analysis_visual/script.tsv',
            'tools/narration_delivery.py', 'tools/coherent_narration.py')
TIMELINES = D/'jobs/csd-review-repair-timeline269/timeline-output'
TIMELINE269_INPUTS = Q/'review-repair-timeline269-inputs.json'
BASELINE = D/'jobs/csd-production-timeline21/timeline-output'
ORIGINAL_NARRATION = D/'jobs/csd-production-assembly19/narration'
NARRATION = D/'jobs/csd-review-repair-rebind269/narration'
SCORE = D/'jobs/csd-production-score21'
STEMS = SCORE/'score/stems'
KINDS = ('timeline', 'cues', 'cuelog')
SUFFIXES = ('.mp4', '.timeline.tsv', '.cues.tsv', '.cuelog.tsv', '.layout.tsv')
TOTAL_FRAMES = 133662
SCENE_DIR = 'explainer_corpus/complex_analysis_visual/src/chapters/'
# The chapters combined268 changes (csd-reviewed-final287.py CHAPTERS). They are never retained.
CHANGED = ('01_the_bump', '04_reading_color', '05_gallery', '06_one_derivative', '07_one_point',
           '08_loop_sums', '09_boundary', '10_rigidity', '11_counting', '12_residues',
           '14_continuation')
# The existing finals that may be retained.
#   worker/job/inputs/permit: chapter-plan183.json.
#   permit, movie and frames: each final's render-summary.json (final02-terminal167,
#     final03-terminal115, final13-terminal200, final15-terminal81).
#   scene: build268 source-copy.json, equal to production-source-preserved214/receipt.json.
RETAINED = {
    '02_turning_numbers': dict(
        host=TRJ, worker=TRJ/'qualification01/production19/trj-final-chapter19-02.py',
        job='trj-production19-final-02-turning-numbers32',
        inputs=TRJ/'qualification01/production19/final02-32-inputs.json',
        permit=TRJ/'qualification01/production19/review02-32/final-render-permit32.json',
        permit_sha256='ccdff19cd315f5e4893b59fdc11e84486e8db15ab6f9fd63ae73535bd7dc5ac2',
        movie_sha256='bf0891a409f589bd4fc125c038e03dd027ec334436e25cdc129463c659984f07', frames=5372,
        scene='turning.rs', scene_sha256='24bffa50a914c21cc1457c5056c740a3f25149baf9c140630b2b47e0380b11ec'),
    '03_two_planes': dict(
        host=TRJ, worker=TRJ/'qualification01/production19/trj-final-chapter19-02.py',
        job='trj-production19-final-03-two-planes28',
        inputs=TRJ/'qualification01/production19/final03-28-inputs.json',
        permit=TRJ/'qualification01/production19/review03-28/final-render-permit28.json',
        permit_sha256='107aebb15239c67d66569c5e1217c1343c7490901431ef935ce044962e7c64ca',
        movie_sha256='20353d01ee564ef101ecad80bdfa8ec1db87ac2188b2e4642969b371f2e82cf3', frames=7249,
        scene='two_planes.rs', scene_sha256='0f22d63febc18e05dc4123bedf84eeadf967938bcbc4260cfb0d9e8925ff4cb9'),
    '13_real_integral': dict(
        host=CSS, worker=CSS/'qualification01/css-final-chapter78.py',
        job='css-production19-final-13-real-integral96',
        inputs=CSS/'qualification01/final13-96-inputs.json',
        permit=CSS/'qualification01/review13-96/final-render-permit96.json',
        permit_sha256='e86b5d886b6adf15ff2e682308b19b2284f0c00a6229e4dced6994b70290f8e2',
        movie_sha256='49934034d4611b81155925e667dab463072b21ba0240467447b9ddfb52d32f51', frames=10641,
        scene='real_integral.rs', scene_sha256='83eadb5372812fc540738bbfe9c980ffc1cc35e959be9db9f75906ba425299ae'),
    '15_closing': dict(
        host=HZ2, worker=HZ2/'qualification01/production19/hz2-final-chapter53.py',
        job='hz2-production19-final-15-closing53',
        inputs=HZ2/'qualification01/production19/final15-53-inputs.json',
        permit=HZ2/'qualification01/production19/review15-53/final-render-permit53.json',
        permit_sha256='ec74b68375331470ac18c2c7e5db4fc60bb82a247e2eeead844233c453fecd20',
        movie_sha256='3e2440a8785319cf9477e73303e6391c8a4165c7fbfd3fa6f4ad97e81f874fd4', frames=4379,
        scene='closing.rs', scene_sha256='0c217b6b5daee2681a61049b0918ceda7cc211d377ecd0c5f70b8562dd36568b'),
}
ROW_KEYS = {'chapter', 'kind', 'worker', 'worker_sha256', 'job', 'inputs', 'permit'}
IMPORT_KEYS = {'import_receipt', 'import_inputs'}


def sha(path):
    with Path(path).open('rb') as stream:
        return hashlib.file_digest(stream, 'sha256').hexdigest()


def read(path):
    return json.loads(Path(path).read_text())


def success(path):
    row = read(path)
    assert row.get('finished') and row.get('exit') == 0, str(path)
    assert row['inputs_sha256'] == row['final_inputs_sha256'], str(path)
    assert not any(row.get(k) for k in ('error', 'resource_stop', 'stopped_for_resource_limit')), str(path)
    return row


def absolute(path, real=True):
    """An absolute path that exists. real: also free of symlinks (CSD-side files under D).
    Foreign rows keep the exact lexical path their own receipts name. Their bytes are bound by
    hash either way."""
    path = Path(path)
    assert path.is_absolute() and '..' not in path.parts and path.resolve(strict=True), str(path)
    assert not real or path == path.resolve(), 'symlink-free path required: '+str(path)
    return path


def header(path):
    return dict(line.split('\t', 1) for line in Path(path).read_text().splitlines()
                if line.startswith(('chapter\t', 'fps\t', 'frames\t')))


def flag(command, name):
    values = [command[i+1] for i, item in enumerate(command[:-1]) if item == name]
    assert len(values) == 1, name
    return values[0]


def load(path, chapter):
    spec = importlib.util.spec_from_file_location('worker_'+chapter, path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def film():
    """Whole-film gates: the 15-chapter registry, all 45 timeline269 tables, score21, narration."""
    files = {BIN, GUARD, TIMELINE269_INPUTS, BUILD268/'qualification.json', BUILD268/'source-copy.json',
             BUILD19/'qualification.json', SCORE/'score21-summary.json'}
    assert sha(BIN) == BINARY_268
    for job in (BUILD268, BUILD19, NARRATION.parent, TIMELINES.parent, BASELINE.parent, SCORE):
        success(job/'resource-receipt.json')
        files.add(job/'resource-receipt.json')
    q268, q19 = read(BUILD268/'qualification.json'), read(BUILD19/'qualification.json')
    assert q268['binary_sha256'] == BINARY_268 and q19['binary_sha256'] == BINARY_19
    registry = q268['chapters']
    assert registry == q19['chapters'] and len(registry) == len(set(registry)) == 15
    assert set(registry) == set(CHANGED) | set(RETAINED) and not set(CHANGED) & set(RETAINED)
    assert sha(TIMELINE269_INPUTS) == read(TIMELINES.parent/'resource-receipt.json')['inputs_sha256']
    frames = {}
    for key in registry:
        for kind in KINDS:
            new, old = TIMELINES/(key+'.'+kind+'.tsv'), BASELINE/(key+'.'+kind+'.tsv')
            expected = old.read_bytes().replace(str(ORIGINAL_NARRATION).encode(), str(NARRATION).encode())
            assert new.read_bytes() == expected, 'timeline269 differs from timeline21: '+str(new)
            files.update((new, old))
        top = header(TIMELINES/(key+'.timeline.tsv'))
        assert (top['chapter'], top['fps']) == (key, '60')
        frames[key] = int(top['frames'])
        assert read(STEMS/'qa'/(key+'.json'))['passed'] is True
    assert sum(frames.values()) == TOTAL_FRAMES
    score = read(SCORE/'score21-summary.json')
    assert score['all_qa_passed'] and score['chapters'] == registry
    for relative, digest in score['files'].items():
        assert sha(SCORE/'score'/relative) == digest, relative
        files.add(SCORE/'score'/relative)
    assert sha(BUILD268/'source-copy.json') == SOURCE_COPY_268_SHA
    copied = read(BUILD268/'source-copy.json')['files']
    for relative in CONTRACT:
        assert sha(WORKSPACE268/relative) == copied[relative], relative
        files.add(WORKSPACE268/relative)
    wavs = sorted(NARRATION.glob('*.wav'))
    assert len(wavs) == 105, 'not the 105 narration WAVs'
    for path in wavs:
        assert sha(path) == sha(ORIGINAL_NARRATION/path.name), 'narration differs from assembly19: '+path.name
        files.add(ORIGINAL_NARRATION/path.name)
    files.update(p for p in NARRATION.rglob('*') if p.is_file())
    return registry, frames, files


def permit_gate(chapter, review, binary, table):
    """check()'s permit assertions that hold on any host (287 check, production19 check)."""
    assert review['binary_sha256'] == binary and review['chapter'] == chapter
    assert review['actually_reviewed_pictures'] is True and review['actually_reviewed_motion'] is True
    assert review['unresolved_defects'] == [] and review['reviewer'].strip() and review['observations'].strip()
    assert review['timeline_sha256'] == sha(table)
    assert review['evidence'] and review['motion_intervals']
    for item in review['evidence']:
        assert sha(item['path']) == item['sha256'], item['path']
    frames = review['frames']
    assert type(frames) is int and frames > 0
    for a, b in review['motion_intervals']:
        assert 0 <= a < b < frames
    return {Path(item['path']) for item in review['evidence']}


def schedule(chapter, render, narration, frames):
    """Timeline269's schedule and audio for one rendered chapter:
    - tables equal timeline269's after the one named narration-folder mapping;
    - every narration file equals rebind269's byte for byte;
    - the score rows are score21's stems at score21's hashes."""
    old, new = str(NARRATION), str(narration)
    files = set()
    for kind in ('timeline', 'cues'):
        expected = (TIMELINES/(chapter+'.'+kind+'.tsv')).read_bytes().replace(old.encode(), new.encode())
        assert (render/(chapter+'.'+kind+'.tsv')).read_bytes() == expected, kind
    assert header(render/(chapter+'.timeline.tsv')) == {'chapter': chapter, 'fps': '60', 'frames': str(frames)}
    lines = [row.split('\t') for row in (render/(chapter+'.timeline.tsv')).read_text().splitlines()
             if row.startswith('line\t')]
    assert lines
    for row in lines:
        path = Path(row[4])
        assert path.parent == narration and sha(path) == sha(NARRATION/path.name), str(path)
        files.update((path, NARRATION/path.name))
    base = (TIMELINES/(chapter+'.cuelog.tsv')).read_text().replace(old, new).splitlines()
    actual = (render/(chapter+'.cuelog.tsv')).read_text().splitlines()
    assert actual[:len(base)] == base
    for row in [x.split('\t') for x in base if x.strip() and not x.startswith('#')]:
        assert Path(row[0]).parent == narration and sha(row[0]) == row[1], row[0]
        files.add(Path(row[0]))
    digests = read(SCORE/'score21-summary.json')['files']
    stems = [x.split('\t') for x in actual[len(base):]]
    assert stems
    for row in stems:
        path = Path(row[0])
        assert path.parent == STEMS and digests.get(str(path.relative_to(SCORE/'score'))) == row[1] == sha(path), row[0]
    return files


def imported(row):
    """A foreign chapter reaches CSD only through a completed guarded CSD import. The files it lists
    sit at their original absolute paths."""
    receipt, manifest = absolute(row['import_receipt']), absolute(row['import_inputs'])
    assert receipt.name == 'resource-receipt.json' and receipt.parent.parent == D/'jobs'
    assert manifest.is_relative_to(D)
    state = success(receipt)
    assert sha(manifest) == state['inputs_sha256']
    files = {receipt, manifest}
    for item in read(manifest)['files']:
        assert sha(item['path']) == item['sha256'], item['path']
        files.add(Path(item['path']))
    return files


def retention(chapter, row, pin):
    """Source evidence for a retained final, plus root's named decision.
    The chapter's scene file is the same bytes in build19's workspace, in build268's pinned source
    copy (outside its changed list) and in build268's workspace."""
    decision = row['retention_decision']
    assert set(decision) == {'path', 'sha256'}
    path = absolute(decision['path'])
    assert path.is_relative_to(Q) and sha(path) == decision['sha256']
    relative = SCENE_DIR+pin['scene']
    copied = read(BUILD268/'source-copy.json')
    assert copied['files'][relative] == pin['scene_sha256'] and relative not in copied['changed']
    for tree in (WORKSPACE19, WORKSPACE268):
        assert sha(tree/relative) == pin['scene_sha256'], str(tree/relative)
    return {path, WORKSPACE19/relative, WORKSPACE268/relative}


def chapter_gate(row, frames_of):
    key, kind = row['chapter'], row['kind']
    worker, inputs, permit = (absolute(row[k], real=False) for k in ('worker', 'inputs', 'permit'))
    job = row['job']
    assert Path(job).name == job and sha(worker) == row['worker_sha256'], key
    if kind == 'retained':
        pin = RETAINED[key]                      # KeyError: only 02/03/13/15 can be retained
        assert set(row) == ROW_KEYS | IMPORT_KEYS | {'retention_decision'}
        assert (worker, job, inputs, permit) == (pin['worker'], pin['job'], pin['inputs'], pin['permit'])
        assert sha(permit) == pin['permit_sha256']
        host, binary, table = pin['host'], BINARY_19, BASELINE/(key+'.timeline.tsv')
    else:
        assert kind == 'combined', kind
        dashed = key.replace('_', '-')
        if worker.parent == Q and worker.name.startswith('csd-reviewed-final'):
            host, pattern = D, 'csd-reviewed-final-'
        elif worker.parent == CSS/'qualification01' and worker.name.startswith('css-final-repair'):
            host, pattern = CSS, 'css-reviewed-final-'
        else:
            raise AssertionError('not a combined-repair chapter worker: '+str(worker))
        assert re.fullmatch(re.escape(pattern+dashed)+'[0-9]+', job), job
        assert set(row) == ROW_KEYS | (set() if host == D else IMPORT_KEYS)
        binary, table = BINARY_268, TIMELINES/(key+'.timeline.tsv')
    module = load(worker, key)
    assert module.BINARY_SHA == binary and sha(module.BIN) == binary
    root = module.CSS if (kind, host) == ('combined', CSS) else module.ROOT    # 290 names its root CSS
    assert root == host, str(root)
    files = {worker, inputs, permit, module.BIN}
    if host != D:
        files |= imported(row)
    source = host/'jobs'/job
    receipt = source/'resource-receipt.json'
    state = success(receipt)
    command = [str(item) for item in state['command']]
    assert str(worker) in command
    assert (flag(command, '--chapter'), flag(command, '--job'), flag(command, '--inputs'),
            flag(command, '--permit')) == (key, job, str(inputs), str(permit))
    assert sha(inputs) == state['inputs_sha256']
    frozen = read(inputs)['files']
    bound = {item['path']: item['sha256'] for item in frozen}
    assert len(bound) == len(frozen)
    for path, digest in bound.items():
        assert sha(path) == digest, path
    assert bound.get(str(worker)) == sha(worker) and bound.get(str(permit)) == sha(permit)
    files.update(Path(path) for path in bound)
    files.add(receipt)
    summary_path = source/'render-summary.json'
    summary = read(summary_path)
    files.add(summary_path)
    review = read(permit)
    frames = review['frames']
    assert frames == frames_of[key]
    assert (summary['chapter'], summary['frames'], summary['binary_sha256'], summary['permit_sha256']) == \
        (key, frames, binary, sha(permit))
    outputs = {item['path']: item['sha256'] for item in summary['files']}
    for path, digest in outputs.items():
        assert sha(path) == digest, path
        files.add(Path(path))
    files |= permit_gate(key, review, binary, table)
    if kind == 'combined':
        assert review['source_binding_evidence']
        for item in review['source_binding_evidence']:
            assert sha(item['path']) == item['sha256'], item['path']
            files.add(Path(item['path']))
        files.update(module.preview_binding(key, review))
        if host == D:
            # The CSD worker's own check(), re-run here as 184 did. A CSS worker's check() first
            # asserts the CSS-only alias, so that row rests on its job's guarded pre- and post-render
            # check() plus the host-independent gates above and below.
            again = module.check(key, permit)
            assert (again[0], again[1]) == (review, sha(permit))
    else:
        assert module.check(key, permit) == (review, sha(permit))
        files |= retention(key, row, pin)
    render = source/'render'
    module.verify_tables(key, render, frames)
    files |= schedule(key, render, Path(module.NARRATION), frames)
    movie = render/(key+'.mp4')
    if kind == 'retained':
        assert sha(movie) == pin['movie_sha256'] and frames == pin['frames']
    captures = sorted((render/'soundtrack'/key).glob('soundtrack_*.wav'), key=lambda p: int(p.stem.split('_')[-1]))
    assert captures
    capture = captures[-1]
    with wave.open(str(capture)) as stream:
        assert (stream.getframerate(), stream.getnchannels(), stream.getsampwidth(),
                stream.getnframes()) == (48000, 2, 2, frames*800)
    staged = [render/(key+suffix) for suffix in SUFFIXES] + [capture]
    device = (D/'jobs').stat().st_dev
    for path in staged:
        assert outputs.get(str(path)) == sha(path) and path.stat().st_dev == device, str(path)
    record = dict(chapter=key, kind=kind, host=str(host), job=job, worker=str(worker),
                  worker_sha256=sha(worker), permit=str(permit), permit_sha256=sha(permit),
                  binary_sha256=binary, narration=str(module.NARRATION), frames=frames,
                  movie_sha256=sha(movie), staged=[str(p) for p in staged])
    if kind == 'retained':
        record['retention_decision'] = row['retention_decision']
    if host != D:
        record['import_receipt'] = row['import_receipt']
    return record, {Path(p) for p in files}


def gates(manifest):
    assert manifest.parent == Q
    rows = read(manifest)['chapters']
    registry, frames_of, files = film()
    assert [row['chapter'] for row in rows] == registry, 'manifest rows must be the registry, in order'
    assert len({row['job'] for row in rows}) == 15
    for row in rows:
        assert row['chapter'] not in CHANGED or row['kind'] == 'combined', row['chapter']+' changed in 268; never retained'
    files.update((manifest, Path(__file__).resolve()))
    records, start = [], 0
    for row in rows:
        record, used = chapter_gate(row, frames_of)
        record['start_frame'] = start
        start += record['frames']
        records.append(record)
        files |= used
    assert start == TOTAL_FRAMES
    return registry, records, {Path(p) for p in files}


def check(args):
    manifest = absolute(args.manifest)
    try:
        registry, frames_of, _ = film()
    except Exception as error:                   # read-only report
        print(json.dumps(dict(film='refused', error=repr(error))))
        return 1
    print(json.dumps(dict(film='ok', chapters=len(registry), frames=sum(frames_of.values()))))
    refused = 0
    for row in read(manifest)['chapters']:
        try:
            record, _ = chapter_gate(row, frames_of)
            print(json.dumps(dict(chapter=row['chapter'], kind=record['kind'], ok=True)))
        except Exception as error:
            refused += 1
            print(json.dumps(dict(chapter=row.get('chapter'), ok=False, error=repr(error))))
    return 1 if refused else 0


def launch(args):
    manifest = absolute(args.manifest)
    assert manifest.parent == Q and 0 <= args.wait_hours <= 48
    job = D/'jobs'/JOB
    assert not job.exists() and not INPUTS.exists(), 'master293 job or inputs already exist; never duplicate'
    rows = read(manifest)['chapters']
    # Workers are bound through the manifest's worker_sha256; foreign ones exist only after import.
    pins = {p: sha(p) for p in (Path(__file__).resolve(), manifest, GUARD)}
    # Wait on the receipt a row needs last: its CSD import, else its CSD render.
    waits = [Path(r['import_receipt']) if 'import_receipt' in r else D/'jobs'/r['job']/'resource-receipt.json' for r in rows]
    deadline = time.monotonic() + args.wait_hours*3600
    while True:
        pending = []
        for receipt in waits:
            if not receipt.exists() or not read(receipt).get('finished'):
                pending.append(str(receipt))
                continue
            success(receipt)                     # a finished failure stops here; no retry
        if not pending:
            break
        assert time.monotonic() < deadline, 'not terminal; no master launched: '+', '.join(pending)
        for path, digest in pins.items():
            assert sha(path) == digest, 'changed while waiting: '+str(path)
        time.sleep(30)
    for path, digest in pins.items():
        assert sha(path) == digest, str(path)
    _, _, files = gates(manifest)
    files.update(pins)
    assert not job.exists()
    with INPUTS.open('x') as stream:
        json.dump({'files': [dict(path=str(p), sha256=sha(p)) for p in sorted(files)]}, stream, indent=2)
    subprocess.run([PYTHON, '-I', '-B', str(GUARD), '--phase', 'native', '--job', JOB, '--inputs', str(INPUTS),
                    '--', PYTHON, '-I', '-B', str(Path(__file__).resolve()), 'master', '--job', JOB,
                    '--inputs', str(INPUTS), '--manifest', str(manifest)], check=True)
    success(job/'resource-receipt.json')
    return 0


def master(args):
    """trj-final-master184.py's run(), on CSD, for the rows gates() accepts."""
    assert args.job == JOB and args.inputs == INPUTS
    job = D/'jobs'/JOB
    receipt = read(job/'resource-receipt.json')
    assert receipt['admitted'] and 'exit' not in receipt
    assert receipt['policy']['stop_free_gib'] == 8
    assert receipt['policy']['phase_growth_gib'] == 4
    assert sha(INPUTS) == receipt['inputs_sha256']
    frozen = read(INPUTS)['files']
    bound = {row['path']: row['sha256'] for row in frozen}
    assert len(bound) == len(frozen)
    for path, digest in bound.items():
        assert sha(path) == digest, path

    def require(path):
        assert bound.get(str(path)) == sha(path), str(path)

    manifest = absolute(args.manifest)
    require(Path(__file__).resolve())
    require(manifest)
    registry, records, files = gates(manifest)
    for path in files:
        require(path)
    out = job/'final'
    assert not out.exists()
    out.mkdir()
    links = []

    def link(src, dst):
        require(src)
        dst.parent.mkdir(parents=True, exist_ok=True)
        os.link(src, dst)
        links.append(dict(source=str(src), path=str(dst), sha256=sha(src)))

    total = 0
    for record in records:
        key = record['chapter']
        *tables, capture = [Path(p) for p in record['staged']]
        for src, suffix in zip(tables, SUFFIXES):
            link(src, out/(key+suffix))
        link(capture, out/'soundtrack'/key/'soundtrack_0.wav')
        total += record['frames']
    assert total == TOTAL_FRAMES
    with (job/'staged-inputs.json').open('x') as stream:
        json.dump(links, stream, indent=2)
    os.environ.update(PATH='/usr/bin:/bin', TMPDIR=str(D/'work/tmp'), XDG_CACHE_HOME=str(D/'work/cache'))
    for command, positional in [('master', []), ('captions', ['all'])]:
        argv = [str(BIN), command, *positional, '--narration', str(NARRATION)]
        if command == 'master':
            argv += ['--score', str(STEMS)]
        argv += ['--out', str(out)]
        with (job/(command+'-native.log')).open('x') as log:
            subprocess.run(argv, cwd=D/'work/production-repo07', stdout=log,
                           stderr=subprocess.STDOUT, check=True, timeout=14400)
    replays = [line.split()[0] for line in (job/'master-native.log').read_text().splitlines()
               if "replay = fmn's soundtrack, bit for bit" in line]
    assert replays == registry
    outputs = {key: out/('complex_analysis_visual'+suffix) for key, suffix in
               [('master', '_master.mp4'), ('srt', '.en.srt'), ('vtt', '.en.vtt')]}
    probe = json.loads(subprocess.check_output(['ffprobe', '-v', 'error', '-show_streams',
                                                '-of', 'json', str(outputs['master'])]))
    videos = [x for x in probe['streams'] if x['codec_type'] == 'video']
    audios = [x for x in probe['streams'] if x['codec_type'] == 'audio']
    assert len(videos) == len(audios) == 1
    video, audio = videos[0], audios[0]
    assert (video['width'], video['height'], video['avg_frame_rate'],
            int(video['nb_frames'])) == (3840, 2160, '60/1', total)
    assert (audio['sample_rate'], audio['channels'], audio['codec_name']) == ('48000', 2, 'aac')
    assert abs(float(audio['duration']) - total/60) <= 1024/48000 + .001
    assert outputs['srt'].stat().st_size and outputs['vtt'].read_text().startswith('WEBVTT')
    for item in links:
        assert sha(item['source']) == sha(item['path']) == item['sha256']
    for path, digest in bound.items():
        assert sha(path) == digest
    for record in records:
        del record['staged']
    result = dict(chapters=registry, frames=total, native_bit_identical_replays=len(replays),
                  outputs={key: dict(path=str(path), sha256=sha(path)) for key, path in outputs.items()},
                  version=VERSION, host='csd', binary_sha256=BINARY_268, narration=str(NARRATION),
                  score=str(STEMS), manifest=dict(path=str(manifest), sha256=sha(manifest)), rows=records,
                  full_decode=False, final_picture_review=False, motion_review=False,
                  audio_heard=False, independent_av_acceptance=False,
                  exact_caption_check=False, publication_accepted=False)
    with (job/'master-summary.json').open('x') as stream:
        json.dump(result, stream, indent=2)
    return 0


if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    modes = parser.add_subparsers(dest='mode', required=True)
    one = modes.add_parser('check')
    one.add_argument('--manifest', required=True, type=Path)
    one = modes.add_parser('launch')
    one.add_argument('--manifest', required=True, type=Path)
    one.add_argument('--wait-hours', type=float, default=0)
    one = modes.add_parser('master')
    one.add_argument('--job', required=True)
    one.add_argument('--inputs', required=True, type=Path)
    one.add_argument('--manifest', required=True, type=Path)
    args = parser.parse_args()
    raise SystemExit({'check': check, 'launch': launch, 'master': master}[args.mode](args))
