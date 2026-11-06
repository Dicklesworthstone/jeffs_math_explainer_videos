"""Freeze transfer inputs on CSD; no transfer or render starts here."""
import importlib.util
import json
from pathlib import Path
import sys

ROOT=Path('/data/projects/jmev-stroke-probe-20261106')
HERE=ROOT/'qualification01'

def prepare():
    spec=importlib.util.spec_from_file_location('launcher',HERE/'launch-final02.py')
    L=importlib.util.module_from_spec(spec);spec.loader.exec_module(L)
    assert not (ROOT/'jobs/stage07-fmd-import01').exists(), 'Preserve prior attempts'
    review=L.F.G.check(L.F.REVIEW)
    source=HERE/'final-fmd-export01.json'
    export=L.S.read(source)
    assignment=L.S.read(HERE/'chapter-assignment01.json')
    keys=[r['chapter'] for r in assignment['assignments'] if r['host']=='fmd']
    assert len(keys)==7 and [r['chapter'] for r in export['records']]==keys
    assert export['host']=='fmd' and export['binary_sha256']==L.S.BINARY_SHA
    assert all(r['review']==review for r in export['records'])
    assert len(export['files'])==72
    assert sum(r['bytes'] for r in export['files']) < 4*1024**3 - 16*1024**2
    frozen=L.frozen_inputs(review)
    paths={r['path']:r['sha256'] for r in frozen['files']}
    for path in (Path(__file__).resolve(),source,HERE/'receive-fmd-final01.py',
                 HERE/'chapter-assignment01.json',HERE/'distributed-review-permit01.json'):
        digest=L.S.sha(path)
        if str(path) in paths:assert paths[str(path)]==digest
        paths[str(path)]=digest
    frozen['files']=[dict(path=p,sha256=h) for p,h in sorted(paths.items())]
    manifest=ROOT/'stage07-fmd-import01-inputs.json'
    if manifest.exists():assert L.S.read(manifest)==frozen
    else:
        with manifest.open('x') as out:json.dump(frozen,out,indent=2)
    print('Import inputs frozen; guarded receiver still must be explicitly launched.')

if __name__=='__main__':
    assert sys.argv[1:]==['--prepare']
    prepare()
