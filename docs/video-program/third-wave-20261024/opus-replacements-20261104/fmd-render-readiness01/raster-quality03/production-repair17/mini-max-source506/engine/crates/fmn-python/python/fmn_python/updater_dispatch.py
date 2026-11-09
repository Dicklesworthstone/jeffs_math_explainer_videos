"""Public child-update dispatch with one deferred native pass per invocation.

Python owns virtual calls and updater-list snapshots. Native updater slots are
run only after the host traversal returns, with no Stage borrow across Python.
Invocation state never enters an object's copied/pickled attribute dictionary.
Public scene updates share one host/native boundary across camera and roots.
The native scheduler may opt into that same boundary when a public override
requires it; unchanged scenes retain the existing native updater dispatch.
"""
from __future__ import annotations

from inspect import getattr_static
from threading import local
from types import GetSetDescriptorType, MemberDescriptorType, MethodType

_class_dict = type.__dict__["__dict__"].__get__
_class_mro = type.__dict__["__mro__"].__get__
_MISSING = object()
_OBJECT_GETATTRIBUTE = _class_dict(object)["__getattribute__"]


def _generic_getattr(cls):
    """Whether attribute lookup on cls is object's own: the first
    __getattribute__ in its MRO is object's and no class defines __getattr__.
    Then `getattr(obj, "__dict__")` is `object.__getattribute__(obj,
    "__dict__")`, the same C lookup, and no authored code can run in it.
    Class namespaces are read through type's own descriptors."""
    getattribute = _MISSING
    for base in _class_mro(cls):
        namespace = _class_dict(base)
        if "__getattr__" in namespace:
            return False
        if getattribute is _MISSING and "__getattribute__" in namespace:
            getattribute = namespace["__getattribute__"]
    return getattribute is _OBJECT_GETATTRIBUTE


def _static_is(obj, name, expected):
    """`getattr_static(obj, name, None) is expected`, exact, in one MRO walk.

    The ordinary case needs no full static lookup: every class in the MRO has
    metaclass `type`, the first class defining `name` defines `expected`, the
    class keeps the standard `__dict__` descriptor (inspect._shadowed_dict's
    test), and the instance's own __dict__ does not hold `name`. Anything else
    takes getattr_static itself. Class dictionaries are read through type's
    own descriptor, and the instance's through the standard slot, so no
    authored descriptor runs (fm-5wq.31).
    """
    found, plain, shadow_seen = _MISSING, True, False
    for base in type(obj).__mro__:
        if type(base) is not type:
            return getattr_static(obj, name, None) is expected
        namespace = _class_dict(base)
        if found is _MISSING and name in namespace:
            found = namespace[name]
        if not shadow_seen and "__dict__" in namespace:
            attr = namespace["__dict__"]
            if not (type(attr) is GetSetDescriptorType
                    and attr.__name__ == "__dict__" and attr.__objclass__ is base):
                shadow_seen = True
                plain = type(attr) is MemberDescriptorType
    if found is not expected:
        return getattr_static(obj, name, None) is expected
    if plain:
        try:
            own = object.__getattribute__(obj, "__dict__")
        except AttributeError:
            own = None
        if own is not None and dict.__contains__(own, name):
            return getattr_static(obj, name, None) is expected
    return True


