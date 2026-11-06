"""Validate one received FMD chapter for native CSD master assembly."""
import copy,importlib.util,json,subprocess,wave,struct
from pathlib import Path
HERE=Path('/data/projects/jmev-stroke-probe-20261106/qualification01/raster-quality12')
spec=importlib.util.spec_from_file_location('receiver',HERE/'receive-fmd-chapter12.py')
RX=importlib.util.module_from_spec(spec);spec.loader.exec_module(RX)
W=RX.load_worker();ROOT=W.ROOT

def verify(chapter):
    W.check_media_paths();W.check_current_preview()
    assignment=W.read(W.ASSIGNMENT);permit=W.read(W.PERMIT)
    csd=[r['chapter'] for r in assignment['assignments'] if r['host']=='csd'];assert csd
    W.check(csd[0])
    selected=[r for r in assignment['assignments'] if r['chapter']==chapter and r['host']=='fmd'];assert len(selected)==1
    frames=selected[0]['frames']
    name='csd-fmd-final-import-'+chapter+'-12';job=ROOT/'jobs'/name;folder=job/'import'
    receipt=W.success(name);manifest=ROOT/(name+'-inputs.json')
    assert receipt['inputs_sha256']==W.sha(manifest)
    for row in W.read(manifest)['files']:assert W.sha(row['path'])==row['sha256']
    assert receipt['command']==['/usr/bin/python3','-I','-B',str(HERE/'receive-fmd-chapter12.py'),'--chapter',chapter]
    export=W.read(folder/'final-export12.json');assert export['host']=='fmd' and export['binary_sha256']==W.BINARY_SHA
    assert (folder/W.ASSIGNMENT.name).read_bytes()==W.ASSIGNMENT.read_bytes()
    assert (folder/W.PERMIT.name).read_bytes()==W.PERMIT.read_bytes()
    rows=RX.selected_rows(export,chapter);by_source={}
    for row in rows:
        p=folder/row['destination'];assert p.stat().st_size==row['bytes'] and W.sha(p)==row['sha256'];by_source[row['source']]=p
    directory=folder/chapter;remote_job=RX.REMOTE/'jobs'/('fmd-raster-final-'+chapter+'-12')
    remote_receipt=W.read(directory/'resource-receipt.json')
    assert remote_receipt.get('finished') and remote_receipt['exit']==0 and remote_receipt['policy']==W.POLICY
    assert not any(remote_receipt.get(k) for k in ('error','resource_stop','stopped_for_resource_limit'))
    assert remote_receipt['inputs_sha256']==remote_receipt['final_inputs_sha256']==W.sha(directory/'inputs.json')
    assert remote_receipt['command']==['/usr/bin/python3','-I','-B',str(RX.REMOTE/'qualification01/raster-quality03/final-chapter12.py'),'--chapter',chapter]
    record=W.read(directory/'render-summary.json')
    assert record==next(r for r in export['records'] if r['chapter']==chapter)
    assert record['host']=='fmd' and record['frames']==frames and record['fps']==60
    assert record['binary_sha256']==W.BINARY_SHA and record['permit_sha256']==W.sha(W.PERMIT)
    assert record['review']==permit['review_gate_result']
    expected={str(remote_job/'render'/(chapter+s)) for s in ('.mp4','.timeline.tsv','.cues.tsv','.cuelog.tsv','.layout.tsv')}
    assert len(record['files'])==5 and {r['path'] for r in record['files']}==expected
    local=copy.deepcopy(record)
    for item in local['files']:
        p=by_source[item['path']];assert W.sha(p)==item['sha256'];item['path']=str(p)
    out=directory/'render'
    for suffix in ('timeline.tsv','cues.tsv'):
        assert (out/(chapter+'.'+suffix)).read_bytes()==(W.TIMELINES/(chapter+'.'+suffix)).read_bytes()
    actual=(out/(chapter+'.cuelog.tsv')).read_text().splitlines()
    base=(W.TIMELINES/(chapter+'.cuelog.tsv')).read_text().splitlines()
    assert actual[:len(base)]==base
    stem=W.STEMS/(chapter+'.music.f32.wav');parts=stem.with_suffix('.parts.tsv')
    files=[(stem,0,frames*800)]
    if parts.exists():files=[(stem.with_name(r[4]),int(r[0]),int(r[1])) for r in [s.split('\t') for s in parts.read_text().splitlines()[1:]]]
    music=[r.split('\t') for r in actual[len(base):]];assert len(music)==len(files)
    end=0
    for row,(path,start,samples) in zip(music,files):
        offset=float(row[5]);bits=struct.unpack('>Q',struct.pack('>d',offset))[0]
        assert row[:5]==[str(path),W.sha(path),str(path.stat().st_size),str(frames),'60']
        assert row[6:]==[f'{bits:#018x}','0.0','0x0000000000000000','48000','2',str(samples),str(start)]
        assert frames*800+round(offset*48000)==start==end;end+=samples
    assert end==frames*800
    # Every native replay source must exist here with the exact captured hash.
    for line in actual:
        cols=line.split('\t')
        if len(cols)>=3 and cols[0].startswith('/'):
            assert W.sha(Path(cols[0]))==cols[1] and Path(cols[0]).stat().st_size==int(cols[2])
    movie=out/(chapter+'.mp4')
    probe=json.loads(subprocess.check_output(['ffprobe','-v','error','-show_streams','-of','json',str(movie)],timeout=120))
    videos=[s for s in probe['streams'] if s['codec_type']=='video'];audios=[s for s in probe['streams'] if s['codec_type']=='audio'];assert len(videos)==len(audios)==1
    v,a=videos[0],audios[0]
    assert (v['width'],v['height'],v['avg_frame_rate'],int(v['nb_frames']),v['codec_name'],v['pix_fmt'])==(3840,2160,'60/1',frames,'h264','yuv420p')
    assert (a['codec_name'],a['sample_rate'],a['channels'])==('aac','48000',2)
    with movie.open('rb') as f:head=f.read(16*1024*1024)
    pos=head.find(b'x264 - core');assert pos>=0
    settings=head[pos:pos+5000].split(b'\0',1)[0].decode('utf-8',errors='replace');assert settings==record['encoder'] and ' crf=16.0' in settings
    capture=record['soundtrack'];p=by_source[capture['path']]
    assert Path(capture['path']).parent==remote_job/'render/soundtrack'/chapter
    assert W.sha(p)==capture['sha256']
    assert (capture['samples'],capture['rate'],capture['channels'],capture['sample_width'])==(frames*800,48000,2,2)
    with wave.open(str(p)) as wav:
        assert (wav.getnframes(),wav.getframerate(),wav.getnchannels(),wav.getsampwidth())==(frames*800,48000,2,2)
        count=0
        while data:=wav.readframes(48000):count+=len(data)
        assert count==frames*800*4
    assert (directory/'command.log').read_text().splitlines()[-1]=='FMD FINAL CHAPTER technical checks passed: '+chapter
    local['soundtrack']['path']=str(p);local['origin_record_sha256']=W.sha(directory/'render-summary.json')
    return local

if __name__=='__main__':
    import argparse
    p=argparse.ArgumentParser();p.add_argument('--chapter',required=True);a=p.parse_args();print(json.dumps(verify(a.chapter),indent=2))
