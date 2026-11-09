# Executable native Studio workers

`fmn_studio::native` connects an owned native Scene to the existing isolated
worker protocol, retained CPU renderer, interactive editor and supervisor
recovery path. It does not serialize native closures or pretend that changing
a clock label executes them.

## Run the browser hosts

From a checkout with the repository's pinned Rust toolchain:

```sh
cargo run --locked -p fmn-studio --example native_live
cargo run --locked -p fmn-studio --example native_camera
```

Open the private loopback URL printed by the selected program. `native_live`
provides affine editing, selection, undo, inspection and overlays. The camera
example combines an animated lit surface, fixed-frame raster image, glow dot,
camera transition and subsequent camera updater. Both support timeline scrub
and worker restart. Press Enter in the launching terminal to stop the host
and reap its child.

The examples use `StdHostEntropy`, whose audited implementation currently
requires Unix; other hosts must provide an audited entropy capability rather
than a predictable fallback token. The parent runs the existing `Supervisor`,
`StdWorkerLauncher`, `FrameHub` and `StudioHost`. Only the disposable child
constructs or executes the scene. It is launched from the exact absolute
executable with a cleared, explicit environment and the bounded versioned pipe
protocol. Build digests are checked at handshake. Checkpoint caches are bounded
and process-local; the examples leave no cache files.

Restart relaunches the same already-built executable. These examples do not
invoke a compiler or implement incremental source rebuilding.

```sh
cargo run --locked -p fmn-studio --example native_live -- --self-test
cargo run --locked -p fmn-studio --example native_camera -- --self-test
```

These headless acceptance modes drive real child processes and authenticated
loopback HTTP. The live example deliberately panics a native callback and
checks supervisor survival, restart, editing/undo and PNG continuation. The
camera example checks camera/surface animation and recovery both during a
Transform and after a stateful camera updater has resumed. Neither example
requires CPython, system fonts or ffmpeg.

## Build a native worker

Construct a fresh `Scene` and owned `Vec<NativeSegment>` in a factory.
`NativeSegment::Play` owns native animations plus `PlayOverrides`;
`NativeSegment::Wait` uses ordinary Scene wait semantics.
`NativeSceneProgram::from_interactive` also accepts an existing editor without
registering duplicate input listeners.

A program requires frame zero, play count zero, and no skip/range or presenter
mode. It steps through Proscenium's existing begin/prepare/complete/finish
operations. Segment finalization happens when execution moves beyond its last
capture, not prematurely while paused on that capture. Updater suspension,
animation cleanup, empty plays and zero-duration waits keep their native
semantics. Every capture is checked against the actual rational Scene clock.

Index zero is the constructed state. Later indices are actual Scene captures.
The program `frame_limit` is the highest admitted capture; the worker's
`frame_count` is that limit plus one. Durations use the existing rational-clock
quantization: do not assume that decimal `0.1` always means exactly three
samples at 30 fps.

Construct `NativeWorkerConfig` with the scene name, executable digest,
source/input closure digest, frame count, fps and retained renderer policy.
Pass it and the factory to `NativeSceneWorker::new`, then pass the worker to
`serve_worker` inside the disposable child. Use the same `ProtocolLimits` for
the service and pipe driver. The examples are complete host compositions.

The worker keeps one live cursor, not a prerendered movie. Clean forward scrubs
continue the same callback owner. Backward scrubs reconstruct from a new factory
and execute to the target. Native Scene/animation/editor snapshots retain their
ordinary memory costs; only one capture packet is held during stepping.

## Mixed camera rendering

The default `camera: None` keeps affine vector rendering. Select a base camera
for mixed vectors, UV-grid surfaces, triangle meshes, image quads and dot clouds:

```rust
worker_config.renderer.engine = fmn_render::EngineIdentity::certified();
let viewport = worker_config.renderer.frame.viewport;
worker_config.camera = Some(fmn_render::CameraConfig {
    resolution: (viewport.width, viewport.height),
    fps: worker_config.fps,
    background: worker_config.renderer.frame.background,
    ..fmn_render::CameraConfig::default()
});
```

This selects Lumen's existing retained camera CPU renderer. Projection,
clipping, painter order, depth, lighting, texture sampling, fixed-frame content
and glow remain its responsibility. Viewport, background and fps must agree
with worker policy. Fast-CPU/annex identities refuse on this route rather than
mislabeling the executed engine. Selecting the certified engine does not itself
claim a completed cross-platform release certification matrix.

Without a camera rig, this configuration stays fixed throughout playback.
With a rig, its output policy stays fixed while native state drives the pose.

## Animated cameras on the same Scene clock

`fmn_scene::CameraRig` creates a point-free family of twelve ordinary native
scalar trackers: center xyz, width, quaternion xyzw, vertical field of view,
and light xyz. Native animations and updaters mutate those channels. They are
part of the existing Stage snapshots and SceneState bytes, not a parallel
camera clock or renderer callback.

```rust
use fmn_scene::{CameraRig, PlayOverrides};
use fmn_studio::native::{NativeSceneProgram, NativeSegment};

// `scene` and `base_camera` are the native scene and validated capture config.
let rig = CameraRig::new(&mut scene, &base_camera)?;
let mut destination = base_camera.clone();
destination.frame.set_center([1.0, 0.0, 0.2])?;
destination.frame.set_width(10.0)?;
destination.frame.set_euler_angles(Some(0.6), Some(0.9), Some(0.0))?;
destination.light_source_position = [4.0, 6.0, 10.0];

let movement = rig.animate_to(&mut scene, &destination)?;
let program = NativeSceneProgram::new(
    scene,
    vec![NativeSegment::Play {
        animations: vec![Box::new(movement)],
        overrides: PlayOverrides { run_time: Some(2.0), ..PlayOverrides::default() },
    }],
    60, // 2 seconds at 30 fps; worker frame_count is 61
)?.with_camera_rig(rig)?;
```

