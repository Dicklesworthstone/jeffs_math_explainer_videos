"""Speed-profile and lifecycle tests; no extension needed for protocol tests.

Run: python crates/fmn-python/tests/test_speed.py
Also exercise the installed native portal: FMN_TEST_NATIVE=1 python <this file>
The protocol doubles test adapter ownership, not native geometry or rendering.
"""
from __future__ import annotations

import importlib.util
import math
import os
from pathlib import Path
import types
import unittest

_SOURCE = Path(__file__).resolve().parents[1] / "python/fmn_python/speed.py"
_spec = importlib.util.spec_from_file_location("speed_under_test", _SOURCE)
speed = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(speed)


def protocol():
    """Only the shared group's public protocol; no alternative production loop."""
    def linear(alpha):
        return alpha

    class Animation:
        def __init__(self, duration=2.0, rate_func=linear):
            self.run_time, self.rate_func = duration, rate_func
            self.mobject = object()
            self.events = []
            self.failure = None

        def get_run_time(self):
            return self.run_time

        def begin(self):
            self.events.append("begin")

        def interpolate(self, alpha):
            self.events.append(("alpha", self.rate_func(alpha)))
            if self.failure is not None:
                raise self.failure

        def update_mobjects(self, dt):
            self.events.append(("dt", dt))

        def finish(self):
            self.events.append("finish")

        def abort(self):
            self.events.append("abort")

        def clean_up_from_scene(self, scene):
            self.events.append(("cleanup", scene))

    class NativeAnimation(Animation):
        pass

    class Builder:
        def __init__(self, child):
            self.child, self.calls = child, 0

        def build(self):
            self.calls += 1
            return self.child

    class AnimationGroup(NativeAnimation):
        def __init__(self, *animations, run_time=-1, rate_func=None, **kwargs):
            self.animations = [a.build() if isinstance(a, Builder) else a for a in animations]
            if not all(isinstance(a, Animation) for a in self.animations):
                raise TypeError("expected Animation")
            self.rate_func = rate_func or linear
            self.run_time = max(a.get_run_time() for a in self.animations)
            self.mobject = object()
            self._composition_driver = None

        def begin(self):
            self._composition_driver = tuple(self.animations)
            for child in self._composition_driver:
                child.begin()
            self.interpolate(0.0)

        def interpolate(self, alpha):
            mapped = self.rate_func(alpha)
            for child in self._composition_driver:
                child.interpolate(mapped)

        def update_mobjects(self, dt):
            for child in self._composition_driver:
                child.update_mobjects(dt)

        def finish(self):
            for child in self._composition_driver:
                child.finish()

        def abort(self):
            children, self._composition_driver = self._composition_driver, None
            if children:
                for child in children:
                    child.abort()

        def clean_up_from_scene(self, scene):
            for child in self._composition_driver:
                child.clean_up_from_scene(scene)
            self._composition_driver = None

    class ChangeSpeed(Animation):
        pass

    module = types.SimpleNamespace(
        ChangeSpeed=ChangeSpeed, AnimationGroup=AnimationGroup, Animation=Animation,
        Builder=Builder, _linear_rate=linear, _RATE_FUNC_NAMES={linear: "linear"},
        _composition_member_run_time=lambda child: child.get_run_time(),
    )
    speed.install_speed(module)
    return module


