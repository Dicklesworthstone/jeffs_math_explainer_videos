"""PREPARED ONLY, NOT REVIEWED (CC0 scratch draft final4k-impl01, successor 02): strict same-host gate for one repaired
chapter row (01/08/09 native-combined01, 11 native-counting02) of the final 4K successor of master993.

Successor of native-linux-master-gate992.py (0d8d83e1) for CC3's route (final4k-integration-proposal-cc3
a60b6815, section 7) with CC0 review01 M1. Root and CC2 review it before any use.
Interface gate(base,row,frames_of) -> (record, used_paths), as 992. Never executes a render/master.
Changes from 992, and nothing else:
  - a CHAPTERS table (kind, chain jobs, binary, tests, scene sha, frames, captures) replaces 09's constants;
  - prerequisites are the chain's build, delivery, timeline, preview and preview pictures and the
    chapter's final; the chain's timeline and preview comparison records must give the chapter's
    frames and normalized table, and agree (CC0 handoff02: 11 must be 29a3afd3 at 11932 frames);
  - the preview admission is the chapter's own root-written record (ADMISSIONS); its evidence goes through
    the bound final runner's review_checks() (whole root review, or 09's REVIEW_ROUTES: complete independent
    and author viewing under root's explicit decision). M1 (successor 02, root 2026-11-11): the row's review
    basis is derived only from the actual final extraction and its complete viewing, with 993's scope fields.
    No basis is carried from the preview admission, master993 or another movie, and no movie sha is replaced;
  - 09 still runs every original313 gate; their basis is kept apart as original313_basis, and
    capture_check_before_master stays as 992 wrote it;
  - the final extraction must hold every preview extraction frame (992: >= 814); 08's two lint anchors
    must be preview extraction frames, and 995's REQUIRED_WINDOWS 08 run must lie in 08's scope;
  - the staged scored capture must equal master993's staged capture (STAGED993);
  - unresolved pins are None and refuse before anything is read: root sets them. Successor 02 fills every
    pin with an actual value (runner 7cafacfa, binaries, job and admission names, worker name, 01/08/09
    preview sizes); STAGE_VERSION, WORKER_SHA and MIN_PREVIEW[11] stay with root.
Keeps original313 history (09), new Linux evidence and final pictures separate. Outer master must retain
all11 other gates and all15 PCM null tests.
Successor 03 (CC0, 2026-11-11): 11's final runs under run11 (FINAL_RUNNER11_NAME/SHA; 7cafacfa stays bound to
01/08/09), whose REVIEW_ROUTES give 11's preview admission the author-only route (CC3's complete authored
review plus root's targeted acceptance; no independent_review). Nothing else changes; the final
extraction's review checks and derived_scope() are successor 02's.
Successor 04 (CC0, 2026-11-11): option b, as in stage03 and materializer03.
CC0 successor (option b; owner via root, 2026-11-11): the actual final extraction's review may take a worker
route, as the previews did. final_review_checks() (identical in stage03, materializer03 and gate04) accepts
root's complete viewing plus an independent subset (the original rule), or root's targeted acceptance
(review_route in FINAL_ROUTES) that binds complete worker reviews of the actual final movie and extraction
by sha256, with honest authorship. The proofs name root_review, independent_review and author_review
(None where the route has none). Nothing else changes.
"""
from pathlib import Path
import re
import json
import subprocess
import wave
import hashlib
import importlib.util

BOUNDARY = '09_boundary'
MANIFEST993_SHA = '8da77ed79e9ccc8a06e703e570c85c44de5cd884a047eaed61355167e397e100'
PCM313_SHA = 'f7f8aa6042a9f05d1b25db4c789f7f6b44cb393445015c057f1e87d9b0c8df51'   # 992's PCM_SHA
SCENE = 'work/%s/workspace/explainer_corpus/complex_analysis_visual/src/chapters/'
COMBINED = dict(kind='native-combined01', build='csd-native-combined-build01',
    binary='complex_analysis_visual-combined01', tests=247, workspace='native-combined01',
    delivery='csd-native-combined-delivery01', delivery_record='delivery-combined01.json',
    timeline='csd-native-combined-timeline01', preview='csd-native-combined-preview01',
    pictures='csd-native-combined-pictures01', comparison='comparison-combined01.json')
