# Native scenes to standalone FMTL artifacts

`fmn::export_bundle` runs an ordinary `SceneConstruct` once and publishes a
validated FMTL/1 file. The existing standalone CLI and bundle players can render
that file without linking the original program or executing its updaters.

```sh
cargo run --locked -p fmn --example export_scene -- scene.fmtl
fmn --format png_sequence scene.fmtl
```

A new destination is required. Exports never overwrite an existing file,
directory or symlink, including when another process creates the destination
between capture and publication.

## Rust API

```rust,no_run
use fmn::prelude::*;

struct Example;
impl SceneConstruct for Example {
    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let circle = stage.add(Circle::new().color(BLUE))?;
        stage.play(circle.animate().shift(RIGHT)?)?;
        stage.wait(0.5)?;
        Ok(())
    }
}

let mut options = BundleExportOptions::new()?;
options.config.camera.fps = 30;
options.limits.max_frames = 30 * 60;
let receipt = export_bundle(&mut Example, "example.fmtl", options)?;
println!("{} frames, sha256 {}", receipt.frame_count, receipt.digest);
# Ok::<(), Box<dyn std::error::Error>>(())
```

`export_bundle_bytes` returns a `SceneBundleExport` without filesystem access.
`export_bundle_with_fs` accepts the existing `FileSystem` capability and uses
its atomic create-only publication contract. The native error, recording error
and filesystem error remain distinct typed variants of `BundleExportError`.
A caller-provided filesystem without atomic create-only support is refused.

## Sampling and replay

This imperative path uses `SceneBundleRecorder`, a real `SceneSink`. It records
actual immutable captures in their emitted order, preserving play/wait
boundaries, zero-frame segments, early-stop waits and explicit `show()` captures
from the lower-level Scene API. All segments are recorded snapshot segments:
there is no endpoint-only purity guess, callback serialization or second scene
execution. Arbitrary native updaters and noncatalog rate functions therefore
retain their observed output on random-access replay.

The exported schedule describes the captured frame grid, not the original
floating-point duration argument. It is validated by the existing exact rational
clock and FMNA/5 reader. Explicit shows are one-frame holds; a native program
that emits no captures produces one terminal still. Stills do not advance the
source scene's clock, so the receipt's scene time and output frame count can
differ. Ordinary native `EndScene` termination remains successful.

The caller must use matching renderer configuration when replaying: resolution,
background, frame height, antialiasing and any fixed camera are not embedded in
FMTL/1. The existing `fmn::rendering::render_bundle` and `render_bundle_with_fs`
functions consume these exported bytes directly.

## Limits and refusals

Capture limits bound admitted frame count and cumulative canonical snapshot and
destination-table storage. They do not interrupt arbitrary Rust code or the
runtime's deterministic completion of an already-started animation segment.
The complete canonical artifact also has the production format limit and the
caller's `max_output_bytes` limit, enforced by bounded writers before growth.
Captured snapshots are released as they are encoded, before output validation.
The recorder rejects missing, duplicated, reordered and mistagged segment
captures, including a missing final capture detected at the finish boundary.
Any capture failure permanently poisons the recording, even when scene code catches its play/wait error. Scene execution,
serialization, size checks and production-reader validation all finish before
file staging starts; a failed or panicking scene cannot publish partial output.

FMTL/1 does not carry audio or an external camera-rig binding. Native sound
requests fail with a capability error rather than producing a silent export.
Camera-rig scenes should continue through the camera-aware renderer. Renderer
policy, source/asset closure attestation, Python CLI export and compact proven
pure-segment export are not added by this native entry point. Existing
declarative `export_timeline_bundle` retains its separate proven-purity path.

This implements the native SceneConstruct portion of bead
`fm-scene-fmtl-export-96ej`; it does not close the wider Python/Studio delivery
work. The regression tests exercise real stateful scene capture, canonical
snapshot equality, direct-versus-replayed PNG bytes at different worker counts,
clock rounding, static scenes, deterministic re-export, errors, budgets,
no-clobber publication and unwinding.
