from pathlib import Path
import os,json,hashlib,shutil,subprocess,re
R=Path('/data/projects/jmev-stroke-probe-20261106');Q=R/'qualification01/production-repair17';W=R/'work';J=R/'jobs/csd-production-thumbnail24';S=W/'production-repair19/workspace';B=W/'thumbnail24/workspace';C=B/'explainer_corpus/complex_analysis_visual'
def sha(p):
 with p.open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest()
def run(name,args,cwd=C):
 with (J/(name+'.stdout')).open('xb') as out,(J/(name+'.stderr')).open('xb') as err:
  subprocess.run(list(map(str,args)),cwd=cwd,stdout=out,stderr=err,check=True,timeout=7200)
 return (J/(name+'.stdout')).read_text()
r=json.loads((J/'resource-receipt.json').read_text());assert r['admitted'] and 'exit' not in r
assert r['policy']==dict(admission_gib=10,stop_free_gib=8,phase_growth_gib=4,work_cap_gib=56,target_growth_cap_gib=20,sample_seconds=10)
prior=json.loads((R/'jobs/csd-production-repair-build19/resource-receipt.json').read_text());assert prior['exit']==0 and prior['inputs_sha256']==prior['final_inputs_sha256'] and not prior.get('resource_stop')
os.sched_setaffinity(0,{0,1,2,3});assert not B.exists();shutil.copytree(S,B)
assert sha(C/'src/chapters/thumbnail.rs')==sha(Q/'before-thumbnail24.rs');shutil.copy2(Q/'thumbnail24.rs',C/'src/chapters/thumbnail.rs')
tool=Path(subprocess.check_output([str(Path.home()/'.cargo/bin/rustup'),'which','--toolchain','nightly-2026-08-31','cargo'],text=True).strip());assert sha(tool)=='b854c9424332416075760b8b5fc8a43254e8eea8f38f14a57deea6126810f1b9';assert sha(tool.parent/'rustc')=='49f9fa06f3e2b93aba85273433fef67b984ddb4cacb4500de56e85f70ec2e339'
for k in ['RUSTFLAGS','CARGO_ENCODED_RUSTFLAGS','RUSTC','RUSTC_WRAPPER','RUSTC_WORKSPACE_WRAPPER']:assert k not in os.environ
os.environ.update(PATH=str(tool.parent)+':/usr/bin:/bin',CARGO_INCREMENTAL='0',CARGO_BUILD_JOBS='2',RCH_SHIM_LOCAL_IDE='1',CARGO_TARGET_DIR=str(W/'target'),TMPDIR=str(W/'tmp'),XDG_CACHE_HOME=str(W/'cache'))
run('fmt',[tool,'fmt','--manifest-path',C/'Cargo.toml','--','--config-path',W/'tree/repo-view/rustfmt.toml'])
old={str(p.relative_to(S)):sha(p) for p in S.rglob('*') if p.is_file()};new={str(p.relative_to(B)):sha(p) for p in B.rglob('*') if p.is_file()};assert old.keys()==new.keys();changed=[p for p in old if old[p]!=new[p]];assert changed==['explainer_corpus/complex_analysis_visual/src/chapters/thumbnail.rs'],changed
(J/'source-copy.json').write_text(json.dumps(dict(files=new,changed=changed),indent=2))
f=['--release','--locked','--offline','--config',str(W/'tree/engine-budget.toml'),'--manifest-path',str(C/'Cargo.toml'),'--bin','complex_analysis_visual']
tests=run('tests',[tool,'test',*f]);counts=re.findall(r'^test result: ok\. (\d+) passed; 0 failed; 0 ignored; 0 measured; 0 filtered out;',tests,re.M);assert counts==['223'],counts
run('clippy',[tool,'clippy',*f,'--no-deps','--','-D','warnings']);run('build',[tool,'rustc',*f,'--','-C','metadata=jmev_thumbnail24'])
binary=W/'bin/thumbnail24';assert not binary.exists();shutil.copy2(W/'target/release/complex_analysis_visual',binary)
run('facts',[binary,'facts']);assert sha(J/'facts.stdout')=='7d528509752e532c07edb3fda7c12cd8251ac4ba77022355ae42ad725add4417'
run('chapters',[binary,'chapters']);assert (J/'chapters.stdout').read_bytes()==(R/'jobs/csd-production-repair-build19/chapters.stdout').read_bytes()
run('thumbnail',[binary,'thumbnail','--res','3840x2160','--out',J/'output'],cwd=W/'production-repo07')
image=J/'output/complex_analysis_visual_thumbnail.png'
from PIL import Image
with Image.open(image) as im:assert im.size==(3840,2160) and im.format=='PNG';im.verify()
assert new=={str(p.relative_to(B)):sha(p) for p in B.rglob('*') if p.is_file()}
(J/'thumbnail-result.json').write_text(json.dumps(dict(binary_sha256=sha(binary),image=str(image),image_sha256=sha(image),tests=223,clippy=True,only_changed_source=changed,pictures_viewed=False,scope='Separate native thumbnail candidate, no production movie or narration changed; actual visual review required.'),indent=2))
