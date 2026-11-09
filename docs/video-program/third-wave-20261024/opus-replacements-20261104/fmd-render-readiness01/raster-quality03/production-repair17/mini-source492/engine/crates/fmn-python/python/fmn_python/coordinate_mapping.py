"""Invert the shipped affine axes chart using its live native line geometry.

Independent axis projections are only an inverse for orthogonal axes. The
scaled dual basis handles shears, reflections and 2D charts embedded in 3D.
This is the host's existing coordinate adapter, not a geometry/renderer owner.
"""
from __future__ import annotations

import math
import sys
import itertools
import inspect
from types import SimpleNamespace


_MAX_LABELS = 4096
_MAX_LABEL_RECORDS = 1_048_576


def _label_values(values, name, convert=float):
    result = []
    for value in itertools.islice(iter(values), _MAX_LABELS + 1):
        if len(result) == _MAX_LABELS:
            raise ValueError(name + " exceeds the 4096-label budget")
        value = convert(value)
        if not (math.isfinite(value.real) and math.isfinite(value.imag)):
            raise ValueError(name + " must contain finite numbers")
        result.append(value)
    return result


def _label_ticks(np, terms, include_tip):
    low, high, step = (float(v) for v in terms)
    if not all(math.isfinite(v) for v in (low, high, step)) or step <= 0 or high < low:
        raise ValueError("coordinate label range must be finite, ordered and have a positive step")
    stop = high if include_tip else high + step
    count = (stop - low) / step
    if not math.isfinite(count) or count > _MAX_LABELS:
        raise ValueError("coordinate label range exceeds the 4096-label budget")
    return [float(v) for v in np.arange(low, stop, step) if v <= high]


