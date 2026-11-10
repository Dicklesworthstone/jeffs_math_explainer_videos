"""Extract Linux968 final4K pictures; preserve allprior775/855samples and new lint."""
import hashlib
import base64
import importlib.util
import json
from pathlib import Path
import re
import signal
from PIL import Image, ImageDraw

R = Path('/data/projects/jmev-stroke-probe-20261106')
Q = R/'qualification01/production-repair17'
JOB = R/'jobs/csd-native-pictures969'
FINAL = R/'jobs/csd-native-final968'
PREVIEW = R/'jobs/csd-native-preview853'
NARRATION = str(R/'jobs/csd-native-delivery849/narration')
OLD = str(R/'jobs/csd-review-repair-rebind313/narration')
EXPECTED = {'cues.tsv':'eed5a50ad5797fa258ee3fcf9319e29946f403a86b3909090e73a3a15dc74249',
    'timeline.tsv':'5a91b8f9e2e5b5d6223a49285c06039b1b43bd16c11cecdd163a8e2e545baa9e',
    'cuelog.tsv':'7ed46c80cede7e49daeaf729337b24c66c8e7ccc125bb18ca6ce2c411d201bbf'}

def sha(p):
    with p.open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest()
def read(p):return json.loads(p.read_text())
def balanced(terms):
    assert terms
    if len(terms)==1:return terms[0]
    middle=len(terms)//2
    return '('+balanced(terms[:middle])+'+'+balanced(terms[middle:])+')'

