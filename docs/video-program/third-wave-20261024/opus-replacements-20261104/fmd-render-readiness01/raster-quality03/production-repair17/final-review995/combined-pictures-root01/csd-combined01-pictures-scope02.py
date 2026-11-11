"""Extract three repaired previews, retaining all plan02 frames and old boundary samples. No viewing claim."""
import hashlib
import importlib.util
import json
from pathlib import Path
import re
import signal
from PIL import Image, ImageDraw

R = Path('/data/projects/jmev-stroke-probe-20261106')
Q = R/'qualification01/production-repair17'
JOB = R/'jobs/csd-native-combined-pictures01'
PREVIEW = R/'jobs/csd-native-combined-preview01'
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
    spec=importlib.util.spec_from_file_location('owned855',base)
    h=importlib.util.module_from_spec(spec);spec.loader.exec_module(h)
    h.own_group();h.STATE['job']=JOB;h.CRATE=R/'work/production-repo07'
    for sig in h.STOP_SIGNALS:signal.signal(sig,h.on_stop)
    guard=read(JOB/'resource-receipt.json')
    assert guard['admitted'] and guard['phase']=='native' and not guard['preflight_only']
    assert 'finished' not in guard and 'exit' not in guard
    assert guard['policy']==dict(admission_gib=10,stop_free_gib=8,phase_growth_gib=4,
        work_cap_gib=56,target_growth_cap_gib=20,sample_seconds=10)
    assert sorted(p.name for p in JOB.iterdir())==['command.log','resource-receipt.json']
    receipt=read(PREVIEW/'resource-receipt.json')
    assert receipt['admitted'] and receipt['finished'] and receipt['exit']==0
    assert receipt['inputs_sha256']==receipt['final_inputs_sha256']
    assert not any(receipt.get(k) for k in ('error','resource_stop','stopped_for_resource_limit'))
    comparison=read(PREVIEW/'comparison-combined01.json')['chapters']
    assert set(comparison)=={'01_the_bump','08_loop_sums','09_boundary'}
    plan=read(Q/'combined-pictures01/sampling-plan02.json')
    source=R/'work/native-combined01/workspace/explainer_corpus/complex_analysis_visual'
    for rel,digest in plan['sources']['staged'].items():assert sha(source/rel)==digest,rel
    results={}
    for chapter in sorted(comparison):
        extract(h,chapter,comparison[chapter]['frames'],results)
    with (JOB/'pictures-combined01.json').open('x') as f:json.dump(results,f,indent=2)

def extract(h,chapter,total,results):
    job=JOB/chapter
    job.mkdir()
    tables={}
    for ext in ('timeline.tsv','cues.tsv','cuelog.tsv'):
        p=PREVIEW/'preview'/(chapter+'.'+ext)
        tables[ext]=sha(p)
    assert (PREVIEW/'preview'/(chapter+'.timeline.tsv')).read_bytes()==(R/'jobs/csd-native-combined-timeline01/timeline-output'/(chapter+'.timeline.tsv')).read_bytes()
    movie=PREVIEW/'preview'/(chapter+'.mp4')
    layout=PREVIEW/'preview'/(chapter+'.layout.tsv')
    probe=json.loads(h.run_step(job,'probe',['/usr/bin/ffprobe','-v','error','-show_streams',
        '-of','json',movie],60,None))
    videos=[s for s in probe['streams'] if s['codec_type']=='video'];assert len(videos)==1
    v=videos[0]
    assert (v['width'],v['height'],v['avg_frame_rate'],int(v['nb_frames']))==(960,540,'60/1',total)
    h.run_step(job,'decode',['/usr/bin/ffmpeg','-nostdin','-v','error','-xerror','-threads','2',
        '-i',movie,'-f','null','-'],600,None)
    selected_file=Q/'combined-pictures01'/('frames-'+chapter+'-all.txt')
    oldframes=[int(x) for x in selected_file.read_text().splitlines()]
    assert oldframes==sorted(set(oldframes)) and min(oldframes)>=0 and max(oldframes)<total
    if chapter=='09_boundary':
        prior=Q/'csd-native855-prior775.json'
        assert sha(prior)=='2a1864771d82f9edb436823e100a3c6b97f8421e272a87bd7c2a8c95c1b4cf99'
        oldframes=sorted(set(oldframes)|set(read(prior)['selected_frames']))
    chosen=set(oldframes);anchors=[]
    if chapter=='08_loop_sums':
        chosen.update(range(10200,10380))
    for row in layout.read_text().splitlines():
        match=re.search(r'@([0-9.]+)s',row);assert match,row
        at=round(float(match[1])*60);assert 0<=at<total
        anchors.append(dict(frame=at,row=row))
        chosen.update(range(at,min(at+121,total),12))
        chosen.update(range(at,total,300))
    frames=sorted(chosen);assert set(oldframes)<=chosen and len(frames)<4000
    select=job/'select.filter'
    with select.open('x') as f:f.write("select='"+balanced([f'eq(n\\,{n})' for n in frames])+"'\n")
    pictures=job/'pictures';pictures.mkdir()
    h.run_step(job,'extract',['/usr/bin/ffmpeg','-nostdin','-n','-v','error','-xerror','-threads','2',
        '-i',movie,'-an','-filter_script:v',select,'-fps_mode','passthrough','-q:v','2',
        pictures/'frame-%06d.jpg'],600,None)
    files=sorted(pictures.glob('frame-*.jpg'));assert len(files)==len(frames)
    sheets=job/'sheets';sheets.mkdir();rows=[]
    for start in range(0,len(files),8):
        sheet=Image.new('RGB',(1280,1536),'#222222');draw=ImageDraw.Draw(sheet)
        for slot,p in enumerate(files[start:start+8]):
            index=start+slot;x,y=(slot%2)*640,(slot//2)*384
            with Image.open(p) as im:
                assert im.size==(960,540)
                sheet.paste(im.resize((640,360),Image.Resampling.LANCZOS),(x,y+24))
            draw.text((x+8,y+5),f'{index+1:03d} | frame {frames[index]} | {frames[index]/60:.3f}s',fill='white')
        p=sheets/f'sheet-{start//8+1:03d}.jpg';sheet.save(p,quality=95)
        rows.append(dict(path=str(p),sha256=sha(p),first_picture=start+1,last_picture=min(start+8,len(files))))
    result=dict(movie=str(movie),movie_sha256=sha(movie),frames=total,tables=tables,full_decode=True,
        selected_frames=frames,sheets=rows,layout_sha256=sha(layout),layout_anchors=anchors,
        retained_samples=len(oldframes),pictures=[dict(path=str(p),sha256=sha(p)) for p in files],
        sampling='All plan02 frames plus prior boundary775 frames, current lint anchors/persistence and complete08 required scope10200:10380. Preview only.',
        scope='Extraction only. No picture viewing, listening, fine-sync or independent AV acceptance.')
    with (job/'pictures.json').open('x') as f:json.dump(result,f,indent=2)
    results[chapter]=result

if __name__=='__main__':main()
