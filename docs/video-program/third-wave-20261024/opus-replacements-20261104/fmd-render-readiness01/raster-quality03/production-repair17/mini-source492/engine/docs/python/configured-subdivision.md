# SceneFileWriter-configured subdivision

`Scene.render()` and `Scene.render_session()` honor the existing
`file_writer_config={"subdivide_output": True}` setting. They select the same
native per-call collection owner as `render_subdivided_scene`, not an ordinary
single-file render which silently ignores the subdivision request.

```python
from manimlib import Scene, Square, RIGHT

class Motion(Scene):
    def construct(self):
        square = Square(fill_opacity=1)
        self.add(square)
        self.play(square.animate.shift(RIGHT), run_time=1)
        self.wait(.5)

scene = Motion(file_writer_config={"subdivide_output": True})
result = scene.render("clips", format="gif", resolution=(640, 360), fps=24)
assert result.completed
assert scene.render_result is result
```

Each selected top-level play/wait creates one native clip; the constructor and
lifecycle are not replayed for each output. Existing camera-frame identity,
source-range selection, output budgets, no-clobber publication, and authored
writer begin/end hooks retain their existing owners.

`scene.render_session(...)` selects a collection under the same setting, but
never calls setup/construct/tear_down. It remains side-effect free until
context entry and requires a pristine scene. Use `record_subdivided_scene` to
record an already populated scene without resetting its clock.

## Destination and format

An explicit destination is the **collection directory**, not one video file.
An explicit `format` selects the clip format. Without it, a recognized suffix
selects the format; a still PNG/SVG suffix is rejected rather than converted.
Otherwise `write_to_movie=True` selects the configured movie extension, and
nonmovie output defaults to per-call PNG sequences. Without an explicit
destination, output goes under `<writer-output-root>/clips/`.

A last-frame-only preference cannot be subdivided. Simultaneously requesting
movies and a final still is refused unless one clip format is explicitly
selected. An audio-bearing format requires the matching native capability;
no requested format is silently replaced when a capability is unavailable.

The result is a `SubdividedRenderResult`. On an execution failure, the original
exception carries `render_subdivision_result`, identifying completed clips
without claiming collection success. Imperative callers can inspect the
context's `partial_result`. Earlier completed clips are not deleted.

## Ordinary and certified output

With `subdivide_output=False`, the Scene methods retain their single-artifact
`RenderResult` contract. The free `render_scene` and `render_session` functions
remain explicit single-artifact APIs; select `render_subdivided_scene` or
`subdivided_render_session` for free-function collection rendering.

The provenance wrapper and the base Scene binding share session selection and
lifecycle execution. Ordinary reproducible renders still acquire and verify
the same runtime/source closure. Subdivided host-history recording is not a
certified closure, so `reproducible=True` is explicitly refused before scene
execution or native output acquisition. The setting cannot bypass runtime
provenance validation or falsely label clip collections as certified.