def main():
    base=Q/'csd-native844-base761.py'
    assert sha(base)=='32bf6d9e08c1f21f2445debfd46ba090e0374007e4da5516310cb122774099e5'
    spec=importlib.util.spec_from_file_location('owned969',base)
    h=importlib.util.module_from_spec(spec);spec.loader.exec_module(h)
    h.own_group();h.STATE['job']=JOB;h.CRATE=R/'work/production-repo07'
    for sig in h.STOP_SIGNALS:signal.signal(sig,h.on_stop)
    guard=read(JOB/'resource-receipt.json')
    assert guard['admitted'] and guard['phase']=='native' and not guard['preflight_only']
    assert 'finished' not in guard and 'exit' not in guard
    assert guard['policy']==dict(admission_gib=10,stop_free_gib=8,phase_growth_gib=4,
        work_cap_gib=56,target_growth_cap_gib=20,sample_seconds=10)
    assert sorted(p.name for p in JOB.iterdir())==['command.log','resource-receipt.json']
    receipt=read(FINAL/'resource-receipt.json')
    assert receipt['admitted'] and receipt['finished'] and receipt['exit']==0
    assert receipt['inputs_sha256']==receipt['final_inputs_sha256']
    assert not any(receipt.get(k) for k in ('error','resource_stop','stopped_for_resource_limit'))
    assert read(PREVIEW/'comparison853.json')['frames']==16504
    tables={}
    for ext in ('timeline.tsv','cues.tsv'):
        p=FINAL/'render'/('09_boundary.'+ext);raw=p.read_bytes()
        assert raw==(PREVIEW/'preview'/p.name).read_bytes()
        assert hashlib.sha256(raw.replace(NARRATION.encode(),OLD.encode())).hexdigest()==EXPECTED[ext]
        tables[ext]=sha(p)
    reference=Q/'csd-native875-reference586.json'
    assert sha(reference)=='8730f969500431cfacba7befe188541c20038b5e869b0dd62415cddfabd08c0c'
    ref=read(reference);original=base64.b64decode(ref['cuelog_base64'],validate=True)
    assert hashlib.sha256(original).hexdigest()==ref['sha256']=='b4d44b08ff51451cf5834c1d65e77f7e959c721b39ecec5b163ecb69dee88a04'
    assert ref['receipt']['finished'] and ref['receipt']['exit']==0
    assert ref['receipt']['inputs_sha256']==ref['receipt']['final_inputs_sha256']
    assert not any(ref['receipt'].get(k) for k in ('error','resource_stop','stopped_for_resource_limit'))
    expected=original.replace(b'/Users/jemanuel/scratch/jmev-complex-native-20261109/jobs/mini-boundary-delivery557/narration',NARRATION.encode())
    expected=expected.replace(b'/Users/jemanuel/scratch/jmev-complex-native-20261109/score586',str(R/'jobs/csd-production-score21/score/stems').encode())
    cuelog=FINAL/'render/09_boundary.cuelog.tsv'
    assert cuelog.read_bytes()==expected, 'Full scored cuelog differs beyond the two named path substitutions'
    tables['cuelog.tsv']=sha(cuelog)
    movie=FINAL/'render/09_boundary.mp4'
    layout=FINAL/'render/09_boundary.layout.tsv'
    probe=json.loads(h.run_step(JOB,'probe',['/usr/bin/ffprobe','-v','error','-show_streams',
        '-of','json',movie],60,None))
    videos=[s for s in probe['streams'] if s['codec_type']=='video'];assert len(videos)==1
    v=videos[0]
    assert (v['width'],v['height'],v['avg_frame_rate'],int(v['nb_frames']))==(3840,2160,'60/1',16504)
    h.run_step(JOB,'decode',['/usr/bin/ffmpeg','-nostdin','-v','error','-xerror','-threads','2',
        '-i',movie,'-f','null','-'],3600,None)
    prior=Q/'csd-native855-prior775.json'
    assert sha(prior)=='2a1968771d82f9edb436823e100a3c6b97f8421e272a87bd7c2a8c95c1b4cf99'
    oldframes=read(prior)['selected_frames']
    assert len(oldframes)==len(set(oldframes))==814 and oldframes==sorted(oldframes)
    assert min(oldframes)>=0 and max(oldframes)<16504
    preview_record=read(R/'jobs/csd-native-pictures855/pictures855.json')
    assert preview_record['movie_sha256']==sha(PREVIEW/'preview/09_boundary.mp4')
    assert preview_record['frames']==16504 and preview_record['full_decode'] is True
    preview_frames=preview_record['selected_frames']
    assert preview_frames==sorted(set(preview_frames)) and set(oldframes)<=set(preview_frames)
    chosen=set(preview_frames);anchors=[]
    for row in layout.read_text().splitlines():
        match=re.search(r'@([0-9.]+)s',row);assert match,row
        at=round(float(match[1])*60);assert 0<=at<16504
        anchors.append(dict(frame=at,row=row))
        chosen.update(range(at,min(at+121,16504),12))
        chosen.update(range(at,16504,300))
    frames=sorted(chosen);assert set(oldframes)<=chosen and len(frames)<1500
    select=JOB/'select969.filter'
    with select.open('x') as f:f.write("select='"+balanced([f'eq(n\\,{n})' for n in frames])+"'\n")
    pictures=JOB/'pictures';pictures.mkdir()
    h.run_step(JOB,'extract',['/usr/bin/ffmpeg','-nostdin','-n','-v','error','-xerror','-threads','2',
        '-i',movie,'-an','-filter_script:v',select,'-fps_mode','passthrough','-q:v','2',
        pictures/'frame-%06d.jpg'],3600,None)
    files=sorted(pictures.glob('frame-*.jpg'));assert len(files)==len(frames)
    sheets=JOB/'sheets';sheets.mkdir();rows=[]
    for start in range(0,len(files),8):
        sheet=Image.new('RGB',(1280,1536),'#222222');draw=ImageDraw.Draw(sheet)
        for slot,p in enumerate(files[start:start+8]):
            index=start+slot;x,y=(slot%2)*640,(slot//2)*384
            with Image.open(p) as im:
                assert im.size==(3840,2160)
                sheet.paste(im.resize((640,360),Image.Resampling.LANCZOS),(x,y+24))
            draw.text((x+8,y+5),f'{index+1:03d} | frame {frames[index]} | {frames[index]/60:.3f}s',fill='white')
        p=sheets/f'sheet-{start//8+1:03d}.jpg';sheet.save(p,quality=95)
        rows.append(dict(path=str(p),sha256=sha(p),first_picture=start+1,last_picture=min(start+8,len(files))))
    result=dict(movie=str(movie),movie_sha256=sha(movie),frames=16504,tables=tables,full_decode=True,
        selected_frames=frames,sheets=rows,layout_sha256=sha(layout),layout_anchors=anchors,
        retained_samples=814,retained_preview_samples=len(preview_frames),resolution=[3840,2160],pictures=[dict(path=str(p),sha256=sha(p)) for p in files],
        sampling='All814 prior775 and all855 preview samples plus current968 final lint anchors and persistence.',
        scope='Extraction only. No picture viewing, listening, fine-sync or independent AV acceptance.')
    with (JOB/'pictures969.json').open('x') as f:json.dump(result,f,indent=2)
    print('969 extracted',len(files),'pictures on',len(rows),'sheets; not viewed',flush=True)

if __name__=='__main__':main()
