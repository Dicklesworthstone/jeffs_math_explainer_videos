"""Build exact retained apportionment sources on CSD; no render or release acceptance."""
from pathlib import Path
import os,json,hashlib,subprocess,re,tomllib,shutil
D=Path('/data/projects/jmev-stroke-probe-20261106');Q=D/'qualification01/production-repair17';J=D/'jobs/csd-apportionment-build656';W=D/'work';B=J/'workspace';E=J/'engine'
OLD='/Users/jemanuel/projects/jeffs_math_explainer_videos/'
ENG='/Users/jemanuel/.cargo/git/checkouts/franken_manim-6641d0e09642f63e/a399d62/'
def sha(p):
 with Path(p).open('rb')as f:return hashlib.file_digest(f,'sha256').hexdigest()
def read(p):return json.loads(Path(p).read_text())
def success(p):
 d=read(p);assert d.get('admitted') and d.get('finished') and d.get('exit')==0 and not any(d.get(k)for k in ['error','stopped_for_resource_limit','resource_stop']);assert d['inputs_sha256']==d['final_inputs_sha256']
def run(name,args,cwd):
 with (J/(name+'.stdout')).open('xb')as o,(J/(name+'.stderr')).open('xb')as e:
  r=subprocess.run(list(map(str,args)),cwd=cwd,stdout=o,stderr=e,timeout=7200)
 assert r.returncode==0,(name,r.returncode)
 return (J/(name+'.stdout')).read_text()
a=read(J/'resource-receipt.json');assert a.get('admitted') and not a.get('finished');assert a['policy']==dict(admission_gib=10,stop_free_gib=8,phase_growth_gib=4,work_cap_gib=56,target_growth_cap_gib=20,sample_seconds=10)
for r in read(Q/'csd-apportionment-build656-inputs.json')['files']:assert sha(r['path'])==r['sha256']
assert not B.exists() and not E.exists();B.mkdir();E.mkdir();allrows={}
for version in ['652','655']:
 p=D/('jobs/csd-apportionment-import'+version+'-a1');success(p/'resource-receipt.json')
 for row in read(p/'import-result.json')['files']:
  assert sha(row['physical'])==row['sha256'];assert Path(row['physical']).stat().st_size==row['bytes']
  if row['path']in allrows:assert allrows[row['path']]['sha256']==row['sha256']
  allrows[row['path']]=row
for name,row in allrows.items():
 out=None
 if name.startswith(ENG):out=E/name.removeprefix(ENG)
 elif name.startswith(OLD):
  rel=name.removeprefix(OLD)
  if rel.startswith(('explainer_corpus/apportionment/','crates/explainer_kit/','crates/explainer_score/')) or rel in ['Cargo.toml','rust-toolchain.toml','rustfmt.toml','LICENSE','.cargo/config.toml']:out=B/rel
 if out:
  out.parent.mkdir(parents=True,exist_ok=True)
  # Copy, not hardlink, so Cargo cannot mutate retained inputs.
  with Path(row['physical']).open('rb')as src,out.open('xb')as dst:shutil.copyfileobj(src,dst)
root=B/'Cargo.toml';s=root.read_text();assert s.count('members = ["crates/*", "explainer_corpus/*"]')==1;root.write_text(s.replace('members = ["crates/*", "explainer_corpus/*"]','members = ["crates/*"]'))
C=B/'explainer_corpus/apportionment';lock=C/'Cargo.lock';original=lock.read_text();old=tomllib.loads(original)
engine=[p for p in old['package']if p.get('source','').startswith('git+https://github.com/Dicklesworthstone/franken_manim?')];assert len(engine)==19
new=re.sub(r'^source = "git\+https://github.com/Dicklesworthstone/franken_manim\?rev=a399d626c43ecc852a5c795288979f52b446bc5c#a399d626c43ecc852a5c795288979f52b446bc5c"\n','',original,flags=re.M)
expected=json.loads(json.dumps(old))
for p in expected['package']:
 if p.get('source','').startswith('git+https://github.com/Dicklesworthstone/franken_manim?'):del p['source']
assert tomllib.loads(new)==expected;lock.write_text(new)
config=J/'pristine-engine.toml'
with config.open('x')as f:
 f.write('[patch."https://github.com/Dicklesworthstone/franken_manim"]\n')
 for p in engine:
  path=E/'crates'/p['name'];assert (path/'Cargo.toml').is_file();f.write(p['name']+' = { path = '+json.dumps(str(path))+' }\n')
source={str(p.relative_to(J)):sha(p)for base in [B,E]for p in base.rglob('*')if p.is_file()}
tool=Path(subprocess.check_output([str(Path.home()/'.cargo/bin/rustup'),'which','--toolchain','nightly-2026-08-31','cargo'],text=True).strip());assert sha(tool)=='b854c9424332416075760b8b5fc8a43254e8eea8f38f14a57deea6126810f1b9'
assert sha(tool.parent/'rustc')=='49f9fa06f3e2b93aba85273433fef67b984ddb4cacb4500de56e85f70ec2e339'
os.sched_setaffinity(0,{0,1,2,3})
for k in ['RUSTFLAGS','CARGO_ENCODED_RUSTFLAGS','RUSTC','RUSTC_WRAPPER','RUSTC_WORKSPACE_WRAPPER']:assert k not in os.environ
os.environ.update(PATH=str(tool.parent)+':/usr/bin:/bin',CARGO_INCREMENTAL='0',CARGO_BUILD_JOBS='2',RCH_SHIM_LOCAL_IDE='1',CARGO_TARGET_DIR=str(W/'target'),TMPDIR=str(W/'tmp'),XDG_CACHE_HOME=str(W/'cache'))
assert not (W/'target/release/apportionment').exists(),'Preserve any prior apportionment binary before using target'
f=['--release','--locked','--offline','--config',config,'--manifest-path',C/'Cargo.toml','--bin','apportionment']
tests=run('tests',[tool,'test',*f],C);assert re.search(r'test result: ok\. 30 passed; 0 failed',tests),tests[-1500:]
run('clippy',[tool,'clippy',*f,'--no-deps','--','-D','warnings'],C);run('build',[tool,'build',*f],C)
binary=J/'apportionment-native656';shutil.copy2(W/'target/release/apportionment',binary)
facts=run('facts',[binary,'facts'],C);oracle=run('oracle',['/usr/bin/python3','-I','-B',C/'oracle.py'],C)
def rows(s):
 p=[x.split('\t',1)for x in s.splitlines()if x.strip()];d=dict(p);assert len(d)==len(p);return d
assert len(rows(facts))==443 and rows(facts)==rows(oracle)
chapters=run('chapters',[binary,'chapters'],C).splitlines();assert len(chapters)==15
assert source=={str(p.relative_to(J)):sha(p)for base in [B,E]for p in base.rglob('*')if p.is_file()}
with (J/'qualification.json').open('x')as f:json.dump(dict(binary_sha256=sha(binary),source=source,tests=30,facts=443,chapters=chapters,scope='Build only; timeline equivalence, rendering and all AV/release review remain open.'),f,indent=2)
print('BUILD656 qualified source and facts only',flush=True)
