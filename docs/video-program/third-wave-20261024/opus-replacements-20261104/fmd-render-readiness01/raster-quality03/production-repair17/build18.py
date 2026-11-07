import os,json,hashlib,subprocess,shutil,re
from pathlib import Path
R=Path('/data/projects/jmev-stroke-probe-20261106');W=R/'work';Q=R/'qualification01/production-repair17';J=R/'jobs/csd-production-repair-build18';S=W/'continuous-quality16/workspace';B=W/'production-repair18/workspace';C=B/'explainer_corpus/complex_analysis_visual'
def sha(p):
 with p.open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest()
def run(n,argv,cwd=C):
 with (J/(n+'.stdout')).open('xb') as o,(J/(n+'.stderr')).open('xb') as e:r=subprocess.run(list(map(str,argv)),cwd=cwd,stdout=o,stderr=e,timeout=7200)
 assert r.returncode==0,(n,r.returncode)
 return (J/(n+'.stdout')).read_bytes()
a=json.loads((J/'resource-receipt.json').read_text());assert a['admitted'] and 'exit' not in a
assert a['policy']==dict(admission_gib=10,stop_free_gib=8,phase_growth_gib=4,work_cap_gib=56,target_growth_cap_gib=20,sample_seconds=10)
os.sched_setaffinity(0,{0,1,2,3})
pins=json.loads((Q/'candidate-receipt02.json').read_text())
for row in pins['files']:
 assert sha(S/'explainer_corpus/complex_analysis_visual'/row['relative'])==row['before_sha256'];assert sha(Q/row['file'])==row['sha256']
assert not B.exists();shutil.copytree(S,B)
for row in pins['files']:shutil.copy2(Q/row['file'],C/row['relative'])

tool=Path(subprocess.check_output([str(Path.home()/'.cargo/bin/rustup'),'which','--toolchain','nightly-2026-08-31','cargo'],text=True).strip());assert sha(tool)=='b854c9424332416075760b8b5fc8a43254e8eea8f38f14a57deea6126810f1b9';assert sha(tool.parent/'rustc')=='49f9fa06f3e2b93aba85273433fef67b984ddb4cacb4500de56e85f70ec2e339'
for k in ['RUSTFLAGS','CARGO_ENCODED_RUSTFLAGS','RUSTC','RUSTC_WRAPPER','RUSTC_WORKSPACE_WRAPPER']:assert k not in os.environ
os.environ.update(PATH=str(tool.parent)+':/usr/bin:/bin',CARGO_INCREMENTAL='0',CARGO_BUILD_JOBS='2',RCH_SHIM_LOCAL_IDE='1',CARGO_TARGET_DIR=str(W/'target'),TMPDIR=str(W/'tmp'),XDG_CACHE_HOME=str(W/'cache'))
run('fmt',[tool,'fmt','--manifest-path',C/'Cargo.toml','--','--config-path',W/'tree/repo-view/rustfmt.toml'])
source={str(p.relative_to(B)):sha(p) for p in B.rglob('*') if p.is_file()};old={str(p.relative_to(S)):sha(p) for p in S.rglob('*') if p.is_file()};assert source.keys()==old.keys();changed=[n for n in source if source[n]!=old[n]];assert set(changed)=={'explainer_corpus/complex_analysis_visual/'+row['relative'] for row in pins['files']},changed
(J/'source-copy.json').write_text(json.dumps(dict(files=source,changed=changed),indent=2))
f=['--release','--locked','--offline','--config',str(W/'tree/engine-budget.toml'),'--manifest-path',str(C/'Cargo.toml'),'--bin','complex_analysis_visual']
tests=run('tests',[tool,'test',*f]).decode();counts=re.findall(r'^test result: ok\. (\d+) passed; 0 failed; 0 ignored; 0 measured; 0 filtered out;',tests,re.M);assert len(counts)==1 and int(counts[0])>=213,tests[-2000:]
run('clippy',[tool,'clippy',*f,'--no-deps','--','-D','warnings'])
run('build',[tool,'rustc',*f,'--','-C','metadata=jmev_production_repair18'])
binary=W/'bin/production18';assert not binary.exists();shutil.copy2(W/'target/release/complex_analysis_visual',binary)
facts=run('facts',[binary,'facts']);assert hashlib.sha256(facts).hexdigest()=='7d528509752e532c07edb3fda7c12cd8251ac4ba77022355ae42ad725add4417'
chapters=run('chapters',[binary,'chapters']).decode().splitlines();prior=json.loads((R/'jobs/fmd-raster-quality-build05/qualification.json').read_text());assert chapters==prior['chapters']
assert source=={str(p.relative_to(B)):sha(p) for p in B.rglob('*') if p.is_file()}
(J/'qualification.json').write_text(json.dumps(dict(binary_sha256=sha(binary),tests_passed=int(counts[0]),clippy_passed=True,facts_sha256=hashlib.sha256(facts).hexdigest(),chapters=chapters,scope='Native build only; no image/motion/listening or release acceptance'),indent=2))
print('Native repair build passed; new narration and actual rendered review still required.')