def install_coordinate_labels(native):
    """Compose Scribe numbers at live coordinates, never reconstructed boxes.

    Preparation is separate from family publication. No glyph layout, axis
    geometry, record generation or scene clock is implemented in this adapter.
    """
    g = vars(native)
    if g.get("_FMN_COORDINATE_LABELS_INSTALLED", False):
        return
    Line, Group, np = (g[name] for name in ("NumberLine", "VGroup", "_np"))
    stock_label, stock_ticks = Line.get_number_mobject, Line.get_tick_range
    Axes, Plane, Complex = (g[name] for name in ("Axes", "NumberPlane", "ComplexPlane"))
    stock_complex_defaults = Complex.get_default_coordinate_values
    active = set()

    def geometry_state(axis):
        return (axis.get_points().copy(), tuple(axis.submobjects),
                vars(axis).get("_scene"), axis._is_bound())

    def check_geometry(axis, before):
        points, children, owner, bound = before
        if (tuple(axis.submobjects) != children or vars(axis).get("_scene") is not owner
                or axis._is_bound() != bound or not np.array_equal(axis.get_points(), points)):
            raise RuntimeError("coordinate geometry or ownership changed while preparing labels")

    def options(kwargs):
        out = dict(kwargs)
        for name in ("font_size", "buff", "unit"):
            if name in out and out[name] is not None:
                value = float(out[name])
                if not math.isfinite(value) or (name == "font_size" and value <= 0) or (name == "unit" and value == 0):
                    raise ValueError("coordinate label " + name + " must be finite and valid")
                out[name] = value
        if out.get("direction") is not None:
            direction = np.asarray(out["direction"], dtype=float)
            if direction.shape != (3,) or not np.isfinite(direction).all():
                raise ValueError("coordinate label direction must be a finite three-vector")
            out["direction"] = direction.copy()
        return out

    def stock_options(kwargs):
        # Keep the existing precise refusal for genuinely unknown keywords,
        # while routing the actual DecimalNumber and unit-label surface.
        allowed = set(inspect.signature(g["DecimalNumber"].__init__).parameters)
        allowed.update(("direction", "buff", "unit", "unit_tex", "color", "opacity",
                        "fill_color", "fill_opacity", "stroke_color", "stroke_width", "stroke_opacity"))
        g["_refuse_unrouted"]("NumberLine.add_numbers()",
                              [(name, True) for name in sorted(kwargs) if name not in allowed])

    def planned_labels(formatter, values, kwargs, budget=None):
        if budget is None:
            budget = [0]
        labels, seen = [], set()
        for value in values:
            label = formatter(value, **kwargs)
            if not isinstance(label, g["VMobject"]):
                raise TypeError("coordinate label formatter must return a VMobject")
            for member in label.get_family():
                if id(member) in seen:
                    continue
                seen.add(id(member))
                budget[0] += member.get_num_points()
                if budget[0] > _MAX_LABEL_RECORDS:
                    raise ValueError("coordinate labels exceed their native record budget")
                points = member.get_points()
                if not np.isfinite(points).all():
                    raise ValueError("coordinate label geometry must be finite")
            labels.append(label)
        return Group(*labels)

    def add_numbers(self, x_values=None, excluding=None, font_size=24, **kwargs):
        if id(self) in active:
            raise RuntimeError("coordinate labeling cannot reenter itself")
        active.add(id(self))
        try:
            before = geometry_state(self)
            parameters = (self.x_min, self.x_max, self.x_step)
            formatter = self.get_number_mobject
            if getattr(formatter, "__func__", None) is stock_label:
                stock_options(kwargs)
            kwargs = options(dict(kwargs, font_size=font_size))
            if x_values is None:
                x_values = (_label_ticks(np, parameters, self.include_tip)
                            if getattr(self.get_tick_range, "__func__", None) is stock_ticks
                            else self.get_tick_range())
            values = _label_values(x_values, "number-line labels")
            excluded = self.numbers_to_exclude if excluding is None else excluding
            excluded = set(() if excluded is None else _label_values(excluded, "excluded labels"))
            group = planned_labels(formatter, [v for v in values if v not in excluded], kwargs)
            check_geometry(self, before)
            if (self.x_min, self.x_max, self.x_step) != parameters:
                raise RuntimeError("number-line range changed while preparing labels")
            self.add(group)
            self.numbers = group
            return group
        finally:
            active.remove(id(self))

    add_numbers.__module__ = Line.__module__
    add_numbers.__qualname__ = Line.__qualname__ + ".add_numbers"
    Line.add_numbers = add_numbers

    def chart_axes(self):
        axes = tuple(itertools.islice(iter(self.get_axes()), 2))
        ranges = tuple(tuple(float(x) for x in itertools.islice(iter(term), 4))
                       for term in itertools.islice(iter(self.get_all_ranges()), 2))
        if len(axes) != 2 or len(ranges) != 2:
            raise ValueError("coordinate labeling requires two live axes")
        for axis, terms in zip(axes, ranges):
            if not isinstance(axis, g["VMobject"]) or axis.get_num_points() < 2:
                raise TypeError("coordinate labeling requires native axis geometry")
            if len(terms) != 3 or not all(math.isfinite(v) for v in terms) or terms[1] <= terms[0] or terms[2] <= 0:
                raise ValueError("coordinate label range must be finite, ordered and have a positive step")
        return axes, ranges

    def axis_config(self, index):
        name = ("x", "y")[index]
        # NumberPlane's legacy _axes_params projection omits the caller's
        # y-axis override. Use the actual constructor inputs for plane shells.
        params = self._plane_params if isinstance(self, Plane) else self._axes_params
        sources = (
            self.default_axis_config, getattr(self, "default_" + name + "_axis_config"),
            params[2], params[index + 3],
        )
        config = {}
        for source in sources:
            for key, value in source.items():
                if key == "decimal_number_config":
                    config[key] = dict(config.get(key) or {}, **(value or {}))
                else:
                    config[key] = value
        return config

    def formatter_for(self, axis, index, terms):
        formatter = getattr(axis, "get_number_mobject", None)
        if callable(formatter):
            return formatter
        config = axis_config(self, index)
        # Axis proxies are currently VMobject shells, not NumberLine instances.
        # Project only the label configuration onto an inert holder. The actual
        # native axis retains its identity, class, records and family ownership.
        holder = SimpleNamespace(
            decimal_number_config=dict(config.get("decimal_number_config") or {"num_decimal_places": 0}),
            line_to_number_direction=config.get("line_to_number_direction", g["DOWN"]),
            line_to_number_buff=config.get("line_to_number_buff", g["MED_SMALL_BUFF"]),
            number_to_point=lambda value: g["_axis_number_to_point"](axis, terms[0], terms[1], value),
        )
        return lambda value, **kw: Line.get_number_mobject(holder, value, **kw)

    def enter_chart(self):
        ids = {id(self)}
        if id(self) in active:
            raise RuntimeError("coordinate labeling cannot reenter itself")
        active.update(ids)
        try:
            axes, ranges = chart_axes(self)
            axis_ids = {id(axis) for axis in axes} - ids
            if axis_ids & active:
                raise RuntimeError("coordinate labeling cannot reenter shared axes")
            active.update(axis_ids)
            ids.update(axis_ids)
            before = [(obj, geometry_state(obj)) for obj in (self, *axes)]
        except BaseException:
            active.difference_update(ids)
            raise
        return axes, ranges, ids, before

    def check_chart(self, axes, ranges, before):
        for obj, state in before:
            check_geometry(obj, state)
        if chart_axes(self) != (axes, ranges):
            raise RuntimeError("coordinate axes or ranges changed while preparing labels")

    def add_coordinate_labels(self, x_values=None, y_values=None, excluding=(0,), **kwargs):
        axes, ranges, ids, before = enter_chart(self)
        try:
            stock_options(kwargs)
            kwargs = options(kwargs)
            if kwargs.get("font_size") is None:
                kwargs["font_size"] = 24.0
            try:
                excluded = set(() if excluding is None else _label_values(excluding, "excluded labels"))
            except TypeError as error:
                if isinstance(self, Plane):
                    raise TypeError("NumberPlane.add_coordinate_labels excluding must be an iterable of real numbers") from error
                raise
            batches = []
            for index, values in enumerate((x_values, y_values)):
                if values is None:
                    axis = axes[index]
                    # coordinate_systems.py:529 labels each axis's own tick
                    # range. The stock one is computed under the label budget
                    # (as NumberLine.add_numbers does); an authored override
                    # is honoured.
                    if not isinstance(axis, Line):
                        values = _label_ticks(np, ranges[index], axis_config(self, index).get("include_tip", False))
                    elif getattr(axis.get_tick_range, "__func__", None) is stock_ticks:
                        values = _label_ticks(np, (axis.x_min, axis.x_max, axis.x_step), axis.include_tip)
                    else:
                        values = axis.get_tick_range()
                batches.append([v for v in _label_values(values, "axis labels") if v not in excluded])
            if sum(map(len, batches)) > _MAX_LABELS:
                raise ValueError("coordinate labels exceed the 4096-label budget")
            budget = [0]
            groups = [planned_labels(formatter_for(self, axis, index, ranges[index]), values, kwargs, budget)
                      for index, (axis, values) in enumerate(zip(axes, batches))]
            check_chart(self, axes, ranges, before)
            # Both axes are completely prepared before either is attached.
            # Each batch is parented by its own axis. coordinate_systems.py:520
            # also keeps the pair as self.coordinate_labels and returns it; that
            # group is an unparented view, so a copy of the chart shares it as
            # the Reference's copy does, while deepcopy and pickle remap it.
            for axis, group in zip(axes, groups):
                axis.add(group)
                axis.numbers = group
            self.coordinate_labels = Group(*groups)
            return self.coordinate_labels
        finally:
            active.difference_update(ids)

    def complex_value(value):
        if isinstance(value, (tuple, list, np.ndarray)):
            if len(value) != 2:
                raise ValueError("complex coordinate labels require complex numbers or real/imaginary pairs")
            return complex(float(value[0]), float(value[1]))
        return complex(value)

    def complex_coordinate_labels(self, numbers=None, skip_first=True, font_size=36, **kwargs):
        axes, ranges, ids, before = enter_chart(self)
        try:
            stock_options(kwargs)
            kwargs = options(dict(kwargs, font_size=font_size))
            if numbers is None:
                if getattr(self.get_default_coordinate_values, "__func__", None) is stock_complex_defaults:
                    ticks = [_label_ticks(np, terms, axis_config(self, index).get("include_tip", False))
                             for index, terms in enumerate(ranges)]
                    if skip_first:
                        ticks = [values[1:] for values in ticks]
                    numbers = [*ticks[0], *(complex(0, v) for v in ticks[1] if v != 0)]
                else:
                    numbers = self.get_default_coordinate_values(skip_first)
            values = _label_values(numbers, "complex coordinate labels", complex_value)
            formatters = [formatter_for(self, axis, i, ranges[i]) for i, axis in enumerate(axes)]
            budget, labels = [0], []
            for value in values:
                imaginary = abs(value.imag) > abs(value.real)
                # Do not leak the imaginary unit into subsequent real labels.
                kw = dict(kwargs, **({"unit_tex": "i"} if imaginary else {}))
                group = planned_labels(formatters[int(imaginary)],
                                       [value.imag if imaginary else value.real], kw, budget)
                labels.extend(group.submobjects)
                # The final group alone should parent these labels.
                group.set_submobjects([])
            group = Group(*labels)
            check_chart(self, axes, ranges, before)
            self.add(group)
            self.coordinate_labels = group
            return self
        finally:
            active.difference_update(ids)

    for cls, method in ((Axes, add_coordinate_labels), (Complex, complex_coordinate_labels)):
        method.__name__ = "add_coordinate_labels"
        method.__module__ = cls.__module__
        method.__qualname__ = cls.__qualname__ + ".add_coordinate_labels"
        cls.add_coordinate_labels = method
    # Do not pass through the old plane wrapper's unbounded tuple conversion.
    Plane.add_coordinate_labels = add_coordinate_labels
    g["_FMN_COORDINATE_LABELS_INSTALLED"] = True


