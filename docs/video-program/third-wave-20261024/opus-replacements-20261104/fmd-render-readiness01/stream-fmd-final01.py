"""Stream only the exact revalidated export payload to stdout. No shell/tar extraction."""
import importlib.util
import json
from pathlib import Path
import sys

HERE=Path('/data/projects/jmev-fmd-render-qualification-20261106-1111/qualification01')
spec=importlib.util.spec_from_file_location('driver',HERE/'launch-fmd-disjoint01.py')
D=importlib.util.module_from_spec(spec)
spec.loader.exec_module(D)

def stream():
    export=D.W.read(HERE/'final-export01.json')
    records=export['records']
    assert [D.completed(r['chapter']) for r in records]==records
    files=export['files']
    assert len(files)==len({r['source'] for r in files})==72
    for row in files:
        path=Path(row['source'])
        assert path.is_relative_to(D.ROOT)
        assert path.stat().st_size==row['bytes'] and D.W.sha(path)==row['sha256']
    for row in files:
        with Path(row['source']).open('rb') as source:
            while block:=source.read(1024*1024):
                sys.stdout.buffer.write(block)
        assert D.W.sha(row['source'])==row['sha256']
    sys.stdout.buffer.flush()

if __name__=='__main__':
    assert sys.argv[1:]==['--stream']
    stream()
