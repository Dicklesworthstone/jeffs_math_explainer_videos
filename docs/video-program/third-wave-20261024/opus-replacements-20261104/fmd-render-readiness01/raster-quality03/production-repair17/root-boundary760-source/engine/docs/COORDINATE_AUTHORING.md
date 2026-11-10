# Authoring coordinate systems

`Axes` and `ThreeDAxes` now construct through `CoordinateSystem`, `VGroup` and
public `create_axis` calls. `NumberPlane` and `ComplexPlane` use those same axes
and invoke `init_background_lines`, `get_lines` and
`get_lines_parallel_to_axis`. Returned axes and grid groups become the actual
scene objects; there is no independently reconstructed native chart replacing
the authored plan.

```python
from manimlib import NumberPlane, ORIGIN, YELLOW

class WidePlane(NumberPlane):
    def create_axis(self, range_terms, axis_config, length):
        axis = super().create_axis(range_terms, axis_config, length)
        axis.stretch(2, 0, about_point=ORIGIN)
        axis.set_color(YELLOW)
        return axis

plane = WidePlane(x_range=(-2, 2), y_range=(-1, 1), faded_line_ratio=2)
```

The group initialization hooks run once, before axes are created. Ranges are
available to `init_data` and `init_points`, but `x_axis` and `y_axis` are not yet
available there. Put axis-dependent work after `super().__init__`, or customize
`create_axis`. Custom root record fields, uniforms and hook-created children
remain attached. Subclass axis defaults participate in the recursive config
merge, and each factory receives independent configuration dictionaries.

The z-axis keeps the Reference/native convention: top-level `unit_size` affects
x and y; configure z through `axis_config` or `z_axis_config`, or set `depth`.
The published class identities and inheritance remain unchanged.

Plane grids follow live axis endpoints and public `n2p` mappings. Updating
`faded_line_ratio` changes subsequent `get_lines()` results, even after affine
chart edits. `background_line_style` and `faded_line_style` are public mutable
recipes. `get_lines()` returns detached groups without changing the scene;
`init_background_lines()` adds them using the Reference's additive behavior,
not an implicit replacement of previously added grids. Major/minor positions
retain the existing `arange` overshoot and index-based classification.

Factory-returned axes must be independent detached `NumberLine` families;
grid factories must return independent detached `VGroup` families. Existing
objects from another scene are refused before rotation or restyling. Invalid
ranges and oversized grids refuse by name. The aggregate grid limit is 65,536
lines; authored grid families and record counts are also bounded. Arbitrary
Python callbacks are not sandboxed or rolled back.

Geometry, ticks, copying, transforms and rendering continue through the native
owners. The new installed-portal suites exercise actual geometry and PNG frames,
including comparisons at one and four rendering threads and independent native
builder/endpoint controls. They do not by themselves close the complete library
subclassing, whole-portal, performance or cross-platform certification gates.
