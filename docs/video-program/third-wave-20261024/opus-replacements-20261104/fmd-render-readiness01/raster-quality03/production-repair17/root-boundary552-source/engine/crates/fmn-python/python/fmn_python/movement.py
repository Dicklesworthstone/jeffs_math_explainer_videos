"""Functional movement on the shared Animation lifecycle and native geometry.

The host executes authored maps; Mobject owns geometry and Choreo owns time.
Installation preserves public classes and qualified aliases. No point sampler,
integrator, frame loop, renderer, or dependency is introduced here.
"""
from __future__ import annotations

from functools import wraps
import inspect
import math
import types
from typing import Any


def _method(cls, name, function):
    function.__name__ = name
    function.__qualname__ = cls.__qualname__ + "." + name
    function.__module__ = cls.__module__
    setattr(cls, name, function)


def _callback_rate(g, rate):
    if isinstance(rate, str):
        name = rate
        rate = next((function for function, label in g["_RATE_FUNC_NAMES"].items()
                     if label == name), None)
        if rate is None:
            raise ValueError("unknown rate function: " + name)
    if not callable(rate):
        raise TypeError("rate_func must be a callable or a catalog name")
    return rate


def _cancel_preserving(animation, error):
    try:
        animation.abort()
    except BaseException as cleanup_error:
        error.add_note(f"movement cleanup also failed: {cleanup_error}")


def _install_lifecycle(g, cls, prepare):
    """Add failure ownership around the existing cooperative base protocol."""
    def release(self, *, update=False):
        prior = getattr(self, "_movement_prior", ())
        self._movement_prior = ()
        acquired = getattr(self, "_movement_suspends", False)
        self.mobject_was_updating = False
        first = None
        try:
            self.mobject.set_animating_status(False)
        except BaseException as error:
            first = error
        if acquired:
            for member, was_suspended in prior:
                if was_suspended:
                    continue
                try:
                    if member._is_updating_suspended():
                        member.resume_updating(recurse=False, call_updater=False)
                except BaseException as error:
                    if first is None:
                        first = error
        if first is not None:
            raise first
        # Successful Animation.finish historically resumes with update(0).
        # Restore flags first so children suspended before this animation stay
        # pruned. Error unwinding deliberately never invokes another updater.
        if update and acquired and prior and not prior[0][1]:
            self.mobject.update(0.0)

    def abort(self):
        if not getattr(self, "_movement_active", False):
            return
        self._movement_active = False
        release(self)

    def defaults(self):
        super(cls, self)._ensure_runtime_defaults()
        self.rate_func = _callback_rate(g, self.rate_func)

    def begin(self):
        if getattr(self, "_movement_active", False):
            self.abort()
        self._movement_finished = self._movement_cleaned = False
        self._ensure_runtime_defaults()
        prepare(self)
        prior, seen = [], set()
        for member in self.mobject.get_family():
            if id(member) not in seen:
                seen.add(id(member))
                prior.append((member, member._is_updating_suspended()))
        self._movement_prior = prior
        self._movement_suspends = bool(self.suspend_mobject_updating)
        self._movement_active, self._movement_finished = True, False
        self._movement_cleaned = False
        try:
            super(cls, self).begin()
        except BaseException as error:
            _cancel_preserving(self, error)
            raise

    def finish(self):
        if not getattr(self, "_movement_active", False):
            if getattr(self, "_movement_finished", False):
                return
            raise RuntimeError(type(self).__name__ + " must begin before finish")
        # Shared finish performs final-alpha interpolation and status updates;
        # member-wise resumption below replaces only its recursive resume.
        self.mobject_was_updating = False
        try:
            super(cls, self).finish()
            release(self, update=True)
        except BaseException as error:
            _cancel_preserving(self, error)
            raise
        self._movement_active, self._movement_finished = False, True

    def interpolate(self, alpha):
        try:
            return super(cls, self).interpolate(alpha)
        except BaseException as error:
            _cancel_preserving(self, error)
            raise

    def update_mobjects(self, dt):
        try:
            return super(cls, self).update_mobjects(dt)
        except BaseException as error:
            _cancel_preserving(self, error)
            raise

    def cleanup(self, scene):
        if not getattr(self, "_movement_finished", False) or self._movement_cleaned:
            return
        super(cls, self).clean_up_from_scene(scene)
        self._movement_cleaned = True

    for name, function in {
        "begin": begin, "finish": finish, "abort": abort,
        "interpolate": interpolate, "update_mobjects": update_mobjects,
        "clean_up_from_scene": cleanup, "_ensure_runtime_defaults": defaults,
    }.items():
        _method(cls, name, function)


