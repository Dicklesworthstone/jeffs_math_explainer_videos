"""PREPARED ONLY: strict same-host native844 boundary gate for a fresh831 successor.

Interface gate(base,row,frames_of) -> (record, used_paths). Never executes a
render/master. Keeps original313 history, new Linux evidence and final pictures
separate. Outer master must retain all14 other gates and all15 PCM null tests.
"""
from pathlib import Path
import re
import json
import subprocess
import wave

CHAPTER = '09_boundary'
WORKER_SHA = '6a40df5cd80c1dce577eb1b0dfd2c27cedbaa946d42743492c201cee93df0bb9'
BINARY_SHA = 'e75258731b1786ae3182ef9eb21a7903a9e6c0bed6242413335f7dd233955384'
SOURCE_SHA = 'cd1900aadd46090083729d9aa768e59e1551c42ab94e5bd055f520cb4eff5e6e'
PCM_SHA = 'f7f8aa6042a9f05d1b25db4c789f7f6b44cb393445015c057f1e87d9b0c8df51'
MANIFEST625_SHA = '6fde63a4029af7765491d0f38f5371feb701f954277d2cf3728d9f0ae4220942'


def gate(base, row, frames_of):
    assert row['chapter'] == CHAPTER and row['kind'] == 'native844'
    assert frames_of[CHAPTER] == 16504
    assert set(row) == {'chapter','kind','job','worker','worker_sha256','inputs','proofs','picture_review'}
    assert re.fullmatch(r'csd-native-stage970-09-a[1-9][0-9]*',row['job'])
    worker = base.Q/'native-linux-materialize970.py'
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
        assert p.is_absolute() and p.is_file() and not p.is_symlink(),p
        assert bound.get(str(p)) == base.sha(p),p
        files.add(p); return p
    for p in bound: require(p)
    require(worker)
    made = base.read(stage/'materialized.json')
    assert made['chapter'] == CHAPTER and made['frames'] == 16504
    assert made['source_job'] == 'csd-native-final968' and made['binary_sha256'] == BINARY_SHA
    proofs_path = require(row['proofs'])
    assert made['proofs'] == dict(path=str(proofs_path),sha256=base.sha(proofs_path))
    proofs = base.read(proofs_path)
    # Bind exact guarded invocation, including the original manifest argument.
    command = state['command']
    assert len(command) == 12 and command[:4] == ['/usr/bin/python3','-I','-B',str(worker)]
    assert command[4:10] == ['--job',row['job'],'--inputs',str(inputs),'--proofs',str(proofs_path)]
    assert command[10] == '--manifest'
    original_manifest = require(command[11])
    assert base.sha(original_manifest) == MANIFEST625_SHA == made['original_manifest_sha256']
    assert made['unchanged_other_manifest_rows'] == 14
    original_rows = base.read(original_manifest)['chapters']
    proposed_path = stage/'master-manifest.json'; files.add(proposed_path)
    proposed = base.read(proposed_path)['chapters']
    assert len(original_rows) == len(proposed) == 15
    assert [r['chapter'] for r in original_rows] == [r['chapter'] for r in proposed]
    assert [r for r in original_rows if r['chapter'] != CHAPTER] == [r for r in proposed if r['chapter'] != CHAPTER]
    assert [r for r in proposed if r['chapter'] == CHAPTER] == [row]
    expected_jobs = {base.D/'jobs'/j/'resource-receipt.json' for j in
        ('csd-native-build844','csd-native-delivery849','csd-native-final968')}
    actual_jobs = set()
    for proof in proofs['prerequisites']:
        receipt = require(proof['receipt']); actual_jobs.add(receipt)
        terminal = base.success(receipt); manifest = require(proof['inputs'])
        assert base.sha(manifest) == terminal['inputs_sha256']
        for item in base.read(manifest)['files']:
            assert base.sha(require(item['path'])) == item['sha256']
    assert expected_jobs <= actual_jobs
    final_runner = require(base.Q/'csd-native968-run.py')
    assert base.sha(final_runner) == '2634c842bacc9b5894511c47ca22c16d1d952e40ea126fd1844d69914e0e48dc'
    final_state = base.success(require(base.D/'jobs/csd-native-final968/resource-receipt.json'))
    assert final_state['command'] == ['/usr/bin/python3','-I','-B',str(final_runner)]
    qualification = base.read(require(base.D/'jobs/csd-native-build844/qualification.json'))
    assert qualification['binary_sha256'] == BINARY_SHA
    assert qualification['tests_passed'] == 235 and qualification['clippy_passed'] is True
    assert qualification['explicit_native_plate_fade_probe_passed'] is True
    assert base.sha(require(base.D/'jobs/csd-native-build844/complex_analysis_visual-844')) == BINARY_SHA
    assert base.sha(require(base.D/'work/native-boundary844/workspace/explainer_corpus/complex_analysis_visual/src/chapters/boundary.rs')) == SOURCE_SHA
    assert base.read(require(base.D/'jobs/csd-native-delivery849/delivery849.json'))['wavs_byte_identical'] == 105

    # Preserve original313 independent source/picture/capture gates, as830 did.
    old = next(x for x in base.read(base.Q/'cc1-master447-manifest-draft534.json')['chapters'] if x['chapter'] == CHAPTER)
    assert old['kind'] == 'boundary313' and old['worker'] == str(base.WORKER344)
    old_worker = base.WORKER344; assert base.sha(old_worker) == old['worker_sha256']
    module = base.load(old_worker,'native844_original_preview')
    binary313,used = base.provenance313(16504); files |= used
    permit = base.absolute(old['permit']); historical_review = base.read(permit)
    basis,used = base.sampled_permit_gate(CHAPTER,historical_review,binary313,
        base.TIMELINE313/'timeline-output'/(CHAPTER+'.timeline.tsv'),old_worker,module)
    files |= used | base.independent344(old,old_worker,historical_review) | {old_worker,permit,base.Q/'cc1-master447-manifest-draft534.json'}
    binding = module.preview_binding(CHAPTER,historical_review)
    files.update(Path(p) for p in binding); files.update(Path(p) for p in module.independent(historical_review))
    checked = module.check(CHAPTER,permit)
    assert (checked[0],checked[1],[str(p) for p in checked[2]]) == (historical_review,base.sha(permit),[str(p) for p in binding])
    for item in historical_review['source_binding_evidence']:
        assert base.sha(item['path']) == item['sha256']; files.add(Path(item['path']))
    replay_check,used = base.capture_check09(16504); files |= used
    preview_capture = require(base.D/'jobs/csd-native-preview853/preview/soundtrack/09_boundary/soundtrack_0.wav')
    assert base.sha(preview_capture) == PCM_SHA == replay_check['binary313_capture']['sha256']

    def pictures(extraction_path,root_path,independent_path,movie,approval):
        extraction_path = require(extraction_path); report = base.read(extraction_path)
        assert extraction_path.parent/'resource-receipt.json' in actual_jobs or extraction_path.parent.name == 'csd-native-pictures855'
        base.success(require(extraction_path.parent/'resource-receipt.json'))
        assert report['movie'] == str(movie) and report['movie_sha256'] == base.sha(require(movie))
        assert report['frames'] == 16504 and report['full_decode'] is True
        frames = report['selected_frames']; assert frames == sorted(set(frames)) and len(frames) >= 814
        assert len(report['pictures']) == len(frames) and len(report['sheets']) == (len(frames)+7)//8
        for item in report['pictures']+report['sheets']:
            assert base.sha(require(item['path'])) == item['sha256']
        root,independent = base.read(require(root_path)),base.read(require(independent_path))
        for evidence in (root,independent):
            assert evidence['movie_sha256'] == report['movie_sha256']
            assert evidence['extraction_sha256'] == base.sha(extraction_path)
            assert evidence['source_sha256'] == SOURCE_SHA and evidence['concrete_defects'] == []
            assert evidence[approval] is True
        assert root['viewed_frame_indices'] == frames
        assert root['viewed_sheets'] == [x['path'] for x in report['sheets']]
        assert independent['authored_repair'] is False and independent['viewed_frame_indices']
        assert set(independent['viewed_frame_indices']) <= set(frames)
        assert not list(base.claimed(root)) and not list(base.claimed(independent))
        return report
    admission = base.read(require(base.Q/'csd-native864-picture-admission.json'))
    assert admission['preview_job'] == 'csd-native-preview853' and admission['extraction_job'] == 'csd-native-pictures855'
    for k in ('root_review','independent_review','extraction_record'):
        assert base.sha(require(admission[k]['path'])) == admission[k]['sha256']
    assert admission['extraction_record']['path'] == str(base.D/'jobs/csd-native-pictures855/pictures855.json')
    pictures(admission['extraction_record']['path'],admission['root_review']['path'],admission['independent_review']['path'],
             base.D/'jobs/csd-native-preview853/preview/09_boundary.mp4','safe_to_advance_to_final4k')
    origin = base.D/'jobs/csd-native-final968/render'
    report = pictures(proofs['extraction'],proofs['root_review'],proofs['independent_review'],
                      origin/(CHAPTER+'.mp4'),'safe_to_advance_to_master')
    assert row['picture_review'] == dict(path=proofs['root_review'],sha256=base.sha(proofs['root_review']))
    assert report['movie_sha256'] == made['raw_movie_sha256']
    outputs = {x['path']:x['sha256'] for x in made['files']}; assert len(outputs) == len(made['files'])
    render = stage/'render'
    for name,digest in outputs.items():
        path = Path(name); assert path.is_relative_to(render) and not path.is_symlink()
        assert base.sha(path) == digest; files.add(path)
    for ext in ('timeline.tsv','cues.tsv','cuelog.tsv'): require(base.TIMELINES/(CHAPTER+'.'+ext))
    require(base.SCORE/'score21-summary.json')
    files |= base.schedule(CHAPTER,render,base.NARRATION313,16504)
    movie = render/(CHAPTER+'.mp4'); assert base.sha(movie) == made['raw_movie_sha256']
    probe = require('/usr/bin/ffprobe')
    streams = json.loads(subprocess.check_output([str(probe),'-v','error','-show_streams','-of','json',str(movie)],timeout=60))['streams']
    videos = [s for s in streams if s['codec_type']=='video']
    assert len(videos)==1
    video = videos[0]
    assert (video['width'],video['height'],video['avg_frame_rate'],int(video['nb_frames'])) == (3840,2160,'60/1',16504)
    settings = base.x264_settings(movie)
    assert settings == made['encoder'] and ' crf=16.0' in settings and ' rc=crf' in settings
    assert {e:base.sha(require(origin/(CHAPTER+'.'+e))) for e in ('timeline.tsv','cues.tsv','cuelog.tsv')} == made['raw_tables']
    capture = render/'soundtrack'/CHAPTER/'soundtrack_0.wav'
    assert Path(made['capture_source']).parent == origin/'soundtrack'/CHAPTER
    assert base.sha(capture) == base.sha(require(made['capture_source']))
    assert base.sha(render/(CHAPTER+'.layout.tsv')) == base.sha(require(origin/(CHAPTER+'.layout.tsv')))
    with wave.open(str(capture)) as w:
        assert (w.getframerate(),w.getnchannels(),w.getsampwidth(),w.getnframes()) == (48000,2,2,16504*800)
    staged = [render/(CHAPTER+suffix) for suffix in base.SUFFIXES]+[capture]
    for p in staged:
        assert outputs[str(p)] == base.sha(p) and p.stat().st_dev == (base.D/'jobs').stat().st_dev
    record = dict(chapter=CHAPTER,kind='native844',host=str(base.D),job=row['job'],
        worker=str(worker),worker_sha256=WORKER_SHA,binary_sha256=BINARY_SHA,narration=str(base.NARRATION313),
        frames=16504,movie_sha256=base.sha(movie),staged=[str(p) for p in staged],encoder=settings,
        native_final_samples=dict(pictures=len(report['selected_frames']),sheets=len(report['sheets']),record=row['picture_review']),
        capture_check_before_master=replay_check,native_preview_capture=dict(source=str(preview_capture),sha256=PCM_SHA,bit_identical_to_retained_preview=True),
        local_proofs=dict(path=str(proofs_path),sha256=base.sha(proofs_path)),**basis)
    return record,files
