import os,json,hashlib,subprocess,shutil,re
from pathlib import Path
R=Path('/data/projects/jmev-stroke-probe-20261106');W=R/'work';Q=R/'qualification01/production-repair17';J=R/'jobs/csd-review-repair-build262';S=W/'production-repair19/workspace';B=W/'review-repair262/workspace';C=B/'explainer_corpus/complex_analysis_visual'
def sha(p):
 with p.open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest()
def run(n,argv,cwd=C):
 with (J/(n+'.stdout')).open('xb') as o,(J/(n+'.stderr')).open('xb') as e:r=subprocess.run(list(map(str,argv)),cwd=cwd,stdout=o,stderr=e,timeout=7200)
 assert r.returncode==0,(n,r.returncode)
 return (J/(n+'.stdout')).read_bytes()
a=json.loads((J/'resource-receipt.json').read_text());assert a['admitted'] and 'exit' not in a
assert a['policy']==dict(admission_gib=10,stop_free_gib=8,phase_growth_gib=4,work_cap_gib=56,target_growth_cap_gib=20,sample_seconds=10)
os.sched_setaffinity(0,{0,1,2,3})
before=S/'explainer_corpus/complex_analysis_visual/src/chapters/boundary.rs'
assert sha(before)=='c9e6b5ba84c4cf5ae327a8268d67c4f134484be8538cb99bdf1828ebe051f630'
assert not B.exists();shutil.copytree(S,B)
recipe=json.loads((Q/'review-repair-recipe262.json').read_text())
assert recipe['reviewed_by_root'] is True and recipe['scope']=='complex visual repairs'
expected={'boundary.rs','loop_sums.rs','one_derivative.rs','mod.rs','rigidity.rs','gallery.rs','draw.rs','plate_fade_probe.rs'}
assert set(recipe['files'])==expected
for target,item in recipe['files'].items():
 candidate=Q/item['candidate']
 assert candidate.parent==Q and candidate.is_file()
 assert sha(candidate)==item['sha256']
 shutil.copy2(candidate,C/'src/chapters'/target)

tool=Path(subprocess.check_output([str(Path.home()/'.cargo/bin/rustup'),'which','--toolchain','nightly-2026-08-31','cargo'],text=True).strip());assert sha(tool)=='b854c9424332416075760b8b5fc8a43254e8eea8f38f14a57deea6126810f1b9';assert sha(tool.parent/'rustc')=='49f9fa06f3e2b93aba85273433fef67b984ddb4cacb4500de56e85f70ec2e339'
for k in ['RUSTFLAGS','CARGO_ENCODED_RUSTFLAGS','RUSTC','RUSTC_WRAPPER','RUSTC_WORKSPACE_WRAPPER']:assert k not in os.environ
os.environ.update(PATH=str(tool.parent)+':/usr/bin:/bin',CARGO_INCREMENTAL='0',CARGO_BUILD_JOBS='2',RCH_SHIM_LOCAL_IDE='1',CARGO_TARGET_DIR=str(W/'target'),TMPDIR=str(W/'tmp'),XDG_CACHE_HOME=str(W/'cache'))
run('fmt',[tool,'fmt','--manifest-path',C/'Cargo.toml','--','--config-path',W/'tree/repo-view/rustfmt.toml'])
source={str(p.relative_to(B)):sha(p) for p in B.rglob('*') if p.is_file()};old={str(p.relative_to(S)):sha(p) for p in S.rglob('*') if p.is_file()};assert set(source)-set(old)=={'explainer_corpus/complex_analysis_visual/src/chapters/plate_fade_probe.rs'};assert not set(old)-set(source);changed=sorted(n for n in source if source[n]!=old.get(n));assert changed==sorted('explainer_corpus/complex_analysis_visual/src/chapters/'+x for x in expected),changed
(J/'source-copy.json').write_text(json.dumps(dict(files=source,changed=changed),indent=2))
f=['--release','--locked','--offline','--config',str(W/'tree/engine-budget.toml'),'--manifest-path',str(C/'Cargo.toml'),'--bin','complex_analysis_visual']
tests=run('tests',[tool,'test',*f]).decode();counts=re.findall(r'^test result: ok\. (\d+) passed; 0 failed; 1 ignored; 0 measured; 0 filtered out;',tests,re.M);assert len(counts)==1 and int(counts[0])==233,tests[-2000:]
probe=run('plate-fade-probe',[tool,'test',*f,'plate_fade_probe','--','--ignored','--nocapture']).decode();assert re.search(r'^test result: ok\. 1 passed; 0 failed; 0 ignored; 0 measured; \d+ filtered out;',probe,re.M),probe[-3000:]
run('clippy',[tool,'clippy',*f,'--no-deps','--','-D','warnings'])
run('build',[tool,'rustc',*f,'--','-C','metadata=jmev_review_repair262'])
binary=W/'bin/review-repair262';assert not binary.exists();shutil.copy2(W/'target/release/complex_analysis_visual',binary)
facts=run('facts',[binary,'facts']);assert hashlib.sha256(facts).hexdigest()=='7d528509752e532c07edb3fda7c12cd8251ac4ba77022355ae42ad725add4417'
chapters=run('chapters',[binary,'chapters']).decode().splitlines();prior=json.loads((R/'jobs/fmd-raster-quality-build05/qualification.json').read_text());assert chapters==prior['chapters']
assert source=={str(p.relative_to(B)):sha(p) for p in B.rglob('*') if p.is_file()}
(J/'qualification.json').write_text(json.dumps(dict(binary_sha256=sha(binary),tests_passed=int(counts[0]),explicit_native_plate_fade_probe_passed=True,clippy_passed=True,facts_sha256=hashlib.sha256(facts).hexdigest(),chapters=chapters,scope='Native build and focused automated fade regression only; no complete picture/motion/listening or release acceptance'),indent=2))
print('Native combined visual-repair build and focused opacity probe passed; exact full schedules and actual repaired pictures remain required.')
