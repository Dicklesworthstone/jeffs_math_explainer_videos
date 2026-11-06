#!/bin/bash
# Paired build and probe of the stroke x index, third version. Root runs one step at a time.
# probe02.sh stays sealed with its refused receipts; this one differs only in requiring the guard's
# CARGO_BUILD_JOBS=2 and keeping it (Mail 5887).
# The patch is unchanged and is read from complex-stroke-index-candidate01, this folder's sibling.
# Writes only under $WORK. Never touches the frozen source, the live engine or a running job.
#
#   prepare  copy the frozen source, rewrite its two absolute-path files, then run deps
#   deps     prove both builds' Linux dependencies are offline; rerunnable once caches are filled
#   base     build the baseline binary, unpatched
#   cand     apply the patch, run fmn-render's tests, build the candidate with the same flags;
#            rerunnable after a resource stop or a missing dependency
#   ab64     alternating 64x36 timing, then lossless identity of chapter 04 at 3 fps
#   ab4k     alternating 4K timing, then lossless identity of chapter 04 at 1 fps
#   film     optional, before adoption: lossless identity of every chapter at 64x36, 1 fps,
#            one binary at a time
#   profile  optional: what is hot in the candidate, sampling a run at 60 fps
#
# Env: WORK (the host's work folder), AB (absolute path of ab_bench.sh). prepare also needs SOURCE
# (the frozen folder holding engine/, repo-view/, engine-budget.toml), CRATE (the stage07
# complex_analysis_visual folder) and CONFIG (the repository's .cargo/config.toml). THREADS: 4K, 24.
set -euo pipefail
TOOLCHAIN=nightly-2026-08-31
TRIPLE=x86_64-unknown-linux-gnu
RUSTC_COMMIT=90850177249efe0321573c569aec5d12b257f8d6
CARGO_SHA=b854c9424332416075760b8b5fc8a43254e8eea8f38f14a57deea6126810f1b9
RUSTC_SHA=49f9fa06f3e2b93aba85273433fef67b984ddb4cacb4500de56e85f70ec2e339
HOME_CONFIG_SHA=14ae6b73244bde60c647383e9c4f6321fa575e7bb8ad75d6916a7adfefe0b4e6
CONFIG_SHA=79fb8766d1f41365d64066e5b2e68b0fb89c50015425d109b155ccaf8eff2716
STROKE_BASE=e0feeab4dfbc0ead02628dcd28a48b4724146ef068dbdccefbcefea2a6ed69a4
ENGINE_BASE=85bdc6b721cf40fb836eaa07d324138ced6d8edd0e3e7919e37379b62f764d8c
STROKE_CAND=668867dffefe1378f1e5f6631cc31c297103d6b197f724c7fdd210db685a0431
ENGINE_CAND=de6551fc9f697b4a529e60d4ef502ca99abce0b4b3c243c2e017658836f1f519
PATCH_SHA=da6b42b652cf4b6b5788e2246951310f7bd349af1b82bcb4e2fcdb256ecc95c5
AB_SHA=9a3e608442f17bc3ca879ffc49c40985de354837edf8dd7d3969f8f680a9ae69
CHAPTER=04_reading_color
NEW_TESTS=(the_x_index_is_bit_exact_on_a_plate_strip the_x_index_keeps_the_earliest_of_tied_segments
           the_x_index_refuses_what_it_cannot_prove the_x_index_leaves_non_finite_queries_to_the_scan
           the_x_index_shades_a_plate_exactly_as_the_scan)

# The real toolchain, then the system, and nothing else. ~/.local/bin is left out because its cargo
# wrapper sources the storage environment and execs the RCH shim; ~/.cargo/bin because its proxies
# are not the pinned bytes.
TOOL_CARGO="$("$HOME/.cargo/bin/rustup" which --toolchain "$TOOLCHAIN" cargo)"
TOOL_BIN="${TOOL_CARGO%/cargo}"
export PATH="$TOOL_BIN:/usr/bin:/bin"

