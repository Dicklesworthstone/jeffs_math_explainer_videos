"""PREPARED, NOT LAUNCHED, NOT REVIEWED (CC0 scratch draft final4k-impl02): guarded same-host
materialization of one repaired chapter row for the final 4K successor of master993.

Successor of native-linux-materialize991.py (3402d437), the 09-only native844 helper, for CC3's route
(final4k-integration-proposal-cc3 a60b6815, section 6) with CC0 review01 M1. Root and CC2 review it
before any use. impl02 = impl01 (85557673) with CC2 stage-review01 F1 fixed: impl01 took only part files,
so it refused 01, whose score21 stem is one whole file. Changes from 991, and nothing else:
  - --chapter picks one CHAPTERS row: 01/08/09 from the combined01 chain (kind native-combined01),
    11 from the counting02 chain (kind native-counting02);
  - prerequisites are that chain's build, delivery, timeline, preview and preview pictures, plus the
    chapter's final; qualification is that chain's binary, test count and workspace scene file;
  - the base manifest is master993's (8da77ed7); one row replaced, 14 unchanged;
  - the final extraction must hold every frame of the chapter's preview extraction (991: >= 814);
  - stem names come from score21-summary.json: the chapter's whole stem or its parts (score21 has 01 as
    one file, 08 and 09 as three parts, 11 as two); 09 must still give 991's three literals;
  - the final's scored capture must equal master993's staged capture of the chapter (STAGED993);
  - unresolved pins are None and refuse before anything is read: root sets them.

--proofs is a frozen JSON object with prerequisites [{receipt, inputs}], extraction,
root_review, independent_review paths. Every path and its transitive immutable
closure must be in --inputs. Reviews must bind the chapter's actual final, not its preview.

A fresh master gate must revalidate this job/inputs/materialized record and all proofs,
the chapter's actual preview admission and review scope, all15 replay/null tests, and full
master verification. This helper grants none of those.

CC0 successor (option b; owner via root, 2026-11-11): the actual final extraction's review may take a worker
route, as the previews did. final_review_checks() (identical in stage03, materializer03 and gate04) accepts
root's complete viewing plus an independent subset (the original rule), or root's targeted acceptance
(review_route in FINAL_ROUTES) that binds complete worker reviews of the actual final movie and extraction
by sha256, with honest authorship. The proofs name root_review, independent_review and author_review
(None where the route has none). Nothing else changes.
"""
import argparse
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import re
import sys
import wave

D = Path('/data/projects/jmev-stroke-probe-20261106')
Q = D/'qualification01/production-repair17'
MANIFEST_SHA = '8da77ed79e9ccc8a06e703e570c85c44de5cd884a047eaed61355167e397e100'   # master993's manifest
SCENE = 'workspace/explainer_corpus/complex_analysis_visual/src/chapters/'
COMBINED = dict(kind='native-combined01', build='csd-native-combined-build01',
    binary='complex_analysis_visual-combined01', tests=247, workspace=D/'work/native-combined01',
    delivery='csd-native-combined-delivery01', timeline='csd-native-combined-timeline01',
    preview='csd-native-combined-preview01', pictures='csd-native-combined-pictures01')
COUNTING = dict(kind='native-counting02', build='csd-native-counting-build02',
    binary='complex_analysis_visual-counting02', tests=250, workspace=D/'work/native-counting02',
    delivery='csd-native-counting-delivery02', timeline='csd-native-counting-timeline02',
    preview='csd-native-counting-preview02', pictures='csd-native-counting-pictures02')