def install_updater_dispatch(native):
    g = vars(native)
    if g.get("_FMN_UPDATER_DISPATCH_INSTALLED", False):
        return
    Mobject = g["Mobject"]
    state = local()
    native_update = getattr_static(Mobject, "_update_native_mobject")

    class NativePass:
        def __init__(self):
            self.entries = []
            self.authored = False
            self.roots = []

    def python_family(self, dt, recurse):
        if self._is_updating_suspended():
            return
        frame = getattr(state, "frame", None)
        # A direct call to this private host-only helper must not accidentally
        # run native slots. Its children still use their public update hooks.
        jobs, ancestors = (NativePass(), ()) if frame is None or frame[0] is not self else frame[1:]
        state.frame = None
        try:
            host_visit(self, dt, recurse, jobs, ancestors)
        finally:
            state.frame = frame

    def host_visit(self, dt, recurse, jobs, ancestors):
        if recurse:
            for child in list(self.submobjects):
                delegate_update(child, dt, jobs, (self, ancestors))
        for updater in list(self.updaters):
            self._dispatch_updater(updater, dt)

    def delegate_update(member, dt, jobs, ancestors, *, root=False):
        previous = getattr(state, "delegate", None)
        state.delegate = (member, jobs, ancestors, root)
        try:
            # Only pass dt: authored overrides choose their own recurse value.
            callback = member.update
            if (type(callback) is not MethodType or callback.__func__ is not update
                    or not _static_is(member, "_update_native_mobject", native_update)):
                jobs.authored = True
            callback(dt)
        finally:
            state.delegate = previous

    def finish_native(jobs, roots):
        if not jobs.authored:
            # The unchanged case keeps one recursive native call per root.
            for member, elapsed, recurse in roots:
                member._update_native_mobject(elapsed, recurse)
            return
        # Child-first, path-wise; no recursive re-tick at every ancestor.
        # Check suspension at execution time, including ancestors suspended by
        # a later host callback or an earlier native updater in this phase.
        for member, elapsed, parents in jobs.entries:
            cursor = (member, parents)
            while cursor and not cursor[0]._is_updating_suspended():
                cursor = cursor[1]
            if not cursor:
                member._update_native_mobject(elapsed, False)

    def native_slots(self):
        # A bound family is one Stage family. A detached one keeps each
        # member's native entry in its own nursery, so ask each member.
        if self._is_bound():
            return self._has_native_updaters_in_family()
        return any(member._has_native_updaters_in_family()
                   for member in g["_family_preorder"](self))

    def update(self, dt=0, recurse=True):
        dt, recurse = float(dt), bool(recurse)
        # Reference Mobject.update (mobject.py:826) returns before recursing
        # when its family has no updaters, so no descendant update runs: an
        # authored override in an updater-free family is not called. The
        # native slots engine-backed features install keep the walk that
        # runs them.
        if not self.has_updaters() and not native_slots(self):
            return self
        delegate = getattr(state, "delegate", None)
        joined = delegate is not None and delegate[0] is self
        jobs, ancestors = (delegate[1], delegate[2]) if joined else (NativePass(), ())
        previous = getattr(state, "frame", None)
        # A deliberate update() inside an updater is an independent invocation,
        # not an implicit child visit, even when it updates this same object.
        state.delegate = None
        state.frame = (self, jobs, ancestors)
        try:
            self._update_python_family(dt, recurse)
            state.frame = None
            jobs.entries.append((self, dt, ancestors))
            if joined and delegate[3]:
                jobs.roots.append((self, dt, recurse))
            if not joined:
                finish_native(jobs, [(self, dt, recurse)])
        finally:
            state.frame = previous
            state.delegate = delegate
            if not joined:
                jobs.entries.clear()
                jobs.roots.clear()
        return self

    for name, function in (("update", update), ("_update_python_family", python_family)):
        function.__name__ = name
        function.__qualname__ = Mobject.__qualname__ + "." + name
        function.__module__ = Mobject.__module__
        setattr(Mobject, name, function)
    Scene = g["Scene"]

    def update_mobjects(self, dt):
        dt = float(dt)
        # CameraFrame is intentionally absent from the drawable Stage. It is
        # nevertheless first in the Reference update order. Snapshot drawable
        # roots before its callbacks, as the native scene release path does.
        roots = list(self.mobjects)
        frame = self.frame
        jobs = NativePass()
        try:
            delegate_update(frame, dt, jobs, (), root=True)
            for root in roots:
                if root is not frame:
                    delegate_update(root, dt, jobs, (), root=True)
            finish_native(jobs, jobs.roots)
        finally:
            # A later root's failure must not leak an earlier root's deferred
            # native callbacks into another update or retain the scene graph.
            jobs.entries.clear()
            jobs.roots.clear()

    def should_update_mobjects(self):
        if self.always_update_mobjects:
            return True
        # has_updaters owns the recursive/live family query. Looking only at
        # each root's list misses callbacks on descendants and the camera.
        return any(root.has_updaters() for root in (self.frame, *self.mobjects))

    for name, function in (("update_mobjects", update_mobjects),
                           ("should_update_mobjects", should_update_mobjects)):
        function.__name__ = name
        function.__qualname__ = Scene.__qualname__ + "." + name
        function.__module__ = Scene.__module__
        setattr(Scene, name, function)
    from .movement import (_changed, _class_implementation, _implementation,
                           _plain_instance_dict, _protocols, _scan_implementation)

    object_hooks = ("update", "_update_python_family", "_update_native_mobject",
                    "_is_updating_suspended", "__getattribute__", "__getattr__")
    scene_hooks = ("update_mobjects", "__getattribute__", "__getattr__")
    object_protocols = scene_protocols = child_descriptors = scene_descriptors = None
    hook_names = ()
    child_protocols = {}
    _instance_dict = object.__getattribute__
    public_update_scan = g["_public_update_scan"]

    def finalize():
        nonlocal object_protocols, scene_protocols, child_descriptors, scene_descriptors
        nonlocal hook_names
        object_protocols = _protocols(g, Mobject, object_hooks)
        # The names movement._changed looks for in an object's own __dict__.
        hook_names = tuple({name for baseline in object_protocols.values() for name in baseline})
        scene_protocols = _protocols(g, Scene, scene_hooks)
        child_descriptors = {cls: _implementation(cls, "submobjects")
                             for cls in object_protocols}
        scene_descriptors = {cls: {name: _implementation(cls, name)
                                  for name in ("frame", "mobjects")}
                             for cls in scene_protocols}
        child_protocols.clear()
        for cls in (list, tuple, g.get("_LiveSubmobjects")):
            if cls is not None:
                child_protocols[cls] = {name: _implementation(cls, name)
                                        for name in ("__iter__", "__len__")}

    def requires_public_update(self):
        # Admission must not execute an authored getter, family walker, or
        # updater. Compare identities before reading ordinary shipping state.
        # Nothing authored runs during this scan, so no class can change in it:
        # per-class lookups are resolved once per scan (fm-5wq.31), not once
        # per family member per frame.
        memo = {}
        if _changed(self, scene_protocols, memo):
            return True
        expected = next(scene_descriptors[cls] for cls in type(self).__mro__
                        if cls in scene_descriptors)
        if any(_class_implementation(type(self), name, memo) is not method
               for name, method in expected.items()):
            return True
        # An object's answer is its class's unless its own __dict__ holds a
        # name the lookup reads (movement._changed, _scan_implementation).
        # Decide each class once, from an object without such an entry, and
        # take the exact per-object path only for objects that have one: the
        # per-member helper calls cost ~9 us, and this runs every frame over
        # every member (PrimePanning: 2,400 members, a third of its time).
        # _public_update_scan runs that loop natively, calling these
        # decisions where the Python loop did and caching them per class for
        # this scan (fm-5wq.31: the loop itself cost ~3 us per member).

        def member_changed(member):
            return _changed(member, object_protocols, memo)

        def submobjects_kept(cls):
            expected = next(child_descriptors[base] for base in cls.__mro__
                            if base in child_descriptors)
            return _class_implementation(cls, "submobjects", memo) is expected

        def container_changed(children, methods):
            return any(_scan_implementation(children, name, memo) is not method
                       for name, method in methods.items())

        return public_update_scan(
            [self.frame, *self.mobjects],
            (Mobject, hook_names, child_protocols, _instance_dict, _plain_instance_dict,
             member_changed, submobjects_kept, container_changed, _generic_getattr))

    def dispatch_public_update(self, dt):
        # A true result means BOTH host and native slots have completed. The
        # caller must then capture without a second native scene-updater pass.
        if not requires_public_update(self):
            return False
        self.update_mobjects(float(dt))
        return True

    for name, function in (("_fmn_requires_public_scene_update", requires_public_update),
                           ("_fmn_dispatch_public_scene_update", dispatch_public_update)):
        function.__name__ = name
        function.__qualname__ = Scene.__qualname__ + "." + name
        function.__module__ = Scene.__module__
        setattr(Scene, name, function)
    finalize()
    g["_fmn_finalize_updater_dispatch"] = finalize
    g["_FMN_UPDATER_DISPATCH_INSTALLED"] = True
