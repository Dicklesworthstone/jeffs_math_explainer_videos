# Secondary budget review: frame tables and camera

Author: WhiteSwan, for BlueSalmon. Reviewed: `engine-secondary-budget01` (`bundle.sha256` 8 of 8 OK) and, as separate scope, `engine-secondary-camera-budget01` (`bundle.sha256` 9 of 9 OK). Both are RubyRaven's. I wrote neither patch and no engine code, so this review is independent of both.

**This is source reading, Python arithmetic and one standalone `rustc` struct-size mirror.** I ran no Cargo build, test, render or memory measurement. I replayed the two patches in order onto copies of `crates/fmn` and `crates/fmn-render` from root's staged tree, and all 12 files both bundles expect match (`checks/route_census.out`). Line numbers are in that patched tree unless I say root's staged tree. In that case the file is one neither patch touches. Where I write "would", it comes from reading code, not from running it.

## Answer

1. **Propagation is sound.** Both budgets reach every facade route: serial and compiled-worker, affine and camera, and the kit hook. Every default is unchanged by text comparison, and the camera bin boundary at its default is the old strict `total > 1 << 24`. I found no reset, clamp or lost field.
2. **One refusal can lose its name (F1, Medium, both patches).** A worker refusal drops its emitter reservation before the run is cancelled. If the scene thread is waiting for a slot at that moment, the error reads "frame N reservation was abandoned before publication", with no budget named. The emitter's own design blocks that thread whenever the sink is slower than the renderer. The mechanism predates both patches, but they turn worker refusals into the normal way an author learns which budget to raise, and no test refuses mid-run behind a slow sink.
3. **The governed tables are bounded and fallible. Three costs next to them aren't, and for z² they're larger.** Affine plan copies (F2) and camera plan clones (F3) are deep, infallible and outside every budget. The affine arena's draw and stroke rows (F4) are infallible, and no budget charges them. On a still z² plate the arena holds about 112 MiB a team, while the plan's segments come to 63 KiB. Neither handoff adds up what coexists on one team (F5).
4. **Docs (F6, Low).** The frame-table hook example declares `max_pieces = 1 << 23`, which the default `max_table_bytes` caps at 5,592,405. The ignored control's memory figure is quoted two ways.
5. **Neither patch needs a change before Ruby's executable validation.** Most findings are about accounting, naming and what that run should measure. F1 deserves a test, and probably a small fix, before films lean on the refusal names.

## Evidence

| File | What it is | Result |
|---|---|---|
| `checks/route_census.py`, `.out` | Text census of root's staged tree (before), my patched copy (after) and the staged kit: defaults, propagation sites, the refusal path, plan copies, arena growth, camera reservations | 45 PASS, `RESULT PASS` |
| `checks/route_census.negative-control.out` | The same census with the unpatched tree given as "after" | 9 patch-specific checks FAIL as they should; the pre-existing-path checks still pass |
| `checks/struct_sizes_mirror.rs`, `.out` | Field lists copied from the engine types into a standalone file, built with the pinned nightly `-O` | Row sizes. A mirror, not the crate |
| `checks/budget_arithmetic.py`, `.out` | Every number below. Row sizes are read from the mirror and cross-checked against hand layouts | 49 PASS, `RESULT PASS` |

Run as `python3 -I -B checks/budget_arithmetic.py` and `python3 -I -B checks/route_census.py BEFORE_CRATES AFTER_CRATES KIT_SRC`. The census docstring says what each argument is. Mirror sizes from `rustc` and from hand layouts agree for all 13 types I hand-checked (`Draw` 512, `Style` 264, `PreparedSegment` 112, `JoinWedge` 104, `Segment` 88, `MonoPiece` 48, `ProjectedCurvePiece` 160 and others). Only a `size_of` inside the real crate proves them.

---

# Part 1: frame-table patch (`engine-secondary-budget01`)

## Confirmed sound, by reading