class ProfileTests(unittest.TestCase):
    def test_identity_and_clamping(self):
        profile = speed._SpeedProfile()
        self.assertEqual(profile.total_time, 1.0)
        for alpha in (0.0, 0.1, 0.25, 0.8, 1.0):
            self.assertAlmostEqual(profile.map(alpha), alpha)
        self.assertEqual(profile.map(-3), 0)
        self.assertEqual(profile.map(3), 1)

    def test_constant_speed_changes_duration_not_normalized_progress(self):
        profile = speed._SpeedProfile({0: 2, 1: 2})
        self.assertEqual(profile.total_time, 0.5)
        self.assertAlmostEqual(profile.map(0.25), 0.25)

    def test_accelerating_from_rest(self):
        profile = speed._SpeedProfile({0: 0, 1: 2})
        self.assertEqual(profile.total_time, 1.0)
        self.assertEqual(profile.map(0.5), 0.25)

    def test_decelerating_to_rest(self):
        self.assertEqual(speed._SpeedProfile({0: 2, 1: 0}).map(0.5), 0.75)

    def test_defaults_and_input_not_mutated(self):
        original = {0.5: 2}
        profile = speed._SpeedProfile(original)
        self.assertEqual(original, {0.5: 2})
        self.assertEqual(profile.nodes, ((0.0, 1.0), (0.5, 2.0), (1.0, 2.0)))
        original[0.5] = 50
        self.assertEqual(profile.nodes[1][1], 2)

    def test_piecewise_duration_knots_and_monotonicity(self):
        profile = speed._SpeedProfile({0: 1, 0.25: 1, 0.75: 0.25, 1: 0.25})
        self.assertAlmostEqual(profile.total_time, 2.05)
        self.assertAlmostEqual(profile.map(0.25 / 2.05), 0.25)
        self.assertAlmostEqual(profile.map(1.05 / 2.05), 0.75)
        values = [profile.map(i / 10000) for i in range(10001)]
        self.assertEqual(values, sorted(values))
        self.assertEqual((values[0], values[-1]), (0, 1))

    def test_large_finite_endpoint_speeds_do_not_overflow(self):
        profile = speed._SpeedProfile({0: 1e308, 1: 1e308})
        self.assertTrue(0 < profile.total_time < 1e-307)
        self.assertAlmostEqual(profile.map(0.5), 0.5)

    def test_invalid_profiles_fail_before_execution(self):
        for value in ([], {True: 1}, {0: True}, {0: -1}, {-0.1: 1}, {1.1: 1},
                      {float("nan"): 1}, {0: float("inf")}, {0: 0, 1: 0},
                      {0: 1e-320, 1: 1e-320}):
            with self.subTest(value=value), self.assertRaises((ValueError, TypeError)):
                speed._SpeedProfile(value)

    def test_budget(self):
        with self.assertRaisesRegex(ValueError, "budget"):
            speed._SpeedProfile({i / 5000: 1 for i in range(5001)}
            )

    def test_nonfinite_alpha_refused(self):
        for value in (float("inf"), float("nan"), True):
            with self.subTest(value=value), self.assertRaises((ValueError, TypeError)):
                speed._SpeedProfile().map(value)

    def test_authored_outer_curve_evaluated_once_per_sample(self):
        calls = []
        curve = speed._ClockCurve(speed._SpeedProfile({0: 0, 1: 2}),
                                  lambda a: calls.append(a) or a)
        self.assertEqual(calls, [])
        self.assertEqual(curve(0.5), 0.25)
        self.assertEqual(calls, [0.5])
        self.assertEqual(curve.last_value, 0.25)