def _dot(left, right):
    return left[0] * right[0] + left[1] * right[1] + left[2] * right[2]


def _cross(left, right):
    return (left[1] * right[2] - left[2] * right[1],
            left[2] * right[0] - left[0] * right[2],
            left[0] * right[1] - left[1] * right[0])


def _unit(vector):
    length = math.hypot(*vector)
    if not math.isfinite(length) or length == 0:
        raise ValueError("axes coordinate mapping requires finite nonzero axis directions")
    return tuple(value / length for value in vector), length


def _inverse(basis, offset):
    """Fixed-order dual-basis solve, with no BLAS or unscaled Gram matrix."""
    normalized = [_unit(column) for column in basis]
    vectors = [item[0] for item in normalized]
    if len(vectors) == 2:
        # The third direction makes the inverse an orthogonal projection onto
        # the actual chart plane, rather than a projection onto screen x/y.
        vectors.append(_unit(_cross(*vectors))[0])
    a, b, c = vectors
    duals = (_cross(b, c), _cross(c, a), _cross(a, b))
    determinant = _dot(a, duals[0])
    if not math.isfinite(determinant) or abs(determinant) <= 64 * sys.float_info.epsilon:
        raise ValueError("axes coordinate mapping is singular or numerically degenerate")
    result = tuple((_dot(offset, dual) / determinant) / scale
                   for dual, (_, scale) in zip(duals, normalized))
    return result


