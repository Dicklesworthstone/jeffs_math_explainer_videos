"""CSS adaptation of the combined-repair scored 4K60 chapter worker, version 352. Prepared only.

Against 309 (css-final-repair309.py, 579fe7ce..., preserved unchanged), only the review gate differs, as in
csd-reviewed-final352.py. 09 is left out (final344 renders it). Like that worker, 352 takes 344's
sampled-preview permit in place of actually_reviewed_motion and motion_intervals:
 - exactly 344's PERMIT_KEYS;
 - 344's sampled_review_scope(), byte for byte;
 - review_basis(), byte for byte as in csd-reviewed-final352.py: frames of the chapter's exact preview
   movie, from records the permit binds;
 - the preview's own sampling.json among the viewed records, and a confirming independent file;
 - playback_seconds [].
independent() is 344's with the files read through the exact alias. The permit transfer also carries the
permit's independent_review files. Permits land in repair352-permits; the names are css-repair-permit352-*
and css-reviewed-final-<chapter>352. 352 reuses 309's completed runtime transfer (css-repair-transfer309)
and narration delivery (css-repair-delivery309): their job names stay 309's, and the runtime and delivery
modes refuse because those jobs already exist. A 309 CSS final of the chapter is refused, like any other
css-reviewed-final job. Every 309 encoder, resource, exclusive-lane, source, table, score, binary, narration,
carried-row and preview gate is kept unchanged. This is not publication authorization, and it accepts no
final picture, motion, audio, fine-sync or caption review.

309's description, which still applies:
Base: css-final-repair290.py (sha256 720e63a66d9e67d04c1cd99d2131d76401e42a5e2d60240fb1afc9d3a93ac8f2),
reviewed as cc2-css-worker-review300.md and kept unchanged. 290's own base is csd-reviewed-final287.py
(f01f4c29049acc86d486d4b083273182febd4cfdf0e7f4b755c9c2fb81247510). The preview contract is the one
CC0's csd-reviewed-final303.py (db3ce8e424f17a604e5f2cc802db81e5dcd7b631301808625e20a7c9b658d057)
names; 303 and its launcher are read here, never changed.
CSS contracts: css-runtime-sender75.py / css-runtime-receiver75.py (frozen copy at the CSD path under
work/frozen-runtime75, reached through the existing alias), css-delivery76.py (narration copy with
resolved source paths, native gate), css-final-chapter78.py and css-launch-final13-96.py (guarded
chapter job), guard_css74.py (10 GiB admission, 8 GiB stop, 4 GiB per job, one exclusive lane).

Every 290 gate is kept: binary 268; timeline269 tables equal to timeline21 after the one narration-path
mapping; score21 files and stem QA; the root-authored permit (pictures and motion actually reviewed, no
unresolved defect, evidence and source_binding_evidence at their hashes, absolute symlink-free path);
the 287 preview binding. Against 290, 309:
  - names CC3's actual preview274 with 303's values and pins both previews' receipt inputs as 303 does,
    so the nine 274 chapters can bind instead of failing closed; carries its job folder, manifest and
    worker, and binds that manifest's rows present on CSS;
  - requires the permit to name host css (303 requires csd: one chapter, one host), a permit file name
    that starts with the chapter key, and writes each permit file once only (review300 L1, L4);
  - writes the manifest with 'x' and lands it on CSS by its sha256; the stream starts with a header
    naming that sha256, which freeze-transfer and receive also require (L2);
  - delivery accepts only a flat folder of regular files (L3);
  - refuses every CSD csd-reviewed-final-<chapter>* job (244/278/287/303) when a permit is carried;
  - launch takes --permit-sha256 and refuses too few CPUs before it writes inputs (as 303's launcher).

Modes. Root runs each one on the named host; nothing here starts a job by itself except the
launch modes, and those start only the one guarded job they name.
  manifest         CSD, read-only except its new --out file: write the transfer manifest for the
                   runtime, or for one permit, and print its sha256.
  land-manifest    CSS: read one manifest from stdin, check its sha256 and version, write it once into
                   qualification01. A banner on the relay's stdout fails here, before any job.
  send             CSD, read-only: re-hash every manifest file, then stream a header and the files.
  freeze-transfer  CSS: write one receive job's guard inputs and print its guard command.
  receive          CSS, inside that guarded job: check the header, stage, verify, place new files only.
  preflight        CSS, read-only: what the CSS gates read, present or missing; with --manifest and
                   --manifest-sha, also whether the landed copy is that manifest.
  launch-delivery  CSS: freeze inputs, run the guarded narration delivery job.
  delivery         CSS, inside that job: rebind269 narration with resolved source paths, native gate.
  launch           CSS: freeze one chapter's inputs, run its guarded render.
  render           CSS, inside that job: the 287 render and checks.
"""
import argparse, datetime, hashlib, json, os, shutil, struct, subprocess, sys, time, wave
from pathlib import Path

VERSION = '352'
WORKER309_SHA = '579fe7cee38af0d9fb249621f53b631645180b5799fc323e6a1ddd157814d18a'   # css-final-repair309.py, the base
HELPER290_SHA = '720e63a66d9e67d04c1cd99d2131d76401e42a5e2d60240fb1afc9d3a93ac8f2'
WORKER287_SHA = 'f01f4c29049acc86d486d4b083273182febd4cfdf0e7f4b755c9c2fb81247510'
WORKER303_SHA = 'db3ce8e424f17a604e5f2cc802db81e5dcd7b631301808625e20a7c9b658d057'   # preview contract source
HOST = 'css'                                                 # 303 requires 'csd': a permit serves one host
CSS = Path('/data/projects/jmev-css-render-worker-20261107')
D = Path('/data/projects/jmev-stroke-probe-20261106')        # on CSS: the existing alias into FROZEN, never replaced
FROZEN = CSS/'work/frozen-runtime75'
DQ = D/'qualification01/production-repair17'                 # 287's Q
CQ = CSS/'qualification01'
PERMITS = CQ/'repair352-permits'                             # a permit must sit on a symlink-free path
GUARD = CSS/'guard_css74.py'
PYTHON = '/usr/bin/python3'
BIN = D/'work/bin/review-repair268'
BINARY_SHA = '983954c508e42b90d443ad6ca09f2baba7565a4f609d09d2ea821640534d6ec7'
WORKSPACE = D/'work/review-repair268/workspace'
TOPIC = WORKSPACE/'explainer_corpus/complex_analysis_visual'
TIMELINES = D/'jobs/csd-review-repair-timeline269/timeline-output'
TIMELINE269_INPUTS = DQ/'review-repair-timeline269-inputs.json'
BASELINE = D/'jobs/csd-production-timeline21/timeline-output'
ORIGINAL_NARRATION = D/'jobs/csd-production-assembly19/narration'
FMD_NARRATION = D/'jobs/csd-review-repair-rebind269/narration'
REBIND269 = D/'jobs/csd-review-repair-rebind269'
STEMS = D/'jobs/csd-production-score21/score/stems'
SCORE = D/'jobs/csd-production-score21'
BUILD268 = D/'jobs/csd-review-repair-build268'
TRANSFER75 = CSS/'jobs/css-runtime-transfer75'
MANIFEST75 = CQ/'css-runtime-manifest75.json'
ENCODER_STOP = CSS/'jobs/css-encoder-stop79'                 # CSS's counterpart of csd-raster-encoder-stop13
TRANSFER = CSS/'jobs/css-repair-transfer309'
DELIVERY = CSS/'jobs/css-repair-delivery309'
NARRATION = DELIVERY/'narration'
DELIVERY_INPUTS = CQ/'css-repair-delivery309-inputs.json'
CHAPTERS = ('01_the_bump', '04_reading_color', '05_gallery', '06_one_derivative', '07_one_point', '08_loop_sums', '10_rigidity', '11_counting', '12_residues', '14_continuation')
# 352: each chapter's preview movie as reviewed (csd-reviewed-final352.py MOVIES, value for value).
MOVIES={'01_the_bump':'1f6834a6b6fb5ed481f66c0a8eded204ede670501eae5d81a277814206353d19',
 '04_reading_color':'8836b7e6d87b170d9cb7327fd51fd3e93fb7b5bfd2b2a89605d7cc1ed2d7e890',
 '05_gallery':'f5123300084eba264276377f36518d263324bb9e28dd5df0f77ecf5f20cf5e48',
 '06_one_derivative':'5e45d72da91701c9c7c346b98ded8491c13ba1befed6be1b6301c1c6d33ed62d',
 '07_one_point':'2517772cb632db4eafe5be1ed266e6e8aa596778d7f3742ecdab8fa5d204b7b0',
 '08_loop_sums':'1ac1ee716f6aa383a9154d6305ffa571bd555df818565d28a07084eb4b5364d6',
 '10_rigidity':'78eb267faa780b2a4d38804deede56088852fed606ac544b7d05f02300672562',
 '11_counting':'0b6244a005f64ff9b9b381a8ec8a58ffb7c174d085aceb452871d7f5dd0f811e',
 '12_residues':'a78c20024c257eaa4783db80f9b99c23442fe67ac8f646e354e633074f842199',
 '14_continuation':'a0d34489e9757ff3db96e3d8b7a438845a90036453b2d4d7c9426efd8dc7c432'}
