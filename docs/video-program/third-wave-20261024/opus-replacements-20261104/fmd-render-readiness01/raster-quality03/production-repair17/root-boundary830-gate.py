"""Native09 master gate; preserves original independent preview and full schedule checks."""
import re
import wave
from pathlib import Path

MAC = Path('/Users/jemanuel/scratch/jmev-complex-native-20261109')
LOCAL_W = Path('/Users/jemanuel/projects/jeffs_math_explainer_videos/narration_lab/third-wave-20261024/root/five-film-release-20261027')
BINARY = '71736c26d9953f08af8f4ce4f6fc0f13aeb98e7135b3dd415fb3ebf00e161c26'
STAGER_SHA = '7a3b9ecf0b7209ff4fab7ad01574a610b2b7b1a7fef482db9808e2143cd081d1'

def gate(base, row, frames_of):
    key = '09_boundary'
    assert row['chapter'] == key and row['kind'] == 'native761' and frames_of[key] == 16504
    assert set(row) == {'chapter', 'kind', 'job', 'worker', 'worker_sha256', 'inputs',
                        'import_receipt', 'import_inputs', 'picture_review'}
    assert re.fullmatch('csd-native-stage829-09-a[1-9][0-9]*', row['job'])
    worker = base.Q/'root-boundary829-materialize.py'
    assert row['worker'] == str(worker) and row['worker_sha256'] == STAGER_SHA == base.sha(worker)
    stage = base.D/'jobs'/row['job']
    inputs = base.absolute(row['inputs'])
    assert inputs == base.Q/(row['job']+'-inputs.json')
    state = base.success(stage/'resource-receipt.json')
    assert base.sha(inputs) == state['inputs_sha256']
    files = {worker, inputs, stage/'resource-receipt.json', stage/'materialized.json', Path(__file__).resolve()}
    entries = base.read(inputs)['files']
    bound = {x['path']: x['sha256'] for x in entries}
    assert len(bound) == len(entries)
    for name, digest in bound.items():
        assert base.sha(name) == digest, name
        files.add(Path(name))
    assert bound[str(worker)] == STAGER_SHA
    made = base.read(stage/'materialized.json')
    assert made['chapter'] == key and made['frames'] == 16504
    assert state['command'] == ['/usr/bin/python3', '-I', '-B', str(worker),
                                '--import-job', made['import_job'], '--job', row['job'], '--inputs', str(inputs)]
    imported = base.D/'jobs'/made['import_job']
    assert row['import_receipt'] == str(imported/'resource-receipt.json')
    assert row['import_inputs'] == str(base.Q/(made['import_job']+'-inputs.json'))
    files |= base.imported(row)
    result = base.read(imported/'import-result.json')
    physical = {r['path']: Path(r['physical']) for r in result['files']}
    assert len(physical) == len(result['files'])
    for r in result['files']:
        assert base.sha(r['physical']) == r['sha256'] and bound.get(r['physical']) == r['sha256']
    files.add(imported/'import-result.json')
    transport_path = base.Q/'root-boundary828-transport.py'
    assert base.sha(transport_path) == '42665086de4112b7593d3a96e5077e2febc97742e80fe1c3510c471f6ee020b6'
    transport = base.load(transport_path, 'native778')
    manifest = base.absolute(result['manifest'])
    assert base.sha(manifest) == made['import_manifest_sha256'] == result['manifest_sha256']
    transport.closure(base.read(manifest), physical)
    files.update((transport_path, manifest))
    qualification = transport.physical_file(physical, MAC/'jobs/native-boundary761/qualification.json')
    assert base.sha(qualification) == 'b225700d230f5880663e786d83b3dc9a43d0776dc9c07fa3d669de6ab6198ba8'
    assert base.read(qualification)['binary_sha256'] == BINARY
    assert base.sha(transport.physical_file(physical, MAC/'jobs/native-boundary761/complex_analysis_visual-761')) == BINARY
    # Preserve original313 source, independent preview and capture requirements.
    old = next(x for x in base.read(base.Q/'cc1-master447-manifest-draft534.json')['chapters'] if x['chapter']==key)
    assert old['kind']=='boundary313' and old['worker']==str(base.WORKER344)
    old_worker=base.WORKER344
    assert base.sha(old_worker)==old['worker_sha256']
    module=base.load(old_worker,'native553_original_preview')
    binary313,used=base.provenance313(16504)
    files |= used
    permit=base.absolute(old['permit']);review=base.read(permit)
    basis,used=base.sampled_permit_gate(key,review,binary313,base.TIMELINE313/'timeline-output'/(key+'.timeline.tsv'),old_worker,module)
    files |= used | base.independent344(old,old_worker,review) | {old_worker,permit,base.Q/'cc1-master447-manifest-draft534.json'}
    binding=module.preview_binding(key,review)
    files.update(Path(p) for p in binding)
    files.update(Path(p) for p in module.independent(review))
    checked=module.check(key,permit)
    assert (checked[0],checked[1],[str(p) for p in checked[2]])==(review,base.sha(permit),[str(p) for p in binding])
    for item in review['source_binding_evidence']:
        assert base.sha(item['path'])==item['sha256'];files.add(Path(item['path']))
    replay_check,used=base.capture_check09(16504)
    files |= used
    # Same narration-only native preview must mix to identical retained PCM.
    preview_folder=Path(transport.retained_key(MAC/'jobs/mini-boundary-preview766/preview/soundtrack/09_boundary'))
    captures=[name for name in physical if Path(name).parent==preview_folder and re.fullmatch(r'soundtrack_[0-9]+\.wav',Path(name).name)]
    assert captures,'Native766 preview capture absent from import closure'
    latest=max(captures,key=lambda n:int(Path(n).stem.split('_')[-1]))
    assert base.sha(physical[latest])==replay_check['binary313_capture']['sha256'],'Native761 preview PCM differs; investigate before master'
    # Root must actually view every extracted final sample. No independent final
    # AV or continuous-motion claim is inferred from that selected-frame pass.
    viewing = row['picture_review']
    assert set(viewing) == {'path', 'sha256'}
    viewing_path = base.absolute(viewing['path'])
    assert viewing_path.parent == base.Q and base.sha(viewing_path) == viewing['sha256']
    seen = base.read(viewing_path)
    extraction = physical[str(LOCAL_W/'complex-local-pictures820/pictures820.json')]
    report = base.read(extraction)
    assert seen['movie_sha256'] == report['movie_sha256'] == made['raw_movie_sha256']
    assert seen['extraction_manifest_sha256'] == base.sha(extraction)
    count = len(report['selected_frames'])
    assert count == len(report['pictures']) >= 814
    assert len(report['sheets']) == (count+7)//8
    assert seen['complete'] is True and seen['selected_pictures_viewed'] == count
    assert seen['picture_indices_viewed'] == list(range(1,count+1))
    assert seen['sheets_viewed'] == list(range(1,len(report['sheets'])+1))
    assert seen['concrete_defects'] == [] and seen['observations']
    assert not list(base.claimed(seen)), 'Selected frames must not claim release or AV acceptance'
    files.add(viewing_path)
    outputs = {r['path']: r['sha256'] for r in made['files']}
    assert len(outputs) == len(made['files'])
    for name, digest in outputs.items():
        assert Path(name).is_relative_to(stage/'render') and base.sha(name) == digest
        files.add(Path(name))
    render = stage/'render'
    files |= base.schedule(key, render, base.NARRATION313, 16504)
    movie = render/(key+'.mp4')
    assert base.sha(movie) == made['raw_movie_sha256']
    settings = base.x264_settings(movie)
    assert settings == made['encoder'] and ' crf=16.0' in settings and ' rc=crf' in settings
    capture = render/'soundtrack'/key/'soundtrack_0.wav'
    assert base.sha(capture) == base.sha(physical[made['capture_source']])
    assert base.sha(render/(key+'.layout.tsv')) == base.sha(physical[str(LOCAL_W/'complex-local-final819/render'/(key+'.layout.tsv'))])
    with wave.open(str(capture)) as wav:
        assert (wav.getframerate(), wav.getnchannels(), wav.getsampwidth(), wav.getnframes()) == (48000,2,2,16504*800)
    staged = [render/(key+suffix) for suffix in base.SUFFIXES]+[capture]
    for path in staged:
        assert outputs[str(path)] == base.sha(path) and path.stat().st_dev == (base.D/'jobs').stat().st_dev
    record = dict(chapter=key, kind='native761', host=str(MAC), job=row['job'],
                  worker=str(worker), worker_sha256=STAGER_SHA, binary_sha256=BINARY,
                  narration=str(base.NARRATION313), frames=16504, movie_sha256=base.sha(movie),
                  staged=[str(p) for p in staged], import_receipt=row['import_receipt'],
                  encoder=settings, native_final_samples=dict(pictures=count, sheets=len(report['sheets']), record=viewing),
                  capture_check_before_master=replay_check,
                  native_preview_capture=dict(source=latest, sha256=base.sha(physical[latest]),
                                              bit_identical_to_retained_preview=True),
                  **basis)
    return record, files
