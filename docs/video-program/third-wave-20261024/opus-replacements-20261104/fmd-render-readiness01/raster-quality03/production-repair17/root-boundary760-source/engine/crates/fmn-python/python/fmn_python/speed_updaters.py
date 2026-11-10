"""Opt-in speed-aware updaters on an execution-owned Scene clock.

Ordinary updaters, animation helper copies, and other Scenes keep their normal
frame dt. Only callbacks registered with ChangeSpeed.add_updater use the child
clock delta, after the shared composition has interpolated the current frame.
No process-global dt, constructor-time ownership, callback probing or new loop.
"""
from __future__ import annotations

from functools import wraps
import inspect
import weakref


_KEY = "_fmn_speed_updater_clock"


class _Scope:
    def __init__(self, owner, scene):
        self.owner, self.scene = owner, scene
        self.previous = None
        self.dt = 0.0
        self.driving = 0
        self.closed = False

    def sample(self, value):
        if value is None or self.closed:
            return
        self.dt = 0.0 if self.previous is None else (
            value - self.previous
        ) * self.owner._speed_child_duration
        self.previous = value

    def close(self):
        self.closed = True
        if vars(self.scene).get(_KEY) is self:
            vars(self.scene).pop(_KEY)


def install_speed_updaters(native):
    g = vars(native)
    if g.get("_FMN_SPEED_UPDATERS_INSTALLED", False):
        return
    if not g.get("_FMN_SPEED_INSTALLED", False):
        raise ImportError("speed updater installation requires ChangeSpeed execution")
    import sys
    speed_mod = sys.modules.get("manimlib.animation.speed")
    ChangeSpeed = g.get("ChangeSpeed") or (getattr(speed_mod, "ChangeSpeed", None) if speed_mod else None)
    if ChangeSpeed is None:
        raise KeyError("ChangeSpeed")
    Mobject = g["Mobject"]
    previous = {name: getattr(ChangeSpeed, name) for name in (
        "__init__", "begin", "interpolate", "update_mobjects", "finish", "abort",
        "clean_up_from_scene",
    )}

    def release(self):
        scope = self.__dict__.pop("_speed_updater_scope", None)
        if scope is not None:
            scope.close()

    def scene_for(self):
        scene = getattr(self, "_composition_scene", None)
        if scene is not None:
            return scene
        root = g["_fmn_ensure_composition_root"](self)
        for member in root.get_family():
            scene = getattr(member, "_scene", None)
            if scene is not None:
                return scene
        raise RuntimeError("ChangeSpeed.begin requires a Scene; use Scene.play(animation)")

    def speed_init(self, anim=None, speedinfo=None, rate_func=None,
                   affects_speed_updaters=True, **kwargs):
        if not isinstance(affects_speed_updaters, bool):
            raise TypeError("affects_speed_updaters must be bool")
        previous["__init__"](self, anim, speedinfo, rate_func, **kwargs)
        self.affects_speed_updaters = affects_speed_updaters

    def begin(self):
        if getattr(self, "_composition_driver", None) is not None:
            self.abort()
        release(self)
        if self.affects_speed_updaters:
            scene = scene_for(self)
            if vars(scene).get(_KEY) is not None:
                raise RuntimeError(
                    "only one ChangeSpeed may affect speed updaters in a Scene at a time; "
                    "set affects_speed_updaters=False on other wrappers"
                )
            scope = _Scope(self, scene)
            vars(scene)[_KEY] = scope
            self._speed_updater_scope = scope
        try:
            return previous["begin"](self)
        except BaseException:
            release(self)
            raise

    def interpolate(self, alpha):
        scope = self.__dict__.get("_speed_updater_scope")
        if scope is not None:
            scope.driving += 1
        try:
            result = previous["interpolate"](self, alpha)
            if scope is not None:
                # Reuse the exact value computed by the real group driver;
                # never call an authored easing twice or forecast a frame.
                scope.sample(self._speed_curve.last_value)
            return result
        finally:
            if scope is not None:
                scope.driving -= 1

    def update_mobjects(self, dt):
        scope = self.__dict__.get("_speed_updater_scope")
        if scope is not None:
            scope.driving += 1
        try:
            return previous["update_mobjects"](self, dt)
        finally:
            if scope is not None:
                scope.driving -= 1

    def finalizer(name):
        @wraps(previous[name])
        def finalize(self, *args):
            try:
                return previous[name](self, *args)
            finally:
                # Keep the final sampled delta until actual finish, not until
                # alpha==1: scene updaters still need that emitted frame.
                release(self)
        return finalize

    def add_updater(mobject, update_function, index=None, call_updater=False):
        """Register a scene-object updater that follows the active child clock.

        With no active ChangeSpeed in this object's Scene, normal dt is used.
        One-argument updaters pass through unchanged. Copied helper objects use
        wall-clock dt; their updates precede interpolation and cannot consume
        this frame's child-clock delta. Zero-dt callbacks always remain zero.
        """
        if not isinstance(mobject, Mobject):
            raise TypeError("ChangeSpeed.add_updater requires a Mobject")
        if not callable(update_function):
            raise TypeError("ChangeSpeed.add_updater requires a callable")
        signature = inspect.signature(update_function)
        if "dt" not in signature.parameters:
            signature.bind(mobject)
            # The Reference Mobject.add_updater names this flag `call`.
            return mobject.add_updater(update_function, index=index, call=call_updater)
        signature.bind(mobject, 0.0)
        anchor = weakref.ref(mobject)

        @wraps(update_function)
        def update(current, dt):
            delta = dt
            if dt != 0 and current is anchor():
                scene = getattr(current, "_scene", None)
                scope = vars(scene).get(_KEY) if scene is not None else None
                if (isinstance(scope, _Scope) and not scope.closed
                        and not scope.driving and scope.previous is not None):
                    delta = scope.dt
            return update_function(current, delta)

        return mobject.add_updater(update, index=index, call=call_updater)

    methods = {
        "__init__": speed_init, "begin": begin, "interpolate": interpolate,
        "update_mobjects": update_mobjects, "finish": finalizer("finish"),
        "abort": finalizer("abort"), "clean_up_from_scene": finalizer("clean_up_from_scene"),
        "add_updater": add_updater,
    }
    for name, function in methods.items():
        function.__name__ = name
        function.__qualname__ = ChangeSpeed.__qualname__ + "." + name
        function.__module__ = ChangeSpeed.__module__
        setattr(ChangeSpeed, name, staticmethod(function) if name == "add_updater" else function)
    g["_FMN_SPEED_UPDATERS_INSTALLED"] = True
