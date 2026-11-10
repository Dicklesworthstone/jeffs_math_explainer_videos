"""Persistent animations on the Scene's existing updater boundary.

Callback animations retain their authored lifecycle. Native-only animations
borrow Choreo's existing scene-bound driver, not another interpolation engine
or clock. Persistent effects never perform scene-removal/publication cleanup.
"""
from __future__ import annotations

from contextlib import contextmanager
from functools import wraps
import inspect
import math
import sys
from typing import Any

from .scene_execution import _Execution


def _note(error, message):
    try:
        error.add_note(message)
    except BaseException:
        pass


def _duration(driver):
    value = float(driver.get_run_time())
    if not math.isfinite(value) or value < 0:
        raise ValueError("animation updater run_time must be finite and nonnegative")
    return value


def _has_python_interpolation(g, animation):
    if getattr(animation, "_native_kind", None) in {"cyclic_replace", "swap"}:
        return False
    base = g["Animation"]
    for name in ("interpolate", "interpolate_mobject", "interpolate_submobject"):
        method = getattr(animation, name)
        implementation = getattr(method, "__func__", method)
        if implementation is not getattr(base, name):
            if not getattr(implementation, "_fmn_schema_placeholder", False):
                return True
    return not getattr(animation, "_native_kind", None)


def _animation_tree(g, animation):
    """Walk shared compositions once and reject cycles, including explicit roots."""
    seen, visiting = set(), set()
    stack = [(animation, False)]
    while stack:
        node, exiting = stack.pop()
        marker = id(node)
        if exiting:
            visiting.remove(marker)
            continue
        if marker in visiting:
            raise ValueError("Animation composition contains a cycle")
        if marker in seen:
            continue
        seen.add(marker)
        visiting.add(marker)
        yield node
        stack.append((node, True))
        if isinstance(node, g["AnimationGroup"]):
            stack.extend((child, False) for child in reversed(node.animations))


def _scene_for(g, animation, anchor, nodes=None, expected_scene=None):
    """Resolve ownership without invoking create_target or changing draw roots."""
    objects = [anchor]
    for node in _animation_tree(g, animation) if nodes is None else nodes:
        obj = getattr(node, "mobject", None)
        if isinstance(obj, g["Mobject"]):
            objects.append(obj)
        objects.extend(getattr(node, "_native_extra_mobjects", ()))
        target_name = getattr(node, "_target_attr", None)
        target = getattr(node, target_name, None) if target_name else None
        if isinstance(target, g["Mobject"]):
            objects.append(target)
    scene, seen = expected_scene, set()
    for obj in objects:
        # The anchor's family normally already contains each leaf. Avoid
        # walking it again for duplicate operands at every callback boundary.
        if id(obj) in seen:
            continue
        for member in obj.get_family():
            if id(member) in seen:
                continue
            seen.add(id(member))
            owner = getattr(member, "_scene", None)
            if owner is not None:
                if scene is not None and owner is not scene:
                    error = g.get("_ForeignStageError", ValueError)
                    raise error("persistent animation cannot reference multiple Scenes; copy its mobjects")
                scene = owner
    return scene


@contextmanager
def _composition_context(g, animation, scene, groups=None):
    """Keep nested authored begin hooks on the resolved Scene, without leaking it."""
    missing, restored = object(), []
    try:
        nodes = _animation_tree(g, animation) if groups is None else groups
        for node in nodes:
            if isinstance(node, g["AnimationGroup"]):
                previous = node.__dict__.get("_composition_scene", missing)
                restored.append((node, previous))
                node._composition_scene = scene
        yield
    finally:
        for node, previous in reversed(restored):
            if previous is missing:
                node.__dict__.pop("_composition_scene", None)
            else:
                node._composition_scene = previous


def _implementation(obj, name):
    value = inspect.getattr_static(obj, name, None)
    return value.__func__ if isinstance(value, (staticmethod, classmethod)) else getattr(value, "__func__", value)


_NATIVE_HOOKS = (
    "begin", "finish", "interpolate", "interpolate_mobject", "interpolate_submobject",
    "update_mobjects", "create_starting_mobject", "get_sub_alpha", "time_spanned_alpha",
    "get_all_mobjects", "get_all_families_zipped", "get_all_mobjects_to_update",
)


