"""Validated callable color fields over the existing native record schemas.

Point/surface records own `rgba`; vector records own `fill_rgba` and
`stroke_rgba`. No new geometry, interpolation, storage or renderer lives here.
Authored callback side effects are not rolled back, but failed field evaluation
never publishes an engine-authored partial family recoloring.
"""
from __future__ import annotations

from .invocation import InvocationGuard

_MAX_POINTS = 1_048_576
_COLOR_EDITS = InvocationGuard()


def _color_edit(members):
    return _COLOR_EDITS.hold(*members, message="callable color fields cannot reenter an active family")


def _colors(np, value, count, channels):
    array = np.asarray(value)
    if array.dtype.kind not in "biuf":
        raise TypeError("callable color fields must return real numeric colors")
    if array.shape in ((channels,), (1, channels)):
        array = np.broadcast_to(array.reshape((1, channels)), (count, channels))
    if array.shape != (count, channels):
        raise ValueError(f"callable color field must return ({count}, {channels}) colors")
    if not np.isfinite(array).all() or np.any(np.abs(array) > np.finfo(np.float32).max):
        raise ValueError("callable colors must be finite and f32-representable")
    # Own the result before another callback can mutate its aliased source.
    return np.array(array, dtype=np.float32, copy=True)


def install_functional_color(native):
    namespace = vars(native)
    if namespace.get("_FMN_FUNCTIONAL_COLOR_INSTALLED", False):
        return
    Mobject, np = namespace["Mobject"], namespace["_np"]

    def apply(self, function, recurse, opacity):
        if not callable(function):
            raise TypeError("color field must be callable")
        rgb = opacity is not None
        if rgb:
            alpha = _colors(np, [opacity], 1, 1)[0, 0]
        # get_family retains public override dispatch and existing family order.
        # Deduplicate shared children by identity, not their overloaded equality.
        members = tuple({id(mob): mob for mob in self.get_family(recurse)}.values())
        total = 0
        inputs = []
        for mob in members:
            if not mob.has_points():
                continue
            points = mob.get_points()
            total += len(points)
            if total > _MAX_POINTS:
                raise ValueError("callable color family exceeds its point budget")
            names = mob.data.dtype.names or ()
            fields = ("rgba",) if "rgba" in names else tuple(
                name for name in ("fill_rgba", "stroke_rgba") if name in names)
            if not fields:
                raise TypeError("mobject record schema has no native color field")
            for name in fields:
                if mob.data[name].shape != (len(points), 4):
                    raise ValueError("native color fields must contain four channels per point")
            inputs.append((mob, np.array(points, copy=True), fields))
        with _color_edit(members):
            plan = []
            for mob, before, fields in inputs:
                values = _colors(np, function(mob.get_points()), len(before), 3 if rgb else 4)
                if rgb:
                    rgba = np.empty((len(before), 4), dtype=np.float32)
                    rgba[:, :3], rgba[:, 3] = values, alpha
                else:
                    rgba = values
                plan.append((mob, before, fields, rgba))
            now = tuple({id(mob): mob for mob in self.get_family(recurse)}.values())
            if tuple(map(id, now)) != tuple(map(id, members)):
                raise RuntimeError("mobject family changed while evaluating its color field")
            for mob, before, fields, _ in plan:
                if not np.array_equal(mob.get_points(), before):
                    raise RuntimeError("mobject geometry changed while evaluating its color field")
                if any(mob.data[name].shape != (len(before), 4) for name in fields):
                    raise RuntimeError("mobject color layout changed during field evaluation")
            for mob, _, fields, rgba in plan:
                for name in fields:
                    # Preserve authored setter dispatch. Point/surface classes
                    # retain the legacy one-argument call; vector schemas name
                    # the two real paint columns rather than inventing `rgba`.
                    # A public setter may mutate its input. Each field owns
                    # a copy so the fill setter cannot corrupt the stroke plan.
                    if name == "rgba":
                        mob.set_rgba_array(rgba.copy())
                    else:
                        mob.set_rgba_array(rgba.copy(), name=name)
        return self

    def set_color_by_rgba_func(self, func, recurse=True):
        return apply(self, func, recurse, None)

    def set_color_by_rgb_func(self, func, opacity=1, recurse=True):
        # `None` is not an RGB alpha sentinel supplied by callers.
        if opacity is None:
            raise TypeError("RGB color opacity must be a real number")
        return apply(self, func, recurse, opacity)

    for name, method in (("set_color_by_rgba_func", set_color_by_rgba_func),
                         ("set_color_by_rgb_func", set_color_by_rgb_func)):
        method.__name__ = name
        method.__qualname__ = Mobject.__qualname__ + "." + name
        method.__module__ = Mobject.__module__
        setattr(Mobject, name, method)
    namespace["_FMN_FUNCTIONAL_COLOR_INSTALLED"] = True
