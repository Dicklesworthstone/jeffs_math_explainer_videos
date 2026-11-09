# Checkpointed host scene editing

`fmn_python.SceneConsole` executes cells against an existing native `Scene`,
without an output file, separate renderer or additional clock. It is useful in
notebooks and host applications as well as embedded terminals.

```python
from manimlib import Scene, Square, RIGHT
from fmn_python import SceneConsole

scene = Scene()
box = Square(fill_opacity=1)
scene.add(box)

with SceneConsole(scene, {"box": box, "RIGHT": RIGHT}) as console:
    console.run_cell("# position\nbox.shift(RIGHT)")
    # Restore the state before # position, then apply the replacement cell.
    console.run_cell("# position\nbox.shift(2 * RIGHT)")
    pixels = scene.camera.get_pixel_array()  # Actual native Lumen RGBA pixels.
```

A leading comment is a checkpoint key. Its first execution captures the
existing native `SceneState`; a retry restores that state and discards later
named checkpoints. Geometry, root/family identity, camera and scene time follow
the same snapshot/restore contract as `Scene.get_state()` and
`Scene.restore_state()`. The console does not serialize Python functions or
invent another snapshot format.

Syntax is checked **before** a checkpoint is restored. A runtime exception or
interrupt preserves its identity and leaves the actual partial effects visible;
the next execution of the same key can restore the scene. This is **not** a
transaction on the Python namespace, filesystem, network or already published
output. Named Python variables and external effects are not rolled back.

`run_cell(..., skip=True, progress_bar=False)` uses `Scene.temp_config_change`
and restores its settings on failure. `record=True` delegates to the same
native recording capability and currently refuses when the insert-recording
path is unavailable; it is not silently ignored. `capture=False` explicitly
turns off the post-cell native camera readback.

An application can pass `clipboard=my_text_reader` and call
`console.checkpoint_paste()`. The reader is invoked once per explicit paste.
No clipboard program is searched for or launched. `run_cell(text)` requires no
clipboard. An optional host-owned IPython-compatible shell can be supplied as
`shell=...`; it owns magic and top-level-await execution. Neither IPython nor a
clipboard package is added to the wheel's runtime dependency closure.

The console belongs to its creating thread, excludes another console on the
same scene, and refuses reentrant execution during a cell, active `play/wait`,
or an owned output generation. Native snapshot failures (including view or
output ownership restrictions) propagate without executing the new cell.
Closing releases the console's checkpoint and namespace references. A caller's
original namespace dictionary or shell is not cleared.

Defaults bound one source cell to 1 MiB and retained named checkpoints to 64.
`max_source_bytes` and `max_checkpoints` select explicit limits; exceeding them
refuses rather than silently evicting a checkpoint. Cell execution itself is
arbitrary host Python: there is no execution-time or security-sandbox claim.

This host editing route does not add an HTTP code-execution endpoint, replace
the isolated Studio worker, make offline native `fmn` depend on Python, or claim
certified input-closure/reload replay for arbitrary Python effects.


## Literal IPython embedding

`InteractiveScene.embed(namespace)` now launches an actual managed
`InteractiveSceneEmbed` shell. `get_ipython_shell_for_embedded_scene()` returns
an unlaunched shell, and `launch()` retains and returns that shell instead of
losing it behind an `IPython.embed()` call. Existing compatibility classes and
qualified import identities remain unchanged. `Scene.embed()` retains its
existing headless/offline behavior.

The host can explicitly grant a clipboard reader through the convenience API:

```python
from fmn_python import embed_scene

shell = embed_scene(scene, {"box": box, "RIGHT": RIGHT},
                    clipboard=my_text_reader)
```

For a lower-level host, construct `InteractiveSceneEmbed(scene)`, assign its
`clipboard` callable, then call `launch()`. No optional clipboard package or
external clipboard command is automatically located. Without a clipboard,
ordinary IPython cells still work; `SceneConsole.run_cell(text)` is the
clipboard-free checkpointed path. IPython is imported only when launching the
shell, so headless scenes and the programmatic cell runner do not require it.

The shell provides scene-bound `play`, `wait`, `add`, `remove`,
`checkpoint_paste`, and `clear_checkpoints` shortcuts. Pasting preserves
`skip`, `record`, and `progress_bar` options. A post-cell event updates native
preview even for ordinary terminal cells. Scoped event callbacks are removed
on exit or interruption; IPython singleton slots and `sys.excepthook` are
restored, preserving an existing notebook's active shell. Terminal nesting is
refused. The returned shell retains its host namespace; it is not a serialized
scene replay.

`InteractiveScene.checkpoint_paste()` no longer returns checkpoint **bytes**.
It runs a cell in the active console/embed session and refuses without one.
Consumers intentionally requesting the old byte snapshot must use
`Scene._checkpoint_bytes()` explicitly. Direct `CheckpointManager` snapshots
remain available through `handle_checkpoint_key`; pasting requires an owned
session rather than guessing clipboard authority.

The isolated Studio worker reserves stdin/stdout for its native framed
protocol, including during source imports and constructors. A request to start
an embedded shell there is a capability error, never a terminal that blocks or
consumes protocol messages. The existing Studio source-watch/reload path is
unchanged. IPython in-place module reload, GUI input hooks, error-border flashes,
and insert-recording are not implemented by this change and retain their
explicit capability refusals. No Python namespace or external-effect rollback
is promised.
