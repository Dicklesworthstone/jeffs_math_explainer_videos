"""Coordinate-system authoring over native NumberLine and VMobject owners.

The host composes existing axes through the public factories. Atlas still owns
axis/tick geometry; Marionette owns records, views and family adoption. Do not
install a prebuilt native tree over a subclass's initialization hooks.
"""
from __future__ import annotations

from collections.abc import Mapping
from copy import deepcopy
from itertools import islice
import math


def _range(g, value, name):
    terms = tuple(islice(iter(value), 4))
    if len(terms) not in (2, 3):
        raise ValueError(name + " must contain two or three values")
    terms = tuple(float(v) for v in g["_FULL_RANGE_SPECIFIER"](terms))
    if (not all(math.isfinite(v) for v in terms)
            or terms[1] <= terms[0] or terms[2] <= 0):
        raise ValueError(name + " must be finite, increasing, with a positive step")
    return terms


def _length(value, name):
    if value is None:
        return None
    value = float(value)
    if not math.isfinite(value) or value <= 0:
        raise ValueError(name + " must be finite and positive")
    return value


def _config(value, name):
    if value is None:
        return {}
    if not isinstance(value, Mapping):
        raise TypeError(name + " must be a mapping")
    return deepcopy(dict(value))


def _bind(cls, name, function):
    function.__name__ = name
    function.__qualname__ = cls.__qualname__ + "." + name
    function.__module__ = cls.__module__
    setattr(cls, name, function)


def _axis(g, value, previous=()):
    if not isinstance(value, g["NumberLine"]):
        raise TypeError("create_axis must return a NumberLine")
    members = tuple(value.get_family())
    if any(getattr(member, "_scene", None) is not None or member._is_bound()
           for member in members):
        raise ValueError("create_axis must return detached geometry, not another Scene's axis")
    ids = {id(member) for member in members}
    if any(id(member) in ids for root in previous for member in root.get_family()):
        raise ValueError("create_axis must return independent axis families")
    return value


def install_axes_lifecycle(native):
    """Honor group hooks, class defaults and create_axis for 2D/3D charts."""
    g = vars(native)
    if g.get("_FMN_AXES_LIFECYCLE_INSTALLED", False):
        return
    Axes, ThreeD = g["Axes"], g["ThreeDAxes"]
    merge, Group = g["_merged_config"], g["VGroup"]

    def axes_init(self, x_range=(-8., 8., 1.), y_range=(-4., 4., 1.),
                  axis_config=None, x_axis_config=None, y_axis_config=None,
                  height=None, width=None, unit_size=1., **kwargs):
        x_range, y_range = _range(g, x_range, "x_range"), _range(g, y_range, "y_range")
        height, width = _length(height, "height"), _length(width, "width")
        unit_size = _length(unit_size, "unit_size")
        if unit_size is None:
            raise TypeError("unit_size must be a number")
        common = _config(axis_config, "axis_config")
        x_config, y_config = _config(x_axis_config, "x_axis_config"), _config(y_axis_config, "y_axis_config")
        self._axes_params = (x_range, y_range, common, x_config, y_config, height, width, unit_size)
        # The Reference initializes the coordinate mixin before the group,
        # so subclass init_data/init_points can inspect the ranges. Axes are
        # built AFTER those hooks, not retroactively swapped into the root.
        g["CoordinateSystem"].__init__(
            self, x_range, y_range, kwargs.pop("num_sampled_graph_points_per_tick", 5),
        )
        Group.__init__(self, **kwargs)
        common = dict(common, unit_size=unit_size)
        self.x_axis = _axis(g, self.create_axis(
            self.x_range, axis_config=deepcopy(merge(
                _config(self.default_axis_config, "default_axis_config"),
                _config(self.default_x_axis_config, "default_x_axis_config"), common, x_config,
            )), length=width,
        ))
        self.y_axis = _axis(g, self.create_axis(
            self.y_range, axis_config=deepcopy(merge(
                _config(self.default_axis_config, "default_axis_config"),
                _config(self.default_y_axis_config, "default_y_axis_config"), common, y_config,
            )), length=height,
        ), (self.x_axis,))
        # Check again after the second authored factory: it may have adopted
        # or changed ownership of the first axis while preparing the second.
        _axis(g, self.x_axis)
        self.y_axis.rotate(math.pi / 2, about_point=g["_ORIGIN"])
        self.axes = Group(self.x_axis, self.y_axis)
        self.add(*self.axes)
        self.center()

    def three_d_init(self, x_range=(-6., 6., 1.), y_range=(-5., 5., 1.),
                     z_range=(-4., 4., 1.), z_axis_config=None, z_normal=None,
                     depth=None, **kwargs):
        z_range = _range(g, z_range, "z_range")
        depth = _length(depth, "depth")
        z_config = _config(z_axis_config, "z_axis_config")
        original_common = _config(kwargs.get("axis_config"), "axis_config")
        try:
            normal = g["_vec3"](g["_DOWN"] if z_normal is None else z_normal)
        except (TypeError, ValueError, IndexError) as error:
            raise TypeError("ThreeDAxes z_normal must be a 3-vector; got " + repr(z_normal)) from error
        if not all(math.isfinite(v) for v in normal):
            raise ValueError("z_normal must be finite")
        super(ThreeD, self).__init__(x_range, y_range, **kwargs)
        self.z_range = z_range
        # Deliberately no top-level unit_size injection here: the pinned
        # Reference and native ThreeDAxes give z its own axis-config scale.
        self.z_axis = _axis(g, self.create_axis(
            z_range, axis_config=deepcopy(merge(
                _config(self.default_axis_config, "default_axis_config"),
                _config(self.default_z_axis_config, "default_z_axis_config"),
                original_common, z_config,
            )), length=depth,
        ), (self.x_axis, self.y_axis))
        self.z_axis.rotate(-math.pi / 2, g["_UP"], about_point=g["_ORIGIN"])
        self.z_axis.rotate(math.atan2(normal[1], normal[0]), g["_OUT"], about_point=g["_ORIGIN"])
        self.z_axis.shift(self.x_axis.n2p(0))
        self.axes.add(self.z_axis)
        self.add(self.z_axis)

    _bind(Axes, "__init__", axes_init)
    _bind(ThreeD, "__init__", three_d_init)
    g["_FMN_AXES_LIFECYCLE_INSTALLED"] = True