HERE="$(cd "$(dirname "$0")" && pwd)"
PATCH="$HERE/../complex-stroke-index-candidate01/stroke-index.patch"
STEP="${1:?usage: probe03.sh prepare|deps|base|cand|ab64|ab4k|film|profile}"
: "${WORK:?WORK is required}"
TREE="$WORK/tree"
RENDER_SRC="$TREE/engine/crates/fmn-render/src"
LOGS="$WORK/logs"
BASE="$WORK/bin/base"
CAND="$WORK/bin/cand"
export CARGO_INCREMENTAL=0 RCH_SHIM_LOCAL_IDE=1 CARGO_TARGET_DIR="$WORK/target"
# The render's work folder and typeset cache would otherwise land in /tmp and ~/.cache, unseen by the guard.
export TMPDIR="$WORK/tmp" XDG_CACHE_HOME="$WORK/cache"
FLAGS=(--release --locked --offline --config "$TREE/engine-budget.toml"
       --manifest-path "$TREE/crate/Cargo.toml" --bin complex_analysis_visual)

die() { echo "probe03 refused: $*" >&2; exit 1; }
sum() { python3 -I -B -c 'import hashlib, sys; print(hashlib.sha256(open(sys.argv[1], "rb").read()).hexdigest())' "$1"; }
pin() { [ "$(sum "$1")" = "$2" ] || die "hash differs: $1"; }

gates() {
    case "$TOOL_BIN" in
        */toolchains/"$TOOLCHAIN"-"$TRIPLE"/bin) ;;
        *) die "cargo does not resolve inside the pinned toolchain" ;;
    esac
    [ "$(command -v cargo)" = "$TOOL_BIN/cargo" ] && [ "$(command -v rustc)" = "$TOOL_BIN/rustc" ] \
        || die "cargo or rustc on PATH is not the toolchain's"
    pin "$TOOL_BIN/cargo" "$CARGO_SHA"
    pin "$TOOL_BIN/rustc" "$RUSTC_SHA"
    case "$(rustc -Vv)" in
        *"commit-hash: $RUSTC_COMMIT"*) ;;
        *) die "rustc is not the pinned build" ;;
    esac
    # This file's build.target-dir loses to CARGO_TARGET_DIR, and its profiles are dev and test only.
    [ ! -e "$HOME/.cargo/config" ] || die "a second home cargo config exists"
    [ ! -e "$HOME/.cargo/config.toml" ] || pin "$HOME/.cargo/config.toml" "$HOME_CONFIG_SHA"
    # The guard's job limit: required, kept, never raised.
    [ "${CARGO_BUILD_JOBS:-}" = 2 ] || die "CARGO_BUILD_JOBS must be the guard's 2, not '${CARGO_BUILD_JOBS:-unset}'"
    for name in $(compgen -e); do
        case "$name" in
            CARGO_BUILD_JOBS) ;;
            RUSTFLAGS|CARGO_ENCODED_RUSTFLAGS|RUSTC|RUSTC_WRAPPER|RUSTC_WORKSPACE_WRAPPER|CARGO_HOME|CARGO_BUILD_*|CARGO_PROFILE_*)
                die "$name is set, so the build could differ from production" ;;
        esac
    done
}

deps() {
    [ -e "$TREE/crate/Cargo.toml" ] || die "run prepare first"
    # Filtered to this platform: unfiltered, cargo metadata wants every platform's crates offline.
    (cd "$WORK" && cargo metadata --locked --offline --format-version 1 --filter-platform "$TRIPLE" \
        --config "$TREE/engine-budget.toml" --manifest-path "$TREE/crate/Cargo.toml") > "$LOGS/metadata-film.json"
    (cd "$TREE/engine" && cargo metadata --locked --offline --format-version 1 --filter-platform "$TRIPLE") \
        > "$LOGS/metadata-engine.json"
    echo "deps ok"
}

