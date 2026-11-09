"""Indication callbacks over Marionette records and Choreo's shared lifecycle.

Unmodified wiggles and width flashes keep their native execution kernels.
Direct use, authored hooks and persistent updaters use the same public methods;
this module owns neither a frame clock nor a renderer.
"""
from __future__ import annotations

from functools import wraps
import math
from typing import Any

from fmn_python.movement import (
    _cancel_preserving, _changed, _custom_rate, _install_lifecycle,
    _method, _protocols,
)


def _finite(value, name):
    value = float(value)
    if not math.isfinite(value):
        raise ValueError(name + " must be finite")
    return value


def _install_wiggle(g, Wiggle):
    def prepare(self):
        for name in ("scale_value", "rotation_angle", "n_wiggles"):
            _finite(getattr(self, name), "WiggleOutThenIn." + name)
        for name in ("scale_about_point", "rotate_about_point"):
            value = getattr(self, name)
            if value is not None:
                point = g["_np"].asarray(value, dtype=float)
                if point.shape != (3,) or not g["_np"].isfinite(point).all():
                    raise ValueError("WiggleOutThenIn." + name + " must be a finite 3D point")

    def interpolate_submobject(self, submobject, starting_submobject, alpha):
        # Absolute, rather than cumulative, geometry. The public getters and
        # Mobject operations deliberately remain virtual for authored scenes.
        alpha = _finite(alpha, "WiggleOutThenIn alpha")
        submobject.match_points(starting_submobject)
        swell = g["there_and_back"](alpha)
        submobject.scale(
            (1.0 - swell) + swell * self.scale_value,
            about_point=self.get_scale_about_point(),
        )
        submobject.rotate(
            g["wiggle"](alpha, self.n_wiggles) * self.rotation_angle,
            about_point=self.get_rotate_about_point(),
        )

    _method(Wiggle, "interpolate_submobject", interpolate_submobject)
    _install_lifecycle(g, Wiggle, prepare)
    return prepare


def _install_width_flash(g, Flash):
    np, VMobject = g["_np"], g["VMobject"]

    def validate(self):
        width = _finite(self.time_width, "VShowPassingFlash.time_width")
        _finite(self.taper_width, "VShowPassingFlash.taper_width")
        if width <= 0 or width / 6.0 == 0:
            raise ValueError("VShowPassingFlash.time_width must have a positive representable sigma")
        if not isinstance(self.mobject, VMobject):
            raise TypeError("VShowPassingFlash requires a VMobject")

    def flash_init(self, vmobject, time_width=0.3, taper_width=0.05,
                   remover=True, **kwargs):
        self.time_width, self.taper_width = float(time_width), float(taper_width)
        super(Flash, self).__init__(vmobject, remover=remover, **kwargs)
        validate(self)

    def prepare(self):
        validate(self)
        profiles, originals, seen = {}, [], set()
        for member in self.mobject.get_family():
            if id(member) in seen:
                continue
            seen.add(id(member))
            if not isinstance(member, VMobject):
                raise TypeError("VShowPassingFlash family members must be VMobjects")
            widths = np.asarray(member.get_stroke_widths(), dtype=float).reshape(-1).copy()
            if not np.isfinite(widths).all():
                raise ValueError("VShowPassingFlash stroke widths must be finite")
            xs = np.linspace(0.0, 1.0, len(widths))
            taper = np.array([_finite(self.taper_kernel(float(x)), "taper_kernel result")
                              for x in xs])
            profile = widths * taper
            if not np.isfinite(profile).all():
                raise ValueError("VShowPassingFlash tapered widths must be finite")
            profiles[hash(member)] = profile
            originals.append((member, widths))
        # No style writes occur until every profile has been admitted. Expose
        # the Reference's public profile map, including point-free roots.
        self.submob_to_widths = profiles
        self._flash_original_widths = originals

    def interpolate_submobject(self, submobject, starting_submobject, alpha):
        del starting_submobject
        alpha = _finite(alpha, "VShowPassingFlash alpha")
        widths = self.submob_to_widths[hash(submobject)]
        if not len(widths):
            return
        sigma = self.time_width / 6.0
        mu = (1.0 - alpha) * (-self.time_width / 2.0) + alpha * (1.0 + self.time_width / 2.0)
        xs = np.linspace(0.0, 1.0, len(widths))
        support = np.abs(xs - mu) <= 3.0 * sigma
        out = np.zeros(len(widths))
        # Evaluate only the compact support, avoiding overflow in squared
        # distances for narrow but valid windows. Native records own writes.
        zs = (xs[support] - mu) / sigma
        out[support] = widths[support] * np.exp(-0.5 * zs * zs)
        submobject.set_stroke(width=out, recurse=False)

    _method(Flash, "__init__", flash_init)
    _method(Flash, "interpolate_submobject", interpolate_submobject)
    _install_lifecycle(g, Flash, prepare)
    lifecycle_finish, lifecycle_abort = Flash.finish, Flash.abort

    def abort(self):
        originals = getattr(self, "_flash_original_widths", ())
        self._flash_original_widths = ()
        first = None
        for member, widths in originals:
            try:
                if len(widths):
                    member.set_stroke(width=widths, recurse=False)
            except BaseException as error:
                if first is None:
                    first = error
        try:
            lifecycle_abort(self)
        except BaseException as error:
            if first is None:
                first = error
        if first is not None:
            raise first

    def finish(self):
        if (getattr(self, "_movement_finished", False)
                and not getattr(self, "_flash_original_widths", ())):
            return
        try:
            lifecycle_finish(self)
            for member, start in self.get_all_families_zipped():
                member.match_style(start)
            self._flash_original_widths = ()
        except BaseException as error:
            _cancel_preserving(self, error)
            raise

    _method(Flash, "abort", abort)
    _method(Flash, "finish", finish)
    return validate


