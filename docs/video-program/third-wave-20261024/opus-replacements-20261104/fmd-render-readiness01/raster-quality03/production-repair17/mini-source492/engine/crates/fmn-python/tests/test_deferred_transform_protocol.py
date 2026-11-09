"""Deferred target/path orchestration; storage is doubled, not native proof."""
from __future__ import annotations

import importlib.util
import math
from pathlib import Path
from types import SimpleNamespace
import unittest

import numpy as np


SOURCE = Path(__file__).resolve().parents[1] / "python/fmn_python/playback.py"
spec = importlib.util.spec_from_file_location("deferred_playback_under_test", SOURCE)
playback = importlib.util.module_from_spec(spec)
spec.loader.exec_module(playback)


def table():
    calls = []

    class Mobject:
        def __init__(self, value=0):
            self.value = value

        def copy(self):
            calls.append(("copy", self.value))
            return type(self)(self.value)

        def shift(self, amount, multiplier=1):
            self.value += amount * multiplier
            return self

        def apply_complex_function(self, function):
            self.value = function(complex(self.value))
            return self

        def get_center(self):
            return self.value

        def move_to(self, point):
            self.value = point
            return self

    class Animation:
        pass

    class Transform(Animation):
        _native_kind = "transform"
        _target_attr = "target_mobject"

        def __init__(self, mobject, target_mobject=None, path_arc=0,
                     path_arc_axis=(0, 0, 1), path_func=None, **kwargs):
            self.mobject, self.target_mobject = mobject, target_mobject
            self.path_arc, self.path_arc_axis = path_arc, path_arc_axis
            self.path_func = path_func
            self.__dict__.update(kwargs)

        def create_target(self):
            return self.target_mobject

        def init_path_func(self):
            if self.path_func is not None:
                return
            calls.append(("path", self.path_arc, self.path_arc_axis))
            self.path_func = lambda start, end, alpha: None

        def begin(self):
            self.init_path_func()
            self.target_mobject = self.create_target()

    class ApplyMethod(Transform):
        def __init__(self, method, *args, **kwargs):
            self.method, self.method_args = method, args
            super().__init__(method.__self__, **kwargs)

        def create_target(self):
            args = list(self.method_args)
            kwargs = args.pop() if args and isinstance(args[-1], dict) else {}
            target = self.method.__self__.copy()
            self.method.__func__(target, *args, **kwargs)
            return target

    class ApplyFunction(Transform):
        def __init__(self, function, mobject, **kwargs):
            self.function = function
            super().__init__(mobject, **kwargs)

        def create_target(self):
            return self.function(self.mobject.copy())

    class ApplyPointwiseFunctionToCenter(ApplyFunction):
        def create_target(self):
            return self.mobject.copy().move_to(self.function(self.mobject.get_center()))

    class ApplyComplexFunction(ApplyMethod):
        def __init__(self, function, mobject, **kwargs):
            self.function = function
            super().__init__(mobject.apply_complex_function, function, **kwargs)

        def init_path_func(self):
            # The previous bootstrap protocol derived an angle but no path.
            self.path_arc = float(np.log(complex(self.function(complex(1)))).imag)

    class Scene:
        def play(self, *animations, **kwargs):
            return animations

    class Restore(Animation):
        pass

    class Builder:
        pass

    def requires(animation):
        calls.append(("dispatch", animation))
        return False

    return SimpleNamespace(
        Mobject=Mobject, Animation=Animation, Transform=Transform,
        ApplyMethod=ApplyMethod, ApplyFunction=ApplyFunction,
        ApplyPointwiseFunctionToCenter=ApplyPointwiseFunctionToCenter,
        ApplyComplexFunction=ApplyComplexFunction, _np=np,
        _requires_python_animation=requires, calls=calls,
        Scene=Scene, Restore=Restore, _AnimationBuilder=Builder, _OUT=(0, 0, 1),
    )


def plan(native, animation):
    """Model the real driver's decision without substituting its frame clock."""
    if native._requires_python_animation(animation):
        return animation.begin
    # Native-spec planning consumes an eager target before any leaf begins.
    target = animation.create_target()
    return lambda: setattr(animation, "target_mobject", target)


