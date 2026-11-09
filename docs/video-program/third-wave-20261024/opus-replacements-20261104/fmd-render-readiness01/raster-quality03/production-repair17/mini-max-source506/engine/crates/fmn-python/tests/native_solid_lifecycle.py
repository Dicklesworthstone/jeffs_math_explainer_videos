"""Installed and embedded solid subclass acceptance against real Atlas kernels."""
import copy
from pathlib import Path
import tempfile
import unittest

import numpy as np
import manimlib as m
from fmn_python import render_session


def native_torus(shape=(5, 4), r1=2., r2=.5, u_range=(0., 6.283185307179586),
                 v_range=(0., 6.283185307179586)):
    """Independent native builder: do not call a public solid constructor."""
    candidate = m.Surface.__new__(m.Surface)
    m._install_live_state(candidate)
    specs = candidate._build_torus(m._native_surface_shell_factory, r1, r2,
        u_range, v_range, shape, 1, .001, .001, 0)
    assert not specs
    candidate.resolution = shape
    candidate.compute_triangle_indices()
    candidate.set_color(m.GREY)
    candidate.set_shading(.3, .2, .4)
    candidate.apply_depth_test()
    return candidate


class TorusLifecycleTests(unittest.TestCase):
    def test_stock_geometry_and_normals_keep_native_bits(self):
        for shape in ((0, 4), (4, 0), (1, 5), (5, 1), (2, 3), (9, 7)):
            for ur, vr in (((0., 4.), (0., 5.)), ((4., 0.), (3., 1.))):
                with self.subTest(shape=shape, u_range=ur, v_range=vr):
                    expected = native_torus(shape, u_range=ur, v_range=vr)
                    actual = m.Torus(r1=2., r2=.5, resolution=shape, u_range=ur, v_range=vr)
                    for name in ('point', 'd_normal_point', 'rgba'):
                        self.assertEqual(actual.data[name].tobytes(), expected.data[name].tobytes())

    def test_cooperative_hooks_see_recipe_and_keep_their_geometry_and_style(self):
        events = []
        class Hooks:
            def init_data(self):
                events.append(('data', self.r1, self.r2, self.resolution))
                super().init_data()
            def init_points(self):
                events.append('points')
                super().init_points()
                self.shift(m.RIGHT)
            def init_uniforms(self):
                events.append('uniforms')
                super().init_uniforms()
                self.set_shading(.1, .4, .2)
            def init_colors(self):
                events.append('colors')
                super().init_colors()
                self.set_color(m.BLUE)
        class Authored(Hooks, m.Torus):
            pass
        surface = Authored(r1=2., r2=.5, resolution=(5, 4), color=m.RED, z_index=7)
        self.assertEqual(events, [('data', 2., .5, (5, 4)), 'points', 'uniforms', 'colors'])
        expected = native_torus().shift(m.RIGHT)
        np.testing.assert_array_equal(surface.get_points(), expected.get_points())
        np.testing.assert_array_equal(surface.data['d_normal_point'], expected.data['d_normal_point'])
        self.assertEqual(surface.get_color(), m.BLUE)
        self.assertEqual(surface.z_index, 7)
        np.testing.assert_allclose(surface.get_shading(), (.1, .4, .2), atol=1e-7)
        self.assertEqual(m._surface_grid_resolution(surface), (5, 4))

    def test_authored_uv_is_sampled_at_native_grid_and_derivative_points(self):
        visits = []
        class Authored(m.Torus):
            def uv_func(self, u, v):
                visits.append((u, v))
                return u, v, self.r1 - self.r2
        surface = Authored(r1=2., r2=.5, resolution=(3, 4))
        self.assertEqual(len(visits), 36)
        np.testing.assert_array_equal(surface.get_points()[:, 2], 1.5)
        np.testing.assert_allclose(surface.get_unit_normals(), [[0, 0, 1]] * 12, atol=1e-5)
        visits.clear()
        surface.init_points()
        self.assertEqual(len(visits), 36)
        surface.r1 = 3.
        surface.set_resolution((4, 5))
        self.assertEqual(m._surface_grid_resolution(surface), (4, 5))
        np.testing.assert_array_equal(surface.get_points()[:, 2], 2.5)

    def test_replacement_hook_retains_custom_schema_and_authored_children(self):
        child = m.Circle()
        class Authored(m.Torus):
            data_dtype = [*m.Surface.data_dtype, ('tag', np.float32, (1,))]
            def init_data(self):
                super().init_data()
                self.add(child)
            def init_points(self):
                self.set_points([[-1, -1, 0], [-1, 1, 0], [1, -1, 0], [1, 1, 0]])
                self.data['d_normal_point'][:] = self.get_points() + .001 * m.OUT
                self.data['tag'][:] = 17.
            def uv_func(self, u, v):
                raise AssertionError('replaced init_points must not sample uv_func')
        surface = Authored(resolution=(2, 2))
        self.assertIs(surface.submobjects[0], child)
        self.assertEqual(m._surface_grid_resolution(surface), (2, 2))
        np.testing.assert_array_equal(surface.get_triangle_indices(), [0, 2, 1, 1, 2, 3])
        np.testing.assert_array_equal(surface.data['tag'], 17.)
        duplicate = copy.deepcopy(surface)
        self.assertIsNot(duplicate.submobjects[0], child)
        np.testing.assert_array_equal(duplicate.data, surface.data)

    def test_stock_regeneration_and_regridding_preserve_owner_and_recipe(self):
        surface = m.Torus(r1=2., r2=.5, resolution=(5, 4))
        scene = m.Scene()
        scene.add(surface)
        surface.r1 = 1.5
        surface.init_points()
        self.assertIs(surface._scene, scene)
        np.testing.assert_array_equal(surface.get_points(), native_torus(r1=1.5).get_points())
        surface.set_resolution((7, 5))
        self.assertIs(surface._scene, scene)
        self.assertEqual(m._surface_grid_resolution(surface), (7, 5))
        np.testing.assert_array_equal(surface.get_points(), native_torus((7, 5), r1=1.5).get_points())

    def test_mutated_shape_recipe_refuses_both_initial_and_live_publication(self):
        roots = []
        class Authored(m.Torus):
            mutate = True
            def init_data(self):
                roots.append(self)
                super().init_data()
            def uv_func(self, u, v):
                if self.mutate:
                    self.r1 += 1.
                return u, v, 0.
        with self.assertRaisesRegex(RuntimeError, 'shape parameters changed'):
            Authored(resolution=(2, 2))
        self.assertEqual(roots[0].n_records(), 0)
        Authored.mutate = False
        surface = Authored(resolution=(2, 2))
        before = surface.data.copy()
        surface.mutate = True
        for action in (surface.init_points, lambda: surface.set_resolution((3, 4))):
            with self.assertRaisesRegex(RuntimeError, 'shape parameters changed'):
                action()
            np.testing.assert_array_equal(surface.data, before)
            self.assertEqual(m._surface_grid_resolution(surface), (2, 2))

    def test_callback_failure_stops_calls_and_propagates_the_original_exception(self):
        error, visits = ValueError('authored UV failed'), []
        class Authored(m.Torus):
            def uv_func(self, u, v):
                visits.append((u, v))
                if len(visits) == 3:
                    raise error
                return u, v, 0.
        with self.assertRaises(ValueError) as caught:
            Authored(resolution=(4, 5))
        self.assertIs(caught.exception, error)
        self.assertEqual(len(visits), 3)
        self.assertEqual(m.Torus(resolution=(2, 2)).n_records(), 4)

    def test_authored_geometry_reaches_the_native_renderer(self):
        class Authored(m.Torus):
            def init_points(self):
                super().init_points()
                self.shift(m.RIGHT)
        def render(path, surface, threads):
            scene = m.Scene()
            with render_session(scene, path, resolution=(64, 40), fps=24, threads=threads) as session:
                scene.add(surface)
                scene.wait(.125)
            self.assertEqual(session.result.frame_count, 3)
            return path.read_bytes()
        with tempfile.TemporaryDirectory() as tmp:
            tmp = Path(tmp)
            reference = render(tmp/'reference.y4m', native_torus().shift(m.RIGHT), 1)
            negative = render(tmp/'unmodified.y4m', native_torus(), 1)
            self.assertNotEqual(reference, negative)
            for threads in (1, 4):
                actual = Authored(r1=2., r2=.5, resolution=(5, 4))
                self.assertEqual(render(tmp/f'authored-{threads}.y4m', actual, threads), reference)


