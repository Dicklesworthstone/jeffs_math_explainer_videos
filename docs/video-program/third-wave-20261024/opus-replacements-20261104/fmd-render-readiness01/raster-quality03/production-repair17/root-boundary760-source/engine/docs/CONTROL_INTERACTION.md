# Scene input and native-backed controls

The wheel installs `fmn_python.interaction` and `fmn_python.control_events`
on the existing public classes. Direct extension embeddings may install those
same adapters after the native module has initialized. Geometry, hit testing,
scalar values, text construction and camera projection remain native-backed.
No window, network endpoint, browser event transport or renderer is added.

```python
from manimlib import Scene, Square, Button, RIGHT

scene = Scene()
shape = Square().shift(2 * RIGHT)
button = Button(shape, lambda mob: mob.shift(RIGHT))
scene.add(button)
scene.on_mouse_press(shape.get_center(), 1, 0)
```

Button and MotionMobject register handlers on their original wrapped objects.
Checkbox and EnableDisableButton register their toggle handlers. A
LinearNumberSlider registers its handle for captured dragging; Textbox binds
box activation and key input; ControlPanel binds opener dragging and panel
scrolling. Registrations use the public misspelled `*_listner` APIs retained
from the Reference, including authored handler/registrar overrides. A failed
later registration attempts to remove only the registrations it just added.

## Authoring the control family

`LinearNumberSlider` owns a public `RoundedRectangle` bar, `Circle` handle and
invisible `Line` axis. `EnableDisableButton` owns a `Rectangle`; `Checkbox`
owns a `Rectangle` and a `VGroup` of two `Line` marks. These are real public
objects, not anonymous proxies cast to a different class. Their constructors
and methods use the existing native geometry and record machinery.

Named children exist before the cooperative `ControlMobject` initialization
hooks run. A subclass can inspect or decorate them in `init_data`,
`init_points`, `init_uniforms` or `init_colors`, including with a custom root
record schema. Checkbox construction calls the public `get_checkmark` or
`get_cross` factory first and retains the actual returned object. Factory and
initialization exceptions propagate rather than falling back to stock shapes.

```python
from manimlib import Checkbox, BLUE, YELLOW

class HighlightedCheckbox(Checkbox):
    def init_colors(self):
        super().init_colors()
        self.box.set_stroke(YELLOW, width=2)

checkbox = HighlightedCheckbox(
    rect_kwargs={"width": 0.8, "height": 0.5, "fill_opacity": 0},
    checkmark_kwargs={"stroke_color": BLUE, "stroke_width": 4},
)
checkbox.set_value(False)
```

Primitive option dictionaries accept the corresponding public shape's
constructor/style options. A standalone control's dictionary is its complete
primitive recipe; the color-bank adapter merges its partial overrides with
the public slider defaults so compact channel geometry is retained. Scalar
controls keep the existing midpoint-at-construction slider behavior and
white-at-construction default toggle box. A later `set_value` applies the live
state style or position. Finite, ordered ranges wholly outside the default
slider interval are supported; invalid ranges, steps and degenerate axes
refuse before tracker initialization.

Stock scalar controls use a serializable dynamic-marker updater. Copies,
deep copies and pickle round trips keep the typed child family and native
values. This does not make arbitrary authored callbacks pickleable. Checkbox
replacement marks fit the current box and inherit its fixed/unfixed camera
status, so a toggle cannot detach the mark from its box when the camera moves.

## Scene input semantics

Scene mouse callbacks consume world-space coordinates. Each event's point is
authoritative, so pressing or scrolling does not require a preceding hover.
Fixed-frame listeners receive coordinates projected by the existing
CameraFrame methods; world-space listeners retain world-space coordinates.
Pointer, key and drag-capture state are kept separately for each scene.
Ordinary registered objects must remain in that scene's drawable family;
arena allocation alone does not make a removed control interactive.

Listeners execute once, before the Scene/InteractiveScene default action.
An exact `False` return stops propagation and the default camera/selection
action. Existing user overrides still run normally; code an override executes
before calling `super()` is not undone. Listener order is snapshotted at input
entry, removals take effect before subsequent delivery, and additions begin
with the next event. Drag capture survives movement outside the object bounds
and ends on release or an input callback failure. Nested input restores the
outer scene context even when a callback raises.

**Compatibility boundary:** direct EventDispatcher calls keep their existing
standalone hover/capture and pointer-identity behavior. An explicit detached
EventListener added only to the global dispatcher remains a global interceptor,
including in Scene input. This is distinct from Mobject-local registrations
used by the installed controls. Bound listener targets are scene-scoped.

## Live control changes and limits

Checkbox marks are placed against the current box bounds. Enable/disable and
Textbox activation change live style rather than replacing positioned boxes.
New text is reseated at the live box. Active textboxes accept the existing
alphanumeric/space/tab/backspace key surface; navigation and modifier keysyms
are not inserted as Unicode letters, and unsupported shortcuts do not execute
scene actions. This is not clipboard, text-selection or IME support.

The subsequently installed color-bank adapter supplies independently
interactive ColorSliders channels and grouped panel controls; see
[Live color controls and composite panels](COLOR_CONTROL_PANELS.md).
These adapters do not change the existing copying/closure semantics, make
every camera or control mutation transactional, or complete the Python/native
Studio worker gateway. The existing Studio capability refusals remain in place.

`test_interaction_protocol.py` and `test_control_events_protocol.py` test
adapter logic against explicit storage fixtures. The installed-wheel gate
also runs `control_interaction.py` with actual native objects and a decoded
input-driven Y4M render, including a one/four-thread byte comparison. Protocol
passes and syntax checks alone do not establish that native acceptance passed.