`animate_to` returns the existing Choreo `Transform`: normal easing, timing,
composition and suspension apply. Its detached target has no inherited
updaters, so source callbacks cannot move the requested destination. Resolution,
background, samples and other output policy are not animated by this helper.
The quaternion target is chosen in the source hemisphere, avoiding the zero
midpoint between equivalent q and -q orientations. Interpolation is normalized
linear interpolation, not spherical interpolation or constant angular speed.

The existing native Transform default keeps source updaters active. Thus a
source updater can still change the visible pose after interpolation, while
the detached target stays fixed. To pause source camera updaters during the
transition, explicitly set the returned animation's configuration before play:

```rust
use fmn_anim::Animation;
let mut movement = rig.animate_to(&mut scene, &destination)?;
movement.state_mut().config.suspend_mobject_updating = true;
```

The runnable camera example uses this explicit mode. Normal animation finish
resumes the source updaters; starting-copy updater behavior is unchanged.

For independent channels, `center()`, `width()`, `orientation()`,
`field_of_view()` and `light()` return the original tracker handles. Attach
ordinary native dt-updaters or use them as native Transform targets. Raw
quaternion-channel authors must choose consistent signs themselves. Near-pole
initial orientations are retained without extracting and rounding Euler angles.

Bind the rig once before execution. A foreign, removed or structurally changed
rig refuses; invalid width/FOV, non-finite values and zero quaternions cannot
produce successful capture or checkpoint receipts. Sampling reads the captured
state without running callbacks or altering it. A rig requires an explicit
worker CameraConfig. Frame height follows that output aspect ratio.

The renderer keeps a monotone camera revision when pose or light changes;
newly constructed camera revision counters cannot accidentally reuse stale
projection caches. Inspector scale is sampled from the current rig even when
a fresh renderer has not drawn yet. Camera inspection still advertises
`input_events: false`: perspective-aware editing and projected diagnostic
overlays remain unavailable and refuse before changing the scene.

## Input, journals and recovery

On the affine route, `Event` dispatches to existing native listeners at the
paused frame, then renders a new PNG. Inspection and overlays share the edited
arena. Input does not tick updaters or advance time. These edits are transient:
the next seek reconstructs from source; they are not reported as durable edits.

`Play` accepts canonical `studio_seek_command` identities. It executes from a
fresh factory and journals real `Scene::state_bytes()`: actual clock, RNG, play
count, geometry and camera tracker state. Entries carry the source/input
closure, worker build and capture contract. Camera scenes additionally bind
normalized base capture policy; rigged scenes bind the rig schema and its
initial root position. A different factory rig selection refuses before
playback. No process-local arena address enters the binding identity.

Renderer records describe the stable base policy and rig binding. Animated
poses are native SceneState, not an ever-growing list of renderer identities.
Changes to base projection/light/output policy invalidate journal reuse.
Use the supervisor's read-validation path for checkpoints: static policy and
role-binding authority live in the journal, not in bare SceneState bytes.

Replay is disabled by default: unknown callback programs are opaque barriers.
`NativeReplayPolicy::ColdVerified` is an explicit author attestation that every
factory creates independent animation/updater/listener captures and depends
only on the hashed input closure. No unjournaled I/O or externally shared mutable
factory state may influence it. It is still stateful, never advertised as
frame-parallel pure.

Checkpoint restore decodes metadata only to locate the frame. The callback-free
snapshot is never installed. A new factory executes from zero and its complete
state bytes must match before replacing the owner. Forward playback then uses
the reconstructed callback state, including camera updaters. Replay validates
ranges, source/effect/backend identities, event ownership, checkpoint hashes
and total frame-work budget before factory calls, then verifies every requested
state. A later divergence installs no earlier prefix. Only executed entries
replace the crash tail; empty replay preserves the owner and tail.

A failed live execution invalidates that cursor rather than pretending record
restoration rolls back closure state. A new seek can construct a healthy owner.
Complete encoded responses are budget-checked before committed state is exposed.
Other limits cover frames, replay work, viewport pixels, checkpoints and JSON.
The supervisor's process deadline remains the boundary for non-returning native
callbacks; an in-process frame counter cannot interrupt arbitrary native code.

## Scope and validation

This is the explicit native-factory front door. The default CLI captured-
artifact worker is unchanged, and these examples do not discover or compile
arbitrary source files. Programs have an owned declarative segment schedule,
not a suspended arbitrary Rust stack. Perspective editing, projected overlays,
Metal presentation, Python callback ownership and audio export are not added.

Recovery is cold execution, not constant-time restoration; Studio latency and
memory/performance budgets have not been benchmarked for this path. Hashes and
state equality detect divergence but do not sandbox effects. Native editing
history does not rewind external state inside closures.

The focused Native Studio workflow runs the whole Studio test suite, native
editing history, and both real-child/HTTP self-tests. Camera-motion tests compare
every PNG against ordinary Scene playback at 8 and 30 fps, check fixed-frame
invariance and 1/4/16-thread equality, repeated reads, invalid input, binding
changes and camera-callback recovery. Transition tests cover opposite-quaternion
signs, updater-free targets under both suspension modes, and pre-allocation
refusal. These tests supplement, not replace, mandatory workspace governance,
formatting, lint and certification. A focused execution pass is not a
full-workspace gate pass.
