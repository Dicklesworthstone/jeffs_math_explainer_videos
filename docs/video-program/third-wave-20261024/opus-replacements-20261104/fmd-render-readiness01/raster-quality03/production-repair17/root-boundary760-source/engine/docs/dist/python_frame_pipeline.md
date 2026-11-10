# Native frame-stage overlap in the Python portal

File exports use the existing native `FramePipeline` to overlap scene execution,
rasterization, conversion, and ordered output. `Scene.render`,
`Scene.render_session`, `render_scene`, `render_scenes`, and the console share
this route; no separate parallel-play API or new scene syntax is required.

```python
from manimlib import Scene, Square, RIGHT

class Motion(Scene):
    def construct(self):
        shape = Square(fill_opacity=1)
        self.add(shape)
        self.play(shape.animate.shift(3 * RIGHT), run_time=2)

result = Motion().render("frames", format="png_sequence", threads=8)
```

## What runs concurrently

The scene, animation callbacks, updaters, and exported NumPy views stay on the
calling thread. The rational clock and six-step frame order are unchanged.
At each capture, the owner freezes the native camera, geometry, painter order,
style, light, and image resources. Workers consume only those owned inputs.
A later view write, camera move, texture replacement, or scene teardown cannot
change an already captured frame.

Workers use the same Lumen camera rasterizer and Reel format converters as the
synchronous route. Native output includes PNG, PNG sequences, GIF and Y4M;
the existing MP4/MOV encoder routes consume the same ordered frame stream.
SVG and soundtrack-only WAV do not use a raster pipeline. Studio preview and
explicit camera readback retain their existing separate capture paths.

The runtime derives render teams and a finite in-flight frame count from its
existing execution plan. A source permit is acquired **before** materializing
and compiling the next native frame, so a slow sink cannot cause an unbounded
queue of captured geometry. Raster buffers and conversion scratch are reused.
This is bounded pipelining, not a claim of allocation-free capture, a particular
speedup, or CPU affinity enforcement.

## Completion, cancellation, and Python responsiveness

Capacity waits, frame submission, normal native drain/publication, and explicit
cancellation release the GIL. Other Python threads can make progress during
those waits; this does not permit moving a Scene or its proxies to another
thread, or making simultaneous calls on the same scene.

Completed frames reach Reel in sequence order even when render teams finish
out of order. Leaving a successful render session drains and joins the native
pipeline before the sink is finalized. Output publication remains atomic and
no-clobber. Pipeline completion alone is not an artifact publication receipt.

An authored exception, including `KeyboardInterrupt`, cancels the sink before
joining the native workers. A worker-side error can surface at a subsequent
capture or at session finish; a partial artifact is not published as success.
Existing destination files are never removed to recover a failed generation.
Use a new Scene for a subsequent render, or the batch checkpoint API to resume
unfinished scenes while preserving completed artifacts.

## Native integration surface

Rust hosts can use `RetainedFrameRenderer::prepare_with_camera` to create a
`PreparedCameraFrame`, then render it on a native worker with `render_into`.
The owned frame is `Send + Sync`; it contains no Stage, Python proxy, or live
record view. Zero-worker requests are rejected before allocation or destination
mutation. `FrameStream` exposes the existing scheduler to imperative producers
through `reserve` / permit `submit`, with explicit `finish` and cancellation.
An unused permit cancels instead of silently dropping a semantic frame.

## Regression evidence

`render_matrix.python_frame_pipeline.v1` executes the real portal and native
sinks. Its acceptance source compares twelve changing camera/live-view frames
against synchronous camera readback, checks byte-identical PNG sequences at
three worker budgets, verifies GIL responsiveness, and exercises cancellation,
competing publication, and independent generations. Native retained-renderer
tests separately cover frozen mixed vector/surface/dot/image scenes and texture
replacement. Runtime tests assert source-capacity bounds and ordered draining
under stage errors, panics, and cancellation.