# 344's permit fields, exactly (csd-reviewed-boundary-final344.py PERMIT_KEYS).
PERMIT_KEYS={'chapter','host','binary_sha256','preview_job','frames','timeline_sha256','actually_reviewed_pictures',
 'unresolved_defects','reviewer','observations','evidence','source_binding_evidence','independent_reviewer',
 'independent_review','review_scope','continuous_motion_acceptance','listening_acceptance','fine_sync_acceptance',
 'final_4k_acceptance','release_acceptance','publication_acceptance'}
KINDS = ('timeline', 'cues', 'cuelog')
SCENE_DIR = 'explainer_corpus/complex_analysis_visual/src/chapters/'
SCENES = {'01_the_bump':'the_bump.rs','04_reading_color':'reading_color.rs','05_gallery':'gallery.rs','06_one_derivative':'one_derivative.rs','07_one_point':'one_point.rs','08_loop_sums':'loop_sums.rs','09_boundary':'boundary.rs','10_rigidity':'rigidity.rs','11_counting':'counting.rs','12_residues':'residues.rs','14_continuation':'continuing.rs'}
# 303's preview names and receipt-inputs pins, value for value (read on csd 2026-11-08).
PREVIEW243 = dict(job='csd-review-repair-preview243', manifest='review-repair-preview243-inputs.json',
    worker='review-repair-preview243.py', worker_sha256='3e5dd06c48b888e4a947a71f57929121fdb76590b0a6e5dfc3e21bd5f1e23c17',
    tables='preview', narration=D/'jobs/csd-review-repair-rebind242/narration',
    binary=D/'work/bin/review-repair238', binary_sha256='f31690e648393923c373922b513a77e65dc44fef10dbd39f8b88b561e283e9da',
    build=D/'jobs/csd-review-repair-build238', source_copy_sha256='6d4c7e93e89a29930cf5b4c2f3101fba023a3e09b99888f3e9db083bdb1eb65c',
    inputs_sha256='6510db7856deec2abfea54d585000ec09d0cf5334ff32b558adb71172a8dcf91')
SOURCE_COPY_268_SHA = '81d28272a7a23b63199e79e6aa1086f01f131f0ddd685d4ae859d04b1de1934c'
# CC3's cc3-preview-fade274.py run by root's launch-reviewed-preview296.py: receipt exit 0 at
# 10:50:17Z, inputs = final inputs; the same binary and build as the final, rebind269 narration.
PREVIEW274 = dict(job='csd-review-repair-preview274', manifest='review-repair-preview274-inputs.json',
    worker='cc3-preview-fade274.py', worker_sha256='d80cf7b6ffba56968f7c6dfd7a2cca50e65769dce1c97deb5d34683ed3fc9669',
    tables='preview', narration=D/'jobs/csd-review-repair-rebind269/narration',
    binary=BIN, binary_sha256=BINARY_SHA, build=BUILD268, source_copy_sha256=SOURCE_COPY_268_SHA,
    inputs_sha256='20d91bdb8585fb229c105c717a0cf486e544b3b37b92f46907e009fe695fb38c')
PREVIEWS = {c: (PREVIEW243 if c in ('06_one_derivative', '08_loop_sums') else PREVIEW274) for c in CHAPTERS}
DELTA_NAME = 'build238-to-build268-delta'
DELTA_EVIDENCE = DQ/'cc1-preview-to-final-binding282.md'
DELTA_EVIDENCE_SHA = '1fbcedef34f65933ff3d818740d124cb2492a39e276916eb600f6753d5c62066'
DELTA_238_268 = frozenset(SCENE_DIR+n for n in ('draw.rs', 'gallery.rs', 'mod.rs', 'plate_fade_probe.rs', 'rigidity.rs'))
# CSD job folders copied whole: what check()/preview_binding() read, and what 287's and 303's
# launchers bind.
JOB_DIRS = (BUILD268, PREVIEW243['build'], REBIND269, TIMELINES.parent, D/'jobs'/PREVIEW243['job'], D/'jobs'/PREVIEW274['job'])
# CSD input manifests whose rows bind the CSD runtime; on CSS every row present must match.
CSD_MANIFESTS = (TIMELINE269_INPUTS, DQ/PREVIEW243['manifest'], DQ/PREVIEW274['manifest'])
MANIFEST_LIMIT = 64 << 20                                    # land-manifest reads at most this much


def sha(path):
    with Path(path).open('rb') as f:
        return hashlib.file_digest(f, 'sha256').hexdigest()


def read(path):
    return json.loads(Path(path).read_text())


def success(path):
    r = read(path)
    assert r.get('finished') and r.get('exit') == 0 and r['inputs_sha256'] == r['final_inputs_sha256'], str(path)
    assert not any(r.get(k) for k in ('error', 'resource_stop', 'stopped_for_resource_limit')), str(path)
    return r


def job_name(chapter):
    return 'css-reviewed-final-'+chapter.replace('_', '-')+VERSION+'-ssd440'


def permit_job_prefix(chapter):
    return 'css-repair-permit'+VERSION+'-'+chapter.replace('_', '-')


def permit_name_ok(chapter, name):
    """Review300 L1: one file name per chapter and issue. The name starts with the chapter key, and
    receive never lets a permit file be written twice, so a re-issue needs a new name."""
    return name.startswith(chapter+'-') and name.endswith('.json') and '/' not in name


def stream_header(digest):
    return ('css-final-repair%s manifest %s\n' % (VERSION, digest)).encode()


def absolute_permit(permit):
    permit = Path(permit)
    assert permit.is_absolute() and permit == permit.resolve(strict=True), 'permit path must be absolute and free of symlinks: '+str(permit)
    return permit


def alias_is_exact():
    assert D.is_symlink() and D.resolve(strict=True) == FROZEN/str(D).lstrip('/'), 'Expected the existing exact CSD alias; never replace it'


def destination(source, kind):
    """Where a CSD file lives on CSS: under FROZEN at its own absolute path, so the alias serves it at
    the path every gate names; a permit instead directly under PERMITS."""
    source = Path(source)
    assert source.is_absolute() and '..' not in source.parts and source.is_relative_to(D), 'not a CSD file: '+str(source)
    return PERMITS/source.name if kind == 'permit' else FROZEN/str(source).lstrip('/')


def tables(folder, chapters):
    return [folder/(c+'.'+k+'.tsv') for c in chapters for k in KINDS]


def files_under(folder):
    return sorted(p for p in Path(folder).rglob('*') if p.is_file())


# ---------------------------------------------------------------- CSD side (read-only)

def runtime_sources():
    """Every CSD file a CSS gate reads that runtime75 did not carry (notes, copy map)."""
    out = [BIN, PREVIEW243['binary']] + files_under(WORKSPACE)
    for folder in JOB_DIRS:
        out += files_under(folder)
    out += [TIMELINE269_INPUTS] + [DQ/p[k] for p in (PREVIEW243, PREVIEW274) for k in ('manifest', 'worker')] + [DELTA_EVIDENCE]
    return out + tables(BASELINE, read(BUILD268/'qualification.json')['chapters'])


