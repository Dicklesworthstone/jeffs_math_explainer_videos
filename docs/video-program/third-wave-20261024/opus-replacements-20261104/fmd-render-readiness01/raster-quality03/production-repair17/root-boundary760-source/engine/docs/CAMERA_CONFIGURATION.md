# Scene camera configuration and frame construction

Scene resolves its camera defaults and constructor overrides before creating the
live frame used by animations. Nested `frame_config` dictionaries merge rather
than replacing the entire default recipe. The frame's native core owns its
configured size, center, field of view and Euler-axis convention. The scene's
`default_frame_orientation` is applied afterward and becomes its reset orientation.

```python
from manimlib import Scene, Square, RIGHT, UP, linear

class Detail(Scene):
    default_camera_config = {
        "resolution": (960, 540),
        "frame_config": {"frame_shape": (5, 3), "fovy": 0.8},
    }

    def construct(self):
        square = Square().move_to((2, 1, 0))
        self.add(square)
        self.play(self.frame.animate.shift(RIGHT), square.animate.shift(UP),
                  run_time=1, rate_func=linear)

scene = Detail(camera_config={"frame_config": {"center_point": (2, 1, 0)}})
scene.render("detail", format="png_sequence")
```

Capture-camera creation remains lazy. Accessing `scene.camera` for the first
time applies the native width-preserving pixel-aspect policy to the **existing
live scene frame**. For example, width 5 at a 960-by-540 resolution yields a
frame height of 2.8125. Until that first access, the frame retains the explicitly
requested shape. Subsequent camera accesses do not reset its height, orientation,
position or updaters. The frame recipe is evaluated only during Scene
construction, not again during lazy capture creation or a retry after a capture
configuration error. Later pose changes go through `scene.frame`, not through
mutations to the already-consumed `camera_config["frame_config"]` dictionary.
A render session realizes the camera before playback.
Unsupported capture options such as a window or background image still fail on
first camera access, not merely on constructing a Scene.

## CameraFrame subclass initialization

CameraFrame allocates a validated native pose before the ordinary Mobject data,
point and uniform hooks. Positional methods can therefore be used during those
hooks without an absent `_core` error. The native core keeps its identity through
construction, including when a hook records that identity or takes a copy.

As in the pinned Reference's constructor, base-hook execution is followed by a
unit-frame/camera-uniform reset and public `set_width(..., stretch=True)`,
`set_height(..., stretch=True)` and `move_to(...)` calls. These final calls now
execute authored overrides instead of being bypassed by a direct native-core
assignment. Preliminary pose edits in `init_points` do **not** override the
constructor's final requested pose; customize these final public methods or
modify the frame after construction to do that. Authored record fields, children
and updaters are not discarded by pose finalization.

This continues to use the separate native CameraFrame pose representation, not
a replacement drawable point buffer. Lumen remains responsible for camera
validation, projection and rendering, and existing camera animation drivers own
interpolation and timing. No new native API, renderer or dependency is introduced.

## Acceptance boundary

`camera_configuration.py` and `camera_frame_lifecycle.py` are registered in the
installed-portal runtime gate. They exercise actual native cameras, copies,
callback failures, independent direct-camera/literal-pose controls and changing
PNG sequences at one/four threads. The unchanged implementation fails these
witnesses. Local development can run them with the changed bootstrap methods
overlaid on a retained native extension, but that is not an exact-tree wheel
build, an embedded Rust-harness pass, or cross-platform certification.

Production installation uses `camera_lifecycle` and `camera_frame_lifecycle`
through the shared portal initializer. The original saved bootstrap patch is
not an additional installation step. Both installers retain class identity and
are idempotent; no runtime source extraction or test harness is needed.
