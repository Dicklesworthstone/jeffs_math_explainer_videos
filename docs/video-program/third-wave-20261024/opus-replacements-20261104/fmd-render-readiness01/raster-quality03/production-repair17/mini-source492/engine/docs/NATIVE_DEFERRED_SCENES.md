# Deferred native scene authoring

`NativeSceneProgram` can now construct later scene content from the state left
by preceding animations. Previously, all `NativeSegment::Play` animation values
and their targets had to exist when the complete schedule was assembled. That
could freeze a target too early and could not express ordinary scene code that
adds or removes objects between plays.

The new authoring steps use the same native Scene, Stage, animation lifecycle,
rational clock and renderer. They introduce no interpreter, second scheduler,
serialization of closures, or independent interpolation implementation.

## Three authoring operations

`NativeSegment::play_with(overrides, build)` creates animation objects just
before their play. The callback sees the completed predecessor, including its
final-alpha, remover, updater-resume and zero-dt cleanup behavior. For example,
a relative move following an earlier rotation can build its target then:

```rust
use fmn_anim::{Animation, Transform};
use fmn_mobject::Mob;
use fmn_scene::PlayOverrides;
use fmn_studio::native::NativeSegment;

fn move_from_completed_state(source: Mob) -> NativeSegment {
    NativeSegment::play_with(
        PlayOverrides { run_time: Some(0.5), ..PlayOverrides::default() },
        move |context| {
            let stage = context.stage_mut();
            let target = stage.copy_family(source)?;
            stage.shift_many(&[target], [3.0, 0.0, 0.0]);
            Ok(vec![Box::new(Transform::new(source, target)) as Box<dyn Animation>])
        },
    )
}
```

`NativeSegment::edit(edit)` performs a data/membership change between segments
without an extra frame or play-count increment. Use it to add/remove roots,
change style, register native updaters, or update trackers. The callback returns
`Result<(), SceneError>`; ordinary native Stage errors propagate with `?`.

`NativeSegment::defer(build)` is the general form. It can create objects,
inspect current state, choose a branch, and return a `Vec<NativeSegment>`.
Those segments execute in their returned order *before* the previously queued
continuation. They can themselves contain deferred steps. Each callback is an
owned `FnOnce`: it runs once in one program instance, not once per render or
inspection. Variables created inside a callback can be moved into its returned
animations and subsequent construction closures.

## Scoped native access

`NativeBuildContext` provides `stage()`, `stage_mut()`, read-only `scene()`,
and `spans_mut()`. Constructors can read actual scene time and play count,
operate on original native handles, and bind inspector spans for late-created
content. They cannot obtain mutable Scene playback through this context, so
ordinary use cannot nest `Scene.play` or `Scene.wait` and consume invisible
frames behind the Studio cursor.

`rng_mut()` exposes the existing scene-serial PCG64DXSM substream. Late random
choices use the same state as ordinary Scene construction, and checkpoint bytes
retain the resulting RNG state. No independent generator or ambient entropy
is introduced. `event_dispatcher_mut()` supports installing/removing native
listeners when their objects are constructed. Registration does not itself
queue or dispatch input, and normal Scene input remains the sole dispatcher.

For an already-bound animated camera, `context.camera_to(rig, &destination)`
returns a boxed native Transform. This constructs the quaternion target in the
hemisphere of the *completed current pose*, which matters in a sequence of
rotations: prebuilding every target against frame zero can choose a different
rotation route. Normal Transform timing, normalized-linear quaternion semantics,
and explicit updater suspension are unchanged.

The context is not a sandbox for arbitrary Rust code. Callbacks can still
allocate or perform I/O; the replay contract and isolated worker remain the
security/effect boundary. Source spans and the camera role must still obey
their existing validity rules. A camera rig cannot be newly bound after any
construction has executed, even if that construction emitted zero frames.

## Exact capture boundaries

Frame zero remains the state returned by the source factory. A leading deferred
step does not run merely because a host constructs the program, inspects it,
requests its checkpoint, or seeks frame zero. It runs when execution advances
toward the next capture.

