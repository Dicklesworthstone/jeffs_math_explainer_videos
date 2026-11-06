"""Revalidate completed FMD chapters and list exact files for lossless transfer."""
import importlib.util
import json
from pathlib import Path
import sys

HERE = Path('/data/projects/jmev-fmd-render-qualification-20261106-1111/qualification01/raster-quality03')
spec = importlib.util.spec_from_file_location('driver', HERE/'launch-fmd-disjoint12.py')
D = importlib.util.module_from_spec(spec)
spec.loader.exec_module(D)
W = D.W

def export():
    assignment = W.read(W.ASSIGNMENT)
    chapters = [r['chapter'] for r in assignment['assignments'] if r['host']=='fmd']
    assert 0 < len(chapters) < 15
    records = [D.completed(chapter) for chapter in chapters]
    files = []
    def add(source, destination):
        source = Path(source)
        assert source.is_relative_to(D.ROOT)
        assert not Path(destination).is_absolute() and '..' not in Path(destination).parts
        files.append(dict(source=str(source), destination=destination,
                          sha256=W.sha(source), bytes=source.stat().st_size))
    add(W.ASSIGNMENT, 'chapter-assignment12.json')
    add(W.PERMIT, 'distributed-review-permit12.json')
    for chapter,record in zip(chapters,records):
        job = D.ROOT/'jobs'/('fmd-raster-final-'+chapter+'-12')
        add(D.manifest(chapter), chapter+'/inputs.json')
        for filename in ('resource-receipt.json','render-summary.json','command.log'):
            add(job/filename, chapter+'/'+filename)
        for row in record['files']:
            add(row['path'], chapter+'/render/'+Path(row['path']).name)
        add(record['soundtrack']['path'], chapter+'/soundtrack.wav')
    assert len(files) == len({r['destination'] for r in files}) == 2 + 10*len(chapters)
    assert [D.completed(chapter) for chapter in chapters] == records
    result = dict(host='fmd', binary_sha256=W.BINARY_SHA, records=records, files=files,
                  scope='Current FMD input hashes and terminal chapters revalidated; destination hashes and native replay still required.')
    path = HERE/'final-export12.json'
    if path.exists():
        assert W.read(path) == result
    else:
        with path.open('x') as stream:
            json.dump(result,stream,indent=2)
    print(f'{len(chapters)} chapters revalidated; {len(files)} transfer files enumerated, none transferred.')

if __name__ == '__main__':
    assert sys.argv[1:] == ['--export']
    export()
