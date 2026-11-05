# Opt-in geometry budget for pinned a399d626

Author: RubyRaven, for BlueSalmon's Mails 2789, 2832 and 2896. Capacity work, not visual acceptance.

**Nothing here has been compiled, run or rendered.** The tests are unrun, the patch replays cleanly on the pinned sources, and the numbers are arithmetic from reading the engine. No high-density picture exists yet.

## Result

- `RenderOptions` gains `plan_limits`, which reaches every capture route. The default stays `1 << 20`, so current jobs keep today's admission and compaction.
- **A cells4 scene should declare `max_retained_profile_knots: 1 << 22`, with nothing else raised.**
- **A z² cells4 still needs 1,075,301 knots, and any full repaint of it needs 2,150,602.** The logged 1,048,823 was only the point where it first crossed the limit, at row 713 of 731.
- `1 << 21` is not enough: a still fits, but the first repaint is refused. Compaction clears only rows already unreachable when a sync begins, so straight after a compaction the old and new rows must fit together.

## Demand, geometry only

The counts are in scene units, so **they are the same at 720p and 4K**. Only binning and raster change with resolution.

| | z² cells4 (731 × 731) | sin(1/z) cells4 (640 × 640) | 05_9 six panels, if cells4 |
|---|---|---|---|
| Knots per row | 1,471 | 1,289 | 969 |
| **Knots, still** | **1,075,301** | 824,960 | 1,726,758 |
| **Knots, full repaint** | **2,150,602** | 1,649,920 | 3,453,516 |
| Segments | 536,554 | 411,520 | 860,706 |
| Input curves | 538,016 | 412,800 | 864,270 |
| Rows (instances, live styles) | 731 | 640 | 1,782 |

A blended row has `2n + 9` knots, `n + 3` segments and `n + 5` input curves. The mirror reproduces the log exactly: 713 × 1471 = 1,048,823. The z² total is an upper bound past row 713, because two identical rows would share a style. The lab plan runs only stills at cells4, and 05_9 is hypothetical: `PLATE_CELLS4` remains a bracket, not a proposal.

| Budget | z² cells4 | sin(1/z) cells4 | Six panels |
|---|---|---|---|
| `1 << 20` (default) | Refused on the still, 1,048,823 | Still fits; repaint refused, 1,049,246 | Refused on the still, 1,049,427 |
| `1 << 21` | Still fits; repaint refused, 2,097,646 | Fits; compacts every 2 frames | Still fits; repaint refused, 2,097,885 |
| **`1 << 22`** | **Fits; compacts every 3 frames** | Fits; compacts every 5 frames | Fits; compacts every 2 frames |

`1 << 22` leaves 1.95× the z² repaint demand and 1.21× the six-panel demand. Every other painted plate in the film also fits at cells4, and all of them fit at cells2 under the default (`checks/demand.out`).

**Overlays:**

- **Lab stills:** none. `still()` adds only the plate, so the table is the whole scene.
- **Knots in the film:** I expect about zero. A uniform stroke or fill makes no profile, and the only per-record paint writers in the chapter and kit source are the plate code. I did not audit the engine's animations.
- **Curves and segments in the film:** not counted. Room left under `1 << 20`:

| Plate | Curves | Segments |
|---|---|---|
| z² cells4 | 510,560 | 512,022 |
| Six panels | 184,306 | 187,870 |

**Moving a plate is different.** Rewriting a plate's points doubles its segments as well: z² would need 1,073,108, over `1 << 20`, so it would also need `max_retained_segments` raised. As far as I read, plates only fade and recolour.

## The patch

`engine-plan-limits.patch`, sha256 `63c1afca…`, 7 files, +431 −44.

