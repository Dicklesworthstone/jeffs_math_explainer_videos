"""Wave/grow/inside-out delegation tests; native acceptance is separate."""
from __future__ import annotations

import math
import unittest

import numpy as np

from test_indication_protocol import native_table
from fmn_python.indication import install_indication
from fmn_python.movement import install_movement


def wave_table():
    n = native_table()
    n._UP = np.array([0., 1., 0.])
    A, M = n.Animation, n.Mobject

    def bounds(self):
        parts = [member.points for member in self.get_family() if len(member.points)]
        return np.concatenate(parts) if parts else np.zeros((1, 3))

    def apply_function(self, function, **kwargs):
        self.last_function_config = kwargs
        # Stage writes are represented by this storage double; the callback
        # and its lifecycle are the production implementations under test.
        for member in self.get_family():
            member.points = np.array([function(point) for point in member.points]).reshape(-1, 3)
        return self

    def move_to(self, point):
        delta = np.asarray(point) - self.get_center()
        for member in self.get_family():
            member.points += delta
        return self

    def set_color(self, color):
        for member in self.get_family():
            member.color = color
        return self

    def reverse_points(self):
        for member in self.get_family():
            member.points = member.points[::-1].copy()
        return self

    M.get_left = lambda self: bounds(self).min(axis=0)
    M.get_right = lambda self: bounds(self).max(axis=0)
    M.apply_function, M.move_to = apply_function, move_to
    M.set_color, M.reverse_points = set_color, reverse_points

    class Homotopy(A):
        apply_function_config = {}

        def __init__(self, homotopy, mobject, run_time=3, **kwargs):
            self.homotopy = homotopy
            super().__init__(mobject, run_time=run_time, **kwargs)

        def function_at_time_t(self, t):
            return lambda point: self.homotopy(*point, t)

        def interpolate_submobject(self, current, start, alpha):
            current.match_points(start)
            current.apply_function(self.function_at_time_t(alpha), **self.apply_function_config)

    class ApplyWave(A):
        _native_kind = "apply_wave"

    class Transform(A):
        _native_kind = "transform"
        _target_attr = "target_mobject"

        def __init__(self, mobject, target_mobject=None, path_arc=0,
                     path_arc_axis=(0, 0, 1), path_func=None, **kwargs):
            super().__init__(mobject, **kwargs)
            self.target_mobject, self.path_arc = target_mobject, path_arc
            self.path_arc_axis, self.path_func = path_arc_axis, path_func

        def create_target(self):
            return self.target_mobject

        def create_starting_mobject(self):
            return self.mobject.copy()

        def _native_target(self):
            self.target_mobject = self.create_target()
            return self.target_mobject

        def _native_params(self):
            return {"path_arc": self.path_arc, "path_arc_axis": self.path_arc_axis}

        def begin(self):
            self.target_mobject = self.create_target()
            self.starting_mobject = self.create_starting_mobject()
            self.families = list(zip(self.mobject.get_family(), self.starting_mobject.get_family(),
                                     self.target_mobject.get_family()))
            self.interpolate(0)

        def interpolate_submobject(self, current, start, target, alpha):
            path = self.path_func or (lambda a, b, t: (1 - t) * a + t * b)
            current.points = path(start.points, target.points, alpha)
            current.color = target.color if alpha >= 1 else start.color

    class GrowFromPoint(Transform):
        _native_kind = "grow_from_point"

        def __init__(self, mobject, point, point_color=None, **kwargs):
            self.point, self.point_color = np.array(point), point_color
            super().__init__(mobject, **kwargs)

        def create_target(self):
            return self.mobject.copy()

        def create_starting_mobject(self):
            start = super().create_starting_mobject().scale(0).move_to(self.point)
            if self.point_color is not None:
                start.set_color(self.point_color)
            return start

        def _native_target(self):
            return None

        def _native_params(self):
            return {"point": self.point}

    class GrowFromCenter(GrowFromPoint):
        _native_kind = "grow_from_center"

        def __init__(self, mobject, **kwargs):
            super().__init__(mobject, mobject.get_center(), **kwargs)

    class GrowArrow(GrowFromPoint):
        _native_kind = "grow_arrow"

    class SpinInFromNothing(GrowFromCenter):
        _native_kind = "spin_in_from_nothing"

        def __init__(self, mobject, **kwargs):
            kwargs.setdefault("path_arc", math.pi)
            super().__init__(mobject, **kwargs)

    class Indicate(Transform):
        _native_kind = "indicate"

        def __init__(self, mobject, scale_factor=1.2, color="yellow", **kwargs):
            self.scale_factor, self.color = scale_factor, color
            kwargs.setdefault("rate_func", n.there_and_back)
            super().__init__(mobject, **kwargs)

        def create_target(self):
            return self.mobject.copy().scale(self.scale_factor).set_color(self.color)

        def _native_target(self):
            return None

    class TurnInsideOut(Transform):
        _native_kind = "turn_inside_out"

        def create_target(self):
            return self.mobject.copy().reverse_points()

        def _native_target(self):
            return None

    for cls in (Homotopy, ApplyWave, Transform, GrowFromPoint, GrowFromCenter,
                GrowArrow, SpinInFromNothing, Indicate, TurnInsideOut):
        setattr(n, cls.__name__, cls)
    for cls in (GrowFromPoint, GrowFromCenter, GrowArrow, SpinInFromNothing):
        cls._native_target = Transform._native_target
        cls._native_params = Transform._native_params
    prev_req = n._requires_python_animation
    n._requires_python_animation = lambda a: (type(a) is not getattr(n, type(a).__name__, None)) or prev_req(a)
    install_movement(n)
    return n


