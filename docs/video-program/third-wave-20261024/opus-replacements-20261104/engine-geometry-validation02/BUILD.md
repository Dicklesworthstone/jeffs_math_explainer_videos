# Geometry validation: build and probe commands

For root's guarded native run. Nothing here has been compiled, run or rendered. The pre-Cargo steps in section 1 were rehearsed in a throwaway folder and passed; every step after them is unrun. Everything is written to `$WORK`, outside the repo. Canonical engine, kit, film, Beads and defaults stay untouched.

## 0. Setup

```sh
REPO=<the repo root>
O="$REPO/docs/video-program/third-wave-20261024/opus-replacements-20261104"
V="$O/engine-geometry-validation02"
B="$O/engine-geometry-budget01"
X="$O/engine-geometry-cross-review01"
PIN="$HOME/.cargo/git/checkouts/franken_manim-6641d0e09642f63e/a399d62"
NLC="$REPO/narration_lab/third-wave-20261024/root/elevenlabs-complex-rebuild-20261104/color-native-stage01"
REV=a399d626c43ecc852a5c795288979f52b446bc5c
KITREV=589ab3dfbfb649773c19b82079bc0b9cd5aa310f
WORK=<a new folder outside the repo>      # Cargo merges ancestor configs, so never inside it
EXPORT="$WORK/engine"; VIEW="$WORK/repo-view"; LAB="$WORK/crate"
CONFIG="$WORK/engine-budget.toml"
export CARGO_INCREMENTAL=0 RCH_SHIM_LOCAL_IDE=1
```

Check the frozen inputs first: `shasum -a 256 -c "$V/bundle.sha256"` from `$V`, `shasum -a 256 -c SHA256SUMS` from `$X`, and `shasum -a 256 -c bundle.sha256` from `$B`.

## 1. Sources (rehearsed, no Cargo)

```sh
mkdir "$WORK" "$EXPORT" "$VIEW"

git -C "$PIN" archive "$REV" | tar -x -C "$EXPORT"
cd "$EXPORT"
git apply "$B/engine-plan-limits.patch"
git apply "$V/engine-compiled-route-test.patch"
git apply "$V/engine-tests-repair.patch"
shasum -a 256 -c "$V/expected-engine.sha256"
python3 -I -B "$V/instantiate_config.py" "$EXPORT" "$CONFIG"

git -C "$REPO" archive "$KITREV" Cargo.toml Cargo.lock rust-toolchain.toml rustfmt.toml .cargo crates | tar -x -C "$VIEW"
cd "$VIEW"
git apply "$X/patches/kit-render-options-hook.patch"
git apply "$V/kit-hook-routes-test.patch"
shasum -a 256 -c "$V/expected-kit.sha256"
cp Cargo.lock "$WORK/Cargo.lock.view-pinned"

cp -R "$NLC/crate" "$LAB"
cd "$LAB"
git apply "$B/lab-plate_lab.patch"
git apply "$V/lab-probes.patch"
git apply "$V/lab-fade-turn-probes.patch"
git apply "$X/patches/film-declare-plan-budget.patch"
shasum -a 256 -c "$V/expected-lab.sha256"
cp Cargo.lock "$WORK/Cargo.lock.pinned"
python3 -I -B "$V/repoint_lab.py" "$LAB" "$VIEW"
```

## 2. Engine

```sh
cd "$EXPORT"
export CARGO_TARGET_DIR="$WORK/target-engine"
```

The engine's own lock pins eight crates that are not on this machine: cc 1.4.0, glam 0.33.11, zerocopy and zerocopy-derive 0.8.55, futures-core, futures-task and futures-util 0.3.33, futures-io 0.3.33. Choose one:

- **A, recommended:** `cargo fetch --locked` once. It reaches crates.io and compiles nothing.
- **B:** drop `--locked` from the engine commands below, in this export only. Unverified.

```sh
rustfmt --check --edition 2024 crates/fmn/tests/native_render.rs crates/fmn/tests/native_bundle_pipeline.rs crates/fmn-render/tests/stroke_profiles.rs
cargo clippy --locked --offline -p fmn-render -p fmn --all-targets -- -D warnings
cargo test --locked --offline -p fmn-render
cargo test --locked --offline -p fmn --test native_render --test native_bundle_pipeline
cargo check --locked --offline -p fmn-cli --all-targets
```

