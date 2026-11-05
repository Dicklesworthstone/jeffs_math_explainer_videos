# Geometry budget cross-review

Author: LavenderPike, for BlueSalmon's Mails 2938 and 3006. Reviewed: `engine-geometry-budget01`, every file matching its `bundle.sha256`.

**This is source reading and Python mirrors only.** I ran no Cargo, render or memory measurement. Every "should pass" and "should fail" below comes from reading the code, not from running it.

## Answer

1. **Keep the design and `1 << 22`.** The knot figures hold, and the plumbing reaches every route through the `fmn` facade. I found no hidden budget reset inside the engine.
2. **Don't apply `engine-plan-limits.patch` unchanged.** One new test should fail its gate on a wrong segment count (F1). The fix is one line, and the same mistake runs through the handoff's segment, monotone, memory and cells8 figures.
3. **The film cannot use the budget yet.** The shared kit rebuilds `RenderOptions` for delivery renders and would drop the field. The glyph probe and thumbnail would also refuse a cells4 plate (F2). `patches/` has staged wiring that leaves other films alone.
4. **A cells4 comparison needs a fade.** Every plate enters and leaves by a fade, and each fade frame is a whole-plate repaint. The lab has no cells4 fade, and the handoff calls a repaint optional (F3).
5. **Mail 3006's "point motion needs 1,073,108 segments" needs a condition.** It holds only when each row gets its own outline. Mail 3006's caution still stands: the knot limit does not permit arbitrary moving geometry (see Point motion).

This review does not rule on cells4. My earlier rejection in `complex-color-comparison01` rested partly on the old cap, which the owner has lifted. That folder stays frozen, and the actual 4K comparison decides.

## Corrected demand

Knots, full-repaint knots and input curves stand exactly as the handoff has them. Only segments change. Numbers are from `checks/geometry.out`.

| | z² cells4 | sin(1/z) cells4 | Six panels |
|---|---|---|---|
| Knots, still | 1,075,301 | 824,960 | 1,726,758 |
| Knots, full repaint | 2,150,602 | 1,649,920 | 3,453,516 |
| Input curves | 538,016 | 412,800 | 864,270 |
| Segments, handoff | 536,554 | 411,520 | 860,706 |
| **Segments, by reading** | **734** | **643** | **1,449** (3 outlines) |
| Segment room under `1 << 20`, handoff | 512,022 | 637,056 | 187,870 |
| **Segment room, by reading** | **1,047,842** | **1,047,933** | **1,047,127** |

| z² cells4 | Handoff | By reading |
|---|---|---|
| Monotone pieces, upper bound | 2,146,216 | 2,936 |
| Monotone table | 98 MiB | About 137 KiB |
| Segment memory | 45 MiB | About 63 KiB |
| Still, geometry state | About 200 MiB | About 152 MiB |

Sharing shrinks only the tables. Raster still walks every instance, so none of this lowers the raster estimate.

## Point motion

For a plate as the film builds it (mirror in `checks/geometry.out`, section "Point motion"):

| Motion | Outlines of 731 | New segments a frame | What it needs |
|---|---|---|---|
| Translation through a placement | Shape reused (`plan.rs:2295`) | 0 | Nothing |
| Translation or uniform scale baked into records | 1 | 734 | Nothing |
| Rotation baked into records | 731 | 536,554 | See below |
| A warp | 731 | 536,554 | See below |

Rotation splits the rows because a point's x then depends on its y, so f32 rounding differs row by row.

When every row has its own outline:
- The second motion frame asks for 1,073,108 segments. With nothing yet unreachable, that is a named refusal under `1 << 20`.
- With `max_retained_segments` raised, 536,554 unreachable segments pass `max_unreachable_segments` (65,536), so every later frame rebuilds.
- At the 4-a-segment bound, that is 4,292,432 monotone pieces against a fixed `1 << 22`. `MonoTable::build` uses its defaults in the render path, and `RenderOptions` cannot change them. I did not work out the real pieces per straight segment.

By `rg` over the chapters, plate groups only fade and recolour, so no film plate reaches this today. This agrees with Ruby's reading, but I read only the group uses `rg` found.

## Findings

Severities are my own judgement.

### F1, High: rows share one outline, so a new test should fail

`shape_digest` hashes each point minus the first (`table.rs:979`), and `plan.rs:893-899` shares a matching shape. Existing tests show it: `plan.rs:1530` (three copies, one outline) and `plan.rs:1608` (8 copies, 2 segments).

