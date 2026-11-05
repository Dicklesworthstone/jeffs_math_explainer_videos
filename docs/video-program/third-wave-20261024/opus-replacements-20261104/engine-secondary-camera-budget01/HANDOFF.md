# Camera budgets: handoff

**Status:** source patch and tests, written by reading the engine. **Nothing here has been compiled, run or rendered.** The patch replayed cleanly on root's stack and rustfmt passed; that is the only tool evidence. RubyRaven wrote it, so it isn't an independent review. Root owns native validation. No film, narration, kit, pin, canonical file or Bead changed, and the Oliver narration is unaffected. The sealed `engine-secondary-budget01` bundle is unchanged; this patch stacks on it.

The owner's aim is for dense or moving visuals not to be thinned to fit a limit. This patch moves the camera route's fixed geometry limits to declared budgets. It doesn't make capacity unlimited, and it doesn't make those frames fast. Nothing here proposes fewer rows, cells or turn steps.

## What changes

`RenderOptions` gains one field, set through the same film hook as `plan_limits`:

```rust
fn declare_camera_budget(opts: &mut fmn::rendering::RenderOptions) {
    opts.camera_limits.max_working_bytes = 1 << 30;
    opts.camera_limits.max_raster_triangles = 1 << 23;
    opts.camera_limits.max_curve_bin_entries = 1 << 26;
}
```

Those numbers only show the shape. `ThreeDPreparationLimits` already existed, and it gains one field. The kit needs no change, because its hook hands over `&mut RenderOptions`.

| Site | Before | Now |
|---|---|---|
| `retained.rs` `render_camera_scene` | `ThreeDJob::new`, fixed defaults | `ThreeDJob::new_with_limits` with the renderer's limits |
| `retained.rs` `RetainedFrameRenderer` | No setting | `set_camera_limits`, starts at the default |
| `retained.rs` `PreparedCameraFrame` | No limits | Carries the limits frozen at capture |
| `pipeline.rs` serial and compiled-worker camera sites | Not set | Both set from `RenderOptions` |
| `three_d.rs` `CURVE_BIN_MAX_ENTRIES` | Fixed `1 << 24` | `max_curve_bin_entries`, default `1 << 24`, held to `u32::MAX` |
| `three_d.rs` fill-piece refusal | Always named `fill pieces` | Names working bytes when those ran out (see below) |

## Guards that stay

No check was removed or widened. These still apply whatever a film declares:

- Row counts and byte totals use checked arithmetic.
- Projected curves are capped at `u32::MAX`, and so are vector segment indices and bin offsets.
- Output rows are reserved fallibly, and a failure refuses by name.
- Refusals happen on the worker before the frame is published, so no artifact appears.

## One refusal was misnamed, now fixed

When working bytes ran out inside a fill, the refusal said `fill pieces` and quoted the piece limit, with a requested count under it. That would send an author to raise the wrong budget. It now names `preparation working bytes` when the rows fit their limit. A real shortage of pieces reads as before. No frame is admitted or refused differently.

## What a camera frame allocates

From reading the route; no size was measured. Bytes are added by hand from the fields.

| Step | What | Bytes | Budget | If allocation fails |
|---|---|---|---|---|
| Capture | Plan sync: retained segments, shapes, styles, knots | 88 a segment | `plan_limits` | Named |
| Capture | Surface meshes, rebuilt each frame | 104 a vertex, 4 an index | **None** | Named |
| Capture | Prepared commands, one per dot | At least 248 each | **None** | Named |
| Capture | **Deep clone of the whole plan into the frame** | 88 a segment, plus tables | **None** | **Aborts, no refusal** |
| Raster | Draw references | At least 248 each | **None** | Named |
| Raster | Compiled rows: draws, curves, pieces, triangles, bins | See below | `camera_limits` | Named |
| Raster | Temporaries: world segments, clipping, screen segments | See below | `camera_limits`, charged and never credited | Named |
| Raster | Tile scratch, per worker per job | Small | None | `FrameError::TooLarge` |
| Raster | Raw frame, per team | 8 a pixel | `max_resident_bytes` checks it | Not fallible |

- **`max_resident_bytes` covers only frame storage.** It doesn't see any geometry above.
- **`max_working_bytes` overstates the peak on strokes.** A plain curve is charged 704 bytes, but much of that is temporary and freed while still counted. I left it conservative, because crediting it back would change what the default admits.
- **The plan clone is the largest uncovered cost.** Each camera frame in flight owns a full copy, and the compiled worker makes one per frame too. An `Arc` wouldn't help a moving scene, since each frame changes the plan. I didn't change it. A film under memory pressure can lower `frames_in_flight`, which is already 2 by default.

## Limits