def native_sphere(shape=(9, 7), radius=.7, true_normals=True, clockwise=False):
    candidate = m.Surface.__new__(m.Surface)
    m._install_live_state(candidate)
    specs = candidate._build_sphere(m._native_surface_shell_factory, radius,
        (0., 6.283185307179586), (0., 3.141592653589793), shape,
        true_normals, clockwise, 1, .001, .001)
    assert not specs
    candidate.resolution = shape
    candidate.compute_triangle_indices()
    candidate.set_color(m.GREY)
    candidate.set_shading(.3, .2, .4)
    candidate.apply_depth_test()
    return candidate


class SphereLifecycleTests(unittest.TestCase):
    def test_stock_and_forwarding_subclasses_preserve_native_bits(self):
        class Forwarding(m.Sphere):
            def init_data(self):
                super().init_data()
            def init_points(self):
                super().init_points()
            def init_uniforms(self):
                super().init_uniforms()
            def init_colors(self):
                super().init_colors()
        for radius in (0., .001, .7, 2.3, -.7):
            for true_normals in (False, True):
                for clockwise in (False, True):
                    options = dict(radius=radius, true_normals=true_normals, clockwise=clockwise)
                    expected = native_sphere(**options)
                    for cls in (m.Sphere, Forwarding):
                        with self.subTest(radius=radius, true_normals=true_normals,
                                          clockwise=clockwise, cls=cls):
                            actual = cls(resolution=(9, 7), **options)
                            for key in ('point', 'd_normal_point', 'rgba'):
                                self.assertEqual(actual.data[key].tobytes(), expected.data[key].tobytes())

    def test_all_hooks_run_before_constructor_radial_normal_correction(self):
        events = []
        class Authored(m.Sphere):
            def init_data(self):
                events.append(('data', self.radius, self.resolution))
                super().init_data()
            def init_points(self):
                events.append('points')
                super().init_points()
                self.shift(m.RIGHT)
            def init_uniforms(self):
                events.append('uniforms')
                super().init_uniforms()
            def init_colors(self):
                events.append('colors')
                super().init_colors()
                self.set_color(m.BLUE)
                self.data['d_normal_point'][:] = 0.
        actual = Authored(radius=2., resolution=(7, 5), normal_nudge=.25)
        self.assertEqual(events, [('data', 2., (7, 5)), 'points', 'uniforms', 'colors'])
        self.assertEqual(actual.get_color(), m.BLUE)
        expected = native_sphere((7, 5), radius=2.).shift(m.RIGHT)
        np.testing.assert_array_equal(actual.get_points(), expected.get_points())
        # Source arithmetic is fixed-order f64 with a single final narrowing.
        normals = (actual.get_points().astype(np.float64) * 1.125).astype(np.float32)
        np.testing.assert_array_equal(actual.data['d_normal_point'], normals)

    def test_false_true_normals_keeps_authored_normal_points(self):
        class Authored(m.Sphere):
            def init_colors(self):
                super().init_colors()
                self.data['d_normal_point'][:] = self.get_points() + m.OUT
        actual = Authored(resolution=(5, 4), true_normals=False)
        np.testing.assert_array_equal(actual.data['d_normal_point'], actual.get_points() + m.OUT)

    def test_uv_override_and_live_regrid_share_normal_semantics(self):
        visits = []
        class Authored(m.Sphere):
            def uv_func(self, u, v):
                visits.append((u, v))
                return u, v, self.radius
        actual = Authored(radius=2., resolution=(3, 4), normal_nudge=.25)
        self.assertEqual(len(visits), 36)
        np.testing.assert_array_equal(actual.get_points()[:, 2], 2.)
        scene = m.Scene()
        scene.add(actual)
        for rebuild, shape in ((actual.init_points, (3, 4)),
                               (lambda: actual.set_resolution((4, 5)), (4, 5))):
            visits.clear()
            rebuild()
            self.assertEqual(len(visits), 3*shape[0]*shape[1])
            self.assertIs(actual._scene, scene)
            self.assertEqual(m._surface_grid_resolution(actual), shape)
            expected = (actual.get_points().astype(np.float64) * 1.125).astype(np.float32)
            np.testing.assert_array_equal(actual.data['d_normal_point'], expected)

    def test_authored_replacement_keeps_schema_children_and_has_native_topology(self):
        child = m.Circle()
        class Authored(m.Sphere):
            data_dtype = [*m.Surface.data_dtype, ('tag', np.float32, (1,))]
            def init_data(self):
                super().init_data()
                self.add(child)
            def init_points(self):
                self.set_points([[-1, -1, 2], [-1, 1, 2], [1, -1, 2], [1, 1, 2]])
                self.data['tag'][:] = 7.
            def uv_func(self, u, v):
                raise AssertionError('replacement geometry must not invoke UV sampling')
        actual = Authored(radius=2., resolution=(2, 2), normal_nudge=.25)
        self.assertIs(actual.submobjects[0], child)
        self.assertEqual(m._surface_grid_resolution(actual), (2, 2))
        np.testing.assert_array_equal(actual.data['tag'], 7.)
        np.testing.assert_array_equal(actual.data['d_normal_point'], actual.get_points() * 1.125)
        duplicate = copy.deepcopy(actual)
        np.testing.assert_array_equal(duplicate.data, actual.data)
        self.assertIsNot(duplicate.submobjects[0], child)

    def test_constructor_radius_remains_the_normal_correction_argument(self):
        class Authored(m.Sphere):
            def init_data(self):
                super().init_data()
                self.radius = 3.
        actual = Authored(radius=2., resolution=(5, 4), normal_nudge=.5)
        expected = native_sphere((5, 4), radius=3.)
        np.testing.assert_array_equal(actual.get_points(), expected.get_points())
        np.testing.assert_array_equal(actual.data['d_normal_point'], actual.get_points() * 1.25)

    def test_empty_and_strip_initial_grids_keep_the_native_contract(self):
        for shape in ((0, 3), (3, 0), (1, 4), (4, 1)):
            with self.subTest(shape=shape):
                actual = m.Sphere(resolution=shape, radius=.7)
                expected = native_sphere(shape)
                for key in ('point', 'd_normal_point'):
                    self.assertEqual(actual.data[key].tobytes(), expected.data[key].tobytes())

    def test_authored_sphere_reaches_native_render_and_preserves_worker_identity(self):
        class Authored(m.Sphere):
            def init_points(self):
                super().init_points()
                self.shift(m.RIGHT)
        def render(path, surface, threads):
            scene = m.Scene()
            with render_session(scene, path, resolution=(64, 40), fps=24, threads=threads) as session:
                scene.add(surface)
                scene.wait(.125)
            self.assertEqual(session.result.frame_count, 3)
            return path.read_bytes()
        with tempfile.TemporaryDirectory() as tmp:
            tmp = Path(tmp)
            expected = native_sphere().shift(m.RIGHT)
            expected.data['d_normal_point'][:] = (
                expected.get_points().astype(np.float64) * ((.7 + .001)/.7))
            reference = render(tmp/'expected.y4m', expected, 1)
            self.assertNotEqual(reference, render(tmp/'unmodified.y4m', native_sphere(), 1))
            for threads in (1, 4):
                actual = Authored(radius=.7, resolution=(9, 7))
                self.assertEqual(reference, render(tmp/f'actual-{threads}.y4m', actual, threads))


def run_native_solid_lifecycle():
    suite = unittest.TestSuite(unittest.defaultTestLoader.loadTestsFromTestCase(cls)
        for cls in (TorusLifecycleTests, SphereLifecycleTests))
    result = unittest.TextTestRunner(verbosity=2).run(suite)
    if not result.wasSuccessful():
        raise AssertionError('native solid lifecycle acceptance failed')


if __name__ in ('__main__', '<run_path>'):
    run_native_solid_lifecycle()
