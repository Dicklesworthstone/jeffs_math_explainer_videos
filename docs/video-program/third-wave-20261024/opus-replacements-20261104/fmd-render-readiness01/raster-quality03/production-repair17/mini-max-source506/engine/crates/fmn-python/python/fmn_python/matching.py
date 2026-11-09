"""Public matching plans executed by the shared Choreo composition lifecycle.

The native shape predicate and Scribe span maps remain authoritative. This
adapter owns factory/override dispatch and piece claiming, not interpolation,
record alignment, animation timing, or a second renderer. Legacy native spec
metadata remains inspectable, but playback must execute the authored plan.
"""
from __future__ import annotations

from itertools import islice
import inspect
import types
from typing import Any

_MAX_PARTS = 65_536
_MAX_COMPARISONS = 1_048_576


def _bounded(values, name):
    result = list(islice(iter(values), _MAX_PARTS + 1))
    if len(result) > _MAX_PARTS:
        raise ValueError(name + " exceeds the 65536-part budget")
    return result


def _unique(values):
    result, seen = [], set()
    for value in values:
        if id(value) not in seen:
            seen.add(id(value))
            result.append(value)
    return result


def _pieces(mobject):
    return _unique(_bounded(mobject.family_members_with_points(), "matching family"))


def _pairs(values, mobject_type, name):
    pairs = []
    for pair in _bounded(values, name):
        try:
            pair = tuple(islice(iter(pair), 3))
        except TypeError:
            raise TypeError(name + " must pair Mobjects") from None
        if len(pair) != 2 or not all(isinstance(obj, mobject_type) for obj in pair):
            raise TypeError(name + " must pair Mobjects")
        pairs.append(pair)
    return pairs


def _validate_members(pairs, source_pieces, target_pieces):
    allowed = [{id(member) for member in pieces} for pieces in (source_pieces, target_pieces)]
    for pair in pairs:
        for side, member in enumerate(pair):
            if any(id(piece) not in allowed[side] for piece in _pieces(member)):
                raise ValueError("matching pair contains a foreign " + ("source" if side == 0 else "target") + " family member")


def _validate_owners(objects, scene=None, error_type=ValueError):
    for obj in objects:
        for member in _bounded(obj.get_family(), "matching family"):
            owner = getattr(member, "_scene", None)
            if owner is not None:
                if scene is not None and owner is not scene:
                    raise error_type("matching transforms cannot reference multiple Scenes; copy the mobjects")
                scene = owner


def _bind(cls, name, function):
    function.__name__ = name
    function.__qualname__ = cls.__qualname__ + "." + name
    function.__module__ = cls.__module__
    setattr(cls, name, function)


def _implementation(obj, name):
    value = inspect.getattr_static(obj, name, None)
    if isinstance(value, (staticmethod, classmethod, types.MethodType)):
        return value.__func__
    return value


def _changed_protocol(obj, protocols):
    nearest = True
    for cls in type(obj).__mro__:
        baseline = protocols.get(cls)
        if baseline is None:
            continue
        if nearest:
            nearest = False
            if any(_implementation(obj, name) is not expected for name, expected in baseline.items()):
                return True
        # An unchanged shipped override can still delegate to a patched base.
        if any(_implementation(cls, name) is not expected for name, expected in baseline.items()):
            return True
    return False


