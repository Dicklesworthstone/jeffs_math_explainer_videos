"""Source-compatible stateful update animations on the callback boundary.

MaintainPositionRelativeTo captures its offset when the Animation object is
constructed, exactly as the pinned Reference does. The current native factory
captures that offset only when Scene.play lowers the animation, so the wheel
keeps the native class identity but executes this dependency through the shared
Python lifecycle. No second clock, geometry kernel, or updater mechanism lives
here.
"""
from __future__ import annotations

from functools import wraps
from typing import Any

from fmn_python.movement import _cancel_preserving, _install_lifecycle


def _method(cls, name, function):
    function.__name__ = name
    function.__qualname__ = cls.__qualname__ + "." + name
    function.__module__ = cls.__module__
    setattr(cls, name, function)


def install_update_animations(native: Any) -> None:
    g = vars(native)
    if g.get("_FMN_UPDATE_ANIMATIONS_INSTALLED", False):
        return
    Maintain = g.get("MaintainPositionRelativeTo")
    if Maintain is None:
        return
    Animation, Mobject, Scene = g["Animation"], g["Mobject"], g["Scene"]
    if not isinstance(Maintain, type) or not issubclass(Maintain, Animation):
        raise TypeError("MaintainPositionRelativeTo must be an Animation class")
    original_init = Maintain.__init__
    previous_requires, previous_play = g["_requires_python_animation"], Scene.play

    def maintain_init(self, mobject, tracked_mobject=None, **kwargs):
        original_init(self, mobject, tracked_mobject, **kwargs)
        # Reference update.py: capture exactly here, before either object can
        # move between Animation construction and Scene.play.
        if not (getattr(original_init, "_fmn_captures_camera_offset", False)
                and isinstance(mobject, g.get("CameraFrame", ()))):
            self.diff = mobject.get_center() - tracked_mobject.get_center()

    def prepare(self):
        tracked = self.tracked_mobject
        if not isinstance(tracked, Mobject):
            raise TypeError("MaintainPositionRelativeTo tracked_mobject must be a Mobject")
        owner = getattr(self.mobject, "_scene", None)
        tracked_owner = getattr(tracked, "_scene", None)
        if owner is not None and tracked_owner is not None and owner is not tracked_owner:
            error = g.get("_ForeignStageError", ValueError)
            raise error(
                "MaintainPositionRelativeTo cannot track a Mobject from another Scene; copy it"
            )

    def interpolate_mobject(self, alpha):
        del alpha
        target = self.tracked_mobject.get_center()
        location = self.mobject.get_center()
        self.mobject.shift(target - location + self.diff)

    _method(Maintain, "__init__", maintain_init)
    _install_lifecycle(g, Maintain, prepare)
    _method(Maintain, "interpolate_mobject", interpolate_mobject)

    def requires(animation):
        # Always callback: construction-time offset capture is observable even
        # with stock methods, while the native factory captures at lowering.
        if isinstance(animation, Maintain):
            return True
        return previous_requires(animation)

    g["_requires_python_animation"] = requires

    @wraps(previous_play)
    def play(self, *proto_animations, **kwargs):
        Builder = g.get("_AnimationBuilder")
        animations = []
        for proto in proto_animations:
            if Builder is not None and isinstance(proto, Builder):
                proto = g["prepare_animation"](proto)
                if not isinstance(proto, Animation):
                    raise TypeError("AnimationBuilder.build must return an Animation")
            animations.append(proto)
        maintains, seen, visiting = [], set(), set()
        stack = [(animation, False) for animation in reversed(animations)]
        while stack:
            animation, leaving = stack.pop()
            marker = id(animation)
            if leaving:
                visiting.remove(marker)
                continue
            if marker in visiting:
                raise ValueError("Animation composition contains a cycle")
            if marker in seen:
                continue
            seen.add(marker)
            visiting.add(marker)
            stack.append((animation, True))
            if isinstance(animation, Maintain):
                maintains.append(animation)
            if isinstance(animation, g["AnimationGroup"]):
                stack.extend((child, False) for child in reversed(animation.animations))
        try:
            return previous_play(self, *animations, **kwargs)
        except BaseException as error:
            for animation in reversed(maintains):
                _cancel_preserving(animation, error)
            raise

    Scene.play = play
    g["_FMN_UPDATE_ANIMATIONS_INSTALLED"] = True
