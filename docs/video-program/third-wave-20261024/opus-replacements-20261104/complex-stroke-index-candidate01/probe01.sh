#!/bin/bash
# Paired build and probe of the stroke x index. Root runs one step at a time on the idle host.
# Writes only under $WORK. Never touches the frozen source, the live engine or a running job.
#
#   prepare  copy the frozen source, rewrite its two absolute-path files, prove the deps are offline
#   base     build the baseline binary, unpatched
#   cand     apply the patch, run fmn-render's tests, build the candidate with the same flags
#   ab64     alternating 64x36 timing, then lossless identity of chapter 04 at 3 fps
#   ab4k     alternating 4K timing, then lossless identity of chapter 04 at 1 fps
#   film     optional, before adoption: lossless identity of every chapter at 64x36, 1 fps
#   profile  optional: what is hot in the candidate, sampling a run at 60 fps
#
# Env: WORK (new folder on the host's data disk), AB (ab_bench.sh). prepare also needs SOURCE (the
# frozen folder holding engine/, repo-view/, engine-budget.toml), CRATE (the stage07
# complex_analysis_visual folder) and CONFIG (the repository's .cargo/config.toml). THREADS: 4K, 24.
set -euo pipefail
export PATH="$HOME/.local/bin:$HOME/.cargo/bin:/usr/bin:/bin"
TOOLCHAIN=nightly-2026-08-31
RUSTC_COMMIT=90850177249efe0321573c569aec5d12b257f8d6
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

HERE="$(cd "$(dirname "$0")" && pwd)"
STEP="${1:?usage: probe01.sh prepare|base|cand|ab64|ab4k|film|profile}"
: "${WORK:?WORK is required}"
TREE="$WORK/tree"
RENDER_SRC="$TREE/engine/crates/fmn-render/src"
LOGS="$WORK/logs"
BASE="$WORK/bin/base"
CAND="$WORK/bin/cand"
export CARGO_INCREMENTAL=0 RCH_SHIM_LOCAL_IDE=1 CARGO_TARGET_DIR="$WORK/target"
FLAGS=(--release --locked --offline --config "$TREE/engine-budget.toml"
       --manifest-path "$TREE/crate/Cargo.toml" --bin complex_analysis_visual)

die() { echo "probe01 refused: $*" >&2; exit 1; }
cargo_() { rustup run "$TOOLCHAIN" cargo "$@"; }
sum() { python3 -I -B -c 'import hashlib, sys; print(hashlib.sha256(open(sys.argv[1], "rb").read()).hexdigest())' "$1"; }
pin() { [ "$(sum "$1")" = "$2" ] || die "hash differs: $1"; }

prepare() {
    : "${SOURCE:?SOURCE is required}" "${CRATE:?CRATE is required}" "${CONFIG:?CONFIG is required}"
    [ ! -e "$TREE" ] || die "$TREE already exists"
    pin "$HERE/stroke-index.patch" "$PATCH_SHA"
    pin "$SOURCE/engine/crates/fmn-render/src/stroke.rs" "$STROKE_BASE"
    pin "$SOURCE/engine/crates/fmn-render/src/engine.rs" "$ENGINE_BASE"
    rustup run "$TOOLCHAIN" rustc -Vv | grep -qx "commit-hash: $RUSTC_COMMIT" || die "rustc is not the pinned build"
    case "$(rustup run "$TOOLCHAIN" sh -c 'command -v cargo')" in
        */toolchains/"$TOOLCHAIN"-*/bin/cargo) ;;
        *) die "cargo does not resolve inside the pinned toolchain" ;;
    esac
    prefix="$(sed -n 's#^fmn = { path = "\(.*\)/engine/crates/fmn" }$#\1#p' "$SOURCE/engine-budget.toml")"
    [ -n "$prefix" ] || die "no engine path in engine-budget.toml"
    mkdir -p "$TREE" "$WORK/.cargo" "$WORK/bin" "$LOGS"
    cp -R "$SOURCE/engine" "$SOURCE/repo-view" "$TREE/"
    cp -R "$CRATE" "$TREE/crate"
    cp "$CONFIG" "$WORK/.cargo/config.toml"
    sed "s#$prefix#$TREE#g" "$SOURCE/engine-budget.toml" > "$TREE/engine-budget.toml"
    sed -i.frozen "s#$prefix#$TREE#g" "$TREE/crate/Cargo.toml"
    if grep -q "$prefix" "$TREE/engine-budget.toml" "$TREE/crate/Cargo.toml"; then die "a frozen path survived"; fi
    (cd "$TREE/engine" && patch -p1 --dry-run < "$HERE/stroke-index.patch") > "$LOGS/patch-dry-run.log"
    # Offline resolution for both builds, before anything compiles.
    (cd "$WORK" && cargo_ metadata --locked --offline --format-version 1 --config "$TREE/engine-budget.toml" \
        --manifest-path "$TREE/crate/Cargo.toml") > "$LOGS/metadata-film.json"
    (cd "$TREE/engine" && cargo_ metadata --locked --offline --format-version 1) > "$LOGS/metadata-engine.json"
    echo "prepare ok: $TREE"
}

build() {
    side="$1"
    bin="$WORK/bin/$side"
    [ ! -e "$bin" ] || die "$bin already exists"
    (cd "$WORK" && cargo_ build "${FLAGS[@]}") 2>&1 | tee "$LOGS/build-$side.log"
    cp "$CARGO_TARGET_DIR/release/complex_analysis_visual" "$bin"
    sum "$bin" > "$bin.sha256"
    echo "$side binary $(cat "$bin.sha256")"
}

base() {
    pin "$RENDER_SRC/stroke.rs" "$STROKE_BASE"
    pin "$RENDER_SRC/engine.rs" "$ENGINE_BASE"
    build base
}

cand() {
    [ -x "$BASE" ] || die "build base first"
    [ ! -e "$CAND" ] || die "$CAND already exists"
    pin "$RENDER_SRC/stroke.rs" "$STROKE_BASE"
    pin "$RENDER_SRC/engine.rs" "$ENGINE_BASE"
    (cd "$TREE/engine" && patch -p1 --no-backup-if-mismatch < "$HERE/stroke-index.patch")
    pin "$RENDER_SRC/stroke.rs" "$STROKE_CAND"
    pin "$RENDER_SRC/engine.rs" "$ENGINE_CAND"
    (cd "$TREE/engine" && CARGO_TARGET_DIR="$WORK/target-test" cargo_ test --locked --offline -p fmn-render --lib) \
        2>&1 | tee "$LOGS/tests-cand.log"
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
    [ ! -e "$out" ] || die "$out already exists"
    mkdir -p "$out"
    pids=()
    for side in base cand; do
        (cd "$WORK" && timeout 43200 "$WORK/bin/$side" render all --res 64x36 --fps 1 --silent --encoder libx264 \
            --crf 0 --preset ultrafast --out "$out/$side") > "$LOGS/film-$side.log" 2>&1 &
        pids+=("$!")
    done
    for pid in "${pids[@]}"; do wait "$pid" || die "a film render failed; see $LOGS"; done
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
    perf report --stdio --no-children --sort symbol -i "$data" 2>/dev/null | head -60 | tee "$LOGS/profile.txt"
}

case "$STEP" in
    prepare|base|cand|ab64|ab4k|film|profile) "$STEP" ;;
    *) die "unknown step $STEP" ;;
esac