def install_matching(native: Any) -> None:
    """Activate real public planning without replacing exported class objects."""
    g = vars(native)
    if g.get("_FMN_MATCHING_INSTALLED", False):
        return
    Parts, Group = g["TransformMatchingParts"], g["AnimationGroup"]
    Mobject, Animation, Transform = g["Mobject"], g["Animation"], g["Transform"]

    def matching_init(self, source, target, matched_pairs=(), match_animation=Transform,
                      mismatch_animation=Transform, run_time=2, lag_ratio=0, **kwargs):
        if not isinstance(source, Mobject) or not isinstance(target, Mobject):
            raise TypeError(type(self).__name__ + " expects two Mobject families")
        if not callable(match_animation) or not callable(mismatch_animation):
            raise TypeError("match_animation and mismatch_animation must be callable")
        source_pieces, target_pieces = _pieces(source), _pieces(target)
        # Empty operands are ordinary states of a changing formula or group.
        # The existing unmatched-piece fades cover creation/deletion; when
        # both sides are empty the shared composition still owns elapsed time.
        pairs = _pairs(matched_pairs, Mobject, type(self).__name__ + " matched_pairs")
        _validate_members(pairs, source_pieces, target_pieces)
        _validate_owners((source, target))
        all_source, all_target = tuple(source_pieces), tuple(target_pieces)
        self.source, self.target = source, target
        self.target_mobject, self.matched_pairs = target, pairs
        self.match_animation, self.mismatch_animation = match_animation, mismatch_animation
        self.anim_config = dict(kwargs)
        self.source_pieces, self.target_pieces = source_pieces, target_pieces
        self.anims = []
        # Explicit claims precede the public matcher. Materialize its result
        # before consuming pieces, including when an override is a generator.
        for pair in pairs:
            self.add_transform(*pair)
        candidates = _pairs(self.find_pairs_with_matching_shapes(self.source_pieces, self.target_pieces),
                            Mobject, "find_pairs_with_matching_shapes")
        _validate_members(candidates, all_source, all_target)
        for pair in candidates:
            self.add_transform(*pair)
        claimed = {id(piece) for animation in self.anims
                   for obj in g["_fmn_animated_mobjects"](animation) for piece in _pieces(obj)}
        for piece in self.source_pieces:
            if id(piece) not in claimed:
                self.anims.append(g["FadeOutToPoint"](piece, target.get_center(), **self.anim_config))
        for piece in self.target_pieces:
            if id(piece) not in claimed:
                self.anims.append(g["FadeInFromPoint"](piece, source.get_center(), **self.anim_config))
        super(Parts, self).__init__(*self.anims, run_time=run_time, lag_ratio=lag_ratio,
                                   rate_func=g["_linear_rate"])
        # Expose the real root before playback for nested groups and persistent
        # updaters. It contains the original operands, not geometry copies.
        g["_fmn_ensure_composition_root"](self)
        self._matching_finished = self._matching_cleanup_started = False

    def add_transform(self, source, target):
        if not isinstance(source, Mobject) or not isinstance(target, Mobject):
            raise TypeError("add_transform expects two Mobjects")
        source_members, target_members = _pieces(source), _pieces(target)
        if not source_members or not target_members:
            return
        available = [{id(obj) for obj in pieces} for pieces in (self.source_pieces, self.target_pieces)]
        if any(id(obj) not in available[side] for side, pieces in enumerate((source_members, target_members)) for obj in pieces):
            return  # Reference: a prior explicit/group claim wins.
        factory = self.match_animation if source.has_same_shape_as(target) else self.mismatch_animation
        animation = factory(source, target, **self.anim_config)
        if not isinstance(animation, Animation):
            raise TypeError("a matching animation factory must return an Animation")
        # A failed factory cannot consume either side of a match.
        self.anims.append(animation)
        for pieces, members in ((self.source_pieces, source_members), (self.target_pieces, target_members)):
            consumed = {id(obj) for obj in members}
            pieces[:] = [obj for obj in pieces if id(obj) not in consumed]

    def find_pairs_with_matching_shapes(self, sources, targets):
        sources, targets = _bounded(sources, "source pieces"), _bounded(targets, "target pieces")
        if len(sources) * len(targets) > _MAX_COMPARISONS:
            raise ValueError("shape matching exceeds the 1048576-comparison budget; supply explicit matched_pairs")
        return _bounded(((source, target) for source in sources for target in targets
                         if source.has_same_shape_as(target)), "shape matches")

    def begin(self):
        # A point-free operand contributes no animated leaf, so the native
        # leaf driver's owner checks cannot see it. Check both public roots
        # before starting any fades or publishing the target, including when
        # ownership changed since the matching plan was constructed.
        scene = getattr(self, "_composition_scene", None)
        if scene is None:
            scene = getattr(self.mobject, "_scene", None)
        _validate_owners((self.source, self.target), scene, g.get("_ForeignStageError", ValueError))
        self._matching_finished = self._matching_cleanup_started = False
        return super(Parts, self).begin()

    def finish(self):
        if not self._matching_finished:
            super(Parts, self).finish()
            self._matching_finished = True

    def abort(self):
        self._matching_finished = False
        return super(Parts, self).abort()

    def clean_up_from_scene(self, scene):
        if self._matching_cleanup_started:
            return
        if not self._matching_finished:
            raise RuntimeError("matching animation must finish before scene cleanup")
        # Child remover/replacement cleanup precedes publication. Never swallow
        # a failed removal or retry a partially completed authored cleanup.
        self._matching_cleanup_started = True
        try:
            super(Parts, self).clean_up_from_scene(scene)
            # Playing a matching animation adopts its point-bearing pieces,
            # not necessarily the original Text/Tex/group wrapper. A wrapper
            # never added to a Scene has no native handle to remove. Do not
            # turn successful implicit adoption into a stale-handle failure;
            # bound (including foreign/stale) sources still use strict remove.
            roots = (self.mobject, self.source) if self.source._is_bound() else (self.mobject,)
            scene.remove(*roots)
            scene.add(self.target)
        finally:
            self._composition_driver = None

    Parts.__bases__ = (Group,)
    for name, function in {
        "__init__": matching_init, "add_transform": add_transform,
        "find_pairs_with_matching_shapes": find_pairs_with_matching_shapes,
        "begin": begin, "finish": finish, "abort": abort,
        "clean_up_from_scene": clean_up_from_scene,
    }.items():
        _bind(Parts, name, function)
    hooks = (
        "begin", "finish", "interpolate", "interpolate_mobject", "interpolate_submobject",
        "update_mobjects", "clean_up_from_scene", "create_target", "create_starting_mobject",
        "init_path_func", "check_target_mobject_validity", "get_all_mobjects",
        "get_all_families_zipped", "get_all_mobjects_to_update", "get_sub_alpha",
        "time_spanned_alpha", "_ensure_runtime_defaults", "__getattribute__", "__getattr__",
    )
    classes = {base for cls in tuple(g.values())
               if isinstance(cls, type) and issubclass(cls, Transform)
               for base in cls.__mro__ if issubclass(base, Animation)}
    protocols = {cls: {name: _implementation(cls, name) for name in hooks} for cls in classes}
    previous_requires = g["_requires_python_animation"]

    def requires(animation):
        # Keep legacy _native_params inspection, but never lower away public
        # planning hooks or rebuild a different native matching plan at play.
        if isinstance(animation, Parts):
            return True
        # A custom matching factory may return a Transform subclass whose only
        # override is interpolation or finish. Older classifiers only compare
        # target/path hooks and otherwise choose the stock native kernel.
        # Static identity checks preserve these hooks without probing callbacks
        # or demoting unchanged built-in specializations.
        if isinstance(animation, Transform) and _changed_protocol(animation, protocols):
            return True
        return previous_requires(animation)

    g["_requires_python_animation"] = requires
    g["_FMN_MATCHING_INSTALLED"] = True