prepare() {
    : "${SOURCE:?SOURCE is required}" "${CRATE:?CRATE is required}" "${CONFIG:?CONFIG is required}"
    [ ! -e "$TREE" ] || die "$TREE already exists"
    pin "$PATCH" "$PATCH_SHA"
    pin "$CONFIG" "$CONFIG_SHA"
    pin "$SOURCE/engine/crates/fmn-render/src/stroke.rs" "$STROKE_BASE"
    pin "$SOURCE/engine/crates/fmn-render/src/engine.rs" "$ENGINE_BASE"
    prefix="$(sed -n 's#^fmn = { path = "\(.*\)/engine/crates/fmn" }$#\1#p' "$SOURCE/engine-budget.toml")"
    [ -n "$prefix" ] || die "no engine path in engine-budget.toml"
    mkdir -p "$TREE" "$WORK/.cargo" "$WORK/bin" "$LOGS"
    cp -R "$SOURCE/engine" "$SOURCE/repo-view" "$TREE/"
    cp -R "$CRATE" "$TREE/crate"
    chmod -R u+w "$TREE"
    cp "$CONFIG" "$WORK/.cargo/config.toml"
    sed "s#$prefix#$TREE#g" "$SOURCE/engine-budget.toml" > "$TREE/engine-budget.toml"
    sed -i.frozen "s#$prefix#$TREE#g" "$TREE/crate/Cargo.toml"
    if grep -q "$prefix" "$TREE/engine-budget.toml" "$TREE/crate/Cargo.toml"; then die "a frozen path survived"; fi
    (cd "$TREE/engine" && patch -p1 --dry-run < "$PATCH") > "$LOGS/patch-dry-run.log"
    echo "copied: $TREE"
    deps
    echo "prepare ok: $TREE"
}

build() {
    side="$1"
    bin="$WORK/bin/$side"
    [ ! -e "$bin" ] || die "$bin already exists"
    (cd "$WORK" && cargo build "${FLAGS[@]}") 2>&1 | tee "$LOGS/build-$side.log"
    [ -f "$CARGO_TARGET_DIR/release/complex_analysis_visual" ] || die "the build did not land in $CARGO_TARGET_DIR"
    cp "$CARGO_TARGET_DIR/release/complex_analysis_visual" "$bin"
    sum "$bin" > "$bin.sha256"
    echo "$side binary $(cat "$bin.sha256")"
}

base() {
    pin "$RENDER_SRC/stroke.rs" "$STROKE_BASE"
    pin "$RENDER_SRC/engine.rs" "$ENGINE_BASE"
    build base
}

apply_patch() {
    if [ "$(sum "$RENDER_SRC/stroke.rs")" = "$STROKE_CAND" ] && [ "$(sum "$RENDER_SRC/engine.rs")" = "$ENGINE_CAND" ]; then
        echo "patch already applied"
        return
    fi
    pin "$PATCH" "$PATCH_SHA"
    pin "$RENDER_SRC/stroke.rs" "$STROKE_BASE"
    pin "$RENDER_SRC/engine.rs" "$ENGINE_BASE"
    (cd "$TREE/engine" && patch -p1 --no-backup-if-mismatch < "$PATCH")
    pin "$RENDER_SRC/stroke.rs" "$STROKE_CAND"
    pin "$RENDER_SRC/engine.rs" "$ENGINE_CAND"
}

cand() {
    [ -x "$BASE" ] || die "build base first"
    [ ! -e "$CAND" ] || die "$CAND already exists"
    apply_patch
    (cd "$TREE/engine" && CARGO_TARGET_DIR="$WORK/target-test" cargo test --locked --offline -p fmn-render --lib) \
        2>&1 | tee "$LOGS/tests-cand.log"
    [ -d "$WORK/target-test/debug" ] || die "the tests did not build in $WORK/target-test"
    for name in "${NEW_TESTS[@]}"; do
        grep -q "$name \.\.\. ok" "$LOGS/tests-cand.log" || die "test did not pass: $name"
    done
    build cand
    if cmp -s "$BASE" "$CAND"; then die "the candidate equals the baseline, so the probe would compare nothing"; fi
}

