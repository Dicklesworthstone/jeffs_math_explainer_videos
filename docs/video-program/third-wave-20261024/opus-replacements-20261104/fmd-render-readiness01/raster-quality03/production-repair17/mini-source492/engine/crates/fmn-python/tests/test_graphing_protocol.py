"""Real NumPy/callable graph protocols with explicit native geometry doubles.

The unchanged production adapter is loaded below. These tests do not establish
native smoothing, engine records, pixels, or installed-wheel acceptance.
"""
from __future__ import annotations

import copy
import importlib.util
import itertools
from pathlib import Path
import sys
from types import ModuleType
import unittest
from unittest.mock import patch

import numpy as np

MODULE = Path(__file__).resolve().parents[1] / "python/fmn_python/graphing.py"
# The adapter uses package-relative imports (6915ef65), so load it as a
# submodule of fmn_python: the importable one, else this source tree's.
try:
    importlib.import_module("fmn_python")
except ImportError:
    sys.path.insert(0, str(MODULE.parents[1]))
spec = importlib.util.spec_from_file_location("fmn_python.graphing_under_test", MODULE)
graphing = importlib.util.module_from_spec(spec)
spec.loader.exec_module(graphing)


def fixture():
    native = ModuleType("native_graph_geometry_double")
    native._np = np
    native.smooth_calls = []
    native.fail_smoothing = None

    class VMobject:
        pointlike_data_keys = ["point"]
        def __init__(self, points=()):
            self.points = np.asarray(points, dtype=float).reshape(-1, 3).copy()
            self.updaters = []
            self.submobjects = []
            self.uniforms = {"depth": True}
            self.style = {"color": "blue", "width": 3}
            self.matches = 0
            self.discontinuities = ()

        def get_points(self):
            return self.points

        # The adapter snapshots the record view and family identity around
        # user callbacks (6915ef65); this detached double's only native
        # record column is its points.
        @property
        def data(self):
            return self.points

        def get_family(self, recurse=True):
            family = [self]
            for child in self.submobjects if recurse else ():
                # Tests hang opaque marker objects as children; they are leaves.
                family.extend(child.get_family() if hasattr(child, "get_family") else [child])
            return family

        def _is_bound(self):
            return False

        def get_num_points(self):
            return len(self.points)

        def set_points_as_corners(self, points):
            # Test geometry only: shared-anchor corner layout, not a renderer.
            points = np.asarray(points, dtype=float)
            out = []
            for left, right in zip(points, points[1:]):
                out.extend((left, (left + right) / 2))
            if len(points):
                out.append(points[-1])
            self.points = np.asarray(out).reshape(-1, 3)
            return self

        def make_smooth(self, **options):
            native.smooth_calls.append((self.points.copy(), options))
            if native.fail_smoothing is not None:
                raise native.fail_smoothing
            return self

        def add_subpath(self, points):
            points = np.asarray(points, dtype=float)
            if not len(self.points):
                self.points = points.copy()
            elif len(points):
                self.points = np.concatenate((self.points, self.points[-1:], points))
            return self

        def set_points(self, points):
            self.points = np.asarray(points).copy()
            self.matches += 1
            return self

        def add_updater(self, function, call=True):
            self.updaters.append(function)
            if call:
                function(self)
            return self

        def remove_updater(self, function):
            self.updaters = [item for item in self.updaters if item is not function]
            return self

        def update(self):
            for function in tuple(self.updaters):
                function(self)
            return self

        def get_subpath_end_indices(self):
            ends = [index for index in range(0, len(self.points) - 2, 2)
                    if np.array_equal(self.points[index], self.points[index + 1])
                    and not np.array_equal(self.points[index + 1], self.points[index + 2])]
            return np.array([*ends, len(self.points) - 1], dtype=int)

        def get_subpaths(self):
            if not len(self.points):
                return []
            ends = self.get_subpath_end_indices()
            starts = [0, *(ends[:-1] + 2)]
            return [self.points[start:end + 1] for start, end in zip(starts, ends)]

        def get_num_curves(self):
            return len(self.points) // 2

        def get_nth_curve_function(self, index):
            a, h, b = self.points[2 * index:2 * index + 3]
            return lambda t: (1 - t) ** 2 * a + 2 * (1 - t) * t * h + t * t * b

    class CoordinateSystem:
        def __init__(self):
            self.x_range = (-2., 2., 1.)
            self.num_sampled_graph_points_per_tick = 5
            self.shift = np.zeros(3)
            self.transform = np.eye(3)
            self.evaluations = []

        def p2c(self, point):
            return np.linalg.solve(self.transform, np.asarray(point) - self.shift)[:2]

        def c2p(self, x, y):
            x, y = np.broadcast_arrays(x, y)
            points = np.stack((x, y, np.zeros_like(x)), axis=-1)
            return points @ self.transform.T + self.shift

        def get_graph(self, function, x_range=None, bind=False, **kwargs):
            domain = self.x_range if x_range is None else x_range
            step = (domain[2] if len(domain) == 3 else 1.) / self.num_sampled_graph_points_per_tick
            xs = np.linspace(domain[0], domain[1], max(2, int(np.ceil((domain[1] - domain[0]) / step)) + 1))
            self.evaluations.append((function, domain, bind, dict(kwargs)))
            graph = VMobject().set_points_as_corners(self.c2p(xs, [function(float(x)) for x in xs]))
            graph.underlying_function = function
            graph.x_range = domain
            graph.discontinuities = kwargs.get("discontinuities", ())
            graph.epsilon = kwargs.get("epsilon", 1e-6)
            if bind:
                self.bind_graph_to_func(graph, function)
            return graph

        def input_to_graph_point(self, x, graph):
            return self.c2p(x, graph.underlying_function(x))

        def i2gp(self, x, graph):
            return self.input_to_graph_point(x, graph)

    native.CoordinateSystem, native.VMobject = CoordinateSystem, VMobject
    return native


