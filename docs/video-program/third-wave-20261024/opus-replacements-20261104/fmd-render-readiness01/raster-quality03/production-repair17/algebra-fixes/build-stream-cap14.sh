#!/bin/sh
set -eu
export CARGO_INCREMENTAL=0
export RCH_SHIM_LOCAL_IDE=1
export CARGO_TARGET_DIR="$CARGO_TARGET_DIR/geometry-kit01"
O="/Users/jemanuel/projects/jeffs_math_explainer_videos/narration_lab/third-wave-20261024/root/five-film-release-20261027/algebra-stream-cap14-inputs"
M="/Users/jemanuel/projects/jeffs_math_explainer_videos/narration_lab/third-wave-20261024/root/five-film-release-20261027/algebra-stream-cap12-inputs/workspace/explainer_corpus/abstract_algebra_expanded/Cargo.toml"
BUDGET=/Users/jemanuel/scratch/jmev-geometry-native-20261104-01/engine-budget.toml
cargo test --release --locked --offline --config "$BUDGET" --manifest-path "$M" --bin abstract_algebra_expanded > "$O/test.log" 2>&1
cargo rustc --release --locked --offline --config "$BUDGET" --manifest-path "$M" --bin abstract_algebra_expanded -- -C metadata=jmev_algebra_stream_cap14 > "$O/build.log" 2>&1
cargo clippy --release --locked --offline --config "$BUDGET" --manifest-path "$M" --bin abstract_algebra_expanded --no-deps -- -D warnings > "$O/clippy.log" 2>&1
cp -p "$CARGO_TARGET_DIR/release/abstract_algebra_expanded" "$O/binary"
"$O/binary" facts > "$O/facts.tsv"
"$O/binary" chapters > "$O/chapters.txt"
cmp "$O/facts.tsv" "/Users/jemanuel/projects/jeffs_math_explainer_videos/narration_lab/third-wave-20261024/root/five-film-release-20261027/algebra-fixes-inputs02/native-facts01.tsv"
cmp "$O/chapters.txt" "/Users/jemanuel/projects/jeffs_math_explainer_videos/narration_lab/third-wave-20261024/root/five-film-release-20261027/algebra-fixes-inputs02/native-chapters01.txt"