Pausing on a segment's last capture does not run its epilogue or the next
constructor early. Advancing to the following capture first finishes that
segment, then executes the deferred steps and opens their returned segments.
Earlier frozen frame packets are unaffected by later root replacement or
geometry edits. Re-requesting an already paused frame is read-only with respect
to construction.

The frame budget still refuses before running any further finalization or
construction. A host that pauses at the declared last frame does not secretly
run the trailing authoring tail. There is no automatic discovery or extension
of `frame_count`: the factory's declared addressable range must cover the
captures its chosen branch actually produces. Seeking beyond a shorter branch
returns the existing ended-before-target error.

## Bounded expansion and failures

The default `MAX_NATIVE_SEGMENTS` budget is 65,536 cumulative admitted steps.
It includes the initial schedule, each deferred callback and every segment that
callback returns. Consumed queue entries do not replenish the budget. This
prevents a self-generating deferred sequence from evading admission limits by
keeping only one pending node.

`program.with_segment_limit(n)` configures a smaller bound before execution;
it must cover the initial schedule and cannot exceed the default ceiling.
Zero permits only an empty initial schedule. Returned batches are checked and
queue capacity is reserved before any returned step executes. The callback
producing a rejected batch may already have edited native data, so the failure
poisons the entire program rather than claiming those edits rolled back.

A construction error or panic never produces a successful new frame or
checkpoint. The program is marked failed *before* entering authored work, so a
native caller catching a panic cannot resume a partly consumed callback.
Subsequent playback, input or checkpoint requests refuse on that instance.
Through `NativeSceneWorker`, the existing crash/report/restart boundary and
fresh-factory recovery remain responsible for rebuilding a healthy owner.
This is not rollback of external side effects or closure captures.

The segment bound does not interrupt a single non-returning callback or cap its
internal allocations. The supervisor's process deadline and existing memory
policies are still necessary. No runtime/performance budget is claimed from
these admission checks alone.

## Checkpoints and deterministic recovery

Deferred closures and the pending executable continuation are not serialized.
The source factory reconstructs them, then ordinary native execution reaches
the requested checkpoint. The worker verifies actual SceneState bytes and
journal hashes before installing that rebuilt owner. This works both before
and after late object construction and native updater registration.

`NativeReplayPolicy::Disabled` remains the default for unclassified callbacks.
`ColdVerified` still requires independently constructed captures and a complete
content-hashed source/input closure, with no unjournaled external influence.
A deferred constructor is not promoted to pure or frame-parallel merely because
it happens between frames. Geometry mutations, target allocation and newly
registered updaters are part of the real scene state and replay path.

## Run the complete application

The existing live Studio example now starts with a blue object, constructs a
new orange object after the first play, animates it, and only then registers
its stateful movement updater. The parent never executes those constructors.

```sh
cargo run --locked -p fmn-studio --example native_live
cargo run --locked -p fmn-studio --example native_live -- --self-test
```

The self-test uses the real child executable, canonical IPC and authenticated
loopback HTTP. It crashes and restores after late construction/updater creation,
compares complete PNG responses, and tests scrubbing, editing, undo, inspection,
overlays and restart. It is run by the existing Native Studio workflow.

## Acceptance and remaining scope

`native_deferred.rs` compares all captured native data with ordinary Scene
playback and covers lifecycle timing, root visibility, immutable prior packets,
nesting, one-shot execution, admission bounds and failures.
`native_deferred_worker.rs` exercises real PNGs, replay/checkpoint continuation,
1/4/16-thread equality, failure publication and canonical crash responses.
`native_deferred_camera.rs` covers late quaternion hemisphere selection,
invalid camera edits and zero-frame binding guards. `native_deferred_context.rs`
compares seeded state with ordinary Scene execution and exercises late native
listener registration against the actual dispatcher.

These changes extend the explicit native-factory Studio path. They do not
compile/discover arbitrary source files, suspend an arbitrary Rust call stack,
add perspective editing, serialize callbacks, or make recovery constant-time.
The ordinary CLI captured-artifact worker and mandatory workspace gates are
unchanged. The authored code must be tested on the exact published checkout;
source inspection alone is not a native execution receipt.