def install_movement(native: Any) -> None:
    """Install available movement classes without replacing their identities.

    A reduced embedding namespace with no movement classes is unchanged; this
    does not manufacture missing exports or alter the independent parity audit.
    """
    g = vars(native)
    if g.get("_FMN_MOVEMENT_INSTALLED", False):
        return
    Homotopy = g.get("Homotopy")
    if Homotopy is None:
        return
    Animation, Scene = g["Animation"], g["Scene"]
    if not isinstance(Homotopy, type) or not issubclass(Homotopy, Animation):
        raise TypeError("Homotopy must be an Animation class")

    def prepare_homotopy(self):
        if not callable(self.homotopy):
            raise TypeError("Homotopy requires a callable (x, y, z, t) map")

    def interpolate_family(self, alpha):
        # Restore the shared get_all_families_zipped/get_sub_alpha pipeline.
        # The previous bootstrap skipped point-free roots, starting-copy hooks
        # and family lag, and never acquired updater suspension in begin.
        return super(Homotopy, self).interpolate_mobject(alpha)

    _install_lifecycle(g, Homotopy, prepare_homotopy)
    _method(Homotopy, "interpolate_mobject", interpolate_family)
    movement_types = [Homotopy]
    PhaseFlow = g.get("PhaseFlow")
    if PhaseFlow is not None:
        _install_phase_flow(g, PhaseFlow)
        movement_types.append(PhaseFlow)
    PathMotion = g.get("MoveAlongPath")
    if PathMotion is not None:
        _install_path_motion(g, PathMotion)
        movement_types.append(PathMotion)
    movement_types = tuple(movement_types)
    previous_play = Scene.play

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
        movements, seen, visiting = [], set(), set()
        stack = [(animation, False) for animation in reversed(animations)]
        while stack:
            animation, exiting = stack.pop()
            marker = id(animation)
            if exiting:
                visiting.remove(marker)
                continue
            if marker in visiting:
                raise ValueError("Animation composition contains a cycle")
            if marker in seen:
                continue
            seen.add(marker)
            visiting.add(marker)
            stack.append((animation, True))
            if isinstance(animation, movement_types):
                movements.append(animation)
            if isinstance(animation, g["AnimationGroup"]):
                stack.extend((child, False) for child in reversed(animation.animations))
        absent = object()
        forced = []
        try:
            if (PathMotion is not None and _custom_rate(g, kwargs.get("rate_func"))
                    and any(isinstance(animation, PathMotion) for animation in animations)):
                for animation in animations:
                    if isinstance(animation, PathMotion):
                        forced.append((animation, animation.__dict__.get("_movement_force_callback", absent)))
                        animation.__dict__["_movement_force_callback"] = True
                if all(isinstance(animation, Animation) and g["_requires_python_animation"](animation)
                       for animation in animations):
                    # The frontend pre-samples its global rate even for an
                    # entirely callback-driven play. Assign the same override
                    # before lowering and omit that redundant payload. Mixed
                    # native plays retain the original global lowering and
                    # sampling density, including native sibling overrides.
                    for animation in animations:
                        animation.rate_func = kwargs["rate_func"]
                    kwargs = dict(kwargs, rate_func=None)
            return previous_play(self, *animations, **kwargs)
        except BaseException as error:
            # Authored overrides can fail outside super(), or a sibling/scene
            # updater can fail after this animation's callbacks have returned.
            for animation in reversed(movements):
                _cancel_preserving(animation, error)
            raise
        finally:
            for animation, previous in reversed(forced):
                if previous is absent:
                    animation.__dict__.pop("_movement_force_callback", None)
                else:
                    animation.__dict__["_movement_force_callback"] = previous

    Scene.play = play
    g["_FMN_MOVEMENT_INSTALLED"] = True