COUNTING = dict(kind='native-counting02', build='csd-native-counting-build02',
    binary='complex_analysis_visual-counting02', tests=250, workspace='native-counting02',
    delivery='csd-native-counting-delivery02', delivery_record='delivery-counting02.json',
    timeline='csd-native-counting-timeline02', preview='csd-native-counting-preview02',
    pictures='csd-native-counting-pictures02', comparison='comparison-counting02.json')
# normalized: the chain's EXPECTED normalized timeline. preview_pcm: the retained narration-only preview
# capture (274/243/853). staged993: master993's staged scored capture (11: root's CSD sha256sum).
CHAPTERS = {
    '01_the_bump': dict(COMBINED, frames=6311, scene='the_bump.rs',
        source_sha='90b51c2b30877911bbb5f5e14ee810268e14efb24e7c4e2a55f7f7e128c06ee7',
        normalized='150147fe5bc7cdb0a111061c4530a16b104d196e004ae29bcf210d1c9c32ac81',
        preview_pcm='79fc6e7dea2c95c385b976409225a472a2d7d69cdb2314a0d6af9053614c4e60',
        staged993='4d77c2d66720d67160cea0feec6d86a5e5ab54720641d4586050104a2d592e07'),
    '08_loop_sums': dict(COMBINED, frames=14959, scene='loop_sums.rs',
        source_sha='a8d62f560321ea6ebea3d3859acd9a3ce3e5dde4375ec0ab42c1a1ba114d61ef',
        normalized='43c8bab2f7ca61c8692e0f1686798d54e187dcfeeab0d03311c2d4c9a5f54812',
        preview_pcm='2c277faaaa88661eab7b533bcc495759051b94b42018aaa39a1fe098e568ba98',
        staged993='d713402c0c7fba3c17705fd624d38598b9aabebe2ef6220db9dc8cfeecbb328b'),
    '09_boundary': dict(COMBINED, frames=16504, scene='boundary.rs',
        source_sha='4b689791748132bd14c3929d191bb78ae7752bf9c3332beb22f774931873e932',
        normalized='5a91b8f9e2e5b5d6223a49285c06039b1b43bd16c11cecdd163a8e2e545baa9e',
        preview_pcm=PCM313_SHA,
        staged993='29899f9e42721331e2abe9088c399bca3eb63ea3b50de45dc731cbb8c4404c6c'),
    '11_counting': dict(COUNTING, frames=11932, scene='counting.rs',
        source_sha='21c5043a3339cfadb10024a2e8227a62a0b06674139eace1a0d9029a4e1aa0db',
        normalized='29a3afd345fa3a376e5c163d5bf5e01267a866e2305bdd27c6413a5bab46807d',
        preview_pcm='7abda95ef24883bd9b4ed2fde0890409b5739939e3cdfe8f4869b2f18b8dd4dc',
        staged993='82bdc443218a5a761c678244089bc8d983a01466c77a7efed23ea358a4ec490a'),
}
# Root's 08 lint anchors (179.8 s and 205.6 s of the repaired 08 preview): they must be extracted, so
# root's viewing covers them. No verdict is implied.
ANCHORS = {'08_loop_sums': (10788, 12336)}
# 995 REQUIRED_WINDOWS (DISPOSITION377): this run must lie inside a sampled interval of 08's scope
# (995:1102-1104). Checked here so a scope without it refuses before the master, not after.
REQUIRED_SCOPE = {'08_loop_sums': ((10200, 10380),)}
# UNRESOLVED pins: root sets each before use; gate() refuses while any is None.
STAGE_VERSION = '1006'       # UNRESOLVED (root): digits; must equal the materializer's
FINAL_JOBS = {ch: 'csd-native-repair-final01-'+ch.replace('_', '-')
              for ch in ('01_the_bump', '08_loop_sums', '09_boundary', '11_counting')}
