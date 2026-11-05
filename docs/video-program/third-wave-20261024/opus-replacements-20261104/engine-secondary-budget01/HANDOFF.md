# Secondary frame budgets: handoff

**Status:** source patch and tests, written by reading the engine. **Nothing here has been compiled, run or rendered.** The patch replayed cleanly and rustfmt passed; that is the only tool evidence. RubyRaven wrote it, so it isn't an independent review. Root owns native validation. No film, narration, kit, pin, canonical file or Bead changed, and the Oliver narration is unaffected.

The owner's aim is for dense or moving visuals not to be thinned to fit a limit. This patch moves two fixed ceilings to declared budgets on the route films use. It doesn't make capacity unlimited, and it doesn't make those frames fast.

## What changes

`RenderOptions` gains one field, set the same way as `plan_limits`:

```rust
fn declare_plan_budget(opts: &mut fmn::rendering::RenderOptions) {
    opts.frame_table_limits.binning.max_tile_references = 1 << 25;
    opts.frame_table_limits.binning.max_working_bytes = 512 * 1024 * 1024;
    opts.frame_table_limits.monotone.max_pieces = 1 << 23;
}
```

`FrameTableLimits { monotone: MonoTableLimits, binning: BinningLimits }` is new. Both inner types already existed, with public fields. The film hook and the kit route tested in validation02 need no change.

| Site | Before | Now |
|---|---|---|
| `vector_frame.rs` `render_cached_into`, `MonoTable` | Default | `frame_tables.monotone` |
| `vector_frame.rs` `render_cached_into`, `Binning` | Default | `frame_tables.binning` |
| `engine.rs` `prepare`, frame-local `pieces_for_segments_into` | Default | The same `frame_tables.monotone` |
| `pipeline.rs` serial compiler and compiled-worker compiler | Not set | Both set from `RenderOptions` |
| `engine.rs` `with_identity_in`, `with_identity`, `for_metal` | Default | Default, unchanged behaviour |
| `retained.rs` `RetainedFrameRenderer::render` | Default | Default, untouched |

## Exact scope

**Governed:** the affine route of the native facade, meaning `fmn::render`, `render_with_fs` and `render_bundle_with_fs` when no camera is set.

**Not governed:**

- **Camera route:** it builds neither table. `prepare_with_camera` reaches `ThreeDJob::new` (`retained.rs:632`) with fixed `ThreeDPreparationLimits`: draws `1<<20`, projected curves `1<<22`, fill pieces `1<<22`, raster triangles `1<<22`, working bytes 512 MiB. `ThreeDJob::new_with_limits` already exists, so a follow-up would be small. A text search found no stage01 chapter calling `request_camera`, so I left it out. A test pins this: a zero budget doesn't refuse on the camera route.
- **Other front doors:** fmn-cli, fmn-python, fmn-wasm and fmn-studio stay at the defaults.
- **Metal:** stays at the default.

## Corrections to validation02's handoff

1. **`retained.rs:242` is not a facade bottleneck.** That handoff named it, but the facade calls only `prepare_with_camera` on that type, and neither the kit nor the lab uses it. The affine route is `VectorFrameCompiler`, so I left `retained.rs` alone.
2. **It missed a site.** `engine.rs` applied the default monotone budget to frame-local pieces for every instance placed by more than a translation. Patching only the table would have refused those frames at the old limit.
3. **The 4,292,432 pieces was a bound, not the demand.** It assumed four pieces a segment. A straight segment makes one, so two generations of rotating z² rows need about 1,074,570, under the default. Rounding can add splits at a turn near 0° or 90°, and that count wasn't measured.
4. **One of my assertions failed in root's run.** `wait(0.1)` at 10 fps emits two frames, as root's frame-clock repair records. The new binning test copies root's corrected expectation.

## Guards that stay

No check was removed or widened. These still apply whatever a film declares:

- The piece count is checked against the limit, then against `u32`.
- Table sizes use checked arithmetic.
- Tile, instance and reference counts are also capped at `u32::MAX`.
- Every table is reserved fallibly, and a failure refuses by name.
- Refusals happen before the frame is published.

## Memory

The numbers below are from reading the code, not from a measurement.

- **Per frame, per render team:** each team builds its own tables, so the real cost is budget × teams. `max_resident_bytes` doesn't cover any of it.
- **Monotone:** the shared table and the frame-local pieces are each checked against the budget separately, so a frame can hold up to twice it.
- **Binning:** `max_working_bytes` counts about 12 bytes a reference. `prune_occluded` then copies offsets, draws and flags beside the originals, about 16 bytes a reference at its peak, and that step isn't in the budget. Those allocations do fail by name. I didn't change this, because it would alter what the default admits.
- **Arena:** `FrameArena` keeps its high-water mark for the life of the team. Frame-local pieces reserve exactly per instance, so a large first frame may reallocate repeatedly. The cost is unmeasured.