def _install_phase_flow(g, Flow):
    def flow_init(self, function, mobject, virtual_time=None,
                  suspend_mobject_updating=False, rate_func=g["_linear_rate"],
                  run_time=3.0, **kwargs):
        if not callable(function):
            raise TypeError("PhaseFlow requires a callable vector field")
        duration = float(run_time if virtual_time is None else virtual_time)
        if not math.isfinite(duration):
            raise ValueError("PhaseFlow virtual_time must be finite")
        self.function, self.virtual_time = function, duration
        super(Flow, self).__init__(
            mobject, run_time=run_time, rate_func=rate_func,
            suspend_mobject_updating=suspend_mobject_updating, **kwargs,
        )

    def prepare(self):
        if not callable(self.function):
            raise TypeError("PhaseFlow requires a callable vector field")
        if not math.isfinite(float(self.virtual_time)):
            raise ValueError("PhaseFlow virtual_time must be finite")
        # Match native PhaseFlow::setup. A reused animation must not perform
        # an accidental backwards Euler step from its last run's final alpha
        # to begin(0). The current live geometry is the next run's initial state.
        self.__dict__.pop("last_alpha", None)

    _method(Flow, "__init__", flow_init)
    _install_lifecycle(g, Flow, prepare)
    # Keep the existing stateful Euler map, raw-alpha convention and native
    # point writes. In particular, do not reinterpret it as an RK integrator
    # or apply rate/lag/time_span a second time.


def _implementation(obj, name):
    value = _getattr_static(obj, name)
    if isinstance(value, (staticmethod, classmethod, types.MethodType)):
        return value.__func__
    return value


_NOT_FOUND = object()


def _plain_mro(cls):
    """cls's MRO when every class in it has metaclass `type` exactly, else None.

    Then no metaclass can shadow `__dict__` (inspect._shadowed_dict of `type`
    is its sentinel), and a class's namespace is its own `__dict__`.
    """
    mro = _class_mro(cls)
    for entry in mro:
        if type(entry) is not type:
            return None
    return mro


def _lookup(mro, name):
    for entry in mro:
        namespace = _class_dict(entry)
        if name in namespace:
            return namespace[name]
    return _NOT_FOUND


def _getattr_static(obj, name):
    """inspect.getattr_static(obj, name, None), exactly, for CPython 3.13.

    The ordinary case, where every class involved has metaclass `type`,
    needs only namespace lookups; anything else is getattr_static itself.
    getattr_static's cost was re-walking each MRO entry's metaclass on every
    call: 1.05M calls, 12 s of one corpus scene's first 150 s (fm-5wq.31).
    """
    # getattr_static's own test; isinstance would consult a faked __class__.
    if type in _class_mro(type(obj)):
        mro = _plain_mro(obj)
        meta_mro = _plain_mro(type(obj)) if mro is not None else None
        if meta_mro is None:
            return inspect.getattr_static(obj, name, None)
        found = _lookup(mro, name)
        if found is _NOT_FOUND:
            found = _lookup(meta_mro, name)  # "for types we check the metaclass too"
        return None if found is _NOT_FOUND else found
    klass = type(obj)
    mro = _plain_mro(klass)
    if mro is None:
        return inspect.getattr_static(obj, name, None)
    instance = _NOT_FOUND
    if _plain_instance_dict(klass):
        try:
            own = object.__getattribute__(obj, "__dict__")
        except AttributeError:
            own = None
        if own is not None:
            if type(own) is not dict:
                return inspect.getattr_static(obj, name, None)
            instance = dict.get(own, name, _NOT_FOUND)
    found = _lookup(mro, name)
    if instance is not _NOT_FOUND and found is not _NOT_FOUND:
        kind = _plain_mro(type(found))
        if kind is None:
            return inspect.getattr_static(obj, name, None)
        if _lookup(kind, "__get__") is not _NOT_FOUND and (
                _lookup(kind, "__set__") is not _NOT_FOUND
                or _lookup(kind, "__delete__") is not _NOT_FOUND):
            return found  # a data descriptor beats the instance dict
    if instance is not _NOT_FOUND:
        return instance
    return None if found is _NOT_FOUND else found


