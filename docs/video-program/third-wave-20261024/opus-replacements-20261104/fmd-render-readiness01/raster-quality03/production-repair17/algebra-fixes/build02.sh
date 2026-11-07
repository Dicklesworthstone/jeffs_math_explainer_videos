#!/bin/sh
# Guarded algebra-fixes build02. Root's corner-label-contrast-repair01/build01.sh with a new output
# folder inside measured WORK, plus checks frozen from that build's receipt: the same lock check, fmt, tests, release
# build and Clippy -D warnings, then every expected test passed, and facts and chapters
# byte-identical to root's build. Run only through root's original guard after
# prepare_build02.py. Pass exactly WORK/algebra-fixes-inputs02 as argument one.
# All outputs are inside measured WORK or its measured shared Cargo target.
set -eu
export CARGO_INCREMENTAL=0
export RCH_SHIM_LOCAL_IDE=1
export CARGO_TARGET_DIR="$CARGO_TARGET_DIR/geometry-kit01"
HERE=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd -P)
REPO=$(CDPATH= cd -- "$HERE/../../../../../../../.." && pwd -P)
test -f "$REPO/AGENTS.md"
test "$(pwd -P)" = "$REPO"
test "$#" -eq 1
WORK="$REPO/narration_lab/third-wave-20261024/root/five-film-release-20261027"
test ! -L "$WORK"
test ! -L "$1"
O=$(CDPATH= cd -- "$1" && pwd -P)
test "$O" = "$WORK/algebra-fixes-inputs02"
OLD="$REPO/narration_lab/third-wave-20261024/root/elevenlabs-expanded-algebra-20261103/native-stage08-root01/corner-label-contrast-repair01"
C="$O/workspace/explainer_corpus/abstract_algebra_expanded"
M="$C/Cargo.toml"
test -f "$O/build02-inputs.json"
test -f "$O/expected-tests.txt"
. "$O/build.env"
test -f "$BUDGET"
test ! -e "$O/abstract_algebra_expanded"
test ! -e "$O/test01.log"
if test -f "$CARGO_TARGET_DIR/release/abstract_algebra_expanded"; then
  test ! -e "$O/before-build02-abstract_algebra_expanded-release"
  cp -p "$CARGO_TARGET_DIR/release/abstract_algebra_expanded" "$O/before-build02-abstract_algebra_expanded-release"
fi
python3 -I -B docs/video-program/third-wave-20261024/opus-replacements-20261104/engine-geometry-validation02/verify_patched_lock.py "$OLD/Cargo.lock.before-engine-patch" "$C/Cargo.lock"
cargo fmt --manifest-path "$M" -- --check
cargo test --release --locked --offline --config "$BUDGET" --manifest-path "$M" --bin abstract_algebra_expanded > "$O/test01.log" 2>&1 || { cat "$O/test01.log"; exit 1; }
cat "$O/test01.log"
python3 -I -B "$HERE/expect_build01.py" tests "$O/expected-tests.txt" "$O/test01.log"
cargo build --release --locked --offline --config "$BUDGET" --manifest-path "$M" --bin abstract_algebra_expanded
cargo clippy --release --locked --offline --config "$BUDGET" --manifest-path "$M" --bin abstract_algebra_expanded --no-deps -- -D warnings
cp -p "$CARGO_TARGET_DIR/release/abstract_algebra_expanded" "$O/abstract_algebra_expanded"
"$O/abstract_algebra_expanded" facts > "$O/native-facts01.tsv"
python3 -I -B "$C/oracle.py" > "$O/oracle-facts01.tsv"
python3 -I -B "$HERE/expect_build01.py" facts "$O/native-facts01.tsv" "$O/oracle-facts01.tsv" "$OLD/native-facts01.tsv" "$OLD/oracle-facts01.tsv"
"$O/abstract_algebra_expanded" chapters > "$O/native-chapters01.txt"
cmp "$O/native-chapters01.txt" "$OLD/native-chapters01.txt"
echo 'Algebra fixes build finished; the 11_17 Frederick take and the 11_17 and 14_14 pictures remain required.'
