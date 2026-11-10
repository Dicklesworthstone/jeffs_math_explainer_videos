# Live color controls and composite panels

The installed wheel applies `fmn_python.color_sliders` after the existing
control-event adapter, on the same public `ColorSliders` and `ControlPanel`
classes. Direct extension embeddings may call `install_color_sliders(native)`
after `install_control_events(native)`. Native scalar storage, geometry,
layout, hit testing, scene timing and rendering retain their existing owners.

```python
from manimlib import ColorSliders, ControlPanel, Group, Checkbox, Scene

scene = Scene()
colors = ColorSliders(default_rgb_value=128, default_a_value=0.5)
options = Group(Checkbox(True), Checkbox(False))
panel = ControlPanel(colors, options).open_panel()
scene.add(panel)

colors.set_value(255, 80, 20, 0.75)
colors.r_slider.set_value(120)
scene.update(0)  # Ordinary updater dispatch refreshes the swatch.
scene.play(colors.b_slider.animate.set_value(200), run_time=0.5)
```

## Four live channels, not generic vector shells

`r_slider`, `g_slider`, `b_slider`, and `a_slider` are actual
`LinearNumberSlider` objects. Each has its own native scalar tracker,
bar/handle/axis objects, and the standard captured-drag registration. Their
initial positions and styles use the existing Atlas bank layout and palettes.

`get_value()` reads those live channels: RGB values are divided by 255 and
alpha is returned directly. Editing one channel, including through its
existing animation surface, is therefore visible without calling the bank's
aggregate setter. The parent updater refreshes the swatch after the channel
updates in the ordinary child-first updater phase. Aggregate `set_value`
refreshes it immediately and retains its existing `None` return value.

Atlas still validates and normalizes the aggregate request before any live
channel setter runs. This preserves the bank's clamping and explicit shared-
step snapping, including alpha's smaller default step when a shared RGB step
exceeds its range. All channel validators run before the first live setter.
Subsequent arbitrary authored setters that mutate and then raise are not
transactionally rolled back.

Aggregate edits retain the live axis geometry, positioned handle objects,
background, swatch, group roots and input registrations. They do not install
the native planner's temporary unpositioned geometry into a moved, scaled or
rotated control. Aggregate normalization still constructs temporary native
geometry; it is not a constant-allocation or benchmarked fast path.

`get_background()` now returns a fresh native checkerboard at the current
swatch center. Its public override is consulted during construction, as are
`get_picked_color()` and `get_picked_opacity()` during painting. Constructor
configuration determines the native planner; changing arbitrary configuration
attributes later is not a general live-reconfiguration API.

The swatch updater uses its receiver rather than closing over the original
bank, so copied banks paint their own remapped channels. This does **not**
change the existing event-listener/callback copy semantics or automatically
register a copied bank as a second independent input controller.

## Composite panel content

Panels accept `ControlMobject` instances, `ColorSliders`, and `Group` objects
containing either kind of control. Such a group may also contain labels and
decorations. Bare shapes, empty groups, and groups containing only decorative
objects remain non-controls and retain the existing `TypeError` diagnostic.

The existing scalar-only constructor path is unchanged. Composite construction
uses the same native extent-based layout helper, grafts the original objects,
and registers the existing panel scroll/opener drag handlers. `add_controls`
accepts the same content and retains the existing content root; the existing
remove/open/close and layout methods remain in use. Invalid content is checked
before attachment. Authored layout hooks can still mutate and fail; there is
no rollback promise for arbitrary Python code.

## Deliberate behavior correction

The Reference and the old portal leave slider handles at the axis midpoint
during construction even when their values are elsewhere. Within a color
bank, initial handles now show their actual normalized values. Code that
relied on that inconsistent initial appearance will see different geometry.
Standalone `LinearNumberSlider` constructor behavior is not changed here.

## Verification and remaining boundaries

`test_color_sliders_protocol.py` and `test_color_panel_protocol.py` exercise
the adapter against explicit geometry, tracker and layout fixtures. They do
not prove native geometry or output. The installed-wheel gate also registers
`tests/color_sliders.py`, requiring native channel values, captured input,
fixed-frame camera mapping, animation, panel composition, legacy refusals,
and a decoded six-frame Y4M render compared across one and four threads.
Those native assertions must actually execute before claiming integration
verified; the local attempt stopped at the missing `manimlib` extension.

This work does not add a browser/worker transport, complete the Python Studio
gateway, certify output, or qualify performance. Existing Studio capability
refusals and the independent parity audit remain in force.
