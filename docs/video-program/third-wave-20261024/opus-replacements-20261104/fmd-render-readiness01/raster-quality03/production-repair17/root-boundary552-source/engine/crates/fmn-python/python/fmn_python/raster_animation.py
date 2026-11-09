"""Image/material transitions on the shared Animation and native texture paths.

The host freezes input conversions once. Native resources own all color,
resampling and publication; Choreo owns timing, compositions and frame capture.
"""
from __future__ import annotations

from functools import wraps
import importlib
import math

from .image_authoring import _editing, _source, _pixels
from .movement import _install_lifecycle

_METADATA = ("image_path", "image_file", "dark_image_file", "texture_file", "num_textures")


def __getattr__(name):
    if name == "RasterTransition":
        return importlib.import_module("manimlib")._RasterAnimation
    raise AttributeError(name)


def install_raster_animation(native):
    g = vars(native)
    if g.get("_FMN_RASTER_ANIMATION_INSTALLED", False):
        return
    if not callable(g.get("_RasterTransition")):
        raise ImportError("native raster transition kernel is missing")
    Animation = g["Animation"]
    Image, Textured = g["ImageMobject"], g["TexturedSurface"]
    NativePlan = g["_RasterTransition"]

    class RasterTransition(Animation):
        """Crossfade one native image or material, without changing its geometry.

        Image inputs use the ordinary array/PNG/JPEG/path boundary and are
        frozen at construction. The starting material is captured at begin(),
        including for just-in-time Succession members. Color interpolation is
        premultiplied linear light; final endpoints keep their original bytes.
        """
        def __init__(self, mobject, image, dark_image=None, **kwargs):
            if not isinstance(mobject, (Image, Textured)):
                raise TypeError("RasterTransition requires an ImageMobject or textured surface/mesh")
            if isinstance(mobject, Image) and dark_image is not None:
                raise ValueError("image quads do not accept a dark-side texture")
            kwargs.setdefault("suspend_mobject_updating", True)
            super().__init__(mobject, **kwargs)
            self._raster_options = dict(kwargs)
            self._raster_plan = None
            with _editing(mobject):
                self._raster_end, path = _source(g, image)
                dark, dark_path = (None, None) if dark_image is None else _source(g, dark_image)
                if path is not None and dark_path is not None:
                    if g["_pathlib"].Path(path).resolve() == g["_pathlib"].Path(dark_path).resolve():
                        dark = None
                self._raster_dark = dark
                self._raster_destination = dict(image_path=path)
                if isinstance(mobject, Textured):
                    self._raster_destination.update(image_file=path, dark_image_file=dark_path,
                                                    num_textures=1 if dark is None else 2)
                    if isinstance(mobject, g["TexturedGeometry"]):
                        self._raster_destination["texture_file"] = path

        def interpolate_mobject(self, alpha):
            if self._raster_plan is None:
                raise RuntimeError("RasterTransition must begin before interpolation")
            # One material, not one interpolation per geometry-family member.
            # Use the same time-span/rate normalization as the base Animation.
            value = float(self.get_sub_alpha(self.time_spanned_alpha(alpha), 0, 1))
            if not math.isfinite(value):
                raise ValueError("raster transition rate function must return a finite alpha")
            self._raster_plan.apply(self.mobject, value)
            attrs = vars(self.mobject)
            if value <= 0:
                for key in _METADATA:
                    if key not in self._raster_metadata:
                        attrs.pop(key, None)
                attrs.update(self._raster_metadata)
            elif value >= 1:
                attrs.update(self._raster_destination)
            else:
                attrs.update({key: None for key in _METADATA if key in attrs and key != "num_textures"})
                if isinstance(self.mobject, Textured):
                    attrs["num_textures"] = 2 if self._raster_plan.has_dark else 1

        def get_all_mobjects_to_update(self):
            # No live dependency geometry: both materials are frozen values.
            # Ordinary updaters of the real mobject still follow Scene's policy.
            return ()

    RasterTransition.__module__ = __name__
    RasterTransition.__qualname__ = "RasterTransition"

    def prepare(animation):
        with _editing(animation.mobject):
            plan = NativePlan(animation.mobject, animation._raster_end, animation._raster_dark)
            attrs = vars(animation.mobject)
            metadata = {key: attrs[key] for key in _METADATA if key in attrs}
            animation._raster_metadata = metadata
            animation._raster_plan = plan

    _install_lifecycle(g, RasterTransition, prepare)
    begin, finish, abort = RasterTransition.begin, RasterTransition.finish, RasterTransition.abort

    def release_failed_begin(self):
        try:
            return begin(self)
        except BaseException:
            self._raster_plan = None
            raise

    def release_finish(self):
        try:
            return finish(self)
        finally:
            self._raster_plan = None

    def release_abort(self):
        try:
            return abort(self)
        finally:
            self._raster_plan = None

    RasterTransition.begin = release_failed_begin
    RasterTransition.finish, RasterTransition.abort = release_finish, release_abort
    g["_RasterAnimation"] = RasterTransition

    def image_override(self, image):
        return RasterTransition(self, image)

    def pixels_override(self, pixels):
        with _editing(self):
            value = g["_RasterImage"](*_pixels(g["_np"], pixels))
        return RasterTransition(self, value)

    def textures_override(self, image, dark_image=None):
        return RasterTransition(self, image, dark_image)

    def texture_pixels_override(self, pixels, dark_pixels=None):
        with _editing(self):
            light = g["_RasterImage"](*_pixels(g["_np"], pixels))
            dark = None if dark_pixels is None else g["_RasterImage"](*_pixels(g["_np"], dark_pixels))
        return RasterTransition(self, light, dark)

    Image.set_image._override_animate = image_override
    Image.set_pixel_array._override_animate = pixels_override
    Textured.set_textures._override_animate = textures_override
    Textured.set_pixel_array._override_animate = texture_pixels_override

    # The reference builder's override hook receives method arguments only.
    # Its stock build() returns that animation unchanged, losing anim_args.
    # Forward arguments solely for our own result; leave other overrides alone.
    Builder = g["_AnimationBuilder"]
    previous_build = Builder.build

    @wraps(previous_build)
    def build(self):
        result = previous_build(self)
        if isinstance(result, RasterTransition) and result is self.overridden_animation:
            if getattr(result, "_movement_active", False):
                raise RuntimeError("cannot rebuild an active raster animation")
            options = dict(result._raster_options)
            options.update(self.anim_args)
            Animation.__init__(result, result.mobject, **options)
        return result

    Builder.build = build
    g["_FMN_RASTER_ANIMATION_INSTALLED"] = True
