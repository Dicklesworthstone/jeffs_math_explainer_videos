"""Graph authoring/query protocols; see test_graphing_protocol for double limits."""
from __future__ import annotations

import copy
import itertools
import math
import unittest
from unittest.mock import patch

import numpy as np

from test_graphing_protocol import fixture, graphing


class GraphConstructionTests(unittest.TestCase):
    def setUp(self):
        self.native = fixture()
        graphing.install_graphing(self.native)
        self.axes = self.native.CoordinateSystem()

    def test_scalar_math_function_remains_scalar_on_every_updater_tick(self):
        called = []
        amplitude = [1.]
        def wave(x):
            self.assertIs(type(x), float)
            called.append(x)
            return amplitude[0] * math.sin(x)
        curve = self.axes.get_graph(wave, bind=True)
        called.clear()
        for factor in (1., 2., .5):
            amplitude[0] = factor
            curve.update()
            anchors = curve.points[::2]
            np.testing.assert_allclose(anchors[:, 1], factor * np.sin(anchors[:, 0]))
        self.assertGreater(len(called), 5)
        np.testing.assert_allclose(self.axes.i2gp(.7, curve), [.7, .5 * math.sin(.7), 0])

    def test_scalar_branch_callback_is_not_vectorized_by_guessing(self):
        threshold = [0.]
        def step(x):
            return -1. if x < threshold[0] else 1.
        graph = self.axes.get_graph(step, bind=True)
        graph.update()
        threshold[0] = 1.
        graph.update()
        self.assertEqual(self.axes.i2gp(.5, graph)[1], -1.)
        self.assertEqual(self.axes.i2gp(1.5, graph)[1], 1.)

    def test_original_native_constructor_keeps_the_callable_and_options(self):
        function = math.sin
        graph = self.axes.get_graph(function, x_range=np.array([-1., 2., .25]), bind=True,
                                    color='gold', stroke_width=6)
        called, domain, bind, options = self.axes.evaluations[-1]
        self.assertIs(called, function)
        self.assertEqual(domain, (-1., 2., .25))
        self.assertFalse(bind)
        self.assertEqual(options, {'color': 'gold', 'stroke_width': 6})
        self.assertIsInstance(graph, self.native.VMobject)
        self.assertIs(graph.underlying_function, function)

    def test_custom_bind_hook_sees_original_function_once(self):
        native = self.native
        events = []
        class Custom(native.CoordinateSystem):
            def bind_graph_to_func(self, graph, function, **kwargs):
                events.append((graph, function, kwargs))
                return super().bind_graph_to_func(graph, function, **kwargs)
        axes = Custom()
        graph = axes.get_graph(math.cos, bind=True)
        self.assertEqual(events, [(graph, math.cos, {})])
        graph.update()
        self.assertEqual(len(events), 1)

    def test_bind_context_is_reset_after_authored_hook_failure(self):
        error = LookupError('bind failed')
        class Custom(self.native.CoordinateSystem):
            def bind_graph_to_func(self, *args, **kwargs):
                raise error
        with self.assertRaises(LookupError) as caught:
            Custom().get_graph(math.sin, bind=True)
        self.assertIs(caught.exception, error)
        self.assertIsNone(graphing._SCALAR_BIND.get())
        graph = self.axes.get_graph(math.sin)
        arrays = []
        self.axes.bind_graph_to_func(graph, lambda xs: arrays.append(xs) or np.cos(xs))
        graph.update()
        self.assertIsInstance(arrays[0], np.ndarray)

    def test_nested_binding_does_not_inherit_another_graphs_scalar_mode(self):
        arrays = []
        other = self.axes.get_graph(lambda x: x)
        native, outer_axes = self.native, self.axes
        class Custom(native.CoordinateSystem):
            def bind_graph_to_func(self, graph, function, **kwargs):
                outer_axes.bind_graph_to_func(other, lambda xs: arrays.append(xs) or xs)
                return super().bind_graph_to_func(graph, function, **kwargs)
        graph = Custom().get_graph(math.sin, bind=True)
        graph.update()
        other.update()
        self.assertIsInstance(arrays[0], np.ndarray)
        self.assertIsNone(graphing._SCALAR_BIND.get())

    def test_direct_vectorized_binding_keeps_its_one_call_contract(self):
        graph = self.axes.get_graph(lambda x: x)
        calls = []
        self.axes.bind_graph_to_func(graph, lambda xs: calls.append(xs) or xs * 2)
        self.assertEqual(len(calls), 1)  # the bind's own evaluation (fm-skpp)
        graph.update()
        self.assertEqual(len(calls), 2)
        self.assertTrue(all(call.ndim == 1 for call in calls))

    def test_scalar_failure_is_not_retried_in_another_call_mode(self):
        failure = RuntimeError('scalar branch failed')
        enabled, calls = [False], []
        def function(x):
            calls.append(x)
            if enabled[0]:
                raise failure
            return x
        graph = self.axes.get_graph(function, bind=True)
        old = graph.points.copy()
        calls.clear()
        enabled[0] = True
        with self.assertRaises(RuntimeError) as raised:
            graph.update()
        self.assertIs(raised.exception, failure)
        self.assertEqual(len(calls), 1)
        self.assertIs(type(calls[0]), float)
        np.testing.assert_array_equal(graph.points, old)

    def test_unbound_scalar_function_metadata_keeps_original_identity(self):
        graph = self.axes.get_graph(math.sin)
        self.assertIs(graph.underlying_function, math.sin)
        self.assertEqual(graph.updaters, [])
        np.testing.assert_allclose(self.axes.i2gp(.3, graph), [.3, math.sin(.3), 0])

    def test_copy_uses_its_own_graph_but_same_scalar_function_dependency(self):
        gain = [1.]
        graph = self.axes.get_graph(lambda x: gain[0] * math.sin(x), bind=True)
        copied = copy.deepcopy(graph)
        old = graph.points.copy()
        gain[0] = 3.
        copied.update()
        np.testing.assert_array_equal(graph.points, old)
        self.assertFalse(np.array_equal(copied.points, old))
        self.assertIs(copied.underlying_function, graph.underlying_function)

    def test_ranges_accept_numpy_two_values_and_bounded_iterators(self):
        for value in (np.array([-1., 1.]), [-1., 1.], (-1., 1., .5), iter([-1., 1.])):
            graph = self.axes.get_graph(math.sin, x_range=value)
            self.assertEqual(graph.points[0, 0], -1.)
            self.assertEqual(graph.points[-1, 0], 1.)

    def test_bad_sampling_requests_refuse_before_any_function_calls(self):
        ranges = ([], [1], [0, 1, 2, 3], [0, 0], [1, -1], [0, 1, 0],
                  [0, 1, -1], [0, np.nan], [0, np.inf], [0, 1, 1e-100],
                  itertools.repeat(1))
        for values in ranges:
            with self.subTest(values=values), self.assertRaises(ValueError):
                self.axes.get_graph(lambda x: self.fail('must not execute'), x_range=values)
        for density in (0, -1, np.nan, np.inf, 1e300):
            self.axes.num_sampled_graph_points_per_tick = density
            with self.subTest(density=density), self.assertRaises(ValueError):
                self.axes.get_graph(lambda x: self.fail('must not execute'))
        self.assertEqual(self.axes.evaluations, [])

    def test_sample_budget_exact_boundary_is_admitted(self):
        with patch.object(graphing, '_MAX_SAMPLES', 5):
            self.axes.num_sampled_graph_points_per_tick = 1
            self.axes.get_graph(lambda x: x, x_range=[0, 4, 1])
            with self.assertRaisesRegex(ValueError, 'budget'):
                self.axes.get_graph(lambda x: self.fail('must not execute'), x_range=[0, 5, 1])

    def test_discontinuity_iterator_is_frozen_for_both_construction_and_binding(self):
        graph = self.axes.get_graph(math.sin, bind=True,
                                    discontinuities=iter([0., 0., 50.]), epsilon=.01)
        self.assertEqual(graph.discontinuities, (0.,))
        graph.update()
        self.assertEqual(len(graph.get_subpaths()), 2)
        self.assertAlmostEqual(graph.get_subpaths()[0][-1, 0], -.01)
        self.assertAlmostEqual(graph.get_subpaths()[1][0, 0], .01)

    def test_invalid_discontinuities_epsilon_function_and_bind_refuse_early(self):
        for options in ({'discontinuities': itertools.repeat(0.)}, {'discontinuities': [np.inf]},
                        {'epsilon': 0}, {'epsilon': -1}, {'epsilon': np.nan}, {'bind': 1}):
            with self.subTest(options=options), self.assertRaises((TypeError, ValueError)):
                self.axes.get_graph(lambda x: self.fail('must not execute'), **options)
        with self.assertRaises(TypeError):
            self.axes.get_graph(42)
        self.assertEqual(self.axes.evaluations, [])


