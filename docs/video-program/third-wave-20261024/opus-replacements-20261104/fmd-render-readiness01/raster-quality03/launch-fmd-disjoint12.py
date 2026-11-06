"""Explicit serial FMD driver for assigned disjoint chapters; never creates approval."""
import fcntl
import importlib.util
import json
from pathlib import Path
import shutil
import subprocess
import sys

ROOT = Path('/data/projects/jmev-fmd-render-qualification-20261106-1111')
HERE = ROOT/'qualification01/raster-quality03'
spec = importlib.util.spec_from_file_location('worker', HERE/'final-chapter12.py')
W = importlib.util.module_from_spec(spec)
spec.loader.exec_module(W)

def child(chapter):
    return ['/usr/bin/python3', '-I', '-B', str(HERE/'final-chapter12.py'), '--chapter', chapter]

def manifest(chapter):
    return ROOT/('fmd-raster-final-'+chapter+'-12-inputs.json')

def verify_inputs(path):
    rows = W.read(path)['files']
    assert len(rows) == len({r['path'] for r in rows})
    for row in rows:
        assert W.sha(row['path']) == row['sha256'], row['path']

def freeze():
    W.check_media_paths()
    paths = {}
    for phase in ('rebind12','encoder-stop11','timeline12','preview12','pictures12'):
        name = 'fmd-raster-'+phase
        base = ROOT/(name+'-inputs.json')
        receipt = W.success(name)
        assert W.sha(base) == receipt['inputs_sha256']
        verify_inputs(base)
        for row in W.read(base)['files']:
            if row['path'] in paths: assert paths[row['path']] == row['sha256']
            paths[row['path']] = row['sha256']
        paths[str(base)] = W.sha(base)
        for path in (ROOT/'jobs'/name).rglob('*'):
            if path.is_file(): paths[str(path)] = W.sha(path)
    extras = [Path(__file__).resolve(), W.PERMIT, W.ASSIGNMENT, ROOT/'guard_fmd01.py']
    extras.extend(HERE.glob('*.py'))
    extras.extend(HERE/n for n in ('preview-reviewed12.json','original-score-summary12.json','original-score-files12.json','original-score-receipt12.json','native-detail11-reviewed.json'))
    for directory in (W.NARRATION, W.TIMELINES, W.STEMS):
        extras.extend(p for p in directory.rglob('*') if p.is_file())
    for tool in ('python3','ffmpeg','ffprobe'):
        executable = shutil.which(tool); assert executable
        extras.append(Path(executable).resolve())
    for path in extras:
        digest = W.sha(path)
        if str(path) in paths: assert paths[str(path)] == digest
        paths[str(path)] = digest
    return {'files':[dict(path=p,sha256=h) for p,h in sorted(paths.items())]}

def completed(chapter):
    row, review, permit_sha = W.check(chapter)
    name = 'fmd-raster-final-'+chapter+'-12'
    job = ROOT/'jobs'/name
    receipt = W.success(name)
    assert receipt['command'] == child(chapter)
    assert receipt['inputs_sha256'] == W.sha(manifest(chapter))
    verify_inputs(manifest(chapter))
    result = W.read(job/'render-summary.json')
    assert result['chapter'] == chapter and result['host'] == 'fmd'
    assert result['binary_sha256'] == W.BINARY_SHA and result['permit_sha256'] == permit_sha
    assert result['review'] == review and result['frames'] == row['frames'] and result['fps'] == 60
    expected = {str(job/'render'/(chapter+s)) for s in
                ('.mp4','.timeline.tsv','.cues.tsv','.cuelog.tsv','.layout.tsv')}
    assert len(result['files']) == 5 and {r['path'] for r in result['files']} == expected
    for item in result['files']:
        assert W.sha(item['path']) == item['sha256']
    capture = result['soundtrack']
    assert Path(capture['path']).parent == job/'render/soundtrack'/chapter
    assert W.sha(capture['path']) == capture['sha256']
    assert (capture['samples'],capture['rate'],capture['channels'],capture['sample_width']) == (row['frames']*800,48000,2,2)
    W.verify_tables(chapter, job/'render', row['frames'])
    assert (job/'command.log').read_text().splitlines()[-1] == 'FMD FINAL CHAPTER technical checks passed: '+chapter
    return result

def launch():
    assert Path(__file__).resolve().parent == HERE
    with (HERE/'final-driver12.lock').open('a') as lock:
        fcntl.flock(lock, fcntl.LOCK_EX | fcntl.LOCK_NB)
        assignment = W.read(W.ASSIGNMENT)
        rows = [r for r in assignment['assignments'] if r['host']=='fmd']
        assert rows and len(rows) < 15
        assert len({r['chapter'] for r in assignment['assignments']}) == 15
        for row in rows:
            W.check(row['chapter'])
        frozen = freeze()
        results = []
        for row in rows:
            chapter = row['chapter']
            name = 'fmd-raster-final-'+chapter+'-12'
            if (ROOT/'jobs'/name).exists():
                results.append(completed(chapter))
                continue
            path = manifest(chapter)
            if path.exists():
                assert W.read(path) == frozen
            else:
                with path.open('x') as stream:
                    json.dump(frozen, stream, indent=2)
            W.check(chapter)
            verify_inputs(path)
            subprocess.run(['/usr/bin/python3','-I','-B',
                '/data/projects/jmev-fmd-render-qualification-20261106-1111/guard_fmd01.py',
                '--phase','native','--job',name,'--inputs',str(path),'--',
                '/usr/bin/python3','-I','-B',
                '/data/projects/jmev-fmd-render-qualification-20261106-1111/qualification01/raster-quality03/final-chapter12.py',
                '--chapter',chapter], cwd=ROOT, check=True)
            results.append(completed(chapter))
        result = dict(chapters=results, host='fmd', assignment_sha256=W.sha(W.ASSIGNMENT),
                      scope='Assigned technical chapters; verified import, native master replay and final review remain.')
        path = HERE/'final-fmd-assigned12.json'
        if path.exists():
            assert W.read(path) == result
        else:
            with path.open('x') as stream:
                json.dump(result, stream, indent=2)

if __name__ == '__main__':
    assert sys.argv[1:] == ['--run']
    launch()
