# Multi-scene Python rendering

The installed wheel can export all locally declared scenes in a source file:

```bash
fmn-python scenes.py --write_all --format gif --video_dir rendered
fmn-python --robot scenes.py --write_all --keep-going --format y4m \
  --resolution 960x540 --fps 30 --threads 4 --video_dir rendered > batch.json
```

`--video_dir` is a **directory** in this mode. File formats publish
`rendered/SceneName.gif` (or the selected extension). The default
`png_sequence` format publishes `rendered/SceneName/frames/`. Without an
explicit directory, the root is `media/videos/<source-stem>/`.

Source is loaded once through the existing portal loader. Only Scene classes
locally declared by that source are selected; imported scenes are excluded.
Names are rendered in sorted order. An individual scene name cannot be
combined with `--write_all`. Sibling modules are importable while loading and
executing the source, including imports made lazily inside `construct()`.

## Programmatic jobs

```python
from fmn_python import RenderJob, render_scenes
from my_scenes import Overview, ParameterSweep

result = render_scenes(
    [
        RenderJob("overview", Overview),
        RenderJob("seed_7", ParameterSweep, {"random_seed": 7}),
        RenderJob("seed_11", ParameterSweep, {"random_seed": 11}),
    ],
    "rendered",
    format="gif",
    resolution=(960, 540),
    fps=30,
    threads=4,
    continue_on_error=True,
)
print(result.as_dict())
```

`render_scenes` also accepts a mapping from output names to Scene classes or
pristine instances, or an iterable of classes/instances whose class names are
used. The API preserves input order. Repeated variants should use separate
names and Scene classes, not the same existing instance twice. `scene_kwargs`
is class-only and receives a shallow dictionary copy, like `render_scene`;
objects inside it are not implicitly cloned.

All names, job types, common options, and destination collisions are checked
before any Scene is constructed. Names must be identifiers of at most 128
UTF-8 bytes, without filesystem-reserved names or case/normalization
collisions. Iteration is bounded by `max_jobs` (default 1024, maximum 65536).
Relative output paths are resolved before authored code can change the
working directory. Existing outputs, including dangling destination symlinks,
are refused, never overwritten or automatically resumed. Reel repeats its
atomic no-clobber check at publication to cover races after preflight.

## Failure, cancellation, and progress

By default, the first ordinary execution error stops the batch and raises
`BatchRenderError`. Its `result` preserves all completed receipts and marks
later jobs `not_run`; the original exception remains its cause.
`continue_on_error=True` records the failure and attempts later jobs. It does
**not** make the batch successful: `result.ok` is false when any job fails.

Each outcome records its name, destination, status (`succeeded`, `failed`,
`cancelled`, or `not_run`), and either its real native `RenderResult` or a
bounded error description. Error notes preserve native cleanup diagnostics
and warnings about an artifact already published before provenance retrieval
failed. `result.as_dict()` uses schema `fmn-python.render-batch`, version 1.
Video frame counts and WAV sample counts remain distinct in native receipts.

KeyboardInterrupt and SystemExit always stop execution, even in keep-going
mode. The API re-raises the original exception after the existing
RenderSession has cancelled its owned generation. The exception's
`render_batch_result` preserves progress. A synchronous `on_result(outcome)`
callback can persist each result after native publication or failure. An
observer exception stops the batch, retains the same progress attribute, and
does not relabel an already successful publication as a failed render.

CLI exit 0 means **every selected scene succeeded**. Ordinary batch execution
failure is exit 5, including with `--keep-going`. Ctrl-C is exit 130 and emits
available progress. Scene `SystemExit(0)` is not mistaken for successful batch
completion. Invalid arguments use exit 2, unsupported capabilities exit 4,
and output preflight/reporting failures exit 6. In robot mode, stdout contains
one `fmn-python.cli` JSON document with the batch report; ordinary Python
print output and progress stream to stderr rather than accumulating in memory.

Atomicity is **per native artifact**, not per batch. Earlier completed outputs
are retained after failure or cancellation. Nothing rolls them back. The API
never cancels a generation owned by an external caller when a new open fails.

## Execution boundaries

Scenes execute sequentially on the calling thread, constructing each class
immediately before its render. They receive separate native render sessions;
`--threads` remains the native renderer's thread budget. This is not the
asupersync-backed native `fmn batch` farm, a Python worker pool, process/crash
isolation, or an independent frame loop. Python globals and imported modules
remain in the same host interpreter. The native clock, camera state, codecs,
renderer, and publication remain the existing Lumen/Choreo/Reel implementations.

All output here is standard-mode and explicitly uncertified. MP4/MOV still
require the governed optional ffmpeg capability. The original single-scene,
construct-only, list, version, parity-audit, and Studio dispatch remain in
place; this does not implement Python Studio or certified input closure.
The write-all CLI route is installed through `fmn_python.__main__.main`, used
by both the console command and `python -m fmn_python`.

`scripts/check_portal_runtime.sh` includes `tests/batch_rendering.py`: eight
real-extension cases check independently decoded Y4M motion, native digests,
thread-count replay, failure/cancellation after capture, preserved outputs,
PNG sequences, and the installed CLI. The protocol suites exercise production
orchestration and production parser/loader definitions with a native-boundary
double. Passing those protocol suites is not native-rendering acceptance.
