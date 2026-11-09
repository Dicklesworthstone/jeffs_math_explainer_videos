"""Wheel playback normalization over the existing Choreo execution boundary.

Builders produce real Animation objects before any native specification is
made. This module owns no interpolation, geometry, renderer, or frame clock.
"""

from __future__ import annotations

from functools import wraps
from typing import Any


def install_scene_playback(native: Any) -> None:
    """Keep authored builder products and Transform endpoint options live.

    The wheel calls this after the shared native animation initialization.
    Direct ExtensionFileLoader consumers may install it on their native module
    explicitly. Published class objects and qualified aliases stay identical.
    """
    g = vars(native)
    if g.get("_FMN_SCENE_PLAYBACK_INSTALLED", False):
        return
    _install_restore_playback(g)
    Scene, Animation = g["Scene"], g["Animation"]
    Builder, Transform = g["_AnimationBuilder"], g["Transform"]
    original_play = Scene.play
    original_requires = g["_requires_python_animation"]

    def requires_python_animation(animation):
        # Choreo's ordinary Transform lowering currently has no general
        # final-alpha/remover parameter application. Its shared Python
        # Transform protocol implements both, including replacement cleanup.
        # Specialized native kinds (notably FadeOut's final_alpha=0) have
        # their own defaults and must not be reclassified by this rule.
        if (isinstance(animation, Transform)
                and getattr(animation, "_native_kind", None)
                in {"transform", "replacement_transform", "transform_from_copy"}
                and (getattr(animation, "final_alpha_value", 1.0) != 1.0
                     or bool(getattr(animation, "remover", False)))):
            return True
        return original_requires(animation)

    @wraps(original_play)
    def play(self, *proto_animations, run_time=None, rate_func=None, lag_ratio=None):
        # CPython evaluates all .animate chains before this call. Build in
        # argument order, at the same dynamic-target lookup point as
        # prepare_animation, and never reconstruct an authored product.
        animations = []
        for proto in proto_animations:
            if isinstance(proto, Builder):
                animation = g["prepare_animation"](proto)
                if not isinstance(animation, Animation):
                    raise TypeError("AnimationBuilder.build must return an Animation")
                animations.append(animation)
            else:
                animations.append(proto)
        return original_play(self, *animations, run_time=run_time,
                             rate_func=rate_func, lag_ratio=lag_ratio)

    g["_requires_python_animation"] = requires_python_animation
    Scene.play = play
    if "CyclicReplace" in g:
        _install_cyclic_replace(g)
    _install_deferred_transforms(g)
    # The wheel exports the complete table; smaller embedding tables may
    # install only their available playback families. Missing exports remain
    # missing and are still the independent parity auditor's responsibility.
    if "Homotopy" in g:
        from fmn_python.movement import install_movement

        install_movement(native)
    if "Rotating" in g:
        from fmn_python.rotation import install_rotation

        install_rotation(native)
    if "WiggleOutThenIn" in g or "VShowPassingFlash" in g:
        from fmn_python.indication import install_indication

        install_indication(native)
    if "AnimationGroup" in g:
        from fmn_python.composite_effects import install_composite_effects

        install_composite_effects(native)
    if "MaintainPositionRelativeTo" in g:
        from fmn_python.update_animations import install_update_animations

        install_update_animations(native)
    if "ValueTracker" in g:
        from fmn_python.trackers import install_tracker_interpolation

        install_tracker_interpolation(native)
    if "_RATE_FUNC_NAMES" in g and "AnimationGroup" in g:
        from fmn_python.live_rates import install_live_rates

        install_live_rates(native)
    if "turn_animation_into_updater" in g and "cycle_animation" in g:
        from fmn_python.animation_updaters import install_animation_updaters

        install_animation_updaters(native)
    # Freeze the completed shipped protocols, after the family installers
    # above have supplied their own lifecycle methods. Authored subclasses
    # and later monkeypatches must not disappear into a native endpoint lerp.
    _install_transform_dispatch(g)
    g["_FMN_SCENE_PLAYBACK_INSTALLED"] = True


