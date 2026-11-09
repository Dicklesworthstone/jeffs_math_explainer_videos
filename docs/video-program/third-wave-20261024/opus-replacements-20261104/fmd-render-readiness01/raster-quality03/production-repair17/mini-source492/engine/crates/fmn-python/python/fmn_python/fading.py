"""Authored fading protocols over native Mobject operations and Choreo timing.

The grouped crossfade keeps two independently shaped families. It interpolates
copies of each family, never aligns one glyph outline onto an unrelated one.
"""
from __future__ import annotations

from functools import wraps
from operator import index
from typing import Any


def _method(cls, name, function):
    function.__name__ = name
    function.__qualname__ = cls.__qualname__ + "." + name
    function.__module__ = cls.__module__
    setattr(cls, name, function)


def install_fading(native: Any) -> None:
    """Install on the existing wheel classes, preserving qualified aliases."""
    g = vars(native)
    if g.get("_FMN_FADING_INSTALLED", False):
        return
    Animation, Transform = g["Animation"], g["Transform"]
    FadeTransform, Pieces = g["FadeTransform"], g["FadeTransformPieces"]
    Mobject, Group, VMobject = g["Mobject"], g["Group"], g["VMobject"]
    original_play = g["Scene"].play

    def fade_init(self, mobject, target_mobject, stretch=True, dim_to_match=1, **kwargs):
        if not isinstance(mobject, Mobject) or not isinstance(target_mobject, Mobject):
            raise TypeError(
                "FadeTransform requires a source Mobject and a target Mobject; got "
                + type(mobject).__name__
                + " and "
                + type(target_mobject).__name__
            )
        dimension = index(dim_to_match)
        if dimension not in (0, 1, 2):
            raise ValueError("FadeTransform dim_to_match must be 0, 1, or 2")
        CameraFrame = g.get("CameraFrame")
        if CameraFrame is not None and any(
            isinstance(member, CameraFrame)
            for root in (mobject, target_mobject) for member in root.get_family()
        ):
            raise TypeError("FadeTransform crossfades drawable families, not camera frames")
        # A public save_state call is part of the pinned constructor protocol.
        # Retain the selected saved object: a later save_state must not change
        # which appearance this already-created transition restores.
        mobject.save_state()
        self._fade_source = mobject
        self._fade_saved_source = mobject.saved_state
        # Piece alignment inserts invisible siblings between existing children,
        # so restoring by aligned index can both leak padding and restore a
        # Circle into the wrong original Square. Retain the original graph,
        # not just its size, while the native record snapshot owns appearance.
        self._fade_source_topology = []
        seen = set()
        for member in mobject.get_family():
            if id(member) not in seen:
                seen.add(id(member))
                self._fade_source_topology.append((member, tuple(member.submobjects)))
        self.to_add_on_completion = target_mobject
        self.stretch, self.dim_to_match = bool(stretch), dimension
        self._fade_active = self._fade_finished = False
        self._fade_suspension = []
        self._fade_cleaned = False
        super(FadeTransform, self).__init__(Group(mobject, target_mobject.copy()), **kwargs)

    def release(self):
        prior, self._fade_suspension = self._fade_suspension, []
        first = None
        try:
            self.mobject.set_animating_status(False)
        except BaseException as error:
            first = error
        for member, was_suspended in prior:
            try:
                if not was_suspended and member._is_updating_suspended():
                    # Do not resume previously suspended descendants, and do
                    # not run an extra updater pass while unwinding a failure.
                    member.resume_updating(recurse=False, call_updater=False)
            except BaseException as error:
                if first is None:
                    first = error
        self.mobject_was_updating = False
        if first is not None:
            raise first

    def abort(self):
        if not self._fade_active:
            return
        self._fade_active = False
        release(self)

    def cancel_preserving(self):
        try:
            self.abort()
        except BaseException:
            pass

    def begin(self):
        if self._fade_active:
            self.abort()
        self._ensure_runtime_defaults()
        self.init_path_func()
        if self.time_span is not None:
            self.run_time = max(self.run_time, float(self.time_span[1]))
        self._fade_active, self._fade_finished, self._fade_cleaned = True, False, False
        self.mobject_was_updating = False
        try:
            self.mobject.set_animating_status(True)
            self.ending_mobject = self.mobject.copy()
            self.starting_mobject = self.create_starting_mobject()
            if not isinstance(self.starting_mobject, Mobject) or len(self.starting_mobject.submobjects) != 2:
                raise TypeError("FadeTransform starting mobject must contain source and target families")
            live_ids = {id(member) for member in self.mobject.get_family()}
            if any(id(member) in live_ids for member in self.starting_mobject.get_family()):
                raise ValueError("FadeTransform starting copy aliases its live animation family")
            start, end = self.starting_mobject, self.ending_mobject
            self.ghost_to(start[1], start[0])
            self.ghost_to(end[0], end[1])
            # Ordinary ghosting changes placement/style only. An authored
            # ghost may also refine its own geometry: reconcile each branch
            # across time, never source geometry against target geometry.
            start.align_data_and_family(end)
            self.mobject.align_data_and_family(start)
            self.mobject.align_data_and_family(end)
            if self.suspend_mobject_updating:
                seen = set()
                for member in self.mobject.get_family():
                    if id(member) not in seen:
                        seen.add(id(member))
                        self._fade_suspension.append((member, member._is_updating_suspended()))
                self.mobject_was_updating = not self.mobject._is_updating_suspended()
                self.mobject.suspend_updating()
            self.families = list(self.get_all_families_zipped())
            # Endpoints are ready before the sole alpha-zero interpolation;
            # nonzero rate(0) and authored hooks see the actual fade state.
            self.interpolate(0.0)
        except BaseException:
            cancel_preserving(self)
            raise

    def all_mobjects(self):
        return self.mobject, self.starting_mobject, self.ending_mobject

    def families_zipped(self):
        return Animation.get_all_families_zipped(self)

    def interpolate_submobject(self, current, start, end, alpha):
        current.interpolate(start, end, alpha, self.path_func)
        return self

    def finish(self):
        if not self._fade_active:
            if self._fade_finished:
                return
            raise RuntimeError("FadeTransform must begin before finish")
        try:
            self.interpolate(self.final_alpha_value)
            release(self)
        except BaseException:
            cancel_preserving(self)
            raise
        self._fade_active, self._fade_finished = False, True

    def cleanup(self, scene):
        # Succession cleans all children, including ones outside a shortened
        # final-alpha range. An unbegun child must not install its target.
        if not self._fade_finished or self._fade_cleaned:
            return
        scene.remove(self.mobject, self._fade_source)
        # Remove only the temporary alignment topology before assigning the
        # saved records. Preserve original proxy identities and shared-child
        # edges; trimming the padded list by length would select wrong pieces.
        for member, children in reversed(self._fade_source_topology):
            if hasattr(member, "set_submobjects"):
                member.set_submobjects(children)
            else:
                member.submobjects = list(children)
        self._fade_source.become(self._fade_saved_source)
        if not self.is_remover():
            scene.add(self.to_add_on_completion)
        self._fade_cleaned = True

    def pieces_init(self, mobject, target_mobject, **kwargs):
        for role, value in (("source", mobject), ("target", target_mobject)):
            if not isinstance(value, VMobject):
                raise TypeError(
                    "FadeTransformPieces requires a non-empty VMobject pair; "
                    + role
                    + " is "
                    + type(value).__name__
                )
            if not value.family_members_with_points():
                raise ValueError(
                    "FadeTransformPieces requires a non-empty VMobject pair; "
                    + role
                    + " has no points"
                )
        super(Pieces, self).__init__(mobject, target_mobject, **kwargs)

    def pieces_begin(self):
        # Family correspondence is required for every piece to ghost. Point
        # alignment between unlike source/target glyphs is neither needed nor
        # desirable; each piece fades independently with its own topology.
        self.mobject[0].align_family(self.mobject[1])
        super(Pieces, self).begin()

    # Retain all published identities; their inherited Transform interpolation
    # now runs rather than reducing the object to a native endpoint-only spec.
    FadeTransform.__bases__ = (Transform,)
    FadeTransform._native_kind = Pieces._native_kind = None
    FadeTransform.replace_mobject_with_target_in_scene = False
    for name, function in {
        "__init__": fade_init, "begin": begin, "finish": finish,
        "get_all_mobjects": all_mobjects, "get_all_families_zipped": families_zipped,
        "interpolate_submobject": interpolate_submobject,
        "clean_up_from_scene": cleanup, "abort": abort,
    }.items():
        _method(FadeTransform, name, function)
    _method(Pieces, "__init__", pieces_init)
    _method(Pieces, "begin", pieces_begin)

    @wraps(original_play)
    def play(self, *animations, **kwargs):
        fades, seen, visiting = [], set(), set()
        def visit(animation):
            if id(animation) in visiting:
                raise ValueError("Animation composition contains a cycle")
            if id(animation) in seen:
                return
            seen.add(id(animation))
            visiting.add(id(animation))
            if isinstance(animation, FadeTransform):
                fades.append(animation)
            if isinstance(animation, g["AnimationGroup"]):
                for child in animation.animations:
                    visit(child)
            visiting.remove(id(animation))
        for animation in animations:
            visit(animation)
        try:
            return original_play(self, *animations, **kwargs)
        except BaseException:
            for animation in reversed(fades):
                cancel_preserving(animation)
            raise

    g["Scene"].play = play
    _install_fade_effects(g)
    g["_FMN_FADING_INSTALLED"] = True


