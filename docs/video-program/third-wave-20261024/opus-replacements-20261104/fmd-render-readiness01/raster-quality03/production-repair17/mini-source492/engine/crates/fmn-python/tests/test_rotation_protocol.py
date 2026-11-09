"""Authored rotation dispatch over fixture storage and the production adapter."""
import copy
import importlib.util
import inspect
from pathlib import Path
import sys
import types
import unittest

ROOT = Path(__file__).resolve().parents[1]
python_dir = str(ROOT / "python")
if python_dir not in sys.path:
    sys.path.insert(0, python_dir)
import fmn_python.movement as movement


def _method(cls, name, function):
    function.__name__ = name
    function.__qualname__ = cls.__qualname__ + "." + name
    function.__module__ = cls.__module__
    setattr(cls, name, function)


def _callback_rate(g, rate):
    if isinstance(rate, str):
        rate = next((fn for fn, label in g["_RATE_FUNC_NAMES"].items() if label == rate), None)
        if rate is None:
            raise ValueError("unknown rate function")
    if not callable(rate):
        raise TypeError("rate_func must be callable")
    return rate


def _cancel_preserving(animation, error):
    try:
        animation.abort()
    except BaseException as cleanup:
        error.add_note(str(cleanup))


def _install_lifecycle(g, cls, prepare):
    def release(self, update=False):
        del update
        self.mobject.set_animating_status(False)
        self.mobject_was_updating = False

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
        self._movement_active = True
        self._movement_finished = False
        self._ensure_runtime_defaults()
        prepare(self)
        try:
            super(cls, self).begin()
        except BaseException as error:
            _cancel_preserving(self, error)
            raise

    def finish(self):
        if not getattr(self, "_movement_active", False):
            if getattr(self, "_movement_finished", False):
                return
            raise RuntimeError("must begin")
        self.mobject_was_updating = False
        try:
            super(cls, self).finish()
            release(self, True)
        except BaseException as error:
            _cancel_preserving(self, error)
            raise
        self._movement_active = False
        self._movement_finished = True

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
        if getattr(self, "_movement_finished", False):
            return super(cls, self).clean_up_from_scene(scene)

    for name, function in {
        "begin": begin, "finish": finish, "abort": abort,
        "interpolate": interpolate, "update_mobjects": update_mobjects,
        "clean_up_from_scene": cleanup, "_ensure_runtime_defaults": defaults,
    }.items():
        _method(cls, name, function)


def _implementation(obj, name):
    value = inspect.getattr_static(obj, name, None)
    if isinstance(value, (staticmethod, classmethod, types.MethodType)):
        return value.__func__
    return value


def _protocols(g, root, names):
    classes = {
        base for cls in tuple(g.values())
        if isinstance(cls, type) and issubclass(cls, root)
        for base in cls.__mro__ if issubclass(base, root)
    }
    return {cls: {name: _implementation(cls, name) for name in names} for cls in classes}


def _changed(obj, protocols):
    found = False
    for cls in type(obj).__mro__:
        baseline = protocols.get(cls)
        if baseline is None:
            continue
        if not found:
            found = True
            if any(_implementation(obj, name) is not expected for name, expected in baseline.items()):
                return True
        if any(_implementation(cls, name) is not expected for name, expected in baseline.items()):
            return True
    return not found


def _custom_rate(g, rate):
    return rate is not None and not isinstance(rate, str) and all(
        rate is not function for function in g["_RATE_FUNC_NAMES"]
    )


import fmn_python.rotation as adapter


