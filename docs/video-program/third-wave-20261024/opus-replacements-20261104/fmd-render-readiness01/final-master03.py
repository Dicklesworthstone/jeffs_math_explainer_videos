"""Native master adapter for disjoint CSD and imported FMD chapters.

Retains the native master's bit-identical replay and all original media checks.
Different host manifests are validated individually, not falsely equated.
"""
import importlib.util
import json
from pathlib import Path
import subprocess
import sys

HERE = Path('/data/projects/jmev-stroke-probe-20261106/qualification01')

def load(name):
    spec=importlib.util.spec_from_file_location(name,HERE/name)
    module=importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module

M=load('final-master02.py')
I=load('verify-fmd-import01.py')
R=load('chapter-routing01.py')
M.ME=HERE/'final-master03.py'

def chapters():
    review=M.F.G.check(M.F.REVIEW)
    assignment=M.S.read(HERE/'chapter-assignment01.json')
    R.validate(assignment,M.S.read(M.ROOT/'jobs/stage07-timeline01/timeline-summary.json')['chapters'])
    imported=I.verify()
    remote={r['chapter']:r for r in imported}
    assert len(remote)==len(imported)==7
    records=[]
    for row in assignment['assignments']:
        chapter=row['chapter']
        if row['host']=='csd':
            record=M.L.verify_completed(chapter,review)
        else:
            record=remote[chapter]
        assert record['frames']==row['frames'] and record['fps']==60
        assert record['review']==review
        records.append(record)
    assert [r['chapter'] for r in records]==list(M.S.CHAPTERS)
    return records

M.chapters=chapters

def launch():
    assert Path(__file__).resolve()==M.ME
    assert not M.JOB.exists() and not M.MANIFEST.exists(), 'Preserve prior master attempts'
    records=chapters()
    review=M.F.G.check(M.F.REVIEW)
    frozen=M.L.frozen_inputs(review)
    paths={r['path']:r['sha256'] for r in frozen['files']}
    extra=[M.ME,HERE/'final-master02.py',HERE/'verify-fmd-import01.py',
           HERE/'chapter-routing01.py',HERE/'chapter-assignment01.json',
           HERE/'distributed-review-permit01.json',I.IMPORT/'final-export01.json']
    for row in M.S.read(I.IMPORT/'final-export01.json')['files']:
        extra.append(I.IMPORT/row['destination'])
    assignment=M.S.read(HERE/'chapter-assignment01.json')
    for row,record in zip(assignment['assignments'],records):
        chapter=row['chapter']
        if row['host']=='csd':
            manifest=M.L.manifest(chapter)
            bound=M.B.collect(manifest,[])
            for item in bound['files']:
                if item['path'] in paths:
                    assert paths[item['path']]==item['sha256']
                paths[item['path']]=item['sha256']
            job=M.ROOT/'jobs'/M.F.job_name(chapter)
            extra.extend([manifest,job/'resource-receipt.json',job/'render-summary.json',job/'command.log'])
        extra.extend(Path(item['path']) for item in record['files'])
        extra.append(Path(record['soundtrack']['path']))
    for path in extra:
        digest=M.S.sha(path)
        if str(path) in paths:
            assert paths[str(path)]==digest
        paths[str(path)]=digest
    frozen['files']=[dict(path=p,sha256=h) for p,h in sorted(paths.items())]
    assert chapters()==records
    with M.MANIFEST.open('x') as stream:
        json.dump(frozen,stream,indent=2)
    subprocess.run(['/usr/bin/python3','-I','-B',
        '/data/projects/jmev-stroke-probe-20261106/guard_csd02.py',
        '--phase','native','--job',M.NAME,'--inputs',str(M.MANIFEST),'--',
        '/usr/bin/python3','-I','-B',
        '/data/projects/jmev-stroke-probe-20261106/qualification01/final-master03.py','--run'],
        cwd=M.ROOT,check=True)
    M.S.success(M.NAME)

if __name__=='__main__':
    if sys.argv[1:]==['--launch']:
        launch()
    elif sys.argv[1:]==['--run']:
        M.run()
    else:
        raise SystemExit('Explicit --launch or guarded --run only')
