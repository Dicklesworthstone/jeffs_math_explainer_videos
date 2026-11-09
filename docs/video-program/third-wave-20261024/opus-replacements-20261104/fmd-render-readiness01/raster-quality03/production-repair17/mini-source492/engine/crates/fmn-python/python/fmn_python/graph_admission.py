"""Bound Python graph controls and isolate failures at the native builder seam.

Atlas/Chisel still choose every sample, discontinuity partition, smooth handle
and contour. This adapter only bounds iterable materialization, validates sample
values, and stops authored execution at the first error.
"""
from __future__ import annotations

import functools
import inspect
import itertools
import math
import operator

from .sampling_callbacks import FirstFailure, point_sample, scalar_sample

_MAX_SAMPLES = 65_536
_BUILDERS = ("_build_parametric_curve", "_build_function_graph", "_build_implicit_function")
_CLASSES = ("ParametricCurve", "FunctionGraph", "ImplicitFunction")


def _numbers(value, name, maximum, minimum=0):
    values = tuple(itertools.islice(iter(value), maximum + 1))
    if not minimum <= len(values) <= maximum:
        raise ValueError(f"{name} requires {minimum}..{maximum} entries")
    values = tuple(float(v) for v in values)
    if any(not math.isfinite(v) for v in values):
        raise ValueError(name + " entries must be finite")
    return values


def _controls(values, implicit):
    # Native admission remains responsible for the aggregate sample, contour
    # depth and work budgets. In particular, finite non-positive curve steps
    # retain Atlas's documented endpoint-only behavior rather than a new rule.
    for key in ("t_range", "x_range", "y_range"):
        if key in values:
            values[key] = _numbers(values[key], key, 2 if implicit else 3, 2)
    if "discontinuities" in values:
        values["discontinuities"] = _numbers(values["discontinuities"], "discontinuities", _MAX_SAMPLES)


def install_graph_admission(native):
    g = vars(native)
    if g.get("_FMN_GRAPH_ADMISSION_INSTALLED", False):
        return
    Mobject, np = g["Mobject"], g["_np"]
    builders = [(name, getattr(Mobject, name), inspect.signature(getattr(Mobject, name)))
                for name in _BUILDERS]
    constructors = [(g[name], g[name].__init__, inspect.signature(g[name].__init__)) for name in _CLASSES]

    def builder(name, signature):
        saved = "_FMN_ADMITTED_ORIGINAL" + name
        implicit = name == "_build_implicit_function"
        curve = name == "_build_parametric_curve"
        key = "t_func" if curve else "func" if implicit else "function"
        convert = (functools.partial(point_sample, np=np, label="curve t_func") if curve else
                   functools.partial(scalar_sample, finite_record=not implicit))
        sentinel = (0., 0., 0.) if curve else float("nan")

        def build(self, *args, **kwargs):
            if self._is_bound():
                raise RuntimeError("a graph constructor requires a detached target; use live binding or become")
            bound = signature.bind(self, *args, **kwargs)
            _controls(bound.arguments, implicit)
            guard = FirstFailure(bound.arguments[key], convert, sentinel)
            bound.arguments[key] = guard
            try:
                # Do not capture native PyCFunctions in installed method closures:
                # their module references defeat the embedded teardown protocol.
                return g[saved](*bound.args, **bound.kwargs)
            finally:
                guard.close()
        build.__name__ = name
        build.__qualname__ = Mobject.__qualname__ + "." + name
        build.__module__ = Mobject.__module__
        build.__signature__ = signature
        return saved, build

    def constructor(cls, original, signature):
        implicit = cls is g["ImplicitFunction"]
        @functools.wraps(original)
        def initialize(self, *args, **kwargs):
            if self._is_bound():
                raise RuntimeError("a graph constructor requires a detached target; use live binding or become")
            bound = signature.bind(self, *args, **kwargs)
            _controls(bound.arguments, implicit)
            for parameter in signature.parameters.values():
                if parameter.kind == inspect.Parameter.VAR_KEYWORD and parameter.name in bound.arguments:
                    _controls(bound.arguments[parameter.name], implicit)
            bound.apply_defaults()
            values = bound.arguments
            style = dict(values["kwargs"])
            if implicit:
                function = values["func"]
                if not callable(function):
                    raise TypeError("func must be callable")
                joint_type = values["joint_type"]
                if joint_type not in g["VMobject"].joint_type_map:
                    raise ValueError(f"unknown VMobject joint type: {joint_type}")
                self.func = function
                self.x_range = values["x_range"]
                self.y_range = values["y_range"]
                self.min_depth = operator.index(values["min_depth"])
                self.max_quads = operator.index(values["max_quads"])
                self.use_smoothing = bool(values["use_smoothing"])
                style["joint_type"] = joint_type
            elif cls is g["ParametricCurve"]:
                function = values["t_func"]
                if not callable(function):
                    raise TypeError("t_func must be callable")
                self.t_func = function
                self.t_range = values["t_range"]
                self.epsilon = float(values["epsilon"])
                self.discontinuities = tuple(float(v) for v in values["discontinuities"])
                self.use_smoothing = bool(values["use_smoothing"])
            else:
                function = values["function"]
                if not callable(function):
                    raise TypeError("function must be callable")
                self.function = function
                self.x_range = values["x_range"]
                self.t_range = self.x_range
                self.epsilon = float(style.pop("epsilon", 1e-8))
                self.discontinuities = tuple(float(v) for v in style.pop("discontinuities", ()))
                self.use_smoothing = bool(style.pop("use_smoothing", True))
                style.setdefault("color", values["color"])

                def parametric_function(t):
                    return [t, scalar_sample(function(t), finite_record=True), 0.0]

                self.t_func = parametric_function
            # The recipe is ready before init_data/init_points. The existing
            # engine dispatcher owns MRO, custom dtypes and all four hooks;
            # graphing/curve_regeneration install native implementations of
            # init_points before any public constructor is exposed.
            g["_init_native_vmobject"](self, style)
        return initialize

    for name, original, signature in builders:
        saved, method = builder(name, signature)
        g[saved] = original
        setattr(Mobject, name, method)
    for cls, original, signature in constructors:
        cls.__init__ = constructor(cls, original, signature)
    g["_FMN_GRAPH_ADMISSION_INSTALLED"] = True
