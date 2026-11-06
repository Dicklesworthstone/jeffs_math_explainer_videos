"""Build corrected stage07 with its native delivery tools in the proper repository layout."""
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import sys
import shutil
import time

ROOT = Path('/data/projects/jmev-fmd-render-qualification-20261106-1111')
WORK = ROOT / 'work'
SOURCE = WORK / 'picture-repair03/workspace'
ORIGINAL_TREE = SOURCE
BUILD_TREE = WORK / 'raster-quality-fmd01/workspace'
CANDIDATE = ROOT / 'qualification01/raster-quality01'
CRATE = BUILD_TREE / 'explainer_corpus/complex_analysis_visual'
OUT = ROOT / 'jobs/fmd-raster-quality-build02'
OLD_BIN = WORK / 'bin/cand'
BIN = WORK / 'bin/production09f'
PIN = 'fda2b880f716e4840f808924835b18ba93f5dc532c56c38f61f6744fa58edf04'
FACTS = '7d528509752e532c07edb3fda7c12cd8251ac4ba77022355ae42ad725add4417'
CHAPTERS = ['01_the_bump','02_turning_numbers','03_two_planes','04_reading_color','05_gallery','06_one_derivative','07_one_point','08_loop_sums','09_boundary','10_rigidity','11_counting','12_residues','13_real_integral','14_continuation','15_closing']

def sha(p):
    with open(p,'rb') as f:
        return hashlib.file_digest(f,'sha256').hexdigest()

def run(name, command, cwd=WORK):
    with (OUT/(name+'.stdout')).open('xb') as stdout, (OUT/(name+'.stderr')).open('xb') as stderr:
        result = subprocess.run([str(x) for x in command], cwd=cwd, stdout=stdout, stderr=stderr)
    assert result.returncode == 0, (name,result.returncode)
    return (OUT/(name+'.stdout')).read_bytes()

assert sys.argv[1:] == ['--run']
assert sha(OLD_BIN) == PIN
assert sha(ROOT/'qualification01/release-facts.tsv') == FACTS
assert os.environ.get('CARGO_BUILD_JOBS') == '2'
for attempt in range(100):
    try:
        receipt=json.loads((OUT/'resource-receipt.json').read_text())
        break
    except json.JSONDecodeError:
        time.sleep(0.05)
else:
    raise RuntimeError('Guard receipt remained incomplete')
assert receipt.get('admitted') is True and 'exit' not in receipt
assert receipt['policy'] == {'admission_gib':10,'stop_free_gib':8,'phase_growth_gib':4,'work_cap_gib':56,'target_growth_cap_gib':20,'sample_seconds':10}
tool=Path(subprocess.check_output([str(Path.home()/'.cargo/bin/rustup'),'which','--toolchain','nightly-2026-08-31','cargo'],text=True).strip())
assert sha(tool) == 'b854c9424332416075760b8b5fc8a43254e8eea8f38f14a57deea6126810f1b9'
assert sha(tool.parent/'rustc') == '49f9fa06f3e2b93aba85273433fef67b984ddb4cacb4500de56e85f70ec2e339'
for key in os.environ:
    assert key not in {'RUSTFLAGS','CARGO_ENCODED_RUSTFLAGS','RUSTC','RUSTC_WRAPPER','RUSTC_WORKSPACE_WRAPPER','CARGO_HOME'}
    assert not key.startswith('CARGO_PROFILE_')
    assert not key.startswith('CARGO_BUILD_') or key=='CARGO_BUILD_JOBS'
os.environ.update(PATH=str(tool.parent)+':/usr/bin:/bin',CARGO_INCREMENTAL='0',RCH_SHIM_LOCAL_IDE='1',CARGO_TARGET_DIR=str(WORK/'target'),TMPDIR=str(WORK/'tmp'),XDG_CACHE_HOME=str(WORK/'cache'))
assert not (Path.home()/'.cargo/config').exists()
home_config=Path.home()/'.cargo/config.toml'
assert not home_config.exists() or sha(home_config)=='14ae6b73244bde60c647383e9c4f6321fa575e7bb8ad75d6916a7adfefe0b4e6'
flags=['--release','--locked','--offline','--config',str(WORK/'tree/engine-budget.toml'),'--manifest-path',str(CRATE/'Cargo.toml'),'--bin','complex_analysis_visual']
assert not BUILD_TREE.exists() and not BIN.exists(), 'Preserve previous attempt'
shutil.copytree(ORIGINAL_TREE, BUILD_TREE)
changes = json.loads((CANDIDATE/'candidate-receipt01.json').read_text())['files']
changed_names = set()
for change in changes:
    name = change['file']
    target = CRATE/'src/chapters'/name
    assert sha(target) == change['before_sha256'], name
    assert sha(CANDIDATE/name) == change['candidate_sha256'], name
    shutil.copy2(target, OUT/('before-'+name))
    target.write_bytes((CANDIDATE/name).read_bytes())
    changed_names.add('explainer_corpus/complex_analysis_visual/src/chapters/'+name)
