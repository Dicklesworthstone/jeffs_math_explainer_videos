"""Read-only stream of one revalidated FMD final chapter."""
import importlib.util
from pathlib import Path
import sys

HERE=Path('/data/projects/jmev-fmd-render-qualification-20261106-1111/qualification01/raster-quality03')
s=importlib.util.spec_from_file_location('exporter',HERE/'export-fmd-final12.py')
E=importlib.util.module_from_spec(s);s.loader.exec_module(E)

def main():
    assert len(sys.argv)==3 and Path(__file__).resolve().parent==HERE
    chapter,expected=sys.argv[1:]
    manifest=HERE/'final-export12.json'
    assert E.W.sha(manifest)==expected
    export=E.W.read(manifest)
    records=[r for r in export['records'] if r['chapter']==chapter]
    assert len(records)==1 and E.D.completed(chapter)==records[0]
    rows=[r for r in export['files'] if r['destination'] in ('chapter-assignment12.json','distributed-review-permit12.json') or r['destination'].startswith(chapter+'/')]
    assert len(rows)==12 and len({r['destination'] for r in rows})==12
    assert sum(r['bytes'] for r in rows)<4*1024**3-16*1024**2
    for row in rows:
        p=Path(row['source'])
        assert p.is_relative_to(E.D.ROOT) and '..' not in p.parts
        assert p.stat().st_size==row['bytes'] and E.W.sha(p)==row['sha256']
    for row in rows:
        with Path(row['source']).open('rb') as f:
            while block:=f.read(1024*1024):sys.stdout.buffer.write(block)
    sys.stdout.buffer.flush()

if __name__=='__main__':main()