def _protocols(g, root, names):
    """{cls: {name: _implementation(cls, name)}} for every class under `root`.

    Exactly getattr_static's answer for a class: the first MRO entry whose
    metaclass keeps the standard `__dict__` and whose namespace holds the
    name, else the same search over the metaclass's MRO. Each entry's
    definitions and each metaclass's check are computed once per call
    instead of once per (class, name, MRO entry): 58,964 getattr_static
    calls at import became none. Nothing is kept across calls, since
    installers change classes between them.

    The table also carries the memo-less _changed cache (see _ProtocolTable),
    so it must not change once a check has run on it.
    """
    classes = {base for cls in tuple(g.values())
               if isinstance(cls, type) and issubclass(cls, root)
               for base in cls.__mro__ if issubclass(base, root)}
    names = tuple(dict.fromkeys(names))
    standard, defined = {}, {}

    def definitions(entry):
        found = defined.get(entry)
        if found is None:
            found = {}
            if _standard_class_dict(type(entry), standard):
                namespace = _class_dict(entry)
                found = {name: namespace[name] for name in names if name in namespace}
            defined[entry] = found
        return found

    table = _ProtocolTable()
    table.tags, table.verdicts = g.get("_type_version_tags"), {}
    for cls in classes:
        row = {}
        for entry in _class_mro(cls):
            for name, value in definitions(entry).items():
                row.setdefault(name, value)
            if len(row) == len(names):
                break
        if len(row) < len(names):
            for entry in _class_mro(type(cls)):
                for name, value in definitions(entry).items():
                    row.setdefault(name, value)
        table[cls] = {name: _unwrapped(row.get(name)) for name in names}
    return table


def _unwrapped(value):
    if isinstance(value, (staticmethod, classmethod, types.MethodType)):
        return value.__func__
    return value


_class_dict = type.__dict__["__dict__"].__get__
_class_mro = type.__dict__["__mro__"].__get__


def _standard_class_dict(meta, memo):
    """inspect._shadowed_dict(meta) is its sentinel: no class in the
    metaclass's MRO replaces the standard `__dict__` descriptor, so a class
    of this metaclass exposes its namespace to getattr_static."""
    known = memo.get(meta)
    if known is None:
        known = True
        for entry in _class_mro(meta):
            namespace = _class_dict(entry)
            if "__dict__" in namespace:
                attr = namespace["__dict__"]
                if not (type(attr) is types.GetSetDescriptorType
                        and attr.__name__ == "__dict__" and attr.__objclass__ is entry):
                    known = False
                    break
        memo[meta] = known
    return known


def _plain_instance_dict(cls):
    """Whether getattr_static consults an instance's own __dict__ for `cls`.

    This is inspect._shadowed_dict's test without its private API: the first
    non-standard `__dict__` entry in the MRO decides, and only a slot keeps
    the instance dictionary visible. Class dictionaries are read through
    type's own descriptor, so no authored metaclass code runs.
    """
    for entry in cls.__mro__:
        namespace = _class_dict(entry)
        if "__dict__" in namespace:
            attr = namespace["__dict__"]
            if not (type(attr) is types.GetSetDescriptorType
                    and attr.__name__ == "__dict__" and attr.__objclass__ is entry):
                return type(attr) is types.MemberDescriptorType
    return True