def runtime_problems():
    problems = []
    def need(condition, message):
        if not condition:
            problems.append(message)
    for folder in JOB_DIRS:
        try:
            success(folder/'resource-receipt.json')
        except (AssertionError, OSError, KeyError, ValueError) as error:
            problems.append('not a clean finished job: %s (%r)' % (folder, error))
    if problems:
        return problems
    q = read(BUILD268/'qualification.json')
    need(sha(BIN) == BINARY_SHA == q['binary_sha256'], 'binary 268 hash')
    need(sha(BUILD268/'source-copy.json') == SOURCE_COPY_268_SHA, 'build268 source copy hash')
    need((TIMELINES.parent/'comparison.json').is_file(), 'timeline269 comparison.json absent')
    need(sha(TIMELINE269_INPUTS) == read(TIMELINES.parent/'resource-receipt.json')['inputs_sha256'], 'timeline269 manifest is not the one its receipt names')
    # The bytes CSS receives are the bytes the CSD jobs froze, which 287 and 303 bind on CSD.
    carried = [(TIMELINE269_INPUTS, [BIN] + files_under(WORKSPACE) + files_under(REBIND269) + files_under(BUILD268))]
    carried += [(DQ/p['manifest'], [p['binary'], DQ/p['worker']] + files_under(p['build'])) for p in (PREVIEW243, PREVIEW274)]
    for manifest, files in carried:
        rows = {x['path']: x['sha256'] for x in read(manifest)['files']}
        for x in files:
            need(rows.get(str(x)) == sha(x), 'not as frozen by %s: %s' % (manifest.name, x))
    for key in q['chapters']:
        for t in tables(TIMELINES, [key]):
            need(t.exists() and t.read_bytes() == (BASELINE/t.name).read_bytes().replace(str(ORIGINAL_NARRATION).encode(), str(FMD_NARRATION).encode()),
                 'timeline269 table absent or not timeline21 after the mapping: '+t.name)
    for p in (PREVIEW243, PREVIEW274):
        name = p['job']
        preview = read(D/'jobs'/name/'resource-receipt.json')
        need(preview['inputs_sha256'] == p['inputs_sha256'], name+' is not the run that was read (303 pin)')
        need(sha(DQ/p['manifest']) == preview['inputs_sha256'], name+' manifest is not the one its receipt names')
        for chapter in (c for c in CHAPTERS if PREVIEWS[c] is p):
            for t in tables(D/'jobs'/name/p['tables'], [chapter]):
                need(t.is_file() and TIMELINES.joinpath(t.name).exists() and t.read_bytes().replace(str(p['narration']).encode(), str(FMD_NARRATION).encode()) == (TIMELINES/t.name).read_bytes(),
                     name+' table differs from timeline269: '+t.name)
        need(sha(DQ/p['worker']) == p['worker_sha256'] and str(DQ/p['worker']) in preview['command'], name+' worker')
        need(sha(p['binary']) == p['binary_sha256'] == read(p['build']/'qualification.json')['binary_sha256'], name+' binary')
        need(sha(p['build']/'source-copy.json') == p['source_copy_sha256'], name+' build source copy')
    need(DELTA_EVIDENCE.is_file() and sha(DELTA_EVIDENCE) == DELTA_EVIDENCE_SHA, 'binding282 is not in CSD Q at its pinned hash')
    for x in runtime_sources():
        need(x.is_file() and not x.is_symlink(), 'missing or symlinked source: '+str(x))
    return problems


def permit_sources(chapter, permit):
    permit = absolute_permit(permit)
    review = read(permit)
    assert chapter in CHAPTERS and review['chapter'] == chapter and review['binary_sha256'] == BINARY_SHA
    assert review.get('host') == HOST, 'permit must name host css (one chapter, one host)'
    assert permit_name_ok(chapter, permit.name), 'permit file name must start with "%s-" and end in .json' % chapter
    assert review['preview_job'] == PREVIEWS[chapter]['job']
    assert review['timeline_sha256'] == sha(TIMELINES/(chapter+'.timeline.tsv'))
    # 287 item 5 where CSD's jobs are visible: no CSD final job of this chapter (244, 278, 287, 303 or
    # later) may exist when its permit is carried to CSS. A point-in-time check; the host field is
    # the lasting one, since 303 refuses a permit that does not name csd.
    others = sorted(p.name for p in (D/'jobs').glob('csd-reviewed-final-'+chapter.replace('_', '-')+'*'))
    assert not others, 'chapter already has a CSD final job: '+', '.join(others)
    pairs = [(permit, 'permit')]
    for item in review['evidence'] + review['source_binding_evidence'] + review['independent_review']:
        assert sha(item['path']) == item['sha256'], item['path']
        pairs.append((Path(item['path']), 'runtime'))
    return pairs


def manifest_rows(pairs):
    seen, rows = set(), []
    for source, kind in pairs:
        source = Path(source)
        if str(source) in seen:
            continue
        seen.add(str(source))
        assert source.is_file() and not source.is_symlink(), str(source)
        st = source.stat()
        rows.append(dict(source=str(source), dest=str(destination(source, kind)), sha256=sha(source), bytes=st.st_size, mode=st.st_mode & 0o777))
    return rows


def manifest(args):
    out = Path(args.out)
    assert out.is_absolute() and out.parent == DQ and out.suffix == '.json' and not out.exists() and not out.is_symlink(), \
        'manifest --out must be a new .json file directly in '+str(DQ)
    if args.permit:
        rows = manifest_rows(permit_sources(args.chapter, args.permit))
        extra = dict(kind='permit', chapter=args.chapter, permit=str(args.permit))
    else:
        problems = runtime_problems()
        if problems:
            print('\n'.join(['runtime manifest refused:'] + problems), file=sys.stderr)
            return 1
        rows = manifest_rows([(x, 'runtime') for x in runtime_sources()])
        extra = dict(kind='runtime')
    text = json.dumps(dict(version=VERSION, derived_from_helper290_sha256=HELPER290_SHA, derived_from_worker287_sha256=WORKER287_SHA,
        utc=datetime.datetime.now(datetime.timezone.utc).isoformat(), bytes=sum(x['bytes'] for x in rows), files=rows, **extra), indent=1)+'\n'
    with out.open('x') as f:                             # noclobber (review300 L2)
        f.write(text)
    print(sha(out), out)
    return 0


def land_manifest(args):
    """CSS: the manifest arrives on stdin (for example `ssh csd cat <it>`). Nothing is written unless
    the whole stream hashes to --manifest-sha, so a login banner on either relay fails here, before a
    transfer job name is used."""
    out = Path(args.out)
    assert out.is_absolute() and out.parent == CQ and out.suffix == '.json' and not out.exists() and not out.is_symlink(), \
        'land-manifest --out must be a new .json file directly in '+str(CQ)
    data = sys.stdin.buffer.read(MANIFEST_LIMIT+1)
    assert len(data) <= MANIFEST_LIMIT, 'manifest stream too large'
    assert hashlib.sha256(data).hexdigest() == args.manifest_sha, 'stream is not the named manifest (banner on stdout, or another file)'
    assert json.loads(data)['version'] == VERSION
    with out.open('xb') as f:
        f.write(data)
    assert sha(out) == args.manifest_sha
    print('landed', out, args.manifest_sha)
    return 0


def send(args):
    assert sha(args.manifest) == args.manifest_sha, 'manifest is not the one named'
    m = read(args.manifest)
    assert m['version'] == VERSION
    for x in m['files']:                                 # every file, before the first byte
        p = Path(x['source'])
        assert not p.is_symlink() and p.stat().st_size == x['bytes'] and sha(p) == x['sha256'], str(p)
    out = sys.stdout.buffer
    out.write(stream_header(args.manifest_sha))
    for x in m['files']:
        h, n = hashlib.sha256(), 0
        with Path(x['source']).open('rb') as f:
            while b := f.read(1 << 20):
                h.update(b)
                n += len(b)
                out.write(b)
        assert n == x['bytes'] and h.hexdigest() == x['sha256'], x['source']
    out.flush()
    return 0


# ---------------------------------------------------------------- CSS side

def admitted(job):
    a = read(Path('/data/ssd/work/jmev-gallery-recovery440/jobs')/job/'resource-receipt.json')
    assert a['admitted'] and 'exit' not in a
    assert a['policy']['stop_free_gib'] == 8 and a['policy']['phase_growth_gib'] == 4 and a['policy']['admission_gib'] == 10
    return a


def frozen_inputs(job, inputs):
    a = admitted(job)
    assert sha(inputs) == a['inputs_sha256']
    rows = read(inputs)['files']
    assert len(rows) == len({x['path'] for x in rows})
    for x in rows:
        assert sha(x['path']) == x['sha256'], x['path']
    return {x['path']: x['sha256'] for x in rows}


def write_inputs(path, files):
    with Path(path).open('x') as f:
        json.dump({'files': [dict(path=str(p), sha256=sha(p)) for p in sorted(set(map(Path, files)))]}, f, indent=2)
    return Path(path)


