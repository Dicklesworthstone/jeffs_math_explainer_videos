"""Live vector-field sampling over Atlas's existing native arrow builder.

Python owns callable dispatch and sample tables. Atlas owns arrow geometry,
length compression and tapered widths; Marionette owns records and resizing.
No field integration, geometry kernel or second animation clock lives here.
"""
from __future__ import annotations

from contextlib import contextmanager
import itertools
import math
import operator
from typing import Any

from .invocation import InvocationGuard

_MAX_SAMPLES = 65_536
_FIELD_UPDATES = InvocationGuard()
_STYLE_TARGETS = {}
_MISSING = object()


def _rows(np, values, context, *, columns=None, count=None):
    if not isinstance(values, (np.ndarray, list, tuple)):
        values = list(itertools.islice(iter(values), _MAX_SAMPLES + 1))
    if len(values) > _MAX_SAMPLES:
        raise ValueError(context + " exceeds the 65536-point resource budget")
    array = np.asarray(values)
    if array.dtype.kind == "c":
        raise TypeError(context + " requires real coordinates")
    if array.size == 0 and array.ndim == 1:
        array = np.empty((0, 3 if columns is None else columns))
    if array.ndim != 2 or not 1 <= array.shape[1] <= 3:
        raise ValueError(context + " must have shape (N, 1..3)")
    if columns is not None and array.shape[1] != columns:
        raise ValueError(context + f" must have {columns} columns")
    if count is not None and len(array) != count:
        raise ValueError(context + " must return one vector per sample")
    array = np.array(array, dtype=float, copy=True)
    if not np.isfinite(array).all():
        raise ValueError(context + " must be finite")
    return array


def _representable(np, values, context):
    if not np.isfinite(values).all() or np.any(np.abs(values) > np.finfo(np.float32).max):
        raise ValueError(context + " must be finite and f32-representable")
    return values


def _finite_nonnegative(value, name):
    value = float(value)
    if not math.isfinite(value) or value < 0:
        raise ValueError(name + " must be finite and nonnegative")
    return value


def _width_profile(np, values, count):
    """Freeze an authored taper before geometry/color callbacks can change it."""
    array = np.asarray(values)
    if array.ndim != 1 or len(array) != count:
        raise ValueError("VectorField base stroke profile must have one scalar per native point")
    if array.dtype.kind == "c":
        raise TypeError("VectorField base stroke profile must be real")
    array = np.array(array, dtype=float, copy=True)
    _representable(np, array, "VectorField base stroke profile")
    if np.any(array < 0):
        raise ValueError("VectorField base stroke profile must be nonnegative")
    return array


def _profile_widths(np, widths, profile, ratio):
    """Apply authored taper to Atlas's per-arrow attenuation, not a new kernel.

    Shaft lane zero contains stroke_width * native_short_vector_scale. Repeat
    that value across the arrow's eight lanes before applying the profile.
    Unchanged profiles retain the exact native bytes, including head rounding.
    """
    count = len(profile)
    if widths.shape != (count, 1):
        raise RuntimeError("native VectorField returned an inconsistent width count")
    stock = np.ones(count)
    stock[4::8], stock[5::8] = ratio, ratio / 2
    stock[6::8], stock[7::8] = 0, 0
    if np.array_equal(profile, stock):
        return widths
    with np.errstate(over="ignore", invalid="ignore"):
        scaled = np.repeat(widths[0::8, 0].astype(float), 8)[:count] * profile
    _representable(np, scaled, "VectorField profiled stroke widths")
    widths[:, 0] = scaled
    return widths


def _bind(cls, name, function):
    function.__name__ = name
    function.__qualname__ = cls.__qualname__ + "." + name
    function.__module__ = cls.__module__
    setattr(cls, name, function)


@contextmanager
def _updating(mob):
    with _FIELD_UPDATES.hold(mob, message="cannot reenter a vector-field update"):
        attrs = vars(mob)
        previous = {name: attrs.get(name, _MISSING) for name in (
            "sample_points", "base_stroke_width_array",
        )}
        try:
            yield
        except BaseException:
            # Roll back only projections we prepared. Authored callbacks retain
            # their ordinary external side effects; no scene rewind is claimed.
            for name, value in previous.items():
                if value is _MISSING:
                    attrs.pop(name, None)
                else:
                    attrs[name] = value
            raise
        finally:
            # The scratch paint destination is invocation state, not an
            # attribute for copy/deepcopy/SceneState to inherit.
            _STYLE_TARGETS.pop(id(mob), None)