def install_indication(native: Any) -> None:
    """Install existing exported classes in place, preserving qualified imports."""
    g = vars(native)
    if g.get("_FMN_INDICATION_INSTALLED", False):
        return
    deferred = _install_transform_indications(g)
    wave_installed = _install_wave(g)
    families, validators, default_removers = [], {}, {}
    for name, installer, remover in (
        ("WiggleOutThenIn", _install_wiggle, False),
        ("VShowPassingFlash", _install_width_flash, True),
    ):
        cls = g.get(name)
        if cls is not None:
            validators[cls] = installer(g, cls)
            families.append(cls)
            default_removers[cls] = remover
    if not families and not deferred:
        if wave_installed:
            g["_FMN_INDICATION_INSTALLED"] = True
        return
    families = tuple(families)
    Animation, Scene, Mobject = g["Animation"], g["Scene"], g["Mobject"]
    hooks = (
        "begin", "finish", "interpolate", "interpolate_mobject",
        "interpolate_submobject", "update_mobjects", "create_starting_mobject",
        "get_all_mobjects", "get_all_families_zipped", "get_all_mobjects_to_update",
        "get_sub_alpha", "time_spanned_alpha", "clean_up_from_scene",
        "_ensure_runtime_defaults", "taper_kernel", "get_scale_about_point",
        "get_rotate_about_point", "__getattribute__", "__getattr__",
    )
    protocols = _protocols(g, Animation, hooks)
    object_protocols = _protocols(g, Mobject, (
        "copy", "get_family", "get_center", "match_points", "scale", "rotate",
        "get_stroke_widths", "set_stroke", "match_style", "update",
        "set_animating_status", "suspend_updating", "resume_updating",
        "__getattribute__", "__getattr__",
    ))
    previous_requires, previous_play = g["_requires_python_animation"], Scene.play

    def requires(animation):
        if isinstance(animation, families):
            default = next(value for cls, value in default_removers.items()
                           if isinstance(animation, cls))
            if (getattr(animation, "_indication_force_callback", False)
                    or getattr(animation, "_movement_active", False)
                    or animation.final_alpha_value != 1.0
                    or bool(animation.remover) != default
                    or _custom_rate(g, animation.rate_func)
                    or _changed(animation, protocols)):
                return True
            Wiggle = g.get("WiggleOutThenIn")
            if (Wiggle is not None and isinstance(animation, Wiggle)
                    and (animation.scale_about_point is not None
                         or animation.rotate_about_point is not None)):
                # Public pivot getters observe the scale-then-rotate order.
                # The native stock kernel snapshots both pivots beforehand.
                return True
            # The walk reads instance state only: memoize per class (fm-5wq.31).
            stack, seen, memo = [animation.mobject], set(), {}
            while stack:
                member = stack.pop()
                if id(member) in seen:
                    continue
                seen.add(id(member))
                if _changed(member, object_protocols, memo):
                    return True
                stack.extend(reversed(tuple(vars(member).get("submobjects", ()))))
        return previous_requires(animation)

    g["_requires_python_animation"] = requires

    @wraps(previous_play)
    def play(self, *proto_animations, **kwargs):
        Builder = g.get("_AnimationBuilder")
        animations = []
        for animation in proto_animations:
            if Builder is not None and isinstance(animation, Builder):
                animation = g["prepare_animation"](animation)
                if not isinstance(animation, Animation):
                    raise TypeError("AnimationBuilder.build must return an Animation")
            animations.append(animation)
        targets, deferred_targets, seen, visiting = [], [], set(), set()
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
            if isinstance(animation, deferred):
                deferred_targets.append(animation)
            if isinstance(animation, families):
                targets.append(animation)
                for cls, validate in validators.items():
                    if isinstance(animation, cls):
                        validate(animation)
                        break
            if isinstance(animation, g["AnimationGroup"]):
                stack.extend((child, False) for child in reversed(animation.animations))
        forced, absent = [], object()
        try:
            if _custom_rate(g, kwargs.get("rate_func")):
                # A global authored curve must remain live for deferred
                # leaves inside groups, not only for direct play arguments.
                candidates = [a for a in animations if isinstance(a, families)]
                for animation in [*candidates, *deferred_targets]:
                    if isinstance(animation, families + deferred):
                        forced.append((animation, animation.__dict__.get("_indication_force_callback", absent)))
                        animation.__dict__["_indication_force_callback"] = True
                if animations and all(g["_requires_python_animation"](animation)
                                      for animation in animations):
                    # An entirely callback-driven play has no consumer for
                    # the bootstrap's sampled global rate table. Keep the
                    # original callable live, without speculative probes.
                    for animation in animations:
                        animation.rate_func = kwargs["rate_func"]
                    kwargs = dict(kwargs, rate_func=None)
            return previous_play(self, *animations, **kwargs)
        except BaseException as error:
            for animation in reversed(targets):
                _cancel_preserving(animation, error)
            raise
        finally:
            for animation, prior in reversed(forced):
                if prior is absent:
                    animation.__dict__.pop("_indication_force_callback", None)
                else:
                    animation.__dict__["_indication_force_callback"] = prior

    Scene.play = play
    g["_FMN_INDICATION_INSTALLED"] = True