- **Every facade site sets the budget.** `fmn/src/rendering/pipeline.rs` sets plan and frame-table limits at the compiled-worker vector site (lines 244 and 245) and the serial vector site (484 and 485). There is one `NativeFramePipeline::with_limits` call, inside `RenderSink` (`rendering.rs:1054`). `render_cached_into` (`fmn-render/src/vector_frame.rs:190-211`) is the only affine executor on the facade, and it passes `frame_tables` to `MonoTable`, `Binning` and the frame-local pieces.
- **Defaults are preserved.** `MonoTableLimits`, `BinningLimits` and `RenderPlanLimits` have byte-identical `Default` impls before and after. `RenderOptions` defaults the new field to `FrameTableLimits::default()` (`rendering.rs:383`). `MonoTable::build`, `Binning::build`, `with_identity*`, `for_metal`, `NativeFramePipeline::new` and `RetainedFrameRenderer::render` keep the defaults, as the handoff says.
- **The kit hook survives every route.** In the staged kit, `cli.rs` runs `adjust(&mut opts)` after `camera_options` rebuilds the options and before both render calls. The repository root's own kit still has only `run_with_contract`, so the hook exists only in the staged kit.
- **Ruby's demand table reproduces exactly.** Re-deriving `bin.rs` term by term gives working bytes = I × 390,912 + 405,188 for I full-frame instances at 4K. All four rows (517, 518, 1,024 and 4,096 instances) match in references, working MiB and pruning MiB. A negative control (dropping the flags term) misses the 4,096 row by 506 MiB, so every term counts.
- **The binning peak is the prune copy, not a sum.** The build's temporaries are dropped before `prune_occluded` runs, and the prune peak (16 bytes a reference) exceeds the build peak (about 12) in every row. Ruby's "peak while pruning" column is the right number.
- **The 1080p control covers the frame.** At `frame_height = 4` the 8 × 6 polygon spans 2,160 × 1,620 px. Then 2,057 × 8,160 = 16,785,120 is the first layer count over 2²⁴, and working bytes (202,782,632) stay under the default 256 MiB, so the fine-reference refusal is the one named.
- **Large declared budgets stay arithmetically safe.** Sizes use checked arithmetic. Fine references, pieces and CSR offsets are held to `u32`, the per-instance count runs before any walk, and every table reserves fallibly. I found no overflow or truncation path for any `u64` a film can declare.
- **Teams follow `frames_in_flight`.** `render_teams.truncate(frames_in_flight)` (root's staged `fmn-runtime/src/plan.rs:536`), so Ruby's advice to lower it for memory is right.

## Findings

### F1 (Medium, both patches): a worker refusal can surface as `ReservationAbandoned`

**Path, all from source:**

1. `capture` and `capture_compiled` take a stream permit, then an emitter slot, whose error maps to `RenderError::Emitter` (`pipeline.rs:582-586`, `642-646`).
2. The emitter has `frames_in_flight` slots (`rendering.rs:1050`). A slot is freed only after the drain has delivered the frame to the sink (root's staged `fmn-output/src/emitter.rs:1086-1092`). Its module note says: "When every slot is outstanding, the next dispatch blocks" (lines 9-12). So whenever the sink is slower than the renderer, the scene thread waits in `wait_for_slot`, holding a permit.
3. `rasterize` destructures `let NativeFrameJob { frame, mut output } = job;` (`pipeline.rs:181`). A budget refusal returns through `?` and drops `output`.
4. `ReservationGuard::drop` records `ReservationAbandoned` and calls `notify_all` (`emitter.rs:877-901`). The runtime raises cancellation only after `rasterize` has returned (root's staged `fmn-runtime/src/pipeline.rs:1043`, `1077-1084`).
5. The waiting scene thread wakes and finds the failure first (`emitter.rs:560`), then returns `RenderError::Emitter(ReservationAbandoned)`. `SceneSink::capture` records it (`rendering.rs:1354-1365`).
6. `surface_failure` finishes the pipeline and returns the worker's own error only for `RenderError::Pipeline` (`rendering.rs:1103`). `compiled.rs:95` does the same. An `Emitter` error is returned as it stands.

**Failure scenario:** a 4K chapter encoding through x264 at a quality where ffmpeg is the bottleneck. At frame 900, one frame needs more tile references than declared. The author sees "frame 900 reservation was abandoned before publication" and nothing about `max_tile_references`. Nothing is published, so safety holds. Only the name is lost. A fast sink (PNG to `VirtualFs`, as in every new test) leaves the scene thread waiting on the stream instead, which sees cancellation and recovers the name. That is why the patches' tests can't see this.

**Source conclusion:** the path exists on both routes. Camera refusals drop `output` the same way (`pipeline.rs:184-187`). Neither patch changes it; both handoffs say refusals "refuse by name".
**Runtime proof needed:** a test with a sink that blocks, such as a `FrameSink` that sleeps, and a lowered budget that refuses at a frame past `frames_in_flight + 1`, asserting the error text names the budget. It runs in existing test binaries, with no 4K and no ffmpeg.
**Direction, not a patch:** in `surface_failure` and `compiled.rs`, treat `Emitter(ReservationAbandoned { .. })` like `Pipeline`: cancel, finish the pipeline and prefer its error.

### F2 (Medium, pre-existing, uncounted): every affine capture copies the whole plan while a frame is in flight

`VectorFrameCompiler::capture` runs `Arc::make_mut(&mut self.plan).sync_with_limits(...)` on every capture (`vector_frame.rs:106`). The only branch before it is the camera-primitive refusal. Each captured frame holds an `Arc` clone (`vector_frame.rs:108`), so in the pipelined steady state `make_mut` finds the count above 1 and deep-copies `RenderPlan`. That includes a still frame, because the copy happens before `sync` can tell whether anything changed. The doc comment says the opposite: "copied only when a subsequent capture mutates it while an earlier worker still holds it" (`vector_frame.rs:37-39`). The pin has the same code (`make_mut` at its line 66). `RenderPlan` derives `Clone` (`fmn-render/src/plan.rs:435`), so every `Vec` and `HashMap` is copied with infallible allocation.

| Case (`checks/budget_arithmetic.out`) | One generation | Live plus 2 in flight |
|---|---|---|
| z², rotation baked in (731 outlines, 536,554 segments) | 53.5 MiB | 160.5 MiB |
| z², rows sharing one outline | About 64 KiB of segments | Negligible |
| The ignored monotone control | About 146 MB | Not in its 0.51 GB |

The 53.5 MiB is segments (88 bytes each), arc-length tables (16 bytes a curve: `ArcLengthTable` has two `Vec<f64>`), shape rows (240) and instance rows (136). It is still a lower bound: style rows (264, plus a 45-word key `Vec` in the index) and the retained maps aren't counted. A cleared scratch `HashMap` clones at its full bucket count (hashbrown's `Clone`, as I know it; not checked against the vendored version).

**Failure scenario:** a rotating z² plate, `frames_in_flight = 2`. Every frame copies about 56 MB on the serial scene thread, even during holds, and up to three generations coexist. If an allocation fails, the process aborts with no refusal. The frame-table handoff's memory section doesn't mention plan copies (`HANDOFF.md:57-65`).
**Source conclusion:** certain from the text; the size is a lower bound.
**Runtime proof needed:** peak footprint and serial capture time on a rotating z² plate at `frames_in_flight` 1 and 2. A cheaper probe is to count `make_mut` copies, for example by `Arc::strong_count` before the call in a test build.

### F4 (Medium, pre-existing, uncounted): affine arena rows are infallible and outside every budget

`prepare` pushes one `Option<Draw>` (512 bytes) per instance (`engine.rs:1746`). It pushes prepared stroke rows (112 bytes a segment) for every stroked draw (`engine.rs:1682-1690`), transformed segments (88) for every non-translation placement (`engine.rs:1581`) and join wedges (104) for Bevel/Miter only (`stroke.rs` `join_wedges_into`). `Pool::put` is `self.buf.push(value)`, and `extend` calls `put` per row (`arena.rs:216-234`). No `try_reserve` exists for those four pools, and `begin_frame` keeps capacity for the team's life (`arena.rs:270-273`). `prepare` runs every frame, before the tile cache is consulted (`vector_frame.rs:203`).

These rows are per draw, not per unique shape, so sharing outlines doesn't shrink them:

| z², 731 rows × 734 segments, per team | Rows | Held after push growth |
|---|---|---|
| Still plate, shared outline, stroked | 57.7 MiB | 112.5 MiB |
| Rotation baked in, stroked, one generation | 102.7 MiB | 200.5 MiB |
| Two generations drawn | 205.4 MiB | 401.0 MiB |
| Governed tables, two generations (mono plus binning peak) | | About 101 MiB |

**Failure scenario:** a film raises the plan and frame-table budgets for a dense stroked plate, exactly as the handoff advises. The governed tables refuse by name at their declared sizes. The arena rows, about twice those tables here, grow without a check and abort if memory runs out. The handoff's arena bullet (`HANDOFF.md:64`) mentions only frame-local pieces.
**Source conclusion:** certain that these rows are unbudgeted and infallible. The sizes depend on the mirror and on push growth reaching the next power of two.
**Runtime proof needed:** the engine already has the counter. `FrameArena::stats().arena_buffer_bytes` sums every pool's capacity (`arena.rs:410-440`). Read it after one z² frame in a test.

### F5 (Medium, accounting): what coexists on one team is never summed

Each handoff bullet is right on its own: mono up to twice its budget, prune outside the working budget, "budget × teams". For the frame-table hook example (references 2²⁵, working bytes 2²⁹, default table bytes), the budgeted ceiling per team is:

| Item | Bytes |
|---|---|
| Shared monotone table | 256 MiB |
| Frame-local pieces (checked separately, capacity kept) | 256 MiB |
| Binning peak while pruning at 2²⁵ references | 512.25 MiB, just above the declared 512 MiB and outside it |
| Pixel tile cache, every tile held (RGBA16F) | 63.3 MiB |
| **Per team** | **1.062 GiB** |
| **Two teams (default `frames_in_flight`)** | **2.124 GiB**, before arena rows (F4), plan generations (F2) and raw frames |

This is a ceiling, not a demand: one scene doesn't have to reach every term. Nothing in admission sees it. `max_resident_bytes` covers only frame storage, and the planner's `memory_slot_cap(total_memory, bytes_per_frame)` (root's staged `fmn-runtime/src/plan.rs:903`) counts only frame bytes when it sizes the team count.
**Source conclusion:** the arithmetic is in `checks/budget_arithmetic.out`.
**Runtime proof needed:** peak footprint of one guarded 4K frame at declared budgets, with the team count recorded.

### F6 (Low, docs): the hook example and the memory figures

- `HANDOFF.md:12-16` declares `max_pieces = 1 << 23` with default `max_table_bytes`. With one shape range the table bytes admit 5,592,405 pieces, so anything above that is refused as "monotone table bytes" and the raise is inert. The handoff notes the crossing at line 80, but the example a film would copy doesn't follow it.
- The ignored monotone control is "about 0.51 GB" in `BUILD.md:46` and "about 0.55 GiB" in its `#[ignore]` text. The handoff's own sum is 514,534,020 bytes: 0.51 GB, or 0.48 GiB. Neither figure includes arc-length tables (22.4 MB) or a plan copy (about 146 MB) if a second capture overlaps the first frame.

### F7 (Low, runtime only): fallible isn't the same as safe

Every governed table uses `try_reserve`, which names a failure only when the allocator refuses. On macOS, a large reservation usually succeeds and the pages are committed later, so memory pressure can end in a jetsam kill instead of a named refusal. That is general OS behaviour, not something I read in this code. **Runtime proof needed:** none to run. The guarded run should record peak footprint against the guard rather than treat "no named refusal" as proof of safety.

### F8 (Low, time): what raised budgets make slower, silently

`Binning` classifies every reference with `covers_tile` (`bin.rs:871-878`). For closed polylines and general hints that calls `covers_convex`, which builds a `Vec` of 8 points per call (`bin.rs:1221`). The admitted 1080p control makes 16,785,120 such calls per frame in an unoptimised test binary (the engine's root `Cargo.toml` defines only release profiles). That is a cost to expect in the run, not a defect. At film scale, references are what the budget raises, so classification time grows with them.

### Test coverage

Neither patch adds a mid-run refusal, a slow-sink refusal (F1), a film-level probe or an e2e scenario (fm-fjq), as both handoffs acknowledge for the last two. Every new refusal test refuses on the first frame or through `finish`.

---

# Part 2: camera follow-on (`engine-secondary-camera-budget01`), separate scope

## Default and refusal preservation, by reading

- **Defaults are byte-for-byte the old values.** The `ThreeDPreparationLimits` default (`three_d.rs:80`) is the old text plus one line, `max_curve_bin_entries: 1 << 24`, which equals the old `CURVE_BIN_MAX_ENTRIES`. `RenderOptions` defaults the field (`rendering.rs:384`). `RetainedFrameRenderer` starts at the default, and `ThreeDJob::new` still uses it.
- **The bin boundary is unchanged at the default.** Before: `if total > CURVE_BIN_MAX_ENTRIES`. After: `if total > curve_bin_entry_limit(budget.limits)` (`three_d.rs:1309`), with `min(declared, u32::MAX)` (`three_d.rs:1159-1161`). Same strict comparison. In both, fewer than 32 curves returns before any entry limit is read, and the skip returns `Ok(None)`, so a bin budget changes the search, never a refusal.
- **The four test literals set to 0 change nothing.** They sit in tests whose scenes have fewer than 32 curves (two four-corner squares), a dot, and a surface (`three_d.rs:4538`, `4637`, `4725`, `4789`).
- **Propagation.** Serial (`pipeline.rs:477-478`) and compiled-worker (`225-226`) camera sites set the limits, `render_camera_scene` builds with `ThreeDJob::new_with_limits`, and the kit hook covers `request_camera` chapters (census).
- **The fill rename keeps admission.** The piece limit is `min(rows left, bytes left / row size)`. When rows still fit, the request exceeds the byte remainder, so naming working bytes is accurate, and the admit/refuse decision is the same comparison as before. Reasoned, not run.
- **Overflow.** `build_curve_bins` caps cells at 2²², charges its three reservations exactly as computed, and keeps entries within `u32`. Bins are charged to working bytes, so a huge `max_curve_bin_entries` can't exceed `max_working_bytes`.

## Plan clone and allocation

### F3 (Medium, pre-existing; extends the handoff): the camera plan clone is bigger and more frequent than stated

The handoff's table is right that the clone is unbudgeted and aborts on failure (`HANDOFF.md:52`). Three additions:

- **Size.** `prepare_with_camera` stores `plan: self.plan.clone()` in each `PreparedCameraFrame`, whose `plan` is a bare `RenderPlan`, not an `Arc` (`retained.rs:326`, `362-368`). "88 a segment" misses the arc-length tables: one z² generation with rotation baked in is at least 53.5 MiB, not 45, before styles and maps. With rows that share one outline, the clone is small, and the handoff's 45/90 MiB lines apply only when every row has its own outline.
- **Frequency.** It happens on every camera capture, including holds, so the serial scene thread copies the whole plan once per frame.
- **"An `Arc` wouldn't help a moving scene"** (`HANDOFF.md:61`) is true. It could help holds only if sync were skipped when no revision changed, because the affine route's `make_mut` copies even still frames (F2).

**Runtime proof needed:** the same as F2, on the camera route.

### F9 (Low-Medium, pre-existing; quantifies the handoff's note): projected curves grow one row at a time

`compile_vector` starts `curves` as `Vec::new()` per vector per frame (`three_d.rs:1789`), and each curve calls `reserve_rows(output, ProjectedCurves, 1)` (`three_d.rs:2146`), which is `try_reserve_exact(1)` (`three_d.rs:562-575`). A full `Vec` therefore grows by exactly one row each time. For z² with one curve a segment, that is 536,554 reallocation calls a frame. If each one moved the buffer it would copy 43 MB a row, 31.5 GB a frame. Whether realloc grows in place is allocator-dependent, so the copy volume is a worst case. The call count isn't. The handoff says it "may reallocate repeatedly" and keeps exact growth "since looser growth would hold more than the budget admits". Doubling, capped at `remaining_rows`, would keep what's charged unchanged and hold at most twice it. That is a design option for root, not a defect in this patch.
**Runtime proof needed:** frame time of one z² camera frame at 1, 2 and 4 curves a segment. The camera route has no reallocation counter.

### F10 (Low): Bevel/Miter strokes run a joins × curves search

For each join, `compile_vector` scans every curve for the nearest end (`three_d.rs:1955-1960`). That only applies to Bevel and Miter (`build_join_wedges` returns empty for Auto and NoJoint, `three_d.rs:2071`). The corpus sets no `joint_type`, so no film reaches it today. It belongs on the handoff's silent-slow list.

### F1 also applies here

Camera rasterize returns through `?` with `output` live (`pipeline.rs:184-187`), so a camera refusal behind a slow sink loses its name the same way. "Refusals happen on the worker before the frame is published" (`HANDOFF.md:37`) holds; "refuses by name" doesn't, in that case.

---

## What Ruby's executable validation could add, cheaply

None of these needs 4K, ffmpeg or a second heavy job:

1. **F1:** a sink that blocks, plus a budget that refuses after `frames_in_flight + 1` frames, asserting the budget's name. One per route if cheap.
2. **F4:** `FrameArena::stats().arena_buffer_bytes` after one z² frame, still and rotating.
3. **F2/F3:** peak footprint and serial capture time at `frames_in_flight` 1 and 2 on one rotating z² frame, both routes.
4. **F5:** record the team count and peak footprint beside any guarded run at declared budgets.
5. **F9:** one camera z² frame's time at 1 and 2 curves a segment.

## Limits of this review

- No Cargo, test, render or memory measurement. Every size comes from a mirror or from arithmetic, and every "would" from reading.
- The struct-size mirror copies field lists with the `metal` feature off. Only a `size_of` in the crate proves them.
- The suite-wide rules file that the engine's `AGENTS.md` names (RULE 0.5) isn't on this machine, so I read only the engine's own `AGENTS.md`.
- I didn't read Ruby's in-progress `engine-secondary-validation01` beyond its header, and I edited nothing outside this folder.
- Production stays native FrankenManim with ElevenLabs Oliver; nothing here touches narration.

## Files

| File | Purpose |
|---|---|
| `REVIEW.md` | This review |
| `checks/route_census.py`, `.out` | Text census, before and after |
| `checks/route_census.negative-control.out` | The census with the unpatched tree as "after" |
| `checks/struct_sizes_mirror.rs`, `.out` | Row-size mirror and its output |
| `checks/budget_arithmetic.py`, `.out` | Every number above |
| `SHA256SUMS` | Every file here |