| Limit | Default | Status |
|---|---|---|
| `max_draws` | `1 << 20` | Declared here, refuses |
| `max_projected_curves` | `1 << 22` | Declared here, refuses |
| `max_fill_pieces` | `1 << 22` | Declared here, refuses |
| `max_raster_triangles` | `1 << 22` | Declared here, refuses |
| `max_working_bytes` | 512 MiB, per frame per team | Declared here, refuses |
| `max_curve_bin_entries` | `1 << 24` per vector | Declared here, **never refuses** |
| `RenderPlanLimits` | Unchanged | Already declared, refuses |
| `CURVE_BIN_MAX_CELLS` | `1 << 22` | Fixed: it only coarsens the grid |
| `CURVE_BIN_MIN_CURVES` | 32 | Fixed: a speed choice |
| `FLATTEN_MAX_DEPTH` | 12 | Fixed: an accuracy cap, refuses by name |
| `u32` index widths | `u32::MAX` | Fixed: absolute |
| Tile sizes, certified | 16 and 128 | Fixed: `RenderOptions` can't set them |

## Where a frame slows with no error

These are the real risk for dense scenes, because nothing refuses and pixels stay correct. The facade has no counter for any of them, so frame time is the only symptom. I didn't add one; it is a small addition if a probe shows an unexplained slow frame.

**A stroked vector leaves its bins, and every stroke sample then scans every curve:**

1. **Over `max_curve_bin_entries`.** Now declared.
2. **Bins don't fit the working bytes that remain.** Vectors late in a frame can lose their bins before anything refuses. Raising `max_working_bytes` cures it.
3. **One curve's stroke can reach the projection horizon.** That sends the whole vector to the scan, and no budget changes it. A camera passing close to geometry could do this.

**Costs no budget touches, from reading:**

- **Fill has no bins.** Each sample walks all of a vector's fill pieces.
- **Every fine tile tests every draw's bounds.** There's no per-tile list, which matters for dot clouds.
- **Curves and triangles reserve one row at a time, exactly.** A very large single vector or mesh may reallocate repeatedly. The cost is unmeasured, and I didn't change it, since looser growth would hold more than the budget admits.

## Front doors

From a text search at the pin. No door below the first group passes limits, so each stays at the defaults.

| Door | Reached through | Camera budgets |
|---|---|---|
| `fmn::render`, `render_with_fs` with `RenderOptions.camera` | `pipeline.rs` serial site | **Governed** |
| `render_camera`, `render_camera_with_fs` | The same `RenderSink` | **Governed** |
| `render_bundle_with_fs` with a camera | `pipeline.rs` compiled worker | **Governed** |
| Kit `run_with_contract_and_options`, including `request_camera` chapters | The routes above | **Governed**, by the hook; this patch adds no kit test |
| `NativeFramePipeline::new`, `with_plan_limits` | Rust callers | **Unsupported**: defaults |
| `fmn` CLI, `offline_cpu.rs:25` | `NativeFramePipeline::new` | **Unsupported**: no flag |
| fmn-python: `lib.rs:803`, `portal_readback.rs:226`, `portal_studio.rs:225`, `portal_studio/bundle.rs:513` | `render_with_camera` | **Unsupported** |
| fmn-studio: `camera_bundle.rs:253`, `native/raster.rs:117` | `render_with_camera` | **Unsupported** |
| fmn-wasm: `camera_player.rs:33` | `render_with_camera` | **Unsupported** |
| `ThreeDJob::new` direct, including `fmn-conformance` tests | Direct | Default; `new_with_limits` remains open to it |
| Metal | Facade refuses it for camera content | Not applicable |

`set_camera_limits` is public, so those doors could adopt it. None of them does in this patch.

## Estimates versus measurements

**Measured, by root, none of it on this patch or the camera route.** `plate_lab` has no `request_camera` and sets no camera, so these are the affine route, from a debug-target build:

| Receipt | Scope | Real | Max resident | Peak footprint |
|---|---|---|---|---|
| `complex-geometry-probes4k-01` | 26 cells4 frames at 3840×2160 | 426.69 s | 2,456,748,032 B | 1,442,990,360 B |
| `complex-geometry-probes720-01` | The cells4 block at 1280×720 | 292.32 s | 2,105,212,928 B | 1,269,581,864 B |

Don't read these as camera costs: the affine route builds different tables. Root's native tests also passed on the tree before the frame-table patch, and that patch and this one are both unrun.

**Arithmetic only, from `checks/camera_demand.py`.** Nothing below has been measured. Each number is logical bytes for one frame on one team.

