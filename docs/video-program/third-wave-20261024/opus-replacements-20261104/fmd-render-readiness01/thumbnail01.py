"""Guarded native thumbnail from the exact production binary; no film approval."""
import hashlib
import json
from pathlib import Path
import subprocess
import sys

ROOT=Path('/data/projects/jmev-fmd-render-qualification-20261106-1111')
JOB=ROOT/'jobs/fmd-thumbnail01'
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
    subprocess.run([str(BIN),'thumbnail','--res','3840x2160','--out',str(OUT)],
                   cwd=ROOT/'production-inputs01/production-repo07',check=True,timeout=600)
    picture=OUT/'complex_analysis_visual_thumbnail.png'
    probe=json.loads(subprocess.check_output(['ffprobe','-v','error','-show_streams','-of','json',str(picture)],timeout=60))
    assert len(probe['streams'])==1
    stream=probe['streams'][0]
    assert (stream['codec_name'],stream['width'],stream['height'])==('png',3840,2160)
    decoded=subprocess.run(['ffmpeg','-nostdin','-v','error','-i',str(picture),'-f','null','-'],capture_output=True,check=True,timeout=60)
    assert not decoded.stderr
    assert sha(BIN)==PIN
    with (JOB/'thumbnail-summary.json').open('x') as f:
        json.dump(dict(path=str(picture),sha256=sha(picture),binary_sha256=PIN,
                       size=[3840,2160],full_png_decode=True,pictures_viewed=False,
                       scope='Native thumbnail only; actual visual review required, no final AV approval.'),f,indent=2)
    print('Native thumbnail decoded; actual picture review remains.')
if __name__=='__main__':
    assert sys.argv[1:]==['--run']
    run()
