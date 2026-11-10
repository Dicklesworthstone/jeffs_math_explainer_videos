# Multi-scene clip collections

Use the existing batch API with `subdivide=True` to render separate clip
collections without writing an outer execution loop:

```python
from fmn_python import RenderJob, render_scenes

report = render_scenes(
    [RenderJob("intro", Intro), RenderJob("detail", Detail, {"scale": 2})],
    "media/lesson",
    subdivide=True, format="y4m", resolution=(1280, 720), fps=30, threads=4,
    max_segments=1000, continue_on_error=True,
)

for outcome in report.outcomes:
    print(outcome.name, outcome.status)
    if outcome.result is not None:
        for clip in outcome.result.segments:
            print(clip.play_index, clip.render.destination, clip.render.digest)
```

Outputs live under `media/lesson/intro/clips/` and
`media/lesson/detail/clips/`. Each child retains its original zero-based
play/wait index. The selected format must be supported by the installed native
subdivision capability; still PNG/SVG formats are not clip collections.
`max_segments` is the completed-clip budget **per scene**. `max_jobs` limits
the batch independently. `animation_range=(start, exclusive_end)` applies
independently to each scene.

Scene classes, pristine instances, and named `RenderJob` values retain the
ordinary batch API's order and constructor-argument semantics. Names, duplicate
instances, destination collisions, and common mode/resource options are
validated before scene construction. Scenes execute sequentially on the
calling thread, while each existing native renderer uses the requested thread
budget. No Python scene or live proxy is sent to a worker thread.

## Completed and partial output

Each selected call publishes one atomic, no-clobber native artifact. A
successful scene outcome has `status="succeeded"` and a completed collection
receipt in `result`. A failed or cancelled scene can also have a `result`, but
it is an **incomplete** collection receipt identifying its already published
clips and any artifacts whose publication completed before receipt processing
failed. Constructor failures occur before collection ownership and have no
invented collection receipt.

Default fail-fast raises the existing `BatchRenderError`; its `result` retains
successful scenes, the failing scene's partial collection, and `not_run` jobs.
`continue_on_error=True` proceeds to the next scene after ordinary failures,
but never swallows `KeyboardInterrupt` or `SystemExit`. Interruptions and
observer exceptions carry `render_batch_result`; an observer cannot relabel
a completed publication as a failed render. Earlier clips are never deleted
or rolled back.

The single-scene `render_subdivided_scene` function now attaches
`render_subdivision_result` to the original execution exception when it owned
a collection. The attribute contains receipts, not a live scene or session.

Checkpoint/resume and certified-output requests are explicitly refused for
subdivided batches. The existing checkpoint stores single-artifact receipts;
it must not silently treat a partial directory as a verified scene output.
Ordinary `render_scenes` behavior is unchanged when `subdivide=False`.

`crates/fmn-python/tests/subdivided_batch.py` exercises actual native frames,
thread-count equivalence, selection, constructor/lifecycle order, partial
output, fail-fast/keep-going, interrupts, observers, admission and directory
collision refusals, and unchanged ordinary batch output.

## Console selection

```sh
fmn-python --robot lesson.py Intro Detail --subdivide --format y4m \
  --resolution 1280x720 --fps 30 --threads 4 --video_dir media/lesson

fmn-python --robot lesson.py --write_all --keep-going --subdivide \
  --format gif --video_dir media/all-scenes -n 1,5
```

Explicit names retain command-line order; write-all selects locally declared
classes in sorted order. The output root may already exist, but every selected
scene's `clips` directory must be new. All names and collection destinations
are preflighted before any scene is constructed. Source-range selection applies
to every job. `--keep-going` cannot suppress Ctrl-C; its ordinary failures
still produce a nonzero exit status.

The existing `render-batch` robot envelope carries collection receipts inside
each outcome's `result`. Failed and cancelled scene results can be partial
collections. Single-scene commands retain `kind="render-subdivided"` and the
existing direct `subdivision` field. The console does not invent another
parser, loader, batch scheduler, or publication protocol.