def _install_transform_indications(g):
    """Use Choreo's begin-time target recipes unless public hooks need Python.

    Grow, Indicate and TurnInsideOut now freeze their native targets at begin,
    including in Succession and replay. Blanket callback routing would instead
    run copied host updaters as well as the live scene updater. Keep the native
    path for unchanged effects without changing Mobject's copy semantics.
    """
    families = tuple(g[name] for name in (
        "GrowFromPoint", "Indicate", "TurnInsideOut",
    ) if name in g)
    if not families:
        return families
    protocols = _protocols(g, g["Animation"], (
        "begin", "finish", "abort", "interpolate", "interpolate_mobject",
        "interpolate_submobject", "create_target", "create_starting_mobject",
        "check_target_mobject_validity", "init_path_func", "update_mobjects",
        "get_all_mobjects", "get_all_families_zipped", "get_all_mobjects_to_update",
        "get_sub_alpha", "time_spanned_alpha", "clean_up_from_scene",
        "_ensure_runtime_defaults", "__getattribute__", "__getattr__",
    ))
    object_protocols = _protocols(g, g["Mobject"], (
        "copy", "get_family", "get_center", "scale", "move_to", "shift",
        "set_color", "reverse_points", "interpolate", "align_data_and_family",
        "is_aligned_with", "lock_matching_data", "unlock_data", "update",
        "set_animating_status", "suspend_updating", "resume_updating",
        "__getattribute__", "__getattr__",
    ))
    previous_requires = g["_requires_python_animation"]

    def requires(animation):
        if isinstance(animation, families):
            if (getattr(animation, "_indication_force_callback", False)
                    or getattr(animation, "final_alpha_value", 1.0) != 1.0
                    or getattr(animation, "remover", False)
                    or _custom_rate(g, getattr(animation, "rate_func", None))
                    or _changed(animation, protocols)):
                return True
            # Native Indicate owns a straight path, unlike Grow/TurnInsideOut
            # whose native specifications carry an explicit arc.
            Indicate = g.get("Indicate")
            if (Indicate is not None and isinstance(animation, Indicate)
                    and getattr(animation, "path_arc", 0.0) != 0.0):
                return True
            # The walk reads instance state only: memoize per class (fm-5wq.31).
            stack, seen, memo = [animation.mobject], set(), {}
            while stack:
                member = stack.pop()
                if id(member) in seen:
                    continue
                seen.add(id(member))
                if _changed(member, object_protocols, memo):
                    return True
                stack.extend(reversed(tuple(vars(member).get("submobjects", ()))))
        # Preserve custom paths, vector trackers and other subsystem decisions.
        # A target_mobject retained by replay is deliberately not a routing flag.
        return previous_requires(animation)

    g["_requires_python_animation"] = requires
    return families


