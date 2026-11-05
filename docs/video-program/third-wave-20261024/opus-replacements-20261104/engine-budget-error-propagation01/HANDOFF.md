# Bead39: a worker's budget refusal keeps its name (F1)

WhiteSwan, 2026-11-05. A narrow engine repair and its regression tests, sealed for RubyRaven's
guarded attempt04. **Nothing here has been compiled, linted or run.** The source checks below are
text and format checks only, so no native pass is claimed. Compile, Clippy and tests belong to the
guarded serial lane.

## Files

| File | What |
|---|---|
| `engine-budget-error-propagation.patch` | git-style patch (`a/`, `b/`), 2 changed files + 1 new |
| `baseline-engine.sha256` | the two changed files as the patch expects them |
| `expected-engine.sha256` | the three files after the patch |
| `checks/source_checks.py`, `checks/source-checks.out` | replay, hash and rustfmt checks (RESULT PASS) |

## Baseline

Line numbers in `crates/fmn` are post-patch unless marked baseline; other crates are unpatched.

Root's engine-secondary-validation01 stage, as it stood at sealing. Its `crates/fmn` was
byte-identical to my baseline copy (`diff -r`).

- `crates/fmn/src/rendering.rs` `0a90d488…`: the stacked state, equal to the `rendering.rs` row of
  `engine-secondary-camera-budget01/expected-engine.sha256`.
- `crates/fmn/src/rendering/compiled.rs` `28b2a6f2…`: no stack entry touches it.

The patch shares no file with the three STACK entries or with root's extended
`test-type-alias01.patch` (`native_render.rs`, `camera_route_limits.rs`, `frame_table_limits.rs`).
`native_render.rs` is untouched, and so is every assertion in it.

## The defect

Root relayed the native result: attempt03, engine-test03, exit 101. `native_render.rs:244` got
`Emitter(ReservationAbandoned { sequence: 0 })`; 33 tests passed and 1 failed. (RubyRaven's
Mail 3548 gives the same; I have not read that log myself.)

How it happens, at stage source:

1. `NativeFramePipeline::capture` takes a stream permit, then an emitter slot
   (`rendering/pipeline.rs:582`, `:585`).
2. `rasterize` owns the slot (`pipeline.rs:181`). A refusal returns through `?`, and the
   reservation drops as it does.
3. `ReservationGuard::drop` records `ReservationAbandoned` and wakes waiters (fmn-output
   `emitter.rs:877-899`). The emitter keeps only its first failure (`emitter.rs:999-1004`).
4. The runtime raises cancellation only after `rasterize` has returned (fmn-runtime
   `pipeline.rs:1043`, then the `Ok(Err(source))` arm at `:1078`).
5. `FrameStream::reserve` checks cancellation, then takes a ticket that may already be waiting
   (fmn-runtime `stream.rs:134-150`). Emitter `reserve` returns a recorded failure before anything
   else, including after each wait (`emitter.rs:555-583`).
6. At baseline the facade read the pipeline's recorded stage error only for `RenderError::Pipeline`
   (`rendering.rs:1103`, `compiled.rs:95`).

A capture fails with `ReservationAbandoned` in two cases:

- **Straddle:** it passed the stream check before cancellation, then reached the emitter after the
  drop. This is root's failure: frame 0 refused, and the scene was at capture 1.
- **Full ring:** it was already waiting on a full ring when the slot dropped.

Either way, the facade returned the abandonment and the refusal's name was lost.

## The repair (`crates/fmn/src`)

- **`rendering.rs:629`** adds a private `RenderError::may_hide_stage_failure`. It is true for
  exactly two cases: `Pipeline(_)`, or `Emitter(ReservationAbandoned { .. })`.
- **`surface_failure`** (`rendering.rs:1121`) and **the compiled loop** (`compiled.rs:95`) now use
  it in place of `matches!(…, Pipeline(_))`. The recovery steps are unchanged: wake output waiters
  (`emitter.cancel()`), then join the workers (`pipeline.finish()?`). If the join is clean, the
  original capture error is returned, as before.
- **Unit test** `rendering::tests::only_a_closed_stream_or_an_abandoned_slot_is_traded_for_the_stage_failure`
  pins the predicate's truth table. It is false for `SinkFailed`, `SinkPanicked`,
  `ReliableSinkWouldBlock`, `Cancelled`, `Closed` and `InvalidOptions`. The other emitter variants
  (`InternalInvariant`, `WorkerPanicked`, `ThreadSpawn`, …) are not in the `matches!` either.

Why a sink failure can never be traded away: the emitter keeps its first failure.

