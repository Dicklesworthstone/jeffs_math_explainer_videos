"""Public NumberPlane construction and grid factories over native Lines.

Coordinate axes keep their public factory identities. Background/faded lines
are composed with the same live NumberLine mappings as the Reference, rather
than independently reconstructing an unrelated native chart from old options.
"""
from __future__ import annotations

from itertools import islice
import math
import operator

from .coordinate_lifecycle import _bind, _config, _range

_MAX_GRID_LINES = 65_536


def _ratio(value):
    if isinstance(value, bool):
        raise TypeError("faded_line_ratio must be an integer, not bool")
    value = operator.index(value)
    if not 0 <= value <= _MAX_GRID_LINES:
        raise ValueError("faded_line_ratio exceeds the plane grid budget")
    return value


def _grid_size(terms, ratio):
    low, high, frequency = (float(v) for v in terms)
    if (not all(math.isfinite(v) for v in (low, high, frequency))
            or high <= low or frequency <= 0):
        raise ValueError("plane grid range must be finite, increasing, with a positive step")
    step = frequency / (1 + ratio)
    if step <= 0 or not math.isfinite(step):
        raise ValueError("plane grid step is not representable")
    count = (high + step - low) / step
    if not math.isfinite(count) or count > _MAX_GRID_LINES:
        raise ValueError("plane grid exceeds the 65536-line budget")
    return math.ceil(count), step


def install_plane_lifecycle(native):
    g = vars(native)
    if g.get("_FMN_PLANE_LIFECYCLE_INSTALLED", False):
        return
    if not g.get("_FMN_AXES_LIFECYCLE_INSTALLED", False):
        raise ImportError("plane construction requires the public axes lifecycle")
    Plane, Group, np = g["NumberPlane"], g["VGroup"], g["_np"]

    def plane_init(self, x_range=(-8., 8., 1.), y_range=(-4., 4., 1.),
                   background_line_style=None, faded_line_style=None,
                   faded_line_ratio=4, make_smooth_after_applying_functions=True,
                   **kwargs):
        xr, yr = _range(g, x_range, "x_range"), _range(g, y_range, "y_range")
        ratio = _ratio(faded_line_ratio)
        if _grid_size(xr, ratio)[0] + _grid_size(yr, ratio)[0] > _MAX_GRID_LINES:
            raise ValueError("plane grid exceeds the 65536-line aggregate budget")
        background = dict(stroke_color=g["_BLUE_D"], stroke_width=2., stroke_opacity=1.)
        background.update(_config(background_line_style, "background_line_style"))
        faded = dict(stroke_width=1., stroke_opacity=.25)
        faded.update(_config(faded_line_style, "faded_line_style"))
        super(Plane, self).__init__(xr, yr, **kwargs)
        self.background_line_style, self.faded_line_style = background, faded
        self.faded_line_ratio = ratio
        self.make_smooth_after_applying_functions = bool(make_smooth_after_applying_functions)
        # Retain the existing label/legacy inspection projection, but grid
        # helpers read public live attributes, not this constructor snapshot.
        x, y, common, xc, yc, height, width, unit = self._axes_params
        self._plane_params = (x, y, common, xc, yc, dict(background), dict(faded),
                              ratio, height, width, unit)
        self.init_background_lines()

    def grid_pair(self, groups):
        groups = tuple(islice(iter(groups), 3))
        if len(groups) != 2 or any(not isinstance(group, Group) for group in groups):
            raise TypeError("get_lines must return two VGroups: background and faded")
        seen, records, members = {id(self)}, 0, 0
        for group in groups:
            for member in group.get_family():
                members += 1
                if members > 2 * _MAX_GRID_LINES + 2:
                    raise ValueError("authored grid exceeds the plane family budget")
                if id(member) in seen:
                    raise ValueError("background and faded grid families must be independent")
                seen.add(id(member))
                if getattr(member, "_scene", None) is not None or member._is_bound():
                    raise ValueError("grid factories must return detached geometry")
                records += member.get_num_points()
                if records > 3 * _MAX_GRID_LINES:
                    raise ValueError("authored grid exceeds the plane record budget")
                if not np.isfinite(member.get_points()).all():
                    raise ValueError("grid geometry must be finite")
        return groups

    def init_background_lines(self):
        background = _config(self.background_line_style, "background_line_style")
        faded = _config(self.faded_line_style, "faded_line_style")
        if "stroke_color" not in faded:
            faded["stroke_color"] = background["stroke_color"]
        # Prepare and style both detached families before adding either. A
        # failed authored factory/style cannot leave a half-installed grid.
        lines, faint = grid_pair(self, self.get_lines())
        lines.set_style(**background)
        faint.set_style(**faded)
        self.add_to_back(faint, lines)
        self.background_lines, self.faded_lines = lines, faint
        self.faded_line_style = faded

    def get_lines(self):
        x_axis, y_axis = self.get_x_axis(), self.get_y_axis()
        ratio = _ratio(self.faded_line_ratio)
        ranges = [(axis.x_min, axis.x_max, axis.x_step) for axis in (x_axis, y_axis)]
        if sum(_grid_size(terms, ratio)[0] for terms in ranges) > _MAX_GRID_LINES:
            raise ValueError("plane grid exceeds the 65536-line aggregate budget")
        x_background, x_faded = self.get_lines_parallel_to_axis(x_axis, y_axis)
        y_background, y_faded = self.get_lines_parallel_to_axis(y_axis, x_axis)
        parts = [tuple(islice(iter(items), _MAX_GRID_LINES + 1))
                 for items in (x_background, y_background, x_faded, y_faded)]
        if sum(len(items) for items in parts) > _MAX_GRID_LINES:
            raise ValueError("authored grid exceeds the 65536-line aggregate budget")
        if any(not isinstance(item, g["VMobject"]) for items in parts for item in items):
            raise TypeError("grid factories must return VMobject children")
        return Group(*parts[0], *parts[1]), Group(*parts[2], *parts[3])

    def get_lines_parallel_to_axis(self, axis1, axis2):
        ratio = _ratio(self.faded_line_ratio)
        _, step = _grid_size((axis2.x_min, axis2.x_max, axis2.x_step), ratio)
        # This is the Reference's existing grid enumeration, including its
        # arange overshoot and index-based major/minor classification. Only
        # Line/native copy/shift own geometry; do not resample axis curves.
        line = g["Line"](axis1.get_start(), axis1.get_end())
        origin = axis2.n2p(0)
        positions = np.arange(axis2.x_min, axis2.x_max + step, step)
        if len(positions) > _MAX_GRID_LINES:
            raise ValueError("plane grid exceeds the 65536-line budget")
        major, minor = Group(), Group()
        for index, position in enumerate(positions):
            if abs(position) < 1e-8:
                continue
            item = line.copy()
            item.shift(axis2.n2p(position) - origin)
            (major if index % (1 + ratio) == 0 else minor).add(item)
        return major, minor

    for name, function in (("__init__", plane_init), ("init_background_lines", init_background_lines),
                           ("get_lines", get_lines), ("get_lines_parallel_to_axis", get_lines_parallel_to_axis)):
        _bind(Plane, name, function)
    g["_FMN_PLANE_LIFECYCLE_INSTALLED"] = True
