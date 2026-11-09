# Export a scene with its camera

Camera-bearing FMTL/1 artifacts contain the observed geometry, camera pose,
field of view, light, linear background and sample policy at every output frame.
The standalone native player replays those captures without loading source code,
calling an updater or requiring host Python. The existing renderer, rational
clock and ordered output pipeline remain the authority.

## Python authoring

```python
import manimlib as m
from fmn_python import export_bundle

class Orbit(m.Scene):
    def construct(self):
        self.frame.rotate(0.4, axis=m.RIGHT).scale(0.6)
        self.add(m.Cube(side_length=1.5, color=m.BLUE))
        self.play(m.Rotate(self.frame, angle=1.0, axis=m.UP), run_time=1.0)

receipt = export_bundle(
    Orbit, "orbit.fmtl", camera=True, resolution=(960, 540), fps=30,
)
assert receipt.camera_track
```

`BundleExportSession(scene, destination, camera=True, ...)` exposes the same
recorder for a program that owns its scene lifecycle. `camera` must be a boolean.
A wheel without the camera recorder refuses before constructing the scene.
Authored setup/construct/tear_down run once; the context manager itself does not
run them. The ordinary ownership guards, cancellation, frame/byte budgets and
atomic create-only publication remain in effect. Static scenes capture one
terminal still without advancing the scene clock.

The default (`camera=False`) is unchanged: minor-0 planar vectors, suitable for
the existing camera-less browser player. The Python console's existing FMTL
export command keeps that default. Use the programmatic API above for camera
export; native CLI **consumption** needs no new switch.

## Rust authoring

`fmn::exporting::export_camera_bundle_bytes` and its file-publication siblings
`export_camera_bundle` / `export_camera_bundle_with_fs` take the same factory
shape as `fmn::rendering::render_camera`:

```rust,ignore
let artifact = export_camera_bundle_bytes(
    |scene, camera| {
        let rig = fmn::scene::CameraRig::new(scene, camera)?;
        Ok((MyOrbitProgram { rig }, rig)) // ordinary SceneConstruct
    },
    camera_config,
    bundle_export_options,
)?;
```

The factory allocates its rig in the actual scene. It may allocate objects or
register updaters, but cannot advance playback before returning. The program
animates the existing rig's trackers; export samples the rig from each immutable
post-updater capture, not from a later live scene. Camera FPS must match the
effective scene FPS. The rig supports view and light motion; its base camera
supplies fixed background and capture policy. Low-level `SceneBundleRecorder`
camera capture can record changing background/policy as well.

## Standalone replay

```sh
fmn --robot orbit.fmtl --format y4m --resolution 960x540 --threads 4 --video_dir media
```

The bundle's frame grid is authoritative; an explicit FPS must match it.
`fmn::rendering::render_bundle` selects its camera-aware CPU path automatically.
An explicit camera override is refused rather than silently replacing recorded
views. Output pixel dimensions may change: equal aspect preserves captured frame
shape and orientation bits, while a changed aspect preserves authored width.

Shared native frame jobs keep the original camera and scene paired even when
rendered out of order. Output remains frame-index ordered. Renderer workers never
run the source scene's callbacks.

## Explicit limits

Camera tracks require the new minor-1 reader. Older strict readers and the
current vector-only WASM player refuse them; no camera-less fallback is used.
Geometry-only materialization also refuses camera-bearing frames. Audio remains
unsupported and causes export to fail, not silently lose a soundtrack. Existing
output files are never replaced, including a destination created by another
writer during scene execution.

This is a code-free **capture artifact**, not complete input-closure certification
of arbitrary Rust/Python source effects. The tests for camera capture, native
replay, installed-wheel export and the actual CLI must pass on their exact build
before claiming native acceptance or cross-platform determinism.
