"""In-memory images and live pixels over immutable native raster resources.

Array conversion is an explicit input boundary, not an image codec or renderer.
PNG/JPEG decoding, texture storage, revisions, copies and snapshots stay native.
Input arrays and returned arrays never become writable aliases of live pixels.
"""
from __future__ import annotations

from contextlib import contextmanager
import math
import os

_MAX_PIXELS = 16_777_216
_MAX_ENCODED_BYTES = 64 * 1024 * 1024
_BUSY = "_fmn_image_authoring_busy"


def _pixels(np, pixels):
    array = np.asarray(pixels)
    if array.dtype != np.uint8:
        raise TypeError("pixels must be uint8 samples; normalize other data explicitly")
    if array.ndim == 2:
        array = array[..., None]
    if array.ndim != 3 or array.shape[2] not in (1, 2, 3, 4):
        raise ValueError("pixels must be (H, W), (H, W, 1/2/3/4) grayscale/gray-alpha/RGB/RGBA")
    height, width, channels = array.shape
    if width < 1 or height < 1 or width * height > _MAX_PIXELS:
        raise ValueError("pixel dimensions are empty or exceed the 16M-pixel budget")
    # Copy before calling native code: strided/reversed/transposed arrays and
    # shared caller storage all have the same top-row-first immutable meaning.
    if channels == 4:
        rgba = array.tobytes(order="C")
    else:
        output = np.empty((height, width, 4), dtype=np.uint8)
        output[..., :3] = array[..., :1] if channels < 3 else array
        output[..., 3] = array[..., 1] if channels == 2 else 255
        rgba = output.tobytes(order="C")
    return width, height, rgba


def _encoded(value):
    size = value.nbytes if isinstance(value, memoryview) else len(value)
    if size > _MAX_ENCODED_BYTES:
        raise ValueError("raster input exceeds the 64 MiB encoded budget")
    return bytes(value)


def _source(g, value):
    """Prepare one raster and informational path without touching a target."""
    if isinstance(value, g["_RasterImage"]):
        return value, None
    if isinstance(value, (str, os.PathLike)):
        # Resolve the filesystem protocol once, before the legacy asset resolver
        # can stringify a PathLike and lose either its path or authored error.
        path = g["_resolve_raster_image_path"](os.fsdecode(os.fspath(value)))
        with path.open("rb") as stream:
            encoded = stream.read(_MAX_ENCODED_BYTES + 1)
        return g["_RasterImage"].decode(_encoded(encoded)), str(path)
    if isinstance(value, (bytes, bytearray, memoryview)):
        return g["_RasterImage"].decode(_encoded(value)), None
    return g["_RasterImage"](*_pixels(g["_np"], value)), None


def _method(cls, name, function):
    function.__name__ = name
    function.__qualname__ = cls.__qualname__ + "." + name
    function.__module__ = cls.__module__
    setattr(cls, name, function)


@contextmanager
def _editing(target):
    attrs = vars(target)
    if attrs.get(_BUSY, False):
        raise RuntimeError("image authoring cannot reenter the same mobject")
    attrs[_BUSY] = True
    try:
        yield
    finally:
        attrs.pop(_BUSY, None)