WORKER_NAME = 'cc0-native-materialize-repaired03.py'   # Q name of the adopted materializer (materializer03)
WORKER_SHA = 'd8230404870270331438d715e9ae5be4deab66f4e07c202458cb8c553209bcb2'          # UNRESOLVED (root): its sha256 after root sets its STAGE_VERSION
FINAL_RUNNER_NAME = 'csd-native-repair-final01-run.py'
FINAL_RUNNER_SHA = '7cafacfa788b1579ce66b9db9880c25890112b7d1d7737ebcd44d7827daf1a09'   # as root staged it
FINAL_RUNNER11_NAME = 'csd-native-repair-final01-run11.py'   # 11 only (CC0's chapter 11 successor of 7cafacfa)
FINAL_RUNNER11_SHA = '01382a17b527b957fa7da7ce7b31e07c7c34023b01c948ac866761a1ef53bdeb'
BINARY_SHA = {'native-combined01': 'aed99576b7a9cc42353c43963e1828acf3465426e028fc424eaf6712133725ce',
              'native-counting02': 'b1bc47b40711d4cfb295339459def8e8fac054d0118bc8896f33a4fd49369be1'}   # qualification.json
ADMISSIONS = {ch: 'csd-native-repair-final01-'+ch.replace('_', '-')+'-picture-admission.json' for ch in FINAL_JOBS}   # root-written
MIN_PREVIEW = {'01_the_bump': 120, '08_loop_sums': 484, '09_boundary': 1506,   # actual preview extraction sizes
               '11_counting': 848}   # UNRESOLVED (root): counting pictures02's size

def unresolved():
    missing = [] if STAGE_VERSION else ['STAGE_VERSION']
    missing += [n for n, v in (('WORKER_NAME', WORKER_NAME), ('WORKER_SHA', WORKER_SHA),
                               ('FINAL_RUNNER_NAME', FINAL_RUNNER_NAME), ('FINAL_RUNNER_SHA', FINAL_RUNNER_SHA),
                               ('FINAL_RUNNER11_NAME', FINAL_RUNNER11_NAME), ('FINAL_RUNNER11_SHA', FINAL_RUNNER11_SHA)) if not v]
    for name, table in (('FINAL_JOBS', FINAL_JOBS), ('BINARY_SHA', BINARY_SHA),
                        ('ADMISSIONS', ADMISSIONS), ('MIN_PREVIEW', MIN_PREVIEW)):
        missing += ['%s[%s]' % (name, k) for k, v in table.items() if not v]
    return missing


FINAL_ROUTES = ('independent-complete-root-targeted', 'author-complete-root-targeted',
                'independent-and-author-complete-root-targeted')


def final_review_checks(chapter, source_sha, movie_sha, extraction_sha, report, root, independent, author,
                        independent_sha, author_sha):
    """The actual final 4K extraction's review (owner via root, 2026-11-11, option b). Pure: reads no file.
    Every record names the actual final movie, the actual final extraction record and the chapter's scene,
    states no concrete defect and is safe to advance to the master. If root's record has no review_route,
    root viewed every selected frame and sheet and an independent reviewer (not the repair's author) viewed
    some: the original rule. If it names a FINAL_ROUTES route, each worker review that route names viewed
    every selected frame and sheet, the independent one is not the repair's author and the author one is,
    and root's record is a targeted acceptance of this chapter that binds those records and the extraction
    by sha256, counts their complete coverage and states root_whole_chapter_review false. Returns the record
    whose viewing is complete; the row's scope is derived from it."""
    frames = report['selected_frames']
    sheets = [item['path'] for item in report['sheets']]
    route = root.get('review_route')
    assert route is None or route in FINAL_ROUTES, route
    assert (independent is not None) == (route is None or 'independent' in route), route
    assert (author is not None) == (route is not None and 'author' in route), route
    for evidence in (root, independent, author):
        if evidence is not None:
            assert evidence['movie_sha256'] == movie_sha and evidence['extraction_sha256'] == extraction_sha
            assert evidence['source_sha256'] == source_sha and evidence['concrete_defects'] == []
            assert evidence['safe_to_advance_to_master'] is True
    if independent is not None:
        assert independent['authored_repair'] is False and independent['viewed_frame_indices']
        assert set(independent['viewed_frame_indices']) <= set(frames)
    if route is None:
        assert root['viewed_frame_indices'] == frames and root['viewed_sheets'] == sheets
        return root
    assert root['chapter'] == chapter and root['extraction_record']['sha256'] == extraction_sha
    assert independent is None or author is None or independent_sha != author_sha, 'one record named twice'
    coverage = root['coverage']
    assert coverage['root_whole_chapter_review'] is False
    complete = None
    for kind, record, record_sha in (('independent', independent, independent_sha), ('author', author, author_sha)):
        if record is None:
            assert kind+'_review' not in root and not coverage.get(kind+'_selected_frames'), kind
            continue
        assert record['viewed_frame_indices'] == frames and record['viewed_sheets'] == sheets, kind
        assert record['authored_repair'] is (kind == 'author'), kind
        assert coverage[kind+'_selected_frames'] == len(frames) and root[kind+'_review']['sha256'] == record_sha, kind
        complete = complete or record
    return complete


