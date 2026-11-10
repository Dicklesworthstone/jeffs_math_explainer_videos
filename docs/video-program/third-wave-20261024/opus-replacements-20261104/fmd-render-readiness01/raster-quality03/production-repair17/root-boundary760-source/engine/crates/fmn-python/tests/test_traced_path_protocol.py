"""Temporal sampling tests; native smooth-path/storage acceptance is separate."""
from __future__ import annotations

import copy
import importlib.util
from pathlib import Path
from types import SimpleNamespace
import unittest

import numpy as np

SOURCE = Path(__file__).resolve().parents[1] / "python/fmn_python/traced_path.py"
spec = importlib.util.spec_from_file_location("traced_path_under_test", SOURCE)
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)


def environment(native_tail_initializer=None):
    class Mobject:
        def __init__(self, **kwargs):
            self.updaters, self.kwargs = [], kwargs
            self.points = np.empty((0, 3))
            self.position = np.zeros(3)
        def get_center(self):
            return self.position
        def add_updater(self, function):
            self.updaters.append(function)
            self.update(0)
            return self
        def update(self, dt=0):
            for function in tuple(self.updaters):
                function(self, dt)
            return self
    class VMobject(Mobject):
        def set_points_smoothly(self, points):
            self.points = np.array(points)
            if getattr(self, "fail_geometry", False):
                raise RuntimeError("native geometry failure")
            return self
        def set_stroke(self, **kwargs):
            self.stroke = kwargs
            return self
    class TracedPath(VMobject):
        pass
    class TracingTail(VMobject):
        pass
    if native_tail_initializer is not None:
        TracingTail.__init__ = native_tail_initializer
        TracingTail._init_native_tracer = lambda self: None

    native = SimpleNamespace(Mobject=Mobject, VMobject=VMobject, TracedPath=TracedPath,
                             TracingTail=TracingTail, _np=np, _WHITE="white")
    module.install_traced_path(native)
    return native