| Scene at 3840×2160 | Demand | Default that binds | Note |
|---|---|---|---|
| z² cells4 rows, 1 curve a segment | 360 MiB, about 16 MiB of bins | None | Fits |
| z² cells4 rows, 2 curves a segment | 487 MiB, about 33 MiB of bins | Bins lost, then working bytes | Over 512 MiB by bins alone, so late rows scan instead of refusing |
| z² cells4 rows, 4 curves a segment | 741 MiB, about 66 MiB of bins | Working bytes | Refuses by name |
| Surface, G = 750 | 472 MiB | None | Mesh is 69 MiB more per frame in flight |
| Surface, G = 1000 | 839 MiB | Working bytes | The 4M-triangle limit is far off |
| Dots, 472,597 visible | At least 512 MiB | Working bytes | Binds long before `max_draws` |
| Dots, 1,048,576 visible | At least 1,136 MiB | Working bytes | Then 3.4 × 10¹⁰ bounds tests a frame |
| One vector of N frame-spanning curves | About 2N² bin entries | Bins lost above N = 2,896 | Silent; 64 MiB of entries at the limit |

- **Which z² row applies is unknown.** A segment splits until it is within 1/256 pixel of the true projection, and the split count under a moving camera is unmeasured. Fill, if any, adds at least 456 bytes a segment.
- **Per team.** Each render team compiles its own frame, so two teams at a declared 1 GiB can hold about 2.1 GiB with raw frames, before plan clones and the scene.
- **Plan clones.** 45 MiB a frame for one generation of z² cells4 segments, 90 MiB for two, before styles and shapes.
- **Time.** Only operation counts appear here, never seconds. Whether a dense camera frame is usable has to be measured.

For a first z² cells4 camera probe, I'd declare `max_working_bytes: 1 << 30`, which holds the 4-curve row by arithmetic. It is a first scene-specific setting, not a ceiling.

## Corrections to the frame-table handoff

That bundle stays sealed, so these corrections live here.

1. **It listed five fixed camera limits.** There was a sixth, the bin-entry cap, and it fails silently.
2. **It called the camera follow-up small.** The wiring is, but the trace also found the misnamed refusal and the unbudgeted plan clone.
3. **Its test still holds.** `declared_frame_tables_reach_the_compiled_affine_workers_and_not_the_camera_route` is still true, because frame tables never reach the camera route. The new test asserts the reverse.

## Tests, none run

| File | Test | What it proves |
|---|---|---|
| `fmn-render/src/three_d.rs` | `every_working_byte_shortage_names_working_bytes` | Every byte short of a filled, stroked vector's demand names working bytes |
| | `the_curve_bin_budget_changes_the_search_and_moves_no_pixel` | Exact boundary: bins at the demand, scan one under, identical frames |
| | `the_curve_bin_budget_is_held_to_the_width_of_the_bin_offsets` | The `u32` hold |
| | `a_bin_budget_above_the_default_bins_what_the_default_scans` | About 20.35 million entries: scanned by default, binned when raised |
| `fmn-render/tests/camera_route_limits.rs` | `the_default_camera_budgets_are_the_limits_the_engine_always_had` | All six defaults |
| | `both_routes_refuse_each_camera_budget_by_name_at_the_same_boundary` | Five budgets, same boundary on both routes, exact counts 4, 4 and 6 |
| | `a_camera_budget_moves_no_pixel_on_either_route` | Identical frames at default, raised and zero-bin budgets, tilted camera |
| | `a_working_byte_budget_above_the_default_is_admitted_and_the_default_still_refuses` | **`#[ignore]`.** 800,000 curves over the real default. About 0.4 GB by arithmetic, for root to run under guard |
| `fmn/tests/native_render.rs` | `a_lowered_camera_budget_refuses_by_name_and_publishes_nothing` | Serial facade, four budgets, nothing published |
| | `a_camera_budget_moves_no_pixel_on_the_serial_facade` | Default pinned on `RenderOptions`, identical frames |
| `fmn/tests/native_bundle_pipeline.rs` | `declared_camera_budgets_reach_the_compiled_camera_workers_and_not_the_affine_route` | Worker compiler, identical frames when raised, affine route ungoverned |
| `fmn/tests/native_camera_pipeline.rs` | `declared_camera_budgets_reach_the_camera_rig_route` | The route `request_camera` chapters use |

No existing assertion changed. Five existing `three_d.rs` test literals gained the new field. No new test renders at 4K or goes through ffmpeg. I didn't register an e2e scenario, which AGENTS.md (fm-fjq) asks of a feature bead, and I wrote no film-level probe. `BUILD.md` has the control plan.

Part 1 of `checks/camera_demand.py` mirrors the counts these tests assert, and a mirror can share a misreading with them.

## Files

| File | Purpose |
|---|---|
| `engine-camera-limits.patch` | The patch: eight files, one new |
| `expected-engine.sha256` | Result on root's staged tree plus the frame-table patch |
| `BUILD.md` | Exact commands, stop table, control plan |
| `checks/patch-replay.out` | Replay by `git apply` and by `patch -p1 -F0` |
| `checks/rustfmt.out` | Formatting, per changed file |
| `checks/struct-literals.out` | Every `ThreeDPreparationLimits` literal in the workspace |
| `checks/camera_demand.py`, `.out` | The arithmetic above |
| `bundle.sha256` | Every file here |
