"""Native10 master gate; preserves original independent preview and full schedule checks."""
import re
import wave
from pathlib import Path

MAC = Path('/Users/jemanuel/scratch/jmev-complex-native-20261109')
BINARY = '5b77a1c8a33a41bab3c3987c9d4f40797bb6a6f1a76bbb57f1ab562ae9960312'
STAGER_SHA = '822047a7f57877378140bb7c81ffd5d89ebd99d47310d17f7d341db6a3b59806'

def gate(base, row, frames_of):
    key = '10_rigidity'
    assert row['chapter'] == key and row['kind'] == 'native521' and frames_of[key] == 7442
    assert set(row) == {'chapter', 'kind', 'job', 'worker', 'worker_sha256', 'inputs',
                        'import_receipt', 'import_inputs', 'picture_review'}
    assert re.fullmatch('csd-native-stage581-10-a[1-9][0-9]*', row['job'])
    worker = base.Q/'root-native581-materialize.py'
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
    assert made['chapter'] == key and made['frames'] == 7442
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
    transport_path = base.Q/'root-mini576-transport.py'
    assert base.sha(transport_path) == '5739b6901d895b02668f6642437260e02afea78078d55a55347ed9ae39c9b969'
    transport = base.load(transport_path, 'native576')
    manifest = base.absolute(result['manifest'])
    assert base.sha(manifest) == made['import_manifest_sha256'] == result['manifest_sha256']
    transport.closure(base.read(manifest), physical)
    files.update((transport_path, manifest))
    qualification = physical[str(MAC/'jobs/native-build521/qualification.json')]
    assert base.sha(qualification) == '63533e3c4ede63060a1801ddf1aa0ff3775f9d6c84851ad5b4ec32accc4fbb60'
    assert base.read(qualification)['binary_sha256'] == BINARY
    assert base.sha(physical[str(MAC/'jobs/native-build521/complex_analysis_visual-521')]) == BINARY
    # Keep the original 268 preview's independent review and source bindings.
    old_worker = base.WORKER352
    assert base.sha(old_worker) == base.WORKERS352[old_worker]
    old_module = base.load(old_worker, 'native521_original_preview')
    permit = base.Q/'10_rigidity-csd-permit366.json'
    review = base.read(permit)
    basis, _, used = base.combined_scope(key, review, base.BINARY_268,
                                       base.TIMELINES/(key+'.timeline.tsv'), old_worker, old_module)
    files |= used | base.independent_files(review) | {old_worker, permit}
    for item in review['source_binding_evidence']:
        assert base.sha(item['path']) == item['sha256']
        files.add(Path(item['path']))
    # Root must actually view every extracted final sample. No independent final
    # AV or continuous-motion claim is inferred from that selected-frame pass.
    viewing = row['picture_review']
    assert set(viewing) == {'path', 'sha256'}
    viewing_path = base.absolute(viewing['path'])
    assert viewing_path.parent == base.Q and base.sha(viewing_path) == viewing['sha256']
    seen = base.read(viewing_path)
    extraction = physical[str(MAC/'jobs/mini-final-pictures560/pictures560.json')]
    report = base.read(extraction)
    assert seen['movie_sha256'] == report['movie_sha256'] == made['raw_movie_sha256']
    assert seen['extraction_manifest_sha256'] == base.sha(extraction)
    assert seen['selected_pictures_viewed'] == 113 and seen['sheets_viewed'] == list(range(1,16))
    assert seen['concrete_picture_defects_observed'] == [] and seen['observations']
    assert not list(base.claimed(seen)), 'Selected frames must not claim release or AV acceptance'
    files.add(viewing_path)
    outputs = {r['path']: r['sha256'] for r in made['files']}
    assert len(outputs) == len(made['files'])
    for name, digest in outputs.items():
        assert Path(name).is_relative_to(stage/'render') and base.sha(name) == digest
        files.add(Path(name))
    render = stage/'render'
    files |= base.schedule(key, render, base.NARRATION, 7442)
    movie = render/(key+'.mp4')
    assert base.sha(movie) == made['raw_movie_sha256']
    settings = base.x264_settings(movie)
    assert settings == made['encoder'] and ' crf=16.0' in settings and ' rc=crf' in settings
    capture = render/'soundtrack'/key/'soundtrack_0.wav'
    assert base.sha(capture) == base.sha(physical[made['capture_source']])
    assert base.sha(render/(key+'.layout.tsv')) == base.sha(physical[str(MAC/'jobs/mini-final555/render'/(key+'.layout.tsv'))])
    with wave.open(str(capture)) as wav:
        assert (wav.getframerate(), wav.getnchannels(), wav.getsampwidth(), wav.getnframes()) == (48000,2,2,7442*800)
    staged = [render/(key+suffix) for suffix in base.SUFFIXES]+[capture]
    for path in staged:
        assert outputs[str(path)] == base.sha(path) and path.stat().st_dev == (base.D/'jobs').stat().st_dev
    record = dict(chapter=key, kind='native521', host=str(MAC), job=row['job'],
                  worker=str(worker), worker_sha256=STAGER_SHA, binary_sha256=BINARY,
                  narration=str(base.NARRATION), frames=7442, movie_sha256=base.sha(movie),
                  staged=[str(p) for p in staged], import_receipt=row['import_receipt'],
                  encoder=settings, native_final_samples=dict(pictures=113, sheets=15, record=viewing),
                  **basis)
    return record, files