The patch's `row_points` uses the same x in every row and one y across a row. My f32 mirror finds 1 outline in 731 rows, and it tells a row apart when one x moves by 2^-20.

**Failure:** `the_picture_that_crossed_the_default_is_admitted_by_a_declared_budget` asserts `plan.segments().len() == 731 * 734`. By reading the value is 734, so root's `cargo test -p fmn-render` gate goes red.

**Fix in the test:**
```rust
assert_eq!(plan.segments().len(), 734);
assert_eq!(plan.shapes().shapes().len(), 1);
assert_eq!(plan.stats().shapes_shared, 730);
```

**Handoff lines to replace:**
- The Segments row, and both Room rows in the overlays table: use the table above.
- "Moving a plate is different...": use the Point motion section.
- The monotone row in the ceilings table: 2,936 pieces at most.
- The memory table's Segments row and still total.
- cells8: "the monotone limit would then need its own setting" is wrong. Segments are 1,466 and pieces at most 5,864. The real limit, which the handoff doesn't mention, is `max_input_curves`: 2,147,684 demanded, crossing at row 715 asking 1,049,620.

**`demand.py`:** `"segments": rows * seg` has no interning, and none of its controls ties segments to anything. "This patch's own test numbers" is circular as a control, which is how this survived.

### F2, Medium: the film can't reach the budget, and setting it in the obvious place loses it

The film's options are built in the kit, not the film (`RenderOptions` appears in no file under the film's `src/`). Three things break:

| Site | What happens at cells4 |
|---|---|
| `crates/explainer_kit/src/cli.rs:648-661`, every `--crf` render | Builds a new `RenderOptions` and copies only `config` and `max_frames`. A field set at `:633` is lost, so 720p drafts pass and the delivery render refuses by name |
| `cli.rs:536-546`, `camera_options` | Same rebuild. This film has no camera shots, but the kit is shared |
| `crates/explainer_kit/src/glyphs.rs:85-89`, `cli.rs:1025` | The glyph capture builds each chapter's real geometry at 32 × 18, and the thumbnail renders too. Geometry doesn't depend on frame size, so a plate over the default refuses |

The kit also can't name the field outright. It is a workspace member shared with films on the unpatched pin, so a plain `opts.plan_limits` there would stop them compiling. A kit Cargo feature enabled only by this film could work, because this film sits outside the workspace, but a hook needs no conditional code.

**Fix, staged in `patches/`:** see Kit wiring.

### F3, Medium: a fade is a whole-plate repaint, and the plan doesn't test one

- `Painter::show` and `leave` fade the whole group (`draw.rs:909`, `:930`). `fade_in` is a `Transform` that writes `stroke_rgba` on every frame (`transform.rs:66`, `fading.rs:52`), so every row gets a new style and new knots.
- Points are locked and not rewritten, so a fade costs knots only. The cadence is the repaint's: z² cells4 at `1 << 22` compacts every 3 frames, and `1 << 21` refuses on the second frame of any z² fade.
- The film's first full-size use of a plate is therefore a fade. The handoff's "Repaint, if wanted" should not be optional, and a still-only comparison cannot decide cells4.
- `Transform` also takes two copies of the family (`transform.rs:506-512`, `:549-552`), which adds 140 MiB of records to the handoff's memory table.
- Fades are where `overlap` failed after looking clean as a still.

**Fix:** add a z² cells4 fade job, either way:
- in `plan()`: `add(Kind::Fade, &["z2"], &["cells4"]);`, with the frame-total test becoming `67 + 84 + 9` (160, exactly at `FRAME_LIMIT`);
- or outside the plan, beside Ruby's probes in validation02.

By my mirror, its seven frames at `1 << 22` include one compaction (`checks/admission.out`). Ruby's probes repaint but don't fade, and they don't cover the alpha path.

**Don't stack my frozen `probe-plate-lab.patch` for this comparison.** It applies with no warning on top of Ruby's lab patch (`patch -p3 -F0` exits 0), and it removes `add(Kind::Still, &["z2", "sinrecip"], &["cells4"])`.

### F4, Low: no Cargo test proves a raise through the facade

`a_declared_plan_budget_reaches...` lowers segments to 1, `every_capture_route...` lowers knots to 1, and validation02's compiled test lowers to `demand - 1`. A clamp such as `min(declared, default)` passes all three. Root's first z² cells4 still would catch it, so this is about pinning it in tests.

