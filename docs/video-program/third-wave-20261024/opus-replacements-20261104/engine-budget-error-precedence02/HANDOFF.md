# engine-budget-error-precedence02: a refused frame outranks the emit failure its dropped slot caused

WhiteSwan, 2026-11-05. Closes Residual R1 from the sealed F1 bundle `engine-budget-error-propagation01` (Ruby's D1, Mail 3581), which attempt 04 then hit natively (Mail 3620).

**Nothing here has been compiled, linted or run.** The only checks were on source and formatting (`checks/source-checks.out`, RESULT PASS). No cargo command of any kind has run since the 02:17:57Z rch incident recorded in the F1 handoff. I claim no native pass: that belongs to the guarded serial lane.

## What attempt 04 showed

`engine-secondary-engine-test04`, `command.log` lines 546–547: round 1 of 8 of `a_worker_budget_refusal_behind_a_held_sink_keeps_its_name_and_publishes_nothing` failed at `slow_sink_budget_refusal.rs:277`. Round 0 passed.

```
round 1, frame 0 held: false: Pipeline(Pipeline(PipelineFailure { error: Stage { sequence: Some(0), stage: Emit,
source: Emitter(ReservationAbandoned { sequence: 1 }) }, stats: PipelineStats { … submitted: 3, prepared: 3,
rasterized: 1, converted: 1, emitted: 0, … backpressure_waits: 1, max_in_flight: 3 … } }))
```

The rest of F1 held natively: `native_render`'s lowered frame-table test and the genuine sink-failure test passed, and so did rustfmt and Clippy.

**The refused frame is 1, not 2.** Segment frames are 1-based, at time `frame / fps` (`fmn-anim/src/clock.rs:220-224`: "there is no zero-alpha emission frame"). So the 0.2 s move at 10 fps puts the circle's centre at x = −5 (off-screen) on frame 0 and at x = 0 on frame 1. Ruby found the same from source (Mail 3626), and the run agrees: only one frame rasterized, and slot 1 was abandoned. The sealed test's comments were off by one, and so was my reading in Mail 3597. That mail put R1 at frame 2's drop landing while frame 0 or 1 was still being published, which made it look rare. With frame 1 refusing, the test sits in that window every round (see the end of the next section).

## Mechanism

Line numbers are the attempt 04 baseline bytes.

1. `NativeFrameStages::rasterize` owns frame k's `FrameReservation` (`fmn/src/rendering/pipeline.rs:181`). An error returns through `?` (186–256), which drops it. `ReservationGuard::drop` then fails the ring with `ReservationAbandoned{k}` (`fmn-output/src/emitter.rs:877-899`). The ring keeps its first failure (999–1004).
2. Only after that does the worker cancel and send `Completion::Error{k}` (`fmn-runtime/src/pipeline.rs:1078-1085`).
3. The coordinator runs emit on its own thread. Suppose it publishes an earlier frame j in between. `publish` then returns the ring's failure (`emitter.rs:843-849`), and the coordinator records `Stage{j, Emit, Emitter(ReservationAbandoned{k})}` first (`pipeline.rs:1450-1458`). When `Completion::Error{k}` arrives, it loses to first-wins (1199–1204).
4. By then the pipeline's own record has no name, so the facade has nothing to recover. F1's `surface_failure` repair cannot help.

The slow-sink test renders with one team (`ThreadPolicy::Fixed(1)`). Frame 1's refusal follows frame 0's raster at once, and frame 0's completion triggers its publication. So the test sits in this window every round.

## Repair

**Runtime** (`fmn-runtime/src/pipeline.rs`, patched lines):

- `PipelineStages::emit_failure_caused_by(error) -> Option<u64>` (:156) is a provided method with default `None`. It is declared `where Self: Sized`, so the trait stays dyn-compatible.
- The `Coordinator` keeps `awaiting_cause` and the stages' rule (:1192, constructed at :556). An emit failure that is the first failure is recorded exactly as before:
  - cancellation fires;
  - later outputs are dropped unemitted;
  - `emitted` is not incremented.

  If the stages say that failure only reports frame k, the coordinator also notes k (:1496).
- Frame k's own `Error`, `Panicked` or `Disconnected` completion goes through `fail_frame` (:1231, called at :1390/:1408/:1425) and replaces the noted emit failure. Every other failure keeps first-wins. k is still outstanding, so its completion is always received, at the latest in the final drain.
- With the default rule nothing ever replaces anything. So every other `PipelineStages` implementor keeps its exact current behaviour: the conformance stages, `fmn-python`'s portal, the stream tests and the asupersync lab.

**Facade** (`fmn/src/rendering/pipeline.rs:301`): `NativeFrameStages` names frame k only for `Emitter(ReservationAbandoned { sequence: k })`, and returns `None` for everything else.

### Why this precedence

**A genuine sink failure stays authoritative.** The ring keeps its first failure. Once a sink failure is recorded, every later publication returns that failure. It names no frame, so it stands, and so does every other emitter error: `SinkPanicked`, `ReliableSinkWouldBlock`, `Cancelled` and `Closed`.

**The worker's cause is kept.** The caller gets `Stage{k, Raster, …}`. Here that is the `LimitExceeded` for `macrotile references`.

**No abandonment is ignored:**
- The publication of j still fails and is still recorded.
- The ring's own `EmitterFailure` still reports `ReservationAbandoned{k}` (asserted).
- An abandonment whose frame never fails on its own stays the reported error (asserted). One example is a scene-thread capture error after its output reservation.

**No frame is overcounted.** Counting is unchanged:
- The pipeline's `emitted` counts frames whose publication succeeded.
- The ring's `published` and `emitted` are untouched.
- The tests assert all three.

**Not chosen:**
- *Emitter scoping:* the publication of j would succeed into a failed ring, so it would be counted but never delivered.
- *Facade `Ok`:* it overcounts and masks the failure.
- *Carrying the slot inside the error:* it frees no slot until the coordinator reads the completion. A full ring whose sink waits on the scene then deadlocks, as this test's held sink would.

## Tests

There are 6 new tests, plus the slow-sink comment fix. Each test orders its race with condvar marks and no sleeps:
- A stage or emit waits only for a mark that another thread sets.
- The coordinator runs emit itself. So a failure that a worker sends while emit is waiting is received only after emit's own failure is recorded.
- A 30 s watchdog turns a hang into a panic, which the pipeline reports. It never passes a test.

`fmn-runtime`, library (synthetic errors, the rule itself):

| Test | Asserts |
|---|---|
| `pipeline::tests::an_emit_failure_reporting_a_later_frame_yields_to_that_frames_own_failure` | Frame 0's emit returns `Abandoned(1)` after frame 1 fails. If frame 1 returned its error, the result is `Stage{1, Raster, Refused(1)}`. If it panicked, the result is `CallbackPanicked{1, Raster}`. `emitted` 0, no leaked slots. |
| `pipeline::tests::a_sinks_own_emit_failure_stands_when_a_later_frame_fails` | Same order with a sink error: `Stage{0, Emit, Sink}` stands. |
| `pipeline::tests::an_emit_failure_reporting_another_frame_is_not_replaced_by_this_one` | `Abandoned(2)`, naming a frame never submitted, stands as `Stage{0, Emit, Abandoned(2)}` against frame 1's refusal. |

`fmn`, library (`NativeFrameStages`'s own rule; the real `OrderedEmitter` ring; the facade's own emit, `output.publish().map_err(NativeFrameError::Emitter)`):

| Test | Asserts |
|---|---|
| `rendering::pipeline::tests::only_an_abandoned_output_slot_names_the_frame_behind_an_emit_failure` | `ReservationAbandoned{3}` gives `Some(3)`. `SinkFailed`, `SinkPanicked`, `ReliableSinkWouldBlock`, `Cancelled`, `Closed` and a worker error give `None`. |
| `rendering::pipeline::tests::a_refusal_that_abandons_its_slot_while_frame_0_publishes_keeps_its_name` | Frame 1's worker drops its real slot, and only then does frame 0 publish. The publication fails with `ReservationAbandoned{1}`, but the result is `Stage{1, Raster, WorkerState(..)}`. Pipeline `emitted` 0. The ring still fails with `ReservationAbandoned{1}`, with `published` 0 and `emitted` 0. |
| `rendering::pipeline::tests::a_sink_failure_published_before_a_later_refusal_stays_the_sinks` | The sink fails frame 0's write. Frame 1's emit first waits on `reserve(3)`: the ring is full until the drain frees frame 0's slot, and the drain records the sink failure under the same lock (`emitter.rs:1087-1110`). Only then does frame 2 refuse and drop its slot. Result: `Stage{1, Emit, Emitter(SinkFailed{sequence: Some(0)})}`. Pipeline `emitted` 1 (frame 0's publication). The ring fails with `SinkFailed`, with `published` 1 and `emitted` 0. |