def derived_scope(base, admission, preview, final, viewed):
    """M1 as root restated it (2026-11-11): the row's review basis is derived from the actual final extraction
    and its actual complete viewing, never from the preview admission or master993. The sampled intervals are
    the maximal runs of two or more consecutive frames that the final extraction holds and its complete review
    (root's, or the final route's complete worker review) viewed; that review must have viewed every retained
    preview frame. 993's scope fields and labels are kept
    (basis 'sampled_preview', purpose 'technical_4k_render', the labels verify02 reads); the scope names the
    admitted preview movie whose frames the final extraction retains, states no playback instant and keeps
    final-picture, motion and audio review open. The preview admission must still claim nothing."""
    have = set(final['selected_frames']) & set(viewed['viewed_frame_indices'])
    assert set(preview['selected_frames']) <= have, 'the final review did not view every retained preview frame'
    intervals, first, last = [], None, None
    for frame in sorted(have):
        if last is None or frame != last+1:
            if last is not None and last > first:
                intervals.append([first, last])
            first = frame
        last = frame
    if last is not None and last > first:
        intervals.append([first, last])
    assert intervals and all(0 <= a < b < final['frames'] for a, b in intervals), 'no sampled intervals'
    flags = sorted(k for k in admission if k.endswith('_acceptance'))
    assert all(admission[k] is False for k in flags), 'every acceptance flag must be false'
    found = list(base.claimed(admission))
    assert not found, 'the admission claims an acceptance, release or publication: '+', '.join(found)
    opened = sorted(base.OPEN_REVIEWS)
    scope = dict(basis='sampled_preview', purpose='technical_4k_render', preview_movie_sha256=preview['movie_sha256'],
                 sampled_intervals=intervals, playback_seconds=[], open_reviews=opened)
    assert set(scope) == base.SCOPE_KEYS, 'review_scope fields'
    return dict(review_basis='sampled_preview', review_scope=scope, sampled_intervals=[list(x) for x in intervals],
                sampled_frames=sum(b-a+1 for a, b in intervals), open_reviews=opened, stated_false=flags)


# Historical aliases are immutable prerequisites, never new output destinations.
ALIAS_FILE = Path('/data/projects/jmev-stroke-probe-20261106/qualification01/production-repair17/historical-aliases991.json')
ALIAS_SHA = 'c6408b16d707164dd20e961a4db0ae05ca1c2c73830ca26118b8fe61e8aba958'
def alias_target(p):
    assert p.is_absolute() and p.is_file(), p
    raw = ALIAS_FILE.read_bytes()
    assert hashlib.sha256(raw).hexdigest() == ALIAS_SHA
    aliases = json.loads(raw)
    if str(p) not in aliases:
        assert not p.is_symlink(), p
        return None
    row = aliases[str(p)]
    assert p.is_symlink() and str(p.readlink()) == row['link'], p
    target = Path(row['target'])
    assert p.resolve(strict=True) == target and target.is_file() and not target.is_symlink(), p
    with target.open('rb') as f:
        assert hashlib.file_digest(f, 'sha256').hexdigest() == row['sha256'], p
    return target

