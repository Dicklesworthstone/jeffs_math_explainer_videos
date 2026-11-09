"""Native09 master gate; preserves original independent preview and full schedule checks."""
import re
import wave
from pathlib import Path

MAC = Path('/Users/jemanuel/scratch/jmev-complex-native-20261109')
BINARY = '119ebe43fbde39e90cb1352656be280b71d52aa736751f0e6f819af6ceb9613d'
STAGER_SHA = 'bc2b3dc320da6ea516f2baad37dcfc248f8d492eec37e84acd3447209be43550'

def gate(base, row, frames_of):
    key = '09_boundary'
    assert row['chapter'] == key and row['kind'] == 'native553' and frames_of[key] == 16504
    assert set(row) == {'chapter', 'kind', 'job', 'worker', 'worker_sha256', 'inputs',
                        'import_receipt', 'import_inputs', 'picture_review'}
    assert re.fullmatch('csd-native-stage595-09-a[1-9][0-9]*', row['job'])
    worker = base.Q/'root-boundary595-materialize.py'
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
    transport_path = base.Q/'root-boundary594-transport.py'
    assert base.sha(transport_path) == 'b894e3f75c7e4a4f22c55f1e87097fc2267e614cb617343839bafded05f297dc'
    transport = base.load(transport_path, 'native594')
    manifest = base.absolute(result['manifest'])
    assert base.sha(manifest) == made['import_manifest_sha256'] == result['manifest_sha256']
    transport.closure(base.read(manifest), physical)
    files.update((transport_path, manifest))
    qualification = physical[str(MAC/'jobs/native-boundary553/qualification.json')]
    assert base.sha(qualification) == 'aee4c6fe288ad2842b19f2b88a8f5b322c0b7bdd0246322872f4e89c02c3ac1c'
    assert base.read(qualification)['binary_sha256'] == BINARY
    assert base.sha(physical[str(MAC/'jobs/native-boundary553/complex_analysis_visual-553')]) == BINARY
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
    preview_folder=MAC/'jobs/mini-boundary-preview571/preview/soundtrack/09_boundary'
    captures=[name for name in physical if Path(name).parent==preview_folder and re.fullmatch(r'soundtrack_[0-9]+\.wav',Path(name).name)]
    assert captures,'Native571 preview capture absent from import closure'
    latest=max(captures,key=lambda n:int(Path(n).stem.split('_')[-1]))
    assert base.sha(physical[latest])==replay_check['binary313_capture']['sha256'],'Native553 preview PCM differs; investigate before master'
    # Root must actually view every extracted final sample. No independent final
    # AV or continuous-motion claim is inferred from that selected-frame pass.
    viewing = row['picture_review']
    assert set(viewing) == {'path', 'sha256'}
    viewing_path = base.absolute(viewing['path'])
    assert viewing_path.parent == base.Q and base.sha(viewing_path) == viewing['sha256']
    seen = base.read(viewing_path)
    extraction = physical[str(MAC/'jobs/mini-boundary-pictures587/pictures587.json')]
    report = base.read(extraction)
    assert seen['movie_sha256'] == report['movie_sha256'] == made['raw_movie_sha256']
    assert seen['extraction_manifest_sha256'] == base.sha(extraction)
    assert seen['selected_pictures_viewed'] == 313 and seen['sheets_viewed'] == list(range(1,41))
    assert seen['concrete_picture_defects_observed'] == [] and seen['observations']
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
    assert base.sha(render/(key+'.layout.tsv')) == base.sha(physical[str(MAC/'jobs/mini-boundary-final586/render'/(key+'.layout.tsv'))])
    with wave.open(str(capture)) as wav:
        assert (wav.getframerate(), wav.getnchannels(), wav.getsampwidth(), wav.getnframes()) == (48000,2,2,16504*800)
    staged = [render/(key+suffix) for suffix in base.SUFFIXES]+[capture]
    for path in staged:
        assert outputs[str(path)] == base.sha(path) and path.stat().st_dev == (base.D/'jobs').stat().st_dev
    record = dict(chapter=key, kind='native553', host=str(MAC), job=row['job'],
                  worker=str(worker), worker_sha256=STAGER_SHA, binary_sha256=BINARY,
                  narration=str(base.NARRATION313), frames=16504, movie_sha256=base.sha(movie),
                  staged=[str(p) for p in staged], import_receipt=row['import_receipt'],
                  encoder=settings, native_final_samples=dict(pictures=313, sheets=40, record=viewing),
                  **basis)
    return record, files
