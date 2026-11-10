"""Authored rotation lifecycles over Choreo's existing native fast path.

Stock Rotating/Rotate remain native. Only behavior that Python can observe
through authored lifecycle or Mobject overrides selects the callback route.
"""
from __future__ import annotations

from functools import wraps
from typing import Any

from fmn_python.movement import (
    _cancel_preserving,
    _changed,
    _custom_rate,
    _install_lifecycle,
    _protocols,
)


def install_rotation(native: Any) -> None:
    """Restore Python-dispatch semantics without replacing public classes."""
    g = vars(native)
    if g.get("_FMN_ROTATION_INSTALLED", False):
        return
    Rotating = g.get("Rotating")
    if Rotating is None:
        return
    Animation, Mobject, Scene = g["Animation"], g["Mobject"], g["Scene"]
    if not isinstance(Rotating, type) or not issubclass(Rotating, Animation):
        raise TypeError("Rotating must be an Animation class")

    def prepare(self):
        if not callable(self.rate_func) and not isinstance(self.rate_func, str):
            raise TypeError("Rotating rate_func must be a callable or catalog name")

    _install_lifecycle(g, Rotating, prepare)

    hooks = (
        "begin", "finish", "interpolate", "interpolate_mobject",
        "interpolate_submobject", "update_mobjects", "create_starting_mobject",
        "get_all_mobjects", "get_all_families_zipped",
        "get_all_mobjects_to_update", "get_sub_alpha", "time_spanned_alpha",
        "clean_up_from_scene", "_ensure_runtime_defaults",
    )
    protocols = _protocols(g, Rotating, hooks)
    protocols.update({
        base: {name: getattr(base, name, None) for name in hooks}
        for base in Rotating.__mro__[1:]
        if isinstance(base, type) and issubclass(base, Animation)
    })
    object_hooks = (
        "rotate", "match_points", "family_members_with_points",
        "get_family", "set_animating_status", "suspend_updating",
        "resume_updating", "update", "__getattribute__", "__getattr__",
    )
    object_protocols = _protocols(g, Mobject, object_hooks)
    previous_requires, previous_play = g["_requires_python_animation"], Scene.play

    def object_changed(animation):
        # Classification must not call authored descriptors. Traverse the
        # already-materialized Python family projection from instance state.
        # No authored code runs in this walk: memoize per class (fm-5wq.31).
        stack, seen, memo = [animation.mobject], set(), {}
        while stack:
            member = stack.pop()
            marker = id(member)
            if marker in seen:
                continue
            seen.add(marker)
            if _changed(member, object_protocols, memo):
                return True
            try:
                children = vars(member).get("submobjects", ())
            except TypeError:
                return True
            stack.extend(reversed(tuple(children)))
        return False

    def requires(animation):
        if isinstance(animation, Rotating):
            if (
                getattr(animation, "_rotation_force_callback", False)
                or getattr(animation, "_movement_active", False)
                or getattr(animation, "final_alpha_value", 1.0) != 1.0
                or bool(getattr(animation, "remover", False))
                or _custom_rate(g, getattr(animation, "rate_func", None))
                or _changed(animation, protocols)
                or object_changed(animation)
            ):
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

        rotations, seen, visiting = [], set(), set()
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
            if isinstance(animation, Rotating):
                rotations.append(animation)
            if isinstance(animation, g["AnimationGroup"]):
                stack.extend((child, False) for child in reversed(animation.animations))

        absent, forced = object(), []
        rate = kwargs.get("rate_func")
        try:
            if _custom_rate(g, rate) and any(isinstance(a, Rotating) for a in animations):
                for animation in animations:
                    if isinstance(animation, Rotating):
                        forced.append((
                            animation,
                            animation.__dict__.get("_rotation_force_callback", absent),
                        ))
                        animation.__dict__["_rotation_force_callback"] = True
            return previous_play(self, *animations, **kwargs)
        except BaseException as error:
            for animation in reversed(rotations):
                _cancel_preserving(animation, error)
            raise
        finally:
            for animation, prior in reversed(forced):
                if prior is absent:
                    animation.__dict__.pop("_rotation_force_callback", None)
                else:
                    animation.__dict__["_rotation_force_callback"] = prior

    Scene.play = play
    g["_FMN_ROTATION_INSTALLED"] = True
