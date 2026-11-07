from pathlib import Path
import json,hashlib,shutil,subprocess,sys
R=Path('/data/projects/jmev-stroke-probe-20261106');Q=R/'qualification01/production-repair17';T=R/'work/production-repair19/workspace';J=R/'jobs/csd-production-assembly19'
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
a=json.loads((J/'resource-receipt.json').read_text());assert a['admitted'] and 'exit' not in a
s=json.loads((R/'jobs/csd-production-repair-build19/source-copy.json').read_text())['files'];p=T/'tools/coherent_narration.py';b=T/'tools/coherent_narration.before-production19.py';assert sha(p)==s['tools/coherent_narration.py'] and not b.exists();shutil.copy2(p,b);shutil.copy2(Q/'coherent_narration.py',p)
subprocess.run([sys.executable,'-I','-B',str(Q/'prepare-assembly19.py'),'run','--adapter-sha256',sha(p)],check=True)
