"""Explicit guarded final verification after the real current master exists."""
import fcntl
import importlib.util
import json
from pathlib import Path
import shutil
import subprocess
import sys

HERE = Path('/data/projects/jmev-stroke-probe-20261106/qualification01/raster-quality12')
spec = importlib.util.spec_from_file_location('final_verifier', HERE/'final-verify12.py')
V = importlib.util.module_from_spec(spec)
spec.loader.exec_module(V)
S, ROOT = V.S, V.ROOT

def runtime_files():
    import PIL
    from PIL import Image, ImageDraw, ImageFont
    assert PIL.__version__ == '12.1.1'
    paths = {Path(sys.executable).resolve()}
    paths.update(p for p in Path(PIL.__file__).parent.rglob('*')
                 if p.is_file() and p.suffix in ('.py', '.so'))
    for name in ('ffmpeg', 'ffprobe', 'bash'):
        executable = shutil.which(name)
        assert executable, name
        paths.add(Path(executable).resolve())
    return paths

def freeze():
    master_manifest = ROOT/'csd-raster-final-master12-inputs.json'
    receipt = S.success('csd-raster-final-master12')
    assert S.sha(master_manifest) == receipt['inputs_sha256']
    V.K.L.verify_inputs(master_manifest)
    files = {row['path']:row['sha256'] for row in S.read(master_manifest)['files']}
    def add(path):
        path = Path(path)
        digest = S.sha(path)
        if str(path) in files:
            assert files[str(path)] == digest, path
        files[str(path)] = digest
    add(master_manifest)
    for p in V.K.JOB.rglob('*'):
        if p.is_file():
            add(p)
    for p in runtime_files():
        add(p)
    for name in ('launch-final-verify12.py', 'final-verify12.py', 'final-samples12.py',
                 'final-master12.py', 'launch-csd-disjoint12.py', 'csd-final-chapter12.py',
                 'verify-fmd-chapter12.py'):
        add(HERE/name)
    shared = V.shared()
    for p in (V.SHARED, V.REPO/shared.STRICT_REL, V.REPO/shared.SCREEN_REL, V.SCRIPT):
        add(p)
    return {'files':[{'path':p,'sha256':h} for p,h in sorted(files.items())]}

def launch():
    assert sys.argv[1:] == ['--run']
    assert Path(__file__).resolve().parent == HERE
    with (HERE/'final-verify-driver12.lock').open('a') as lock:
        fcntl.flock(lock, fcntl.LOCK_EX | fcntl.LOCK_NB)
        assert not V.JOB.exists() and not V.MANIFEST.exists(), 'Preserve attempts; never duplicate'
        plan = V.plan()
        frozen = freeze()
        assert V.plan() == plan
        with V.MANIFEST.open('x') as f:
            json.dump(frozen, f, indent=2)
        V.K.L.verify_inputs(V.MANIFEST)
        subprocess.run(['/usr/bin/python3','-I','-B',str(ROOT/'guard_csd02.py'),
                        '--phase','native','--job',V.NAME,'--inputs',str(V.MANIFEST),
                        '--',*V.child()], cwd=ROOT, check=True)
        S.success(V.NAME)
        result = S.read(V.JOB/'verification.json')
        assert result['pictures_viewed'] is False and result['native_detail_viewed'] is False
        print('Final technical verification complete; actual final picture review remains.')

if __name__ == '__main__':
    launch()
