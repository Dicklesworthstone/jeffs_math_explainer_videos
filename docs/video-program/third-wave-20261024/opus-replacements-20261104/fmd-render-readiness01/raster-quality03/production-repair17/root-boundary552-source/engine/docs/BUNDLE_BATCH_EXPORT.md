# Exporting multiple portable scenes

The existing Python batch API can now capture independent FMTL artifacts:

```python
from fmn_python import RenderJob, render_scenes

report = render_scenes(
    [RenderJob("Intro", Intro), RenderJob("Orbit", Orbit)],
    "bundles",
    format="fmtl",
    bundle_camera=True,
    resolution=(960, 540),
    fps=30,
    bundle_limits={
        "max_frames": 30_000,
        "max_capture_bytes": 128 * 1024 * 1024,
        "max_output_bytes": 128 * 1024 * 1024,
    },
    continue_on_error=True,
)
```

This creates `bundles/Intro.fmtl` and `bundles/Orbit.fmtl`. A mapping or an iterable
of Scene classes, pristine instances, or `RenderJob` entries is accepted, using
the same name validation and collision rules as pixel/video batches. Constructor
arguments can be supplied through each `RenderJob.scene_kwargs`.

`bundle_camera=True` uses the native camera-aware recorder: each captured view,
light and background travels with its geometry. The default `False` retains the
planar, browser-compatible bundle. These are the same formats and the same
native writer used by single-scene `export_bundle`; the batch does not implement
another animation loop, serializer, clock, or renderer.

## Ownership, bounds and partial success

The batch planner checks every job and destination before constructing the first
scene. Recorder capabilities, explicit resolution/FPS, and bundle limits are
also checked first. `bundle_limits` accepts only `max_frames`,
`max_capture_bytes`, and `max_output_bytes`. Its values are copied and normalized
before execution, so an authored scene cannot change subsequent jobs' bounds by
mutating the caller's mapping. Limits apply independently to each scene;
`max_jobs` bounds the number of planned jobs.

Each scene runs sequentially on the calling Python thread through one
`BundleExportSession`. Native scene/proxy ownership never crosses a thread.
Exported bundles can subsequently be replayed by the standalone native engine;
exporting them does not create a Python render farm.

Publication is atomic **per artifact**, not across the batch. A later failure
never deletes a completed bundle. Fail-fast raises `BatchRenderError`, whose
`result` contains succeeded, failed and not-run outcomes. With
`continue_on_error=True`, ordinary failures are recorded and later jobs run.
KeyboardInterrupt and SystemExit always stop execution and carry progress in
`render_batch_result`. An observer failure preserves the already-published
receipt and never relabels its artifact as a failed render.

Successful outcomes contain `BundleExportResult`, including native frame/segment
counts, byte count and SHA-256. Camera outcomes identify FMTL minor 1. Neither
captured bundles nor the batch claim source-effect certification.

Pixel-output options, `threads`, partial animation selection, subdivision,
paired last-frame output and source certification are
explicitly unsupported in this mode. They are refused before scene construction,
rather than being ignored or written into an incompatible checkpoint receipt.
Audio remains outside the current bundle format.

## Tests

`test_bundle_batch_protocol.py` exercises batch planning, strict preflight,
immutable limits, failure ordering and ownership against an explicitly modeled
native boundary. `bundle_batching.py` runs against a real installed extension:
batched planar/camera bytes equal independent single-scene exports; exact native
byte limits and one-byte-too-small refusals are verified; swallowed capture
failures, cancellation, collision and observer exceptions preserve publication
and recovery semantics. The installed-wheel runtime gate includes this suite.

## Console selection

The ordinary console front door accepts the same bundle mode and batch owner:

```sh
python -m fmn_python --robot scenes.py Orbit Diagram \
    --format fmtl --bundle-camera --video_dir bundles

python -m fmn_python --robot scenes.py --write_all \
    --format fmtl --bundle-camera --keep-going --video_dir bundles
```