## Which budget binds first

Estimates at 3840×2160 with certified 16-pixel tiles (32,400 fine tiles). They come from `checks/secondary_demand.py`, which is arithmetic only.

| Scene | Tile references (default 16,777,216) | Monotone pieces (default 4,194,304) | Working bytes (default 256 MiB) | Binning peak while pruning | Must declare |
|---|---|---|---|---|---|
| 517 full-frame instances | 16,750,800 | Far under | 193 MiB | 256 MiB | Nothing |
| 518 full-frame instances | 16,783,200 | Far under | 193 MiB | 256 MiB | `max_tile_references` |
| 1,024 full-frame instances | 33,177,600 | Far under | 382 MiB | 506 MiB | References and working bytes |
| 4,096 full-frame instances | 132,710,400 | Far under | 1,527 MiB | 2,025 MiB | References and working bytes |
| z² cells4 rotating, at 45°, two generations | About 3,380,000 | 1,074,570 | Under | About 52 MiB | Plan limits only, already exposed |

- **Full-frame** means the instance's bounding box covers the frame. Binning uses the box, so one long diagonal line counts in full.
- **Raising only `max_tile_references`:** working bytes then bind near 22.2 million references, about 686 full-frame instances.
- **Raising only `max_pieces`:** `max_table_bytes` binds at 5,592,405 pieces.
- **The moving z² plate:** these budgets don't limit it today. The retained-segment limit does, then per-frame rebuilds.

**Frame time is the open question.** Binning runs every frame, in time proportional to references. A stroked diagonal is also referenced by tiles it never touches, and those tiles still have to process it. A raised budget makes such a scene renderable, but its speed has to be measured on a real scene.

## Tests, none run

| File | Test | What it proves |
|---|---|---|
| `fmn-render/tests/frame_table_limits.rs` | `the_default_budgets_are_the_limits_the_engine_always_had` | All seven default values |
| | `monotone_budgets_govern_the_shared_table_and_the_frame_local_pieces` | Exact boundaries at both sites: 2 pieces and 104 bytes in the table, 3 pieces and 144 bytes in the frame |
| | `binning_budgets_each_refuse_by_name_at_their_exact_boundary` | Fine tiles, instances, partition cells, both reference passes, working bytes |
| | `a_larger_budget_moves_no_pixel` | Identical frame bytes under raised budgets |
| | `a_monotone_budget_above_the_default_is_admitted_and_the_default_still_refuses` | **`#[ignore]`.** About 0.51 GB by arithmetic, for root to run under guard |
| `fmn/tests/native_render.rs` | `a_lowered_frame_table_budget_refuses_by_name_and_publishes_nothing` | Serial facade route, nothing published |
| | `a_declared_binning_budget_above_the_default_is_admitted_and_the_default_still_refuses` | No clamp to the default, 2,057 full-frame layers at 1080p, about 340 MB by arithmetic |
| `fmn/tests/native_bundle_pipeline.rs` | `declared_frame_tables_reach_the_compiled_affine_workers_and_not_the_camera_route` | Worker compiler, identical frames under raised budgets, camera route ungoverned |

No existing test was edited. No new test renders at 4K or goes through ffmpeg. I didn't register an e2e scenario, which AGENTS.md (fm-fjq) asks of a feature bead. I also wrote no film-level probe; the real dense or moving scene is root's to choose.

Part 1 of `checks/secondary_demand.py` mirrors the piece and tile counts these tests assert. A mirror can share a misreading with the tests. It did catch one wrong number: 1,365 outlines cross the limit, not the 1,366 in my first draft.

## Still fixed on this route

- `MAX_STROKE_PROFILE_KNOTS` and `MAX_FILL_PROFILE_KNOTS`, `1<<20` per mobject.
- `u32` index widths, which are absolute.
- Tile sizes under certified determinism (16 and 128), which `RenderOptions` can't set.

## Files

| File | Purpose |
|---|---|
| `engine-frame-table-limits.patch` | The patch: eight files, one new |
| `expected-engine.sha256` | Result on root's staged tree, including its frame-clock repair |
| `BUILD.md` | Exact commands, stop table, and which test catches which mis-wiring |
| `checks/patch-replay.out` | Replay by `git apply` and by `patch -p1 -F0` |
| `checks/rustfmt.out` | Formatting, per changed file |
| `checks/secondary_demand.py`, `.out` | The arithmetic above |
| `bundle.sha256` | Every file here |
