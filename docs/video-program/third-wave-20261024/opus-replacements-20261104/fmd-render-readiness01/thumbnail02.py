"""Guarded native thumbnail from the exact production binary; no film approval."""
import hashlib
import json
from pathlib import Path
import subprocess
import sys

ROOT=Path('/data/projects/jmev-fmd-render-qualification-20261106-1111')
JOB=ROOT/'jobs/fmd-thumbnail02'
OUT=JOB/'output'
BIN=ROOT/'production07d'
PIN='2ccda773be0d6d9b6a83660a5481277d59b5ce2e6a4b80852d826388cad0cd0e'
POLICY=dict(admission_gib=10,stop_free_gib=8,phase_growth_gib=4,work_cap_gib=56,target_growth_cap_gib=20,sample_seconds=10)
def sha(path):
    with Path(path).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest()
def run():
    receipt=json.loads((JOB/'resource-receipt.json').read_text())
    assert receipt['admitted'] and 'exit' not in receipt and receipt['policy']==POLICY
    assert receipt['command']==['/usr/bin/python3','-I','-B',str(Path(__file__).resolve()),'--run']
    assert sha(BIN)==PIN and not OUT.exists()
    results=[]
    for width,height in ((1280,720),(320,180)):
        target=OUT/str(width)
        subprocess.run([str(BIN),'thumbnail','--res',f'{width}x{height}','--out',str(target)],
                       cwd=ROOT/'production-inputs01/production-repo07',check=True,timeout=600)
        picture=target/'complex_analysis_visual_thumbnail.png'
        probe=json.loads(subprocess.check_output(['ffprobe','-v','error','-show_streams','-of','json',str(picture)],timeout=60))
        assert len(probe['streams'])==1
        stream=probe['streams'][0]
        assert (stream['codec_name'],stream['width'],stream['height'])==('png',width,height)
        decoded=subprocess.run(['ffmpeg','-nostdin','-v','error','-i',str(picture),'-f','null','-'],capture_output=True,check=True,timeout=60)
        assert not decoded.stderr
        results.append(dict(path=str(picture),sha256=sha(picture),size=[width,height],bytes=picture.stat().st_size))
    assert sha(BIN)==PIN
    assert results[0]['bytes'] < 2*1024*1024
    with (JOB/'thumbnail-summary.json').open('x') as f:
        json.dump(dict(images=results,binary_sha256=PIN,full_png_decode=True,pictures_viewed=False,
                       scope='Native upload-size and phone-size thumbnail only; actual visual review required.'),f,indent=2)
    print('Native thumbnail decoded; actual picture review remains.')
if __name__=='__main__':
    assert sys.argv[1:]==['--run']
    run()