Tests that must appear and pass:

| Test | File | What it holds |
|---|---|---|
| `the_default_stays_where_every_existing_render_has_it` | `stroke_profiles.rs` | default stays `1 << 20` |
| `the_knot_boundary_is_exact_and_a_refusal_changes_nothing` | `stroke_profiles.rs` | exact boundary |
| `the_picture_that_crossed_the_default_is_admitted_by_a_declared_budget` | `stroke_profiles.rs` | repaired: 1,075,301 knots, 734 segments, 1 shape, 730 shared, 731 instances |
| `a_repaint_needs_twice_the_live_knots_and_says_so_when_it_has_less` | `stroke_profiles.rs` | repaint demand |
| `a_budget_moves_compaction_and_never_the_pixels` | `stroke_profiles.rs` | a small budget compacts earlier with identical pixels, on `RetainedFrameRenderer`, not the film's route |
| `every_capture_route_answers_to_the_declared_budget` | `stroke_profiles.rs` | capture routes |
| `a_declared_plan_budget_reaches_the_renderer_and_refuses_by_name` | `native_render.rs` | facade, lowered limit |
| `a_declared_plan_budget_above_the_default_is_admitted_and_the_default_still_refuses` | `native_render.rs` | **new.** Facade, raised limit. Catches a clamp to the default |
| `a_declared_plan_budget_reaches_the_compiled_workers_on_both_routes` | `native_bundle_pipeline.rs` | compiled workers, lowered limit |

Optional controls for the compiled-route test. Each must make that test fail with its named panic.

```sh
git apply "$V/controls/mutant-vector-worker.patch"
cargo test --locked --offline -p fmn --test native_bundle_pipeline a_declared_plan_budget_reaches_the_compiled_workers_on_both_routes
#   expected panic: "camera route false: a worker admitted one knot over its declared budget"
git apply -R "$V/controls/mutant-vector-worker.patch"

git apply "$V/controls/mutant-camera-worker.patch"
cargo test --locked --offline -p fmn --test native_bundle_pipeline a_declared_plan_budget_reaches_the_compiled_workers_on_both_routes
#   expected panic: "camera route true: a worker admitted one knot over its declared budget"
git apply -R "$V/controls/mutant-camera-worker.patch"

shasum -a 256 -c "$V/expected-engine.sha256"
```

## 3. Kit routes

The view has fewer workspace members than the repo, so its first resolve should only drop packages. That resolve is unrun; the `--subset` check in 3b is what confirms it.

```sh
cd "$VIEW"
# 3a. The hook and its test on the unpatched pin. This is what lets the hook land without moving the pin.
cargo metadata --offline --format-version 1 > /dev/null
CARGO_TARGET_DIR="$WORK/target-view-pin" cargo check --locked --offline -p explainer_kit --all-targets

# 3b. The same kit on the patched engine. Check the lock before compiling anything.
export CARGO_TARGET_DIR="$WORK/target-view"
cargo metadata --offline --config "$CONFIG" --format-version 1 > /dev/null
python3 -I -B "$V/verify_patched_lock.py" --subset "$WORK/Cargo.lock.view-pinned" "$VIEW/Cargo.lock"
rustfmt --check --edition 2024 --config-path "$VIEW/rustfmt.toml" crates/explainer_kit/tests/options_hook_routes.rs
cargo clippy --locked --offline --config "$CONFIG" -p explainer_kit --all-targets --no-deps -- -D warnings
cargo test --locked --offline --config "$CONFIG" -p explainer_kit --lib --test options_hook_routes --test camera_chapter --test glyph_probe --test cli_pipeline
```

- Do not use `cargo fmt --check -p explainer_kit`. `crates/explainer_kit/src/narration.rs` is already flagged at `$KITREV`, independent of this work.
- The tests need ffmpeg with libx264 on PATH.
- The repo's `[profile.test] opt-level = 2` makes this a separate, slower compile.