- **To pin it:** one scene through `render_with_fs` with 8 rows of 65,600 cells (8 × 131,209 = 1,049,672 knots). Refused at the default, published when declared.
- `the_knot_boundary...`'s "a refusal changes nothing" runs on an empty plan and compares counts. `fmn_render::snapshot::encode(&plan)` is the stronger form (`stroke_profiles.rs:251-263`), and a non-empty plan would test more.
- `a_budget_moves_compaction...` uses `RetainedFrameRenderer::render`, not the film's route (`VectorFrameCompiler` with an earlier frame in flight across a compaction).

### F5, Low: "reaches every capture route" is true of the facade only

These non-test routes stay at the default with no option: `fmn-wasm/src/lib.rs:482`, `fmn-wasm/src/camera_player.rs:24`, `fmn-studio/src/camera_bundle.rs:208`, `fmn-studio/src/native/raster.rs:45`, `fmn-python/src/portal_readback.rs:216` and `fmn-python/src/portal_studio.rs:116` (`checks/sites.out`). So do `fmn-cli/src/lib.rs:3163` and `:4488`, `offline_cpu.rs:242` and `studio_live.rs:912`, which I found by direct search because the script's simple rule hides them. The film doesn't use any of these, so this is only a wording fix.

### F6, Low: memory and time estimates leave out some costs

All arithmetic, none measured. For z² cells4:

| | Handoff | With what was left out |
|---|---|---|
| Stage records in a fade | 70 MiB | 209 MiB |
| Knots and keys during a rebuild | 328 MiB | 377 MiB, with the pending index's keys |
| Upper bound in a fade, before frame buffers and caches | Not given | About 882 MiB, if a rebuild and two in-flight frames at 3L coincide |

The handoff's "approach 1 GiB" is still a fair rough figure, but its route should be a fade.

Scene-thread work each capture, holds included, also goes uncounted in "raster is the likely real cost":
- identity hashing: 30.1 MB at cells4, 7.6 MB at cells2 (`plan.rs:1126`);
- key copies when a frame is in flight: 49 MiB at L, 148 MiB at 3L (`vector_frame.rs:66`).

I have bytes only and can't say how much time they take. The 8.6 s figure also remains a 720p extrapolation.

## Kit wiring

All three patches are uncompiled. Each replays with `patch -p1 -F0` on the current bytes, and the kit patch reverses back to them (`checks/wiring-replay.out`). `rustfmt --check` passes on every changed file.

| Patch | sha256 | Applies to | When |
|---|---|---|---|
| `kit-render-options-hook.patch` | `c838f471…` | Kit copy | For any film use. It never names the field, so by reading it builds on the unpatched pin too |
| `film-declare-plan-budget.patch` | `ba322818…` | Lab copy's `src/main.rs` | Only with the engine patch |
| `select-cells4.patch` | `9f0f44fc…` | Lab copy's `draw.rs` | Only if cells4 goes into film chapters |

**The design.** A new `run_with_contract_and_options(e, argv, topic, adjust)` takes a plain `fn(&mut RenderOptions)`, and `run` and `run_with_contract` pass a no-op, so no other film changes. `adjust` runs last at all four sites: chapter, camera chapter (after each rebuild), thumbnail and glyph capture. The glyph sheet at `glyphs.rs:140` keeps its defaults because it draws no chapter. The film declares `max_retained_profile_knots: 1 << 22` and nothing else.

**Canonical route, if cells4 wins.** `complex_analysis_visual` is excluded from the root workspace and has its own `[workspace]` and lock, so it can take the engine `[patch]` and the film patch without moving any other film's pin. The kit is a workspace member, and by reading the hook patch builds for all of them. None of this is applied, and both defaults and the pin are untouched.

**Isolated build, unverified:**
1. Make a kit copy. Inheritance needs the root manifest beside it:
   ```sh
   mkdir -p "$KIT/crates"
   cp Cargo.toml "$KIT/"
   cp -R crates/explainer_kit crates/explainer_score "$KIT/crates/"
   (cd "$KIT" && patch -p1 -F0 -i "$X/patches/kit-render-options-hook.patch")
   ```
2. In the lab copy's `Cargo.toml`, point the two absolute `explainer_kit` and `explainer_score` paths at `$KIT/crates/...`. `[patch]` can't redirect a path dependency, so this has to be an edit.
3. In the lab copy, apply `film-declare-plan-budget.patch` and `select-cells4.patch`.
4. Add Ruby's engine `[patch]` config and build as validation02 says, with no `--locked`.

