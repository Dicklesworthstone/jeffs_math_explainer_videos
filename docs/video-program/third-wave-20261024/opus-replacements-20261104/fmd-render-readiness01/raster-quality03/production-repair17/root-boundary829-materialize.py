"""Inside a new guarded job, bind imported native09 tables to canonical CSD audio.

Preserves raw imported bytes. Only path columns in separately written tables
change. Does not grant final picture, listening, motion or independent acceptance.
"""
import argparse
import hashlib
import importlib.util
import json
import os
import re
import wave
from pathlib import Path

D = Path('/data/projects/jmev-stroke-probe-20261106')
Q = D/'qualification01/production-repair17'
MAC = Path('/Users/jemanuel/scratch/jmev-complex-native-20261109')
LOCAL_W = Path('/Users/jemanuel/projects/jeffs_math_explainer_videos/narration_lab/third-wave-20261024/root/five-film-release-20261027')
CHAPTER = '09_boundary'
PINS = {
    'root-boundary828-transport.py': '42665086de4112b7593d3a96e5077e2febc97742e80fe1c3510c471f6ee020b6',
    'root-native574-table-rebind.py': '92407bbf10b0a83e8a19a106a15d0c17752503f9bf6c0887ee25fd246103bd8f',
    'combined-master447.py': '2b01ecaab2b409268d86fe13b83e97a7e374dd61f1c657680a32444fdb6aa524',
}

def sha(path):
    with Path(path).open('rb') as f:
        return hashlib.file_digest(f, 'sha256').hexdigest()

def read(path):
    return json.loads(Path(path).read_text())

def load(name):
    p = Q/name
    assert sha(p) == PINS[name], p
    spec = importlib.util.spec_from_file_location(name.replace('-', '_'), p)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module

def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--import-job', required=True)
    parser.add_argument('--job', required=True)
    parser.add_argument('--inputs', type=Path, required=True)
    a = parser.parse_args()
    assert re.fullmatch('csd-local-import828-09-a[1-9][0-9]*', a.import_job)
    assert re.fullmatch('csd-native-stage829-09-a[1-9][0-9]*', a.job)
    job, source = D/'jobs'/a.job, D/'jobs'/a.import_job
    assert sorted(p.name for p in job.iterdir()) == ['command.log', 'resource-receipt.json']
    state = read(job/'resource-receipt.json')
    assert state.get('admitted') and not state.get('preflight_only')
    assert not state.get('finished') and 'exit' not in state
    expected = ['/usr/bin/python3', '-I', '-B', str(Path(__file__).resolve()),
                '--import-job', a.import_job, '--job', a.job, '--inputs', str(a.inputs)]
    assert state['command'] == expected and a.inputs.parent == Q
    assert sha(a.inputs) == state['inputs_sha256']
    rows = read(a.inputs)['files']
    frozen = {r['path']: r['sha256'] for r in rows}
    assert len(rows) == len(frozen)

    def require(path):
        path = Path(path)
        assert frozen.get(str(path)) == sha(path), path
        return path

    for name in frozen:
        require(name)
    require(Path(__file__).resolve())
    for name in PINS:
        require(Q/name)
    transport = load('root-boundary828-transport.py')
    assert {k: state['policy'][k] for k in transport.POLICY} == transport.POLICY
    imported = read(require(source/'resource-receipt.json'))
    transport.terminal(imported)
    import_inputs = require(Q/(a.import_job+'-inputs.json'))
    assert sha(import_inputs) == imported['inputs_sha256']
    for row in read(import_inputs)['files']:
        assert sha(require(row['path'])) == row['sha256']
    record = read(require(source/'import-result.json'))
    manifest_path = require(record['manifest'])
    assert sha(manifest_path) == record['manifest_sha256']
    manifest = read(manifest_path)
    physical = {r['path']: require(r['physical']) for r in record['files']}
    assert len(physical) == len(record['files']) == len(manifest['files'])
    assert all(p.is_relative_to(source/'incoming') and not p.is_symlink() for p in physical.values())
    transport.closure(manifest, physical)
    master = load('combined-master447.py')
    mapper = load('root-native574-table-rebind.py')
    origin = LOCAL_W/'complex-local-final819/render'
    tables = {ext: physical[str(origin/(CHAPTER+'.'+ext))].read_bytes()
              for ext in ('timeline.tsv', 'cues.tsv', 'cuelog.tsv')}
    bindings = {}
    narration = LOCAL_W/'complex-local-delivery818/narration'
    stem_dir = MAC/'score586'
    stem_names = {f'09_boundary.music.f32.wav.part{i:04d}.wav' for i in range(3)}
    actual_stems = []
    for line in tables['cuelog.tsv'].decode().splitlines():
        if not line.strip() or line.startswith('#'):
            continue
        fields = line.split('\t')
        assert len(fields) == 13
        old = Path(fields[0])
        if old.parent == narration:
            target = master.NARRATION313/old.name
        else:
            assert old.parent == stem_dir and old.name in stem_names, old
            actual_stems.append(old.name)
            target = master.STEMS/old.name
        require(target)
        assert sha(transport.physical_file(physical, old)) == sha(target) == fields[1]
        bindings[str(old)] = dict(path=str(target), sha256=fields[1])
    assert set(actual_stems) == stem_names and len(actual_stems) == 3
    canonical, edits = mapper.rebind(tables, bindings)
    out = job/'render'
    out.mkdir()
    for ext, data in canonical.items():
        (out/(CHAPTER+'.'+ext)).open('xb').write(data)
    for ext in ('mp4', 'layout.tsv'):
        os.link(physical[str(origin/(CHAPTER+'.'+ext))], out/(CHAPTER+'.'+ext))
    captures = [name for name in physical if Path(name).parent == origin/'soundtrack'/CHAPTER
                and re.fullmatch(r'soundtrack_[0-9]+\.wav', Path(name).name)]
    assert captures
    capture_name = max(captures, key=lambda n: int(Path(n).stem.split('_')[-1]))
    capture = physical[capture_name]
    with wave.open(str(capture)) as stream:
        assert (stream.getframerate(), stream.getnchannels(), stream.getsampwidth(), stream.getnframes()) == (48000, 2, 2, 16504*800)
    dest = out/'soundtrack'/CHAPTER/'soundtrack_0.wav'
    dest.parent.mkdir(parents=True)
    os.link(capture, dest)
    for path in master.schedule(CHAPTER, out, master.NARRATION313, 16504):
        require(path)
    settings = master.x264_settings(out/(CHAPTER+'.mp4'))
    assert ' rc=crf' in settings and ' crf=16.0' in settings
    require(master.SCORE/'score21-summary.json')
    for ext in ('timeline.tsv', 'cues.tsv', 'cuelog.tsv'):
        require(master.TIMELINES/(CHAPTER+'.'+ext))
    for name in frozen:
        require(name)
    result = dict(chapter=CHAPTER, frames=16504, import_job=a.import_job,
                  import_manifest_sha256=sha(manifest_path), path_edits=edits,
                  raw_tables={ext: sha(physical[str(origin/(CHAPTER+'.'+ext))]) for ext in tables},
                  files=[dict(path=str(p), sha256=sha(p)) for p in sorted(out.rglob('*')) if p.is_file()],
                  capture_source=capture_name, raw_movie_sha256=manifest['movie_sha256'], encoder=settings,
                  scope='Canonical path binding and native schedule check only. Raw import preserved. Final picture, motion, listening, fine sync, independent AV and master replay checks remain open.')
    (job/'materialized.json').open('x').write(json.dumps(result, indent=2)+'\n')

if __name__ == '__main__':
    main()
