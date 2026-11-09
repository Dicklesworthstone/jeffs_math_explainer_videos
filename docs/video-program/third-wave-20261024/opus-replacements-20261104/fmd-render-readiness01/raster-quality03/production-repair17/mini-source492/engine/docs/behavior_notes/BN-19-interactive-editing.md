# BN-19 — InteractiveScene gestures without a window toolkit

**Status:** Draft.

**Contract.** The Python `InteractiveScene` class uses its existing native
selection `Group`, geometry operations, camera transforms, event dispatcher,
and `SceneState` history. The optional Python Studio worker supplies keyboard
state and per-event modifiers; no synthetic `Window`, geometry mirror, or
second renderer is installed. User subclasses retain their normal methods and
`super()` behavior. The same public editing methods work in an installed wheel.

Start a scene with `fmn-python studio lesson.py Example --interactive`, select
the final timeline frame, and enable **Scene input**. These are the default
bindings from the existing key configuration:

| Gesture | Behavior |
| --- | --- |
| Hold `s`, move, release `s` | Rectangle-overlap selection; a tap toggles the topmost hit. |
| Hold Shift+`s`, sweep, release `s` | Add objects along the pointer path without toggling prior hits away. |
| `u` | Clear selection. |
| Hold `g` / `h` / `v` / `z` and move | Free / x / y / z grab, preserving the initial pointer offset. |
| Hold `t` and move | Uniform resize around the selection center. |
| Start Shift+`t` and move | Resize around the opposite selection corner. |
| During `t`, hold Control and move | Independent x/y stretch, including axis reflection. |
| Arrow / Shift+Arrow | Small / 10x selection nudge. |
| Control/Command+`g` / +Shift+`g` | Group / ungroup the selection. |
| Backspace | Delete the selection. |
| Control/Command+`z` / +Shift+`z` | Undo / redo through the existing bounded native-backed scene history. |
| `c`, then click a swatch or object | Sample its native color into the selection. |

Browser motion and wheel events carry their actual modifier snapshot even
though the Reference's `on_mouse_motion` and `on_mouse_scroll` signatures do not
have a modifier argument. Delivery scopes that snapshot to the owning scene;
nested callbacks and exceptions restore the prior context. A real host window,
when explicitly supplied, retains precedence. A consumed event never also
runs the selection gesture. An editing drag does not dispatch a second motion
event or pan the camera.

The Python portal translates browser ASCII letter keys to the Reference's
unshifted integer symbols; Shift/Caps Lock text casing does not disable bindings
such as Shift+`t` or Control+Shift+`z`. Modifier flags remain separate. Native
Studio wire values, Unicode characters, and non-letter keys are unchanged.

## Correctness differences

The pinned Reference is `3b1b/manim` at
`6199a00d4c1b1127ebe45cb629c3f22538b10e13`. Its window-dependent gesture model is
retained, with the following deliberate corrections:

- Plain `t` initializes the resize reference; resizing does not require an
  accidental Shift modifier. Auto-repeat does not rebase an active grab.
- One changed grab/resize gesture creates one history checkpoint, not one per
  pointer sample or repeated key press. Empty or cancelled gestures are no-ops.
  Undo is not followed by a new `z`-grab snapshot. A new actual edit clears redo.
- Resizing uses cumulative scale relative to the initial gesture, correcting
  the prior applied scale before each native transform. Crossing an axis can
  reflect it. Magnitudes are clamped away from a collapsed pivot at `1e-6`,
  allowing the pointer to recover; non-finite values or magnitudes over `1e6`
  are refused before geometry or history changes. A zero initial radial
  reference is a no-op; a zero axis reference is left unchanged during stretch.
- Selection projects all eight native bounds corners, not only the minimum
  and maximum. Marquee and sweep hit tests therefore retain their extents under
  camera rotation. They remain bounding-box tests, not per-pixel picking.
- Fixed-frame objects are picked and transformed in their existing frame
  coordinates, not projected twice through a moved camera. A transform mixing
  world-space and fixed-frame members is explicitly refused; select one plane
  at a time. In a live worker that refusal follows the ordinary callback-failure
  freeze policy, so reload is required before further live execution.
- Palette swatches remain pickable even though the palette is intentionally
  not selectable. Opening/closing the palette does not add history; a color
  edit's history does not resurrect the transient palette.

The normal Studio generation/revision checks, bounded captures, worker timeout,
and failed-callback freeze policy still apply. Earlier captured frames remain
immutable: scene undo/redo does not rewrite the captured timeline. This is
scene-state history, **not rollback of arbitrary Python side effects**, durable
callback replay, or a certified Python session. Live-clock updates are separate
from edit history. Clipboard and IPython/embed capabilities, audio playback,
and the full external-window lifecycle are not supplied by this change.