class LifecycleTests(unittest.TestCase):
    def setUp(self):
        self.g = protocol()
        self.child = self.g.Animation()

    def test_published_identity_and_install_once(self):
        alias = self.g.ChangeSpeed
        method = alias.begin
        speed.install_speed(self.g)
        self.assertIs(alias, self.g.ChangeSpeed)
        self.assertIs(method, alias.begin)
        self.assertTrue(issubclass(alias, self.g.AnimationGroup))
        self.assertIsNone(alias._native_kind)

    def test_builder_prepared_once(self):
        builder = self.g.Builder(self.child)
        animation = self.g.ChangeSpeed(builder, {0: 2, 1: 2})
        self.assertIs(animation.anim, self.child)
        self.assertEqual(builder.calls, 1)
        self.assertEqual(animation.run_time, 1.0)
        self.assertEqual(self.child.run_time, 2.0)

    def test_shared_child_lifecycle_and_wall_clock_helpers(self):
        animation = self.g.ChangeSpeed(self.child, {0: 0, 1: 2})
        animation.begin()
        animation.update_mobjects(0.125)
        animation.interpolate(0.5)
        animation.finish()
        animation.finish()
        animation.clean_up_from_scene("scene")
        animation.clean_up_from_scene("scene")
        self.assertEqual(self.child.events,
                         ["begin", ("alpha", 0), ("dt", 0.125), ("alpha", 0.25),
                          "finish", ("cleanup", "scene")])

    def test_child_easing_applies_once_after_timeline_map(self):
        calls = []
        self.child.rate_func = lambda a: calls.append(a) or a * a
        animation = self.g.ChangeSpeed(self.child, {0: 0, 1: 2})
        self.assertEqual(calls, [])
        animation.begin()
        animation.interpolate(0.5)
        self.assertEqual(calls, [0, 0.25])
        self.assertEqual(self.child.events[-1], ("alpha", 0.0625))

    def test_child_rate_override_is_execution_scoped(self):
        old = self.child.rate_func
        replacement = lambda a: a * a
        animation = self.g.ChangeSpeed(self.child, {0: 1}, rate_func=replacement)
        self.assertIs(self.child.rate_func, old)
        animation.begin()
        self.assertIs(self.child.rate_func, replacement)
        animation.finish()
        self.assertIs(self.child.rate_func, old)
        animation.clean_up_from_scene(None)
        animation.begin()
        animation.abort()
        self.assertIs(self.child.rate_func, old)

    def test_nested_wrapper_restores_composite_rate_without_cycle(self):
        inner = self.g.ChangeSpeed(self.child, {0: 0, 1: 2})
        old = inner._speed_curve.easing
        outer = self.g.ChangeSpeed(inner, {0: 1}, rate_func=lambda a: a * a)
        outer.begin()
        outer.interpolate(0.5)
        self.assertEqual(self.child.events[-1], ("alpha", 0.0625))
        outer.finish()
        self.assertIs(inner._speed_curve.easing, old)
        self.assertEqual(inner.rate_func(0.5), 0.25)

    def test_scene_rate_assignment_preserves_profile(self):
        animation = self.g.ChangeSpeed(self.child, {0: 0, 1: 2})
        curve = animation.rate_func
        animation.rate_func = lambda a: a * a
        self.assertIs(animation.rate_func, curve)
        self.assertEqual(animation.rate_func(0.5), 0.0625)

    def test_failure_aborts_and_restores_child_without_finishing(self):
        old = self.child.rate_func
        animation = self.g.ChangeSpeed(self.child, {0: 1}, rate_func=lambda a: a)
        animation.begin()
        failure = RuntimeError("authored callback")
        self.child.failure = failure
        with self.assertRaises(RuntimeError) as caught:
            animation.interpolate(0.5)
        self.assertIs(caught.exception, failure)
        self.assertIs(self.child.rate_func, old)
        self.assertEqual(self.child.events[-1], "abort")
        self.assertNotIn("finish", self.child.events)
        self.assertIsNone(animation._composition_driver)

    def test_secondary_abort_error_does_not_replace_primary(self):
        animation = self.g.ChangeSpeed(self.child, {})
        animation.begin()
        failure = RuntimeError("primary")
        self.child.failure = failure
        def abort():
            raise ValueError("secondary")
        self.child.abort = abort
        with self.assertRaises(RuntimeError) as caught:
            animation.interpolate(0.5)
        self.assertIs(caught.exception, failure)
        self.assertTrue(any("ValueError" in note for note in failure.__notes__))

    def test_failed_cleanup_is_not_retried(self):
        calls = []
        def cleanup(scene):
            calls.append(scene)
            raise RuntimeError("publication failed")
        self.child.clean_up_from_scene = cleanup
        animation = self.g.ChangeSpeed(self.child, {})
        animation.begin()
        animation.finish()
        with self.assertRaises(RuntimeError):
            animation.clean_up_from_scene("scene")
        animation.clean_up_from_scene("scene")
        self.assertEqual(calls, ["scene"])
        self.assertIsNone(animation._composition_driver)

    def test_invalid_child_duration_and_explicit_duration(self):
        for duration in (-1, float("nan"), float("inf")):
            self.child.run_time = duration
            with self.assertRaises(ValueError):
                self.g.ChangeSpeed(self.child, {})
        self.child.run_time = 2
        self.assertEqual(self.g.ChangeSpeed(self.child, {0: 2}, run_time=3).run_time, 3)
        self.assertEqual(self.g.ChangeSpeed(self.child, {}, run_time=0).run_time, 0)
        for duration in (-1, True, float("inf")):
            with self.assertRaises((ValueError, TypeError)):
                self.g.ChangeSpeed(self.child, {}, run_time=duration)

    def test_invalid_child_and_rate(self):
        with self.assertRaisesRegex(NotImplementedError, "clock-remap seam"):
            self.g.ChangeSpeed(None, {})
        with self.assertRaises(TypeError):
            self.g.ChangeSpeed(object(), {})
        with self.assertRaises(ValueError):
            self.g.ChangeSpeed(self.child, {}, rate_func="missing")
        with self.assertRaises(TypeError):
            self.g.ChangeSpeed(self.child, {}, rate_func=2)


@unittest.skipUnless(os.environ.get("FMN_TEST_NATIVE") == "1", "requires installed native portal; set FMN_TEST_NATIVE=1")
class NativeSpeedTests(unittest.TestCase):
    def test_native_transform_and_clock(self):
        import manimlib as m
        from manimlib.animation.speed import ChangeSpeed
        scene, dot = m.Scene(), m.Dot()
        scene.add(dot)
        start = scene.time
        scene.play(ChangeSpeed(dot.animate(run_time=2, rate_func=m.linear).shift(m.RIGHT * 2),
                               {0: 2, 1: 2}))
        self.assertAlmostEqual(scene.time - start, 1.0, places=6)
        self.assertAlmostEqual(float(dot.get_center()[0]), 2.0, places=5)

    def test_native_nested_group_and_remover(self):
        import manimlib as m
        from manimlib.animation.speed import ChangeSpeed
        scene, left, right = m.Scene(), m.Dot(), m.Dot()
        scene.add(left, right)
        group = m.AnimationGroup(left.animate.shift(m.RIGHT), m.FadeOut(right), run_time=1)
        scene.play(ChangeSpeed(group, {0: 1, 1: 2}))
        self.assertAlmostEqual(float(left.get_center()[0]), 1.0, places=5)
        self.assertNotIn(right, scene.mobjects)


if __name__ == "__main__":
    unittest.main()