def install_vector_fields(native: Any) -> None:
    """Keep published classes and native construction; complete live updates."""
    g = vars(native)
    if g.get("_FMN_VECTOR_FIELDS_INSTALLED", False):
        return
    Field, VMobject, np = g["VectorField"], g["VMobject"], g["_np"]

    def evaluate(self):
        if not callable(self.func):
            raise TypeError("VectorField func must be callable")
        coordinates = _rows(np, self.sample_coords, "VectorField sample_coords")
        return _rows(np, self.func(coordinates), "VectorField callback output", count=len(coordinates))

    def update_sample_points(self):
        coordinates = _rows(np, self.sample_coords, "VectorField sample_coords")
        if len(coordinates) < 2:
            raise ValueError("VectorField needs at least two sample points")
        if len(coordinates):
            points = _rows(np, self.coordinate_system.c2p(*coordinates.T),
                           "VectorField c2p output", columns=3, count=len(coordinates))
        else:
            points = np.empty((0, 3))
        self.sample_points = _representable(np, points, "VectorField sample points")

    def geometry_inputs(self, outputs):
        outputs = _rows(np, outputs, "VectorField callback output", count=len(self.sample_points))
        dimension = operator.index(getattr(self.coordinate_system, "dimension", 2))
        if not 1 <= dimension <= 3:
            raise ValueError("VectorField coordinate-system dimension must be in 1..3")
        origin = np.asarray(self.coordinate_system.c2p(*([0.0] * dimension)), dtype=float)
        if origin.shape != (3,) or not np.isfinite(origin).all():
            raise ValueError("VectorField origin must be a finite 3-vector")
        if len(outputs):
            points = _rows(np, self.coordinate_system.c2p(*outputs.T),
                           "VectorField vector c2p output", columns=3, count=len(outputs))
        else:
            points = np.empty((0, 3))
        with np.errstate(over="ignore", invalid="ignore"):
            vectors = points - origin
            norms = np.linalg.norm(outputs, axis=1)
        _representable(np, vectors, "VectorField scene vectors")
        if not np.isfinite(norms).all():
            raise ValueError("VectorField magnitudes must be finite")
        return vectors, norms

    def base_widths(self, n_sample_points):
        count = operator.index(n_sample_points)
        if not 0 <= count <= _MAX_SAMPLES:
            raise ValueError("VectorField sample count must be in 0..65536")
        ratio = _finite_nonnegative(self.tip_width_ratio, "tip_width_ratio")
        values = np.ones(max(0, 8 * count - 1))
        values[4::8], values[5::8] = ratio, ratio / 2
        values[6::8], values[7::8] = 0, 0
        self.base_stroke_width_array = values

    def set_sample_coords(self, sample_coords):
        if _FIELD_UPDATES.busy(self):
            raise RuntimeError("cannot resample during a vector-field update")
        coordinates = _rows(np, sample_coords, "VectorField sample_coords")
        if len(coordinates) < 2:
            raise ValueError("VectorField needs at least two sample points")
        self.sample_coords = coordinates
        return self

    def apply_callback_style(self, outputs):
        target = _STYLE_TARGETS.get(id(self), self)
        outputs = _rows(np, outputs, "VectorField callback output")
        with np.errstate(over="ignore", invalid="ignore"):
            norms = np.repeat(np.linalg.norm(outputs, axis=1), 8)[:max(0, 8 * len(outputs) - 1)]
        if not np.isfinite(norms).all():
            raise ValueError("VectorField magnitudes must be finite")
        rgb, opacity = None, None
        if self.color_map is not None:
            if not callable(self.color_map):
                raise TypeError("VectorField color_map must be callable")
            low, high = (float(value) for value in self.magnitude_range)
            if not math.isfinite(low) or not math.isfinite(high) or high < low:
                raise ValueError("VectorField magnitude_range must be finite and ordered")
            # Match Atlas's defined constant-range policy, including an all-zero
            # field. Never divide by zero and overwrite valid native color with NaN.
            alphas = np.zeros(len(norms)) if high == low else (norms - low) / (high - low)
            rgba = np.asarray(self.color_map(alphas))
            if rgba.dtype.kind == "c":
                raise TypeError("VectorField colors must be real")
            rgba = np.asarray(rgba, dtype=float)
            if rgba.ndim != 2 or rgba.shape[0] != len(norms) or rgba.shape[1] not in (3, 4):
                raise ValueError("VectorField color_map must return one RGB or RGBA row per point")
            rgb = _representable(np, rgba[:, :3], "VectorField colors").copy()
        if self.norm_to_opacity_func is not None:
            if not callable(self.norm_to_opacity_func):
                raise TypeError("VectorField norm_to_opacity_func must be callable")
            opacity = np.asarray(self.norm_to_opacity_func(norms.copy()))
            if opacity.dtype.kind == "c":
                raise TypeError("VectorField opacities must be real")
            opacity = np.asarray(opacity, dtype=float)
            if opacity.shape == (len(norms), 1):
                opacity = opacity[:, 0]
            if opacity.ndim == 0:
                opacity = np.full(len(norms), float(opacity))
            if opacity.shape != (len(norms),):
                raise ValueError("VectorField opacity callback must return a scalar or one value per point")
            opacity = _representable(np, opacity, "VectorField opacities").copy()
        # Validate BOTH callbacks before either style column changes.
        if rgb is not None:
            target.data["stroke_rgba"][:, :3] = rgb
        if opacity is not None:
            target.data["stroke_rgba"][:, 3] = opacity

    def update_from_samples(self, prepared_outputs=None):
        with _updating(self):
            self.get_points()
            before = self.data.copy()
            family = tuple(id(member) for member in self.get_family())
            owner, bound = vars(self).get("_scene"), self._is_bound()
            pointlike = tuple(self.pointlike_data_keys)
            coordinates = _rows(np, self.sample_coords, "VectorField sample_coords")
            chart = self.coordinate_system
            self.update_sample_points()
            outputs = self._evaluate_outputs() if prepared_outputs is None else prepared_outputs
            outputs = _rows(np, outputs, "VectorField callback output", count=len(self.sample_points))
            self.init_base_stroke_width_array(len(outputs))
            count = max(0, 8 * len(outputs) - 1)
            profile = _width_profile(np, self.base_stroke_width_array, count)
            ratio = _finite_nonnegative(self.tip_width_ratio, "tip_width_ratio")
            scratch = VMobject.__new__(VMobject)
            g["_install_live_state"](scratch)
            specs = self._build_geometry(g["_native_shell_factory"], outputs, target=scratch)
            if specs:
                raise RuntimeError("native VectorField unexpectedly returned children")
            points = np.array(scratch.get_points(), copy=True)
            if points.shape != (count, 3):
                raise RuntimeError("native VectorField returned an inconsistent point count")
            _representable(np, points, "VectorField native points")
            widths = np.array(scratch.data["stroke_width"], copy=True)
            _representable(np, widths, "VectorField native widths")
            widths = _profile_widths(np, widths, profile, ratio)
            # Keep live paint (including user edits) when no color/opacity
            # callback owns that channel. Scratch construction defaults are not
            # allowed to recolor a field or wipe other live record lanes.
            paint = np.array(self._style_data()["stroke_rgba"], copy=True)
            paint = g["resize_preserving_order"](paint, count)
            scratch.data["stroke_rgba"][:] = paint
            _STYLE_TARGETS[id(self)] = scratch
            self._apply_callback_style(outputs)
            paint = np.array(scratch.data["stroke_rgba"], copy=True)
            _representable(np, paint, "VectorField stroke paint")
            self.get_points()  # Observe retained placement edits from authored callbacks.
            if (vars(self).get("_scene") is not owner or self._is_bound() != bound
                    or tuple(id(member) for member in self.get_family()) != family
                    or tuple(self.pointlike_data_keys) != pointlike
                    or self.coordinate_system is not chart
                    or not np.array_equal(self.sample_coords, coordinates)
                    or self.tip_width_ratio != ratio
                    or not np.array_equal(self.base_stroke_width_array, profile)
                    or not np.array_equal(self.data, before)):
                raise RuntimeError("vector field changed during sampling; candidate was not published")
            # Only now touch live records. The ordinary public set_points
            # protocol preserves live style and respects subclass hooks and
            # Marionette's view/resize rules.
            self.set_points(points)
            self.data["stroke_width"][:] = widths
            self.data["stroke_rgba"][:] = paint
        return self

    def update_vectors(self):
        return update_from_samples(self)

    g["_fmn_update_field_from_samples"] = update_from_samples

    for name, function in (
        ("_evaluate_outputs", evaluate), ("_geometry_inputs", geometry_inputs),
        ("update_sample_points", update_sample_points),
        ("init_base_stroke_width_array", base_widths), ("set_sample_coords", set_sample_coords),
        ("_apply_callback_style", apply_callback_style), ("update_vectors", update_vectors),
    ):
        _bind(Field, name, function)
    g["_FMN_VECTOR_FIELDS_INSTALLED"] = True
