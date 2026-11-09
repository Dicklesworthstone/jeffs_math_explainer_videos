# Python Studio preview

Run an existing Python Scene in a disposable process and inspect its native
frames in the same authenticated Studio UI used by native scene workers:

```bash
fmn-python studio lesson.py Example
fmn-python --robot studio lesson.py Example --resolution 960x540 --fps 24
fmn-python studio lesson.py Example --autoreload --watch assets/values.csv
fmn-python studio lesson.py Example --interactive --autoreload
```

Or keep the preview alongside other Python work:

```python
from fmn_python.studio import Studio

with Studio("lesson.py", "Example", resolution=(640, 360), fps=30) as preview:
    print(preview.url)
    input("Press Enter to close Studio")
```

The optional installed Python wheel owns the interpreter. The standalone `fmn`
executable has not acquired a CPython dependency, locator, or Python launcher.
The worker uses the exact host interpreter/virtual environment and installed
engine, not a `python` found later on PATH. Install the wheel before launching;
source-tree `PYTHONPATH` overlays are deliberately not propagated to a worker.

## Capture, playback and reload

The worker loads the selected source, constructs the requested class normally,
and runs its ordinary lifecycle once. `play`, `wait`, native/Python updaters,
and camera synchronization use the existing scene scheduler. Lumen's retained
camera renderer supplies the actual PNGs. Matching native inspector snapshots
are captured at those same frame boundaries. Static scenes receive one final
capture, rather than a blank or lifecycle-only “preview.”

The UI becomes available after this initial capture. Its playback and scrub
controls read the captured timeline, including reverse scrubbing, without
executing the source or its side effects again. The inspector's mobject records
and hierarchy follow the displayed frame. The viewport is read-only by default.
`--interactive` additionally retains the actual scene and its local import
context in the worker, allowing input at its final frame as described below.

**Reload executes source again**, in a new worker with fresh helper imports,
and starts the new timeline at frame zero. Equal-size, same-timestamp source
edits do not reuse old Python bytecode. Starting at zero also permits a shorter
edited scene to replace a longer one. A syntax error in the entry source is
reported before replacing the current worker. A runtime failure is contained
in the disposable worker; the last displayed PNG and stable host remain, and
fixing the source permits another explicit Reload. A failed capture is not
published as a successful truncated timeline, even if scene code catches its
exception. Automatic crash re-execution is disabled.

### Automatic source reload

`--autoreload` explicitly authorizes re-executing authored source after stable
edits. It uses Studio's native content-based watcher, not modification times:
the entry file and local `.py`/`.pyw` helpers are watched by default, including
new and deleted modules. Repeated `--watch PATH` arguments add source directories
or explicit files of any type, such as CSV inputs or image assets. Directory
scans ignore non-Python output files, `__pycache__`, virtual environments, Git
metadata and Cargo's `target`. This avoids reload loops caused by generated
PNGs, bytecode, or logs. Explicit asset files remain watched regardless of
their extension. The snapshot is taken before the initial worker executes, so
an edit made during a slow initial capture is not missed.

Changes are debounced (`--debounce_ms`, default 200). A failed edit produces one
failure status and leaves the old worker/preview usable. That same unchanged
source is not continually retried; fixing it schedules a fresh attempt.
Scanning itself is bounded to 4,096 entries, 64 MiB of selected file bytes and
64 directory levels. Symlinks/non-regular watched inputs and exceeded budgets
are reported rather than silently watching an incomplete project. Additional
dependencies outside the local directory must be declared with `--watch`.

The programmatic interface exposes the same functionality:

```python
with Studio("lesson.py", "Example", autoreload=True,
            watch_paths=["assets/values.csv"]) as preview:
    print(preview.url)
    # Explicit reload is also available, even when autoreload is disabled.
    receipt = preview.reload()
    print(receipt["frame_index"], receipt["sha256"])
    print(preview.reload_status)
    input("Press Enter to close Studio")
```

`reload_status` is a detached dictionary containing a monotonically increasing
revision, completed-reload count, last error, and last native frame receipt.
Robot mode emits separate, nonterminal `studio-reload` success/failure events.
Manual and automatic reloads use the same authenticated native HTTP route as
the browser, including its operation serialization and rate limits. No proxy
or redirect receives the capability URL. Closing Studio stops and joins the
watcher before releasing the host; an in-flight worker operation remains bounded
by the configured request timeout.

Committed preview positions are explicitly opaque journal barriers, not a
serialization of arbitrary Python callbacks. This implementation does not
restore callback checkpoints, replay arbitrary Python effects, or export a
durable certified Python session. It also does not provide IPython `embed`,
audio playback, or the full live `InteractiveScene` editing lifecycle.

## Live input at the final frame

Pass `--interactive`, or `Studio(..., interactive=True)`, to enable live Python
callbacks. In the Studio UI, select the last timeline frame and enable **Scene
input**. Keyboard press/release, pointer motion, press/release, drag and wheel
events then reach the existing `Scene.on_*` methods and event dispatcher. A
plain `Scene` with mobject listeners works without a window toolkit:

```python
from manimlib import Scene, Square, BLUE, RED, RIGHT

class Clickable(Scene):
    def construct(self):
        self.box = Square(fill_color=BLUE, fill_opacity=1)
        self.box.add_mouse_press_listner(
            lambda box, event: box.set_color(RED)
        )
        self.box.add_mouse_drag_listner(self.drag_box)
        self.add(self.box)
        self.wait(0.5)

    def drag_box(self, box, event):
        box.shift(event["d_point"])
        return False  # Consume the drag; do not also pan the camera.

    def on_key_press(self, symbol, modifiers):
        super().on_key_press(symbol, modifiers)
        if symbol == ord("k"):
            self.play(self.box.animate.shift(RIGHT), run_time=0.25)
```

This is paused, final-frame editing, not a second animation loop. The complete
callback runs on the original worker/interpreter thread. Ordinary `play` and
`wait` still advance the same scene clock, execute updaters and synchronize
geometry; only their final state is rendered into the live view after successful
callback completion. Repeated input replaces that one frame rather than adding
unbounded history. Earlier captured frames remain immutable and read-only;
scrubbing back to them does not undo or reexecute the live scene. Return to the
last frame to resume input.

The native camera maps the browser's coordinates into the live world frame,
including translation, scale and rotation. The existing dispatcher handles
fixed-frame controls, hit tests, drag capture and listener order. Holding the
configured pan key (default `f`) or 3D-orbit key (default `d`) while moving the
pointer applies the existing native camera operations. Key release ends the
mode; no synthetic Window is installed and offline hover is unchanged. Events carry
the selected frame, an increasing input revision and the worker generation.
Stale events, foreign scenes and edits against historical frames are rejected
before any Python callback runs. Reload starts a fresh input revision and a new
worker generation, so queued input cannot mutate the replacement scene.

**A callback exception freezes further input until reload.** The last good PNG
and matching inspector snapshot remain available; intermediate `play`/`wait`
captures are not published. This is not rollback of arbitrary Python side
effects. A hung callback is bounded by the worker request timeout and terminates
that worker, without automatically executing the source again. The stable host
retains its displayed frame and accepts an explicit reload. Live commands are
opaque journal barriers, never serialized callback replay or certified state.

Select the final frame to enable **Step live frame** and **Run live**. A step
advances the existing rational scene clock by exactly `1/fps`, then runs the
ordinary Python and native updater passes once. It does not manufacture a
`Scene.wait`, consume an authored play index, or grow the captured timeline.
The final capture is replaced atomically; historical frames remain unchanged.

**Run live** paces one nominal frame at a time with at most one outstanding
request. Slow rendering runs slower than wall time rather than skipping scene
samples or accumulating catch-up work. Pause stops admission immediately; an
already accepted frame may finish. Scene input, history navigation, reload,
loss of focus, a hidden tab, or a disconnected stream stops the live clock.
Returning to a tab never restarts authored execution automatically.

Programmatic hosts can call `host.advance(frames=1)` after selecting the final
timeline frame. A call accepts at most one nominal second (and at most 240
frames), publishes its completed final frame, and returns its native PNG
receipt. This is a bounded update batch, not playback of its intermediate
frames. Browser and programmatic requests share `/api/advance` and the same
worker-generation, selected-frame and input-revision guards as live input.
Conflicts and timeouts are never automatically retried.

If an updater fails partway through a batch, the last good PNG and inspector
remain available and both live time and live input freeze until reload. The
underlying authored effects are **not rolled back**. Live advances are opaque
journal entries; they do not enable callback replay or certification. The
existing 4,096-command generation budget applies to clock steps, edits and
committed seeks together. On exhaustion, reload creates a fresh generation.

`InteractiveScene` supports configurable selection, axis grabs, center/corner
resize, Control-stretch, grouping, nudging, color picking, and bounded edit
undo/redo through the same native records. Rotated-camera and fixed-frame
selection use native camera coordinates. See [BN-19 — InteractiveScene editing](../behavior_notes/BN-19-interactive-editing.md)
for keys, gesture bounds, and the scene-state history scope.

This mode does not yet provide animated playback of the frames inside an
input callback,
IPython `embed`, audio playback, or the complete `InteractiveScene` windowed
resize/sweep lifecycle. Read-only captured Studio and offline output remain
separate modes with their existing behavior.

## Budgets and host access

The defaults are 640×360, 30 FPS, 7,200 captured frames, a 256 MiB encoded capture
budget, and a 120-second worker-request timeout. Set `--max_frames`,
`--max_bytes`, or `--timeout` explicitly for longer scenes. The encoded budget
accounts for PNGs, inspector documents and per-frame metadata; native inspector
traversal/field limits and a 16M-pixel frame ceiling apply independently. It is
not an operating-system memory quota on arbitrary authored Python. The maximum
accepted controls are 100,000 frames, 1 GiB encoded captures, and 900 seconds.

The native HTTP host binds only loopback and requires its random bearer
capability for the UI, frames, inspector and mutation routes. Its existing
origin, request-size, concurrency, and rate limits remain in force. The URL is
a capability: do not share it with untrusted users. There is no automatic
browser opening or public-network bind. Ctrl-C or `Studio.close()` stops the
host and reaps its disposable worker; context-manager use is recommended.

Preview is silent and needs no ffmpeg. Use `render_scene` / ordinary
`fmn-python lesson.py Example` for soundtrack or movie export.

**Process isolation is not a sandbox.** Scene code has the host user's file and
process permissions. Source/runtime digests bind this preview generation; they
do not constitute the complete C1–C10 certified Python input closure. Python
Studio does not make a certified-render claim.