class WaveGrowthProtocol(unittest.TestCase):
    def setUp(self):
        self.n = wave_table()
        install_indication(self.n)

    def line(self):
        return self.n.VMobject([[-1, 0, 0], [0, 0, 0], [1, 0, 0]])

    def test_wave_preserves_class_identity_and_restores_homotopy_lineage(self):
        n = wave_table()
        before = n.ApplyWave
        class Authored(before):
            pass
        install_indication(n)
        self.assertIs(n.ApplyWave, before)
        self.assertTrue(issubclass(Authored, n.Homotopy))
        self.assertTrue(n._requires_python_animation(n.ApplyWave(n.VMobject([[-1, 0, 0], [1, 0, 0]]))))

    def test_wave_matches_analytic_spatial_phase(self):
        line = self.line()
        animation = self.n.ApplyWave(line, amplitude=.7, rate_func=self.n.linear)
        animation.begin()
        animation.interpolate(.5)
        expected = [.7 * self.n.there_and_back(.5 ** math.exp(x)) for x in (-1, 0, 1)]
        np.testing.assert_allclose(line.points[:, 1], expected, atol=1e-12)
        np.testing.assert_array_equal(line.points[:, 0], [-1, 0, 1])
        animation.finish()
        np.testing.assert_array_equal(line.points, [[-1, 0, 0], [0, 0, 0], [1, 0, 0]])

    def test_wave_bounds_and_displacement_are_captured_at_construction(self):
        line, direction = self.line(), np.array([0., 1., 0.])
        animation = self.n.ApplyWave(line, direction=direction, amplitude=.7)
        direction[:] = [1, 0, 0]
        line.scale(2)
        animation.begin()
        animation.interpolate(.5)
        expected = [.7 * self.n.there_and_back(.5 ** math.exp(x)) for x in (-2, 0, 2)]
        np.testing.assert_allclose(line.points[:, 1], expected, atol=1e-12)
        np.testing.assert_array_equal(line.points[:, 0], [-2, 0, 2])

    def test_wave_runtime_and_nondefault_direction_are_forwarded(self):
        animation = self.n.ApplyWave(self.line(), direction=[0, 0, 2], amplitude=.3, run_time=2.5)
        self.assertEqual(animation.run_time, 2.5)
        np.testing.assert_allclose(animation.homotopy(0, 0, 1, .5), [0, 0, 1.6])

    def test_wave_public_map_and_function_factory_overrides_remain_live(self):
        calls = []
        class Authored(self.n.ApplyWave):
            def function_at_time_t(self, t):
                calls.append(t)
                return lambda point: point + [0, 0, t]
        line = self.line()
        animation = Authored(line)
        animation.begin()
        animation.interpolate(.5)
        np.testing.assert_array_equal(line.points[:, 2], [.5, .5, .5])
        self.assertEqual(calls, [0, .5])
        animation.homotopy = lambda x, y, z, t: np.array([x, y + t, z])
        np.testing.assert_array_equal(animation.homotopy(1, 2, 3, .5), [1, 2.5, 3])

    def test_wave_repeated_alpha_is_not_cumulative(self):
        animation = self.n.ApplyWave(self.line())
        animation.begin()
        animation.interpolate(.4)
        first = animation.mobject.points.copy()
        animation.interpolate(.4)
        np.testing.assert_array_equal(animation.mobject.points, first)

    def test_wave_invalid_geometry_parameters_fail_before_mutation(self):
        for points in ([], [[0, 0, 0], [0, 1, 0]]):
            with self.assertRaisesRegex(ValueError, "horizontal extent"):
                self.n.ApplyWave(self.n.VMobject(points))
        for options in ({"amplitude": math.inf}, {"direction": [0, 1]},
                        {"direction": [0, math.nan, 0]}):
            line = self.line()
            with self.subTest(options=options), self.assertRaises(ValueError):
                self.n.ApplyWave(line, **options)
            self.assertFalse(line.animating)

    def test_wave_scene_failure_releases_owned_suspension(self):
        animation = self.n.ApplyWave(self.line(), suspend_mobject_updating=True)
        scene = self.n.Scene()
        scene.failure = RuntimeError("wave sibling")
        with self.assertRaisesRegex(RuntimeError, "wave sibling"):
            scene.play(animation)
        self.assertFalse(animation.mobject.suspended or animation.mobject.animating)

    def test_growth_target_is_sampled_at_begin_after_predecessor_state(self):
        line = self.line()
        animation = self.n.GrowFromCenter(line, rate_func=self.n.linear)
        line.scale(2).move_to([3, 0, 0])
        expected = line.points.copy()
        # Stock growth animations stay native per commit 17355d37
        self.assertFalse(self.n._requires_python_animation(animation))
        self.assertIsNone(animation.target_mobject)
        animation.begin()
        np.testing.assert_array_equal(line.points, np.zeros((3, 3)))
        animation.interpolate(.5)
        np.testing.assert_allclose(line.points, expected / 2)
        animation.interpolate(1)
        np.testing.assert_array_equal(line.points, expected)

    def test_growth_preserves_point_color_and_explicit_path(self):
        calls = []
        def path(a, b, t):
            calls.append(t)
            return (1 - t) * a + t * b + [0, t * (1 - t), 0]
        animation = self.n.GrowFromPoint(self.line(), [-2, 0, 0], point_color="red", path_func=path)
        animation.begin()
        self.assertEqual(animation.starting_mobject.color, "red")
        animation.interpolate(.5)
        self.assertEqual(calls, [0, .5])
        np.testing.assert_allclose(animation.mobject.points[:, 1], [.25] * 3)

    def test_all_growth_aliases_use_the_same_deferred_transform_seam(self):
        for name in ("GrowFromPoint", "GrowFromCenter", "GrowArrow", "SpinInFromNothing"):
            cls = getattr(self.n, name)
            self.assertTrue(cls._native_kind.startswith("grow_") or cls._native_kind == "spin_in_from_nothing")
            self.assertIs(cls._native_target, self.n.Transform._native_target)
            self.assertIs(cls._native_params, self.n.Transform._native_params)
        self.assertEqual(self.n.SpinInFromNothing(self.line()).path_arc, math.pi)

    def test_indicate_target_recolors_and_swells_from_live_geometry(self):
        line = self.line()
        animation = self.n.Indicate(line, scale_factor=2, color="yellow")
        line.scale(3)
        # Stock Indicate stays native per commit 17355d37
        self.assertFalse(self.n._requires_python_animation(animation))
        animation.begin()
        animation.interpolate(.5)
        np.testing.assert_array_equal(line.points[:, 0], [-6, 0, 6])
        self.assertEqual(line.color, "yellow")
        animation.interpolate(1)
        np.testing.assert_array_equal(line.points[:, 0], [-3, 0, 3])

    def test_inside_out_uses_begin_time_reversal_and_custom_hooks(self):
        calls = []
        class Authored(self.n.TurnInsideOut):
            def create_target(self):
                calls.append("target")
                return super().create_target()
        line = self.line()
        animation = Authored(line)
        line.points[:, 1] = [1, 2, 3]
        expected = line.points[::-1].copy()
        self.assertTrue(self.n._requires_python_animation(animation))
        animation.begin()
        animation.interpolate(1)
        self.assertEqual(calls, ["target"])
        np.testing.assert_array_equal(line.points, expected)

    def test_unrelated_transform_dispatch_is_not_changed(self):
        animation = self.n.Transform(self.line(), self.line())
        self.assertFalse(self.n._requires_python_animation(animation))

    def test_growth_cooperates_with_parent_starting_object_override(self):
        calls = []
        def starting(animation):
            calls.append(animation)
            return animation.mobject.copy()
        self.n.Transform.create_starting_mobject = starting
        animation = self.n.GrowFromCenter(self.line())
        animation.begin()
        self.assertEqual(calls, [animation])

    def test_stock_explicit_wiggle_pivots_use_ordered_public_getters(self):
        animation = self.n.WiggleOutThenIn(self.line(), scale_about_point=[1, 0, 0])
        self.assertTrue(self.n._requires_python_animation(animation))


if __name__ == "__main__":
    unittest.main()