def manifest_kind(m, job):
    if m['kind'] == 'runtime':
        assert job == TRANSFER.name and not any(Path(x['dest']).parent == PERMITS for x in m['files'])
    else:
        assert m['kind'] == 'permit' and m['chapter'] in CHAPTERS and job.startswith(permit_job_prefix(m['chapter']))
        assert [x['source'] for x in m['files'] if Path(x['dest']).parent == PERMITS] == [m['permit']]
        assert permit_name_ok(m['chapter'], Path(m['permit']).name)
    return m['kind']


def freeze_transfer(args):
    assert not (CSS/'jobs'/args.job).exists()
    m = Path(args.manifest)
    assert m.is_absolute() and m.parent == CQ and read(m)['version'] == VERSION
    assert sha(m) == args.manifest_sha, 'the CSS copy is not the manifest named'
    manifest_kind(read(m), args.job)
    inputs = write_inputs(CQ/(args.job+'-inputs.json'), [Path(__file__).resolve(strict=True), m, GUARD])
    print(json.dumps([PYTHON, '-I', '-B', str(GUARD), '--phase', 'native', '--job', args.job, '--inputs', str(inputs), '--',
        PYTHON, '-I', '-B', str(Path(__file__).resolve()), 'receive', '--job', args.job, '--manifest', str(m), '--manifest-sha', args.manifest_sha]))
    return 0


def receive(args):
    job = CSS/'jobs'/args.job
    bound = frozen_inputs(args.job, CQ/(args.job+'-inputs.json'))
    assert str(Path(__file__).resolve()) in bound and str(args.manifest) in bound, 'helper and manifest must be frozen'
    assert bound[str(args.manifest)] == args.manifest_sha, 'the frozen manifest is not the one named'
    alias_is_exact()
    m = read(args.manifest)
    assert m['version'] == VERSION
    manifest_kind(m, args.job)
    plan = []
    for x in m['files']:                                 # every destination, before the first byte
        dest = Path(x['dest'])
        assert dest == destination(x['source'], 'permit' if dest.parent == PERMITS else 'runtime'), str(dest)
        if dest.parent == PERMITS:
            assert not dest.exists() and not dest.is_symlink(), 'a permit file is written once; re-issue it under a new name: '+str(dest)
        if dest.exists() or dest.is_symlink():
            assert not dest.is_symlink() and sha(dest) == x['sha256'], 'existing destination differs: '+str(dest)
        plan.append((x, dest, not dest.exists()))
    header = stream_header(args.manifest_sha)            # the sender hashed this same manifest
    assert sys.stdin.buffer.read(len(header)) == header, 'stream does not start with this manifest header'
    # Phase 1: stage every new file inside this job, so a broken stream leaves FROZEN untouched.
    incoming = job/'incoming'
    incoming.mkdir(mode=0o700)
    for i, (x, dest, new) in enumerate(plan):
        remaining, h = x['bytes'], hashlib.sha256()
        with ((incoming/('%05d' % i)).open('xb') if new else open(os.devnull, 'wb')) as f:
            while remaining:
                b = sys.stdin.buffer.read(min(remaining, 1 << 20))
                assert b, 'Truncated transfer at '+x['source']
                f.write(b)
                h.update(b)
                remaining -= len(b)
        assert h.hexdigest() == x['sha256'], x['source']
    assert not sys.stdin.buffer.read(1), 'Unexpected trailing data'
    # Phase 2: place the verified files at new paths only.
    rows = []
    for i, (x, dest, new) in enumerate(plan):
        if new:
            dest.parent.mkdir(mode=0o700, parents=True, exist_ok=True)
            with (incoming/('%05d' % i)).open('rb') as src, dest.open('xb') as dst:
                shutil.copyfileobj(src, dst, 1 << 20)
            os.chmod(dest, x['mode'])
        assert sha(dest) == x['sha256'], str(dest)
        rows.append(dict(source=x['source'], path=str(dest), sha256=x['sha256'], bytes=x['bytes'], written=new))
    with (job/'transfer-result.json').open('x') as f:
        json.dump(dict(manifest=str(args.manifest), manifest_sha256=sha(args.manifest), kind=m['kind'], files=rows,
            written=sum(r['written'] for r in rows), already_present=sum(not r['written'] for r in rows), all_hashes_match=True,
            scope='Frozen copy only; no render, picture, motion or audio acceptance.'), f, indent=2)
    print('Verified', len(rows), 'files;', sum(r['written'] for r in rows), 'written')
    return 0


def frozen75_paths():
    return [FROZEN/x['path'].lstrip('/') for x in read(MANIFEST75)['files']]


def launch_delivery(args):
    assert not DELIVERY.exists()
    success(TRANSFER75/'resource-receipt.json')
    success(TRANSFER/'resource-receipt.json')
    result = read(TRANSFER/'transfer-result.json')
    assert result['kind'] == 'runtime' and result['all_hashes_match']
    files = [Path(__file__).resolve(strict=True), GUARD, TRANSFER75/'resource-receipt.json', MANIFEST75, TRANSFER/'resource-receipt.json',
             TRANSFER/'transfer-result.json', Path(result['manifest'])] + frozen75_paths() + [Path(r['path']) for r in result['files']]
    inputs = write_inputs(DELIVERY_INPUTS, files)
    subprocess.run([PYTHON, '-I', '-B', str(GUARD), '--phase', 'native', '--job', DELIVERY.name, '--inputs', str(inputs), '--',
        PYTHON, '-I', '-B', str(Path(__file__).resolve()), 'delivery'], check=True)
    return 0


def delivery(args):
    """css-delivery76.py for rebind269 and binary 268, with stricter equality checks."""
    frozen_inputs(DELIVERY.name, DELIVERY_INPUTS)
    success(TRANSFER75/'resource-receipt.json')
    success(TRANSFER/'resource-receipt.json')
    alias_is_exact()
    os.environ.update(PATH='/usr/bin:/bin', TMPDIR=str(CSS/'work/tmp'), XDG_CACHE_HOME=str(CSS/'work/cache'))
    flat = sorted(FMD_NARRATION.iterdir())               # review300 L3: regular files only, no folders or links
    assert flat and all(p.is_file() and not p.is_symlink() for p in flat), 'rebind269 narration must be a flat folder of regular files'
    shutil.copytree(FMD_NARRATION, NARRATION)
    report = read(NARRATION/'report.json')
    changes = []
    for cue, rows in report['lines'].items():
        for row in rows:
            old = Path(row['source'])
            new = old.resolve(strict=True)
            assert old.is_relative_to(D) and new == D.resolve()/old.relative_to(D)
            assert sha(new) == row['source_sha256']
            if str(old) != str(new):
                changes.append(dict(cue=cue, old=str(old), new=str(new)))
                row['source'] = str(new)
    (NARRATION/'report.json').write_text(json.dumps(report, indent=2)+'\n')
    assembly = read(NARRATION/'assembly.json')
    assembly['report_sha256'] = sha(NARRATION/'report.json')
    (NARRATION/'assembly.json').write_text(json.dumps(assembly, indent=2)+'\n')
    prior = read(FMD_NARRATION/'report.json')
    assert report['lines'].keys() == prior['lines'].keys()
    for cue, rows in report['lines'].items():
        assert len(rows) == len(prior['lines'][cue])
        for row, before in zip(rows, prior['lines'][cue]):
            assert {k: v for k, v in row.items() if k != 'source'} == {k: v for k, v in before.items() if k != 'source'}
    assert {k: v for k, v in report.items() if k != 'lines'} == {k: v for k, v in prior.items() if k != 'lines'}
    before = read(FMD_NARRATION/'assembly.json')
    assert {k: v for k, v in assembly.items() if k != 'report_sha256'} == {k: v for k, v in before.items() if k != 'report_sha256'}
    names = [p.name for p in flat]
    copied = sorted(NARRATION.iterdir())
    assert [p.name for p in copied] == names and all(p.is_file() and not p.is_symlink() for p in copied)
    for name in names:
        if name not in ('report.json', 'assembly.json'):
            assert sha(FMD_NARRATION/name) == sha(NARRATION/name), name
    wavs = [n for n in names if n.endswith('.wav')]
    assert len(wavs) == 105 and all(sha(NARRATION/n) == sha(ORIGINAL_NARRATION/n) for n in wavs), 'not the 105 Frederick WAVs of assembly19'
    q = read(BUILD268/'qualification.json')
    assert sha(BIN) == q['binary_sha256'] == BINARY_SHA
    for name in ('facts', 'chapters'):
        with (DELIVERY/(name+'.stdout')).open('xb') as o, (DELIVERY/(name+'.stderr')).open('xb') as e:
            subprocess.run([str(BIN), name], cwd=D/'work/production-repo07', stdout=o, stderr=e, check=True, timeout=120)
    assert sha(DELIVERY/'facts.stdout') == q['facts_sha256'] and (DELIVERY/'chapters.stdout').read_text().splitlines() == q['chapters']
    command = [PYTHON, '-I', '-B', str(WORKSPACE/'tools/narration_delivery.py'), '--topic', str(TOPIC), '--narration', str(NARRATION),
               '--consumer', 'render', '--gain-db', '0']
    for c in q['chapters']:
        command += ['--chapter-key', c]
    with (DELIVERY/'delivery.stdout').open('xb') as o, (DELIVERY/'delivery.stderr').open('xb') as e:
        subprocess.run(command, cwd=D/'work/production-repo07', stdout=o, stderr=e, check=True, timeout=1200)
    x = read(DELIVERY/'delivery.stdout')
    assert x['verified_cues'] == x['selected_cues'] == 105 and x['plan_mode'] == 'external-ui'
    with (DELIVERY/'delivery-result.json').open('x') as f:
        json.dump(dict(binary_sha256=BINARY_SHA, native_facts_and_registry_match=True, delivery=x, path_rebindings=changes, source=str(FMD_NARRATION),
            scope='Resolved alias paths only; 105 WAVs, sentence rows and phrase marks unchanged; native delivery passed. No picture or listening acceptance.'), f, indent=2)
    return 0