def _install_restore_playback(g: dict[str, Any]) -> None:
    """Restore the public Transform lineage and constructor-selected target."""
    Restore, Transform, Mobject = g["Restore"], g["Transform"], g["Mobject"]

    def restore_init(self, mobject, path_arc=0.0, path_arc_axis=g["_OUT"],
                     path_func=None, **kwargs):
        saved = getattr(mobject, "saved_state", None)
        if saved is None:
            raise Exception("Trying to restore without having saved")
        if not isinstance(saved, Mobject):
            raise TypeError("Restore saved_state must be a Mobject")
        if path_func is not None and not callable(path_func):
            raise TypeError("Restore path_func must be callable")
        callback_path = (path_func is not None
                         and getattr(path_func, "_fmn_path_arc", None) is None)
        # Capture the saved object by reference at construction, as the
        # Reference does. Rebinding mobject.saved_state later must not change
        # this animation's destination; edits to the saved object remain live.
        super(Restore, self).__init__(
            mobject, saved, path_arc=path_arc, path_arc_axis=path_arc_axis,
            path_func=None if callback_path else path_func, **kwargs,
        )
        # Shared initialization still guards the old native-only Restore
        # class. This installer has made it a real Transform: install the
        # validated authored path after ordinary endpoint initialization so
        # live-path dispatch uses the existing callback/camera driver.
        if callback_path:
            self.path_func = path_func

    # Keep every qualified import and existing subclass attached to the same
    # public class object. Native Transform owns ordinary restoration; the
    # shared Transform protocol owns authored paths, hooks and camera poses.
    # Neither route performs the legacy play-time saved_state relinking.
    Restore.__bases__ = (Transform,)
    Restore._native_kind = "transform"
    restore_init.__name__ = "__init__"
    restore_init.__qualname__ = Restore.__qualname__ + ".__init__"
    restore_init.__module__ = Restore.__module__
    Restore.__init__ = restore_init


def _install_cyclic_replace(g: dict[str, Any]) -> None:
    """Cycle a real group through Transform, not a targetless spec.

    The group retains the original operands. Its destination is built at
    begin, so edits between construction and playback remain observable.
    Copy, placement, alignment and interpolation use the existing Mobject
    and Transform implementations; there is no second animation kernel.
    """
    Cyclic = g["CyclicReplace"]
    Transform, Mobject, Group = g["Transform"], g["Mobject"], g["Group"]
    Swap = g.get("Swap")

    def cyclic_init(self, *mobjects, path_arc=1.5707963267948966, **kwargs):
        if len(mobjects) < 2:
            raise ValueError(
                type(self).__name__ + " needs at least two mobjects to cycle; got "
                + str(len(mobjects))
            )
        if not all(isinstance(mobject, Mobject) for mobject in mobjects):
            raise TypeError(type(self).__name__ + " cycles Mobjects only")
        super(Cyclic, self).__init__(Group(*mobjects), path_arc=path_arc, **kwargs)

    def create_target(self):
        source = self.mobject
        target = source.copy()
        # Reference direction: operand i moves to operand i+1; the last
        # returns to the first. Iterate the copied children in that order
        # without ever moving a source operand while planning destinations.
        if len(target):
            for destination, original in zip([target[-1], *target[:-1]], source):
                destination.move_to(original)
        return target

    Cyclic._native_kind = "transform"
    Cyclic._target_attr = "target_mobject"
    Cyclic.__init__ = cyclic_init
    Cyclic.create_target = create_target
    Cyclic._native_target = Transform._native_target
    Cyclic._native_params = Transform._native_params
    if Swap is not None:
        Swap._native_kind = "transform"
        Swap._target_attr = "target_mobject"
    for name in ("__init__", "create_target"):
        function = vars(Cyclic)[name]
        function.__name__ = name
        function.__qualname__ = Cyclic.__qualname__ + "." + name
        function.__module__ = Cyclic.__module__

    # A normal native Transform spec resolves _native_target while the whole
    # composition is being planned, before a preceding Succession member has
    # finished. Cycles instead sample the live group in Transform.begin. Use
    # the existing ordered Python-leaf driver for this deferred protocol,
    # including stock cycles; native record storage and Choreo's clock remain
    # authoritative. This also preserves Mobject.interpolate overrides.
    previous_requires = g["_requires_python_animation"]

    def requires(animation):
        return isinstance(animation, Cyclic) or previous_requires(animation)

    g["_requires_python_animation"] = requires


def _install_deferred_transforms(g: dict[str, Any]) -> None:
    """Evaluate authored target functions at begin, not during spec planning."""
    deferred = tuple(g[name] for name in (
        "ApplyMethod", "ApplyFunction", "ApplyPointwiseFunctionToCenter",
    ) if name in g)
    if deferred:
        previous_requires = g["_requires_python_animation"]

        def requires(animation):
            # Native specs capture their target while the entire composition
            # is lowered. These families instead construct it from live state
            # in Transform.begin, after preceding Succession leaves finish.
            # Do not inspect target_mobject: a reused animation retains its
            # previous target but must still sample again at the next begin.
            return isinstance(animation, deferred) or previous_requires(animation)

        g["_requires_python_animation"] = requires
    Complex = g.get("ApplyComplexFunction")
    if Complex is not None:
        np = g["_np"]

        def init_path_func(self):
            self.path_arc = float(np.log(complex(self.function(complex(1)))).imag)
            # Computing the angle alone leaves path_func=None, silently
            # turning a rotation into a straight interpolation. The shared
            # Transform path factory owns arc construction and respects an
            # already supplied path; retain cooperative subclass dispatch.
            super(Complex, self).init_path_func()

        init_path_func.__name__ = "init_path_func"
        init_path_func.__qualname__ = Complex.__qualname__ + ".init_path_func"
        init_path_func.__module__ = Complex.__module__
        Complex.init_path_func = init_path_func


