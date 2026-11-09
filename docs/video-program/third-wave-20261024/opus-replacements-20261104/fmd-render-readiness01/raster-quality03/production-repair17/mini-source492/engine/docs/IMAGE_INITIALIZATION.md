# Native-backed image initialization

`ImageMobject` now uses `Mobject`'s data, point and uniform initialization hooks,
then calls `init_colors`. The exported class and inheritance stay unchanged.
The constructor decodes and owns the input pixels before any of these hooks.
`from_pixel_array` and `from_bytes` follow the same subclass constructor.

```python
from manimlib import ImageMobject, RIGHT

class CroppedImage(ImageMobject):
    data_dtype = ImageMobject.data_dtype + [("weight", 1)]

    def init_data(self):
        super().init_data()
        self.data["weight"][:] = 1
        self.weights = self.data["weight"]

    def init_points(self):
        super().init_points()
        self.data["im_coords"][:, 0] *= 0.5
        self.shift(RIGHT)

    def init_colors(self):
        self.set_opacity(0.75)
```

The ordinary data initializer obtains the six-vertex quad and texture-coordinate
order from Atlas, normalizes its initial size, and writes its columns through
the existing record interface. It does not install a replacement native tree.
Custom data columns and hook-created children remain the object's own. The
native initializer then admits this exact table as an image primitive.

`height`, `opacity`, `image_path`, pixel dimensions and an owned
`get_pixel_array()` result are available during construction. Input arrays are
already frozen when hooks run. After construction, pixel readback and dimensions
follow the current native image resource, including after replacement, copying,
restore and pickle. The temporary constructor resource is not a copied or
serialized Python attribute.

Default `init_points` applies the configured height and current aspect ratio.
A subclass can call `super()` and then transform the points, alter texture
coordinates or opacity, or supply its own six-row table instead. When bypassing
`super().init_data()`, the subclass owns initialization of all point, im_coords
and opacity columns; the material is attached by final admission. Native
`point_to_rgb` during a hook requires that ordinary data initialization has
already attached the material. Array readback and dimension properties also
work before that attachment.

Constructor flags are applied in the default `init_uniforms`; overrides can
change them afterward. The existing image `set_color` remains a no-op rather
than introducing an undocumented tint model. Use opacity or pixel/material
operations for actual image changes.

## Storage and failures

Native admission validates detached ownership, the declared record schema,
exactly six records and finite rendering columns/world placement. It changes
only image metadata: original object-space records and placement are retained.
Generic `become` deliberately detaches storage, so its metadata transfer is
performed against private scratch storage and the exact original buffer is
restored before any proxy can observe it. No Python callback runs within that
borrowed-stage operation. This preserves constructor-created same-size views
without altering the general `become`, copy, or resize contracts.

Malformed input fails before hooks. A hook exception propagates once and stops
the later phases. Final admission follows the color hook, so invalid final rows
or an attempted Scene adoption cannot be returned as a successfully initialized
image. Arbitrary authored effects are not rolled back. Reentrant image-content
replacement on the same object during construction is refused; ordinary
post-construction `set_image` and `set_pixel_array` remain available.

## Validation

`raster_initialization.py` exercises the native admission operation;
`raster_lifecycle.py` exercises the public subclass constructor, actual pixels,
custom records, callbacks, copies and animated output compared with the separate
Atlas builder at one/four threads. Both run in the embedded PyO3 harness and the
installed-portal script. They require a rebuilt native extension containing
`_initialize_raster_image`; an older extension is rejected rather than silently
falling back to the constructor that bypassed these hooks.