def carried_rows():
    """CSS's form of 287's and 303's inherited-manifest binding: every row of the CSD timeline269,
    preview243 and preview274 manifests that exists on CSS must match it, and the runtime 268 reads
    must be among them."""
    present = []
    for manifest in CSD_MANIFESTS:
        for x in read(manifest)['files']:
            p = Path(x['path'])
            if p.exists():
                assert sha(p) == x['sha256'], 'CSS copy differs from the CSD manifest row: '+str(p)
                present.append(p)
    runtime = [BIN] + files_under(WORKSPACE) + files_under(REBIND269) + files_under(BUILD268)
    missing = set(map(str, runtime)) - set(map(str, present))
    assert not missing, 'runtime files absent from the timeline269 rows on CSS: '+', '.join(sorted(missing)[:5])
    return sorted(set(present))


def sampled_review_scope(review, frames, movie_sha256, sampled):
    """344's review scope, the part shared with later workers and masters. The permit states what was
    actually reviewed, never a motion boolean. It authorizes a technical 4K render from a sampled-preview
    disposition only, with final-picture, motion and audio review still open. Any key that claims an
    acceptance, a release or a publication and is not exactly false is refused.
    review: the permit; frames: the chapter's frames (60 fps); movie_sha256: the preview movie that was
    reviewed; sampled: the frame numbers of that preview's own sampled pictures. Returns the sampled
    intervals. Builtins only, so a later worker or master copies it unchanged and proves that by AST."""
    assert 'actually_reviewed_motion' not in review and 'motion_intervals' not in review, \
        'motion is stated by review_scope, not by actually_reviewed_motion or motion_intervals'
    def claims(value, at):
        if type(value) is dict:
            for key, item in value.items():
                if any(word in key.lower() for word in ('accept', 'release', 'publication')) and item is not False:
                    yield at+key
                yield from claims(item, at+key+'.')
        elif type(value) is list:
            for n, item in enumerate(value):
                yield from claims(item, at+str(n)+'.')
    found = list(claims(review, ''))
    assert not found, 'the permit claims an acceptance, release or publication: '+', '.join(found)
    for key in ('continuous_motion_acceptance', 'listening_acceptance', 'fine_sync_acceptance',
                'final_4k_acceptance', 'release_acceptance', 'publication_acceptance'):
        assert review.get(key) is False, key+' must be stated false'
    scope = review['review_scope']
    assert type(scope) is dict and set(scope) == {'basis', 'purpose', 'preview_movie_sha256', 'sampled_intervals',
                                                   'playback_seconds', 'open_reviews'}, 'review_scope fields'
    assert (scope['basis'], scope['purpose']) == ('sampled_preview', 'technical_4k_render'), 'not a sampled-preview technical 4K scope'
    assert scope['preview_movie_sha256'] == movie_sha256, 'review_scope names another preview movie'
    reviews = scope['open_reviews']
    assert type(reviews) is list and len(set(reviews)) == len(reviews) and all(type(x) is str for x in reviews)
    assert {'final_picture', 'motion', 'audio'} <= set(reviews), 'final-picture, motion and audio review must stay open'
    seconds = scope['playback_seconds']
    assert type(seconds) is list and all(type(x) in (int, float) and 0 <= x < frames/60 for x in seconds), 'playback_seconds'
    intervals = scope['sampled_intervals']
    assert type(intervals) is list and intervals, 'no sampled intervals'
    end = -1
    for pair in intervals:
        assert type(pair) is list and len(pair) == 2 and all(type(x) is int for x in pair), 'sampled interval %r' % (pair,)
        a, b = pair
        assert end < a < b < frames, 'sampled intervals must be sorted, disjoint and inside the chapter: %r' % (pair,)
        missing = [n for n in range(a, b+1) if n not in sampled]
        assert not missing, 'sampled interval [%d, %d] has unsampled frames, first %d' % (a, b, missing[0])
        end = b
    return [tuple(pair) for pair in intervals]


def independent(review):
    """344's, on CSS: the permit names an independent review by someone other than its reviewer. Its files
    are in CSD's Q at their hashes, read through the exact alias (the permit transfer carries them)."""
    alias_is_exact()
    assert review['independent_reviewer'].strip() and review['independent_reviewer'].strip() != review['reviewer'].strip()
    items = review['independent_review']
    assert items
    for item in items:
        path = Path(item['path'])
        assert path.is_absolute() and path.parent == DQ and path.resolve(strict=True) == FROZEN/str(path).lstrip('/') and sha(path) == item['sha256'], item['path']
    return [Path(item['path']) for item in items]


def review_basis(chapter,review):
 """352: one chapter's sampled-preview basis. It is read only from the pinned preview movie and from the
 files the permit binds in evidence and independent_review, each at its stated hash. It only reads, so it
 gives the same result on csd and, through the alias, on css.
 A frame record is a bound JSON object whose movie hash is this chapter's MOVIES hash. The hash sits
 under 'movie_sha256', 'source_sha256' or 'sha256', whichever of these keys comes first in that order.
 The record lists frames as 'samples' [{'frame': n}], 'frames' [n or {'frame': n}] or 'pictures'
 [{'frame': n}], read in that order.
 A frame record is viewed only when root says so in the permit. A bound item's 'views' lists the sha256
 of each frame record whose pictures that item's file records viewing. The file must cite each one, either
 by containing its sha256 or, for a 'samples' record, by containing the sha256 of every item of its
 'sheets' ({'path', 'sha256'}), each sheet being at that hash. Merely mentioning a hash views nothing.
 sampled is the set of frames of the viewed records, and runs are its runs of two or more consecutive
 frames. A bound frame record that no item views adds nothing and is listed as unviewed. An
 independent_review file confirms the chapter when it views a record and contains the movie hash."""
 movie=D/'jobs'/PREVIEWS[chapter]['job']/'preview'/(chapter+'.mp4')
 assert sha(movie)==MOVIES[chapter],'not the preview movie that was reviewed: '+str(movie)
 bound=review['evidence']+review['independent_review']
 items={x['path']:x['sha256'] for x in bound}
 data={}
 for path,digest in items.items():
  data[path]=Path(path).read_bytes();assert hashlib.sha256(data[path]).hexdigest()==digest,'not at the hash the permit binds: '+path
 found={}
 for path,raw in data.items():
  try:record=json.loads(raw)
  except ValueError:continue
  if type(record) is not dict:continue
  key=next((k for k in ('movie_sha256','source_sha256','sha256') if k in record),None)
  if key is None or record[key]!=MOVIES[chapter]:continue
  for name in ('samples','frames','pictures'):
   if type(record.get(name)) is list:
    frames=[x['frame'] if type(x) is dict else x for x in record[name]]
    assert name=='frames' or all(type(x) is dict for x in record[name]),'frame record items: '+path
    assert frames and all(type(n) is int for n in frames),'frame record without integer frames: '+path
    found[path]=(record,name,frames);break
 named={items[p]:p for p in found};by={p:set() for p in found};sheets=[]
 for x in bound:
  views=x.get('views',[]);assert type(views) is list and all(type(d) is str for d in views),'views must list sha256 strings: '+x['path']
  for digest in views:
   assert digest in named,'views names no bound frame record of '+chapter+': '+digest
   record,name,_=found[named[digest]];raw=data[x['path']]
   if digest.encode() not in raw:
    shown=[(s['path'],s['sha256']) for s in record.get('sheets') or []] if name=='samples' else []
    assert shown and all(d.encode() in raw for _,d in shown),x['path']+' does not cite the record it views: '+digest
    for p,d in shown:assert sha(p)==d,'sheet not at its hash: '+p
    sheets+=[p for p,_ in shown if p not in sheets]
   by[named[digest]].add(x['path'])
 records,unviewed,sampled=[],[],set()
 for path,(record,name,frames) in found.items():
  if not by[path]:unviewed.append(path);continue
  sampled.update(frames)
  records.append(dict(path=path,sha256=items[path],frames=len(set(frames)),viewed_by=sorted(by[path])))
 confirmed=sorted(x['path'] for x in review['independent_review'] if x.get('views') and MOVIES[chapter].encode() in data[x['path']])
 runs=[]
 for n in sorted(sampled):
  if runs and n==runs[-1][1]+1:runs[-1][1]=n
  else:runs.append([n,n])
 return dict(chapter=chapter,preview_job=PREVIEWS[chapter]['job'],movie=dict(path=str(movie),sha256=MOVIES[chapter]),
  frame_records=records,unviewed=sorted(unviewed),independent=confirmed,sampled=sorted(sampled),runs=[r for r in runs if r[0]<r[1]],sheets=sheets)


