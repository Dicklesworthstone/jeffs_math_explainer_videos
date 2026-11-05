# Camera budgets: build commands

For root's guarded native run, serially, after the frame-table bundle has had its own run. Nothing here has been compiled or run. Only the apply and hash steps in section 1, and rustfmt, have been exercised, on copies.

## 0. Base

Two steps stack, in this order, on root's staged tree from validation02, which includes root's frame-clock repair to `native_render.rs`:

1. `engine-secondary-budget01/engine-frame-table-limits.patch`, unchanged and still sealed.
2. `engine-camera-limits.patch`, from this folder.

If the frame-table run needs a repair, this patch may need rebasing, so stop and send back the repaired hashes. Apply to a new copy, outside any git work tree, and leave the staged tree as it is.

```sh
REPO=<the repo root>
O="$REPO/docs/video-program/third-wave-20261024/opus-replacements-20261104"
T="$O/engine-secondary-budget01"
S="$O/engine-secondary-camera-budget01"
ENGINE=<a new copy of the staged engine folder, outside the repo>
export CARGO_INCREMENTAL=0 RCH_SHIM_LOCAL_IDE=1
```

## 1. Apply (replayed on copies, see `checks/patch-replay.out`)

```sh
(cd "$T" && shasum -a 256 -c bundle.sha256)
(cd "$S" && shasum -a 256 -c bundle.sha256)
cd "$ENGINE"
git apply "$T/engine-frame-table-limits.patch"
shasum -a 256 -c "$T/expected-engine.sha256"
git apply "$S/engine-camera-limits.patch"
shasum -a 256 -c "$S/expected-engine.sha256"
```

## 2. Native checks (not run)

Under your guard, with your own target directory. Run them in this order, so the cheap ones fail first. The new `fmn-conformance` check is there because `perf_pg6_peak.rs` names the extended struct.

```sh
rustfmt --check --edition 2024 --config skip_children=true crates/fmn-render/tests/camera_route_limits.rs crates/fmn/tests/native_render.rs crates/fmn/tests/native_bundle_pipeline.rs crates/fmn/tests/native_camera_pipeline.rs
cargo clippy --locked --offline -p fmn-render -p fmn --all-targets -- -D warnings
cargo test --locked --offline -p fmn-render --test camera_route_limits
cargo test --locked --offline -p fmn-render --lib three_d
cargo test --locked --offline -p fmn --test native_bundle_pipeline
cargo test --locked --offline -p fmn --test native_camera_pipeline
cargo test --locked --offline -p fmn --test native_render
cargo test --locked --offline -p fmn-render
cargo check --locked --offline -p fmn-cli --all-targets
cargo check --locked --offline -p fmn-conformance --all-targets
```

- `a_bin_budget_above_the_default_bins_what_the_default_scans` builds about 20 million bin entries, about 80 MB by arithmetic, and never rasterizes. Its time is unmeasured, and the tests are unoptimised.
- `every_working_byte_shortage_names_working_bytes` recompiles one small vector once per byte of its demand, a few thousand times by reading.
- `both_routes_refuse_each_camera_budget_by_name_at_the_same_boundary` bisects five budgets on two routes, about 110 small renders.
- Plain `cargo test` skips the ignored test below.

## 3. The ignored control (not run)

Root's call whether and when to run it. It holds about 0.4 GB by arithmetic, and its raster time is unmeasured.

```sh
cargo test --locked --offline -p fmn-render --test camera_route_limits -- --ignored a_working_byte_budget_above_the_default
```

## 4. Stop table