class DeferredTransformProtocol(unittest.TestCase):
    def setUp(self):
        self.native = table()
        playback._install_deferred_transforms(vars(self.native))

    def test_methods_sample_predecessor_state_and_keep_kwargs(self):
        n = self.native
        source = n.Mobject()
        kwargs = {"multiplier": 3}
        animation = n.ApplyMethod(source.shift, 2, kwargs)
        begin = plan(n, animation)
        self.assertEqual(n.calls, [])
        source.value = 10
        begin()
        self.assertEqual(animation.target_mobject.value, 16)
        self.assertEqual(source.value, 10)
        self.assertIs(animation.method_args[-1], kwargs)
        self.assertEqual([call for call in n.calls if call[0] == "copy"], [("copy", 10)])

    def test_successive_methods_do_not_share_an_eager_snapshot(self):
        n = self.native
        source = n.Mobject()
        first, second = n.ApplyMethod(source.shift, 2), n.ApplyMethod(source.shift, 2)
        starts = [plan(n, animation) for animation in (first, second)]
        starts[0]()
        source.value = first.target_mobject.value  # The predecessor finishes.
        starts[1]()
        self.assertEqual(first.target_mobject.value, 2)
        self.assertEqual(second.target_mobject.value, 4)

    def test_eager_dispatch_negative_control_exposes_stale_target(self):
        n = table()
        source = n.Mobject()
        animation = n.ApplyMethod(source.shift, 2)
        begin = plan(n, animation)
        source.value = 10
        begin()
        self.assertEqual(animation.target_mobject.value, 2)
        self.assertNotEqual(animation.target_mobject.value, 12)

    def test_apply_function_waits_to_call_the_authored_function(self):
        n = self.native
        values = []
        def function(target):
            values.append(target.value)
            return target.shift(3)
        source = n.Mobject(1)
        animation = n.ApplyFunction(function, source)
        begin = plan(n, animation)
        self.assertEqual(values, [])
        source.value = 8
        begin()
        self.assertEqual(values, [8])
        self.assertEqual(animation.target_mobject.value, 11)
        self.assertIs(animation.function, function)

    def test_center_function_uses_current_center(self):
        n = self.native
        source = n.Mobject(1)
        animation = n.ApplyPointwiseFunctionToCenter(lambda value: 2 * value, source)
        begin = plan(n, animation)
        source.value = 7
        begin()
        self.assertEqual(animation.target_mobject.value, 14)
        self.assertEqual(source.value, 7)

    def test_derived_method_families_are_deferred_even_after_a_previous_begin(self):
        n = self.native
        class Derived(n.ApplyMethod):
            pass
        animation = Derived(n.Mobject().shift, 2)
        animation.begin()
        self.assertTrue(n._requires_python_animation(animation))
        animation.mobject.value = 8
        plan(n, animation)()
        self.assertEqual(animation.target_mobject.value, 10)

    def test_explicit_targets_keep_the_existing_dispatch(self):
        n = self.native
        target = n.Mobject(9)
        animation = n.Transform(n.Mobject(), target)
        self.assertFalse(n._requires_python_animation(animation))
        self.assertIs(animation.target_mobject, target)
        self.assertEqual(n.calls, [("dispatch", animation)])

    def test_complex_rotation_initializes_the_shared_path_factory(self):
        n = self.native
        animation = n.ApplyComplexFunction(lambda z: 1j * z, n.Mobject(2))
        self.assertTrue(n._requires_python_animation(animation))
        animation.begin()
        self.assertAlmostEqual(animation.path_arc, math.pi / 2)
        self.assertTrue(callable(animation.path_func))
        self.assertEqual(animation.target_mobject.value, 2j)
        self.assertAlmostEqual(n.calls[0][1], math.pi / 2)
        self.assertEqual(n.calls[0][0], "path")

    def test_missing_super_negative_control_leaves_the_path_uninitialized(self):
        n = table()
        animation = n.ApplyComplexFunction(lambda z: 1j * z, n.Mobject(2))
        animation.begin()
        self.assertAlmostEqual(animation.path_arc, math.pi / 2)
        self.assertIsNone(animation.path_func)

    def test_complex_axis_and_existing_authored_path_are_preserved(self):
        n = self.native
        path = lambda start, end, alpha: start
        animation = n.ApplyComplexFunction(lambda z: -1j * z, n.Mobject(),
                                            path_arc_axis=(1, 0, 0), path_func=path)
        animation.init_path_func()
        self.assertIs(animation.path_func, path)
        self.assertEqual(animation.path_arc_axis, (1, 0, 0))
        self.assertAlmostEqual(animation.path_arc, -math.pi / 2)
        self.assertEqual(n.calls, [])

    def test_complex_path_dispatch_remains_cooperative(self):
        n = self.native
        calls = []
        class Custom(n.ApplyComplexFunction):
            def init_path_func(self):
                calls.append("child")
                super().init_path_func()
        n.Transform.init_path_func = lambda self: calls.append("parent")
        Custom(lambda z: z, n.Mobject()).init_path_func()
        self.assertEqual(calls, ["child", "parent"])

    def test_complex_probe_error_propagates_unchanged(self):
        n = self.native
        failure = RuntimeError("bad map")
        def function(z):
            raise failure
        animation = n.ApplyComplexFunction(function, n.Mobject())
        with self.assertRaises(RuntimeError) as context:
            animation.init_path_func()
        self.assertIs(context.exception, failure)
        self.assertIsNone(animation.path_func)
        self.assertEqual(n.calls, [])

    def test_partial_native_tables_need_no_extra_exports(self):
        marker = lambda animation: False
        native = {"_requires_python_animation": marker}
        playback._install_deferred_transforms(native)
        self.assertIs(native["_requires_python_animation"], marker)

    def test_wheel_installer_activates_once_without_replacing_public_classes(self):
        n = table()
        original = n.ApplyComplexFunction
        playback.install_scene_playback(n)
        function = n.ApplyComplexFunction.init_path_func
        dispatch, play = n._requires_python_animation, n.Scene.play
        playback.install_scene_playback(n)
        self.assertIs(n.ApplyComplexFunction, original)
        self.assertIs(n.ApplyComplexFunction.init_path_func, function)
        self.assertIs(n._requires_python_animation, dispatch)
        self.assertIs(n.Scene.play, play)
        self.assertEqual(function.__module__, original.__module__)
        self.assertEqual(function.__qualname__, original.__qualname__ + ".init_path_func")
        animation = n.ApplyComplexFunction(lambda z: 1j * z, n.Mobject(2))
        self.assertTrue(n._requires_python_animation(animation))
        animation.begin()
        self.assertTrue(callable(animation.path_func))


if __name__ == "__main__":
    unittest.main()