run('fmt-candidate',[tool,'fmt','--manifest-path',CRATE/'Cargo.toml','--','--config-path',WORK/'tree/repo-view/rustfmt.toml'])
assert Path('/data/projects/jmev-stroke-probe-20261106').samefile(ROOT)
assert (BUILD_TREE/'tools/narration_delivery.py').is_file()
assert sha(BUILD_TREE/'tools/narration_delivery.py')==sha(SOURCE/'tools/narration_delivery.py')
manifest = CRATE/'Cargo.toml'
original = manifest.read_text()
prefix = '/Users/jemanuel/scratch/jmev-geometry-native-20261104-01/repo-view'
assert original== (SOURCE/'explainer_corpus/complex_analysis_visual/Cargo.toml').read_text().replace(prefix,'/data/projects/jmev-stroke-probe-20261106/work/tree/repo-view')
source_hashes={str(p.relative_to(BUILD_TREE)):sha(p) for p in BUILD_TREE.rglob('*') if p.is_file()}
assert set(source_hashes)=={str(p.relative_to(SOURCE)) for p in SOURCE.rglob('*') if p.is_file()}
for name,digest in source_hashes.items():
    if name not in changed_names | {'explainer_corpus/complex_analysis_visual/Cargo.toml'}:
        assert digest==sha(SOURCE/name),name
with (OUT/'source-copy.json').open('x') as f:
    json.dump({'files':source_hashes,'changed':sorted(changed_names),'change':'Native mathematically sampled ImageQuad colour field replacing production row strips; all other repaired source preserved.'},f,indent=2)
run('fmt',[tool,'fmt','--manifest-path',CRATE/'Cargo.toml','--','--config-path',WORK/'tree/repo-view/rustfmt.toml','--check'])
# The inherited content fingerprint cache reused a location-dependent binary.
# A root-crate metadata salt forces recompilation, retaining dependency caches
# and all rendering/optimization settings. No source or resource guard changes.
test=run('tests',[tool,'test',*flags]).decode()
assert re.findall(r'^test result: (.+?); finished in ',test,re.M)==['ok. 208 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out']
assert len(re.findall(r'^test \S+ \.\.\. ok$',test,re.M))==208
run('build',[tool,'rustc',*flags,'--','-C','metadata=jmev_raster_quality09f'])
shutil.copy2(WORK/'target/release/complex_analysis_visual',BIN)
assert os.access(BIN,os.X_OK)
PIN=sha(BIN)
assert PIN!=sha(OLD_BIN), 'Compiled manifest path must change'
run('clippy',[tool,'clippy',*flags,'--no-deps','--','-D','warnings'])
native=run('facts-native',[BIN,'facts'],CRATE)
# This host lacks SciPy. Verify unchanged oracle and facts against the successful
# source build receipt, then compare the new binary's facts exactly. This does not
# claim a fresh independent oracle execution on FMD.
prior=json.loads((ROOT/'qualification01/picture-source-receipts08/build-qualification.json').read_text())
assert prior['facts_sha256']==FACTS
source_binding=json.loads((CANDIDATE/'fmd-source-binding01.json').read_text())
assert sha(CRATE/'oracle.py')==source_binding['oracle_sha256']
assert sha(CRATE/'src/facts.rs')==source_binding['facts_source_sha256']
assert native==(ROOT/'qualification01/release-facts.tsv').read_bytes()
versions='Prior CSD oracle retained; unchanged oracle/facts source hashes verified; no FMD oracle run'
chapters=run('chapters',[BIN,'chapters']).decode().splitlines()
assert chapters==CHAPTERS
assert sha(BIN)==PIN==sha(WORK/'target/release/complex_analysis_visual')
assert source_hashes=={str(p.relative_to(BUILD_TREE)):sha(p) for p in BUILD_TREE.rglob('*') if p.is_file()}
assert sha(OLD_BIN)=='fda2b880f716e4840f808924835b18ba93f5dc532c56c38f61f6744fa58edf04'
with (OUT/'qualification.json').open('x') as f:
    json.dump({'binary_sha256':PIN,'tests_passed':208,'clippy_passed':True,'facts_sha256':hashlib.sha256(native).hexdigest(),'oracle_versions':versions,'chapters':chapters,'scope':'Native raster candidate build only. Actual native4K/fade/repaint review, source rebind and schedule equality still required; no listening/final acceptance.'},f,indent=2)
print('QUALIFICATION PASS: isolated raster candidate, 208 tests, clippy, pinned facts equality, 15 chapters; native4K pictures still unaccepted.',flush=True)
