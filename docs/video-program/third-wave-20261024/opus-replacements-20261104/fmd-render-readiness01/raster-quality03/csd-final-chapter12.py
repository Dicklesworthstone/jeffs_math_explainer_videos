"""CSD guarded raster chapter worker. Root must validate the FMD review before dispatch.

The permit carries that exact review and extraction record; this worker validates
their binding, but does not claim to have viewed the images itself.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import struct
import subprocess
import wave

ROOT = Path('/data/projects/jmev-stroke-probe-20261106')
CANONICAL = Path('/data/projects/jmev-fmd-render-qualification-20261106-1111')
INPUTS = ROOT/'production-inputs01'
TIMELINES = ROOT/'production-inputs-raster12/timeline-output'
NARRATION = ROOT/'jobs/csd-raster-media15/narration'
FMD_NARRATION = CANONICAL/'jobs/fmd-raster-rebind12/narration'
STEMS = CANONICAL/'production-inputs01/scored/stems'
BIN = ROOT/'work/bin/production09i'
BINARY_SHA = 'd3d6273deb37497f4de2c7004534a2207d91641747e0feb6651509676cd10e2f'
POLICY = dict(admission_gib=10, stop_free_gib=8, phase_growth_gib=4,
              work_cap_gib=56, target_growth_cap_gib=20, sample_seconds=10)
PERMIT = ROOT/'qualification01/raster-quality12/distributed-review-permit12.json'
ASSIGNMENT = ROOT/'qualification01/raster-quality12/chapter-assignment12.json'

def sha(path):
    with Path(path).open('rb') as f:
        return hashlib.file_digest(f, 'sha256').hexdigest()

def read(path):
    return json.loads(Path(path).read_text())

def success(name):
    r = read(ROOT/'jobs'/name/'resource-receipt.json')
    assert r.get('finished') and r.get('exit') == 0 and r['policy'] == POLICY
    assert not any(r.get(k) for k in ('error','resource_stop','stopped_for_resource_limit'))
    assert r['inputs_sha256'] == r['final_inputs_sha256']
    return r

def validate_permit(permit, assignment, chapter):
    rows = assignment['assignments']
    keys = [r['chapter'] for r in rows]
    assert len(keys) == len(set(keys)) == 15
    assert assignment['launch_authorized'] is False
    selected = [r for r in rows if r['chapter'] == chapter]
    assert len(selected) == 1 and selected[0]['host'] == 'csd'
    assert {r['host'] for r in rows} == {'csd','fmd'}
    assert permit['origin_host'] == 'fmd'
    assert permit['binary_sha256'] == BINARY_SHA
    assert permit['chapters'] == keys
    review, extraction, result = permit['review_record'], permit['extraction_record'], permit['review_gate_result']
    for name in ('review', 'extraction'):
        raw = permit[name + '_json'].encode('utf-8')
        assert hashlib.sha256(raw).hexdigest() == permit[name + '_sha256']
        assert json.loads(raw) == permit[name + '_record']
    assert review['pictures_viewed'] is True and review['unresolved_visual_defects'] == []
    assert review['reviewer'].strip() and review['chapters'] == keys
    assert review['review_scope'] in ('producer selected-frame review','independent selected-frame review')
    for key in ('listening_accepted','fine_sync_accepted','full_independent_av_accepted'):
        assert review[key] is False
    sheets = extraction['sheets']
    assert sheets and extraction['samples']
    assert len({r['path'] for r in sheets}) == len(sheets)
    assert {r['chapter'] for r in sheets} == set(keys)
    assert review['viewed_sheets'] == [{'path':r['path'],'sha256':r['sha256']} for r in sheets]
    assert result['chapters'] == 15 and result['viewed_sheets'] == len(sheets)
    assert result['selected_pictures'] == len(extraction['samples'])
    assert result['extraction_sha256'] == review['extraction_sha256'] == permit['extraction_sha256']
    assert result['review_sha256'] == permit['review_sha256']
    assert result['preview_summary_sha256'] == review['preview_summary_sha256']
    assert extraction['preview_summary_sha256'] == result['preview_summary_sha256']
    assert result['binary_sha256'] == BINARY_SHA
    return selected[0], result

def check_media_paths():
    assert CANONICAL.resolve(strict=True) == ROOT
    assert NARRATION.resolve(strict=True) == NARRATION
    assert STEMS.resolve(strict=True) == ROOT/'jobs/stage07-score01/scored/stems'
    success('csd-raster-media15')
    media=read(ROOT/'jobs/csd-raster-media15/media-summary.json')
    assert media['wav_files_byte_identical']==105 and media['phrase_timings_byte_identical'] is True
    assert media['delivery']['verified_cues']==media['delivery']['selected_cues']==105

def check_current_preview():
    # These are exact imported completed artifacts, not CSD-local review claims.
    folder=ROOT/'production-inputs-raster12'
    transfer=read(folder/'transfer.json')
    assert transfer['origin_host']=='fmd' and transfer['binary_sha256']==BINARY_SHA
    paths={r['relative_path'] for r in transfer['files']}
    required={'preview-summary.json','extraction.json','comparison.json','preview-resource-receipt.json','pictures-resource-receipt.json','timeline-resource-receipt.json'}
    assert required<=paths
    for row in transfer['files']:
        rel=Path(row['relative_path']);assert not rel.is_absolute() and '..' not in rel.parts
        assert sha(folder/rel)==row['sha256']
    for phase in ('preview','pictures','timeline'):
        r=read(folder/(phase+'-resource-receipt.json'))
        assert r.get('finished') and r['exit']==0 and r['policy']==POLICY
        assert r['inputs_sha256']==r['final_inputs_sha256'] and not r.get('resource_stop')
    comparison=read(folder/'comparison.json');assert comparison['table_count']==len(comparison['tables'])==45
    keys=read(PERMIT)['chapters']
    assert {(r['chapter'],r['table']) for r in comparison['tables']}=={(c,k) for c in keys for k in ('timeline','cues','cuelog')}
    for row in comparison['tables']:
        assert sha(TIMELINES/(row['chapter']+'.'+row['table']+'.tsv'))==row['new_sha256']
    summary=read(folder/'preview-summary.json');assert summary['binary_sha256']==BINARY_SHA
    assert [r['chapter'] for r in summary['chapters']]==keys
    permit=read(PERMIT)
    assert permit['extraction_sha256']==sha(folder/'extraction.json')
    assert permit['review_record']['preview_summary_sha256']==sha(folder/'preview-summary.json')

def check(chapter):
    check_media_paths()
    check_current_preview()
    assignment, permit = read(ASSIGNMENT), read(PERMIT)
    assert permit['assignment_sha256'] == sha(ASSIGNMENT)
    assert sha(BIN) == BINARY_SHA
    row, review = validate_permit(permit, assignment, chapter)
    success('csd-raster-media15')
    success('csd-raster-encoder-stop13')
    assert sha(TIMELINES/(chapter+'.timeline.tsv')) == row['timeline_sha256']
    return row, review, sha(PERMIT)

def verify_tables(chapter, out, frames):
    # Timeline line rows also contain absolute narration paths. Only the exact
    # qualified narration directory changes; every other byte must match.
    expected = (TIMELINES/(chapter+'.timeline.tsv')).read_bytes().replace(
        str(FMD_NARRATION).encode(), str(NARRATION).encode())
    assert (out/(chapter+'.timeline.tsv')).read_bytes() == expected
    assert (out/(chapter+'.cues.tsv')).read_bytes() == (TIMELINES/(chapter+'.cues.tsv')).read_bytes()
    base = (TIMELINES/(chapter+'.cuelog.tsv')).read_text().replace(str(FMD_NARRATION),str(NARRATION)).splitlines()
    actual = (out/(chapter+'.cuelog.tsv')).read_text().splitlines()
    assert actual[:len(base)] == base
    stem = STEMS/(chapter+'.music.f32.wav')
    parts = stem.with_suffix('.parts.tsv')
    files = [(stem,0,frames*800)]
    if parts.exists():
        files = [(stem.with_name(r[4]),int(r[0]),int(r[1]))
                 for r in [s.split('\t') for s in parts.read_text().splitlines()[1:]]]
    rows = [r.split('\t') for r in actual[len(base):]]
    assert len(rows) == len(files)
    end = 0
    for row,(path,start,samples) in zip(rows,files):
        offset = float(row[5])
        bits = struct.unpack('>Q',struct.pack('>d',offset))[0]
        assert row[:5] == [str(path),sha(path),str(path.stat().st_size),str(frames),'60']
        assert row[6:] == [f'{bits:#018x}','0.0','0x0000000000000000','48000','2',str(samples),str(start)]
        assert frames*800+round(offset*48000) == start == end
        end += samples
    assert end == frames*800

def run(chapter):
    row, review, permit_sha = check(chapter)
    os.sched_setaffinity(0,{0,1,2,3})
    assert os.sched_getaffinity(0)=={0,1,2,3}
    job = ROOT/'jobs'/('csd-raster-final-'+chapter+'-12')
    receipt = read(job/'resource-receipt.json')
    assert receipt['admitted'] and 'exit' not in receipt and receipt['policy'] == POLICY
    assert receipt['command'] == ['/usr/bin/python3','-I','-B',str(Path(__file__).resolve()),'--chapter',chapter]
    out = job/'render'
    assert not out.exists()
    os.environ.update(TMPDIR=str(ROOT/'work/tmp'),XDG_CACHE_HOME=str(ROOT/'work/cache'))
    command = [str(BIN),'render',chapter,'--res','3840x2160','--fps','60',
               '--narration',str(NARRATION),'--score',str(STEMS),'--out',str(out),
               '--lint','--crf','16','--preset','veryfast']
    with (job/'native.stdout').open('xb') as stdout,(job/'native.stderr').open('xb') as stderr:
        subprocess.run(command,cwd=INPUTS/'production-repo07',stdout=stdout,stderr=stderr,check=True,timeout=43200)
    frames = row['frames']
    verify_tables(chapter,out,frames)
    movie = out/(chapter+'.mp4')
    assert set(out.glob('*.mp4')) == {movie}
    probe = json.loads(subprocess.check_output(['ffprobe','-v','error','-show_streams','-of','json',str(movie)],timeout=120))
    videos = [s for s in probe['streams'] if s['codec_type']=='video']
    audios = [s for s in probe['streams'] if s['codec_type']=='audio']
    assert len(videos)==len(audios)==1
    v,a = videos[0],audios[0]
    assert (v['width'],v['height'],v['avg_frame_rate'],int(v['nb_frames'])) == (3840,2160,'60/1',frames)
    assert (v['codec_name'],v['pix_fmt'],a['codec_name'],a['sample_rate'],a['channels']) == ('h264','yuv420p','aac','48000',2)
    with movie.open('rb') as f: head=f.read(16*1024*1024)
    pos=head.find(b'x264 - core');assert pos>=0
    settings=head[pos:pos+5000].split(b'\0',1)[0].decode('utf-8',errors='replace')
    assert ' rc=crf' in settings and ' crf=16.0' in settings
    captures=sorted((out/'soundtrack'/chapter).glob('soundtrack_*.wav'),key=lambda p:int(p.stem.split('_')[-1]))
    assert captures
    indices=[int(p.stem.split('_')[-1]) for p in captures]
    assert indices==list(range(indices[0],indices[-1]+1))
    capture=captures[-1]
    with wave.open(str(capture)) as w:
        assert (w.getframerate(),w.getnchannels(),w.getsampwidth(),w.getnframes())==(48000,2,2,frames*800)
        total=0
        while data:=w.readframes(48000):total+=len(data)
        assert total==frames*800*4
    assert check(chapter)==(row,review,permit_sha)
    result=dict(chapter=chapter,host='csd',frames=frames,fps=60,binary_sha256=BINARY_SHA,
        review=review,permit_sha256=permit_sha,encoder=settings,
        files=[dict(path=str(out/(chapter+s)),sha256=sha(out/(chapter+s)))
               for s in ('.mp4','.timeline.tsv','.cues.tsv','.cuelog.tsv','.layout.tsv')],
        soundtrack=dict(path=str(capture),sha256=sha(capture),samples=frames*800,rate=48000,channels=2,sample_width=2),
        scope='Technical chapter only; import verification, bit-identical master replay and full final AV/picture checks remain.')
    with (job/'render-summary.json').open('x') as f:json.dump(result,f,indent=2)
    print('CSD FINAL CHAPTER technical checks passed: '+chapter)

if __name__=='__main__':
    parser=argparse.ArgumentParser();parser.add_argument('--chapter',required=True);parser.add_argument('--check',action='store_true');args=parser.parse_args()
    if args.check:print(check(args.chapter))
    else:run(args.chapter)
