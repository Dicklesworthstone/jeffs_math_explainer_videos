import json,os,subprocess,hashlib
from pathlib import Path
R=Path('/data/projects/jmev-fmd-render-qualification-20261106-1111')
J=R/'jobs/fmd-raster-fetch01'
a=json.loads((J/'resource-receipt.json').read_text());assert a['admitted'] and 'exit' not in a
cargo=Path(subprocess.check_output([str(Path.home()/'.cargo/bin/rustup'),'which','--toolchain','nightly-2026-08-31','cargo'],text=True).strip())
assert hashlib.sha256(cargo.read_bytes()).hexdigest()=='b854c9424332416075760b8b5fc8a43254e8eea8f38f14a57deea6126810f1b9'
os.environ.update(PATH=str(cargo.parent)+':/usr/bin:/bin',CARGO_INCREMENTAL='0',CARGO_TARGET_DIR=str(R/'work/target'),TMPDIR=str(R/'work/tmp'))
with (J/'fetch.stdout').open('xb') as out,(J/'fetch.stderr').open('xb') as err:
 subprocess.run([str(cargo),'fetch','--locked','--config',str(R/'work/tree/engine-budget.toml'),'--manifest-path',str(R/'work/picture-repair03/workspace/explainer_corpus/complex_analysis_visual/Cargo.toml')],stdout=out,stderr=err,check=True,timeout=1200)
print('Locked fetch completed; build/tests still required')
