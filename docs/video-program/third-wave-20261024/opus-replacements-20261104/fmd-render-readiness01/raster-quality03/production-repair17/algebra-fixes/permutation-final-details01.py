from pathlib import Path
import json,subprocess,hashlib
R=Path.cwd();B=R/'narration_lab/third-wave-20261024/root/five-film-release-20261027';O=B/'algebra-final14-permutation-details01';J=B/'algebra-final14-11_permutations';r=json.loads((J/'resource-receipt.json').read_text());assert r['exit']==0 and r['inputs_sha256']==r['final_inputs_sha256']
movie=J/'render/11_permutations.mp4';frames=[14160,14880,15300,42264];selector="select='"+'+'.join('eq(n,'+str(x)+')' for x in frames)+"'";p=O/'selection.filter';p.write_text(selector)
subprocess.run(['ffmpeg','-nostdin','-n','-v','error','-xerror','-threads','2','-i',str(movie),'-an','-filter_script:v',str(p),'-vsync','0',str(O/'frame-%02d.png')],check=True,timeout=1800)
files=sorted(O.glob('frame-*.png'));assert len(files)==len(frames);out=[]
for f,p in zip(frames,files):
 v=json.loads(subprocess.check_output(['ffprobe','-v','error','-show_streams','-of','json',str(p)]))['streams'][0];assert (v['width'],v['height'])==(3840,2160);out.append(dict(frame=f,seconds=f/60,path=str(p),sha256=hashlib.sha256(p.read_bytes()).hexdigest()))
(O/'pictures.json').write_text(json.dumps(dict(pictures=out,viewed=False,scope='Four native4K detail samples only; no motion/listening/fullchapter acceptance.'),indent=2))
