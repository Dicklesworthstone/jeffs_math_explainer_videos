"""Failure ownership at the existing Scene/Choreo callback boundary.

Choreo still lowers animations, samples time, runs native updaters and emits
frames. This adapter releases Python-owned animation transients after failures;
it neither rewinds geometry/clock/output nor implements another frame loop.
"""
from __future__ import annotations

from contextlib import nullcontext

from functools import wraps
import math
from typing import Any


_MISSING = object()
_LOCKS = ("locked_data_keys", "const_data_keys", "locked_uniform_keys")
_OWNER_KEY = "_fmn_scene_execution"


def _note(primary, message):
    # An authored exception may itself have a broken __str__/add_note method.
    try:
        BaseException.add_note(primary, message)
    except BaseException:
        pass


def _roots(callback, g):
    """Inspect only known animation containers, never arbitrary attributes."""
    pending, seen = [callback], set()
    animation_type = g["Animation"]
    composition_type = g.get("_CompositionCallbackDriver", ())
    while pending:
        current = pending.pop()
        if id(current) in seen:
            continue
        seen.add(id(current))
        root = getattr(current, "mobject", None)
        if isinstance(root, g["Mobject"]):
            yield root
        if isinstance(current, (g.get("AnimationGroup", ()),
                                g.get("_fmn_camera_clock_driver_type", ()))):
            # A mixed camera play is lowered to one private clock slot. Its
            # public animations (including nested cameras and drawable
            # siblings) own the real transients, not the empty slot.
            pending.extend(reversed(tuple(current.animations)))
        elif isinstance(current, composition_type):
            pending.append(current.group)
            pending.extend(reversed(tuple(current.children)))
        elif not isinstance(current, animation_type):
            # Native leaf drivers expose no Python Mobject to snapshot. Their
            # own abort method remains the native state authority.
            continue


class _TransientState:
    def __init__(self, mob):
        self.mob = mob
        self.suspended = mob._is_updating_suspended()
        attrs = vars(mob)
        self.animating = attrs.get("_is_animating", _MISSING)
        self.raw_animating = attrs.get("animating", _MISSING)
        self.locks = {name: (attrs[name], set(attrs[name])) for name in _LOCKS if name in attrs}
        self.absent_locks = tuple(name for name in _LOCKS if name not in attrs)

    def restore(self, primary):
        mob = self.mob
        first_error = None
        # Independent best efforts: a stale native handle must not prevent
        # recovery of another family member or its Python lock projection.
        try:
            now = mob._is_updating_suspended()
            if self.suspended and not now:
                mob.suspend_updating(recurse=False)
            elif not self.suspended and now:
                mob.resume_updating(recurse=False, call_updater=False)
        except BaseException as error:
            first_error = error
            _note(primary, "animation suspension recovery failed: " + type(error).__name__)
        try:
            attrs = vars(mob)
            if self.animating is _MISSING:
                attrs.pop("_is_animating", None)
            else:
                attrs["_is_animating"] = self.animating
            if self.raw_animating is _MISSING:
                attrs.pop("animating", None)
            else:
                attrs["animating"] = self.raw_animating
            for name, (original, values) in self.locks.items():
                original.clear()
                original.update(values)
                attrs[name] = original
            for name in self.absent_locks:
                attrs.pop(name, None)
        except BaseException as error:
            if first_error is None:
                first_error = error
            _note(primary, "animation lock recovery failed: " + type(error).__name__)
        return first_error


class _Callback:
    """Transparent backend callback handle; authored hooks keep their receiver."""
    def __init__(self, callback, owner):
        self.callback, self.owner = callback, owner
        self.finished = False

    def __getattr__(self, name):
        return getattr(self.callback, name)

    def begin(self):
        # Record before calling: begin itself can fail after suspending state.
        self.owner.begun.append(self)
        return self.callback.begin()

    def finish(self):
        result = self.callback.finish()
        self.finished = True
        return result


