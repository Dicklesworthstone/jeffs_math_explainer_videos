"""Live StreamLines rebuilding over Atlas's single native seed/RK45 pipeline.

Author hooks choose coordinate seeds and fields. Geometry is built off-scene;
only a fully validated family replaces the old children. Native record identity,
curve smoothing, arc-length spacing, styles and animation clocks stay native.
"""
from __future__ import annotations

from functools import wraps
import inspect
import itertools
import math
import operator
from types import SimpleNamespace

from fmn_python.streamline_authoring import _dimension, _field, _finite, _style_plan

_MAX_POINTS = 65_536
_DRAW_COUNT = "_fmn_streamline_sample_draws"
_MISSING = object()


def _integer(value, name, minimum=0):
    result = operator.index(value)
    if not minimum <= result <= _MAX_POINTS:
        raise ValueError(f"StreamLines {name} must be in {minimum}..65536")
    return result


def _ranges(coordinates):
    result = []
    for row in itertools.islice(iter(coordinates.get_all_ranges()), 4):
        row = tuple(itertools.islice(iter(row), 4))
        if len(row) == 2:
            row += (1.,)
        if len(row) != 3:
            raise ValueError("StreamLines coordinate ranges must have two or three entries")
        row = tuple(_finite(v, "StreamLines coordinate range") for v in row)
        if row[2] <= 0:
            raise ValueError("StreamLines coordinate steps must be positive")
        result.append(row)
    if len(result) != _dimension(coordinates):
        raise ValueError("StreamLines ranges must match the coordinate dimension")
    return result


def _point(np, value, name):
    array = np.asarray(value)
    if array.shape != (3,) or array.dtype.kind not in "biuf":
        raise ValueError(name + " must be a real three-component scene point")
    array = np.asarray(array, dtype=float)
    if not np.isfinite(array).all() or np.any(np.abs(array) > np.finfo(np.float32).max):
        raise ValueError(name + " must be finite and f32-representable")
    return array


def _seeds(np, values, dimension):
    if not isinstance(values, (np.ndarray, list, tuple)):
        values = list(itertools.islice(iter(values), _MAX_POINTS + 1))
    if len(values) > _MAX_POINTS:
        raise ValueError("StreamLines seeds exceed the 65536-point resource budget")
    array = np.asarray(values)
    if array.shape == (0,):
        array = np.empty((0, dimension))
    if array.ndim != 2 or array.shape[1] not in (dimension, 3):
        raise ValueError("StreamLines seeds must have shape (N, dimension) or (N, 3)")
    if array.dtype.kind not in "biuf":
        raise TypeError("StreamLines seeds must be real numeric coordinates")
    if not np.isfinite(array).all():
        raise ValueError("StreamLines seeds must be finite")
    padded = np.zeros((len(array), 3))
    padded[:, :array.shape[1]] = array
    return padded


def _native_widths(g, points, width, taper):
    return g["_np"].asarray(g["_stream_line_widths"](points.tolist(), width, taper), dtype=float)


def _settings(owner):
    result = {}
    for name in ("density", "solution_time", "dt", "cutoff_norm"):
        value = _finite(getattr(owner, name), "StreamLines " + name)
        if value <= 0:
            raise ValueError("StreamLines " + name + " must be positive")
        result[name] = value
    result["arc_len"] = float(owner.arc_len)
    if math.isnan(result["arc_len"]) or result["arc_len"] <= 0:
        raise ValueError("StreamLines arc_len must be positive (infinity is allowed)")
    for name, minimum in (("n_repeats", 0), ("max_time_steps", 0), ("n_samples_per_line", 0)):
        result[name] = _integer(getattr(owner, name), name, minimum)
    result["noise_factor"] = (None if owner.noise_factor is None else
                              _finite(owner.noise_factor, "StreamLines noise_factor"))
    return result


def _ensure_idle(owner):
    # AnimatedStreamLines owns a fixed set of per-line flash drivers. Replacing
    # their operands would orphan running effects; the owner must stop first.
    for line in owner.submobjects:
        for parent in getattr(line, "parents", ()):
            controller = vars(parent).get("_streamline_controller")
            if controller is not None and not controller.closed:
                raise RuntimeError("stop active AnimatedStreamLines before rebuilding its source")


