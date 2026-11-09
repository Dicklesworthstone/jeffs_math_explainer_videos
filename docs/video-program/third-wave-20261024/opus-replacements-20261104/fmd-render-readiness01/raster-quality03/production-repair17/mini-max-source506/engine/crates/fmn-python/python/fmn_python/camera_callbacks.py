"""Authored camera animations on the existing Choreo camera callback track.

CameraFrame is a pose, not a drawable Stage root. Admit normal Animation and
UpdateFromFunc/UpdateFromAlphaFunc lifecycles without adopting that pose or
creating another clock. Admission is scoped to playback/driver construction;
no private opt-in is required from authored scenes or retained on animations.
"""
from __future__ import annotations

from contextlib import contextmanager
from functools import wraps
from typing import Any


_KEY = "_fmn_allow_camera_callback"
_MISSING = object()


def _camera_callbacks(g, scene, animations):
    """Validate the entire known graph before any callback begins or is marked."""
    Camera, Group, Transform = g["CameraFrame"], g["AnimationGroup"], g["Transform"]
    supported = (Transform, *g.get("_fmn_camera_motion_types", ()))
    callbacks, seen, visiting = [], set(), set()
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
        if isinstance(animation, Group):
            stack.extend((child, False) for child in reversed(animation.animations))
        if not isinstance(getattr(animation, "mobject", None), Camera):
            continue
        if animation.mobject is not scene.frame:
            raise ValueError("Camera animation must target this Scene.frame")
        if (getattr(animation, "remover", False)
                or getattr(animation, "replace_mobject_with_target_in_scene", False)):
            raise NotImplementedError(
                "Camera animation cannot remove or replace the scene's camera identity"
            )
        native_kind = getattr(animation, "_native_kind", None)
        if native_kind and not isinstance(animation, supported):
            raise NotImplementedError(
                type(animation).__name__ + " has no camera-pose animation protocol; "
                "use Transform, frame.animate or a custom Animation callback"
            )
        if isinstance(animation, Transform):
            if animation._target_attr is None:
                raise NotImplementedError("A camera Transform requires a camera target")
            target = getattr(animation, "target_mobject", None)
            if target is not None and not isinstance(target, Camera):
                raise TypeError("Camera Transform target must be a CameraFrame")
        # A live camera has no Stage owner. Comparing camera._scene with a
        # helper would miss cross-scene paths and tracked objects entirely.
        for name, attribute in (("MoveAlongPath", "path"),
                                ("MaintainPositionRelativeTo", "tracked_mobject")):
            cls = g.get(name)
            if cls is not None and isinstance(animation, cls):
                for member in getattr(animation, attribute).get_family():
                    owner = getattr(member, "_scene", None)
                    if owner is not None and owner is not scene:
                        error = g.get("_ForeignStageError", ValueError)
                        raise error("Camera motion cannot reference a " + attribute
                                    + " from another Scene; copy it")
        if not native_kind:
            callbacks.append(animation)
    return callbacks


@contextmanager
def _admit(g, scene, animations):
    callbacks = _camera_callbacks(g, scene, animations)
    saved = []
    try:
        for animation in callbacks:
            attrs = vars(animation)
            saved.append((attrs, attrs.get(_KEY, _MISSING)))
            attrs[_KEY] = True
        yield
    finally:
        for attrs, previous in reversed(saved):
            if previous is _MISSING:
                attrs.pop(_KEY, None)
            else:
                attrs[_KEY] = previous


def install_camera_callbacks(native: Any) -> None:
    """Enable callbacks without replacing public classes or authored receivers."""
    g = vars(native)
    if g.get("_FMN_CAMERA_CALLBACKS_INSTALLED", False):
        return
    Scene, Animation = g["Scene"], g["Animation"]
    previous_play = Scene.play
    previous_driver = g["_fmn_make_camera_driver"]

    @wraps(previous_play)
    def play(self, *proto_animations, **kwargs):
        animations = []
        for proto in proto_animations:
            animation = g["prepare_animation"](proto)
            if not isinstance(animation, Animation):
                raise TypeError("prepare_animation must return an Animation")
            animations.append(animation)
        with _admit(g, self, animations):
            return previous_play(self, *animations, **kwargs)

    def make_driver(scene, animation):
        # Composition and persistent-animation drivers also use this factory.
        # Validate here too: nested authored begin hooks can create new leaves,
        # and a caller must never use the scoped flag to evade scene ownership.
        with _admit(g, scene, [animation]):
            return previous_driver(scene, animation)

    Scene.play = play
    g["_fmn_make_camera_driver"] = make_driver
    g["_FMN_CAMERA_CALLBACKS_INSTALLED"] = True
