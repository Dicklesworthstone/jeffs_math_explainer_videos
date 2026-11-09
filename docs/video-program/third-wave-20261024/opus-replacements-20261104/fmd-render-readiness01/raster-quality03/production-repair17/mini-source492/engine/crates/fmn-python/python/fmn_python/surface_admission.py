"""Bound surface construction at the portal's native sampling boundary.

Atlas remains the geometry and normal sampler. This layer admits Python inputs
before native allocation, and stops invoking user code after its first failure.
The old infallible native sampler may finish its *bounded* remaining iterations;
its original exception channel still refuses publication of the failed result.
"""
from __future__ import annotations

import functools
import inspect
import itertools
import math
import operator

from .sampling_callbacks import FirstFailure, point_sample, F32_MAX as _F32_MAX

# Marionette's MAX_SURFACE_GRID_POINTS, which Atlas samples under as
# fmn_library::solids::SURFACE_SAMPLING_BUDGET. Every UV-grid mirror in this
# package imports it; curves, fields and wireframe output keep 65536.
GRID_BUDGET = 262_144


def _pair(value, name):
    values = tuple(itertools.islice(iter(value), 3))
    if len(values) != 2:
        raise ValueError(name + " requires exactly two values")
    return values


def grid_shape(value, *, copies=1):
    shape = tuple(operator.index(v) for v in _pair(value, "surface resolution"))
    if (any(v < 0 or v > GRID_BUDGET for v in shape)
            or math.prod(shape) * copies > GRID_BUDGET):
        raise ValueError(f"surface construction exceeds its {GRID_BUDGET}-point aggregate UV-grid budget")
    return shape


def _finite(value, name, *, record=False):
    result = float(value)
    if not math.isfinite(result) or (record and abs(result) > _F32_MAX):
        raise ValueError(name + " must be finite" + (" and f32-representable" if record else ""))
    return result


def sampling_options(options):
    """Normalize controls once, before a grid-sized allocation or any UV call."""
    result = dict(options)
    result["resolution"] = grid_shape(result.get("resolution", (101, 101)))
    for key in ("u_range", "v_range"):
        pair = tuple(_finite(v, key) for v in _pair(result.get(key, (0, 1)), key))
        if not math.isfinite(pair[1] - pair[0]):
            raise ValueError(key + " span must remain finite")
        result[key] = pair
    epsilon = _finite(result.get("epsilon", .001), "surface epsilon")
    nudge = _finite(result.get("normal_nudge", .001), "surface normal_nudge", record=True)
    if epsilon <= 0 or nudge < 0:
        raise ValueError("surface epsilon must be positive and normal_nudge nonnegative")
    if any(not math.isfinite(v + epsilon) for key in ("u_range", "v_range") for v in result[key]):
        raise ValueError("surface derivative samples must remain finite")
    axis = operator.index(result.get("preferred_creation_axis", 1))
    if axis not in (0, 1):
        raise ValueError("surface preferred_creation_axis must be 0 or 1")
    result.update(epsilon=epsilon, normal_nudge=nudge, preferred_creation_axis=axis)
    return result


_BUILDERS = (
    "_build_parametric_surface", "_build_sphere", "_build_torus",
    "_build_cylinder", "_build_cone", "_build_line3d", "_build_disk3d",
    "_build_square3d", "_build_cube", "_build_prism",
)
_CLASSES = ("Surface", "ParametricSurface", "Sphere", "Torus", "Cylinder",
            "Cone", "Line3D", "Disk3D", "Square3D", "Cube", "Prism")
_DIMENSIONS = ("radius", "height", "width", "depth", "r1", "r2", "side_length")


def install_surface_admission(native):
    g = vars(native)
    if g.get("_FMN_SURFACE_ADMISSION_INSTALLED", False):
        return
    Mobject, np = g["Mobject"], g["_np"]
    # Resolve the complete installation before changing method resolution.
    builders = [(name, getattr(Mobject, name), inspect.signature(getattr(Mobject, name)))
                for name in _BUILDERS]
    constructors = [(g[name], g[name].__init__, inspect.signature(g[name].__init__))
                    for name in _CLASSES]

    def builder(name, signature):
        saved = "_FMN_ADMITTED_ORIGINAL" + name

        def build(self, *args, **kwargs):
            if self._is_bound():
                raise RuntimeError("a surface constructor requires a detached target; use init_points or become")
            bound = signature.bind(self, *args, **kwargs)
            values = bound.arguments
            if "resolution" in values:
                normalized = sampling_options({key: values[key] for key in
                    ("resolution", "u_range", "v_range", "epsilon", "normal_nudge", "preferred_creation_axis")
                    if key in values})
                values.update({key: val for key, val in normalized.items() if key in values})
            if "square_resolution" in values:
                values["square_resolution"] = grid_shape(values["square_resolution"], copies=6)
            for key in _DIMENSIONS:
                if key in values:
                    values[key] = _finite(values[key], "surface " + key, record=True)
            for key in ("start", "end", "axis"):
                if key in values:
                    vector = tuple(itertools.islice(iter(values[key]), 4))
                    if len(vector) != 3:
                        raise ValueError("surface " + key + " must contain three coordinates")
                    values[key] = tuple(_finite(v, "surface " + key, record=True) for v in vector)
            guard = None
            if "uv_func" in values:
                guard = FirstFailure(values["uv_func"],
                    functools.partial(point_sample, np=np, label="surface uv_func"), (0., 0., 0.))
                values["uv_func"] = guard
            try:
                # Dictionary lookup avoids retaining a native module through a
                # PyO3 function captured by every installed method closure.
                return g[saved](*bound.args, **bound.kwargs)
            finally:
                if guard is not None:
                    guard.close()
        build.__name__ = name
        build.__qualname__ = Mobject.__qualname__ + "." + name
        build.__module__ = Mobject.__module__
        build.__signature__ = signature
        return saved, build

    def constructor(cls, original, signature):
        @functools.wraps(original)
        def initialize(self, *args, **kwargs):
            if self._is_bound():
                raise RuntimeError("a surface constructor requires a detached target; use init_points or become")
            bound = signature.bind(self, *args, **kwargs)
            # Freeze caller-controlled shape values before bootstrap int casts
            # can truncate floats. Do not populate defaults: subclass keyword
            # forwarding must keep the original inheritance and defaults.
            containers = [bound.arguments]
            containers.extend(bound.arguments[p.name] for p in signature.parameters.values()
                              if p.kind == inspect.Parameter.VAR_KEYWORD and p.name in bound.arguments)
            for values in containers:
                for key, copies in (("resolution", 1), ("square_resolution", 6)):
                    if key in values:
                        values[key] = grid_shape(values[key], copies=copies)
                if "preferred_creation_axis" in values:
                    axis = operator.index(values["preferred_creation_axis"])
                    if axis not in (0, 1):
                        raise ValueError("surface preferred_creation_axis must be 0 or 1")
                    values["preferred_creation_axis"] = axis
            return original(*bound.args, **bound.kwargs)
        return initialize

    for name, original, signature in builders:
        saved, method = builder(name, signature)
        g[saved] = original
        setattr(Mobject, name, method)
    for cls, original, signature in constructors:
        cls.__init__ = constructor(cls, original, signature)
    g["_FMN_SURFACE_ADMISSION_INSTALLED"] = True