def install_streamlines(native):
    g = vars(native)
    if g.get("_FMN_STREAMLINES_INSTALLED", False):
        return
    if not g.get("_FMN_STREAMLINE_REBUILD_INSTALLED", False):
        raise ImportError("StreamLines seed authoring requires the native family-rebuild adapter")
    # A wheel missing its paired native binding must refuse at initialization.
    sample_native = g["_stream_line_samples"]
    g["_stream_line_widths"]  # Resolve both paired bindings before mutating classes.
    Lines, np = g["StreamLines"], g["_np"]
    from fmn_python import streamline_authoring
    streamline_authoring._widths = _native_widths
    previous = Lines.__init__
    signature = inspect.signature(previous)

    @wraps(previous)
    def initialize(self, *args, **kwargs):
        bound = signature.bind(self, *args, **kwargs)
        bound.apply_defaults()
        controls = dict(bound.arguments)
        controls.pop("self")
        extras = controls.pop("kwargs")
        if not callable(controls["func"]):
            raise TypeError(
                "StreamLines func must be a callable vector field; got "
                + type(controls["func"]).__name__
            )
        coordinates = controls["coordinate_system"]
        if not (
            hasattr(coordinates, "c2p")
            and hasattr(coordinates, "p2c")
            and hasattr(coordinates, "get_all_ranges")
        ):
            raise TypeError(
                "StreamLines requires a coordinate system with "
                "c2p/p2c/get_all_ranges; got "
                + type(coordinates).__name__
            )
        g["_preflight_vmobject_style_kwargs"](dict(extras))
        g["_install_live_state"](self)
        # The staged solver initializes its candidate, not this public root.
        # Give the root the ordinary empty native record buffer before the
        # first validated family replacement; do not adopt it into a Scene.
        self._engine_init()
        for name, value in controls.items():
            setattr(self, name, value)
        self._stream_virtual_times, self._stream_rng_draws = [], 0
        self.draw_lines()
        self.init_style()
        g["_apply_vmobject_style_kwargs"](self, dict(extras))

    def get_sample_coords(self):
        controls = _settings(self)
        coordinates, dimension = self.coordinate_system, _dimension(self.coordinate_system)
        zero = [0.] * dimension
        one = [1.] + [0.] * (dimension - 1)
        origin = _point(np, coordinates.c2p(*zero), "StreamLines origin")
        unit = _point(np, coordinates.c2p(*one), "StreamLines x unit")
        x_unit = g["get_norm"](unit - origin)
        seeds, draws = sample_native(_ranges(coordinates), dimension, float(x_unit),
                                     controls["density"], controls["n_repeats"],
                                     controls["noise_factor"], 0)
        # This is a per-query substream replay, not consumption of global RNG.
        # Publish a draw count only into the enclosing redraw's temporary slot.
        if _DRAW_COUNT in vars(self):
            vars(self)[_DRAW_COUNT] = draws
        return np.array(seeds, dtype=float).reshape((-1, 3))[:, :dimension].copy()

    def build_candidate(self, controls):
        _ensure_idle(self)
        controls = dict(controls)
        controls.update(_settings(SimpleNamespace(**controls)))
        # Empty-family planning validates paint controls without touching
        # geometry or calling the field before seed/solver admission.
        _style_plan(g, SimpleNamespace(**dict(controls, submobjects=[])))
        function, coordinates = controls["func"], controls["coordinate_system"]
        if not callable(function):
            raise TypeError(
                "StreamLines func must be a callable vector field; got "
                + type(function).__name__
            )
        if not (
            hasattr(coordinates, "c2p")
            and hasattr(coordinates, "p2c")
            and hasattr(coordinates, "get_all_ranges")
        ):
            raise TypeError(
                "StreamLines requires a coordinate system with "
                "c2p/p2c/get_all_ranges; got "
                + type(coordinates).__name__
            )
        dimension, ranges = _dimension(coordinates), _ranges(coordinates)
        previous_count = vars(self).get(_DRAW_COUNT, _MISSING)
        vars(self)[_DRAW_COUNT] = 0
        try:
            seeds = _seeds(np, self.get_sample_coords(), dimension)
            draws = operator.index(vars(self)[_DRAW_COUNT])
            if len(seeds) * max(0, 2 * controls["n_samples_per_line"] - 1) > _MAX_POINTS:
                raise ValueError("StreamLines drawn paths exceed the 65536-point resource budget")
            def field(rows):
                return _field(np, function, rows, dimension).tolist()
            def c2p(coords):
                return tuple(_point(np, coordinates.c2p(*coords[:dimension]), "StreamLines c2p"))
            def p2c(point):
                values = np.asarray(coordinates.p2c(np.array(point, dtype=float)))
                if values.shape != (dimension,) or values.dtype.kind not in "biuf" or not np.isfinite(values).all():
                    raise ValueError("StreamLines p2c must return a finite coordinate vector")
                return tuple(_seeds(np, [values], dimension)[0])
            scratch = g["VGroup"]()
            # Atlas constructs every curve. The existing live style
            # planner then paints the candidate before it is published.
            specs, virtual_times, _ = scratch._build_stream_lines(
                g["_native_shell_factory"], field, c2p, p2c, ranges, dimension, 0,
                controls["density"], controls["n_repeats"], controls["noise_factor"],
                controls["solution_time"], controls["dt"], controls["arc_len"],
                controls["max_time_steps"], controls["n_samples_per_line"], controls["cutoff_norm"],
                1., None, 1., False, (0., 2.), False, seeds.tolist(),
            )
            g["_hang_native_children"](scratch, specs)
            children = list(scratch.submobjects)
            if len(children) != len(seeds) or len(virtual_times) != len(children):
                raise RuntimeError("native StreamLines family/metadata mismatch")
            for line, duration in zip(children, virtual_times):
                points = np.asarray(line.get_points())
                if not np.isfinite(points).all():
                    raise ValueError("native StreamLines points must be finite")
                line.virtual_time = _finite(duration, "StreamLines virtual_time", nonnegative=True)
            # Reuse the live style planner against the private candidate,
            # so a restyling error cannot discard the displayed family.
            paint_owner = SimpleNamespace(**dict(controls, submobjects=children))
            for line, _, rgba, widths in _style_plan(g, paint_owner):
                line.data["stroke_rgba"][:] = rgba
                line.data["stroke_width"][:, 0] = widths
            scratch._stream_virtual_times = list(virtual_times)
            scratch._stream_rng_draws = draws
            return scratch
        finally:
            if previous_count is _MISSING:
                vars(self).pop(_DRAW_COUNT, None)
            else:
                vars(self)[_DRAW_COUNT] = previous_count

    g["_fmn_streamline_candidate"] = build_candidate
    for name, method in (("__init__", initialize), ("get_sample_coords", get_sample_coords)):
        method.__name__ = name
        method.__qualname__ = Lines.__qualname__ + "." + name
        method.__module__ = Lines.__module__
        setattr(Lines, name, method)
    g["_FMN_STREAMLINES_INSTALLED"] = True
