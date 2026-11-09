"""Recover interrupted CSD chapter10 on the already qualified native Mac. One guarded 4K render."""
import hashlib
import json
import os
from pathlib import Path
import subprocess

R=Path.home()/'scratch/jmev-complex-native-20261109'
Q=R/'qualification'
PY='/opt/homebrew/opt/python@3.14/bin/python3.14'

def sha(p):
    with p.open('rb') as f:
        return hashlib.file_digest(f,'sha256').hexdigest()

def main():
    assert not (R/'jobs/mini-final555').exists(), 'no duplicate final job'
    rows=json.loads((Q/'mini-preview542-inputs.json').read_text())['files']
    for row in rows:
        assert sha(Path(row['path']))==row['sha256'], row['path']
    for job in ['native-build521','mini-delivery540','mini-timeline541','mini-preview542','mini-pictures543']:
        p=R/'jobs'/job/'resource-receipt.json'
        d=json.loads(p.read_text())
        assert d.get('admitted') and d.get('finished') and d.get('exit')==0, job
        assert d['inputs_sha256']==d['final_inputs_sha256'] and not d.get('resource_stop'),job
        rows.append(dict(path=str(p),sha256=sha(p)))
    review=Q/'root-mini-preview542-pictures-viewed548.json'
    d=json.loads(review.read_text())
    assert d['selected_pictures_viewed']==113 and d['sheets_viewed']==list(range(1,16))
    assert d['movie_sha256']=='9ef7ea9df7f2bff62a9e6c01ab221536d4a24045408a60216abab5b4f2c1f6d6'
    for ext in ['timeline.tsv','cues.tsv','cuelog.tsv']:
        a=R/'jobs/mini-timeline541/timeline-output'/('10_rigidity.'+ext)
        b=R/'jobs/mini-preview542/preview'/('10_rigidity.'+ext)
        assert a.read_bytes()==b.read_bytes(),ext
    stem=R/'inputs523/score21/10_rigidity.music.f32.wav'
    assert sha(stem)=='3307c2c9bddcb58d2c21899f1fcc68a836d815d71e430de179fd41cdcf1e0083'
    assert stem.stat().st_size==47628880
    for p in [review,stem,Path(__file__)]:
        rows.append(dict(path=str(p),sha256=sha(p)))
    unique={row['path']:row['sha256'] for row in rows}
    manifest=Q/'root-mini-final555-inputs.json'
    with manifest.open('x') as f:
        json.dump(dict(schema='mini-final555',files=[dict(path=k,sha256=v) for k,v in sorted(unique.items())]),f,indent=2)
    cmd=[PY,'-I','-B',str(Q/'cc2-mini-max-guard508.py'),'--phase','native','--job','mini-final555',
         '--inputs',str(manifest),'--',str(R/'jobs/native-build521/complex_analysis_visual-521'),
         'render','10_rigidity','--res','3840x2160','--fps','60','--threads','4',
         '--narration',str(R/'jobs/mini-delivery540/narration'),'--score',str(stem.parent),
         '--out',str(R/'jobs/mini-final555/render'),'--lint','--crf','16','--preset','veryfast']
    env=dict(os.environ,PATH=str(Path.home()/'.local/bin')+':/opt/homebrew/bin:/usr/bin:/bin')
    return subprocess.run(cmd,env=env,check=False).returncode

if __name__=='__main__':
    raise SystemExit(main())