def _native_hooks_unchanged(animation, protocols):
    found = False
    for cls in type(animation).__mro__:
        baseline = protocols.get(cls)
        if baseline is None:
            continue
        if not found:
            found = True
            if any(_implementation(animation, name) is not value for name, value in baseline.items()):
                return False
        if any(_implementation(cls, name) is not value for name, value in baseline.items()):
            return False
    return found


class _PersistentAnimation:
    def __init__(self, g, animation, anchor, cycle, driver_factory=None):
        self.g, self.animation, self.anchor = g, animation, anchor
        self.cycle = bool(cycle)
        self.driver = animation
        self.driver_factory = driver_factory
        self.needs_scene = driver_factory is not None or isinstance(animation, g["AnimationGroup"])
        self.scene = None
        self.nodes = self.groups = ()
        self.closed = self.busy = self.begun = False
        self.cleanup_pending = False
        self.execution = _Execution(g)

        def update(current, dt):
            return self.step(current, dt)

        update._fmn_persistent_controller = self
        self.update = update

    def detach(self):
        self.anchor.remove_updater(self.update)

    def call(self, method, *args, checked=False):
        # Authored hooks (including duration lookup and native lowering) can
        # adopt participants between phases. Never hand a foreign-owned
        # family to the next phase or commit that tick's elapsed time.
        # `checked` means the previous phase's closing check just ran with
        # nothing executed since; repeating it walks every family again.
        if not checked:
            self.check_scene()
        if self.needs_scene:
            with _composition_context(self.g, self.animation, self.scene, self.groups):
                result = getattr(self.driver, method)(*args)
        else:
            result = getattr(self.driver, method)(*args)
        if not self.closed:
            self.check_scene()
        return result

    def cancel(self, original=None):
        if self.closed:
            if self.cleanup_pending and original is not None:
                self._unwind(original)
            return
        self.closed = True
        if self.busy and original is None:
            # A callback may remove its own updater. Finish this invocation
            # before unwinding a group driver that is still on the stack.
            self.cleanup_pending = True
            self.detach()
            return
        self._unwind(original)

    def _unwind(self, original=None):
        self.cleanup_pending = False
        first = None
        diagnostic = original if original is not None else RuntimeError("animation updater cancellation failed")
        try:
            if self.begun:
                # Use the same snapshots and exactly-once abort ownership as
                # Scene.play. Root-only resumption lost prior locks, ancestor
                # flags and already-suspended descendants after a bad callback.
                with _composition_context(self.g, self.animation, self.scene, self.groups):
                    first = self.execution.unwind(diagnostic)
        except BaseException as error:
            first = error
            if original is not None:
                _note(original, "animation updater cleanup failed: " + type(error).__name__)
        try:
            self.detach()
        except BaseException as error:
            if first is None:
                first = error
            if original is not None:
                _note(original, "animation updater detachment failed: " + type(error).__name__)
        finally:
            self.execution.release()
            self.driver = None
            self.nodes = self.groups = ()
        if first is not None and original is None:
            raise first

    def check_scene(self):
        # Detached callback animations are legal, but their operands cannot
        # belong to different Scenes. Once an owner is observed, keep the
        # execution attached to it, including native handles frozen at begin.
        self.scene = _scene_for(self.g, self.animation, self.anchor,
                                self.nodes, self.scene)

    def activate(self):
        nodes = tuple(_animation_tree(self.g, self.animation))
        scene = _scene_for(self.g, self.animation, self.anchor, nodes)
        if self.needs_scene and scene is None:
            return False
        self.scene = scene
        # Drivers freeze their child timeline at begin. Validate those same
        # leaves on subsequent ticks, not a later edited animation list.
        self.nodes = nodes
        self.groups = tuple(node for node in nodes
                            if isinstance(node, self.g["AnimationGroup"]))
        if self.needs_scene and not self.anchor._is_bound():
            self.scene._adopt(self.anchor)
        self.animation._ensure_runtime_defaults()
        if self.driver_factory is not None:
            self.driver = self.driver_factory(self.scene)
        if self.closed:
            self.driver = None
            return False
        duration = _duration(self.driver)
        if self.cycle and duration == 0:
            raise ValueError("a cycling animation updater requires positive run_time")
        # Snapshot actual public roots before begin, including ancestors and
        # shared descendants. A native-only adapter may expose no Python root,
        # so capture the public animation even when driving a native handle.
        public_handle, = self.execution.wrap((self.animation,))
        if self.driver is self.animation:
            self.driver = public_handle
        else:
            self.driver, = self.execution.wrap((self.driver,))
        self.begun = True
        self.call("begin")
        return not self.closed

    def start(self):
        duration = _duration(self.animation)
        if self.cycle and duration == 0:
            raise ValueError("a cycling animation updater requires positive run_time")
        self.animation.suspend_mobject_updating = False
        self.animation.total_time = 0.0
        try:
            self.activate()
            if not self.closed:
                # Detached native/group animations register a pending updater.
                # Zero-dt registration cannot allocate an unrelated Scene or
                # advance their clock before the owner has adopted the anchor.
                self.anchor.add_updater(self.update)
        except BaseException as error:
            self.cancel(error)
            raise
        return self.anchor

    def step(self, current, dt):
        # Starting/target copies share callables, but cannot drive the original.
        if self.closed or self.busy or current is not self.anchor:
            return
        self.busy = True
        awaiting_adoption = False
        try:
            delta = float(dt)
            if not math.isfinite(delta):
                raise ValueError("animation updater dt and total_time must be finite")
            if not self.begun and self.needs_scene:
                if _scene_for(self.g, self.animation, self.anchor) is None:
                    if delta != 0:
                        awaiting_adoption = True
                        raise RuntimeError("native animation updater requires Scene adoption before advancing")
                    return
            if not self.begun and not self.activate():
                return
            # Check before interpolation, helper updates AND endpoint finish.
            # Detached participants may have been adopted since registration.
            self.check_scene()
            elapsed = float(self.animation.total_time)
            if not math.isfinite(elapsed):
                raise ValueError("animation updater dt and total_time must be finite")
            duration = _duration(self.driver)
            if self.cycle and duration == 0:
                raise ValueError("a cycling animation updater requires positive run_time")
            if not self.cycle and (duration == 0 or elapsed >= duration):
                self.call("finish")
                if not self.closed:
                    self.closed = True
                    self.detach()
                    # Ordinary Scene.play keeps these until scene cleanup.
                    # Persistent helpers must not remove/publish scene objects,
                    # but a completed group must not retain native children.
                    group_driver = self.g.get("_CompositionCallbackDriver")
                    if group_driver is not None:
                        for group in self.groups:
                            retained = group.__dict__.get("_composition_driver")
                            if isinstance(retained, group_driver) and retained.group is group:
                                group._composition_driver = None
                    self.driver = None
                    self.execution.release()
                    self.nodes = self.groups = ()
                return
            following = elapsed + delta
            if not math.isfinite(following):
                raise ValueError("animation updater accumulated time is not finite")
            alpha = (elapsed / duration) % 1.0 if self.cycle else max(0.0, elapsed / duration)
            # The helper's pinned contract is interpolate THEN helper update.
            self.call("interpolate", alpha)
            if self.closed:
                return
            self.call("update_mobjects", delta, checked=True)
            if not self.closed:
                self.animation.total_time = following
        except BaseException as error:
            if not awaiting_adoption:
                if self.closed and self.driver is not None:
                    self._unwind(error)
                else:
                    self.cancel(error)
            raise
        finally:
            self.busy = False
            if self.cleanup_pending:
                self._unwind()


