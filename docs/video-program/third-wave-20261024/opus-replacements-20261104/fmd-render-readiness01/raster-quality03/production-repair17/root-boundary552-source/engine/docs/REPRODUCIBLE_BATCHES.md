# Reproducible multi-scene exports

The optional Python portal can send a named scene list or `--write_all` through
its existing reproducible render-session path. Native rendering, frame timing,
codecs and manifest publication are unchanged. Python scenes run sequentially;
`--threads` controls each native renderer, not concurrent Python execution.

```sh
fmn-python scenes.py Intro Main Outro --reproducible --format png_sequence --video_dir output
fmn-python --robot scenes.py --write_all --reproducible --format png --video_dir stills
```

Supported reproducible batch formats are `png`, `png_sequence` and `wav`.
Named lists retain command-line order; write-all sorts locally declared scene
names. Range selection, final stills, per-scene constructor behavior and output
options continue through the existing adapters.

```python
from pathlib import Path
from fmn_python.batch_rendering import render_scenes
from fmn_python.scene_loading import SceneSource
from manimlib import Scene

with SceneSource(Path("scenes.py"), Scene) as loaded:
    report = render_scenes(
        {name: loaded.scenes[name] for name in ("Intro", "Main")},
        "output", reproducible=True, sources=lambda: loaded.sources,
        resolution=(640, 360), fps=30, threads=4,
    )
for outcome in report.outcomes:
    print(outcome.name, outcome.status, outcome.result.manifest)
```

All selected artifact and sidecar destinations are checked before any scene is
constructed. Existing destinations, including dangling sidecar links, are never
overwritten. Native no-clobber publication remains the final race authority.
Each completed scene returns its own native artifact and adjacent `.manifest`
directory; there is no new batch manifest format or all-or-nothing publication.

Static source mappings freeze before the first scene. A provider executes after
each scene so lazy imports remain in that scene's source table. The console
supplies the existing loader's provider automatically and keeps its import scope
open throughout the batch. A shared measured runtime identity is checked before
each constructor; the guarded session also performs its existing publication
checks. A runtime edit cannot silently start the next scene with a different
runtime generation. Standard mode does not perform runtime hashing.

`--keep-going` retains successful artifacts and manifests while reporting failed
scenes; without it, the first failure stops and remaining jobs are `not_run`.
Interruptions retain progress and exit 130. Robot stdout remains one aggregate
receipt, with authored output and progress on stderr. `reproducible_requested`
records intent; `all_scenes_certified` is true only when every selected scene
returned a complete certified receipt. Aggregate `certified` remains false:
the batch report itself is not a new certified artifact or scheduler claim.

Checkpoint/resume is deliberately excluded in reproducible mode. That journal
verifies ordinary artifacts, not source closures and native sidecars, and cannot
be promoted into a certified cache by a flag. Use fresh output destinations.

## Verification boundary

The 42 batch/CLI protocol cases use explicit native render, runtime, parser and
loader doubles. They exercise production orchestration, source freezing and
receipt logic, not native rendering. Four installed-native cases in
`reproducible_batch.py` are registered in `check_portal_runtime.sh`; they require
a freshly built wheel and check actual still/sequence PNG bytes across one and
four threads, console manifests and preconstruction failure.

All four native cases passed using the GitHub-built `33d78f7` extension with
current Python sources installed as a separately inventoried test payload. The
combined run exceeded this environment's execution limit after the first three
cases; the fourth passed in isolation. Runtime hashing and source guards stayed
active. This is native execution evidence, not a freshly built final-tree wheel
or full-workspace/cross-platform validation.

This exposes the existing per-scene reproducible route to whole projects. It
does not close G4b or expand the runtime provenance contract to uncontrolled
Python effects, arbitrary undeclared dependencies or cross-platform proof. See
`RUNTIME_PROVENANCE.md` for those limits. Supplied source maps remain declarations,
not automatic certification of every effect in arbitrary Python code.
