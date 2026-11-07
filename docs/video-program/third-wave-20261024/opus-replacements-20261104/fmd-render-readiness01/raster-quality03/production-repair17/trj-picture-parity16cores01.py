"""Compare every decoded frame of the existing 4K diagnostic on TRJ and CSD."""
import hashlib
import json
import os
from pathlib import Path
import subprocess
import time

ROOT = Path('/data/projects/jmev-render-worker-20261107')
JOB = ROOT/'jobs/trj-picture-parity16cores01'
BIN = ROOT/'work/bin/production16'
PIN = '8dc5e8c4f640b73d9e4e6c4575cb492ebcc0deb0c2de0a658bbccf40d46082d1'
CHAPTER = '04_reading_color'
SOURCE = 'csd:/data/projects/jmev-stroke-probe-20261106/jobs/csd-continuous-detail16-04_reading_color/'

def sha(p):
    with p.open('rb') as f:
        return hashlib.file_digest(f,'sha256').hexdigest()

assert sha(BIN)==PIN
stopdir=ROOT/'jobs/trj-encoder-stop02'
stopreceipt=json.loads((stopdir/'resource-receipt.json').read_text())
assert stopreceipt.get('finished') and stopreceipt.get('exit')==0
assert stopreceipt['inputs_sha256']==stopreceipt['final_inputs_sha256']
assert not any(stopreceipt.get(k) for k in ['error','resource_stop','stopped_for_resource_limit'])
stop=json.loads((stopdir/'stop-summary.json').read_text())
assert stop['binary_sha256']==PIN and stop['pass'] and len(stop['tests'])==2
assert all(t['pass'] and not t['survivors'] and t['native_exit']<0 for t in stop['tests'])
allowed = sorted(os.sched_getaffinity(0)); assert len(allowed) >= 16
os.sched_setaffinity(0,set(allowed[:16]))
os.environ.update(TMPDIR=str(ROOT/'work/tmp'),XDG_CACHE_HOME=str(ROOT/'work/cache'))
for private in [ROOT/'work',ROOT/'work/tmp',ROOT/'work/cache']:
    assert private.stat().st_mode & 0o777 == 0o700
for source,name,digest in [
    ('render/04_reading_color.mp4','reference.mp4','eecf77538d835bcc77cd0e452b8d481a86837d4db1a2cadecd748a5176f30b01'),
    ('resource-receipt.json','reference-receipt.json','046a7480c224cf6a625f242fbb744848550e9f8df88ec9ba8b750dfabd3e4c6e'),
    ('detail-summary.json','reference-summary.json','58d9ac95b54cd8dfb18c16132e91c42a4418bbb0bd5918ac10f4f7a1f7757221'),
]:
    target=JOB/name
    assert not target.exists()
    subprocess.run(['scp','-p','-o','BatchMode=yes','-o','ConnectTimeout=6',SOURCE+source,str(target)],check=True,timeout=180)
    assert sha(target)==digest
receipt=json.loads((JOB/'reference-receipt.json').read_text())
assert receipt.get('exit')==0 and receipt.get('finished')
assert receipt['inputs_sha256']==receipt['final_inputs_sha256']
assert not any(receipt.get(k) for k in ['error','resource_stop','stopped_for_resource_limit'])
reference=json.loads((JOB/'reference-summary.json').read_text())
assert reference['binary_sha256']==PIN and reference['frames']==111
command=[str(BIN),'render',CHAPTER,'--silent','--res','3840x2160','--fps','1',
         '--threads','16','--out',str(JOB/'render'),'--crf','0','--preset','veryfast']
start=time.monotonic()
with (JOB/'native.stdout').open('xb') as out,(JOB/'native.stderr').open('xb') as err:
    subprocess.run(command,cwd=ROOT,stdout=out,stderr=err,check=True,timeout=7200)
elapsed=time.monotonic()-start
movies=[JOB/'reference.mp4',JOB/'render'/(CHAPTER+'.mp4')]
hashes=[]
for index,movie in enumerate(movies):
    streams=json.loads(subprocess.check_output(['ffprobe','-v','error','-show_streams','-of','json',str(movie)],timeout=60))['streams']
    videos=[s for s in streams if s['codec_type']=='video']
    assert len(videos)==1
    v=videos[0]
    assert (v['width'],v['height'],v['avg_frame_rate'],int(v['nb_frames']))==(3840,2160,'1/1',111)
    output=JOB/f'frames-{index}.framemd5'
    subprocess.run(['ffmpeg','-nostdin','-n','-v','error','-xerror','-threads','2','-i',str(movie),
                    '-map','0:v:0','-an','-pix_fmt','yuv420p','-threads','2','-f','framemd5',str(output)],check=True,timeout=600)
    rows=[line.strip() for line in output.read_text().splitlines() if line and not line.startswith('#')]
    assert len(rows)==111
    hashes.append(rows)
different=[i for i,(a,b) in enumerate(zip(*hashes)) if a!=b]
result={'binary_sha256':PIN,'chapter':CHAPTER,'frames':111,'resolution':[3840,2160],
        'fps':1,'different_frames':different,'all_decoded_frames_equal':not different,
        'render_seconds':elapsed,'reference_mp4_sha256':sha(movies[0]),'trj_mp4_sha256':sha(movies[1]),
        'scope':'Full decoded YUV-frame parity for one silent 4K diagnostic only. Not narrated timing, audio mux, 60fps motion, all scenes or release approval.'}
with (JOB/'parity-summary.json').open('x') as f:json.dump(result,f,indent=2)
assert not different, f'Decoded frame mismatch; preserve and inspect frames {different}'
print('All111 native4K diagnostic decoded frames equal across CSD and TRJ.')