Explicit names run in the requested order. `--write_all` (or `-a`) selects only
locally declared scenes and runs them in sorted name order. Imported classes
are not silently added. The source and sibling imports remain scoped through
the entire batch; source code is loaded once, not once for every output.

With multiple names or `--write_all`, `--video_dir` is a directory, and every
scene writes `DIRECTORY/SCENE.fmtl`. With a single selected scene and no
`--write_all`, its existing exact-file meaning and single-export receipt stay
unchanged. Omitting the flag keeps the existing planar/browser-compatible
format. `--bundle-camera` selects native camera-bearing bundles for every job.

All selected names and destinations are preflighted before any constructor.
An occupied destination for a later scene prevents the entire batch from
starting; a concurrent writer is still protected by each native publisher.
The default stops on the first ordinary scene error. `--keep-going` permits
later scenes to run, but any failure still yields exit 5. Ctrl-C always stops
with exit 130 and `SystemExit`, even `SystemExit(0)`, stops with a nonzero code.
Already published bundles survive and appear in the aggregate receipt.

Robot mode writes one terminal JSON batch receipt to stdout. Authored print
output and per-scene progress go to stderr. The receipt records succeeded,
failed, cancelled, and not-run jobs separately. There is no new scene loop,
renderer, worker scheduler, or shared mutable native owner in this route.


## Checkpoint and resume

Long batches can keep completed native bundles across failures or interruptions:

```python
options = dict(
    format="fmtl", bundle_camera=True, resolution=(960, 540), fps=30,
    checkpoint="progress.json", resume_key="scene-and-assets-v3",
)
report = render_scenes(jobs, "bundles", **options)
# A later invocation retries only failed, cancelled, or unattempted jobs:
report = render_scenes(jobs, "bundles", resume=True, **options)
```

The console accepts the same recovery options with named batches or write-all:

```sh
python -m fmn_python --robot scenes.py --write_all --format fmtl \
    --bundle-camera --video_dir bundles \
    --checkpoint progress.json --resume-key scene-and-assets-v3

python -m fmn_python --robot scenes.py --write_all --format fmtl \
    --bundle-camera --video_dir bundles \
    --checkpoint progress.json --resume-key scene-and-assets-v3 --resume
```

This uses the existing versioned, atomically written batch journal and exclusive
kernel lock. It does not add a bundle format, replay renderer, or source cache.
The saved plan binds job order, names, constructor inputs, explicit resolution
and FPS, camera mode, and normalized per-scene capture/output limits. These
settings must match on resume, including limits omitted in favor of defaults.
Plain JSON constructor arguments have the same isolated value semantics as
other checkpointed batches. Invalid settings are refused before construction.

Before executing any unfinished scene, every completed artifact is rehashed as
a regular, non-symlink file and compared with its inventory and original native
publication receipt. Bundle receipt schemas, counter types, SHA-256, dimensions,
FPS, camera flags, and bounds are checked independently. Unknown future receipt
or camera versions fail closed. A damaged/missing file, changed plan, or output
that exists without a completed journal entry is an error, never permission to
rewrite or silently trust an artifact.

Progress is journaled after native publication and integrity validation, before
observers run. A completed scene is not constructed or executed on resume; its
immutable bundle receipt is restored with the same public shape. All-completed
resumes run no scenes. The console still loads the source module to discover
classes; this is not a promise that top-level Python effects are skipped.

The caller supplies `resume_key` as an explicit version of scene code/assets.
Change it when inputs for completed scenes change, then use fresh destinations
and a new checkpoint. Neither the journal nor successful hash verification
certifies Python source effects. A crash between artifact publication and
journal publication leaves an unrecorded file which must be dealt with
explicitly; it is never auto-adopted, overwritten, or deleted.

The native recovery suite exercises both planar and camera bundles, partial
success, interruption after frame capture, complete reuse, exact byte bounds,
process-level console recovery and output corruption. Protocol suites test
malformed receipts, plan changes, locks, symlinks and option-value parsing.
