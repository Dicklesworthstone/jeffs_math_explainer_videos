"""Guard child: receive exact bounded files into the counted job directory."""
import hashlib
import importlib.util
import json
from pathlib import Path
import sys

ROOT=Path('/data/projects/jmev-stroke-probe-20261106')
HERE=ROOT/'qualification01'
JOB=ROOT/'jobs/stage07-fmd-import01'
OUT=JOB/'import'
EXPORT=HERE/'final-fmd-export01.json'
REMOTE=Path('/data/projects/jmev-fmd-render-qualification-20261106-1111')

def receive(source, destination, rows):
    assert not destination.exists()
    assert len(rows)==len({r['destination'] for r in rows})==72
    total=sum(r['bytes'] for r in rows)
    assert 0 < total < 4*1024**3 - 16*1024**2, 'Split oversized transfer into new guarded jobs; never bypass cap'
    for row in rows:
        path=Path(row['destination'])
        assert not path.is_absolute() and '..' not in path.parts
        assert Path(row['source']).is_relative_to(REMOTE)
        assert '..' not in Path(row['source']).parts
        assert type(row['bytes']) is int and row['bytes']>=0
    destination.mkdir()
    checked=[]
    for row in rows:
        path=destination/row['destination']
        path.parent.mkdir(parents=True,exist_ok=True)
        remaining=row['bytes'];digest=hashlib.sha256()
        with path.open('xb') as out:
            while remaining:
                block=source.read(min(1024*1024,remaining))
                assert block, 'Premature EOF; preserve partial transfer'
                assert len(block)<=remaining
                out.write(block);digest.update(block);remaining-=len(block)
        assert digest.hexdigest()==row['sha256'], 'Transfer mismatch; preserve attempt'
        checked.append(dict(path=str(path),bytes=row['bytes'],sha256=digest.hexdigest()))
    assert source.read(1)==b'', 'Unexpected trailing bytes'
    return checked

def run():
    spec=importlib.util.spec_from_file_location('launch',HERE/'launch-final02.py')
    L=importlib.util.module_from_spec(spec);spec.loader.exec_module(L)
    receipt=L.S.read(JOB/'resource-receipt.json')
    assert receipt['admitted'] and 'exit' not in receipt and receipt['policy']==L.S.POLICY
    assert receipt['command']==['/usr/bin/python3','-I','-B',str(Path(__file__).resolve()),'--run']
    review=L.F.G.check(L.F.REVIEW)
    export=L.S.read(EXPORT)
    assignment=L.S.read(HERE/'chapter-assignment01.json')
    keys=[r['chapter'] for r in assignment['assignments'] if r['host']=='fmd']
    assert len(keys)==7 and [r['chapter'] for r in export['records']]==keys
    assert export['binary_sha256']==L.S.BINARY_SHA and export['host']=='fmd'
    assert all(r['review']==review for r in export['records'])
    checked=receive(sys.stdin.buffer,OUT,export['files'])
    with (OUT/'final-export01.json').open('xb') as out:
        out.write(EXPORT.read_bytes())
    assert L.F.G.check(L.F.REVIEW)==review
    with (JOB/'transfer-summary.json').open('x') as out:
        json.dump(dict(files=checked,bytes=sum(r['bytes'] for r in checked),review=review,
            scope='Transferred bytes verified only; imported media qualification and native master replay remain.'),out,indent=2)
    print('FMD transfer bytes verified; import media checks remain.')

if __name__=='__main__':
    assert sys.argv[1:]==['--run']
    run()