| Result | Meaning | Do |
|---|---|---|
| Apply or hash fails | The base isn't the stack in section 0 | Stop and compare to the two `expected-engine.sha256` files |
| Compile or Clippy error | My source is wrong: none of it has been compiled | Send it back. Don't relax a lint |
| `both_routes…` fails on `draw rows`, `projected curves` or `raster triangles` counts | My scene arithmetic is wrong (4, 4 and 6) | Send the counts back. Don't widen the test |
| `both_routes…` fails on a resource name | A refusal is misnamed, or the fix below it isn't reached | Send the name and value back |
| `every_working_byte_shortage…` fails | Another path misnames a byte shortage | Send the `max_working_bytes` and error back |
| `a_bin_budget_above…` fails on `is_none` or on the count | The bin arithmetic is wrong (about 20.35 million) | Send the count back |
| `a_camera_budget_moves_no_pixel…` differs | A budget changed pixels, which would be a real defect | Stop and report the route and limits |
| `a_lowered_camera_budget…` or a declared-budget test admits the frame | That route isn't wired | Send it back |
| The ignored control isn't refused at the default | The 704-byte charge is wrong | Send the result back. Don't raise the scene size |
| An `AllocationFailed` refusal | A named memory guard worked | Note it. It isn't a pass |
| Resource stop from your controller | The estimates were low | Stop and report the peak |

## 5. Control plan

**Each budget's controls, per route:**

| Control | Fixed route | Worker route | Facade serial | Compiled worker | Camera rig |
|---|---|---|---|---|---|
| Below the demand refuses by name | `both_routes…` | `both_routes…` | `a_lowered_camera_budget…` | `declared_camera_budgets_reach_the_compiled…` | `declared_camera_budgets_reach_the_camera_rig…` |
| Raised budget, identical pixels | `a_camera_budget_moves_no_pixel_on_either_route` | Same test | `a_camera_budget_moves_no_pixel_on_the_serial_facade` | `declared_camera_budgets_reach_the_compiled…` | `declared_camera_budgets_reach_the_camera_rig…` |
| Over the real default, admitted when raised | Ignored control | Ignored control | Not tested | Not tested | Not tested |
| Bin budget over the default | `a_bin_budget_above…`, on `ThreeDJob` directly | Not tested | Not tested | Not tested | Not tested |

The whole struct travels as one value, so a route that carries one field carries all six.

**Which test catches which mis-wiring.** These come from reading the tests. No mutant was built or run. Each row is one optional small edit on a scratch copy, if a wiring result ever looks doubtful, and each costs a rebuild.

| Mutant | Should fail |
|---|---|
| `render_camera_scene` calls `ThreeDJob::new` | `both_routes…`, the facade tests |
| `PreparedCameraFrame.limits` is always the default | `both_routes…` on the worker route |
| `render_with_camera` passes the default | `both_routes…` on the caller route |
| Compiled-worker site in `pipeline.rs` omits `set_camera_limits` | `declared_camera_budgets_reach_the_compiled…` |
| Serial site in `pipeline.rs` omits `set_camera_limits` | `a_lowered_camera_budget…`, `declared_camera_budgets_reach_the_camera_rig…` |
| `rendering.rs` passes the default to `with_limits` | The same two |
| `build_curve_bins` compares against a fixed `1 << 24` | `a_bin_budget_above…`, `the_curve_bin_budget_changes…` |
| `curve_bin_entry_limit` loses its `u32` hold | `the_curve_bin_budget_is_held…` |
| The fill refusal is named `fill pieces` again | `every_working_byte_shortage…` |
| A default value changes | `the_default_camera_budgets…` |

**A film-level control doesn't exist yet.** `plate_lab` has no camera route. The first useful one is a dense scene through `request_camera`, 720p first, under `/usr/bin/time -l`, recording these per frame:

1. Refused or not, and under which name, at the default.
2. Frame time and peak memory at a raised budget, with `frames_in_flight` recorded.
3. Whether frame time jumps as the scene grows. That is the only symptom of a vector leaving its bins.

The scene is root's to choose. This bundle adds no scene, and it doesn't propose fewer rows, cells or turn steps.

## 6. Not covered

A real dense camera scene through these budgets: frame time, real peak memory, and the full route to an encoded chapter.