def install_image_authoring(native):
    """Keep the original ImageMobject class and qualified import identities."""
    g = vars(native)
    if g.get("_FMN_IMAGE_AUTHORING_INSTALLED", False):
        return
    Image, np = g["ImageMobject"], g["_np"]
    initializer = g.get("_initialize_raster_image")
    if not callable(initializer):
        raise ImportError("native image initialization seam is missing")
    # Invocation state is not a copied/pickled image attribute. No proxy is
    # retained here, and every constructor path releases its prepared resource.
    constructing = {}

    def image_resource(self):
        pending = constructing.get(id(self))
        return pending if pending is not None else g["_read_raster_image"](self)

    def dimensions(self):
        pending = constructing.get(id(self))
        return pending.size if pending is not None else self._image_dimensions()

    def initialize(self, filename, height=4.0, **kwargs):
        if self._is_bound():
            raise RuntimeError("an image constructor requires a detached target; use set_image()")
        with _editing(self):
            opacity = float(kwargs.pop("opacity", 1.0))
            z_index = int(kwargs.pop("z_index", 0))
            fixed = bool(kwargs.pop("is_fixed_in_frame", False))
            depth = bool(kwargs.pop("depth_test", False))
            color = kwargs.pop("color", None)
            if kwargs:
                raise TypeError("ImageMobject() got unexpected keyword arguments: " + ", ".join(sorted(kwargs)))
            height = float(height)
            if (not math.isfinite(height) or height <= 0 or height > np.finfo(np.float32).max
                    or not math.isfinite(opacity) or abs(opacity) > np.finfo(np.float32).max):
                raise ValueError("image height/opacity must be finite and f32-representable; height must be positive")
            if not -(1 << 31) <= z_index < (1 << 31):
                raise ValueError("image z_index must fit a signed 32-bit integer")
            raster, path = _source(g, filename)
            width, pixel_height = raster.size
            if height * width / pixel_height > np.finfo(np.float32).max:
                raise ValueError("image aspect ratio produces an unrepresentable scene width")
            # Every conversion and decode succeeds before native initialization.
            # Mobject owns live state and the one native data/points/uniforms
            # hook sequence. Pass seeds through its ordinary constructor; an
            # inherited _install_live_state must not reset the requested alpha.
            constructing[id(self)] = raster
            try:
                # ImageMobject's own configuration is set here; Mobject takes
                # only its Reference keywords (fm-5wq.20).
                self.height, self.image_path = height, path
                self._fmn_image_z_index, self._fmn_image_fixed = z_index, fixed
                self._fmn_image_depth = depth
                super(Image, self).__init__(
                    opacity=opacity, color=g["_WHITE"] if color is None else color,
                )
                # An authored init_data can supply all six records instead of
                # calling super(). Admit that actual table, never replace it
                # with the canonical quad after the authored hooks have run.
                self.init_colors()
                initializer(self, raster)
            finally:
                constructing.pop(id(self), None)

    def from_pixel_array(cls, pixels, height=4.0, **kwargs):
        """Construct from uint8 grayscale, gray-alpha, RGB or RGBA samples."""
        raster = g["_RasterImage"](*_pixels(np, pixels))
        return cls(raster, height=height, **kwargs)

    def from_bytes(cls, encoded, height=4.0, **kwargs):
        """Construct from bounded native PNG/JPEG bytes, without filesystem I/O."""
        if not isinstance(encoded, (bytes, bytearray, memoryview)):
            raise TypeError("encoded image must be PNG/JPEG bytes")
        return cls(g["_RasterImage"].decode(_encoded(encoded)), height=height, **kwargs)

    def set_image(self, image):
        """Replace raster content, retaining geometry, UVs, opacity and identity.

        Pixel dimensions may change without resizing or reorienting the scene
        quad. Existing copies, camera captures and checkpoints retain old pixels.
        A failed preparation does not replace the current native resource.
        """
        with _editing(self):
            raster, path = _source(g, image)
            g["_replace_raster_image"](self, raster)
            width, height = raster.size
            vars(self).update(image_path=path, pixel_width=width, pixel_height=height)
        return self

    def set_pixel_array(self, pixels):
        """Publish an owned uint8 image; the supplied array is never retained."""
        with _editing(self):
            raster = g["_RasterImage"](*_pixels(np, pixels))
            g["_replace_raster_image"](self, raster)
            width, height = raster.size
            vars(self).update(image_path=None, pixel_width=width, pixel_height=height)
        return self

    def init_data(self):
        super(Image, self).init_data()
        raster = image_resource(self)
        candidate = g["_native_shell_factory"]()
        specs = g["_build_raster_image"](
            candidate, raster, g["_native_shell_factory"], 2.0, float(self.opacity), 0,
        )
        if specs:
            raise RuntimeError("a native image quad unexpectedly returned children")
        # Reference init_data starts from a 2x2 quad. Its vertices and UV order
        # still come from Atlas; native positional math normalizes its aspect.
        candidate.set_width(2.0, stretch=True)
        values = candidate.data.copy()
        self.resize_points(len(values))
        for key in ("point", "im_coords", "opacity"):
            self.data[key][:] = values[key]
        if id(self) in constructing:
            initializer(self, raster)
        return self

    def init_points(self):
        """Reapply configured height and the current raster's aspect ratio."""
        width, pixel_height = dimensions(self)
        height = float(self.height)
        scene_width = height * width / pixel_height
        if (not math.isfinite(height) or height <= 0
                or not math.isfinite(scene_width)
                or max(height, scene_width) > np.finfo(np.float32).max):
            raise ValueError("image dimensions must produce positive finite f32-representable extents")
        self.set_height(height)
        self.set_width(scene_width, stretch=True)
        return self

    def init_uniforms(self):
        super(Image, self).init_uniforms()
        self.set_z_index(getattr(self, "_fmn_image_z_index", 0))
        if getattr(self, "_fmn_image_fixed", False):
            self.fix_in_frame()
        if getattr(self, "_fmn_image_depth", False):
            self.apply_depth_test()
        return self

    def get_pixel_array(self):
        """Return a writable, detached RGBA8 copy of the native image pixels."""
        raster = image_resource(self)
        width, height = raster.size
        return np.frombuffer(raster.pixels(), dtype=np.uint8).reshape(height, width, 4).copy()

    for name, function in (("__init__", initialize), ("set_image", set_image),
                           ("set_pixel_array", set_pixel_array), ("get_pixel_array", get_pixel_array),
                           ("init_data", init_data), ("init_points", init_points),
                           ("init_uniforms", init_uniforms)):
        _method(Image, name, function)
    for name, function in (("from_bytes", from_bytes), ("from_pixel_array", from_pixel_array)):
        function.__name__ = name
        function.__qualname__ = Image.__qualname__ + "." + name
        function.__module__ = Image.__module__
        setattr(Image, name, classmethod(function))
    # The resource, not a cached Python attribute, owns its pixel shape.
    # Native become/restore and checkpoint replay can replace a resource without
    # running these setters. Reading dimensions must follow that actual image.
    # Retained dictionary entries from earlier pickles are harmless projections.
    def pixel_width(self):
        return dimensions(self)[0]

    def pixel_height(self):
        return dimensions(self)[1]

    Image.pixel_width = property(pixel_width, doc="Native raster width in pixels (read-only).")
    Image.pixel_height = property(pixel_height, doc="Native raster height in pixels (read-only).")

    def unsupported_raster_animation(self, *args, **kwargs):
        del self, args, kwargs
        raise g["_CapabilityError"](
            "raster contents are not interpolated by .animate; "
            "call set_image/set_pixel_array in an ordinary scene updater instead"
        )

    # These new data-publication methods are not record-field transformations.
    # Without an override the generic builder mutates a target's resource, then
    # silently drops it while interpolating only records/uniforms. Refuse before
    # evaluating an authored array/path conversion rather than claim success.
    set_image._override_animate = unsupported_raster_animation
    set_pixel_array._override_animate = unsupported_raster_animation

    original_identical = g["Mobject"].looks_identical

    def looks_identical(self, mobject):
        # Record/uniform equality alone says nothing about two images. Preserve
        # the established geometric tolerance, then compare complete native
        # materials (dimensions, transfer, sampler, light and dark pixels).
        if not original_identical(self, mobject):
            return False
        left = self.family_members_with_points()
        right = mobject.family_members_with_points()
        return len(left) == len(right) and all(
            g["_raster_images_equal"](a, b) for a, b in zip(left, right)
        )

    _method(g["Mobject"], "looks_identical", looks_identical)

    def set_textures(self, image, dark_image=None):
        """Prepare and replace a whole material atomically; None removes the pair.

        Both images may be arrays, bounded PNG/JPEG bytes, or local paths.
        No geometry, UV, opacity, lighting, child or updater state is rebuilt.
        """
        with _editing(self):
            light, path = _source(g, image)
            dark, dark_path = (None, path) if dark_image is None else _source(g, dark_image)
            if path is not None and dark_path is not None:
                if g["_pathlib"].Path(path).resolve() == g["_pathlib"].Path(dark_path).resolve():
                    dark = None
            g["_replace_raster_image"](self, light, dark)
            vars(self).update(image_file=path, image_path=path, dark_image_file=dark_path,
                              num_textures=1 if dark is None else 2)
            if isinstance(self, g["TexturedGeometry"]):
                self.texture_file = path
        return self

    def set_texture_pixels(self, pixels, dark_pixels=None):
        """Replace a light/dark material from uint8 arrays without encoding files."""
        with _editing(self):
            light = g["_RasterImage"](*_pixels(np, pixels))
            dark = None if dark_pixels is None else g["_RasterImage"](*_pixels(np, dark_pixels))
            g["_replace_raster_image"](self, light, dark)
            vars(self).update(image_file=None, image_path=None, dark_image_file=None,
                              num_textures=1 if dark is None else 2)
            if isinstance(self, g["TexturedGeometry"]):
                self.texture_file = None
        return self

    def texture_pixels(self, *, dark=False):
        """Return an owned copy of the selected side; absent dark sides refuse."""
        if type(dark) is not bool:
            raise TypeError("dark must be bool")
        raster = g["_read_raster_image"](self, dark)
        width, height = raster.size
        return np.frombuffer(raster.pixels(), dtype=np.uint8).reshape(height, width, 4).copy()

    set_textures._override_animate = unsupported_raster_animation
    set_texture_pixels._override_animate = unsupported_raster_animation

    for name, function in (("set_textures", set_textures), ("set_pixel_array", set_texture_pixels),
                           ("get_pixel_array", texture_pixels)):
        _method(g["TexturedSurface"], name, function)
    g["_FMN_IMAGE_AUTHORING_INSTALLED"] = True
