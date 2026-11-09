"""Actual-native 3D coordinate charts, regenerated plots and live wireframes.

The chart oracle uses independent coordinate queries or explicit transforms;
no replacement sampler, normal calculator or renderer stands in for Atlas.
"""
import copy
import io
from pathlib import Path
import pickle
import tempfile
import types
import unittest

import numpy as np
import manimlib as m


DOMAIN = dict(u_range=(-1, 1), v_range=(-1, 1), resolution=(3, 4), shading=(0, 0, 0))


def axes():
    return m.ThreeDAxes(x_range=(-2, 2), y_range=(-2, 2), z_range=(-2, 2))


def coordinates(shape=(3, 4)):
    return [(u, v) for u in np.linspace(-1, 1, shape[0]) for v in np.linspace(-1, 1, shape[1])]


def expected(chart, function, shape=(3, 4)):
    return np.array([chart.c2p(*function(u, v)) for u, v in coordinates(shape)])


class SurfacePlottingTests(unittest.TestCase):
    def test_parametric_surface_uses_rotated_live_axis_vectors(self):
        chart = axes().rotate(np.pi / 2).shift((1, 2, 3))
        plot = chart.get_parametric_surface(lambda u, v: (u, v, 0), **DOMAIN)
        oracle = np.array([(1 - v, 2 + u, 3) for u, v in coordinates()])
        np.testing.assert_allclose(plot.get_points(), oracle, atol=2e-6, rtol=0)

    def test_nonuniform_scale_reflection_and_shear_map_all_three_components(self):
        for matrix in (np.diag((-2., 3., .5)), np.array(((1, .5, .25), (0, 2, .5), (0, 0, 1)))):
            with self.subTest(matrix=matrix):
                chart = axes().apply_matrix(matrix, about_point=m.ORIGIN).shift((.5, 1, -.5))
                function = lambda u, v: (u, v, u + 2*v)
                plot = chart.get_parametric_surface(function, **DOMAIN)
                oracle = np.array([matrix @ np.array(function(u, v)) + (.5, 1, -.5) for u, v in coordinates()])
                np.testing.assert_allclose(plot.get_points(), oracle, atol=3e-6, rtol=0)

    def test_graph_and_parametric_front_doors_share_the_same_native_geometry(self):
        chart = axes().rotate(.6, axis=m.RIGHT).stretch(2, 1).shift((1, -1, .5))
        direct = chart.get_parametric_surface(lambda u, v: (u, v, u*v), **DOMAIN)
        graph = chart.get_graph(lambda u, v: u*v, **DOMAIN)
        np.testing.assert_array_equal(direct.data, graph.data)
        np.testing.assert_allclose(graph.get_points(), expected(chart, lambda u, v: (u, v, u*v)), atol=2e-6)

    def test_numpy_domains_are_not_used_as_ambiguous_truth_values(self):
        chart = axes()
        plot = chart.get_graph(lambda u, v: u + v, u_range=np.array((-1., 1.)),
                               v_range=np.array((-1., 1.)), resolution=(3, 4))
        np.testing.assert_allclose(plot.get_points(), expected(chart, lambda u, v: (u, v, u+v)), atol=2e-6)

    def test_default_domains_follow_axes_not_unit_surface_defaults(self):
        chart = m.ThreeDAxes(x_range=(-3, 2), y_range=(-1, 4), z_range=(-2, 2))
        graph = chart.get_graph(lambda u, v: 0, resolution=(2, 2))
        self.assertEqual(tuple(graph.u_range), (-3, 2))
        self.assertEqual(tuple(graph.v_range), (-1, 4))
        np.testing.assert_allclose(graph.get_points(), [chart.c2p(u, v, 0) for u in (-3, 2) for v in (-1, 4)], atol=2e-6)

    def test_regeneration_retains_chart_and_observes_new_chart_and_function_state(self):
        for method in ('get_graph', 'get_parametric_surface'):
            with self.subTest(method=method):
                chart = axes().rotate(.7, axis=m.RIGHT).shift((1, 2, 3))
                state = [0.]
                xyz = lambda u, v: (u, v, state[0] + u*v)
                function = (lambda u, v: xyz(u, v)[2]) if method == 'get_graph' else xyz
                plot = getattr(chart, method)(function, **DOMAIN)
                initial = plot.data.copy()
                plot.init_points()
                np.testing.assert_array_equal(plot.data, initial)
                state[0] = 1
                chart.shift((.5, 0, 0)).rotate(.2, axis=m.OUT)
                plot.init_points()
                np.testing.assert_allclose(plot.get_points(), expected(chart, xyz), atol=3e-6, rtol=0)

    def test_world_space_normals_follow_affine_chart_not_world_axis_scaling(self):
        matrix = np.array(((2., .5, 0), (0, 1, .5), (.25, 0, -1)))
        chart = axes().apply_matrix(matrix, about_point=m.ORIGIN)
        plot = chart.get_parametric_surface(lambda u, v: (u, v, u + 2*v), **DOMAIN)
        normal = np.cross(matrix @ (1, 0, 1), matrix @ (0, 1, 2))
        normal /= np.linalg.norm(normal)
        np.testing.assert_allclose(plot.get_unit_normals(), np.tile(normal, (12, 1)), atol=3e-4, rtol=0)
        chart.shift((1, 2, 3))
        plot.init_points()
        np.testing.assert_allclose(plot.get_unit_normals(), np.tile(normal, (12, 1)), atol=5e-4, rtol=0)

    def test_authored_nonlinear_coordinate_chart_is_not_linearized(self):
        class CurvedAxes(m.ThreeDAxes):
            def coords_to_point(self, *coords):
                point = super().coords_to_point(*coords)
                return point + (0, 0, coords[0]**2)
        chart = CurvedAxes(x_range=(-2, 2), y_range=(-2, 2), z_range=(-2, 2))
        plot = chart.get_parametric_surface(lambda u, v: (u, v, 0), **DOMAIN)
        np.testing.assert_allclose(plot.get_points(), [(u, v, u*u) for u, v in coordinates()], atol=2e-6)
        chart.shift((0, 1, 0))
        plot.init_points()
        np.testing.assert_allclose(plot.get_points(), [(u, v+1, u*u) for u, v in coordinates()], atol=2e-6)

    def test_authored_c2p_override_receives_actual_sampled_coordinates(self):
        chart = axes()
        calls = []
        def map_point(self, x, y, z):
            calls.append((x, y, z))
            return (x, 2*y, z + x*x)
        chart.c2p = types.MethodType(map_point, chart)
        plot = chart.get_graph(lambda u, v: u*v, **DOMAIN)
        self.assertTrue(calls)
        self.assertIn((0., -1., 0.), calls)
        np.testing.assert_allclose(plot.get_points(), [(u, 2*v, u*v+u*u) for u, v in coordinates()], atol=2e-6)

    def test_authored_function_is_called_once_per_native_sample(self):
        chart = axes().rotate(.3)
        calls, reference_calls = [], []
        def authored(u, v):
            calls.append((u, v))
            return u + v
        def reference(u, v):
            reference_calls.append((u, v))
            return (u, v, u + v)
        plot = chart.get_graph(authored, **DOMAIN)
        m.ParametricSurface(reference, **DOMAIN)
        self.assertEqual(calls, reference_calls)
        calls.clear()
        plot.init_points()
        self.assertEqual(calls, reference_calls)

    def test_scene_bound_regeneration_preserves_style_children_and_live_views(self):
        chart = axes().rotate(.6).shift((1, 2, 0))
        plot = chart.get_graph(lambda u, v: u + v, **DOMAIN)
        plot.set_color(m.RED).set_opacity(.3)
        extra = m.Dot()
        plot.add(extra)
        calls = []
        updater = lambda obj: calls.append(1)
        plot.add_updater(updater, call=False)
        scene = m.Scene()
        scene.add(plot)
        view = plot.get_points()
        colors = plot.data['rgba'].copy()
        chart.shift((0, 0, 1))
        plot.init_points()
        np.testing.assert_allclose(view, expected(chart, lambda u, v: (u, v, u+v)), atol=2e-6)
        np.testing.assert_array_equal(plot.data['rgba'], colors)
        self.assertIs(plot._scene, scene)
        self.assertIs(plot[0], extra)
        self.assertIs(plot.updaters[0], updater)
        self.assertEqual(calls, [])
        self.assertEqual(tuple(scene.mobjects), (plot,))

    def test_copy_and_deepcopy_keep_external_axes_binding_without_mutating_source(self):
        chart = axes().rotate(.5)
        plot = chart.get_graph(lambda u, v: u + v, **DOMAIN)
        before = plot.get_points().copy()
        clones = (plot.copy(), copy.copy(plot), copy.deepcopy(plot))
        chart.shift((0, 0, 1))
        for clone in clones:
            clone.init_points()
            np.testing.assert_allclose(clone.get_points(), before + (0, 0, 1), atol=2e-6)
            np.testing.assert_array_equal(plot.get_points(), before)

    def test_picklable_parametric_recipe_round_trips_with_its_axes(self):
        chart = axes().rotate(.5)
        function = m.Surface(resolution=(2, 2)).uv_func
        plot = chart.get_parametric_surface(function, **DOMAIN)
        before = plot.get_points().copy()
        clone = pickle.loads(pickle.dumps(plot))
        np.testing.assert_array_equal(clone.get_points(), before)
        clone.passed_uv_func.axes.shift((0, 0, 1))
        clone.init_points()
        np.testing.assert_allclose(clone.get_points(), before + (0, 0, 1), atol=2e-6)
        np.testing.assert_array_equal(plot.get_points(), before)

    def test_authored_exception_is_preserved_without_partial_publication(self):
        error = LookupError('authored surface error')
        fail = [False]
        def function(u, v):
            if fail[0]:
                raise error
            return (u, v, 0)
        chart = axes().rotate(.5)
        plot = chart.get_parametric_surface(function, **DOMAIN)
        before = plot.data.copy()
        fail[0] = True
        with self.assertRaises(LookupError) as raised:
            plot.init_points()
        self.assertIs(raised.exception, error)
        np.testing.assert_array_equal(plot.data, before)
        fail[0] = False
        plot.init_points()
        np.testing.assert_array_equal(plot.data, before)

    def test_non_real_nonfinite_and_non_scalar_graph_outputs_are_refused(self):
        chart = axes()
        for output in (np.nan, np.inf, 1+1j, [1], np.array((1, 2)), 'one'):
            with self.subTest(output=output), self.assertRaises((TypeError, ValueError)):
                chart.get_graph(lambda u, v: output, **DOMAIN)
        for output in ((1, 2), (1, 2, np.inf), (1, 2, 3j), (1, 2, 3, 4)):
            with self.subTest(output=output), self.assertRaises((TypeError, ValueError)):
                chart.get_parametric_surface(lambda u, v: output, **DOMAIN)

    def test_finite_data_coordinates_can_map_into_representable_world_coordinates(self):
        chart = axes()
        chart.c2p = types.MethodType(lambda self, x, y, z: (x*1e-20, y, z), chart)
        plot = chart.get_parametric_surface(lambda u, v: (1e40*u, v, 0), **DOMAIN)
        np.testing.assert_allclose(plot.get_points(), [(1e20*u, v, 0) for u, v in coordinates()], rtol=1e-6)
        with self.assertRaisesRegex(ValueError, 'float32'):
            chart.get_parametric_surface(lambda u, v: (1e80, v, 0), **DOMAIN)

    def test_axes_mutation_during_sampling_refuses_inconsistent_candidate(self):
        chart = axes()
        mutate = [False]
        def function(u, v):
            if mutate[0]:
                mutate[0] = False
                chart.shift((0, 0, 1))
            return (u, v, 0)
        plot = chart.get_parametric_surface(function, **DOMAIN)
        before = plot.data.copy()
        mutate[0] = True
        with self.assertRaisesRegex(RuntimeError, 'axes changed during sampling'):
            plot.init_points()
        np.testing.assert_array_equal(plot.data, before)
        np.testing.assert_allclose(chart.c2p(0, 0, 0), (0, 0, 1), atol=2e-6)
        plot.init_points()
        np.testing.assert_allclose(plot.get_points(), before['point'] + (0, 0, 1), atol=2e-6)
        mutate[0] = True
        with self.assertRaisesRegex(RuntimeError, 'axes changed during sampling'):
            chart.get_parametric_surface(function, **DOMAIN)

    def test_chart_method_replacement_during_sampling_is_detected(self):
        chart = axes()
        mutate = [False]
        original = chart.c2p
        def function(u, v):
            if mutate[0]:
                mutate[0] = False
                chart.c2p = types.MethodType(lambda self, x, y, z: (x, y, z+1), chart)
            return (u, v, 0)
        plot = chart.get_parametric_surface(function, **DOMAIN)
        before = plot.data.copy()
        mutate[0] = True
        with self.assertRaisesRegex(RuntimeError, 'axes changed during sampling'):
            plot.init_points()
        np.testing.assert_array_equal(plot.data, before)
        chart.c2p = original
        plot.init_points()

    def test_invalid_request_does_not_evaluate_function(self):
        chart = axes()
        calls = []
        with self.assertRaises((TypeError, ValueError)):
            chart.get_graph(lambda u, v: calls.append(1), resolution=(513, 513))
        self.assertEqual(calls, [])
        with self.assertRaises(TypeError):
            chart.get_parametric_surface(3, **DOMAIN)

    def test_direct_parametric_constructor_preflights_grid_and_preserves_subclass_recipe(self):
        calls = []
        for shape in ((513, 513), (-1, 2), (2.5, 3), (2, 3, 4)):
            with self.subTest(shape=shape), self.assertRaises((ValueError, TypeError)):
                m.ParametricSurface(lambda u, v: calls.append((u, v)), resolution=shape)
        self.assertEqual(calls, [])
        class Authored(m.ParametricSurface):
            pass
        function = lambda u, v: (u, v, 0)
        obj = Authored(function, **DOMAIN)
        self.assertIs(obj.passed_uv_func, function)
        self.assertIsInstance(obj, Authored)
        obj.init_points()
        np.testing.assert_allclose(obj.get_points(), [(u, v, 0) for u, v in coordinates()], atol=2e-6)

    def test_extreme_sampling_options_are_bounded_without_native_allocation(self):
        from fmn_python.surface_plotting import _sampling_options
        import itertools
        for shape in ((50_000, 50_000), (2**63, 0), itertools.repeat(2)):
            with self.subTest(shape=shape), self.assertRaises((ValueError, TypeError)):
                _sampling_options(dict(resolution=shape))
        for options in (dict(epsilon=0), dict(normal_nudge=np.inf), dict(u_range=(0, np.inf)),
                        dict(v_range=itertools.repeat(1))):
            with self.subTest(options=options), self.assertRaises(ValueError):
                _sampling_options(options)
        self.assertEqual(_sampling_options(dict(resolution=(512, 512)))["resolution"], (512, 512))

    def test_live_chart_plot_and_mesh_render_match_independent_world_recipe(self):
        samples = []
        def scene_type(direct):
            class Scene(m.Scene):
                def construct(self):
                    chart = axes().rotate(np.pi/2).shift((0, 0, .25))
                    tracker = m.ValueTracker(0)
                    if direct:
                        # Independent analytic chart: (x,y,z) -> (-y,x,z+.25).
                        plot = m.ParametricSurface(lambda u, v: (-v, u, .25 + tracker.get_value() + u*v/4), **DOMAIN)
                    else:
                        plot = chart.get_graph(lambda u, v: tracker.get_value() + u*v/4, **DOMAIN)
                    plot.set_color(m.BLUE).set_opacity(.9)
                    plot.add_updater(lambda obj: obj.init_points(), call=False)
                    wire = m.SurfaceMesh(plot, resolution=(3, 3), stroke_width=3, normal_nudge=.02)
                    wire.add_updater(lambda obj: obj.init_points(), call=False)
                    wire.add_updater(lambda obj: samples.append((len(obj), plot.uv_to_point(0, 0).copy())), call=False)
                    self.camera.frame.set_euler_angles(phi=.7)
                    self.add(plot, wire)
                    self.play(tracker.animate.set_value(1), run_time=.5, rate_func=m.linear)
            return Scene
        with tempfile.TemporaryDirectory(prefix='fmn-live-chart-') as directory:
            outputs = []
            for direct, threads in ((False, 1), (False, 4), (True, 1)):
                path = Path(directory) / f'{direct}-{threads}.y4m'
                result = scene_type(direct)().render(path, format='y4m', resolution=(96, 54), fps=8, threads=threads)
                self.assertEqual(result.frame_count, 4)
                outputs.append(path.read_bytes())
            self.assertEqual(outputs[0], outputs[1])
            self.assertEqual(outputs[0], outputs[2])
            payload = outputs[0].split(b'\n', 1)[1]
            frame_size = 96*54*3//2
            frames = [payload[i+6:i+6+frame_size] for i in range(0, len(payload), frame_size+6)]
            self.assertEqual(len(frames), 4)
            self.assertNotEqual(frames[0], frames[-1])
        self.assertTrue(samples)
        self.assertTrue(all(count == 6 and np.isfinite(point).all() for count, point in samples))
        self.assertAlmostEqual(samples[-1][1][2], 1.25, places=5)


def run_surface_plotting_acceptance():
    stream = io.StringIO()
    result = unittest.TextTestRunner(stream=stream, verbosity=2).run(
        unittest.defaultTestLoader.loadTestsFromTestCase(SurfacePlottingTests))
    if not result.wasSuccessful():
        raise AssertionError(stream.getvalue())
    print(stream.getvalue())
    return result.testsRun


if __name__ == '__main__':
    run_surface_plotting_acceptance()
