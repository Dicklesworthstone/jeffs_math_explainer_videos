#!/bin/sh
# Native gate for the integrated stage06 candidate. Root runs this, inside its guarded lane, from the
# repo root, with CARGO_TARGET_DIR set. Modelled on combined-build-repair02/build.sh.
# NOT RUN: no Cargo command here has been executed by the author. It compiles; it renders nothing.
set -eu
: "${CARGO_TARGET_DIR:?set CARGO_TARGET_DIR to the target folder of the guarded lane}"
export CARGO_INCREMENTAL=0
HERE=docs/video-program/third-wave-20261024/opus-replacements-20261104/logarithm-stage06-preflight01
CRATE=explainer_corpus/logarithm_mysteries
python3 -I -B "$HERE/preflight.py" --mode canonical --quiet
python3 -I -B "$CRATE/docs/check_cues.py"
cargo test --locked --offline --manifest-path "$CRATE/Cargo.toml"
cargo build --locked --offline --manifest-path "$CRATE/Cargo.toml"
cargo clippy --locked --offline --manifest-path "$CRATE/Cargo.toml" --all-targets --no-deps -- -D warnings
python3 -I -B "$HERE/post_build_check.py" --binary "$CARGO_TARGET_DIR/debug/logarithm_mysteries"