class TemporalTracingTests(unittest.TestCase):
    def setUp(self):
        self.native = environment()
    def trace(self, **kwargs):
        position = np.zeros(3)
        calls = []
        def source():
            calls.append(position.copy())
            return position
        trace = self.native.TracedPath(source, **kwargs)
        return position, calls, trace
    def drive(self, trace, position, steps):
        time = trace.time
        for dt in steps:
            time += dt
            position[:] = (time, 2 * time, 0)
            trace.update(dt)
    def test_no_source_read_at_construction_or_zero_dt(self):
        _, calls, trace = self.trace()
        trace.update(0)
        self.assertEqual(calls, [])
        self.assertEqual(trace.time, 0)
    def test_spacing_controls_anchor_count_and_keeps_live_endpoint(self):
        p, calls, trace = self.trace(time_per_anchor=.2)
        self.drive(trace, p, [.1] * 10)
        np.testing.assert_allclose(trace.points[:, 0], [.1, .3, .5, .7, .9, 1])
        np.testing.assert_allclose(trace.points[:, 1], 2 * trace.points[:, 0])
        self.assertEqual(len(calls), 10)
    def test_variable_dt_keeps_the_same_time_window_for_linear_motion(self):
        p, _, a = self.trace(time_traced=.45, time_per_anchor=.2)
        q, _, b = self.trace(time_traced=.45, time_per_anchor=.2)
        self.drive(a, p, [.1] * 10)
        self.drive(b, q, [.1, .3, .1, .5])
        np.testing.assert_allclose(a.points, b.points)
        np.testing.assert_allclose(a.points[:, 0], [.55, .7, .9, 1])
    def test_short_window_keeps_an_interpolated_left_edge_not_old_history(self):
        p, _, trace = self.trace(time_traced=.01, time_per_anchor=1)
        self.drive(trace, p, [.1, .02])
        np.testing.assert_allclose(trace.points[:, 0], [.11, .12])
        self.assertLessEqual(len(trace._trace_anchors), 2)
    def test_large_dt_discards_expired_grid_work_before_allocating(self):
        p, calls, trace = self.trace(time_traced=.4, time_per_anchor=.1)
        self.drive(trace, p, [.1, 1_000_000])
        self.assertLessEqual(len(trace._trace_anchors), 8)
        self.assertLessEqual(len(trace.traced_points), 7)
        self.assertEqual(len(calls), 2)
        self.assertAlmostEqual(trace.points[0, 0], trace.time - .4)
    def test_finite_trail_storage_stays_bounded_over_many_ticks(self):
        p, _, trace = self.trace(time_traced=.4, time_per_anchor=.1)
        self.drive(trace, p, [.01] * 2000)
        self.assertLessEqual(len(trace._trace_anchors), 7)
        self.assertLessEqual(len(trace.points), 7)
    def test_zero_window_keeps_only_current_point(self):
        p, _, trace = self.trace(time_traced=0)
        self.drive(trace, p, [.1, .3, .2])
        self.assertEqual(len(trace.points), 1)
        self.assertEqual(len(trace._trace_anchors), 1)
        np.testing.assert_allclose(trace.points[0], p)
    def test_mutable_source_array_and_public_mirror_cannot_change_history(self):
        p, _, trace = self.trace(time_per_anchor=.1)
        self.drive(trace, p, [.1, .1])
        old = trace._trace_anchors
        p[:] = 100
        trace.traced_points[0][:] = -100
        self.assertEqual(trace._trace_anchors, old)
        self.drive(trace, p, [.1])
        np.testing.assert_allclose(trace.points[0], [.1, .2, 0])
    def test_copy_has_independent_evolution_with_shared_callback_identity(self):
        p, _, trace = self.trace(time_per_anchor=.1)
        self.drive(trace, p, [.1, .1])
        cloned = copy.copy(trace)
        old = trace._trace_anchors
        cloned.update(.1)
        self.assertEqual(trace._trace_anchors, old)
        self.assertNotEqual(cloned.time, trace.time)
        self.assertIsNot(cloned.traced_points, trace.traced_points)
    def test_runtime_spacing_and_duration_edits_are_applied(self):
        p, _, trace = self.trace(time_per_anchor=.1)
        self.drive(trace, p, [.1, .1, .1])
        trace.time_per_anchor, trace.time_traced = .2, .3
        self.drive(trace, p, [.4])
        np.testing.assert_allclose(trace.points[:, 0], [.4, .5, .7])
    def test_bad_dt_does_not_invoke_source_or_mutate_state(self):
        p, calls, trace = self.trace()
        for dt in (-.1, float("inf"), float("nan")):
            with self.assertRaises(ValueError):
                trace.update_path(dt)
        self.assertEqual(calls, [])
        self.assertEqual(trace.time, 0)
    def test_external_time_rewind_is_rejected_before_source(self):
        p, calls, trace = self.trace()
        self.drive(trace, p, [.1])
        trace.time = 0
        with self.assertRaisesRegex(ValueError, "outside its updater"):
            trace.update(.1)
        self.assertEqual(len(calls), 1)
    def test_bad_point_leaves_published_temporal_state_unchanged(self):
        p, _, trace = self.trace()
        self.drive(trace, p, [.1])
        old = trace._trace_anchors
        for bad in ((1, 2), (1, 2, float("nan")), (1, 2, float("inf"))):
            trace.traced_point_func = lambda bad=bad: bad
            with self.assertRaises(ValueError):
                trace.update(.1)
        self.assertEqual(trace._trace_anchors, old)
        self.assertEqual(trace.time, .1)
    def test_native_failure_does_not_publish_new_time_or_history(self):
        p, _, trace = self.trace()
        self.drive(trace, p, [.1])
        old = trace._trace_anchors
        trace.fail_geometry = True
        with self.assertRaisesRegex(RuntimeError, "native geometry"):
            trace.update(.1)
        self.assertEqual(trace._trace_anchors, old)
        self.assertEqual(trace.time, .1)
    def test_invalid_configuration_is_rejected_before_source(self):
        for kwargs in ({"time_traced": -1}, {"time_traced": float("nan")},
                       {"time_per_anchor": 0}, {"time_per_anchor": float("inf")},
                       {"time_per_anchor": -1}, {"time_traced": 1, "time_per_anchor": 1e-12}):
            with self.assertRaises(ValueError):
                self.trace(**kwargs)
    def test_unbounded_grid_budget_fails_without_silent_downsampling(self):
        p, _, trace = self.trace(time_per_anchor=.001)
        self.drive(trace, p, [.1])
        with self.assertRaisesRegex(ValueError, "budget"):
            trace.update(1000)
        self.assertEqual(trace.time, .1)
    def test_tail_accepts_mobject_and_retains_fading_style_profiles(self):
        mob = self.native.Mobject()
        tail = self.native.TracingTail(mob, time_traced=.4, time_per_anchor=.1)
        self.assertIsInstance(tail, self.native.TracedPath)
        self.assertEqual(tail.stroke_config["width"], (0, 3))
        self.assertEqual(tail.stroke_config["opacity"], (0, 1))
        mob.position[:] = (1, 0, 0)
        tail.update(.1)
        self.assertEqual(tail.stroke["width"], (0, 3))
        self.assertEqual(tail.stroke["opacity"], (0, 1))
        np.testing.assert_allclose(tail.points[-1], mob.position)
        np.testing.assert_allclose(tail.points[0], [0, 0, 0])
    def test_tail_seeds_history_without_initial_geometry_or_style(self):
        position = np.zeros(3)
        calls = []
        def source():
            calls.append("source")
            return position

        class Authored(self.native.TracingTail):
            def set_points_smoothly(self, points):
                calls.append("geometry")
                return super().set_points_smoothly(points)
            def set_stroke(self, **kwargs):
                calls.append("style")
                return super().set_stroke(**kwargs)
        tail = Authored(source, time_traced=.45, time_per_anchor=.1)
        self.assertEqual(calls, ["source"])
        self.assertEqual(len(tail.points), 0)
        self.assertFalse(hasattr(tail, "stroke"))
        self.assertEqual(len(tail.traced_points), 4)
        self.assertTrue(all(
            not np.shares_memory(a, b)
            for a, b in zip(tail.traced_points, tail.traced_points[1:])
        ))
        tail.update(0)
        self.assertEqual(calls, ["source"])
        self.assertEqual(len(tail.points), 0)
        position[:] = (1, 0, 0)
        tail.update(.1)
        self.assertEqual(calls, ["source", "source", "geometry", "style"])
        np.testing.assert_allclose(tail.points[0], [0, 0, 0])
        np.testing.assert_allclose(tail.points[-1], position)
    def test_tail_callable_and_zero_length_work(self):
        tail = self.native.TracingTail(lambda: [1, 2, 3], time_traced=0)
        self.assertEqual(len(tail.points), 0)
        self.assertEqual(tail.traced_points, [])
        tail.update(.1)
        np.testing.assert_allclose(tail.points, [[1, 2, 3]])
    def test_tail_rejects_infinite_history(self):
        with self.assertRaises(ValueError):
            self.native.TracingTail(lambda: [0, 0, 0], time_traced=np.inf)
    def test_native_tail_route_cannot_bypass_initial_history_validation(self):
        calls = []
        def initialize_native(tail, source, **kwargs):
            calls.append(kwargs)
        native = environment(native_tail_initializer=initialize_native)
        mob = native.Mobject()
        mob._is_bound = lambda: True
        for kwargs in ({"time_traced": -1}, {"time_traced": float("inf")},
                       {"time_traced": float("nan")}, {"time_traced": 1, "time_per_anchor": 1e-12}):
            with self.assertRaises(ValueError):
                native.TracingTail(mob, **kwargs)
        self.assertEqual(calls, [])
        native.TracingTail(mob, time_traced=.5, time_per_anchor=.125)
        self.assertEqual(len(calls), 1, "valid bound source lost the native route")
        self.assertEqual(calls[0]["time_traced"], .5)
        self.assertEqual(calls[0]["time_per_anchor"], .125)
    def test_trace_geometry_and_style_overrides_remain_virtual(self):
        parent = self.native.TracedPath
        calls = []
        class Authored(parent):
            def set_points_smoothly(self, points):
                calls.append("geometry")
                return super().set_points_smoothly(points)
            def set_stroke(self, **kwargs):
                calls.append("style")
                return super().set_stroke(**kwargs)
        trace = Authored(lambda: [0, 0, 0])
        trace.update(.1)
        self.assertEqual(calls, ["geometry", "style"])
    def test_installer_is_idempotent_and_keeps_qualified_class_identity(self):
        cls = self.native.TracedPath
        constructor = cls.__init__
        module.install_traced_path(self.native)
        self.assertIs(self.native.TracedPath, cls)
        self.assertIs(constructor, cls.__init__)
        self.assertEqual(cls.update_path.__qualname__, cls.__qualname__ + ".update_path")


if __name__ == "__main__":
    unittest.main()