def install_matching_strings(native: Any) -> None:
    """Invoke the existing native-span block planner through the public hook."""
    from collections.abc import Mapping

    g = vars(native)
    if g.get("_FMN_MATCHING_STRINGS_INSTALLED", False):
        return
    if not g.get("_FMN_MATCHING_INSTALLED", False):
        raise ImportError("string matching requires the matching composition lifecycle")
    Parts, Strings = g["TransformMatchingParts"], g["TransformMatchingStrings"]
    Mobject, StringMobject = g["Mobject"], g["StringMobject"]

    def span_pieces(self, source, target):
        result = []
        for obj in (source, target):
            # Blank/whitespace strings need no ink or provenance entries, but
            # dropping a nonempty string's map must not turn it into an empty
            # match. Keep malformed maps out even with an authored matcher.
            spans = obj._string_sub_spans
            paths = getattr(obj, "_string_sub_paths", None)
            if paths is not None and len(paths) != len(spans):
                raise g["_TexError"]("matching native span and family-path counts differ")
            if not spans and _pieces(obj):
                raise g["_TexError"]("matching point-bearing strings require a native span map")
            # Validate UTF-8 and live native span-map parts even when an
            # authored matching_blocks implementation never calls super().
            keys = _bounded(self._native_span_keys(obj), "native span map")
            result.append(_unique(_bounded((piece for part, _ in keys for piece in _pieces(part)), "native span family")))
        return result

    def strings_init(self, source, target, matched_keys=(), key_map=None, matched_pairs=(),
                     run_time=2, lag_ratio=0, **kwargs):
        if not isinstance(source, StringMobject) or not isinstance(target, StringMobject):
            raise TypeError(type(self).__name__ + " expects two StringMobject instances")
        explicit = _pairs(matched_pairs, Mobject, type(self).__name__ + " matched_pairs")
        keys = tuple(_bounded(matched_keys, "matched_keys"))
        if key_map is not None and not isinstance(key_map, Mapping):
            raise TypeError("key_map must be a mapping of strings")
        mapping = {} if key_map is None else dict(_bounded(key_map.items(), "key_map"))
        if not all(isinstance(key, str) for key in (*keys, *mapping, *mapping.values())):
            raise TypeError("matching string keys must be strings")
        self.matched_pairs, self.matched_keys, self.key_map = explicit, keys, mapping
        sources, targets = span_pieces(self, source, target)
        _validate_owners((source, target))
        claimed = [set(), set()]
        for pair in explicit:
            for side, member in enumerate(pair):
                ids = {id(piece) for piece in _pieces(member)}
                if ids & claimed[side]:
                    raise ValueError("matched_pairs claims the same part twice")
                claimed[side].update(ids)
        allowed = [{id(member) for member in pieces} for pieces in (sources, targets)]
        has_foreign = any(
            not _pieces(member) or any(id(piece) not in allowed[side] for piece in _pieces(member))
            for pair in explicit
            for side, member in enumerate(pair)
        )
        if has_foreign:
            self.source = source
            self.target = target
            self.target_mobject = target
            self.mobject = source
            self.animations = []
            self._has_foreign = True
            super(Parts, self).__init__(run_time=run_time, lag_ratio=lag_ratio, **kwargs)
            self._matching_finished = self._matching_cleanup_started = False
            return
        blocks = _pairs(self.matching_blocks(source, target, keys, mapping), Mobject, "matching_blocks")
        _validate_members(blocks, sources, targets)
        super(Strings, self).__init__(source, target, matched_pairs=explicit + blocks,
                                      run_time=run_time, lag_ratio=lag_ratio, **kwargs)
        # Preserve the public explicit-pair inventory and old _native_params
        # inspection. Inferred groups are execution-plan children, not user
        # claims; conflating the two breaks native source_keys/target_keys.
        self.matched_pairs = explicit

    def strings_begin(self):
        if getattr(self, "_has_foreign", False):
            self._native_params()
        return super(Strings, self).begin()

    Strings.__bases__ = (Parts,)
    _bind(Strings, "__init__", strings_init)
    _bind(Strings, "begin", strings_begin)
    # The shared initializer already installs matching_blocks and the D-09
    # no-shape-fallback rule. Reuse both, including all authored overrides.
    g["_FMN_MATCHING_STRINGS_INSTALLED"] = True
