"""Actual-native affine charts and downstream live/geometry graph consumers."""
import copy
import math
from pathlib import Path
import tempfile
import unittest

import numpy as np
import manimlib as m
from fmn_python.coordinate_mapping import install_coordinate_mapping


class CoordinateMappingTests(unittest.TestCase):
    def axes(self, three=False):
        options = dict(x_range=(-2, 2, 1), y_range=(-2, 2, 1), width=4, height=4)
        if three:
            return m.ThreeDAxes(z_range=(-2, 2, 1), depth=4, **options)
        return m.Axes(**options)

    def test_shear_is_an_actual_inverse_in_two_and_three_dimensions(self):
        for three in (False, True):
            with self.subTest(three=three):
                axes = self.axes(three)
                axes.apply_matrix([[1, 1, .2], [0, 1, .3], [0, 0, 1]])
                for coordinates in ((1, 2, 3), (-1.5, .25, -2), (0, 0, 0), (4.5, -6, 1)):
                    coordinates = coordinates[:3 if three else 2]
                    np.testing.assert_allclose(axes.p2c(axes.c2p(*coordinates)), coordinates, atol=2e-6)

    def test_rotation_reflection_and_anisotropic_scale_compose(self):
        for three in (False, True):
            axes = self.axes(three)
            axes.apply_matrix([[-2, .5, .2], [0, .25, .3], [0, 0, 4]])
            axes.rotate(.7, axis=(1, 2, 3)).shift((2, -3, 1))
            point = (1.25, -.75, .5)[:3 if three else 2]
            np.testing.assert_allclose(axes.p2c(axes.c2p(*point)), point, atol=2e-6)

    def test_batched_and_empty_points_preserve_the_reference_tuple_of_arrays(self):
        axes = self.axes(True)
        axes.apply_matrix([[1, .75, .25], [0, 1, .125], [0, 0, -2]])
        coordinates = np.arange(18, dtype=float).reshape(2, 3, 3) / 8
        points = np.array([axes.c2p(*c) for c in coordinates.reshape(-1, 3)]).reshape(2, 3, 3)
        before = points.copy()
        result = axes.p2c(points)
        self.assertIsInstance(result, tuple)
        self.assertEqual(len(result), 3)
        for index, values in enumerate(result):
            self.assertEqual(values.shape, (2, 3))
            np.testing.assert_allclose(values, coordinates[..., index], atol=2e-6)
        result[0][:] = 100
        np.testing.assert_array_equal(points, before)
        empty = axes.p2c(np.empty((0, 3)))
        self.assertEqual([values.shape for values in empty], [(0,), (0,), (0,)])

    def test_numeric_conversion_protocols_remain_supported(self):
        from decimal import Decimal
        axes = self.axes()
        self.assertEqual(axes.p2c([Decimal('1.25'), Decimal('-.5'), Decimal('0')]), (1.25, -.5))

    def test_two_dimensional_off_plane_points_project_onto_the_actual_plane(self):
        axes = self.axes()
        axes.apply_matrix([[1, .75, 0], [0, 1, 0], [.5, -.25, 1]])
        x, y = axes.c2p(1, 0)-axes.c2p(0, 0), axes.c2p(0, 1)-axes.c2p(0, 0)
        normal = np.cross(x, y)
        normal /= np.linalg.norm(normal)
        point = axes.c2p(.75, -1.25) + 7 * normal
        np.testing.assert_allclose(axes.p2c(point), (.75, -1.25), atol=2e-6)

    def test_independently_moved_axes_follow_full_forward_chart_origin(self):
        axes = self.axes(True)
        axes.x_axis.shift((1, 2, 3))
        axes.y_axis.shift((-2, 1, -1))
        axes.z_axis.shift((.5, -.5, 2))
        for point in ((0, 0, 0), (.25, -.5, 1.25)):
            np.testing.assert_allclose(axes.p2c(axes.c2p(*point)), point, atol=2e-6)

    def test_unit_normalization_avoids_overflow_and_underflow(self):
        axes = self.axes(True)
        # Remove constructor rotation residue before applying a forty-order
        # anisotropy: amplified residue would genuinely make the chart singular.
        for index, axis in enumerate(axes.axes):
            endpoints = np.zeros((3, 3))
            endpoints[0, index], endpoints[2, index] = -2, 2
            axis.set_points(endpoints)
        axes.apply_matrix([[1e20, 1e-20, 0], [0, 1e-20, 0], [0, 0, -2]])
        # An enormous x contribution would erase the tiny y contribution in
        # world x. This test chooses a point whose tiny coordinate is observable.
        for point in ((0, 1, .5), (1, 0, -.5)):
            np.testing.assert_allclose(axes.p2c(axes.c2p(*point)), point, atol=2e-6)

    def test_degenerate_charts_refuse_instead_of_returning_plausible_numbers(self):
        for matrix in ([[1, 1, 0], [0, 0, 0], [0, 0, 1]],
                       [[0, 0, 0], [0, 1, 0], [0, 0, 1]],
                       [[1, 1, 0], [0, 1e-16, 0], [0, 0, 1]]):
            axes = self.axes()
            axes.apply_matrix(matrix)
            with self.assertRaises(ValueError):
                axes.p2c((1, 1, 0))
        axes = self.axes(True)
        axes.apply_matrix([[1, 0, 1], [0, 1, 1], [0, 0, 0]])
        with self.assertRaises(ValueError):
            axes.p2c((1, 1, 1))

    def test_invalid_points_and_axis_ranges_are_rejected(self):
        axes = self.axes()
        for point in ((1, 2), (1, 2, 3, 4), (float('nan'), 0, 0), (1j, 2, 0)):
            with self.assertRaises(ValueError):
                axes.p2c(point)
        for domain in ((0, 0, 1), (0, float('inf'), 1)):
            axes.x_range = domain
            with self.assertRaises(ValueError):
                axes.p2c((1, 2, 0))

    def test_native_axes_edits_are_observed_without_stale_basis_caches(self):
        axes = self.axes(True)
        point = (.3, -.7, 1.2)
        for index in range(4):
            axes.apply_matrix([[1, .25, 0], [0, 1, .1], [0, 0, 1]])
            axes.shift((.25, -.1, .2))
            np.testing.assert_allclose(axes.p2c(axes.c2p(*point)), point, atol=2e-6)
        other = copy.deepcopy(axes)
        other.shift((3, 2, 1))
        np.testing.assert_allclose(other.p2c(other.c2p(*point)), point, atol=2e-6)
        np.testing.assert_allclose(axes.p2c(axes.c2p(*point)), point, atol=2e-6)

    def test_complex_plane_alias_uses_the_correct_inverse(self):
        axes = m.ComplexPlane(x_range=(-2, 2, 1), y_range=(-2, 2, 1))
        axes.apply_matrix([[1, .75, 0], [0, -2, 0], [0, 0, 1]])
        value = .75 - 1.25j
        self.assertAlmostEqual(axes.p2n(axes.n2p(value)), value, places=6)

    def test_authored_inverse_and_forward_dispatch_are_not_replaced(self):
        class Authored(m.Axes):
            def coords_to_point(self, *coordinates):
                return super().coords_to_point(*coordinates) + np.array((0, 0, 2))
            def point_to_coords(self, point):
                return ('authored', tuple(point))
        axes = Authored()
        self.assertEqual(axes.p2c((1, 2, 3)), ('authored', (1, 2, 3)))
        # A custom forward chart without an inverse keeps the old dispatch,
        # rather than being silently fitted from a few affine probe points.
        class Forward(m.Axes):
            def c2p(self, *coordinates):
                raise AssertionError('inverse must not probe authored forward code')
        self.assertEqual(Forward().p2c((1, 2, 0)), (1, 2))

    def test_idempotent_installation_preserves_class_and_alias_identity(self):
        import inspect
        method, alias = m.Axes.point_to_coords, m.Axes.p2c
        install_coordinate_mapping(m._FMN_ROOT)
        self.assertIs(m.Axes.point_to_coords, method)
        self.assertIs(m.Axes.p2c, alias)
        from manimlib.mobject.coordinate_systems import Axes
        self.assertIs(Axes, m.Axes)
        self.assertEqual(tuple(inspect.signature(method).parameters), ('self', 'point'))
        np.testing.assert_allclose(self.axes().point_to_coords(point=(1, 2, 0)), (1, 2), atol=2e-6)

    def test_queries_leave_scene_clock_styles_and_updaters_alone(self):
        axes = self.axes()
        scene = m.Scene()
        scene.add(axes)
        calls = []
        axes.add_updater(lambda obj, dt: calls.append(dt), call=False)
        before = [axis.data.copy() for axis in axes.axes]
        for _ in range(5):
            np.testing.assert_allclose(axes.p2c((1, 2, 0)), (1, 2))
        self.assertEqual(calls, [])
        self.assertEqual(scene.get_time(), 0)
        for axis, records in zip(axes.axes, before):
            np.testing.assert_array_equal(axis.data, records)

    def test_bound_graph_retains_its_authored_x_domain_after_shear(self):
        axes = self.axes()
        axes.apply_matrix([[1, .75, 0], [0, 1, 0], [.25, -.2, 1]])
        amplitude = m.ValueTracker(.5)
        graph = axes.get_graph(lambda x: amplitude.get_value()*x*x, x_range=(-1, 1, .5), bind=True)
        scene = m.Scene()
        scene.add(graph)
        scene.play(amplitude.animate.set_value(1), run_time=.125, rate_func=m.linear)
        np.testing.assert_allclose(graph.get_start(), axes.c2p(-1, 1), atol=5e-6)
        np.testing.assert_allclose(graph.get_end(), axes.c2p(1, 1), atol=5e-6)
        np.testing.assert_allclose(axes.i2gp(.5, graph), axes.c2p(.5, .25), atol=5e-6)
        self.assertEqual(len(graph.updaters), 1)

    def test_geometry_only_graph_queries_find_the_correct_coordinate(self):
        axes = self.axes()
        axes.apply_matrix([[1, 1, 0], [0, 1, 0], [.2, -.3, 1]])
        graph = m.VMobject().set_points_as_corners([axes.c2p(-1, 1), axes.c2p(1, -1)])
        for x in (-.75, 0, .5):
            np.testing.assert_allclose(axes.i2gp(x, graph), axes.c2p(x, -x), atol=2e-6)

    def test_sheared_live_graph_renders_like_an_independent_world_line(self):
        def world(x, y):
            return (x + .75*y, y, 0)
        def render(path, reference, threads):
            class Animation(m.Scene):
                def construct(self):
                    amplitude = m.ValueTracker(.5)
                    if reference:
                        samples = np.linspace(-1, 1, 17)
                        graph = m.VMobject(color=m.BLUE).set_points_as_corners([world(x, .5*x) for x in samples])
                        graph.add_updater(lambda obj: obj.set_points_as_corners([
                            world(x, amplitude.get_value()*x) for x in samples]), call=False)
                    else:
                        axes = m.Axes(x_range=(-1, 1, 1), y_range=(-1, 1, 1), width=2, height=2,
                                      num_sampled_graph_points_per_tick=4)
                        axes.apply_matrix([[1, .75, 0], [0, 1, 0], [0, 0, 1]])
                        graph = axes.get_graph(lambda x: amplitude.get_value()*x, x_range=(-1, 1, 1),
                                               color=m.BLUE, use_smoothing=False)
                        axes.bind_graph_to_func(graph, lambda xs: amplitude.get_value()*xs, jagged=True)
                    self.add(graph)
                    self.play(amplitude.animate.set_value(1), run_time=.25, rate_func=m.linear)
            receipt = Animation().render(path, format='y4m', resolution=(96, 54), fps=8, threads=threads)
            self.assertEqual(receipt.frame_count, 2)
            return path.read_bytes()
        with tempfile.TemporaryDirectory(prefix='fmn-coordinates-') as directory:
            outputs = [render(Path(directory)/f'{reference}-{threads}.y4m', reference, threads)
                       for reference in (False, True) for threads in (1, 4)]
        self.assertEqual(outputs[0], outputs[1])
        self.assertEqual(outputs[2], outputs[3])
        self.assertEqual(outputs[0], outputs[2])


def run_coordinate_mapping_acceptance():
    result = unittest.TextTestRunner(verbosity=2).run(unittest.defaultTestLoader.loadTestsFromTestCase(CoordinateMappingTests))
    if not result.wasSuccessful():
        raise AssertionError('native coordinate mapping acceptance failed')


if __name__ in ('__main__', '<run_path>'):
    run_coordinate_mapping_acceptance()