def environment():
    native = types.ModuleType("manimlib.rotation_fixture")
    g = vars(native)

    def linear(t):
        return t

    def smooth(t):
        return t * t * (3 - 2 * t)

    class Mobject:
        def __init__(self, value=1, *children):
            self.value = float(value)
            self.submobjects = list(children)
            self.animating = False
        def copy(self):
            return copy.deepcopy(self)
        def get_family(self):
            result = [self]
            for child in self.submobjects:
                result.extend(child.get_family())
            return result
        def family_members_with_points(self):
            return self.get_family()
        def match_points(self, other):
            self.value = other.value
            return self
        def rotate(self, angle, axis=None, about_point=None, about_edge=None):
            del axis, about_point, about_edge
            self.value += float(angle)
            return self
        def set_animating_status(self, value):
            self.animating = bool(value)
        def _is_updating_suspended(self):
            return False
        def suspend_updating(self, recurse=True):
            del recurse
        def resume_updating(self, recurse=True, call_updater=True):
            del recurse, call_updater
        def update(self, dt=0):
            del dt
            return self

    class Animation:
        _native_kind = None
        def __init__(self, mobject, run_time=1, rate_func=None, lag_ratio=0,
                     time_span=None, final_alpha_value=1, remover=False,
                     suspend_mobject_updating=False, **kwargs):
            del kwargs
            self.mobject = mobject
            self.run_time = run_time
            self.rate_func = rate_func or smooth
            self.lag_ratio = lag_ratio
            self.time_span = time_span
            self.final_alpha_value = final_alpha_value
            self.remover = remover
            self.suspend_mobject_updating = suspend_mobject_updating
        def _ensure_runtime_defaults(self):
            if self.run_time is None:
                self.run_time = 1
            if self.rate_func is None:
                self.rate_func = smooth
        def create_starting_mobject(self):
            return self.mobject.copy()
        def begin(self):
            self._ensure_runtime_defaults()
            self.mobject.set_animating_status(True)
            self.starting_mobject = self.create_starting_mobject()
            self.families = list(self.get_all_families_zipped())
            self.interpolate(0)
        def finish(self):
            self.interpolate(self.final_alpha_value)
            self.mobject.set_animating_status(False)
        def interpolate(self, alpha):
            return self.interpolate_mobject(float(alpha))
        def interpolate_mobject(self, alpha):
            del alpha
        def interpolate_submobject(self, *args):
            del args
        def update_mobjects(self, dt):
            for mobject in self.get_all_mobjects_to_update():
                mobject.update(dt)
        def get_all_mobjects(self):
            return self.mobject, self.starting_mobject
        def get_all_families_zipped(self):
            return zip(*(mobject.get_family() for mobject in self.get_all_mobjects()))
        def get_all_mobjects_to_update(self):
            return [mobject for mobject in self.get_all_mobjects() if mobject is not self.mobject]
        def get_sub_alpha(self, alpha, index, total):
            del index, total
            return self.rate_func(alpha)
        def time_spanned_alpha(self, alpha):
            if self.time_span is None:
                return alpha
            start, end = self.time_span
            return min(max(alpha * self.run_time - start, 0), end - start) / (end - start)
        def clean_up_from_scene(self, scene):
            if self.remover:
                scene.remove(self.mobject)

    class Rotating(Animation):
        _native_kind = "rotating"
        def __init__(self, mobject, angle=6, axis=(0, 0, 1), about_point=None,
                     about_edge=None, run_time=5, rate_func=linear, **kwargs):
            super().__init__(mobject, run_time=run_time, rate_func=rate_func, **kwargs)
            self.angle, self.axis = float(angle), axis
            self.about_point, self.about_edge = about_point, about_edge
        def interpolate_mobject(self, alpha):
            for current, starting in zip(
                self.mobject.family_members_with_points(),
                self.starting_mobject.family_members_with_points(),
            ):
                current.match_points(starting)
            self.mobject.rotate(
                self.rate_func(self.time_spanned_alpha(float(alpha))) * self.angle,
                axis=self.axis, about_point=self.about_point, about_edge=self.about_edge,
            )

    class Rotate(Rotating):
        _native_kind = "rotate"

    class AnimationGroup(Animation):
        _native_kind = "animation_group"
        def __init__(self, *animations):
            self.animations = list(animations)
            super().__init__(Mobject(*[animation.mobject for animation in animations]))

    class Builder:
        pass

    class Scene:
        def __init__(self):
            self.calls = []
            self.removed = []
        def play(self, *animations, **kwargs):
            result = [g["_requires_python_animation"](animation) for animation in animations]
            self.calls.append((animations, kwargs, result))
            return self.calls[-1]
        def remove(self, mobject):
            self.removed.append(mobject)

    def requires(animation):
        return not getattr(animation, "_native_kind", None)

    g.update(
        Mobject=Mobject, Animation=Animation, Rotating=Rotating, Rotate=Rotate,
        AnimationGroup=AnimationGroup, _AnimationBuilder=Builder, Scene=Scene,
        _requires_python_animation=requires,
        _RATE_FUNC_NAMES={linear: "linear", smooth: "smooth"},
        prepare_animation=lambda value: value,
    )
    return native


