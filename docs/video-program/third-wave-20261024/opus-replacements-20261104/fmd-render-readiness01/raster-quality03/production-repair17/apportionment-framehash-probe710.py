"""Two actual chapter01 framehash rows only; not full decode or throughput qualification."""
import ast, hashlib, json, re, subprocess
from pathlib import Path
D=Path('/data/projects/jmev-stroke-probe-20261106');Q=D/'qualification01/production-repair17';J=D/'jobs/csd-apportionment-framehash710'
result=json.loads((D/'jobs/csd-apportionment-import683-a1/import-result.json').read_text())
rows=[r for r in result['files'] if r['path'].endswith('/01_one_more_seat.mp4')]
assert len(rows)==1
movie=Path(rows[0]['physical']);assert hashlib.sha256(movie.read_bytes()).hexdigest()==rows[0]['sha256']
verifier=Q/'apportionment-csd-verify701.py'
assert hashlib.sha256(verifier.read_bytes()).hexdigest()=='02ef7a5f4aae336b4a3795b066352a85c9a536d953c8920fab4755a2f174417f'
node=next(n for n in ast.parse(verifier.read_text()).body if isinstance(n,ast.FunctionDef) and n.name=='frame_digests')
scope={'Path':Path,'re':re};exec(compile(ast.Module(body=[node],type_ignores=[]),str(verifier),'exec'),scope)
out=J/'two-frames.sha256'
subprocess.run(['/usr/bin/ffmpeg','-nostdin','-v','error','-xerror','-threads','4','-i',str(movie),'-map','0:v:0','-frames:v','2','-fps_mode','passthrough','-f','framehash','-hash','sha256',str(out)],check=True,timeout=120)
digests=scope['frame_digests'](out);assert len(digests)==2
with(J/'probe710.json').open('x') as f:json.dump({'chapter':'01_one_more_seat','frames_checked':2,'actual_701_parser_passed':True,'framehash_sha256':hashlib.sha256(out.read_bytes()).hexdigest(),'scope':'Framehash row form only. Not full decode, throughput, frame identity, listening or release acceptance.'},f,indent=2)