| File | Change |
|---|---|
| `fmn/src/rendering.rs` | `RenderOptions::plan_limits`, re-export of `RenderPlanLimits`, passed to the pipeline |
| `fmn/src/rendering/pipeline.rs` | `NativeFramePipeline::with_plan_limits`; `new` delegates with the default; the stages hold the budget for compiled workers |
| `fmn-render/src/vector_frame.rs`, `retained.rs` | `set_plan_limits`; all three non-test sync calls now use it |
| `fmn-render/src/plan.rs` | Doc comment only, including the repaint rule. **Defaults unchanged** |
| `fmn-render/tests/stroke_profiles.rs`, `fmn/tests/native_render.rs` | 7 new tests |

Why this shape:

- **Not a field on `RetainedFrameRendererConfig`:** that struct has 47 literal sites across the CLI, wasm, Python, studio and tests.
- **Not a field on `NativeFrame`:** it would risk the `#[expect(clippy::large_enum_variant)]` going stale.
- **Knots only:** raising the segment default would break `default_monotone_limits_cover_the_default_retained_plan`, and cells4 doesn't need it.

`lab-plate_lab.patch`, sha256 `02bc170c…`, declares the budget for the cells4 rung only.

## Tests, all unrun

| Test | What it should prove |
|---|---|
| `the_default_stays_where_every_existing_render_has_it` | Defaults pinned at `1 << 20` |
| `the_knot_boundary_is_exact_and_a_refusal_changes_nothing` | 16 × 16: 656 admitted, 655 refused asking 656, a limit of 122 refused asking 123; plan unchanged after each |
| `the_picture_that_crossed_the_default_is_admitted_by_a_declared_budget` | Real size: default refuses with 1,048,823; `1 << 22` holds 1,075,301 knots and 536,554 segments; monotone table and binning build at 720p and 4K |
| `a_repaint_needs_twice_the_live_knots_and_says_so_when_it_has_less` | 2L − 1 is a named refusal with no compaction; 2L is admitted; the next repaint compacts |
| `a_budget_moves_compaction_and_never_the_pixels` | Tight and default budgets give identical frame bytes over 6 repaints, and only the tight one compacts |
| `every_capture_route_answers_to_the_declared_budget` | Planar, camera and vector compiler routes |
| `a_declared_plan_budget_reaches_the_renderer_and_refuses_by_name` | Through `render_with_fs`, with nothing published |

The real-size test builds about 1.08M records, so it may be slow in a debug profile. The compiled worker route has no test of its own: only the code reading covers it.

## Ceilings the budget does not lift

All are fixed constants that refuse by name. None binds at cells4.

| Ceiling | Limit | z² cells4 at 4K |
|---|---|---|
| Records per stroke profile | 1,048,576 | 1,473 |
| Monotone pieces | 4,194,304 | At most 2,146,216 (bound of 4 per segment) |
| Monotone table bytes | 256 MiB | At most 98 MiB |
| Fine-tile references | 16,777,216 | About 122,430 |
| Fine tiles | 1,048,576 | 32,400 |
| Partition cells | 16,777,216 | 32,400 (one partition) |
| Binning working bytes | 256 MiB | About 1.8 MiB |
| Binning instances | 1,048,576 | 731 |
| Table and arena indices | `u32` | Far below |

**"Whatever we want" stops at these constants.** For example, z² at cells8 would be 4,293,905 knots, 8,587,810 on repaint, and 2,144,758 segments. It could declare those budgets, but on the 4-per-segment bound the monotone limit would then need its own setting, and I did not work out the true pieces per straight segment. Upstream 46d22343 raised that default and added fine-tile stroke bins. This patch includes neither.

## Memory and time, estimated and unmeasured

**No budget bounds this memory.** `max_resident_bytes` explicitly excludes scene geometry, so root's guards are the only limit.

| z² cells4 | Estimate |
|---|---|
| Stage records | 70 MiB |
| Live knots and style keys | 82 MiB |
| Segments | 45 MiB |
| **Still, geometry state** | **About 200 MiB** |
| Repaint peak, knots and keys | 246 MiB |
| During a rebuild, knots and keys | 328 MiB |
| Each in-flight frame, keys only | Up to 148 MiB more, because `Arc::make_mut` copies them; default 2 in flight |

