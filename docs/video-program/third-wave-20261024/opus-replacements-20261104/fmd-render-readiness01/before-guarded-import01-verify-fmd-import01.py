"""Verify imported FMD artifacts on CSD before native master replay.

Remote current-input validation is performed by export-fmd-final01.py on FMD.
This checker revalidates the transported terminal evidence and actual media,
not an assertion that the remote source filesystem was inspected from CSD.
"""
import copy
import importlib.util
import json
from pathlib import Path
import subprocess
import sys
import wave

HERE = Path('/data/projects/jmev-stroke-probe-20261106/qualification01')
ROOT = HERE.parent
IMPORT = ROOT/'imports/fmd-final01'
REMOTE = Path('/data/projects/jmev-fmd-render-qualification-20261106-1111')
spec = importlib.util.spec_from_file_location('launch', HERE/'launch-final02.py')
L = importlib.util.module_from_spec(spec)
spec.loader.exec_module(L)
S, F = L.S, L.F

def verify():
    review = F.G.check(F.REVIEW)
    assignment_path = HERE/'chapter-assignment01.json'
    assignment = S.read(assignment_path)
    chapters = [r['chapter'] for r in assignment['assignments'] if r['host']=='fmd']
    assert len(chapters) == 7
    assert (IMPORT/'chapter-assignment01.json').read_bytes() == assignment_path.read_bytes()
    assert (IMPORT/'distributed-review-permit01.json').read_bytes() == (HERE/'distributed-review-permit01.json').read_bytes()
    permit = S.read(IMPORT/'distributed-review-permit01.json')
    assert permit['review_gate_result'] == review
    export = S.read(IMPORT/'final-export01.json')
    assert export['host']=='fmd' and export['binary_sha256']==S.BINARY_SHA
    assert [r['chapter'] for r in export['records']]==chapters
    files = export['files']
    assert len(files)==len({r['destination'] for r in files})==72
    assert len({r['source'] for r in files})==72
    expected = {'chapter-assignment01.json', 'distributed-review-permit01.json'}
    for chapter in chapters:
        expected.update(chapter+'/'+name for name in
                        ('inputs.json','resource-receipt.json','render-summary.json','command.log','soundtrack.wav'))
        expected.update(chapter+'/render/'+chapter+s for s in
                        ('.mp4','.timeline.tsv','.cues.tsv','.cuelog.tsv','.layout.tsv'))
    assert {r['destination'] for r in files}==expected
    by_source = {}
    for row in files:
        relative = Path(row['destination'])
        assert not relative.is_absolute() and '..' not in relative.parts
        path = IMPORT/relative
        assert path.resolve().is_relative_to(IMPORT.resolve())
        assert path.stat().st_size==row['bytes'] and S.sha(path)==row['sha256']
        by_source[row['source']] = path
    accepted = []
    for chapter, record in zip(chapters, export['records']):
        directory = IMPORT/chapter
        remote_job = REMOTE/'jobs'/('fmd-final-'+chapter+'-01')
        receipt = S.read(directory/'resource-receipt.json')
        assert receipt.get('finished') and receipt['exit']==0 and receipt['policy']==S.POLICY
        assert not any(receipt.get(k) for k in ('error','resource_stop','stopped_for_resource_limit'))
        assert receipt['inputs_sha256']==receipt['final_inputs_sha256']==S.sha(directory/'inputs.json')
        assert receipt['command']==['/usr/bin/python3','-I','-B',str(REMOTE/'qualification01/final-chapter02.py'),'--chapter',chapter]
        assert S.read(directory/'render-summary.json')==record
        assert record['chapter']==chapter and record['host']=='fmd' and record['fps']==60
        assert record['binary_sha256']==S.BINARY_SHA and record['review']==review
        assert record['permit_sha256']==S.sha(HERE/'distributed-review-permit01.json')
        frames = next(r['frames'] for r in assignment['assignments'] if r['chapter']==chapter)
        assert record['frames']==frames
        expected_sources = {str(remote_job/'render'/(chapter+s)) for s in
                            ('.mp4','.timeline.tsv','.cues.tsv','.cuelog.tsv','.layout.tsv')}
        assert len(record['files'])==5 and {r['path'] for r in record['files']}==expected_sources
        local_record = copy.deepcopy(record)
        for item in local_record['files']:
            local = by_source[item['path']]
            assert S.sha(local)==item['sha256']
            item['path']=str(local)
        out = directory/'render'
        for suffix in ('timeline.tsv','cues.tsv'):
            assert (out/(chapter+'.'+suffix)).read_bytes()==(S.TIMELINES/(chapter+'.'+suffix)).read_bytes()
        F.P.check_cue_log(chapter,frames,out/(chapter+'.cuelog.tsv'),S.TIMELINES/(chapter+'.cuelog.tsv'))
        movie = out/(chapter+'.mp4')
        probe = json.loads(subprocess.check_output(['ffprobe','-v','error','-show_streams','-of','json',str(movie)],timeout=120))
        F.video_facts(probe,frames)
        assert F.encoder_settings(movie)==record['encoder']
        capture=record['soundtrack'];local=by_source[capture['path']]
        assert Path(capture['path']).parent==remote_job/'render/soundtrack'/chapter
        assert S.sha(local)==capture['sha256']
        assert (capture['samples'],capture['rate'],capture['channels'],capture['sample_width'])==(frames*800,48000,2,2)
        with wave.open(str(local)) as wav:
            assert (wav.getnframes(),wav.getframerate(),wav.getnchannels(),wav.getsampwidth())==(frames*800,48000,2,2)
            count=0
            while data:=wav.readframes(48000):count+=len(data)
            assert count==frames*800*4
        local_record['soundtrack']['path']=str(local)
        assert (directory/'command.log').read_text().splitlines()[-1]=='FMD FINAL CHAPTER technical checks passed: '+chapter
        local_record['origin_record_sha256']=S.sha(directory/'render-summary.json')
        accepted.append(local_record)
    assert F.G.check(F.REVIEW)==review
    return accepted

if __name__=='__main__':
    assert sys.argv[1:]==['--check']
    print(json.dumps(verify(),indent=2))