def _install_transform_dispatch(g: dict[str, Any]) -> None:
    """Preserve authored object protocols on the existing callback boundary.

    The shared classifier's helper-only fast path returns before consulting
    its older lifecycle checker. Compare the completed shipped protocols here,
    including changed bases reached through super(), without running authored
    descriptors while deciding which execution route is valid.
    """
    from fmn_python.movement import _changed, _class_implementation, _implementation, _protocols

    Transform = g["Transform"]
    hooks = (
        "begin", "finish", "update_mobjects", "interpolate",
        "interpolate_mobject", "interpolate_submobject", "clean_up_from_scene",
        "create_target", "create_starting_mobject", "init_path_func",
        "check_target_mobject_validity", "get_all_mobjects",
        "get_all_families_zipped", "get_all_mobjects_to_update",
        "get_sub_alpha", "time_spanned_alpha", "_ensure_runtime_defaults",
        "__getattribute__", "__getattr__",
    )
    # Include Animation/_NativeAnimation: shipped wrappers call their live
    # base methods, so a late base patch must not disappear into native code.
    animation_root = g.get("Animation", Transform)
    protocols = _protocols(g, animation_root, hooks)
    # A stock Transform is not necessarily a stock interpolation: its source
    # can be an authored Mobject, including a child deep inside a stock Group.
    # Freeze after tracker/camera interpolation has been installed so those
    # shipping implementations keep their native route.
    Mobject = g["Mobject"]
    mobject_protocols = _protocols(g, Mobject, (
        # Native Transform owns all of these operations, not only the final
        # field lerp. An authored starting/target copy, alignment, updater or
        # release hook must execute through the shared Python lifecycle too.
        "copy", "is_aligned_with", "align_data_and_family", "align_family",
        "align_data", "align_points", "add_n_more_submobjects", "invisible_copy",
        "has_updaters", "lock_matching_data", "lock_data", "lock_uniforms",
        "unlock_data", "set_animating_status",
        "_is_updating_suspended", "suspend_updating", "resume_updating", "update",
        "interpolate", "get_family", "__getattribute__", "__getattr__",
    ))
    # submobjects is normally a per-instance _LiveSubmobjects, not a method.
    # Compare only its class descriptor; comparing its value to a class-level
    # baseline would incorrectly send every ordinary Mobject to callbacks.
    child_descriptors = {cls: _implementation(cls, "submobjects")
                         for cls in mobject_protocols}
    child_types = (list, tuple, g.get("_LiveSubmobjects"))
    previous_requires = g["_requires_python_animation"]
    reveal_types = tuple(g[name] for name in ("ShowPartial", "DrawBorderThenFill") if name in g)
    reveal_hooks = hooks + ("get_bounds", "get_outline")
    reveal_protocols = _protocols(g, animation_root, reveal_hooks)
    # Creation and writing use the same Animation copy/update/suspension
    # lifecycle. Writing additionally builds and styles a separate outline.
    partial_hooks = (
        "copy", "get_family", "set_animating_status", "_is_updating_suspended",
        "suspend_updating", "resume_updating", "update", "pointwise_become_partial",
        "__getattribute__", "__getattr__",
    )
    border_hooks = partial_hooks + (
        "align_data_and_family", "align_family", "align_data", "align_points",
        "add_n_more_submobjects", "invisible_copy", "set_fill", "set_stroke",
        "family_members_with_points", "get_stroke_width", "get_stroke_color",
        "get_color", "match_style", "set_data", "set_uniform", "get_uniforms",
        "interpolate", "refresh_joint_angles",
    )
    partial_protocols = _protocols(g, Mobject, partial_hooks)
    border_protocols = _protocols(g, Mobject, border_hooks)
    rotation_type = g.get("Rotating", ())
    # Rotating restores a starting copy before every absolute-angle rotation.
    # Checking rotate alone misses authored snapshots, point restoration and
    # the geometry/placement hooks called by the stock rotate implementation.
    rotation_hooks = partial_hooks + (
        "match_points", "rotate", "family_members_with_points", "get_points",
        "set_points", "apply_points_function", "apply_points_function_about_point",
        "get_bounding_box_point", "get_center", "get_bounding_box",
    )
    rotation_protocols = _protocols(g, Mobject, rotation_hooks)
    path_type = g.get("MoveAlongPath", ())
    path_object_hooks = partial_hooks + (
        "move_to", "shift", "get_center", "get_bounding_box_point",
        "get_bounding_box", "_bbox_rows", "_each_stage_target",
    )
    path_object_protocols = _protocols(g, Mobject, path_object_hooks)
    path_hooks = ("point_from_proportion", "_point_from_proportion", "has_points",
                  "get_num_points", "__getattribute__", "__getattr__")
    path_protocols = _protocols(g, g.get("VMobject", Mobject), path_hooks)

    def authored_family(root, object_protocols):
        # This is behavior admission, not interpolation's path-wise family
        # traversal. Deduplicate identities to bound deep/shared DAGs, and
        # inspect each member before reading its child-list descriptor. Never
        # call an authored get_family merely to decide how to execute it.
        # The walk runs no authored code, so one memo resolves each class's
        # hooks once per walk instead of once per member (fm-5wq.31).
        stack, seen, memo = [root], set(), {}
        while stack:
            member = stack.pop()
            marker = id(member)
            if marker in seen:
                continue
            seen.add(marker)
            if not isinstance(member, Mobject) or _changed(member, object_protocols, memo):
                return True
            expected = next(child_descriptors[cls] for cls in type(member).__mro__
                            if cls in child_descriptors)
            if _class_implementation(type(member), "submobjects", memo) is not expected:
                return True
            children = getattr(member, "submobjects", ())
            if type(children) not in child_types:
                # Do not invoke an authored iterable twice (once here and
                # once at begin) or consume a one-shot traversal at admission.
                return True
            stack.extend(children)
        return False

    def requires(animation):
        if isinstance(animation, Transform):
            if _changed(animation, protocols):
                return True
            # Inspect current endpoints on every lowering; families and
            # instance methods may change between plays of the same object.
            for name in ("mobject", "target_mobject"):
                root = getattr(animation, name, None)
                if isinstance(root, Mobject) and authored_family(root, mobject_protocols):
                    return True
        elif isinstance(animation, reveal_types):
            if _changed(animation, reveal_protocols):
                return True
            # mobject is ordinary instance state on the shipped protocols.
            # Never execute an authored descriptor just to classify its owner.
            if _implementation(type(animation), "mobject") is not None:
                return True
            object_protocols = (border_protocols if isinstance(
                animation, g.get("DrawBorderThenFill", ())) else partial_protocols)
            if authored_family(animation.mobject, object_protocols):
                return True
        elif isinstance(animation, rotation_type):
            if (_changed(animation, protocols)
                    or _implementation(type(animation), "mobject") is not None):
                return True
            if authored_family(animation.mobject, rotation_protocols):
                return True
        elif isinstance(animation, path_type):
            # Path and receiver are independent protocols: a stock sampler
            # does not license bypassing authored copies, updates or placement.
            # Inspect descriptors before older path classifiers read operands.
            if (_changed(animation, protocols) or any(
                    _implementation(type(animation), name) is not None
                    for name in ("mobject", "path"))):
                return True
            if (authored_family(animation.mobject, path_object_protocols)
                    or _changed(animation.path, path_protocols)):
                return True
        # Keep specialized native defaults and existing family capability
        # decisions. The guards above run BEFORE older classifiers which
        # traverse get_family or inspect methods through dynamic attributes.
        return previous_requires(animation)

    def finalize_protocols():
        # The complete initializer installs native surface alignment after
        # playback. Freeze its final shipping methods, not the temporary ones
        # seen here, or every stock Transform would become a Python callback.
        # Direct installers without later adapters keep the initial snapshot.
        nonlocal protocols, mobject_protocols, child_descriptors
        nonlocal reveal_protocols, partial_protocols, border_protocols, rotation_protocols
        nonlocal path_object_protocols, path_protocols
        protocols = _protocols(g, animation_root, hooks)
        names = next(iter(mobject_protocols.values())).keys()
        mobject_protocols = _protocols(g, Mobject, names)
        child_descriptors = {cls: _implementation(cls, "submobjects")
                             for cls in mobject_protocols}
        reveal_protocols = _protocols(g, animation_root, reveal_hooks)
        partial_protocols = _protocols(g, Mobject, partial_hooks)
        border_protocols = _protocols(g, Mobject, border_hooks)
        rotation_protocols = _protocols(g, Mobject, rotation_hooks)
        path_object_protocols = _protocols(g, Mobject, path_object_hooks)
        path_protocols = _protocols(g, g.get("VMobject", Mobject), path_hooks)

    g["_requires_python_animation"] = requires
    g["_fmn_finalize_transform_dispatch"] = finalize_protocols