def preview_binding(chapter, review):
    """303's preview_binding: 287's, with the receipt-inputs pin; DQ is CSD's Q read through the alias."""
    preview = PREVIEWS[chapter]
    assert review['preview_job'] == preview['job']
    job = D/'jobs'/preview['job']
    receipt = success(job/'resource-receipt.json')
    assert receipt['inputs_sha256'] == preview['inputs_sha256'], 'not the preview run that was read: '+preview['job']
    worker = DQ/preview['worker']
    assert str(worker) in receipt['command'] and sha(worker) == preview['worker_sha256']
    manifest = DQ/preview['manifest']
    assert sha(manifest) == receipt['inputs_sha256']
    rows = {x['path']: x['sha256'] for x in read(manifest)['files']}
    build = preview['build']
    success(build/'resource-receipt.json')
    assert sha(preview['binary']) == preview['binary_sha256'] == read(build/'qualification.json')['binary_sha256']
    frozen = [preview['binary'], worker, build/'source-copy.json', build/'qualification.json']
    for p in frozen:
        assert rows.get(str(p)) == sha(p), 'not in the preview manifest as is: '+str(p)
    shown = [job/preview['tables']/(chapter+'.'+kind+'.tsv') for kind in KINDS]
    for t in shown:
        assert t.read_bytes().replace(str(preview['narration']).encode(), str(FMD_NARRATION).encode()) == (TIMELINES/t.name).read_bytes(), 'preview table differs from timeline269: '+str(t)
    assert sha(build/'source-copy.json') == preview['source_copy_sha256'] and sha(BUILD268/'source-copy.json') == SOURCE_COPY_268_SHA
    before = read(build/'source-copy.json')['files']
    after = read(BUILD268/'source-copy.json')['files']
    scene = SCENE_DIR+SCENES[chapter]
    assert before[scene] == after[scene], scene
    delta = []
    if build == BUILD268:
        assert preview['binary'] == BIN and preview['binary_sha256'] == BINARY_SHA
    else:
        named = [x for x in review['source_binding_evidence'] if x.get('name') == DELTA_NAME]
        assert [(x['path'], x['sha256']) for x in named] == [(str(DELTA_EVIDENCE), DELTA_EVIDENCE_SHA)], 'one named '+DELTA_NAME+' item for binding282 required'
        delta = [DELTA_EVIDENCE]
        differ = {k for k in before.keys() | after.keys() if before.get(k) != after.get(k)}
        assert differ == DELTA_238_268, sorted(differ ^ DELTA_238_268)
    # 352: the sampled-preview scope, as csd-reviewed-final352.py's preview_binding states it.
    basis = review_basis(chapter, review)
    sampling = job/chapter/'sampling.json'
    evidence = {x['path'] for x in review['evidence']}
    assert any(r['path'] == str(sampling) and evidence & set(r['viewed_by']) for r in basis['frame_records']), 'the permit must bind '+str(sampling)+' and root evidence that views it'
    assert basis['independent'], 'no independent_review file views a frame record of this preview and names its movie: '+chapter
    assert basis['runs'], 'no two consecutive viewed frames of '+chapter+': a sampled interval needs a framewise picture record'
    sampled_review_scope(review, review['frames'], MOVIES[chapter], set(basis['sampled']))
    assert review['review_scope']['playback_seconds'] == [], '352 rests on no playback record: playback_seconds must be []'
    return frozen+shown+[job/'resource-receipt.json', manifest, build/'resource-receipt.json', BUILD268/'source-copy.json', BUILD268/'qualification.json']+delta+[Path(basis['movie']['path'])]+[Path(p) for p in basis['sheets']]


def check(chapter, permit):
    assert chapter in CHAPTERS
    permit = absolute_permit(permit)
    others = sorted(p.name for p in (CSS/'jobs').glob('css-reviewed-final-'+chapter.replace('_', '-')+'*') if p.name != job_name(chapter))
    assert chapter == '05_gallery'
    failed = 'css-reviewed-final-05-gallery352'
    assert others == [failed]
    failure = read(CSS/'jobs'/failed/'resource-receipt.json')
    assert failure.get('finished') and failure.get('exit') == -15 and failure.get('stopped_for_resource_limit') is True
    assert failure['inputs_sha256'] == failure['final_inputs_sha256'] == '8c45915acbe9f50e127d36cb2d64395cd2cfb407e9c9323b966238894fb5b9ab'
    alias_is_exact()
    assert sha(BIN) == BINARY_SHA
    for p in (ENCODER_STOP, TRANSFER75, TRANSFER, DELIVERY, D/'jobs/csd-production-timeline21', SCORE):
        success(p/'resource-receipt.json')
    for p in (BUILD268, REBIND269, TIMELINES.parent):
        success(p/'resource-receipt.json')
    result = read(DELIVERY/'delivery-result.json')
    assert result['binary_sha256'] == BINARY_SHA and result['delivery']['verified_cues'] == 105 and result['source'] == str(FMD_NARRATION)
    assert sha(TIMELINE269_INPUTS) == read(TIMELINES.parent/'resource-receipt.json')['inputs_sha256']
    carried = carried_rows()
    qualification = read(BUILD268/'qualification.json')
    assert qualification['binary_sha256'] == BINARY_SHA
    for key in qualification['chapters']:
        for suffix in ('timeline.tsv', 'cues.tsv', 'cuelog.tsv'):
            assert (TIMELINES/(key+'.'+suffix)).read_bytes() == (BASELINE/(key+'.'+suffix)).read_bytes().replace(str(ORIGINAL_NARRATION).encode(), str(FMD_NARRATION).encode())
    assert chapter in qualification['chapters']
    score = read(SCORE/'score21-summary.json')
    assert score['all_qa_passed'] and score['chapters'] == qualification['chapters']
    for relative, digest in score['files'].items():
        assert sha(SCORE/'score'/relative) == digest
    review = read(permit)
    assert set(review) == PERMIT_KEYS, 'permit fields are not the 344 set; differing: %s' % sorted(set(review) ^ PERMIT_KEYS)
    assert review['binary_sha256'] == BINARY_SHA and review['chapter'] == chapter
    assert review.get('host') == HOST, 'permit must name host css (one chapter, one host)'
    assert review['actually_reviewed_pictures'] is True
    assert review['unresolved_defects'] == [] and review['reviewer'].strip() and review['observations'].strip()
    assert review['timeline_sha256'] == sha(TIMELINES/(chapter+'.timeline.tsv'))
    assert review['evidence']
    assert review['source_binding_evidence'], 'Bind preview source to final binary; preview243 predates combined268'
    for item in review['source_binding_evidence']:
        assert sha(item['path']) == item['sha256']
    for item in review['evidence']:
        assert sha(item['path']) == item['sha256']
    independent(review)
    binding = preview_binding(chapter, review)
    frames = review['frames']
    assert type(frames) is int and frames > 0
    header = dict(line.split('\t', 1) for line in (TIMELINES/(chapter+'.timeline.tsv')).read_text().splitlines() if line.startswith(('chapter\t', 'fps\t', 'frames\t')))
    assert header == {'chapter': chapter, 'fps': '60', 'frames': str(frames)}
    assert read(STEMS/'qa'/(chapter+'.json'))['passed'] is True
    return review, sha(permit), binding, carried