class RotationProtocolTests(unittest.TestCase):
    def setUp(self):
        self.native = environment()
        adapter.install_rotation(self.native)
        self.mob = self.native.Mobject()
        self.scene = self.native.Scene()

    def test_stock_rotation_stays_native(self):
        animation = self.native.Rotating(self.mob)
        self.scene.play(animation)
        self.assertEqual(self.scene.calls[-1][2], [False])

    def test_custom_rate_and_endpoint_options_route_callback(self):
        self.assertTrue(vars(self.native)["_requires_python_animation"](
            self.native.Rotating(self.mob, rate_func=lambda t: t * t)
        ))
        self.assertTrue(vars(self.native)["_requires_python_animation"](
            self.native.Rotate(self.mob, final_alpha_value=.5)
        ))
        self.assertTrue(vars(self.native)["_requires_python_animation"](
            self.native.Rotate(self.mob, remover=True)
        ))

    def test_subclass_and_instance_hooks_route_callback(self):
        parent = self.native.Rotate
        class Custom(parent):
            def interpolate_mobject(self, alpha):
                return super().interpolate_mobject(alpha)
        self.assertTrue(vars(self.native)["_requires_python_animation"](Custom(self.mob)))
        animation = self.native.Rotate(self.mob)
        animation.interpolate = lambda alpha: None
        self.assertTrue(vars(self.native)["_requires_python_animation"](animation))

    def test_mobject_override_routes_without_executing_it_during_classification(self):
        parent = self.native.Mobject
        class Custom(parent):
            calls = 0
            def rotate(self, *args, **kwargs):
                type(self).calls += 1
                return super().rotate(*args, **kwargs)
        mob = Custom()
        self.assertTrue(vars(self.native)["_requires_python_animation"](
            self.native.Rotate(mob)
        ))
        self.assertEqual(Custom.calls, 0)

    def test_child_hook_routes_callback(self):
        parent = self.native.Mobject
        class Child(parent):
            def match_points(self, other):
                return super().match_points(other)
        root = parent(1, Child())
        self.assertTrue(vars(self.native)["_requires_python_animation"](
            self.native.Rotate(root)
        ))

    def test_callback_lifecycle_rebuilds_from_start_and_honors_final_alpha(self):
        animation = self.native.Rotate(
            self.mob, angle=4, rate_func=lambda t: t, final_alpha_value=.25
        )
        animation.begin()
        self.mob.value = 99
        animation.interpolate(.5)
        self.assertAlmostEqual(self.mob.value, 3)
        animation.finish()
        self.assertAlmostEqual(self.mob.value, 2)
        self.assertFalse(self.mob.animating)

    def test_catalog_string_resolves_on_callback_route(self):
        animation = self.native.Rotate(
            self.mob, angle=4, rate_func="linear", final_alpha_value=.5
        )
        animation.begin()
        animation.finish()
        self.assertAlmostEqual(self.mob.value, 3)

    def test_base_animation_monkeypatch_is_observable(self):
        animation = self.native.Rotate(self.mob)
        self.native.Animation.update_mobjects = lambda self, dt: None
        self.assertTrue(vars(self.native)["_requires_python_animation"](animation))

    def test_abort_releases_animating_state(self):
        animation = self.native.Rotate(self.mob, final_alpha_value=.5)
        animation.begin()
        self.assertTrue(self.mob.animating)
        animation.abort()
        self.assertFalse(self.mob.animating)

    def test_reinstall_is_idempotent(self):
        play = self.native.Scene.play
        requires = vars(self.native)["_requires_python_animation"]
        adapter.install_rotation(self.native)
        self.assertIs(self.native.Scene.play, play)
        self.assertIs(vars(self.native)["_requires_python_animation"], requires)


if __name__ == "__main__":
    unittest.main()