# staged993: master993's staged scored capture of the chapter (01, 08: css352 import manifests 392/533;
# 09: master993 boundary_replay; 11: root's sha256sum on CSD, a recorded audio hash only).
CHAPTERS = {
    '01_the_bump': dict(COMBINED, frames=6311, scene='the_bump.rs',
        source_sha='90b51c2b30877911bbb5f5e14ee810268e14efb24e7c4e2a55f7f7e128c06ee7',
        staged993='4d77c2d66720d67160cea0feec6d86a5e5ab54720641d4586050104a2d592e07'),
    '08_loop_sums': dict(COMBINED, frames=14959, scene='loop_sums.rs',
        source_sha='a8d62f560321ea6ebea3d3859acd9a3ce3e5dde4375ec0ab42c1a1ba114d61ef',
        staged993='d713402c0c7fba3c17705fd624d38598b9aabebe2ef6220db9dc8cfeecbb328b'),
    '09_boundary': dict(COMBINED, frames=16504, scene='boundary.rs',
        source_sha='4b689791748132bd14c3929d191bb78ae7752bf9c3332beb22f774931873e932',
        staged993='29899f9e42721331e2abe9088c399bca3eb63ea3b50de45dc731cbb8c4404c6c'),
    '11_counting': dict(COUNTING, frames=11932, scene='counting.rs',
        source_sha='21c5043a3339cfadb10024a2e8227a62a0b06674139eace1a0d9029a4e1aa0db',
        staged993='82bdc443218a5a761c678244089bc8d983a01466c77a7efed23ea358a4ec490a'),
}
# UNRESOLVED pins: root sets each before use; main() refuses while any is None.
STAGE_VERSION = '1006'   # digits; stage jobs are csd-native-stage<VERSION>-<nn>-a<k>
FINAL_JOBS = {'01_the_bump': 'csd-native-repair-final01-01-the-bump', '08_loop_sums': 'csd-native-repair-final01-08-loop-sums', '09_boundary': 'csd-native-repair-final01-09-boundary', '11_counting': 'csd-native-repair-final01-11-counting'}   # the 968-run successor's job per chapter
PINS = {
    'root-native574-table-rebind.py': '92407bbf10b0a83e8a19a106a15d0c17752503f9bf6c0887ee25fd246103bd8f',
    'combined-master447.py': '2b01ecaab2b409268d86fe13b83e97a7e374dd61f1c657680a32444fdb6aa524',
}
POLICY = dict(admission_gib=10, stop_free_gib=8, phase_growth_gib=4,
              work_cap_gib=56, target_growth_cap_gib=20, sample_seconds=10)

def unresolved():
    missing = [] if STAGE_VERSION else ['STAGE_VERSION']
    return missing + ['FINAL_JOBS[%s]' % k for k, v in FINAL_JOBS.items() if not v]

def sha(p):
    with Path(p).open('rb') as f: return hashlib.file_digest(f, 'sha256').hexdigest()
def read(p): return json.loads(Path(p).read_text())
def write(p, value):
    with p.open('x') as f: json.dump(value, f, indent=2); f.write('\n')
def success(p):
    r = read(p)
    assert r.get('admitted') and r.get('finished') and r.get('exit') == 0
    assert not any(r.get(k) for k in ('error','resource_stop','stopped_for_resource_limit','preflight_only'))
    assert r['inputs_sha256'] == r['final_inputs_sha256']
    return r


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


