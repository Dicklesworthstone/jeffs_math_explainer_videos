from pathlib import Path
import json,hashlib,subprocess,os,time
R=Path('/data/projects/jmev-hz2-render-worker-20261107');D=Path('/data/projects/jmev-stroke-probe-20261106');J=R/'jobs/hz2-native-profile50';N=R/'jobs/hz2-production19-delivery49/narration';B=D/'work/bin/production19'
r=json.loads((J/'resource-receipt.json').read_text());assert r['admitted'] and 'exit' not in r
r=json.loads((R/'jobs/hz2-production19-delivery49/resource-receipt.json').read_text());assert r['exit']==0 and r['inputs_sha256']==r['final_inputs_sha256'] and not r.get('stopped_for_resource_limit')
assert hashlib.sha256(B.read_bytes()).hexdigest()=='f4836eeeb4f2465c0d27ca0497f123f0af6bc0db7bc65096098bb20c5abb0f22'
os.environ.update(PATH='/usr/bin:/bin',TMPDIR=str(R/'work/tmp'),XDG_CACHE_HOME=str(R/'work/cache'))
args=[str(B),'render','04_reading_color','--narration',str(N),'--score',str(D/'jobs/csd-production-score21/score/stems'),'--out',str(J/'render'),'--res','3840x2160','--fps','60','--crf','16','--preset','veryfast','--threads','12','--max-frames','120']
t=time.monotonic()
with (J/'native.stdout').open('xb') as o,(J/'native.stderr').open('xb') as e:subprocess.run(args,cwd=D/'work/production-repo07',stdout=o,stderr=e,check=True,timeout=1800)
movies=list((J/'render').rglob('*.mp4'));assert len(movies)==1,movies
v=json.loads(subprocess.check_output(['ffprobe','-v','error','-show_streams','-of','json',str(movies[0])]))['streams'];v=[x for x in v if x['codec_type']=='video'];assert len(v)==1
v=v[0];assert (v['width'],v['height'],v['avg_frame_rate'],int(v['nb_frames']))==(3840,2160,'60/1',120)
with (J/'profile-summary.json').open('x') as f:json.dump(dict(frames=120,elapsed_s=time.monotonic()-t,movie=str(movies[0]),sha256=hashlib.sha256(movies[0].read_bytes()).hexdigest(),scope='Bounded120frame technical profile, not full chapter, picture review, final replay, listening or release acceptance'),f,indent=2)