- If a capture sees `ReservationAbandoned`, the abandonment came before any sink failure.
- If the sink failed first, every later reserve and publish sees `SinkFailed` (or the sink's own
  variant), and the predicate rejects it.

The patch adds no new wrapping of sink errors.

## RubyRaven's checks (Mail 3564), answered from source

**Named refusal on the first frame** (`native_render.rs:235`, both halves, unchanged).

- Each half refuses frame 0. The scene then fails with `ReservationAbandoned{0}` or a closed
  stream. Both are now traded for `pipeline.finish()`'s `Stage{0, Raster, …}`, which carries
  `"monotone pieces"` or `"macrotile references"`.
- No frame of that scene is ever emitted, because every frame refuses. So residual R1 below cannot
  occur there.

**Every door.** These are the only three `RenderSink` owners (`RenderSink::new` at
`rendering.rs:685`, `compiled.rs:88`, `camera_animation.rs:66`):

- `render_with_fs` (and `render`) goes through `surface_failure`.
- `render_bundle_with_fs` (and `render_bundle`) goes through the compiled loop.
- `render_camera_with_fs` (and `render_camera`): its `RigSink::capture` records into
  `inner.failure`, and `surface_failure` reads it (`camera_animation.rs:88`, `:139-148`).

The no-capture static-frame calls (`rendering.rs:697`, `camera_animation.rs:94`) submit the only
frame. No earlier worker exists to abandon a slot, and `finish` joins the pipeline before it drains
the output.

**Genuine sink errors keep their identity.**

- A capture that sees `SinkFailed` or `ReliableSinkWouldBlock` returns it unchanged, with the same
  variant.
- One path does surface a sink failure inside `Pipeline(Stage{Emit, Emitter(SinkFailed…)})`. It
  predates this patch and is unchanged: the coordinator's publish fails, so the scene sees a closed
  stream. The negative test therefore asserts the sink failure's words rather than one variant,
  since thread order picks between the two at baseline too.
- No sink or refusal classification test changes.

**Precedence by cause.**

