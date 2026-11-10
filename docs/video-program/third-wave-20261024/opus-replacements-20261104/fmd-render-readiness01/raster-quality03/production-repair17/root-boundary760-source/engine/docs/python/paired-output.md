# Animation output and a final PNG in one execution

The Python portal can publish an animation or soundtrack and an independent,
full-resolution RGBA final image without reconstructing the scene. Geometry,
clocks, updaters, RNG, output encoding and publication use the existing native
owners. The PNG is a snapshot of the scene after its lifecycle completes.

## Explicit output

```python
from manimlib import *
from fmn_python import render_scene_with_still

class Example(Scene):
    def construct(self):
        box = Square(fill_opacity=1)
        self.add(box)
        self.play(box.animate.shift(RIGHT), run_time=1)

result = render_scene_with_still(
    Example, "outputs/example.y4m", "outputs/example.png",
    resolution=(960, 540), fps=30, threads=4,
)
assert result.completed
print(result.primary.digest, result.still.digest)
```

The primary supports Y4M, GIF, PNG sequences, WAV, and the existing optional
ffmpeg-backed MP4/MOV modes. It retains its ordinary native limits, audio mix,
codec options and receipts. The final image is always a separate PNG, including
for transparent animations. `result.as_dict()` includes both receipts and a
completion flag; it is JSON-serializable. A scene instance must still be pristine
when its render generation starts. Constructor kwargs are accepted when the
first argument is a Scene class.

For imperative construction, use `paired_render_session(scene, movie_path,
image_path, ...)` as a context manager around `add`, `play` and `wait`. The context
does not invoke `construct`; `session.result` becomes available on successful
exit. Successful `finish()` is idempotent; `abort()` cancels an active session.
The session is confined to its creating thread, like its native primary owner.

## Existing SceneFileWriter preferences

`scene.render()` and `scene.render_session()` now honor the combination
`write_to_movie=True, save_last_frame=True` when neither destination nor format
is explicitly supplied. The normal writer paths are used for both outputs.
With `subdivide_output=True`, the primary is the existing per-call clip collection
and one final PNG accompanies it. An explicit ordinary destination or format
continues to select a single output; use the paired functions to select two
explicitly.

## Failure and publication semantics

The PNG is captured, encoded and prepared privately **before** primary
finalization, and published **after** the primary finishes successfully. Failed
construction, failed capture, an existing still, or failed primary finalization
cancels the unpublished PNG. Existing files and dangling symlink destinations
are refused; movie and still destinations cannot overlap. Caller paths are
made absolute before entering the generation so later `chdir` does not redirect
publication.

Each artifact is atomic and create-only, but two arbitrary filesystem paths are
**not one transaction**. A concurrent writer can claim the PNG path after
preparation. In that case the movie remains published and the original exception
has `render_pair_result`: `completed=False`, the completed primary receipt, and
no still receipt. A failed subdivided render similarly retains earlier completed
clips. Artifacts are never destructively rolled back or silently overwritten.
Missing receipts after reported publication are listed separately rather than
represented as successful outputs.

The observational final PNG uses the standard fast-CPU camera path. The paired
workflow refuses reproducible/certified implicit rendering; it never inherits a
certification claim from a different artifact. A thread-invariance test is not
cross-platform certification. `animation_range` applies to the primary's existing
scene execution semantics; the still observes the state that execution reaches.

## Publish an existing immutable capture

```python
snapshot = scene.camera.capture_snapshot(*scene.mobjects)
path, size, digest = snapshot.save_png("outputs/still.png", threads=4)
```

`scene.camera.save_png(...)` publishes its last captured frame without recapture.
`scene.file_writer.save_final_image(snapshot)` uses its configured image path and
accepts a native capture, not an arbitrary array or Pillow image. For coordinated
publication, `snapshot.prepare_png(...)` returns an owner with `commit()` and
`abort()`; aborting or dropping an unpublished preparation removes only its
private native file. A committed preparation returns the same receipt on repeated
commit and never deletes the published image on abort.

## Console and ordered batches

```bash
fmn-python scenes.py Example --format y4m --save-last-frame
fmn-python scenes.py Intro Main Outro --format mp4 --save-last-frame --robot
fmn-python scenes.py --write_all --format gif --save_last_frame --keep-going
fmn-python scenes.py Example --format y4m --subdivide --save-last-frame
```

Both spellings select the same boolean option. Native option parsing still owns
resolution, fps, codec parameters and output directories. This switch never
consumes another flag's value, and duplicate aliases are rejected. It cannot be
combined with `-s`/`--skip_animations`, still-only formats, certified output, or
checkpoint/resume. Incompatible modes are refused before importing scene code.

A single file output `Example.mp4` is paired with `Example.png`. Directory
outputs use an adjacent image: `Example/frames` with `Example/frames.png`, or
`Example/clips` with `Example/clips.png`. Neither the still nor an extra frame
is inserted into the primary sequence/clip collection. The usual `--video_dir`
meaning is retained: a destination for a single scene and an output root for
named/`--write_all` batches.

The same workflow is available programmatically:

```python
from fmn_python import render_scenes

report = render_scenes(
    {"intro": Intro, "main": Main}, "outputs",
    format="y4m", save_last_frame=True, threads=4,
)
for outcome in report.outcomes:
    print(outcome.name, outcome.result.primary.digest, outcome.result.still.digest)
```

Batch admission checks **both destinations for all selected scenes before the
first constructor**. Each scene executes once, sequentially on its creating
thread. Success requires both receipts. A failure retains an incomplete paired
receipt in the outcome when available; `continue_on_error=True` / `--keep-going`
continues only after ordinary exceptions. KeyboardInterrupt/SystemExit still
cancel the batch, preserving completed results and marking later jobs not run.

Robot output emits one JSON record; source/constructor chatter goes to stderr.
Single success has kind `render-paired` with a `pair` object. Batch outcomes
carry the pair under `result`; failures never claim whole-scene success merely
because the movie was already published. Animation-range selection applies to
the existing primary execution, and the PNG observes the resulting final state.
