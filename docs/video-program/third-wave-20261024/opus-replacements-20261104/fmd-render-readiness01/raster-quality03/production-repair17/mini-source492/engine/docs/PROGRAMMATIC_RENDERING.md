# Programmatic Python rendering

The installed wheel exposes `Scene.render`, `fmn_python.render_scene`, and
`fmn_python.render_session`. All three use the existing native Lumen renderer,
Choreo clock, and Reel output generation. They do not launch the command-line
program, another Python interpreter, or a second renderer.

## Render a scene

```python
from manimlib import Scene, Square, RIGHT, linear
from fmn_python import render_scene

class Example(Scene):
    def construct(self):
        square = Square(fill_opacity=1)
        self.add(square)
        self.play(square.animate.shift(2 * RIGHT), run_time=1, rate_func=linear)

result = render_scene(
    Example,
    "example.gif",
    resolution=(960, 540),
    fps=30,
    threads=4,
)
print(result.as_dict())
```

A Scene class is instantiated once. Pass constructor arguments through
`scene_kwargs`; an existing, pristine Scene instance is also accepted.

The equivalent instance front door is:

```python
scene = Example(camera_config={"resolution": (960, 540), "fps": 30})
result = scene.render("example.y4m", threads=4)
```

`Scene.render` delegates to the scene's `run` hook. `run` itself remains the
lifecycle-only operation, preserving construct-only execution and CLI-owned
render generations. The wheel installs `render` on the existing Scene class,
so qualified imports keep the same identity and authored subclass overrides
remain in force. Direct ExtensionFileLoader consumers can call `render_scene`
or `render_session` without the wheel's Scene-method installation.

## Record imperative operations

```python
from manimlib import Scene, Square, RIGHT
from fmn_python import render_session

scene = Scene(camera_config={"resolution": (960, 540), "fps": 30})
with render_session(scene, "frames", format="png_sequence", threads=4) as output:
    square = Square(fill_opacity=1)
    scene.add(square)
    scene.play(square.animate.shift(2 * RIGHT), run_time=1)
    scene.wait(0.5)

print(output.result.destination)
```

A generation must begin **before** the Scene adopts mobjects or advances its
clock. Construct geometry in `construct`, or inside the recording context.
The native pristine-Stage check remains authoritative; this interface does
not reset or discard an already-built Scene to make rendering succeed.

## Output selection and receipts

Supported formats are `png`, `png_sequence`, `gif`, `y4m`, `wav`, `mp4`, and
`mov`. A recognized file suffix selects its format; a suffix-free destination
selects a PNG-sequence directory. Use an explicit `format` for other directory
names. Unknown formats are rejected, never silently substituted.

Without a destination, `Scene.render()` uses the SceneFileWriter's path
methods. `save_last_frame=True` selects PNG; `write_to_movie=True` selects the
configured movie extension. Otherwise it writes a PNG sequence beneath the
writer's output root in `frames/`. An explicit destination or format selects
one output independently of those preferences. Requesting a movie and a final
PNG together, without selecting one explicitly, is currently a capability
error rather than a silently dropped output.

The returned `RenderResult` records the actual published path, byte count,
native digest, renderer identity, used thread count, resolution, frame rate,
and seed. WAV reports `sample_frames`, not a fictitious video-frame count.
MP4/MOV retain the native ffmpeg invocation facts. All results are explicitly
**standard-mode, not certified**. Format support and orchestration do not
establish the complete certified Python input closure.

Publication and no-clobber behavior belong to Reel. No success receipt exists
until native finish succeeds. Before successful finish, an exception or
KeyboardInterrupt cancels the owned generation; cleanup failures are attached
to the original exception instead of replacing it. `EndScene` during
`render_scene` or `Scene.render` is normal early completion. A failed open
never aborts someone else's active generation. Sessions are thread-confined.
Calling `finish()` explicitly publishes immediately; subsequent exceptions do
not undo that already-published artifact.

## Current boundaries

Custom ffmpeg executable/codec/pixel-format settings, subdivision, opener
flags, nondefault gamma/saturation, and non-RGBA PNG mode are rejected when
relevant. They are not implemented by ignoring the caller's settings. MP4/MOV
still require the governed optional ffmpeg capability. In-memory Camera
readback/get_image and the Python Studio worker are separate, unfinished
interfaces; this change does not claim to implement them.

Run `scripts/check_portal_runtime.sh` against the installed wheel to exercise
`programmatic_rendering.py` together with the existing output suite. Its 11
cases decode PNG and Y4M independently, check real motion, verify native
receipts, replay at another thread count, exercise imperative recording and
configured paths, retain existing destinations, cancel after capture, and
validate early completion and PCM WAV output. Source-level protocol tests
are separate and do not establish native-rendering acceptance.