def install_coordinate_mapping(native):
    g = vars(native)
    if g.get("_FMN_COORDINATE_MAPPING_INSTALLED", False):
        return
    Axes, np = g["Axes"], g["_np"]
    original = Axes.point_to_coords
    forward, abbreviated = Axes.coords_to_point, Axes.c2p

    def vector(value, name):
        array = np.asarray(value)
        if array.shape != (3,) or np.iscomplexobj(array):
            raise ValueError(name + " must contain exactly three real coordinates")
        values = tuple(float(v) for v in array)
        if not all(math.isfinite(v) for v in values):
            raise ValueError(name + " must contain finite coordinates")
        return values

    def point_to_coords(self, point):
        # Do not fit an affine model to an authored nonlinear chart. Explicit
        # inverse overrides keep normal Python dispatch; an authored forward
        # override inheriting this method keeps its previous inverse behavior.
        if (getattr(self.coords_to_point, "__func__", None) is not forward
                or getattr(self.c2p, "__func__", None) is not abbreviated):
            return original(self, point)
        target = np.asarray(point)
        if target.ndim == 0 or target.shape[-1] != 3 or np.iscomplexobj(target):
            raise ValueError("axes points must have a final dimension of three real coordinates")
        target = target.astype(float, copy=False)
        if not np.isfinite(target).all():
            raise ValueError("axes points must contain finite coordinates")
        axes, ranges = tuple(self.axes), tuple(self.get_all_ranges())
        if len(axes) not in (2, 3) or len(ranges) != len(axes):
            raise ValueError("affine coordinate mapping requires two or three axes and matching ranges")
        basis, origins = [], []
        for axis, domain in zip(axes, ranges):
            low, high = float(domain[0]), float(domain[1])
            span = high - low
            if not math.isfinite(low) or not math.isfinite(high) or not math.isfinite(span) or span == 0:
                raise ValueError("axes coordinate ranges must have distinct finite bounds")
            start, end = vector(axis._get_start(), "axis start"), vector(axis._get_end(), "axis end")
            column = tuple((b - a) / span for a, b in zip(start, end))
            alpha = -low / span
            origin = tuple((1 - alpha) * a + alpha * b for a, b in zip(start, end))
            basis.append(column)
            origins.append(origin)
        # Match coords_to_point's full-coordinate origin even when individual
        # axes were moved independently. Do not assume they still intersect.
        origin = tuple(origins[0][d] + sum(row[d] - origins[0][d] for row in origins)
                       for d in range(3))
        offset = tuple(target[..., d] - origin[d] for d in range(3))
        if not all(np.isfinite(v).all() for v in offset):
            raise ValueError("axes point displacement must be finite")
        result = _inverse(basis, offset)
        if not all(np.isfinite(v).all() for v in result):
            raise ValueError("axes inverse coordinates are not finite")
        return tuple(float(v) for v in result) if target.ndim == 1 else result

    point_to_coords.__name__ = "point_to_coords"
    point_to_coords.__qualname__ = Axes.__qualname__ + ".point_to_coords"
    point_to_coords.__module__ = Axes.__module__
    Axes.point_to_coords = point_to_coords
    g["_FMN_COORDINATE_MAPPING_INSTALLED"] = True