def _install_removal(g):
    Mobject = g["Mobject"]
    previous_remove = Mobject.remove_updater

    @wraps(previous_remove)
    def remove(self, updater):
        owner = getattr(updater, "_fmn_persistent_controller", None)
        if isinstance(owner, _PersistentAnimation) and owner.anchor is self and not owner.closed:
            owner.cancel()
        return previous_remove(self, updater)

    Mobject.remove_updater = remove
    previous_clear = getattr(Mobject, "clear_updaters", None)
    if previous_clear is None:
        return

    @wraps(previous_clear)
    def clear(self, recurse=True):
        first = None
        for member in self.get_family() if recurse else [self]:
            for updater in tuple(member.updaters):
                owner = getattr(updater, "_fmn_persistent_controller", None)
                if isinstance(owner, _PersistentAnimation) and owner.anchor is member:
                    try:
                        owner.cancel()
                    except BaseException as error:
                        if first is None:
                            first = error
        result = previous_clear(self, recurse=recurse)
        if first is not None:
            raise first
        return result

    Mobject.clear_updaters = clear


def install_animation_updaters(native: Any) -> None:
    """Install helpers without replacing any public animation class."""
    g = vars(native)
    if g.get("_FMN_ANIMATION_UPDATERS_INSTALLED", False):
        return
    Animation, Mobject = g["Animation"], g["Mobject"]
    previous = {name: g[name] for name in ("turn_animation_into_updater", "cycle_animation")}
    protocols = {
        cls: {name: _implementation(cls, name) for name in _NATIVE_HOOKS}
        for cls in tuple(g.values()) if isinstance(cls, type) and issubclass(cls, Animation)
    }
    _install_removal(g)

    def turn_animation_into_updater(animation, cycle=False, **kwargs):
        if not isinstance(animation, Animation):
            raise TypeError(
                "turn_animation_into_updater requires an Animation; got "
                + type(animation).__name__
            )
        current_anchor = getattr(animation, "mobject", None)
        if isinstance(current_anchor, Mobject):
            for updater in tuple(current_anchor.updaters):
                owner = getattr(updater, "_fmn_persistent_controller", None)
                if isinstance(owner, _PersistentAnimation) and not owner.closed and owner.animation is animation:
                    raise RuntimeError("animation already has a persistent updater")
        animation.update_rate_info(**kwargs)
        animation._ensure_runtime_defaults()
        if isinstance(animation, g["AnimationGroup"]):
            g["_fmn_ensure_composition_root"](animation)
        anchor = animation.mobject
        if not isinstance(anchor, Mobject):
            raise TypeError("animation updater must animate a Mobject")
        for updater in tuple(anchor.updaters):
            owner = getattr(updater, "_fmn_persistent_controller", None)
            if isinstance(owner, _PersistentAnimation) and not owner.closed and owner.animation is animation:
                raise RuntimeError("animation already has a persistent updater")
        factory = None
        if not _has_python_interpolation(g, animation):
            make_driver = g.get("_fmn_make_animation_driver")
            if make_driver is None:
                raise NotImplementedError(type(animation).__name__ + " requires a scene-bound native animation driver")
            def validate_native():
                if not _native_hooks_unchanged(animation, protocols):
                    raise NotImplementedError("native-only persistent animation has authored lifecycle hooks without a Python interpolator")
                if float(animation.final_alpha_value) != 1.0:
                    raise NotImplementedError("native-only persistent animation requires its default final_alpha_value=1")
            validate_native()
            def factory(scene):
                # A pending animation can be edited before adoption. Recheck
                # hooks/endpoints before freezing the native specification.
                validate_native()
                return make_driver(scene, animation)
        return _PersistentAnimation(g, animation, anchor, cycle, factory).start()

    def cycle_animation(animation, **kwargs):
        return turn_animation_into_updater(animation, cycle=True, **kwargs)

    replacements = {"turn_animation_into_updater": turn_animation_into_updater, "cycle_animation": cycle_animation}
    for name, function in replacements.items():
        old = previous[name]
        function.__name__ = function.__qualname__ = name
        function.__module__ = old.__module__
        for module in tuple(sys.modules.values()):
            module_name = getattr(module, "__name__", "")
            if module_name == "manimlib" or module_name.startswith("manimlib."):
                namespace = vars(module)
                for key, value in tuple(namespace.items()):
                    if value is old:
                        namespace[key] = function
        for key, value in tuple(g.items()):
            if value is old:
                g[key] = function
        g[name] = function
    g["_FMN_ANIMATION_UPDATERS_INSTALLED"] = True