class _Execution:
    def __init__(self, g):
        self.g = g
        self.begun = []
        self.snapshots = {}
        self.primary = None
        self.native_active = False
        self.closed = False

    def wrap(self, callbacks):
        result = []
        # Snapshot the whole affected family BEFORE the first callback begins.
        # Two animations may share a descendant; later begin-time snapshots
        # would otherwise mistake the first animation's suspension as prior.
        for callback in callbacks:
            if callback is None:
                result.append(None)
                continue
            for root in _roots(callback, self.g):
                family = list(root.get_family())
                # set_animating_status propagates upward, including parents
                # outside the animated root. Only those ancestors' transients
                # are captured; siblings are not traversed.
                index = 0
                while index < len(family):
                    mob = family[index]
                    index += 1
                    if id(mob) in self.snapshots:
                        continue
                    self.snapshots[id(mob)] = _TransientState(mob)
                    family.extend(tuple(getattr(mob, "parents", ())))
            result.append(_Callback(callback, self))
        return result

    def release(self):
        # Drop proxy/callback owners on the calling thread, not a later GC
        # pass on an unrelated thread (native proxies are unsendable).
        self.begun.clear()
        self.snapshots.clear()
        self.primary = None

    def fail(self, error):
        if self.primary is None:
            self.primary = error

    def unwind(self, error):
        if self.closed:
            return
        self.closed = True
        first_error = None
        seen = set()
        for handle in reversed(self.begun):
            callback = handle.callback
            if id(callback) in seen:
                continue
            seen.add(id(callback))
            if not handle.finished:
                try:
                    abort = getattr(callback, "abort", None)
                    if callable(abort) and not getattr(abort, "_fmn_schema_placeholder", False):
                        abort()
                except BaseException as cleanup_error:
                    if first_error is None:
                        first_error = cleanup_error
                    _note(error, "animation abort also failed: " + type(cleanup_error).__name__)
        # Restore only after all specialized/legacy abort handlers have run.
        if self.begun:
            for snapshot in reversed(tuple(self.snapshots.values())):
                recovery_error = snapshot.restore(error)
                if first_error is None:
                    first_error = recovery_error
        # Scene failures preserve their primary exception; explicit persistent
        # cancellation has no primary and must surface a failed abort/recovery.
        return first_error


