#!/bin/sh
set -eu
export CARGO_INCREMENTAL=0
python3 -I -B - <<'SAVE'
from pathlib import Path
import os,shutil
old=Path(os.environ['CARGO_TARGET_DIR'])/'debug/apportionment'
if old.exists():
 with old.open('rb') as src,Path('/Users/jemanuel/projects/jeffs_math_explainer_videos/narration_lab/third-wave-20261024/root/five-film-release-20261027/apportionment-picture-build564/before-build-apportionment').open('xb') as dst:shutil.copyfileobj(src,dst)
SAVE
cargo test --locked --offline --manifest-path explainer_corpus/apportionment/Cargo.toml
cargo build --locked --offline --manifest-path explainer_corpus/apportionment/Cargo.toml
cargo clippy --locked --offline --manifest-path explainer_corpus/apportionment/Cargo.toml --all-targets --no-deps -- -D warnings
python3 -I -B - <<'CHECK'
import subprocess, os
from pathlib import Path
binary = Path(os.environ['CARGO_TARGET_DIR']) / 'debug/apportionment'
def rows(text):
    pairs = [line.split('\t', 1) for line in text.splitlines() if line.strip()]
    result = dict(pairs)
    assert len(result) == len(pairs), 'duplicate fact keys'
    return result
rust = rows(subprocess.check_output([str(binary), 'facts'], text=True))
oracle = rows(subprocess.check_output(['python3', '-I', '-B', 'explainer_corpus/apportionment/oracle.py'], text=True))
assert len(rust) == 443, len(rust)
assert rust == oracle, {'rust': rust, 'oracle': oracle}
print('Native Rust and Python oracle agree:', len(rust), 'facts')
subprocess.run([str(binary), 'chapters'], check=True)
CHECK

python3 -I -B /Users/jemanuel/projects/jeffs_math_explainer_videos/narration_lab/third-wave-20261024/root/elevenlabs-apportionment-rebuild-20261104/frederick-generation89/cue02_3-repair135/validate-plan135.py /Users/jemanuel/projects/jeffs_math_explainer_videos /Users/jemanuel/projects/jeffs_math_explainer_videos/explainer_corpus/apportionment /Users/jemanuel/projects/jeffs_math_explainer_videos/narration_lab/third-wave-20261024/root/five-film-release-20261027/apportionment-picture-build564/native-plan-validation

python3 -I -B - <<'SAVE'
from pathlib import Path
import os,shutil,hashlib,json
p=Path(os.environ['CARGO_TARGET_DIR'])/'debug/apportionment';out=Path('/Users/jemanuel/projects/jeffs_math_explainer_videos/narration_lab/third-wave-20261024/root/five-film-release-20261027/apportionment-picture-build564/apportionment-native564')
with p.open('rb')as src,out.open('xb')as dst:shutil.copyfileobj(src,dst)
out.chmod(0o700)
with Path('/Users/jemanuel/projects/jeffs_math_explainer_videos/narration_lab/third-wave-20261024/root/five-film-release-20261027/apportionment-picture-build564/binary564.json').open('x')as f:json.dump(dict(path=str(out),sha256=hashlib.sha256(out.read_bytes()).hexdigest()),f,indent=2)
SAVE