def _install_fade_effects(g):
    """Honor authored vector fades and geometric fade endpoint options."""
    Animation, Transform = g["Animation"], g["Transform"]
    VFadeIn, VFadeOut = g["VFadeIn"], g["VFadeOut"]
    FadeIn, FadeOut = g["FadeIn"], g["FadeOut"]
    VMobject = g["VMobject"]
    previous_requires, previous_play = g["_requires_python_animation"], g["Scene"].play

    def states(mobject):
        result, seen = [], set()
        for member in mobject.get_family():
            if id(member) not in seen:
                seen.add(id(member))
                result.append((member, member._is_updating_suspended()))
        return result

    def release(animation, prior):
        first = None
        try:
            animation.mobject.set_animating_status(False)
            if isinstance(animation, Transform):
                animation.mobject.unlock_data()
        except BaseException as error:
            first = error
        if animation.suspend_mobject_updating:
            for member, was_suspended in prior:
                try:
                    if not was_suspended and member._is_updating_suspended():
                        member.resume_updating(recurse=False, call_updater=False)
                except BaseException as error:
                    if first is None:
                        first = error
        animation.mobject_was_updating = False
        if first is not None:
            raise first

    def callback_rate(rate):
        if isinstance(rate, str):
            name = rate
            rate = next((function for function, label in g["_RATE_FUNC_NAMES"].items()
                         if label == name), None)
            if rate is None:
                raise ValueError("unknown rate function: " + name)
        if rate is not None and not callable(rate):
            raise TypeError("rate_func must be a callable or a catalog name")
        return rate

    def vector_defaults(self):
        Animation._ensure_runtime_defaults(self)
        self.rate_func = callback_rate(self.rate_func)

    def vector_abort(self):
        if not getattr(self, "_vector_fade_active", False):
            return
        self._vector_fade_active = False
        prior, self._vector_fade_prior = self._vector_fade_prior, []
        release(self, prior)

    def vector_begin(self):
        vector_abort(self)
        self._vector_fade_prior = states(self.mobject)
        self._vector_fade_active, self._vector_fade_finished = True, False
        try:
            Animation.begin(self)
        except BaseException:
            try:
                vector_abort(self)
            except BaseException:
                pass
            raise

    def vector_finish(self):
        if not getattr(self, "_vector_fade_active", False):
            if getattr(self, "_vector_fade_finished", False):
                return
            raise RuntimeError("Vector fade must begin before finish")
        # The shared finish owns final-alpha interpolation. Resume per member
        # below rather than reviving descendants suspended before this fade.
        self.mobject_was_updating = False
        try:
            Animation.finish(self)
            vector_abort(self)
        except BaseException:
            try:
                vector_abort(self)
            except BaseException:
                pass
            raise
        self._vector_fade_finished = True

    def vector_in(self, submob, start, alpha):
        # Match the native VFade kernel: the Reference getters read the first
        # opacity lane and setters broadcast it. Do not restore point/color/
        # width data, so concurrent geometry updaters remain live. Family lag
        # is per member, not a recursive opacity write from its parent.
        submob.set_stroke(opacity=float(alpha) * start.get_stroke_opacity(), recurse=False)
        submob.set_fill(opacity=float(alpha) * start.get_fill_opacity(), recurse=False)

    def vector_out(self, submob, start, alpha):
        super(VFadeOut, self).interpolate_submobject(submob, start, 1.0 - float(alpha))

    VFadeOut.__bases__ = (VFadeIn,)
    for name, function in {"begin": vector_begin, "finish": vector_finish,
                           "abort": vector_abort, "interpolate_submobject": vector_in,
                           "_ensure_runtime_defaults": vector_defaults}.items():
        _method(VFadeIn, name, function)
    _method(VFadeOut, "interpolate_submobject", vector_out)

    def fade_out_init(self, mobject, shift=g["_ORIGIN"], remover=True,
                      final_alpha_value=0.0, **kwargs):
        super(FadeOut, self).__init__(mobject, shift=shift, remover=remover,
                                     final_alpha_value=final_alpha_value, **kwargs)

    _method(FadeOut, "__init__", fade_out_init)
    hooks = ("begin", "finish", "interpolate", "interpolate_mobject", "interpolate_submobject",
             "create_starting_mobject", "get_all_mobjects", "get_all_families_zipped",
             "get_all_mobjects_to_update", "get_sub_alpha", "time_spanned_alpha",
             "update_mobjects", "clean_up_from_scene", "_ensure_runtime_defaults")
    object_hooks = ("set_stroke", "set_fill", "get_stroke_opacity", "get_fill_opacity")
    protocol_classes = {base for cls in tuple(g.values())
                        if isinstance(cls, type) and issubclass(cls, VFadeIn)
                        for base in cls.__mro__ if issubclass(base, Animation)}
    protocols = {cls: {name: getattr(cls, name) for name in hooks} for cls in protocol_classes}
    object_protocols = {cls: {name: getattr(cls, name) for name in object_hooks}
                        for cls in tuple(g.values()) if isinstance(cls, type) and issubclass(cls, VMobject)}

    def changed(obj, baselines):
        found = False
        for cls in type(obj).__mro__:
            baseline = baselines.get(cls)
            if baseline is None:
                continue
            if not found:
                found = True
                if any(getattr(getattr(obj, name), "__func__", getattr(obj, name)) is not method
                       for name, method in baseline.items()):
                    return True
            # A shipped override may delegate through super(): changing
            # VFadeIn must remain observable even on an unchanged VFadeOut.
            if any(getattr(cls, name) is not method for name, method in baseline.items()):
                return True
        return not found

    def requires(animation):
        kind = getattr(animation, "_native_kind", None)
        if isinstance(animation, VFadeIn):
            if kind == "v_fade_in" and animation.remover:
                return True
            if changed(animation, protocols) or any(
                changed(member, object_protocols) for member in animation.mobject.get_family()
            ):
                return True
        if isinstance(animation, FadeOut) and kind == "fade_out":
            if not animation.remover or animation.final_alpha_value != 0.0:
                return True
        elif isinstance(animation, FadeIn) and kind == "fade_in":
            if animation.remover or animation.final_alpha_value != 1.0:
                return True
        return previous_requires(animation)

    @wraps(previous_play)
    def play(self, *animations, **kwargs):
        selected, seen = [], set()
        def visit(animation):
            if id(animation) in seen:
                return
            seen.add(id(animation))
            if isinstance(animation, (VFadeIn, FadeIn, FadeOut)) and requires(animation):
                selected.append((animation, states(animation.mobject)))
            if isinstance(animation, g["AnimationGroup"]):
                for child in animation.animations:
                    visit(child)
        for animation in animations:
            visit(animation)
        for animation, _ in selected:
            animation.rate_func = callback_rate(animation.rate_func)
        if selected and "rate_func" in kwargs:
            kwargs["rate_func"] = callback_rate(kwargs["rate_func"])
        try:
            return previous_play(self, *animations, **kwargs)
        except BaseException:
            for animation, prior in reversed(selected):
                try:
                    if isinstance(animation, VFadeIn):
                        vector_abort(animation)
                    # Also cover an authored begin that fails outside super.
                    release(animation, prior)
                except BaseException:
                    pass
            raise

    g["_requires_python_animation"] = requires
    g["Scene"].play = play
