"""Composite effects use their real children and preserve the authored root.

There is no effect-specific frame loop here. Geometry belongs to Mobject and
all begin/update/interpolate/finish work belongs to the existing composition
and leaf implementations, including the borrow-free native driver.
"""
from __future__ import annotations

from functools import wraps
import math
import operator
from typing import Any


def _method(cls, name, function):
    function.__name__ = name
    function.__qualname__ = cls.__qualname__ + "." + name
    function.__module__ = cls.__module__
    setattr(cls, name, function)


def _nonnegative(value, name):
    value = float(value)
    if not math.isfinite(value) or value < 0:
        raise ValueError(name + " must be nonnegative and finite")
    return value


def _count(value, name):
    try:
        count = operator.index(value)
    except TypeError:
        raise TypeError(name + " must be an integer") from None
    if count <= 0:
        raise ValueError(name + " must be greater than zero")
    return count


def _point(g, point, name):
    if isinstance(point, g["Mobject"]):
        return point
    if point is None:
        raise TypeError(name + " must be a 3D point or a Mobject; got NoneType")
    np = g["_np"]
    try:
        coordinates = np.asarray(point, dtype=float)
    except (TypeError, ValueError) as error:
        raise TypeError(
            name + " must be a 3D point or a Mobject; got " + type(point).__name__
        ) from error
    if coordinates.shape != (3,) or not np.isfinite(coordinates).all():
        raise ValueError(name + " must be a Mobject or a finite 3D point")
    # A caller may mutate the supplied array/list while the effect runs.
    # Validate its coordinates without replacing that live input identity.
    return point



def _bind_root(animation, root):
    # This existing flag makes both the shared Scene.play wrapper and its
    # captured classifier prepare the root and owner context. Merely changing
    # _native_kind would not reach those older composition closures.
    animation.mobject = animation.group = root
    animation._composition_authored_root = True


def _install_flash(g):
    Flash = g["Flash"]

    def flash_init(self, point, color=g["_YELLOW"], line_length=0.2,
                   num_lines=12, flash_radius=0.3, line_stroke_width=3.0,
                   run_time=1.0, **kwargs):
        self.point = _point(g, point, "Flash point")
        self.color = color
        self.line_length = _nonnegative(line_length, "Flash line_length")
        self.num_lines = _count(num_lines, "Flash num_lines")
        self.flash_radius = _nonnegative(flash_radius, "Flash flash_radius")
        self.line_stroke_width = _nonnegative(line_stroke_width, "Flash line_stroke_width")
        self.lines = self.create_lines()
        if not isinstance(self.lines, g["Mobject"]):
            raise TypeError("Flash.create_lines must return a Mobject")
        # Both factories are public authoring hooks. In particular, a moving
        # point must not replace their products with a hard-coded reveal.
        super(Flash, self).__init__(
            *self.create_line_anims(), group=self.lines, run_time=run_time, **kwargs,
        )

    def create_lines(self):
        lines = g["VGroup"]()
        for index in range(self.num_lines):
            angle = index * math.tau / self.num_lines
            line = g["Line"](g["_ORIGIN"], self.line_length * g["_RIGHT"])
            line.shift((self.flash_radius - self.line_length) * g["_RIGHT"])
            line.rotate(angle, about_point=g["_ORIGIN"])
            lines.add(line)
        lines.set_stroke(color=self.color, width=self.line_stroke_width)
        # Follow the live point on the scene-updater phase, after child
        # interpolation, just like the Reference's group updater. Keep it on
        # the original group so copies of individual lines cannot steal it.
        lines.add_updater(lambda group: group.move_to(self.point))
        return lines

    def create_line_anims(self):
        return [g["ShowCreationThenDestruction"](line) for line in self.lines]

    def flash_clean_up(self, scene):
        super(Flash, self).clean_up_from_scene(scene)
        self.lines.update()


    Flash._native_kind = "animation_group"
    for name, function in {
        "__init__": flash_init, "create_lines": create_lines,
        "create_line_anims": create_line_anims,
        "clean_up_from_scene": flash_clean_up,
    }.items():
        _method(Flash, name, function)
    # The shared initializer froze a legacy leaf-style Flash lifecycle on
    # this class before installing AnimationGroup. Remove the entire frozen
    # protocol so normal MRO lookup reaches the group at every stage, including
    # later authored changes to AnimationGroup itself. Do not freeze aliases.
    for name in (
        "_ensure_runtime_defaults", "get_all_mobjects", "begin",
        "update_mobjects", "interpolate", "finish", "abort",
    ):
        if name in vars(Flash):
            delattr(Flash, name)
    if "interpolate_mobject" in vars(Flash):
        delattr(Flash, "interpolate_mobject")


def _install_lagged_map(g):
    Map = g["LaggedStartMap"]

    def map_init(self, anim_func, group, run_time=2.0, lag_ratio=0.05, **kwargs):
        if not callable(anim_func):
            raise TypeError(
                "LaggedStartMap requires an animation constructor to map; "
                "got " + type(anim_func).__name__
            )
        if not isinstance(group, g["Mobject"]):
            raise TypeError(
                "LaggedStartMap requires a Mobject family to map over; "
                "got " + type(group).__name__
            )
        # Let the group's own iterator control membership, as in the pinned
        # Reference: a factory may intentionally mutate the group. kwargs
        # belong to each child; run_time/lag_ratio belong to the group.
        animations = [anim_func(child, **kwargs) for child in group]
        super(Map, self).__init__(
            *animations, run_time=run_time, lag_ratio=lag_ratio, group=group,
        )

    _method(Map, "__init__", map_init)


def _install_rooted_effect(g, name, root_attribute, kind):
    """Keep the existing constructor/leaf factories; retain their real group."""
    cls = g.get(name)
    if cls is None:
        return
    original_init = cls.__init__

    @wraps(original_init)
    def initialize(self, *args, **kwargs):
        original_init(self, *args, **kwargs)
        if self.mobject is None and root_attribute is not None:
            root = getattr(self, root_attribute)
        else:
            # This also preserves explicit group/group_type arguments. For
            # FlashyFadeIn the root contains both the live object and outline;
            # the shared helper determines group type and de-duplicates them.
            root = g["_fmn_ensure_composition_root"](self)
        if not isinstance(root, g["Mobject"]):
            raise TypeError(name + " must retain a Mobject composition root")
        _bind_root(self, root)

    # With an explicit root the shared composition driver executes the exact
    # authored children. Normal group kinds also make play-level easing use
    # the live-rate protocol, not the obsolete special-case sampled lowering.
    cls._native_kind = kind
    _method(cls, "__init__", initialize)


def install_composite_effects(native: Any) -> None:
    """Install real composite protocols without replacing public class objects."""
    g = vars(native)
    if g.get("_FMN_COMPOSITE_EFFECTS_INSTALLED", False):
        return
    if "Flash" in g:
        _install_flash(g)
    if "LaggedStartMap" in g:
        _install_lagged_map(g)
    _install_rooted_effect(g, "Broadcast", "circles", "lagged_start")
    _install_rooted_effect(g, "ClockPassesTime", "clock", "animation_group")
    _install_rooted_effect(g, "FlashyFadeIn", None, "animation_group")
    g["_FMN_COMPOSITE_EFFECTS_INSTALLED"] = True