def _scan_implementation(obj, name, memo):
    """_implementation(obj, name), exact, with the per-class answer memoized.

    For an instance, getattr_static depends on the object only through the
    instance's own __dict__, and only when its class keeps the ordinary
    `__dict__` descriptor. An object without its own `name` entry therefore
    shares its class's answer, which is computed once per scan. `memo` must
    not outlive a scan that executes no authored code: class mutation
    between scans invalidates it.
    """
    if memo is None:
        return _implementation(obj, name)
    cls = type(obj)
    plain = memo.get((cls, "__dict__"))
    if plain is None:
        plain = memo[(cls, "__dict__")] = _plain_instance_dict(cls)
    if plain:
        try:
            own = object.__getattribute__(obj, "__dict__")
        except AttributeError:
            own = None
        if own is not None and dict.__contains__(own, name):
            return _implementation(obj, name)
    key = (cls, name, False)
    try:
        return memo[key]
    except KeyError:
        value = memo[key] = _implementation(obj, name)
        return value


def _class_implementation(cls, name, memo):
    """_implementation(cls, name), memoized for one scan (see _scan_implementation)."""
    if memo is None:
        return _implementation(cls, name)
    key = (cls, name, True)
    try:
        return memo[key]
    except KeyError:
        value = memo[key] = _implementation(cls, name)
        return value


def _changed(obj, protocols, memo=None):
    if memo is not None:
        # Without its own entry for any hook, an object's verdict is its
        # class's (see _scan_implementation): decide once per class per scan.
        cls = type(obj)
        names = memo.get(id(protocols))
        if names is None:
            names = memo[id(protocols)] = tuple(
                {name for baseline in protocols.values() for name in baseline})
        plain = memo.get((cls, "__dict__"))
        if plain is None:
            plain = memo[(cls, "__dict__")] = _plain_instance_dict(cls)
        own = None
        if plain:
            try:
                own = object.__getattribute__(obj, "__dict__")
            except AttributeError:
                own = None
        if own is None or not any(dict.__contains__(own, name) for name in names):
            key = (cls, id(protocols), "changed")
            verdict = memo.get(key)
            if verdict is None:
                verdict = memo[key] = _changed_uncached(obj, protocols, memo)
            return verdict
    elif type(protocols) is _ProtocolTable and protocols.tags is not None:
        return _changed_across_calls(obj, protocols)
    return _changed_uncached(obj, protocols, memo)


class _ProtocolTable(dict):
    """A _protocols table with its own cache of memo-less _changed verdicts.

    fm-xte3: memo-less checks (Mobject.rotate, DecimalNumber updates, per-play
    admission) run inside authored code, so no per-scan memo applies, and
    each re-derived its static lookups over the whole MRO: 93 of rotate's
    151 us. `verdicts` keeps a class's verdict with CPython's version tags of
    the class and its metaclass (`tags`, the ratified method-cache read,
    ADR-0015 Amendment 1): any mutation of either MRO changes a tag or zeroes
    it, and a zero tag never caches. The cache lives on the table, which the
    installer closures own, so it never outlives the module that built it.
    """
    __slots__ = ("tags", "verdicts")