`the_hook_governs_every_render_route` covers retained and camera draft, retained and camera `--crf` delivery, the thumbnail, and retained and camera glyph capture. Each route must exit 0 with an idle hook and non-zero with one that sets `max_output_bytes = 1`. `delivery_options` raises that field, so a hook that ran before the rebuild would lose its value and the test would fail.

## 4. Lab and film build

`$REPO` supplies the toolchain and Cargo config. Sections 5 to 8 keep this target folder.

```sh
cd "$REPO"
export CARGO_TARGET_DIR="$WORK/target-lab"
M="$LAB/Cargo.toml"
cargo metadata --offline --config "$CONFIG" --manifest-path "$M" --format-version 1 > /dev/null
python3 -I -B "$V/verify_patched_lock.py" "$WORK/Cargo.lock.pinned" "$LAB/Cargo.lock"
rustfmt --check --edition 2024 --config-path "$REPO/rustfmt.toml" "$LAB/examples/plate_lab.rs"
cargo test   --locked --offline --config "$CONFIG" --manifest-path "$M" --example plate_lab
cargo build  --locked --offline --config "$CONFIG" --manifest-path "$M" --example plate_lab
cargo clippy --locked --offline --config "$CONFIG" --manifest-path "$M" --example plate_lab --no-deps -- -D warnings
cargo build  --locked --offline --config "$CONFIG" --manifest-path "$M" --bin complex_analysis_visual
cargo clippy --locked --offline --config "$CONFIG" --manifest-path "$M" --bin complex_analysis_visual --no-deps -- -D warnings
P="$WORK/target-lab/debug/examples/plate_lab"
F="$WORK/target-lab/debug/complex_analysis_visual"
```

## 5. Dry runs and identity control

```sh
"$P" "$WORK/dry01" --res 1280x720 --no-truth --dry-run
#   plan: 81 scenes, 153 frames at 1280x720       (unchanged)
"$P" "$WORK/dry02" --res 1280x720 --stage refusals,probes,fade-probes,turns --no-truth --dry-run
#   plan: 4 scenes, 32 frames at 1280x720

"$P" "$WORK/id01" --res 1280x720 --stage stills --subjects z2 --candidates cells2 --no-truth
cmp "$WORK/id01/stills/z2__cells2/frame_000000.png" "$NLC/lab720-01/stills/z2__cells2/frame_000000.png"
"$P" "$WORK/id02" --res 1280x720 --stage fades --subjects z --candidates blend --no-truth
diff -r "$WORK/id02/fades/z__blend" "$NLC/lab720-fades02/fades/z__blend"
```

Both comparisons must find no differences. They show that a rung inside the default keeps its bytes on the patched engine.

## 6. Dense probes, under root's controller

One command, `1280x720`, 32 frames. The refusal control runs first, so if it is admitted nothing heavy starts.

```sh
"$P" "$WORK/probe720-01" --res 1280x720 --stage refusals,probes,fade-probes,turns --no-truth
cmp "$WORK/probe720-01/probes/z2__cells4/frame_000004.png" "$WORK/probe720-01/probes/z2__cells4/frame_000005.png"
```

| Stage | Output | Budget | Expected, by reading |
|---|---|---|---|
| `refusals` | `refusals/z2__cells4.txt`, no frame folder | `1<<21` | Holds `resource: "retained profile knots", requested: 2097646, limit: 2097152`. Receipt says `refused by name` |
| `probes` | `probes/z2__cells4/`, 6 frames | `1<<22` | Receipt says `repainted in place`. Peak 3,225,903, compaction at capture 3. Frames 5 and 6 byte-identical |
| `fade-probes` | `fade-probes/z2__cells4/`, 7 frames | `1<<22` | Receipt says `kept through fades`. Peak 3,225,903 (Lavender, `checks/admission.out`, part 3) |
| `turns` | `turns/quintic__cells4/`, 13 frames | `1<<22` | Receipt says `turned after the takeover`, 640 rows, 640 cols. Model peak 4,124,800, at most 69,504 under the budget |