class LiveGraphTests(unittest.TestCase):
    def setUp(self):
        self.native = fixture()
        graphing.install_graphing(self.native)
        self.axes = self.native.CoordinateSystem()
        self.graph = self.native.VMobject().set_points_as_corners(
            self.axes.c2p(np.linspace(-2, 2, 9), np.zeros(9)))

    def bind(self, function=lambda xs: xs * xs, **kwargs):
        return self.axes.bind_graph_to_func(self.graph, function, **kwargs)

    def test_binding_evaluates_once_and_installs_one_callback(self):
        # fm-skpp: the Reference binds through add_updater(call=True), so the
        # function shapes the graph at bind time; frames then update it.
        calls = []
        result = self.bind(lambda xs: calls.append(xs.copy()) or xs)
        self.assertIs(result, self.graph)
        self.assertEqual(len(calls), 1)
        self.graph.update()
        self.assertEqual(len(calls), 2)
        self.assertEqual(len(self.graph.updaters), 1)

    def test_a_failing_first_evaluation_leaves_the_graph_unbound(self):
        author = lambda current: None
        self.graph.add_updater(author)
        previous = self.graph.points.copy()
        failure = LookupError("first evaluation failed")
        def function(xs):
            raise failure
        with self.assertRaises(LookupError) as raised:
            self.bind(function)
        self.assertIs(raised.exception, failure)
        self.assertEqual(self.graph.updaters, [author])
        self.assertNotIn(graphing._BINDING, vars(self.graph))
        np.testing.assert_array_equal(self.graph.points, previous)

    def test_updates_keep_full_domain_as_discontinuities_move(self):
        position = [0.]
        self.bind(lambda xs: 1 / (xs - position[0]), jagged=True,
                  get_discontinuities=lambda: position)
        for value in [0., 1., -.75, .5, 0., 1.25] * 4:
            position[0] = value
            self.graph.update()
            paths = self.graph.get_subpaths()
            self.assertEqual(len(paths), 2)
            self.assertEqual(paths[0][0, 0], -2)
            self.assertEqual(paths[-1][-1, 0], 2)
            self.assertLess(paths[0][-1, 0], value)
            self.assertGreater(paths[1][0, 0], value)
            self.assertTrue(np.isfinite(self.graph.points).all())

    def test_baseline_grid_is_not_mutated_or_pruned_across_updates(self):
        calls, positions = [], [0.]
        def evaluate(xs):
            calls.append(xs.copy())
            return np.where(xs < positions[0], -1., 1.)
        self.bind(evaluate, get_discontinuities=lambda: positions)
        self.graph.update()
        first = calls[-1].copy()
        positions[0] = 1.
        self.graph.update()
        positions[0] = 0.
        self.graph.update()
        np.testing.assert_array_equal(calls[-1], first)
        self.assertFalse(vars(self.graph)[graphing._BINDING].samples.flags.writeable)

    def test_smoothing_occurs_per_disconnected_path(self):
        self.bind(lambda xs: np.sign(xs), get_discontinuities=lambda: [0.])
        self.native.smooth_calls.clear()  # the bind's own evaluation smoothed too
        self.graph.update()
        self.assertEqual(len(self.native.smooth_calls), 2)
        for points, options in self.native.smooth_calls:
            self.assertTrue(np.all(points[:, 0] < 0) or np.all(points[:, 0] > 0))
            self.assertEqual(options, {"approx": True, "recurse": False})

    def test_jagged_never_calls_smoother(self):
        self.bind(jagged=True)
        self.graph.update()
        self.assertEqual(self.native.smooth_calls, [])

    def test_discontinuity_endpoints_duplicates_and_out_of_range(self):
        self.bind(lambda xs: xs, get_discontinuities=lambda: [-2., 0., 0., 2., 99., -99.])
        self.graph.update()
        paths = self.graph.get_subpaths()
        self.assertEqual(len(paths), 2)
        self.assertGreater(paths[0][0, 0], -2)
        self.assertLess(paths[-1][-1, 0], 2)

    def test_overlapping_exclusion_bands_merge_without_negative_segments(self):
        self.graph.epsilon = .2
        self.bind(lambda xs: xs, get_discontinuities=lambda: [.1, 0., .05])
        self.graph.update()
        paths = self.graph.get_subpaths()
        self.assertEqual(len(paths), 2)
        self.assertAlmostEqual(paths[0][-1, 0], -.2)
        self.assertAlmostEqual(paths[1][0, 0], .3)

    def test_domain_can_disappear_and_recover_without_rebinding(self):
        self.graph.epsilon = 5
        positions = [0.]
        calls = []
        self.bind(lambda xs: calls.append(xs) or xs, get_discontinuities=lambda: positions)
        self.graph.update()
        self.assertEqual(len(self.graph.points), 0)
        self.assertEqual(calls, [])
        positions.clear()
        self.graph.update()
        self.assertGreater(len(self.graph.points), 2)
        self.assertEqual(self.graph.points[0, 0], -2)
        self.assertEqual(self.graph.points[-1, 0], 2)

    def test_user_and_coordinate_failures_preserve_last_complete_geometry(self):
        failure = LookupError("author failure")
        current = [lambda xs: xs]
        self.bind(lambda xs: current[0](xs))
        self.graph.update()
        previous = self.graph.points.copy()
        def fail(xs):
            raise failure
        current[0] = fail
        with self.assertRaises(LookupError) as raised:
            self.graph.update()
        self.assertIs(raised.exception, failure)
        np.testing.assert_array_equal(self.graph.points, previous)
        self.assertFalse(graphing._GRAPH_UPDATES.busy(self.graph))
        current[0] = lambda xs: xs * 2
        self.graph.update()
        self.assertFalse(np.array_equal(self.graph.points, previous))

    def test_invalid_return_shapes_and_nonfinite_values_never_modify_graph(self):
        for function in [lambda xs: xs[:, None], lambda xs: xs[:-1],
                         lambda xs: xs.astype(complex), lambda xs: np.nan,
                         lambda xs: ["bad"] * len(xs), lambda xs: np.inf * xs]:
            with self.subTest(function=function):
                previous = self.graph.points.copy()
                with np.errstate(invalid="ignore"), self.assertRaises((TypeError, ValueError)):
                    self.bind(function)
                np.testing.assert_array_equal(self.graph.points, previous)
                self.assertEqual(self.graph.updaters, [])

    def test_coordinate_failure_preserves_geometry(self):
        self.bind()
        previous = self.graph.points.copy()
        self.axes.c2p = lambda xs, ys: np.full((len(xs), 3), 1e100)
        with self.assertRaisesRegex(ValueError, "coordinates"):
            self.graph.update()
        np.testing.assert_array_equal(self.graph.points, previous)

    def test_smoothing_failure_never_publishes_a_partial_update(self):
        self.bind()
        previous = self.graph.points.copy()
        failure = RuntimeError("native smoothing failure")
        self.native.fail_smoothing = failure
        with self.assertRaises(RuntimeError) as raised:
            self.graph.update()
        self.assertIs(raised.exception, failure)
        np.testing.assert_array_equal(self.graph.points, previous)

    def test_constant_functions_broadcast(self):
        self.bind(lambda xs: 3.5)
        self.graph.update()
        np.testing.assert_allclose(self.graph.points[:, 1], 3.5)

    def test_mutating_callback_input_does_not_change_x_samples(self):
        def function(xs):
            xs[:] = 1.
            return xs
        self.bind(function)
        for _ in range(3):
            self.graph.update()
            self.assertEqual(self.graph.points[0, 0], -2)
            self.assertEqual(self.graph.points[-1, 0], 2)
            np.testing.assert_allclose(self.graph.points[:, 1], 1.)

    def test_axes_transforms_and_public_override_are_live(self):
        self.bind(lambda xs: 2 * xs, jagged=True)
        self.axes.shift[:] = [3, 4, 0]
        self.axes.transform[:] = [[0., -2., 0.], [3., 0., 0.], [0., 0., 1.]]
        self.graph.update()
        for point in self.graph.points:
            x, y = self.axes.p2c(point)
            self.assertAlmostEqual(y, 2 * x)
        self.assertEqual(self.axes.p2c(self.graph.points[0])[0], -2.)
        self.assertEqual(self.axes.p2c(self.graph.points[-1])[0], 2.)

    def test_binding_uses_generic_public_inverse_not_x_axis_private_fields(self):
        self.axes.shift[:] = [4, 5, 0]
        self.graph.points += self.axes.shift
        self.bind(lambda xs: xs)
        self.graph.update()
        np.testing.assert_allclose(self.graph.points[0], [2, 3, 0])
        self.assertFalse(hasattr(self.axes, "x_axis"))

    def test_style_uniforms_family_and_original_updaters_survive(self):
        child = object()
        self.graph.submobjects.append(child)
        author = lambda current: None
        self.graph.add_updater(author)
        uniforms, style = self.graph.uniforms, self.graph.style
        self.bind(get_discontinuities=lambda: [0.])
        self.graph.update()
        self.assertIs(self.graph.uniforms, uniforms)
        self.assertIs(self.graph.style, style)
        self.assertIs(self.graph.submobjects[0], child)
        self.assertIs(self.graph.updaters[0], author)
        self.assertEqual(self.graph.matches, 2)  # the bind's evaluation, then the update

    def test_rebinding_replaces_only_the_owned_updater_and_preserves_grid(self):
        calls = []
        self.bind(lambda xs: calls.append("old") or xs, get_discontinuities=lambda: [0.])
        self.graph.update()
        baseline = vars(self.graph)[graphing._BINDING].samples
        author = lambda current: calls.append("author")
        self.graph.add_updater(author, call=False)
        self.bind(lambda xs: calls.append("new") or xs)
        self.assertEqual(len(self.graph.updaters), 2)
        self.assertIs(vars(self.graph)[graphing._BINDING].samples, baseline)
        self.graph.update()
        # Each bind evaluates its own function once; the author's updater
        # waits for an update, and the old function never runs again.
        self.assertEqual(calls, ["old", "old", "new", "author", "new"])
        self.assertEqual(len(self.graph.get_subpaths()), 1)

    def test_unbind_retains_geometry_but_stops_future_function_updates(self):
        self.bind()
        self.graph.update()
        previous = self.graph.points.copy()
        self.assertIs(self.axes.unbind_graph_from_func(self.graph), self.graph)
        self.graph.update()
        np.testing.assert_array_equal(self.graph.points, previous)
        self.assertNotIn("underlying_function", vars(self.graph))
        self.assertNotIn(graphing._BINDING, vars(self.graph))
        self.axes.unbind_graph_from_func(self.graph)

    def test_foreign_axes_cannot_remove_another_axes_binding(self):
        self.bind()
        with self.assertRaisesRegex(ValueError, "another coordinate"):
            self.native.CoordinateSystem().unbind_graph_from_func(self.graph)
        self.assertEqual(len(self.graph.updaters), 1)

    def test_reentry_and_rebind_during_callback_refuse_and_recover(self):
        for operation in (lambda: self.graph.update(), lambda: self.bind(),
                          lambda: self.axes.unbind_graph_from_func(self.graph)):
            # The bind's own first evaluation already re-enters.
            with self.assertRaises(RuntimeError):
                self.bind(lambda xs: operation())
            self.assertFalse(graphing._GRAPH_UPDATES.busy(self.graph))
            self.assertEqual(self.graph.updaters, [])
        self.bind()
        self.graph.update()

    def test_count_budgets_stop_infinite_discontinuity_iterables(self):
        with self.assertRaisesRegex(ValueError, "budget"):
            self.bind(get_discontinuities=lambda: itertools.repeat(0.))
        self.assertEqual(self.graph.matches, 0)

    def test_nonfinite_discontinuity_and_oversized_sample_refuse(self):
        with self.assertRaises(ValueError):
            self.bind(get_discontinuities=lambda: [np.nan])
        with patch.object(graphing, "_MAX_SAMPLES", 2), self.assertRaisesRegex(ValueError, "budget"):
            graphing._segments(np.arange(5.), [], 1e-6, np)

    def test_invalid_bind_requests_do_not_install_updaters(self):
        for options in ({"func": None}, {"func": lambda xs: xs, "jagged": 1},
                        {"func": lambda xs: xs, "get_discontinuities": []}):
            with self.assertRaises(TypeError):
                self.axes.bind_graph_to_func(self.graph, **options)
        self.assertEqual(self.graph.updaters, [])
        self.graph.pointlike_data_keys = ["point", "custom_point"]
        with self.assertRaisesRegex(TypeError, "pointlike schema"):
            self.bind()

    def test_deepcopy_preserves_the_closure_binding_and_updates_only_the_copy(self):
        self.bind()
        copied = copy.deepcopy(self.graph)
        self.assertIs(vars(copied)[graphing._BINDING], vars(self.graph)[graphing._BINDING])
        previous = self.graph.points.copy()
        copied.update()
        np.testing.assert_array_equal(self.graph.points, previous)
        self.axes.unbind_graph_from_func(copied)
        self.assertEqual(len(self.graph.updaters), 1)
        self.assertEqual(copied.updaters, [])

    def test_bound_callable_queries_use_one_element_arrays_not_scalar_guessing(self):
        def only_arrays(xs):
            self.assertEqual(xs.ndim, 1)
            return xs ** 2
        self.bind(only_arrays)
        self.assertEqual(self.graph.underlying_function(3.), 9.)
        np.testing.assert_array_equal(self.graph.underlying_function(np.array([[1, 2]])), [[1, 4]])
        np.testing.assert_array_equal(self.axes.i2gp(2., self.graph), [2, 4, 0])

    def test_installer_retains_class_and_method_identity(self):
        cls = self.native.CoordinateSystem
        method = cls.bind_graph_to_func
        graphing.install_graphing(self.native)
        self.assertIs(self.native.CoordinateSystem, cls)
        self.assertIs(cls.bind_graph_to_func, method)


if __name__ == "__main__":
    unittest.main()