def main():
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument('--chapter', required=True, choices=sorted(CHAPTERS))
    ap.add_argument('--job', required=True)
    ap.add_argument('--inputs', type=Path, required=True)
    ap.add_argument('--proofs', type=Path, required=True)
    ap.add_argument('--manifest', type=Path, required=True)
    a = ap.parse_args()
    missing = unresolved(); assert not missing, 'unresolved pins: '+', '.join(missing)
    assert re.fullmatch(r'[0-9]+', STAGE_VERSION)
    CHAPTER = a.chapter; c = CHAPTERS[CHAPTER]; FRAMES = c['frames']
    NARRATION = D/'jobs'/c['delivery']/'narration'
    FINAL = FINAL_JOBS[CHAPTER]
    ORIGIN = D/'jobs'/FINAL/'render'
    assert re.fullmatch(r'csd-native-stage%s-%s-a[1-9][0-9]*' % (STAGE_VERSION, CHAPTER[:2]), a.job)
    job = D/'jobs'/a.job
    assert sorted(p.name for p in job.iterdir()) == ['command.log','resource-receipt.json']
    state = read(job/'resource-receipt.json')
    assert state['admitted'] and not state.get('preflight_only') and 'finished' not in state and 'exit' not in state
    assert state['policy'] == POLICY and state['phase'] == 'native'
    assert os.environ['CARGO_TARGET_DIR'] == str(D/'work')
    assert state['command'] == [sys.executable,'-I','-B',str(Path(__file__).resolve()),'--chapter',CHAPTER,
        '--job',a.job,'--inputs',str(a.inputs),'--proofs',str(a.proofs),'--manifest',str(a.manifest)]
    assert a.inputs.parent == Q and sha(a.inputs) == state['inputs_sha256']
    entries = read(a.inputs)['files']; bound = {r['path']:r['sha256'] for r in entries}
    assert len(bound) == len(entries)
    def require(p):
        p = Path(p)
        target = alias_target(p)
        assert bound.get(str(ALIAS_FILE)) == ALIAS_SHA
        if target is not None: assert bound.get(str(target)) == sha(target), target
        assert bound.get(str(p)) == sha(p), p
        return p
    for p in bound: require(p)
    require(Path(__file__).resolve())
    def load(name):
        p = require(Q/name); assert sha(p) == PINS[name]
        spec = importlib.util.spec_from_file_location(name.replace('-','_'),p)
        m = importlib.util.module_from_spec(spec); spec.loader.exec_module(m); return m
    master, mapper = load('combined-master447.py'), load('root-native574-table-rebind.py')
    proofs = read(require(a.proofs))
    required = {D/'jobs'/n/'resource-receipt.json' for n in
        (c['build'],c['delivery'],c['timeline'],c['preview'],c['pictures'],FINAL)}
    actual = set()
    for item in proofs['prerequisites']:
        receipt = require(item['receipt']); actual.add(receipt); r = success(receipt)
        manifest = require(item['inputs']); assert sha(manifest) == r['inputs_sha256']
        for row in read(manifest)['files']:
            assert sha(require(row['path'])) == row['sha256']
    assert required <= actual
    # The build's immutable input closure excludes the later declared canonical rebind; the
    # delivery, timeline, preview, pictures and final closures and proof artifacts bind that state.
    qualification = read(require(D/'jobs'/c['build']/'qualification.json'))
    binary = require(D/'jobs'/c['build']/c['binary'])
    assert sha(binary) == qualification['binary_sha256']
    assert qualification['tests_passed'] == c['tests'] and qualification['clippy_passed']
    assert qualification['explicit_native_plate_fade_probe_passed']
    assert sha(require(c['workspace']/SCENE/c['scene'])) == c['source_sha']
    extraction = require(proofs['extraction']); report = read(extraction)
    movie = require(ORIGIN/(CHAPTER+'.mp4'))
    assert report['movie'] == str(movie) and report['movie_sha256'] == sha(movie)
    assert report['frames'] == FRAMES and report['full_decode'] is True
    assert D/'jobs'/extraction.relative_to(D/'jobs').parts[0]/'resource-receipt.json' in actual
    frames = report['selected_frames']
    previewed = read(require(D/'jobs'/c['pictures']/CHAPTER/'pictures.json'))
    assert previewed['movie'] == str(D/'jobs'/c['preview']/'preview'/(CHAPTER+'.mp4'))
    assert previewed['frames'] == FRAMES and previewed['full_decode'] is True
    assert frames == sorted(set(frames)) and set(previewed['selected_frames']) <= set(frames)
    assert len(report['pictures']) == len(frames) and len(report['sheets']) == (len(frames)+7)//8
    for item in report['pictures']+report['sheets']:
        assert sha(require(item['path'])) == item['sha256']
    assert set(proofs) == {'prerequisites','extraction','root_review','independent_review','author_review'}
    review_paths = [require(proofs['root_review'])] + [require(proofs[k]) if proofs[k] else None
                                                       for k in ('independent_review','author_review')]
    reviews = [read(p) if p else None for p in review_paths]
    final_review_checks(CHAPTER, c['source_sha'], sha(movie), sha(extraction), report, *reviews,
                        *(sha(p) if p else None for p in review_paths[1:]))
    old = master.NARRATION313
    names = {p.name for p in old.iterdir()}; assert names == {p.name for p in NARRATION.iterdir()}
    wavs = [n for n in names if n.endswith('.wav')]; assert len(wavs) == 105
    metadata = {'report.json','assembly.json','external-units.json'}
    for n in names-metadata: assert sha(require(old/n)) == sha(require(NARRATION/n))
    old_report,new_report = read(require(old/'report.json')),read(require(NARRATION/'report.json'))
    old_assembly,new_assembly = read(require(old/'assembly.json')),read(require(NARRATION/'assembly.json'))
    old_plan,new_plan = read(require(old/'external-units.json')),read(require(NARRATION/'external-units.json'))
    assert {k:v for k,v in old_plan.items() if k!='direction_contract'} == {k:v for k,v in new_plan.items() if k!='direction_contract'}
    assert old_report['lines'] == new_report['lines']
    assert old_assembly['outputs'] == new_assembly['outputs']
    assert old_assembly['manifest']['units'] == new_assembly['manifest']['units']
    tables = {e:require(ORIGIN/(CHAPTER+'.'+e)).read_bytes() for e in ('timeline.tsv','cues.tsv','cuelog.tsv')}
    bindings = {}; stems = []
    prefix = 'stems/'+CHAPTER+'.music.f32.wav'
    stem_names = {k[len('stems/'):] for k in read(require(master.SCORE/'score21-summary.json'))['files']
                  if k.startswith(prefix) and k.endswith('.wav')}
    assert stem_names
    if CHAPTER == '09_boundary':
        assert stem_names == {f'09_boundary.music.f32.wav.part{i:04d}.wav' for i in range(3)}
    for line in tables['cuelog.tsv'].decode().splitlines():
        if not line.strip() or line.startswith('#'): continue
        fields = line.split('\t'); assert len(fields) == 13
        src = require(fields[0])
        if src.parent == NARRATION: target = old/src.name
        else:
            assert src.parent == master.STEMS and src.name in stem_names
            stems.append(src.name); target = src
        assert sha(require(target)) == sha(src) == fields[1]
        bindings[str(src)] = dict(path=str(target),sha256=fields[1])
    assert set(stems) == stem_names and len(stems) == len(stem_names)
    canonical,edits = mapper.rebind(tables,bindings)
    out = job/'render'; out.mkdir()
    for ext,data in canonical.items():
        with (out/(CHAPTER+'.'+ext)).open('xb') as f: f.write(data)
    for ext in ('mp4','layout.tsv'):
        src = require(ORIGIN/(CHAPTER+'.'+ext)); assert src.stat().st_dev == job.stat().st_dev
        os.link(src,out/(CHAPTER+'.'+ext))
    captures = sorted((ORIGIN/'soundtrack'/CHAPTER).glob('soundtrack_*.wav'))
    assert len(captures) == 1, 'Account for every capture before materialization'
    capture = require(captures[0])
    with wave.open(str(capture)) as w:
        assert (w.getframerate(),w.getnchannels(),w.getsampwidth(),w.getnframes()) == (48000,2,2,FRAMES*800)
    dest = out/'soundtrack'/CHAPTER/'soundtrack_0.wav'; dest.parent.mkdir(parents=True)
    assert capture.stat().st_dev == job.stat().st_dev; os.link(capture,dest)
    # The repair changes no audio: the scored capture is master993's staged one (the master's replay
    # null test proves the same later).
    assert sha(dest) == c['staged993']
    # schedule reads these four evidence files without returning them.
    for ext in ('timeline.tsv','cues.tsv','cuelog.tsv'):
        require(master.TIMELINES/(CHAPTER+'.'+ext))
    require(master.SCORE/'score21-summary.json')
    for p in master.schedule(CHAPTER,out,old,FRAMES): require(p)
    require(Path('/usr/bin/ffprobe'))
    os.environ['PATH'] = '/usr/bin:/bin'
    settings = master.x264_settings(movie); assert ' crf=16.0' in settings and ' rc=crf' in settings
    original_manifest = read(require(a.manifest)); assert sha(a.manifest) == MANIFEST_SHA
    rows = original_manifest['chapters']; assert len(rows) == 15 and len({x['chapter'] for x in rows}) == 15
    assert sum(x['chapter'] == CHAPTER for x in rows) == 1
    new_row = dict(chapter=CHAPTER,kind=c['kind'],job=a.job,worker=str(Path(__file__).resolve()),
        worker_sha256=sha(__file__),inputs=str(a.inputs),proofs=str(a.proofs),
        picture_review=dict(path=str(review_paths[0]),sha256=sha(review_paths[0])))
    successor = dict(original_manifest,chapters=[new_row if x['chapter']==CHAPTER else x for x in rows])
    assert [x for x in successor['chapters'] if x['chapter']!=CHAPTER] == [x for x in rows if x['chapter']!=CHAPTER]
    write(job/'master-manifest.json',successor)
    for p in bound: require(p)
    write(job/'materialized.json',dict(chapter=CHAPTER,frames=FRAMES,source_job=FINAL,
        binary_sha256=sha(binary),proofs=dict(path=str(a.proofs),sha256=sha(a.proofs)),
        original_manifest_sha256=sha(a.manifest),unchanged_other_manifest_rows=14,path_edits=edits,
        raw_tables={e:sha(ORIGIN/(CHAPTER+'.'+e)) for e in tables},raw_movie_sha256=sha(movie),
        capture_source=str(capture),staged993_capture_sha256=c['staged993'],encoder=settings,
        files=[dict(path=str(p),sha256=sha(p)) for p in sorted(out.rglob('*')) if p.is_file()],
        scope='Same-host path materialization only; fresh master gate, preview admission and review scope, all15 PCM replays, fullAV/hearing/fine-sync and publication remain separate.'))

if __name__ == '__main__': main()