On the baseline code, the unit tests do not compile, because they implement the new hook. Forced into the same order, the baseline coordinator keeps the emit failure, so both positive cases would get `Stage{0, Emit, …}` instead of frame 1's own failure. The end-to-end regression that failed natively on the baseline is the held-sink test, in round 1 of attempt 04.

**`fmn/tests/slow_sink_budget_refusal.rs`** changes only comments and one panic message. The panic message changes from "frame 2 needs" to "frame 1 needs" and asserts nothing. The source checks prove the rest:
- outside comments, only that line changes;
- all 8 `assert` lines are unchanged;
- `ROUNDS` stays 8.

The corrected comments say:
- Frame 1 is refused.
- Nothing orders frame 1's slot drop against frame 0's publication, and both orders occur. Either the scene's next capture wakes to the abandonment (F1's path; `held: true` is possible), or frame 0's publication itself fails (R1, `held: false`).
- Each round must name the refusal in either order.
- The unit tests force the second order.

I kept the scene as it is. Moving the refusal to frame 2 would favour the full-ring path and run R1 less often, which would weaken this regression. A separate variant can be added if root wants it.

## Application

Apply on top of the four STACK entries attempt 04 used, as a fifth entry `(engine-budget-error-precedence.patch, expected-engine.sha256)`. The patch is git-style with `a/`/`b/` headers and goes against the engine root. `git apply` and `patch -p1 --forward` are both clean on a fresh copy of the stage's `crates/fmn-runtime` and `crates/fmn`.