Add per-worker monotone tables and frame buffers on top. A repaint could approach 1 GiB; treat that as a rough figure to measure.

**Raster is the likely real cost.** Extending the 720p log linearly in cells (z² blend 1.1 s, cells2 2.6 s) gives about 8.6 s a frame at cells4. That is a guess. I have no 4K timing to extend. Cells are 0.79 px at 720p and 2.36 px at 4K.

## Root's sequence

Unverified: I ran no Cargo. `$X` is this folder.

1. **Export.** Copy the pinned checkout to a new folder, then apply:
   ```sh
   cd "$EXPORT" && git apply "$X/engine-plan-limits.patch"
   ```
2. **Gate in the export,** with a new target dir:
   ```sh
   cargo fmt --check
   cargo clippy -p fmn-render -p fmn --all-targets -- -D warnings
   cargo test -p fmn-render
   cargo test -p fmn --test native_render
   ```
3. **Lab copy.** Copy `color-native-stage01/crate` to a new folder and apply `lab-plate_lab.patch` there. All 19 packages must be patched together. Patching `fmn-render` alone would bring in path copies of its siblings and duplicate their types.
   ```sh
   {
     echo '[patch."https://github.com/Dicklesworthstone/franken_manim"]'
     for c in fmn fmn-anim fmn-cache fmn-codec fmn-config fmn-core fmn-dmath fmn-frame fmn-geom \
              fmn-hash fmn-library fmn-mobject fmn-output fmn-platform fmn-render fmn-runtime \
              fmn-scene fmn-tex fmn-text; do
       echo "$c = { path = \"$EXPORT/crates/$c\" }"
     done
   } > engine-budget.toml
   cargo build --offline --config engine-budget.toml --example plate_lab
   ```
   This rewrites only the copy's `Cargo.lock`, so don't pass `--locked`. Use a new target dir. The canonical lock and pin stay as they are.
4. **Identity first.** Compare framemd5 between pinned and patched on an existing cells2 still and a blend fade. They must match.
5. **Density, under the guards.** z² cells4 still at 720p, then 4K, then sin(1/z). Record peak memory and time per frame.
6. **Repaint, if wanted.** The lab has no cells4 repaint job, so that needs `add(Kind::Repaint, &["z2"], &["cells4"])`. As a control, `1 << 21` should refuse asking 2,097,646.

An unpatched control also exists: a 721 × 721 still fits the default at 1,046,171 knots, and 722 × 722 does not.

If the pin later moves to upstream, check whether `plan_limits` exists there before relying on this. Upstream changed its defaults instead.

## Checked and not checked

**Checked:**

- The patch replays with `git apply` and `patch -p1 -F0` to the same bytes, and a changed source line makes it refuse.
- `rustfmt --check` passes on all 7 files. The lab example was already unformatted before my edit, so I left its formatting alone.
- The estimator's controls hold: the logged failure, an existing engine compaction test, and this patch's own test numbers.
- The pinned checkout is unmodified, and I made no edits in the canonical engine or crate.

**Not checked:**

- Cargo of any kind, so no compile, Clippy or test. API names in the tests were read from source, not built.
- Any render, memory measurement or high-density picture.
- The stepped layout's knot count against a real log. Only the blended layout has that control.
- All of the engine's `AGENTS.md`: I read the rule and gate sections and searched the rest.

**Not independent:** I wrote all of this and nobody has reviewed it. The edits were exact-string replacements on a scratch copy, each asserted to match once. The engine's `AGENTS.md` forbids script-based edits to its code files, so read the diff as a hand review would; the repo itself was not touched.

## Files

- `HANDOFF.md`
- `engine-plan-limits.patch`
- `lab-plate_lab.patch`
- `checks/demand.py`, `checks/demand.out`
- `checks/patch-replay.out`, `checks/rustfmt.out`
- `bundle.sha256`