def verify_tables(chapter, out, frames):
    # 287's (and 78's), with the CSS narration copy in place of rebind269's folder.
    expected = (TIMELINES/(chapter+'.timeline.tsv')).read_bytes().replace(str(FMD_NARRATION).encode(), str(NARRATION).encode())
    assert (out/(chapter+'.timeline.tsv')).read_bytes() == expected
    assert (out/(chapter+'.cues.tsv')).read_bytes() == (TIMELINES/(chapter+'.cues.tsv')).read_bytes()
    base = (TIMELINES/(chapter+'.cuelog.tsv')).read_text().replace(str(FMD_NARRATION), str(NARRATION)).splitlines()
    actual = (out/(chapter+'.cuelog.tsv')).read_text().splitlines()
    assert actual[:len(base)] == base
    stem = STEMS/(chapter+'.music.f32.wav')
    parts = stem.with_suffix('.parts.tsv')
    files = [(stem, 0, frames*800)]
    if parts.exists():
        files = [(stem.with_name(r[4]), int(r[0]), int(r[1])) for r in [s.split('\t') for s in parts.read_text().splitlines()[1:]]]
    rows = [r.split('\t') for r in actual[len(base):]]
    assert len(rows) == len(files)
    end = 0
    for row, (path, start, samples) in zip(rows, files):
        offset = float(row[5])
        bits = struct.unpack('>Q', struct.pack('>d', offset))[0]
        assert row[:5] == [str(path), sha(path), str(path.stat().st_size), str(frames), '60']
        assert row[6:] == [f'{bits:#018x}', '0.0', '0x0000000000000000', '48000', '2', str(samples), str(start)]
        assert frames*800+round(offset*48000) == start == end
        end += samples
    assert end == frames*800


def required(permit, review, binding, carried):
    """287's run() binding, with the CSS delivery inputs and the carried CSD rows in place of
    timeline269's full CSD manifest."""
    receipt = success(DELIVERY/'resource-receipt.json')
    assert sha(DELIVERY_INPUTS) == receipt['inputs_sha256']
    chapter = review['chapter']
    return ([Path(x['path']) for x in review['source_binding_evidence']] + binding + independent(review)
        + [BIN, Path(__file__).resolve(), Path(permit)] + tables(TIMELINES, [chapter])
        + list(CSD_MANIFESTS) + carried
        + [DELIVERY_INPUTS, DELIVERY/'delivery-result.json'] + [Path(x['path']) for x in read(DELIVERY_INPUTS)['files']]
        + [p for p in NARRATION.rglob('*') if p.is_file()]
        + [SCORE/'score'/p for p in read(SCORE/'score21-summary.json')['files']])


def launch(args):
    raise RuntimeError('Legacy launch disabled; SSD guard required')
    helper = Path(__file__).resolve(strict=True)
    permit = absolute_permit(args.permit)
    assert sha(permit) == args.permit_sha256, 'permit is not at the hash root passed: '+str(permit)
    assert permit.parent == PERMITS and permit_name_ok(args.chapter, permit.name)
    assert args.permit_transfer.startswith(permit_job_prefix(args.chapter))
    carried_permit = CSS/'jobs'/args.permit_transfer
    success(carried_permit/'resource-receipt.json')
    moved = read(carried_permit/'transfer-result.json')
    placed = [r for r in moved['files'] if r['path'] == str(permit)]
    assert moved['kind'] == 'permit' and len(placed) == 1 and placed[0]['sha256'] == sha(permit) and placed[0]['written'], \
        'the permit must be the file that transfer wrote'
    files = {helper, permit, GUARD, carried_permit/'resource-receipt.json', carried_permit/'transfer-result.json', Path(moved['manifest'])}
    if args.after:
        before = CSS/'jobs'/args.after/'resource-receipt.json'
        deadline = time.monotonic()+43200
        while not (before.exists() and read(before).get('finished')):
            assert time.monotonic() < deadline, 'predecessor wait timed out'
            time.sleep(15)
        success(before)
        files.add(before)
    review, _, binding, carried = check(args.chapter, permit)
    assert review['frames'] > 0
    job = job_name(args.chapter)
    assert not (CSS/'jobs'/job).exists(), 'Existing job must not be duplicated'
    # As 303's launcher: render pins itself to the first --threads CPUs; refuse before any file is written.
    assert len(os.sched_getaffinity(0)) >= args.threads, 'fewer CPUs in the inherited affinity than --threads'
    for p in (ENCODER_STOP, TRANSFER75, TRANSFER, DELIVERY, D/'jobs/csd-production-timeline21', SCORE):
        files.add(p/'resource-receipt.json')
    for folder in JOB_DIRS:                       # 287's launcher binds every file of these job folders
        files.update(files_under(folder))
    chapters = read(BUILD268/'qualification.json')['chapters']
    files.update(tables(BASELINE, chapters) + tables(TIMELINES, chapters))
    files.update((BUILD268/'qualification.json', SCORE/'score21-summary.json', TRANSFER/'transfer-result.json', MANIFEST75))
    files.update(Path(x['path']) for x in review['evidence'])
    files.update(required(permit, review, binding, carried))
    inputs = write_inputs(CQ/('final-'+args.chapter+'-inputs'+VERSION+'.json'), files)
    subprocess.run([PYTHON, '-I', '-B', str(GUARD), '--phase', 'native', '--job', job, '--inputs', str(inputs), '--',
        PYTHON, '-I', '-B', str(helper), 'render', '--chapter', args.chapter, '--job', job, '--inputs', str(inputs),
        '--permit', str(permit), '--threads', str(args.threads)], check=True)
    return 0