- The coordinator keeps the first error to arrive (fmn-runtime `pipeline.rs:1199-1204`).
- A later frame on the same render team is dropped as `Cancelled` before it rasterizes. That
  records no error (`render_worker` checks cancellation per job; `accept`'s `Cancelled` arm at `:1391`).
- Outputs that arrive after cancellation are dropped without being emitted.
- So the recovered error is the refusal, with one exception, R1, below. With several render teams,
  two frames can refuse; the first to arrive wins, and both are named refusals.

**Clean exit.**

- `pipeline.finish()` joins the workers before the error returns.
- Both new tests assert that nothing is published. Each then retries to the same destination with
  default budgets and an unheld sink, and asserts that the retry publishes.

**Test.** Its own section follows. Expected hashes are against the stacked bytes, above.
The patch adds **3 tests**:

- the unit test above;
- `slow_sink_budget_refusal::a_worker_budget_refusal_behind_a_held_sink_keeps_its_name_and_publishes_nothing`;
- `slow_sink_budget_refusal::a_genuine_sink_failure_stays_the_sinks_and_publishes_nothing`.

**No suppression.** No `allow` and no lint attribute. rustfmt `--check` is clean (pinned nightly,
edition 2024). Clippy has not been run. The test gate is a `Mutex<bool>` with a `Condvar`. I know
of no `clippy::all` lint on that pattern, but I did not confirm it (see the incident below).

## The regression test (`crates/fmn/tests/slow_sink_budget_refusal.rs`)

**Setup.**

- `GatedFs` wraps `VirtualFs` and delegates every method. Streamed files get a writer whose second
  call, frame 0, is held or refused; the first call is the Y4M header (fmn-output
  `sinks.rs:1562-1610`).
- `ArrivingCircle`: a circle at x = -10 plays +10 over 0.2 s, then waits 0.2 s.
- Options: Y4M, 96×64, 10 fps, certified, `ThreadPolicy::Fixed(1)`, `frames_in_flight = 3`.
- An offline certified plan has at least 3 slots, so it gets exactly 3 (fmn-runtime
  `plan.rs:508-532`, `:712-725`).

**Positive test (8 rounds).**

- `max_tile_references = 0`. Frames 0 and 1 are off-screen and bin into no tile. Frame 2 is
  refused on its worker. Frame 0's write is held until the scene returns.
- Asserts: `RenderError::Pipeline(_)`; `resource: "macrotile references"` and `limit: 0`; no
  `ReservationAbandoned`; nothing published; the retry publishes.

**Negative test.**

- Default budgets; frame 0's write fails with an injected I/O error.
- Asserts: `SinkFailed` and `injected sink failure`; neither `ReservationAbandoned` nor
  `LimitExceeded`; nothing published; the retry publishes.

**What synchronization establishes, with no sleeps:**

- Frame 0 cannot leave the sink until the scene has returned, and the scene can return only after
  one of its captures fails.
- So from frame 2's reservation onward, all 3 slots stay taken. Capture 3 can never get a slot. It
  must end on the refusal: it wakes at the ring to `ReservationAbandoned` (the F1 path), or it sees
  the closed stream.
- Meanwhile the coordinator waits in `Source::next` for frame 3 (fmn-runtime `stream.rs:280-299`).
  It emits nothing more until cancellation.

**What it cannot establish:** whether frame 2's refusal lands before or after capture 3 starts
waiting. No public seam can hold a worker.

- One render team rasterizes in order, so capture 3 starts waiting while frame 1 is still being
  rasterized. The F1 ordering is the expected one.
- That is an expectation, not a proof, and I have not measured it.

**Consequences.**

- **Patched:** both outcomes recover the name, so each round passes in either order (except R1).
- **Baseline:** the test fails in the F1 ordering and passes in the other.
- To probe the baseline, root can apply only the test hunk to an unpatched copy
  (`git apply --include=crates/fmn/tests/slow_sink_budget_refusal.rs …`). This is optional and
  informative only; it is not a pass condition. The native reproduction already exists:
  `native_render.rs:244` in attempt03.

**The watchdog (30 s)** fires only if the scene never returns: for example, if frame 2 were
admitted, or the plan had fewer than 3 slots. It turns that hang into a failed `SinkFailed`
("watchdog expired"). It never orders anything and never passes a test.

## Residual R1, not fixed here (fmn-runtime)

**What happens.**

- The coordinator publishes admitted frames in its emit callback (facade closure `pipeline.rs:496`;
  fmn-runtime `emit_ready` `:1400-1471`), and checks cancellation first.
- Suppose a publish lands between a refusing worker's reservation drop and its cancellation. The
  publish returns the recorded `ReservationAbandoned`, and the coordinator records a
  `Stage{Emit, …}` failure first. The worker's refusal is then dropped by first-error-wins.
- `finish()` therefore returns the abandonment, and the name is lost inside the pipeline. No
  facade recovery can get it back.

**When it can occur.** It needs a frame emitted while a later frame refuses, and a publish inside a
window of a few statements (rasterize return → `cancellation.cancel()`).

- It cannot occur in `native_render.rs:235`.
- In the new test it can occur only if frame 0 or 1 is published inside frame 2's window. If it
  does, the positive test fails; it does not hide it.

**Options for root:**

- (a) Raise the pipeline cancellation before a stage's reservation can drop. This narrows the window
  but does not close it.
- (b) Let a raster or convert error replace an emit-stage error whose source the stage marks as
  secondary (an abandonment). This closes it, but it is a cross-crate change to the generic
  runtime.

Neither is in this patch. Root decides.

## Commands for the guarded lane (not run here)

From the stage `engine` root, with the lane's usual flags:

1. `shasum -a 256 -c <bundle>/baseline-engine.sha256`
2. `git apply --check <bundle>/engine-budget-error-propagation.patch`, then `git apply` (or
   `patch -p1`)
3. `shasum -a 256 -c <bundle>/expected-engine.sha256`
4. rustfmt and Clippy `-D warnings` over `fmn --all-targets`, as engine-lint already does
5. `cargo test --locked --offline -p fmn --lib rendering::tests::only_a_closed_stream_or_an_abandoned_slot_is_traded_for_the_stage_failure`
6. `cargo test --locked --offline -p fmn --test slow_sink_budget_refusal`
7. `cargo test --locked --offline -p fmn --test native_render a_lowered_frame_table_budget_refuses_by_name_and_publishes_nothing`
   (the attempt03 failure; it must now pass with its assertion unchanged)

## Incident: a cargo command reached the rch hook

At 02:17:57Z I ran `cargo +nightly-2026-08-31 clippy --explain mutex_atomic` to look up a lint
group. The rch hook intercepted it. It logged:

- worker hz3 selected;
- `RCH-E412` (dependency planner fail-open);
- "Starting remote compilation pipeline for jeffs_math_explainer_videos (hash d9ded9df81dab91c)";
- "Prepared dependency sync manifest for 1 roots".

The shell's 2-minute limit ended the command (exit 143). I did not see whether anything reached
hz3. Afterwards, `ps` showed no process from that command. That was my only cargo command, and I
ran none after root's instruction.

## Not done

- No compile, Clippy, tests or renders.
- No Beads, TODO or commits.
- Nothing edited outside this folder. The working copies are in my scratch space.
- Staged, root and canonical sources are untouched.