class GeometricGraphQueryTests(unittest.TestCase):
    def setUp(self):
        self.native = fixture()
        graphing.install_graphing(self.native)
        self.axes = self.native.CoordinateSystem()
        self.axes.x_range = (-8., 8., 1.)

    def line(self, start=(-5., -3.), stop=(5., 7.)):
        return self.native.VMobject().set_points_as_corners(
            self.axes.c2p(np.array([start[0], stop[0]]), np.array([start[1], stop[1]])))

    def test_axis_data_domain_is_never_used_as_bezier_parameter(self):
        graph = self.line()
        for x in (-5., -4.25, -1., 0., 1., 4.75, 5.):
            np.testing.assert_allclose(self.axes.i2gp(x, graph), [x, x + 2, 0], atol=1e-6)
        self.assertIsNone(self.axes.i2gp(-6., graph))
        self.assertIsNone(self.axes.i2gp(6., graph))

    def test_data_domain_wholly_outside_zero_one_is_still_queryable(self):
        self.axes.x_range = (10., 20., 1.)
        graph = self.line((10., 0.), (20., 10.))
        np.testing.assert_allclose(self.axes.i2gp(12., graph), [12., 2., 0.])

    def test_decreasing_x_curve_and_exact_endpoints(self):
        graph = self.line((5., 7.), (-5., -3.))
        for x in (-5., -3.2, 0., 4.1, 5.):
            np.testing.assert_allclose(self.axes.i2gp(x, graph), [x, x + 2, 0], atol=1e-6)

    def test_native_quadratic_evaluator_handles_nonlinear_x_parameterization(self):
        # x(t)=2t+2t^2, y(t)=4t^2. No line/point interpolation is substituted.
        graph = self.native.VMobject([[0, 0, 0], [1, 0, 0], [4, 4, 0]])
        for x in np.linspace(0., 4., 31):
            t = (-1 + math.sqrt(1 + 2 * x)) / 2
            np.testing.assert_allclose(self.axes.i2gp(x, graph), [x, 4 * t * t, 0], atol=1e-6)

    def test_breaks_are_not_false_chords_through_missing_intervals(self):
        graph = self.line((-4, -2), (-1, -2))
        graph.add_subpath(self.line((1, 2), (4, 2)).get_points())
        for x in (-.9, 0., .9):
            self.assertIsNone(self.axes.i2gp(x, graph))
        np.testing.assert_array_equal(self.axes.i2gp(-1., graph), [-1, -2, 0])
        np.testing.assert_array_equal(self.axes.i2gp(1., graph), [1, 2, 0])

    def test_translated_and_rotated_axes_use_public_inverse(self):
        self.axes.shift[:] = [7, -3, 2]
        self.axes.transform[:] = [[0, -2, 0], [3, 0, 0], [0, 0, 1]]
        graph = self.line()
        for x in (-5., -2.3, 1., 5.):
            np.testing.assert_allclose(self.axes.i2gp(x, graph), self.axes.c2p(x, x + 2), atol=1e-6)

    def test_live_record_edits_are_observed_without_stale_query_cache(self):
        graph = self.line()
        before = self.axes.i2gp(2., graph)
        graph.points[:, 1] += 3
        after = self.axes.i2gp(2., graph)
        np.testing.assert_allclose(after - before, [0, 3, 0])

    def test_unbound_graph_queries_its_last_geometry_not_changing_function(self):
        factor = [1.]
        graph = self.axes.get_graph(lambda x: x)
        self.axes.bind_graph_to_func(graph, lambda xs: factor[0] * xs, jagged=True)
        graph.update()
        self.axes.unbind_graph_from_func(graph)
        factor[0] = 3.
        np.testing.assert_allclose(self.axes.i2gp(1.5, graph), [1.5, 1.5, 0])
        graph.update()
        np.testing.assert_allclose(self.axes.i2gp(1.5, graph), [1.5, 1.5, 0])

    def test_empty_and_single_point_paths(self):
        self.assertIsNone(self.axes.i2gp(0., self.native.VMobject()))
        graph = self.native.VMobject([[2, 3, 0]])
        np.testing.assert_array_equal(self.axes.i2gp(2., graph), [2, 3, 0])
        self.assertIsNone(self.axes.i2gp(1., graph))

    def test_vertical_curve_chooses_first_exact_point(self):
        graph = self.line((2, -2), (2, 3))
        np.testing.assert_array_equal(self.axes.i2gp(2., graph), [2, -2, 0])
        self.assertIsNone(self.axes.i2gp(1.99, graph))

    def test_overlapping_x_ranges_choose_first_continuous_curve(self):
        graph = self.line((-2, 1), (2, 1))
        graph.add_subpath(self.line((-2, 3), (2, 3)).points)
        np.testing.assert_allclose(self.axes.i2gp(.2, graph), [.2, 1, 0])

    def test_public_curve_evaluator_override_runs_and_exceptions_propagate(self):
        graph = self.line()
        original = graph.get_nth_curve_function
        calls = []
        def factory(index):
            function = original(index)
            def evaluate(t):
                calls.append(t)
                return function(t)
            return evaluate
        graph.get_nth_curve_function = factory
        self.axes.i2gp(.35, graph)
        self.assertGreater(len(calls), 2)
        self.assertTrue(all(0 <= t <= 1 for t in calls))
        failure = LookupError('curve evaluator failed')
        def fail(index):
            raise failure
        graph.get_nth_curve_function = fail
        with self.assertRaises(LookupError) as raised:
            self.axes.i2gp(0., graph)
        self.assertIs(raised.exception, failure)

    def test_nonfinite_query_refuses_before_function_or_curve_evaluation(self):
        graph = self.line()
        graph.underlying_function = lambda x: self.fail('must not execute')
        for value in (np.nan, np.inf, -np.inf):
            with self.assertRaises(ValueError):
                self.axes.i2gp(value, graph)

    def test_nonfinite_and_malformed_curve_points_and_coordinates_refuse(self):
        for value in ([1, 2], [1, 2, np.nan], [1j, 2, 3]):
            graph = self.line()
            graph.get_nth_curve_function = lambda index: lambda t: value
            with self.subTest(value=value), self.assertRaises(ValueError):
                self.axes.i2gp(0, graph)
        graph = self.line()
        self.axes.p2c = lambda point: [np.nan, 0]
        with self.assertRaisesRegex(ValueError, 'inverse coordinates'):
            self.axes.i2gp(0., graph)

    def test_invalid_boundary_tables_refuse_instead_of_searching_fake_edges(self):
        graph = self.line()
        for ends in ([], [1, 2], [2, 2], [-2, 2], [[2]], [0], [2.], [0, 999, 2]):
            graph.get_subpath_end_indices = lambda: np.array(ends)
            with self.subTest(ends=ends), self.assertRaises(ValueError):
                self.axes.i2gp(0., graph)

    def test_query_budgets_refuse_before_curve_callbacks(self):
        graph = self.line()
        graph.get_nth_curve_function = lambda index: self.fail('must not execute')
        with patch.object(graphing, '_MAX_RECORDS', 1), self.assertRaisesRegex(ValueError, 'bounded'):
            self.axes.i2gp(0., graph)

    def test_analytic_queries_keep_scalar_semantics_and_coordinate_dispatch(self):
        graph = self.line()
        calls = []
        graph.underlying_function = lambda x: calls.append(x) or x * x
        np.testing.assert_allclose(self.axes.i2gp(.25, graph), [.25, .0625, 0])
        self.assertEqual(calls, [.25])
        graph.underlying_function = 42
        with self.assertRaises(TypeError):
            self.axes.i2gp(0., graph)


if __name__ == '__main__':
    unittest.main()