def _changed_across_calls(obj, protocols):
    """_changed(obj, protocols) without a memo, exactly.

    As in the per-scan path, an object whose own __dict__ holds none of the
    looked-up names has its class's verdict; one that holds any is decided
    alone. Only classes whose whole MRO has metaclass `type` are cached: tags
    cover edits to a class, its bases, and `__bases__`, but not `__class__`
    assignment on a base, which only a base with a heap metaclass allows. A
    class passed as `obj` is never cached. Nothing here runs authored code:
    tags are read, never forced.
    """
    cls = type(obj)
    tags = protocols.tags(cls)
    entry = protocols.verdicts.get(cls)
    if entry is None or entry[0] != tags or not (tags[0] and tags[1]):
        entry = None
        names = tuple({name for baseline in protocols.values() for name in baseline})
        plain = _plain_instance_dict(cls)
    else:
        names, plain = entry[1], entry[2]
    own = None
    if plain:
        try:
            own = object.__getattribute__(obj, "__dict__")
        except AttributeError:
            own = None
    if own is not None and (type(own) is not dict or not dict.keys(own).isdisjoint(names)):
        return _changed_uncached(obj, protocols, None)
    if entry is not None:
        return entry[3]
    verdict = _changed_uncached(obj, protocols, None)
    if (tags[0] and tags[1] and _plain_mro(cls) is not None
            and type not in _class_mro(cls)):
        protocols.verdicts[cls] = (tags, names, plain, verdict)
    return verdict


def _changed_uncached(obj, protocols, memo):
    found = False
    for cls in type(obj).__mro__:
        baseline = protocols.get(cls)
        if baseline is None:
            continue
        if not found:
            found = True
            if any(_scan_implementation(obj, name, memo) is not expected
                   for name, expected in baseline.items()):
                return True
        # A shipped override may still call a changed base through super().
        if any(_class_implementation(cls, name, memo) is not expected
               for name, expected in baseline.items()):
            return True
    return not found


def _custom_rate(g, rate):
    return rate is not None and not isinstance(rate, str) and all(
        rate is not known for known in g["_RATE_FUNC_NAMES"]
    )


def _install_path_motion(g, PathMotion):
    def prepare(self):
        if not isinstance(self.path, g["VMobject"]) or not self.path.has_points():
            raise ValueError("MoveAlongPath requires a nonempty VMobject path")
        owner = getattr(self.mobject, "_scene", None)
        path_owner = getattr(self.path, "_scene", None)
        if owner is not None and path_owner is not None and owner is not path_owner:
            error = g.get("_ForeignStageError", ValueError)
            raise error("MoveAlongPath cannot reference a path from another Scene; copy it")

    _install_lifecycle(g, PathMotion, prepare)
    hooks = ("begin", "finish", "interpolate", "interpolate_mobject", "interpolate_submobject",
             "create_starting_mobject", "get_all_mobjects", "get_all_families_zipped",
             "get_all_mobjects_to_update", "update_mobjects", "clean_up_from_scene",
             "_ensure_runtime_defaults", "__getattribute__", "__getattr__")
    protocols = _protocols(g, PathMotion, hooks)
    # Include the shared bases: a later Animation.begin replacement is an
    # authored hook even when PathMotion's installed wrapper has not changed.
    protocols.update({base:{name:_implementation(base, name) for name in hooks}
                      for base in PathMotion.__mro__[1:] if issubclass(base, g["Animation"])})
    path_protocols = _protocols(g, g["VMobject"],
                                ("point_from_proportion", "_point_from_proportion",
                                 "__getattribute__", "__getattr__"))
    object_protocols = _protocols(g, g["Mobject"],
                                  ("move_to", "shift", "get_center", "get_bounding_box_point",
                                   "__getattribute__", "__getattr__"))
    previous_requires = g["_requires_python_animation"]

    def requires(animation):
        if isinstance(animation, PathMotion):
            if (getattr(animation, "_movement_force_callback", False)
                    or getattr(animation, "_movement_active", False)
                    or animation.final_alpha_value != 1.0 or animation.remover
                    or _custom_rate(g, animation.rate_func)
                    or _changed(animation, protocols)
                    or _changed(animation.path, path_protocols)
                    or _changed(animation.mobject, object_protocols)):
                return True
        return previous_requires(animation)

    # The existing interpolation calls path.point_from_proportion at the real
    # current alpha. The default implementation is Chisel's true-arclength
    # sampler; custom path/move_to methods are invoked, never probe-classified.
    # Unchanged stock animations still use Choreo's native MoveAlongPath.
    g["_requires_python_animation"] = requires