def _install_wave(g):
    """Restore ApplyWave's Homotopy lineage and construction-time wave map."""
    Wave = g.get("ApplyWave")
    if Wave is None:
        return False
    Homotopy, Mobject, np = g["Homotopy"], g["Mobject"], g["_np"]

    def wave_init(self, mobject, direction=g["_UP"], amplitude=0.2,
                  run_time=1.0, **kwargs):
        if not isinstance(mobject, Mobject):
            raise TypeError("ApplyWave requires a Mobject")
        left = _finite(mobject.get_left()[0], "ApplyWave left extent")
        right = _finite(mobject.get_right()[0], "ApplyWave right extent")
        extent = right - left
        if not math.isfinite(extent) or extent <= 0:
            raise ValueError("ApplyWave requires a finite nonzero horizontal extent")
        direction = np.asarray(direction, dtype=float)
        if direction.shape != (3,) or not np.isfinite(direction).all():
            raise ValueError("ApplyWave.direction must be a finite 3D vector")
        amplitude = _finite(amplitude, "ApplyWave.amplitude")
        vector = amplitude * direction
        if not np.isfinite(vector).all():
            raise ValueError("ApplyWave displacement must be finite")

        def homotopy(x, y, z, t):
            point = np.array([x, y, z], dtype=float)
            # The endpoints are identity maps, including after a large live
            # translation beyond the original construction-time envelope.
            if t == 0.0 or t == 1.0:
                return point
            with np.errstate(over="raise", invalid="raise", divide="raise"):
                try:
                    power = np.exp(2.0 * ((x - left) / extent - 0.5))
                    phase = np.power(t, power)
                except FloatingPointError as error:
                    raise ValueError("ApplyWave phase is outside the finite real domain") from error
            return point + g["there_and_back"](float(phase)) * vector

        self.direction, self.amplitude = tuple(direction), amplitude
        super(Wave, self).__init__(homotopy, mobject, run_time=run_time, **kwargs)

    # Keep every qualified alias and already-existing subclass attached to
    # the original class object. Homotopy delegates all geometry writes to
    # Mobject.apply_function and runs on the shared frame/family lifecycle.
    Wave.__bases__ = (Homotopy,)
    Wave._native_kind = None
    _method(Wave, "__init__", wave_init)
    return True