def gate(base, row, frames_of):
    missing = unresolved(); assert not missing, 'unresolved pins: '+', '.join(missing)
    CHAPTER = row['chapter']; c = CHAPTERS[CHAPTER]; FRAMES = c['frames']; FINAL = FINAL_JOBS[CHAPTER]
    BINARY = BINARY_SHA[c['kind']]
    assert row['kind'] == c['kind']
    assert frames_of[CHAPTER] == FRAMES
    assert set(row) == {'chapter','kind','job','worker','worker_sha256','inputs','proofs','picture_review'}
    assert re.fullmatch(r'csd-native-stage%s-%s-a[1-9][0-9]*' % (STAGE_VERSION, CHAPTER[:2]),row['job'])
    worker = base.Q/WORKER_NAME
    assert row['worker'] == str(worker) and row['worker_sha256'] == WORKER_SHA == base.sha(worker)
    stage = base.D/'jobs'/row['job']
    inputs = base.absolute(row['inputs']); assert inputs.parent == base.Q
    state = base.success(stage/'resource-receipt.json')
    assert base.sha(inputs) == state['inputs_sha256']
    entries = base.read(inputs)['files']; bound = {x['path']:x['sha256'] for x in entries}
    assert len(entries) == len(bound)
    files = {worker,inputs,stage/'resource-receipt.json',stage/'materialized.json',Path(__file__).resolve()}
    def require(p):
        p = Path(p)
        target = alias_target(p)
        assert bound.get(str(ALIAS_FILE)) == ALIAS_SHA
        files.add(ALIAS_FILE)
        if target is not None:
            assert bound.get(str(target)) == base.sha(target), target
            files.add(target)
        assert bound.get(str(p)) == base.sha(p),p
        files.add(p); return p
    for p in bound: require(p)
    require(worker)
    made = base.read(stage/'materialized.json')
    assert made['chapter'] == CHAPTER and made['frames'] == FRAMES
    assert made['source_job'] == FINAL and made['binary_sha256'] == BINARY
    assert made['staged993_capture_sha256'] == c['staged993']
    proofs_path = require(row['proofs'])
    assert made['proofs'] == dict(path=str(proofs_path),sha256=base.sha(proofs_path))
    proofs = base.read(proofs_path)
    # Bind exact guarded invocation, including the chapter and the original manifest argument.
    command = state['command']
    assert len(command) == 14 and command[:6] == ['/usr/bin/python3','-I','-B',str(worker),'--chapter',CHAPTER]
    assert command[6:12] == ['--job',row['job'],'--inputs',str(inputs),'--proofs',str(proofs_path)]
    assert command[12] == '--manifest'
    original_manifest = require(command[13])
    assert base.sha(original_manifest) == MANIFEST993_SHA == made['original_manifest_sha256']
    assert made['unchanged_other_manifest_rows'] == 14
    original_rows = base.read(original_manifest)['chapters']
    proposed_path = stage/'master-manifest.json'; files.add(proposed_path)
    proposed = base.read(proposed_path)['chapters']
    assert len(original_rows) == len(proposed) == 15
    assert [r['chapter'] for r in original_rows] == [r['chapter'] for r in proposed]
    assert [r for r in original_rows if r['chapter'] != CHAPTER] == [r for r in proposed if r['chapter'] != CHAPTER]
    assert [r for r in proposed if r['chapter'] == CHAPTER] == [row]
    expected_jobs = {base.D/'jobs'/j/'resource-receipt.json' for j in
        (c['build'],c['delivery'],c['timeline'],c['preview'],c['pictures'],FINAL)}
    actual_jobs = set()
    for proof in proofs['prerequisites']:
        receipt = require(proof['receipt']); actual_jobs.add(receipt)
        terminal = base.success(receipt); manifest = require(proof['inputs'])
        assert base.sha(manifest) == terminal['inputs_sha256']
        for item in base.read(manifest)['files']:
            assert base.sha(require(item['path'])) == item['sha256']
    assert expected_jobs <= actual_jobs
    runner_name, runner_sha = ((FINAL_RUNNER11_NAME, FINAL_RUNNER11_SHA) if CHAPTER == '11_counting'
                               else (FINAL_RUNNER_NAME, FINAL_RUNNER_SHA))
    final_runner = require(base.Q/runner_name)
    assert base.sha(final_runner) == runner_sha
    spec = importlib.util.spec_from_file_location('repaired_final_run_gate', final_runner)
    run = importlib.util.module_from_spec(spec); spec.loader.exec_module(run)
    # 7cafacfa has no review_keys(); for 01/08/09 the keys are written out as successor 02 read them.
    keys = (run.review_keys(CHAPTER) if CHAPTER == '11_counting' else
            ('root_review','independent_review','extraction_record')+(('author_review',) if CHAPTER in run.REVIEW_ROUTES else ()))
    author_only = 'independent_review' not in keys
    final_state = base.success(require(base.D/'jobs'/FINAL/'resource-receipt.json'))
    assert final_state['command'] == ['/usr/bin/python3','-I','-B',str(final_runner),'--chapter',CHAPTER]
    qualification = base.read(require(base.D/'jobs'/c['build']/'qualification.json'))
    assert qualification['binary_sha256'] == BINARY
    assert qualification['tests_passed'] == c['tests'] and qualification['clippy_passed'] is True
    assert qualification['explicit_native_plate_fade_probe_passed'] is True
    assert base.sha(require(base.D/'jobs'/c['build']/c['binary'])) == BINARY
    assert base.sha(require(base.D/(SCENE % c['workspace'])/c['scene'])) == c['source_sha']
    assert base.read(require(base.D/'jobs'/c['delivery']/c['delivery_record']))['wavs_byte_identical'] == 105
    # CC0 handoff02: the chain's actual timeline and preview tables are the chapter's exact normalized table.
    timeline = base.read(require(base.D/'jobs'/c['timeline']/c['comparison']))['chapters'][CHAPTER]
    previewed = base.read(require(base.D/'jobs'/c['preview']/c['comparison']))['chapters'][CHAPTER]
    assert (timeline['frames'],timeline['normalized_sha256']) == (FRAMES,c['normalized']) and previewed == timeline

    original313 = replay_check = None
    if CHAPTER == BOUNDARY:
        # Preserve original313 independent source/picture/capture gates, as830 did.
        old = next(x for x in base.read(base.Q/'cc1-master447-manifest-draft534.json')['chapters'] if x['chapter'] == CHAPTER)
        assert old['kind'] == 'boundary313' and old['worker'] == str(base.WORKER344)
        old_worker = base.WORKER344; assert base.sha(old_worker) == old['worker_sha256']
        module = base.load(old_worker,'native844_original_preview')
        binary313,used = base.provenance313(16504); files |= used
        permit = base.absolute(old['permit']); historical_review = base.read(permit)
        original313,used = base.sampled_permit_gate(CHAPTER,historical_review,binary313,
            base.TIMELINE313/'timeline-output'/(CHAPTER+'.timeline.tsv'),old_worker,module)
        files |= used | base.independent344(old,old_worker,historical_review) | {old_worker,permit,base.Q/'cc1-master447-manifest-draft534.json'}
        binding = module.preview_binding(CHAPTER,historical_review)
        files.update(Path(p) for p in binding); files.update(Path(p) for p in module.independent(historical_review))
        checked = module.check(CHAPTER,permit)
        assert (checked[0],checked[1],[str(p) for p in checked[2]]) == (historical_review,base.sha(permit),[str(p) for p in binding])
        for item in historical_review['source_binding_evidence']:
            assert base.sha(item['path']) == item['sha256']; files.add(Path(item['path']))
        replay_check,used = base.capture_check09(16504); files |= used
        assert replay_check['binary313_capture']['sha256'] == PCM313_SHA
    preview_capture = require(base.D/'jobs'/c['preview']/'preview/soundtrack'/CHAPTER/'soundtrack_0.wav')
    assert base.sha(preview_capture) == c['preview_pcm']

    def pictures(extraction_path,root_path,independent_path,movie,approval,minimum,admission=None,author_path=None):
        extraction_path = require(extraction_path); report = base.read(extraction_path)
        receipt = base.D/'jobs'/extraction_path.relative_to(base.D/'jobs').parts[0]/'resource-receipt.json'
        assert receipt in actual_jobs
        base.success(require(receipt))
        assert report['movie'] == str(movie) and report['movie_sha256'] == base.sha(require(movie))
        assert report['frames'] == FRAMES and report['full_decode'] is True
        frames = report['selected_frames']; assert frames == sorted(set(frames)) and len(frames) >= minimum
        assert len(report['pictures']) == len(frames) and len(report['sheets']) == (len(frames)+7)//8
        for item in report['pictures']+report['sheets']:
            assert base.sha(require(item['path'])) == item['sha256']
        root = base.read(require(root_path))
        independent = base.read(require(independent_path)) if independent_path is not None else None
        if admission is None:
            # the actual final extraction (option b): the original rule or a FINAL_ROUTES worker route
            assert approval == 'safe_to_advance_to_master'
            author = base.read(require(author_path)) if author_path is not None else None
            complete = final_review_checks(CHAPTER,c['source_sha'],report['movie_sha256'],base.sha(extraction_path),
                report,root,independent,author,*(base.sha(p) if p is not None else None for p in (independent_path,author_path)))
            assert not any(list(base.claimed(x)) for x in (root,independent,author) if x is not None)
            return report,complete
        for evidence in (x for x in (root,independent) if x is not None):
            assert evidence['movie_sha256'] == report['movie_sha256']
            assert evidence['extraction_sha256'] == base.sha(extraction_path)
            assert evidence['source_sha256'] == c['source_sha'] and evidence['concrete_defects'] == []
            assert evidence[approval] is True
        read = tuple(x for x in (root,independent) if x is not None)
        # the preview admission: the bound final runner's own checks (whole root review, or REVIEW_ROUTES)
        author = base.read(require(admission['author_review']['path'])) if CHAPTER in run.REVIEW_ROUTES else None
        run.review_checks(CHAPTER,admission,root,independent,author,report,base.sha(extraction_path))
        read += (author,) if author is not None else ()
        assert not any(list(base.claimed(evidence)) for evidence in read)
        return report,root
    admission_path = require(base.Q/ADMISSIONS[CHAPTER]); admission = base.read(admission_path)
    assert admission['preview_job'] == c['preview'] and admission['extraction_job'] == c['pictures']
    assert admission['all_selected_pictures_viewed'] is True and admission['concrete_defects'] == []
    assert (admission.get('independent_blockers', []) if author_only else admission['independent_blockers']) == []
    assert admission['safe_to_advance_to_final4k'] is True
    for k in keys:
        assert base.sha(require(admission[k]['path'])) == admission[k]['sha256']
    extraction_record = base.D/'jobs'/c['pictures']/CHAPTER/'pictures.json'
    assert admission['extraction_record']['path'] == str(extraction_record)
    preview_report,_ = pictures(extraction_record,admission['root_review']['path'],
        None if author_only else admission['independent_review']['path'],
        base.D/'jobs'/c['preview']/'preview'/(CHAPTER+'.mp4'),'safe_to_advance_to_final4k',MIN_PREVIEW[CHAPTER],admission)
    for frame in ANCHORS.get(CHAPTER,()):
        assert frame in preview_report['selected_frames'], '%s lint anchor %d was not extracted' % (CHAPTER,frame)
    origin = base.D/'jobs'/FINAL/'render'
    assert set(proofs) == {'prerequisites','extraction','root_review','independent_review','author_review'}
    report,final_viewed = pictures(proofs['extraction'],proofs['root_review'],proofs['independent_review'],
                        origin/(CHAPTER+'.mp4'),'safe_to_advance_to_master',len(preview_report['selected_frames']),
                        author_path=proofs['author_review'])
    assert set(preview_report['selected_frames']) <= set(report['selected_frames'])
    basis = derived_scope(base,admission,preview_report,report,final_viewed)
    for a,b in REQUIRED_SCOPE.get(CHAPTER,()):
        assert any(x <= a and b <= y for x,y in basis['sampled_intervals']), '%s: required window %d-%d is not in the scope' % (CHAPTER,a,b)
    assert row['picture_review'] == dict(path=proofs['root_review'],sha256=base.sha(proofs['root_review']))
    assert report['movie_sha256'] == made['raw_movie_sha256']
    outputs = {x['path']:x['sha256'] for x in made['files']}; assert len(outputs) == len(made['files'])
    render = stage/'render'
    for name,digest in outputs.items():
        path = Path(name); assert path.is_relative_to(render) and not path.is_symlink()
        assert base.sha(path) == digest; files.add(path)
    for ext in ('timeline.tsv','cues.tsv','cuelog.tsv'): require(base.TIMELINES/(CHAPTER+'.'+ext))
    require(base.SCORE/'score21-summary.json')
    files |= base.schedule(CHAPTER,render,base.NARRATION313,FRAMES)
    movie = render/(CHAPTER+'.mp4'); assert base.sha(movie) == made['raw_movie_sha256']
    probe = require('/usr/bin/ffprobe')
    streams = json.loads(subprocess.check_output([str(probe),'-v','error','-show_streams','-of','json',str(movie)],timeout=60))['streams']
    videos = [s for s in streams if s['codec_type']=='video']
    assert len(videos)==1
    video = videos[0]
    assert (video['width'],video['height'],video['avg_frame_rate'],int(video['nb_frames'])) == (3840,2160,'60/1',FRAMES)
    settings = base.x264_settings(movie)
    assert settings == made['encoder'] and ' crf=16.0' in settings and ' rc=crf' in settings
    assert {e:base.sha(require(origin/(CHAPTER+'.'+e))) for e in ('timeline.tsv','cues.tsv','cuelog.tsv')} == made['raw_tables']
    capture = render/'soundtrack'/CHAPTER/'soundtrack_0.wav'
    assert Path(made['capture_source']).parent == origin/'soundtrack'/CHAPTER
    assert base.sha(capture) == base.sha(require(made['capture_source'])) == c['staged993']
    assert base.sha(render/(CHAPTER+'.layout.tsv')) == base.sha(require(origin/(CHAPTER+'.layout.tsv')))
    with wave.open(str(capture)) as w:
        assert (w.getframerate(),w.getnchannels(),w.getsampwidth(),w.getnframes()) == (48000,2,2,FRAMES*800)
    staged = [render/(CHAPTER+suffix) for suffix in base.SUFFIXES]+[capture]
    for p in staged:
        assert outputs[str(p)] == base.sha(p) and p.stat().st_dev == (base.D/'jobs').stat().st_dev
    record = dict(chapter=CHAPTER,kind=c['kind'],host=str(base.D),job=row['job'],
        worker=str(worker),worker_sha256=WORKER_SHA,binary_sha256=BINARY,narration=str(base.NARRATION313),
        frames=FRAMES,movie_sha256=base.sha(movie),staged=[str(p) for p in staged],encoder=settings,
        native_final_samples=dict(pictures=len(report['selected_frames']),sheets=len(report['sheets']),record=row['picture_review']),
        native_preview_capture=dict(source=str(preview_capture),sha256=c['preview_pcm'],bit_identical_to_retained_preview=True),
        preview_admission=dict(path=str(admission_path),sha256=base.sha(admission_path)),
        local_proofs=dict(path=str(proofs_path),sha256=base.sha(proofs_path)),**basis)
    if CHAPTER == BOUNDARY:
        record.update(capture_check_before_master=replay_check,original313_basis=original313)
    return record,files