def install_scene_execution(native: Any) -> None:
    """Install after the animation playback adapters, preserving class aliases."""
    g = vars(native)
    if g.get("_FMN_SCENE_EXECUTION_INSTALLED", False):
        return
    Scene = g["Scene"]
    original_play, original_core = Scene.play, Scene._play_animations
    original_wait = Scene.wait

    def enter(scene):
        if vars(scene).get(_OWNER_KEY) is not None:
            raise RuntimeError("cannot start another play/wait while this Scene is executing one")
        owner = _Execution(g)
        vars(scene)[_OWNER_KEY] = owner
        return owner

    def leave(scene, owner):
        owner.release()
        if vars(scene).get(_OWNER_KEY) is owner:
            vars(scene).pop(_OWNER_KEY)

    @wraps(original_core)
    def core(self, specs, callbacks, camera, run_time, rate_func, lag_ratio):
        owner = vars(self).get(_OWNER_KEY)
        independent = owner is None
        if independent:
            owner = enter(self)
        if owner.native_active:
            raise RuntimeError("cannot reenter this Scene's active native animation segment")
        try:
            wrapped = owner.wrap(callbacks)
            owner.native_active = True
            return original_core(self, specs, wrapped, camera, run_time, rate_func, lag_ratio)
        except BaseException as error:
            owner.fail(error)
            if independent:
                owner.unwind(error)
            raise
        finally:
            owner.native_active = False
            if independent:
                leave(self, owner)

    def execute(scene, operation, args, kwargs, prepare):
        # Acquire a clip before entering the execution boundary, and finish it
        # only after all animation locks have been released. The same native
        # recording generation handles frames; this is not another scheduler.
        if vars(scene).get(_OWNER_KEY) is not None:
            raise RuntimeError("cannot start another play/wait while this Scene is executing one")
        subdivision = vars(scene).get("_fmn_subdivision_session")
        capture = (nullcontext(None) if subdivision is None else
                   subdivision._segment("wait" if operation is original_wait else "play"))
        with capture as segment:
            owner = enter(scene)
            try:
                args, kwargs, hooks = prepare(scene, args, kwargs)
                if hooks:
                    scene.pre_play()
                if segment is not None:
                    # Range selection belongs to pre_play. In particular, do
                    # not publish an endpoint still for skipped preroll or an
                    # empty play(), and do not run authored pre_play twice.
                    segment.select(hooks and not scene.skip_animations)
                result = operation(scene, *args, **kwargs)
                if hooks:
                    scene.post_play()
                return result
            except BaseException as error:
                # An older animation-specific wrapper may throw while unwinding.
                # Keep the execution failure, not that secondary cleanup failure.
                primary = owner.primary if owner.primary is not None else error
                if primary is not error:
                    _note(primary, "outer animation cleanup also failed: " + type(error).__name__)
                owner.unwind(primary)
                if primary is error:
                    raise
                raise primary from None
            finally:
                leave(scene, owner)

    def prepare_play(scene, animations, kwargs):
        unknown = set(kwargs) - {"run_time", "rate_func", "lag_ratio"}
        if unknown:
            raise TypeError("Scene.play unexpected keyword(s): " + ", ".join(sorted(unknown)))
        if not animations:
            return animations, kwargs, False
        # Builders produce real Animations now, not a restricted native spec.
        # Let the selected constructor validate its options: a static allowlist
        # here rejected live paths, time windows, suspension, remover/final-alpha
        # semantics and even valid authored build() overrides. Prepare once in
        # argument order, before pre_play or native work; never filter kwargs or
        # reconstruct the returned animation and lose its authored identity.
        builder_cls = g.get("_AnimationBuilder")
        valid_types = (g["Animation"], builder_cls) if builder_cls is not None else (g["Animation"],)
        for item in animations:
            if not isinstance(item, valid_types):
                # Reference animation.py:216 (prepare_animation) rejects the
                # same inputs, bound Mobject methods included, this way.
                raise TypeError(f"Object {item} cannot be converted to an animation")
        prepared = tuple(g["prepare_animation"](item) for item in animations)
        if not all(isinstance(item, g["Animation"]) for item in prepared):
            raise TypeError("prepare_animation must return an Animation")

        for key in ("run_time", "lag_ratio"):
            value = kwargs.get(key)
            if value is not None:
                value = float(value)
                if not math.isfinite(value) or (key == "run_time" and value < 0):
                    raise ValueError("Scene.play " + key + " must be finite"
                                     + (" and non-negative" if key == "run_time" else ""))
                kwargs[key] = value
        rate = kwargs.get("rate_func")
        if rate is not None and not isinstance(rate, str) and not callable(rate):
            raise TypeError("rate_func must be a callable or a catalog name")
        for animation in prepared:
            animation.update_rate_info(**kwargs)
        return prepared, kwargs, True

    @wraps(original_play)
    def play(self, *animations, **kwargs):
        return execute(self, original_play, animations, kwargs, prepare_play)

    @wraps(original_wait)
    def wait(self, duration=None, stop_condition=None, note=None,
             ignore_presenter_mode=False, **kwargs):
        def prepare_wait(scene, args, options):
            if options:
                raise NotImplementedError("Scene.wait unsupported keyword(s): " + ", ".join(sorted(options)))
            if stop_condition is not None and not callable(stop_condition):
                raise TypeError(
                    "Scene.wait stop_condition must be callable or None; got "
                    + type(stop_condition).__name__
                )
            value = float(scene.default_wait_time if duration is None else duration)
            if not math.isfinite(value) or value < 0:
                raise ValueError("Scene.wait duration must be finite and non-negative")
            return (value, stop_condition), {
                "note": note, "ignore_presenter_mode": ignore_presenter_mode,
            }, True
        return execute(self, original_wait, (), kwargs, prepare_wait)

    # The stock writer's partial-movie hooks used to refuse unconditionally.
    # An acquired native subdivision now owns exactly those boundaries. Keep
    # authored overrides and all ordinary writer paths on their original route;
    # never suppress a hook merely because subdivide_output was requested.
    writer_type = g.get("SceneFileWriter")
    if writer_type is not None:
        def owned_writer_hook(original):
            @wraps(original)
            def hook(writer):
                if writer.subdivide_output and writer.write_to_movie:
                    namespace = vars(writer.scene)
                    collection = namespace.get("_fmn_subdivision_session")
                    recording = getattr(collection, "_current", None)
                    if (recording is not None
                            and namespace.get("_fmn_owned_render_session") is recording):
                        return None
                return original(writer)
            return hook
        for name in ("begin_animation", "end_animation"):
            setattr(writer_type, name, owned_writer_hook(getattr(writer_type, name)))

    Scene._play_animations = core
    Scene.play, Scene.wait = play, wait
    g["_FMN_SCENE_EXECUTION_INSTALLED"] = True
