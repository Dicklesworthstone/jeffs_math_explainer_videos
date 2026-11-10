# Live source definitions in the embedded editor

The repository's Python portal can refresh a file-backed project's definitions
while keeping the existing native Scene, mobjects, clock and checkpoints alive.
IPython is an optional host dependency; this feature never starts a second
Python process or reloads the native extension.

Run the included example from a matching installed wheel with host IPython:

```bash
python demo/python/live_source_reload.py
```

At the embedded prompt:

```python
auto_reload()
place(square)
```

Edit the offset inside `place` in the example file and save it. The next
`place(square)` uses the new function on the same existing native square.
`auto_reload()` is idempotent and checks observed project source contents before
each cell. Unchanged cells do not reexecute the project. The existing
`manim_config.embed.autoreload` setting enables this automatically when the
embedded session installs its post-cell preview hook. Source changes are also
available explicitly through `reload_source()`.

## What is refreshed

Project modules use the same scoped loader as ordinary source execution.
Observed transitive helpers, package-relative imports and local namespace
packages are refreshed from compilation bytes, not timestamp-validated pyc
files. Previously observed files are syntax-checked before the reload executes.
Newly imported files share the same 4,096-source / 64 MiB reload budget.

Updated source-owned names are published into the active cell namespace and its
module globals, so functions defined in interactive cells also see refreshed
source helpers. Interactive local overrides, native scene references, editor
shortcuts and IPython internals are retained. Deleted source names disappear
only while the shell still holds the previous source-owned value.

A syntax or import failure preserves the last working project import graph and
namespace. IPython displays a failed pre-cell hook and can execute the cell
against those last working definitions; the next corrected edit retries. A
forced `reload_source()` failure propagates through the normal cell exception
path. Exiting the embedded session unregisters callbacks and closes only source
contexts the editor acquired, not an enclosing caller's `SceneSource`.

For a host that already owns its source context, the same mechanism is directly
available:

```python
from fmn_python.scene_loading import SceneSource
from manimlib import Scene

with SceneSource("my_scene.py", Scene) as loaded:
    # Use discovered classes/functions while their module context is active.
    module = loaded.reload(if_changed=True)
```

## Deliberate limits

This is **definition reload**, not full scene reconstruction. Existing objects
retain their class identity and existing callbacks until the author deliberately
replaces them. `reload()` / `reload_scene()` keeps its existing full-rebuild
capability boundary; it is not silently redirected to this smaller operation.

Import rollback cannot undo arbitrary authored host effects: files, sockets,
external mutable objects, native mutations deliberately performed at import,
and previously published output remain real effects. This is not a sandbox.
Changed generations retain the one-version-per-path provenance refusal rather
than claiming that a mixed-source render has a single certified source closure.
Autoreload refuses certified output generations and active scene segments.
Declared Studio-worker sources reload through the supervisor, never through a
host IPython callback. The engine, NumPy and interpreter modules are not reloaded.

The installed-wheel acceptance is
`crates/fmn-python/tests/source_autoreload.py`, registered in
`scripts/check_portal_runtime.sh`. It requires the actual native extension and
host IPython and drives the real embedded mainloop without a blocking prompt.
The separate `test_source_*` and `test_embedded_source_identity.py` suites exercise
actual imports/IPython with explicit minimal native-ownership test doubles;
they do not establish native pixel or full Studio acceptance.
