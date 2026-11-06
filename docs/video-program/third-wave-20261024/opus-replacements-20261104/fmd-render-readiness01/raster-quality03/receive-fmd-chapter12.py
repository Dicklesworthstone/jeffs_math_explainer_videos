"""Receive one exact FMD final chapter under the original CSD resource guard."""
import argparse,hashlib,importlib.util,json,sys
from pathlib import Path
ROOT=Path('/data/projects/jmev-stroke-probe-20261106')
HERE=ROOT/'qualification01/raster-quality12'
REMOTE=Path('/data/projects/jmev-fmd-render-qualification-20261106-1111')
EXPORT=HERE/'final-fmd-export12.json'
CAP=4*1024**3-16*1024**2

def read(p):return json.loads(p.read_text())
def sha(p):
    with p.open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest()
def load_worker():
    spec=importlib.util.spec_from_file_location('worker',HERE/'csd-final-chapter12.py')
    w=importlib.util.module_from_spec(spec);spec.loader.exec_module(w);return w

def selected_rows(export,chapter):
    rows=export['files']
    assert len(rows)==len({r['destination'] for r in rows})
    selected=[r for r in rows if r['destination'] in ('chapter-assignment12.json','distributed-review-permit12.json') or r['destination'].startswith(chapter+'/')]
    expected={'chapter-assignment12.json','distributed-review-permit12.json'}
    expected.update(chapter+'/'+n for n in ('inputs.json','resource-receipt.json','render-summary.json','command.log','soundtrack.wav'))
    expected.update(chapter+'/render/'+chapter+s for s in ('.mp4','.timeline.tsv','.cues.tsv','.cuelog.tsv','.layout.tsv'))
    assert len(selected)==12 and {r['destination'] for r in selected}==expected
    for row in selected:
        path=Path(row['destination']);source=Path(row['source'])
        assert not path.is_absolute() and '..' not in path.parts
        assert source.is_relative_to(REMOTE) and '..' not in source.parts
        assert type(row['bytes']) is int and row['bytes']>=0
    assert 0<sum(r['bytes'] for r in selected)<CAP, 'Oversized chapter: preserve guard, use separately bounded transfer chunks'
    return selected

def receive(source,destination,rows):
    assert not destination.exists()
    destination.mkdir()
    checked=[]
    for row in rows:
        path=destination/row['destination'];path.parent.mkdir(parents=True,exist_ok=True)
        remaining=row['bytes'];digest=hashlib.sha256()
        with path.open('xb') as out:
            while remaining:
                block=source.read(min(1024*1024,remaining))
                assert block and len(block)<=remaining, 'Premature EOF; preserve partial files'
                out.write(block);digest.update(block);remaining-=len(block)
        assert digest.hexdigest()==row['sha256'], 'Digest mismatch; preserve partial transfer'
        checked.append(dict(path=str(path),bytes=row['bytes'],sha256=digest.hexdigest()))
    assert source.read(1)==b'', 'Unexpected trailing bytes'
    return checked

def run(chapter):
    w=load_worker();w.check_media_paths();w.check_current_preview()
    assignment=read(w.ASSIGNMENT);permit=read(w.PERMIT)
    csd=[r['chapter'] for r in assignment['assignments'] if r['host']=='csd'];assert csd
    w.check(csd[0])
    keys=[r['chapter'] for r in assignment['assignments'] if r['host']=='fmd'];assert chapter in keys
    export=read(EXPORT)
    assert export['host']=='fmd' and export['binary_sha256']==w.BINARY_SHA
    assert [r['chapter'] for r in export['records']]==keys
    assert all(r['review']==permit['review_gate_result'] and r['permit_sha256']==sha(w.PERMIT) for r in export['records'])
    rows=selected_rows(export,chapter)
    job=ROOT/'jobs'/('csd-fmd-final-import-'+chapter+'-12')
    receipt=read(job/'resource-receipt.json')
    assert receipt['admitted'] and 'exit' not in receipt and receipt['policy']==w.POLICY
    assert receipt['command']==['/usr/bin/python3','-I','-B',str(Path(__file__).resolve()),'--chapter',chapter]
    out=job/'import';checked=receive(sys.stdin.buffer,out,rows)
    assert (out/w.PERMIT.name).read_bytes()==w.PERMIT.read_bytes()
    assert (out/w.ASSIGNMENT.name).read_bytes()==w.ASSIGNMENT.read_bytes()
    with (out/'final-export12.json').open('xb') as f:f.write(EXPORT.read_bytes())
    w.check(csd[0])
    with (job/'transfer-summary.json').open('x') as f:
        json.dump(dict(chapter=chapter,export_sha256=sha(EXPORT),files=checked,bytes=sum(r['bytes'] for r in checked),scope='Exact transfer only; imported media and native replay still require verification.'),f,indent=2)
    print('Chapter bytes transferred and verified; media qualification remains.')
if __name__=='__main__':
    p=argparse.ArgumentParser();p.add_argument('--chapter',required=True);a=p.parse_args();run(a.chapter)