identity_files() {
    if cmp -s "$1/base.identity/$CHAPTER.mp4" "$1/cand.identity/$CHAPTER.mp4"; then
        echo "encoded $CHAPTER.mp4 files are also byte-identical"
    else
        echo "encoded $CHAPTER.mp4 files differ; the decoded frame hashes above are the test"
    fi
}

ab() {
    name="$1"; shift
    [ -x "$BASE" ] && [ -x "$CAND" ] || die "build both binaries first"
    pin "${AB:?AB is required}" "$AB_SHA"
    identity="$1"; shift
    (cd "$WORK" && ROUNDS=3 WARMUP=1 IDENTITY_FILE="$CHAPTER.mp4" IDENTITY_ARGS="$identity" \
        timeout 21600 bash "$AB" "$BASE" "$CAND" "$WORK/$name" -- "$@") 2>&1 | tee "$LOGS/$name.log"
    grep -qx "IDENTITY PASS" "$LOGS/$name.log" || die "no IDENTITY PASS in $name"
    identity_files "$WORK/$name" | tee -a "$LOGS/$name.log"
}

ab64() {
    ab ab64 "render $CHAPTER --res 64x36 --fps 3 --silent --encoder libx264 --crf 0 --preset ultrafast" \
        render "$CHAPTER" --res 64x36 --fps 3 --silent --crf 23 --max-frames 120
}

ab4k() {
    threads="${THREADS:-24}"
    ab ab4k "render $CHAPTER --res 3840x2160 --fps 1 --silent --encoder libx264 --crf 0 --preset ultrafast --threads $threads" \
        render "$CHAPTER" --res 3840x2160 --fps 3 --silent --crf 18 --preset veryfast --threads "$threads" --max-frames 60
}

film() {
    out="$WORK/film"
    [ -x "$BASE" ] && [ -x "$CAND" ] || die "build both binaries first"
    [ ! -e "$out" ] || die "$out already exists"
    mkdir -p "$out"
    for side in base cand; do
        (cd "$WORK" && timeout 43200 "$WORK/bin/$side" render all --res 64x36 --fps 1 --silent --encoder libx264 \
            --crf 0 --preset ultrafast --out "$out/$side") > "$LOGS/film-$side.log" 2>&1 \
            || die "the $side film render failed; see $LOGS"
    done
    count=0
    for chapter in "$out"/base/*.mp4; do
        name="$(basename "$chapter")"
        ffmpeg -nostdin -v error -i "$chapter" -map 0:v -f framemd5 "$out/$name.base.framemd5"
        ffmpeg -nostdin -v error -i "$out/cand/$name" -map 0:v -f framemd5 "$out/$name.cand.framemd5"
        [ -s "$out/$name.base.framemd5" ] || die "empty decode: $name"
        cmp -s "$out/$name.base.framemd5" "$out/$name.cand.framemd5" || die "IDENTITY FAIL: $name"
        count=$((count + 1))
    done
    [ "$count" -eq 15 ] || die "expected 15 chapters, compared $count"
    echo "FILM IDENTITY PASS: $count chapters" | tee "$LOGS/film.log"
}

profile() {
    [ -x "$CAND" ] || die "build the candidate first"
    data="$WORK/candidate.perf.data"
    [ ! -e "$data" ] || die "$data already exists"
    (cd "$WORK" && timeout 3600 perf record -F 99 -o "$data" -- "$CAND" render "$CHAPTER" --res 64x36 --fps 60 --silent \
        --crf 23 --max-frames 1200 --out "$WORK/profile-out") > "$LOGS/profile-run.log" 2>&1 || true
    perf report --stdio --no-children --sort symbol -i "$data" > "$LOGS/profile-full.txt" 2>/dev/null
    sed -n '1,60p' "$LOGS/profile-full.txt" | tee "$LOGS/profile.txt"
}

mkdir -p "$TMPDIR" "$XDG_CACHE_HOME"
case "$STEP" in
    prepare|deps|base|cand) gates; "$STEP" ;;
    ab64|ab4k|film|profile) "$STEP" ;;
    *) die "unknown step $STEP" ;;
esac
