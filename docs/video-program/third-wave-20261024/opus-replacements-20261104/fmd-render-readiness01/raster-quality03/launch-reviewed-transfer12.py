"""Launch the transfer only after actual raster12 review and a bound permit."""
import hashlib, importlib.util, json, subprocess, sys
from pathlib import Path
R=Path('/data/projects/jmev-fmd-render-qualification-20261106-1111')
Q=R/'qualification01/raster-quality03'
def sha(p):
    with p.open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest()
def main():
    assert sys.argv[1:]==['--run']
    assert Path(__file__).resolve().parent==Q
    spec=importlib.util.spec_from_file_location('gate',Q/'final-review-gate12.py')
    gate=importlib.util.module_from_spec(spec);spec.loader.exec_module(gate)
    verdict=gate.check(Q/'preview-reviewed12.json')
    permit=json.loads((Q/'distributed-review-permit12.json').read_text())
    assert permit['review_gate_result']==verdict
    assert permit['assignment_sha256']==sha(Q/'chapter-assignment12.json')
    assert not (R/'jobs/fmd-raster-transfer12').exists()
    paths={Path(__file__).resolve(),Path(sys.executable).resolve(),R/'guard_fmd01.py'}
    paths.update(Q.glob('*.py'))
    paths.update(Q/n for n in ('preview-reviewed12.json','distributed-review-permit12.json','chapter-assignment12.json'))
    for phase in ('timeline','preview','pictures'):
        gate.S.success('fmd-raster-'+phase+'12')
        manifest=R/('fmd-raster-'+phase+'12-inputs.json')
        paths.add(manifest)
        for row in json.loads(manifest.read_text())['files']:
            p=Path(row['path']);assert sha(p)==row['sha256'],p;paths.add(p)
        paths.update(p for p in (R/'jobs'/('fmd-raster-'+phase+'12')).rglob('*') if p.is_file())
    manifest=R/'fmd-raster-transfer12-inputs.json'
    with manifest.open('x') as f:
        json.dump({'files':[{'path':str(p),'sha256':sha(p)} for p in sorted(paths)]},f,indent=2)
    assert gate.check(Q/'preview-reviewed12.json')==verdict
    subprocess.run(['/usr/bin/python3','-I','-B',str(R/'guard_fmd01.py'),'--phase','native','--job','fmd-raster-transfer12','--inputs',str(manifest),'--','/usr/bin/timeout','1800',sys.executable,'-I','-B',str(Q/'prepare-csd-transfer12.py'),'--run'],check=True)
if __name__=='__main__':main()