| File | Baseline (attempt 04 stage) | Expected |
|---|---|---|
| `crates/fmn-runtime/src/pipeline.rs` | `83b3324636c5a0d5b3b54be9b97e411fb744a84a962c1a7b2afa106d88d06845` | `c4d546ffdf641de9929bcd591d6367924d6ad197e593c7a8087b4d3596388c88` |
| `crates/fmn/src/rendering/pipeline.rs` | `2e4df1ca1ef0b67cfb694ffcacbada5e8630be5f20bc0ee9810fa37ddd72b079` | `7a79bfafd00864de1f0a0497c0758f89a92f5836b3350edaa348e25064ef69c0` |
| `crates/fmn/tests/slow_sink_budget_refusal.rs` | `80148eb6582c3f2092d3f4a6eb8ffc1bd053c5b26c15fd165f91f162181c2876` (F1's expected) | `92802f8fecf476563ce0edf7a511ec799c7968f0d728016181f2c867bb9ca61c` |

The source checks also confirm three things:
- the stage still carries F1's three expected rows;
- the F1 bundle still verifies against its `SHA256SUMS`;
- the new hook has exactly four definitions: the runtime default, the facade's, and one per test stage.

Rerun them on any stage with:

```
python3 -I -B checks/source_checks.py STAGE_ENGINE NEW_EMPTY_SCRATCH_DIR
```

## For the guarded lane (I ran none of these)

**Lint.** Run rustfmt on the three files with the runner's flags: `--check --edition 2024 --config skip_children=true`. Clippy `-D warnings` must add `-p fmn-runtime`. The runner names only `-p fmn-render -p fmn --all-targets` (`run_validation.py:329`), and `--all-targets` covers only the packages named. So without it, the new tests in `fmn-runtime`'s test module are never linted.

**Build.** Add `cargo test -p fmn-runtime --no-run`. `fmn --lib` is already built.

**Test:**
- `cargo test -p fmn-runtime`, the whole crate, because the coordinator changed. This covers the library tests (with the 3 new ones), the stream tests and `tests/asupersync_lab.rs`.
- `fmn --lib`: the 3 new tests by name, plus F1's `rendering::tests::only_a_closed_stream_or_an_abandoned_slot_is_traded_for_the_stage_failure`.
- `fmn --test slow_sink_budget_refusal`: both tests, with all 8 rounds of the held-sink test.
- The existing `fmn` suites, unchanged.

**Self-test count.** The runner's count rises by 6: 3 in `fmn-runtime` and 3 in `fmn`'s library.

**Other implementors.** These are compile-only risks, since they keep the default hook. I suggest `fmn-conformance --test frame_pipeline --test ordered_emitter` and a build of `fmn-python`'s `portal_pipeline.rs`. Root decides.

## Read, not compiled: the compile points I checked by hand

- An impl may leave out the trait method's `where Self: Sized`; the facade and both test stages do.
- `Coordinator::new(S::emit_failure_caused_by)` coerces a fn item to `fn(&E) -> Option<u64>`, with `E` inferred as `S::Error`. The lifetime is late-bound, and the struct is `Sized` by default.
- `Choreography::mark` passes `&mut MutexGuard<Marks>` where `&mut Marks` is expected (deref coercion at the call). The `wait` predicates receive `&mut Marks` where `&Marks` is expected.
- `FrameReservation`, `PipelineStats` and `EmitterReport` derive or implement `Debug`, as `expect_err` needs. `EmitterError` is `PartialEq`, as `assert_eq!` needs.
- No `use` is unused. The test modules shadow nothing in their parents (searched).

## Limits

- No compile, Clippy or test run. Ruby's guarded attempt is the evidence.
- `fmn-python`'s portal has the same shape: it publishes reservations and drops them on error, but its errors are `String`. It keeps the default hook, so it is not repaired here.
- If frame k's slot is dropped because of external cancellation, k's completion is `Cancelled`. The abandonment then stays the reported error, as on the baseline.
- If two frames on different teams refuse together, the ring's first drop decides which frame is named. Either is a real refusal.
- Ownership: TODO, Beads, integration and every native job belong to root and Ruby. The stage, `inputs05` and the F1 bundle were read, never written.