def render(args):
    chapter = args.chapter
    review, permit_sha, binding, carried = check(chapter, args.permit)
    assert args.job == job_name(chapter)
    job = Path('/data/ssd/work/jmev-gallery-recovery440/jobs')/args.job
    bound = frozen_inputs(args.job, args.inputs)
    for p in required(args.permit, review, binding, carried):
        assert bound.get(str(p)) == sha(p), str(p)
    available = sorted(os.sched_getaffinity(0))
    assert len(available) >= args.threads
    os.sched_setaffinity(0, set(available[:args.threads]))
    out = job/'render'
    assert not out.exists()
    os.environ.update(PATH='/usr/bin:/bin', TMPDIR='/data/ssd/work/jmev-gallery-recovery440/work/tmp', XDG_CACHE_HOME='/data/ssd/work/jmev-gallery-recovery440/work/cache')
    command = [str(BIN), 'render', chapter, '--res', '3840x2160', '--fps', '60', '--threads', str(args.threads), '--narration', str(NARRATION),
               '--score', str(STEMS), '--out', str(out), '--lint', '--crf', '16', '--preset', 'veryfast']
    with (job/'native.stdout').open('xb') as o, (job/'native.stderr').open('xb') as e:
        subprocess.run(command, cwd=D/'work/production-repo07', stdout=o, stderr=e, check=True, timeout=43200)
    frames = review['frames']
    verify_tables(chapter, out, frames)
    movie = out/(chapter+'.mp4')
    assert set(out.glob('*.mp4')) == {movie}
    probe = json.loads(subprocess.check_output(['ffprobe', '-v', 'error', '-show_streams', '-of', 'json', str(movie)], timeout=120))
    vs = [x for x in probe['streams'] if x['codec_type'] == 'video']
    aus = [x for x in probe['streams'] if x['codec_type'] == 'audio']
    assert len(vs) == len(aus) == 1
    v, a = vs[0], aus[0]
    assert (v['width'], v['height'], v['avg_frame_rate'], int(v['nb_frames'])) == (3840, 2160, '60/1', frames)
    assert (v['codec_name'], v['pix_fmt'], a['codec_name'], a['sample_rate'], a['channels']) == ('h264', 'yuv420p', 'aac', '48000', 2)
    with movie.open('rb') as f:
        head = f.read(16*1024*1024)
    pos = head.find(b'x264 - core')
    assert pos >= 0
    settings = head[pos:pos+5000].split(b'\0', 1)[0].decode('utf8', errors='replace')
    assert ' rc=crf' in settings and ' crf=16.0' in settings
    captures = sorted((out/'soundtrack'/chapter).glob('soundtrack_*.wav'), key=lambda p: int(p.stem.split('_')[-1]))
    assert captures
    with wave.open(str(captures[-1])) as w:
        assert (w.getframerate(), w.getnchannels(), w.getsampwidth(), w.getnframes()) == (48000, 2, 2, frames*800)
    assert check(chapter, args.permit) == (review, permit_sha, binding, carried)
    for path, digest in bound.items():
        assert sha(path) == digest
    preview = PREVIEWS[chapter]
    basis = review_basis(chapter, review)
    result = dict(host=HOST, chapter=chapter, frames=frames, binary_sha256=BINARY_SHA, permit_sha256=permit_sha,
        worker352_sha256=sha(Path(__file__).resolve()), derived_from_worker309_sha256=WORKER309_SHA, derived_from_helper290_sha256=HELPER290_SHA,
        derived_from_worker287_sha256=WORKER287_SHA, preview_contract_worker303_sha256=WORKER303_SHA,
        preview_job=preview['job'], preview_inputs_sha256=preview['inputs_sha256'], preview_binary_sha256=preview['binary_sha256'],
        preview_binding=[dict(path=str(p), sha256=sha(p)) for p in binding],
        independent_review=review['independent_review'], review_scope=review['review_scope'],
        review_basis={k: basis[k] for k in ('movie', 'frame_records', 'independent', 'runs')},
        carried_csd_rows=dict(count=len(carried), manifests={str(m): sha(m) for m in CSD_MANIFESTS}),
        narration=str(NARRATION), encoder=settings,
        files=[dict(path=str(p), sha256=sha(p)) for p in sorted(out.rglob('*')) if p.is_file()],
        scope='Technical scored4K60 chapter only, authorized from a sampled-preview disposition (review_scope); final-picture, motion and audio review remain open, as do full decode, master replay, captions and publication. Not publication authorization.')
    with (job/'render-summary.json').open('x') as f:
        json.dump(result, f, indent=2)
    return 0


def preflight(args):
    """Read-only: every input the CSS gates read, with its state. Exit 0 only when nothing is missing."""
    def state(p, digest=None):
        p = Path(p)
        if not p.exists():
            return 'missing'
        return 'present' if digest is None or sha(p) == digest else 'DIFFERENT'
    rows = [('alias', str(D), 'present' if D.is_symlink() and D.resolve() == FROZEN/str(D).lstrip('/') else 'DIFFERENT')]
    rows += [(k, str(p), state(p)) for k, p in (('guard', GUARD), ('runtime75', TRANSFER75/'resource-receipt.json'),
        ('encoder stop', ENCODER_STOP/'resource-receipt.json'), ('cwd', D/'work/production-repo07'), ('assembly19', ORIGINAL_NARRATION/'report.json'))]
    rows.append(('binary 268', str(BIN), state(BIN, BINARY_SHA)))
    rows += [('workspace', str(p), state(p)) for p in (TOPIC/'script.tsv', TOPIC/'narration_units.json', WORKSPACE/'tools/narration_delivery.py',
             TOPIC/'docs/narration-direction-map-frederick-review269.json')]
    for folder in JOB_DIRS:
        rows.append(('csd job', str(folder), state(folder/'resource-receipt.json')))
    rows.append(('timeline269', str(TIMELINES.parent/'comparison.json'), state(TIMELINES.parent/'comparison.json')))
    rows += [('csd q', str(x), state(x, h)) for x, h in ((TIMELINE269_INPUTS, None), (DELTA_EVIDENCE, DELTA_EVIDENCE_SHA))]
    for p in (PREVIEW243, PREVIEW274):
        rows += [('csd q', str(x), state(x, h)) for x, h in ((DQ/p['manifest'], p['inputs_sha256']), (DQ/p['worker'], p['worker_sha256']))]
    rows.append(('binary 238', str(PREVIEW243['binary']), state(PREVIEW243['binary'], PREVIEW243['binary_sha256'])))
    chapters = ('01_the_bump', '02_turning_numbers', '03_two_planes', '04_reading_color', '05_gallery', '06_one_derivative', '07_one_point', '08_loop_sums',
                '09_boundary', '10_rigidity', '11_counting', '12_residues', '13_real_integral', '14_continuation', '15_closing')
    for name, folder in (('timeline269 tables', TIMELINES), ('timeline21 tables', BASELINE)):
        missing = [t.name for t in tables(folder, chapters) if not t.exists()]
        rows.append((name, str(folder), 'present' if not missing else 'missing %d of 45' % len(missing)))
    score = read(SCORE/'score21-summary.json') if (SCORE/'score21-summary.json').exists() else {'files': {}}
    bad = [k for k, v in score['files'].items() if state(SCORE/'score'/k, v) != 'present']
    rows.append(('score21', str(SCORE), 'present (%d files)' % len(score['files']) if score['files'] and not bad else 'missing or different: %s' % bad[:5]))
    rows += [('css job', str(j), state(j/'resource-receipt.json')) for j in (TRANSFER, DELIVERY)]
    rows.append(('permits', str(PERMITS), 'present' if PERMITS.is_dir() else 'missing'))
    if args.manifest:                                    # review300 L2: the landed copy is the manifest named
        rows.append(('manifest copy', str(args.manifest), state(args.manifest, args.manifest_sha)))
    for r in rows:
        print('%-18s %-16s %s' % (r[0], r[2], r[1]))
    return 0 if all(r[2].startswith('present') for r in rows) else 1


def main():
    p = argparse.ArgumentParser(description=__doc__.split('\n')[0])
    sub = p.add_subparsers(dest='mode', required=True)
    s = sub.add_parser('manifest'); s.add_argument('--chapter', choices=CHAPTERS); s.add_argument('--permit', type=Path)
    s.add_argument('--out', type=Path, required=True)
    s = sub.add_parser('land-manifest'); s.add_argument('--out', type=Path, required=True); s.add_argument('--manifest-sha', required=True)
    s = sub.add_parser('send'); s.add_argument('--manifest', type=Path, required=True); s.add_argument('--manifest-sha', required=True)
    s = sub.add_parser('freeze-transfer'); s.add_argument('--job', required=True); s.add_argument('--manifest', type=Path, required=True)
    s.add_argument('--manifest-sha', required=True)
    s = sub.add_parser('receive'); s.add_argument('--job', required=True); s.add_argument('--manifest', type=Path, required=True)
    s.add_argument('--manifest-sha', required=True)
    s = sub.add_parser('preflight'); s.add_argument('--manifest', type=Path); s.add_argument('--manifest-sha')
    sub.add_parser('launch-delivery')
    sub.add_parser('delivery')
    s = sub.add_parser('launch'); s.add_argument('--chapter', choices=CHAPTERS, required=True); s.add_argument('--permit', type=Path, required=True)
    s.add_argument('--permit-sha256', required=True)
    s.add_argument('--permit-transfer', required=True); s.add_argument('--after'); s.add_argument('--threads', type=int, default=16)
    s = sub.add_parser('render'); s.add_argument('--chapter', choices=CHAPTERS, required=True); s.add_argument('--job', required=True)
    s.add_argument('--inputs', type=Path, required=True); s.add_argument('--permit', type=Path, required=True); s.add_argument('--threads', type=int, default=16)
    args = p.parse_args()
    if args.mode == 'manifest':
        assert (args.chapter is None) == (args.permit is None), 'a permit manifest needs both --chapter and --permit'
    if args.mode == 'preflight':
        assert (args.manifest is None) == (args.manifest_sha is None), 'preflight needs both --manifest and --manifest-sha, or neither'
    if args.mode in ('launch', 'render'):
        assert 1 <= args.threads <= 16
    return {'manifest': manifest, 'land-manifest': land_manifest, 'send': send, 'freeze-transfer': freeze_transfer, 'receive': receive,
            'preflight': preflight, 'launch-delivery': launch_delivery, 'delivery': delivery, 'launch': launch, 'render': render}[args.mode](args)


if __name__ == '__main__':
    raise SystemExit(main())
