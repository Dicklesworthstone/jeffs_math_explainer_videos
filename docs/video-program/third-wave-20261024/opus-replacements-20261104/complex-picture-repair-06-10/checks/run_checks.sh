#!/usr/bin/env bash
# Everything source-level the chapter 06-10 repair can show without a build. Nothing here
# compiles, renders or synthesizes, and nothing writes outside EMPTY_SCRATCH.
#
#   bash docs/video-program/third-wave-20261024/opus-replacements-20261104/complex-picture-repair-06-10/checks/run_checks.sh EMPTY_SCRATCH
#
# Run from the repo root. EMPTY_SCRATCH must be an empty or new directory.
set -eu

SCRATCH=${1:?an empty scratch directory}
mkdir -p "$SCRATCH"
if [ -n "$(/bin/ls -A "$SCRATCH")" ]; then
    echo "not empty: $SCRATCH" >&2
    exit 2
fi

HERE=docs/video-program/third-wave-20261024/opus-replacements-20261104/complex-picture-repair-06-10
STAGED=$HERE/explainer_corpus/complex_analysis_visual
CANON=explainer_corpus/complex_analysis_visual

digest() {
    find "$HERE" -type f | LC_ALL=C sort | xargs shasum -a 256 | shasum -a 256 | cut -c1-16
}
BEFORE=$(digest)

echo "== canonical against the baselines"
shasum -a 256 -c "$HERE/checks/baseline-hashes.sha256"

echo "== static checks of the staged sources"
python3 -I -B "$HERE/checks/source_checks.py"

echo "== numbers and label boxes"
python3 -I -B "$HERE/checks/geometry_checks.py"

echo "== patches rebuild the staged tree from canonical, with no fuzz"
RT=$SCRATCH/roundtrip
mkdir -p "$RT"
(cd "$CANON/../.." && tar cf - "$CANON/src/chapters" "$CANON/docs/storyboard.md") | (cd "$RT" && tar xf -)
PATCHES=$(pwd)/$HERE/patches
for p in "$PATCHES"/*.patch; do
    (cd "$RT" && patch -p1 --no-backup-if-mismatch --fuzz=0 -s -i "$p")
done
for f in one_derivative one_point loop_sums boundary rigidity draw mod; do
    cmp "$RT/$CANON/src/chapters/$f.rs" "$STAGED/src/chapters/$f.rs"
done
cmp "$RT/$CANON/docs/storyboard.md" "$STAGED/docs/storyboard.md"
echo "rebuilt byte for byte"

echo "== rustfmt parses and passes every module, staged files overlaid on a copy of the crate"
FMT=$SCRATCH/fmt
mkdir -p "$FMT/crate"
(cd "$CANON" && tar cf - src script.tsv docs/storyboard.md) | (cd "$FMT/crate" && tar xf -)
for f in one_derivative one_point loop_sums boundary rigidity draw mod; do
    cp "$STAGED/src/chapters/$f.rs" "$FMT/crate/src/chapters/$f.rs"
done
rustfmt --edition 2024 --config max_width=120 --check "$FMT/crate/src/main.rs"
echo "rustfmt clean"

echo "== storyboard: picture cells only, the same gate problems as canonical, in isolated roots"
python3 -I -B "$HERE/checks/storyboard_compare.py" "$SCRATCH/storyboard"

AFTER=$(digest)
if [ "$BEFORE" != "$AFTER" ]; then
    echo "the repair folder changed during the checks" >&2
    exit 1
fi
echo "repair folder unchanged: $AFTER"
echo "ALL CHECKS PASSED. This is not a build: no Rust was compiled and no test ran."
