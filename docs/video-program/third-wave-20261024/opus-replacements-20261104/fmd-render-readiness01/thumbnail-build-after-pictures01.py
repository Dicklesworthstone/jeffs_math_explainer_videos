"""One guarded isolated thumbnail build after the existing preview extraction."""
import hashlib,json,os,subprocess,time
from pathlib import Path
R=Path('/data/projects/jmev-stroke-probe-20261106')
HERE=R/'qualification01'
def main():
    deadline=time.monotonic()+48*3600
    while time.monotonic()<deadline:
        ready=True
        for name in ('stage07-preview01','stage07-pictures01'):
            try:r=json.loads((R/'jobs'/name/'resource-receipt.json').read_text())
            except (FileNotFoundError,json.JSONDecodeError):ready=False;continue
            if not r.get('finished'):ready=False;continue
            assert r.get('exit')==0 and not r.get('error'),(name,r)
            assert not r.get('stopped_for_resource_limit') and not r.get('resource_stop'),name
            assert r['inputs_sha256']==r['final_inputs_sha256'],name
        if ready:
            manifest=R/'stage07-thumbnail-build01-inputs.json'
            for f in json.loads(manifest.read_text())['files']:
                assert hashlib.sha256(Path(f['path']).read_bytes()).hexdigest()==f['sha256'],f['path']
            assert not (R/'jobs/stage07-thumbnail-build01').exists()
            env=os.environ.copy();env['CARGO_BUILD_JOBS']='2'
            return subprocess.run(['/usr/bin/python3','-I','-B',str(R/'guard_csd02.py'),'--phase','native','--job','stage07-thumbnail-build01','--inputs',str(manifest),'--','/usr/bin/timeout','7200','/usr/bin/python3','-I','-B',str(HERE/'build-thumbnail-smooth01.py'),'--run'],env=env).returncode
        time.sleep(30)
    print('Observation deadline only; no job launched',flush=True)
    return 3
if __name__=='__main__':raise SystemExit(main())
