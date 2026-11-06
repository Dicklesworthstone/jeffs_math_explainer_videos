"""Explicit launch only after every assigned final chapter is qualified."""
import fcntl
import importlib.util
import json
from pathlib import Path
import subprocess
import sys

HERE = Path('/data/projects/jmev-stroke-probe-20261106/qualification01/raster-quality12')
spec = importlib.util.spec_from_file_location('master', HERE/'final-master12.py')
M = importlib.util.module_from_spec(spec)
spec.loader.exec_module(M)
W, L, ROOT = M.W, M.L, M.ROOT
NAME = 'csd-raster-final-master12'
MANIFEST = ROOT/(NAME+'-inputs.json')

def freeze(records):
    paths = {r['path']:r['sha256'] for r in L.freeze()['files']}
    def add(path):
        path = Path(path)
        digest = W.sha(path)
        if str(path) in paths:
            assert paths[str(path)] == digest
        paths[str(path)] = digest
    add(Path(__file__).resolve())
    for row, record in zip(W.read(W.ASSIGNMENT)['assignments'], records):
        chapter = row['chapter']
        name = ('csd-raster-final-'+chapter+'-12') if row['host']=='csd' else ('csd-fmd-final-import-'+chapter+'-12')
        manifest = ROOT/(name+'-inputs.json')
        receipt = W.success(name)
        assert W.sha(manifest) == receipt['inputs_sha256']
        L.verify_inputs(manifest)
        add(manifest)
        for item in W.read(manifest)['files']:
            add(item['path'])
        for path in (ROOT/'jobs'/name).rglob('*'):
            if path.is_file():
                add(path)
        for item in record['files']:
            path = Path(item['path'])
            add(path)
            if path.name.endswith('.cuelog.tsv'):
                for line in path.read_text().splitlines():
                    fields = line.split('\t')
                    if len(fields)>=3 and fields[0].startswith('/'):
                        source = Path(fields[0])
                        assert W.sha(source)==fields[1] and source.stat().st_size==int(fields[2])
                        add(source)
        add(record['soundtrack']['path'])
    return {'files':[dict(path=p,sha256=h) for p,h in sorted(paths.items())]}

def launch():
    assert sys.argv[1:]==['--run']
    assert Path(__file__).resolve().parent==HERE
    with (HERE/'master-driver12.lock').open('a') as lock:
        fcntl.flock(lock,fcntl.LOCK_EX|fcntl.LOCK_NB)
        assert not M.JOB.exists() and not MANIFEST.exists(), 'Preserve all attempts; never duplicate'
        records=M.chapters()
        frozen=freeze(records)
        assert M.chapters()==records
        with MANIFEST.open('x') as f:
            json.dump(frozen,f,indent=2)
        L.verify_inputs(MANIFEST)
        subprocess.run(['/usr/bin/python3','-I','-B',str(ROOT/'guard_csd02.py'),'--phase','native','--job',NAME,'--inputs',str(MANIFEST),'--','/usr/bin/python3','-I','-B',str(HERE/'final-master12.py'),'--run'],cwd=ROOT,check=True)
        W.success(NAME)

if __name__=='__main__':
    launch()