Read the frames as images, and record peak memory and seconds per frame from the controller. No budget in this work bounds memory.

If those pass, render the dense stills and then the same probes at 4K, each with a new OUT:

```sh
"$P" "$WORK/stills720-01" --res 1280x720 --stage stills --candidates cells4 --no-truth
"$P" "$WORK/probe4k-01" --res 3840x2160 --stage probes,fade-probes,turns --no-truth
```

## 7. Real film routes

Geometry depends on the plate style, not on `--res`, so `64x36` tests admission at low raster cost. It does not measure 4K memory or time.

```sh
cd "$LAB" && git apply "$X/patches/select-cells4.patch" && shasum -a 256 -c "$V/expected-cells4-draw.sha256"
cd "$REPO" && cargo build --locked --offline --config "$CONFIG" --manifest-path "$M" --bin complex_analysis_visual

"$F" render 04_reading_color --res 64x36 --fps 3 --silent --crf 23 --out "$WORK/film-cells4-01"    # must exit 0
"$F" render 05_gallery       --res 64x36 --fps 3 --silent --crf 23 --out "$WORK/film-cells4-01"    # must exit 0
"$F" thumbnail --res 64x36 --out "$WORK/film-cells4-01"
```

Do not use `--max-frames` for this. A chapter longer than the cap returns `scene exceeded max_frames`, publishes nothing and exits non-zero, which would look like a failed admission.

Control: the film hook is what admits the plate.

```sh
cd "$LAB" && git apply -R "$X/patches/film-declare-plan-budget.patch"
cd "$REPO" && cargo build --locked --offline --config "$CONFIG" --manifest-path "$M" --bin complex_analysis_visual
"$F" render 04_reading_color --res 64x36 --fps 3 --silent --crf 23 --out "$WORK/film-control-01"
#   must refuse by name: retained profile knots, requested 1048823, limit 1048576
cd "$LAB" && git apply "$X/patches/film-declare-plan-budget.patch"
cd "$REPO" && cargo build --locked --offline --config "$CONFIG" --manifest-path "$M" --bin complex_analysis_visual
```

Then, as the resource guard allows: `01_the_bump`, `07_one_point`, `09_boundary`, `10_rigidity`, `11_counting`, `12_residues`, `14_continuation`, and `"$F" glyphs "$WORK/glyphs01"`.

Film unit tests under the flip use the assertions `select-cells4.patch` updates:

```sh
cargo test --locked --offline --config "$CONFIG" --manifest-path "$M" --bin complex_analysis_visual
```

## 8. Real 11_7 turn, after root's merge

No tree yet holds all three of: the plate core, `complex11-counting-teaching02`, and its C1 correction, `picture_cells(&inp, &super::draw::PLATE)`. `color-native-stage01` has the plate core and no turn. `combined-repair-stage02` has the turn, a one-argument `picture_cells`, and no `PLATE`.

Once a tree holds all three:

```sh
"$F" render 11_counting --res 64x36 --fps <the film's rate> --silent --crf 23 --out <new>
```

Use the film's real frame rate, since a low rate reaches few of the 48 steps. Keep `TURN_STEPS` at 48. Before accepting it, check that `recolor` returned true, because the film ignores a false return (C1).

## 9. When to stop

| Observation | Action |
|---|---|
| A hash check fails or a patch refuses | Stop. An input moved |
| `verify_patched_lock.py` prints FAULT | Stop before compiling. Do not edit the lock by hand |
| `the control was admitted` from the lab, or the film control exits 0 | Stop. A limit is not reaching the renderer |
| An identity control differs | Stop. The engine patch changed existing output |
| A route test: `rendered past the hook` | Stop. That route ignores the hook |
| Refusal has a different `requested` | Record it. That is an estimate fault, not a budget fault |
| A probe is refused at `limit: 4194304` | Report the number. `1<<22` is short for that case. Do not lower the frames or the density |
| Refused by a different resource (`max_input_curves`, monotone pieces, binning) | Report it. The knot knob does not cover that limit |
| Probe frames 5 and 6 differ | Report it. A write happened where the reading says none does |
| Memory or time is more than the guard allows | Stop and report what was measured |