**Proof the wiring works, in the isolated copy:**
- `render 04_reading_color --res 1280x720 --fps 15 --silent --crf 23 --max-frames N` is admitted. Choose `N` to pass the plate's first fade.
- **Control:** the same command with `select-cells4.patch` but without the film patch should refuse, asking 1,048,823. This shows that cells4 is really in effect and that the default still applies.
- `glyphs` and `thumbnail` complete under the same binary.

`cargo test -p explainer_kit --test cli_pipeline the_options_hook` covers the order and sites. It needs ffmpeg on PATH. I am unsure of its `--silent --crf` path until it runs.

## Confirmed by reading

- **Design:** a knots-only raise is right, and more so with the segment correction.
- **Repaint rule:** a sync can only reclaim rows already unreachable when it begins, so straight after a compaction old and new must fit together. My own mirror of `plan.rs:696-721` reproduces the z² and panel refusals and the cadences (`checks/admission.out`). Agreement between the mirrors isn't independent proof, since both come from reading the same code.
- **Six panels:** the 3,453,516 row is a safe bound and not the film's demand. The panels fade one at a time, so the live six plus the one fading need 2,014,551 knots, and `1 << 22` is 2.08 times that. z² at 2,150,602 is the film's real worst case.
- **Compaction:** `rebuild_at_safe_point` uses the limits passed in, with no reset. `*self` is replaced only after success, and errors leave the plan as it was except for scratch.
- **Plumbing:**
  - no plain `.sync(` is left in `retained.rs` or `vector_frame.rs` after the patch;
  - all four constructors in `pipeline.rs` take the budget;
  - lazily created worker compilers use `self.plan_limits`;
  - nothing on the patched paths adds a snapshot, journal or second limit.
- **Overflow:** no overflow path at these sizes. Knot limits are `u64`, and style rows stay far under `u32`.
- **Lab patch:** by reading it compiles, and the lab route is Vector. It applies on the current bytes, and the opt-in follows the candidate, so a cells4 fade is covered.
- **Kit guard:** `GROUP_TRANSFORM_LIMIT` is 2,000, and cells4's 731 rows pass it.
- **Tests:** every new test but the one in F1 should pass by my trace.
- **Replay:** the engine patch replays on the pin with `-F0`. The pin is clean except for `.cargo-ok`.

## Gates that have not run

- All of root's engine gates: `cargo fmt --check`, Clippy, `cargo test -p fmn-render`, `cargo test -p fmn --test native_render`.
- Any compile of the three patches here, and the kit's `cli_pipeline` tests.
- The film's `draw.rs` tests under `select-cells4.patch`.
- Any render, at any size, and any memory or time measurement. There is still no high-density picture from this work.

## Limits

- **Not checked:**
  - Cargo's handling of the kit copy's workspace, including a `members` glob with no `explainer_corpus/` directory;
  - Ruby's `verify_patched_lock.py` against the changed kit path;
  - real monotone pieces per straight segment;
  - the 122,430 fine-tile reference estimate, which I only sanity-checked by hand against the plate's size in tiles.
- **Film overlays:** I didn't count overlay knots or curves. I rely on Ruby's reading that overlays make no profiles.
- **Validation02:** I read only parts of `lab-probes.patch`, `engine-compiled-route-test.patch` and its manifest output, and did not review that folder.
- **The mirror:** it keeps the shape digest's equality classes, not its bytes. Motion cases run on one frame, and a real animation's placement handling could differ.
- **`checks/sites.py`:** it treats everything after a file's first `#[cfg(test)]` module as test code, and skips `*_tests.rs`. That hides real sites in `fmn-cli/src/lib.rs`, so its list is a floor, not a full inventory. The film's route is covered separately by reading `pipeline.rs` and `rendering.rs`.

## Authorship and shared memory

This is a cross-review, not an independent review. I wrote the plate lab, `PLATE_CELLS4`, the `2n + 9` knot derivation and the frozen rejection of cells4 that this work revisits, so I am not neutral on the knot figures. RubyRaven and I use the same model and share one auto-memory directory, so we may share blind spots.

The three patches and the mirrors here are my own, and nobody has reviewed them. The kit patch in particular needs a reader other than me.

## Files

- `REVIEW.md`
- `patches/kit-render-options-hook.patch`, `patches/film-declare-plan-budget.patch`, `patches/select-cells4.patch`
- `checks/geometry.py`, `checks/admission.py`, `checks/sites.py`, with their `.out` files
- `checks/wiring-replay.out`
- `SHA256SUMS`
