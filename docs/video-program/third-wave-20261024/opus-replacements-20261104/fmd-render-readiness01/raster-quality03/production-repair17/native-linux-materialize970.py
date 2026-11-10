"""PREPARED, NOT LAUNCHED: guarded same-host native844 boundary materialization.

--proofs is a frozen JSON object with prerequisites [{receipt, inputs}], extraction,
root_review, independent_review paths. Every path and its transitive immutable
closure must be in --inputs. Reviews must bind actual final968, not preview853.

A fresh master gate must revalidate this job/inputs/materialized record and all
proofs, old313 historical gates, independent preview and PCM evidence, all15
replay/null tests, and full master verification. This helper grants none of those.
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
CHAPTER = '09_boundary'
NARRATION = D/'jobs/csd-native-delivery849/narration'
ORIGIN = D/'jobs/csd-native-final968/render'
SOURCE_SHA = 'cd1900aadd46090083729d9aa768e59e1551c42ab94e5bd055f520cb4eff5e6e'
MANIFEST_SHA = '6fde63a4029af7765491d0f38f5371feb701f954277d2cf3728d9f0ae4220942'
PINS = {
    'root-native574-table-rebind.py': '92407bbf10b0a83e8a19a106a15d0c17752503f9bf6c0887ee25fd246103bd8f',
    'combined-master447.py': '2b01ecaab2b409268d86fe13b83e97a7e374dd61f1c657680a32444fdb6aa524',
}
POLICY = dict(admission_gib=10, stop_free_gib=8, phase_growth_gib=4,
              work_cap_gib=56, target_growth_cap_gib=20, sample_seconds=10)

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

def main():
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument('--job', required=True)
    ap.add_argument('--inputs', type=Path, required=True)
    ap.add_argument('--proofs', type=Path, required=True)
    ap.add_argument('--manifest', type=Path, required=True)
    a = ap.parse_args()
    assert re.fullmatch(r'csd-native-stage970-09-a[1-9][0-9]*', a.job)
    job = D/'jobs'/a.job
    assert sorted(p.name for p in job.iterdir()) == ['command.log','resource-receipt.json']
    state = read(job/'resource-receipt.json')
    assert state['admitted'] and not state.get('preflight_only') and 'finished' not in state and 'exit' not in state
    assert state['policy'] == POLICY and state['phase'] == 'native'
    assert os.environ['CARGO_TARGET_DIR'] == str(D/'work')
    assert state['command'] == [sys.executable,'-I','-B',str(Path(__file__).resolve()),
        '--job',a.job,'--inputs',str(a.inputs),'--proofs',str(a.proofs),'--manifest',str(a.manifest)]
    assert a.inputs.parent == Q and sha(a.inputs) == state['inputs_sha256']
    entries = read(a.inputs)['files']; bound = {r['path']:r['sha256'] for r in entries}
    assert len(bound) == len(entries)
    def require(p):
        p = Path(p)
        assert p.is_absolute() and p.is_file() and not p.is_symlink(), p
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
        ('csd-native-build844','csd-native-delivery849','csd-native-final968')}
    actual = set()
    for item in proofs['prerequisites']:
        receipt = require(item['receipt']); actual.add(receipt); r = success(receipt)
        manifest = require(item['inputs']); assert sha(manifest) == r['inputs_sha256']
        for row in read(manifest)['files']:
            assert sha(require(row['path'])) == row['sha256']
    assert required <= actual
    #844's full immutable input closure excludes the later declared canonical rebind;
    #849 and final968 closures and proof artifacts bind that explicit successor state.
    qualification = read(require(D/'jobs/csd-native-build844/qualification.json'))
    binary = require(D/'jobs/csd-native-build844/complex_analysis_visual-844')
    assert sha(binary) == qualification['binary_sha256']
    assert qualification['tests_passed'] == 235 and qualification['clippy_passed']
    assert qualification['explicit_native_plate_fade_probe_passed']
    assert sha(require(D/'work/native-boundary844/workspace/explainer_corpus/complex_analysis_visual/src/chapters/boundary.rs')) == SOURCE_SHA
    extraction = require(proofs['extraction']); report = read(extraction)
    movie = require(ORIGIN/(CHAPTER+'.mp4'))
    assert report['movie'] == str(movie) and report['movie_sha256'] == sha(movie)
    assert report['frames'] == 16504 and report['full_decode'] is True
    assert extraction.parent/'resource-receipt.json' in actual
    frames = report['selected_frames']
    assert frames == sorted(set(frames)) and len(frames) >= 814
    assert len(report['pictures']) == len(frames) and len(report['sheets']) == (len(frames)+7)//8
    for item in report['pictures']+report['sheets']:
        assert sha(require(item['path'])) == item['sha256']
    review_paths = [require(proofs[k]) for k in ('root_review','independent_review')]
    reviews = [read(p) for p in review_paths]
    for review in reviews:
        assert review['movie_sha256'] == sha(movie) and review['extraction_sha256'] == sha(extraction)
        assert review['source_sha256'] == SOURCE_SHA and review['concrete_defects'] == []
        assert review['safe_to_advance_to_master'] is True
    assert reviews[0]['viewed_frame_indices'] == frames
    assert reviews[0]['viewed_sheets'] == [x['path'] for x in report['sheets']]
    assert reviews[1]['authored_repair'] is False and reviews[1]['viewed_frame_indices']
    assert set(reviews[1]['viewed_frame_indices']) <= set(frames)
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
    stem_names = {f'09_boundary.music.f32.wav.part{i:04d}.wav' for i in range(3)}
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
    assert set(stems) == stem_names and len(stems) == 3
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
        assert (w.getframerate(),w.getnchannels(),w.getsampwidth(),w.getnframes()) == (48000,2,2,16504*800)
    dest = out/'soundtrack'/CHAPTER/'soundtrack_0.wav'; dest.parent.mkdir(parents=True)
    assert capture.stat().st_dev == job.stat().st_dev; os.link(capture,dest)
    # schedule reads these four evidence files without returning them.
    for ext in ('timeline.tsv','cues.tsv','cuelog.tsv'):
        require(master.TIMELINES/(CHAPTER+'.'+ext))
    require(master.SCORE/'score21-summary.json')
    for p in master.schedule(CHAPTER,out,old,16504): require(p)
    require(Path('/usr/bin/ffprobe'))
    os.environ['PATH'] = '/usr/bin:/bin'
    settings = master.x264_settings(movie); assert ' crf=16.0' in settings and ' rc=crf' in settings
    original_manifest = read(require(a.manifest)); assert sha(a.manifest) == MANIFEST_SHA
    rows = original_manifest['chapters']; assert len(rows) == 15 and len({x['chapter'] for x in rows}) == 15
    assert sum(x['chapter'] == CHAPTER for x in rows) == 1
    new_row = dict(chapter=CHAPTER,kind='native844',job=a.job,worker=str(Path(__file__).resolve()),
        worker_sha256=sha(__file__),inputs=str(a.inputs),proofs=str(a.proofs),
        picture_review=dict(path=str(review_paths[0]),sha256=sha(review_paths[0])))
    successor = dict(original_manifest,chapters=[new_row if x['chapter']==CHAPTER else x for x in rows])
    assert [x for x in successor['chapters'] if x['chapter']!=CHAPTER] == [x for x in rows if x['chapter']!=CHAPTER]
    write(job/'master-manifest.json',successor)
    for p in bound: require(p)
    write(job/'materialized.json',dict(chapter=CHAPTER,frames=16504,source_job='csd-native-final968',
        binary_sha256=sha(binary),proofs=dict(path=str(a.proofs),sha256=sha(a.proofs)),
        original_manifest_sha256=sha(a.manifest),unchanged_other_manifest_rows=14,path_edits=edits,
        raw_tables={e:sha(ORIGIN/(CHAPTER+'.'+e)) for e in tables},raw_movie_sha256=sha(movie),
        capture_source=str(capture),encoder=settings,files=[dict(path=str(p),sha256=sha(p)) for p in sorted(out.rglob('*')) if p.is_file()],
        scope='Same-host path materialization only; fresh master gate, all15 PCM replays, fullAV/listening/fine-sync and publication remain separate.'))

if __name__ == '__main__': main()
