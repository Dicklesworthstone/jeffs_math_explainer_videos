"""Axial and flat solid subclass behavior through real native geometry and output."""
from pathlib import Path
import tempfile
import unittest

import numpy as np
import manimlib as m
from fmn_python import render_session


def kernel(kind, *, height=2., radius=1., axis=(0., 0., 1.),
           shape=(7, 5), ur=(0., 6.283185307179586), vr=None):
    """An independent Atlas constructor, without a public Cylinder or Cone."""
    surface = m.Surface.__new__(m.Surface)
    m._install_live_state(surface)
    vr = (0., 1.) if kind == 'cone' and vr is None else ((-1., 1.) if vr is None else vr)
    specs = getattr(surface, '_build_' + kind)(m._native_surface_shell_factory,
        height, radius, axis, ur, vr, shape, 1, .001, .001, 0)
    assert not specs
    surface.resolution = shape
    surface.compute_triangle_indices()
    surface.set_color(m.GREY)
    surface.set_shading(.3, .2, .4)
    surface.apply_depth_test()
    return surface


def render(path, surface, threads):
    scene = m.Scene()
    with render_session(scene, path, resolution=(64, 40), fps=24, threads=threads) as session:
        scene.add(surface)
        scene.wait(.125)
    assert session.result.frame_count == 3
    return path.read_bytes()


class AxialSolidTests(unittest.TestCase):
    def test_cooperative_hooks_run_once_with_live_recipe(self):
        for base in (m.Cylinder, m.Cone, m.Line3D):
            with self.subTest(base=base.__name__):
                events = []
                class Hooks:
                    def init_data(self):
                        events.append(('data', self.height, self.radius, tuple(self.axis)))
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
                class Authored(Hooks, base):
                    pass
                args, kwargs = ((), dict(height=2., radius=.5))
                if base is m.Line3D:
                    args, kwargs = (([2, 0, -1], [2, 0, 1]), dict(width=1.))
                surface = Authored(*args, resolution=(7, 5), color=m.RED, z_index=7, **kwargs)
                self.assertEqual(events, [('data', 2., .5, (0., 0., 2.) if base is m.Line3D
                                           else (0., 0., 1.)), 'points', 'uniforms', 'colors'])
                expected = kernel('cone' if base is m.Cone else 'cylinder', radius=.5)
                expected.shift(m.RIGHT * (3 if base is m.Line3D else 1))
                np.testing.assert_array_equal(surface.get_points(), expected.get_points())
                self.assertEqual(surface.get_color(), m.BLUE)
                self.assertEqual(surface.z_index, 7)
                np.testing.assert_allclose(surface.get_shading(), (.1, .4, .2), atol=1e-7)

    def test_stock_and_forwarding_subclasses_keep_native_geometry(self):
        for base, kind in ((m.Cylinder, 'cylinder'), (m.Cone, 'cone')):
            class Forwarding(base):
                def init_points(self):
                    super().init_points()
            for shape in ((0, 4), (4, 0), (1, 4), (4, 1), (7, 5)):
                for axis in ((0., 0., 1.), (1., 2., 3.), (0., 0., -1.)):
                    for cls in (base, Forwarding):
                        with self.subTest(kind=kind, shape=shape, axis=axis, cls=cls):
                            actual = cls(height=3., radius=.7, axis=axis, resolution=shape)
                            expected = kernel(kind, height=3., radius=.7, axis=axis, shape=shape)
                            for key in ('point', 'd_normal_point', 'rgba'):
                                # Exact numerical equality, including degenerate grids.
                                # Raw signed-zero representation is a separate publication contract.
                                np.testing.assert_array_equal(actual.data[key], expected.data[key])

    def test_authored_uv_gets_radius_height_axis_and_native_normals(self):
        for base in (m.Cylinder, m.Cone):
            calls = []
            def uv(u, v):
                return np.array([u, u*u, v])
            class Authored(base):
                def uv_func(self, u, v):
                    calls.append((u, v))
                    return uv(u, v)
            actual = Authored(height=3., radius=.4, axis=m.RIGHT, resolution=(4, 5))
            self.assertEqual(len(calls), 60)
            expected = m.ParametricSurface(uv, u_range=(0, m.TAU),
                v_range=(0, 1) if base is m.Cone else (-1, 1), resolution=(4, 5))
            expected.scale(.4).set_depth(3., stretch=True).apply_matrix(m.z_to_vector(m.RIGHT))
            for key in ('point', 'd_normal_point'):
                np.testing.assert_array_equal(actual.data[key], expected.data[key])

    def test_placement_overrides_and_existing_children_use_the_actual_receiver(self):
        events = []
        child = m.Point([1, 0, 0])
        class Authored(m.Cylinder):
            def init_data(self):
                super().init_data()
                self.add(child)
            def scale(self, factor, **kwargs):
                events.append(('scale', self, factor))
                return super().scale(factor, **kwargs)
            def set_depth(self, depth, **kwargs):
                events.append(('depth', self, depth))
                return super().set_depth(depth, **kwargs)
            def apply_matrix(self, matrix, **kwargs):
                events.append(('matrix', self))
                return super().apply_matrix(matrix, **kwargs)
        actual = Authored(height=3., radius=.5, axis=m.RIGHT, resolution=(9, 5))
        self.assertEqual([event[0] for event in events], ['scale', 'depth', 'matrix'])
        self.assertTrue(all(event[1] is actual for event in events))
        self.assertIs(actual.submobjects[0], child)
        expected = m.ParametricSurface(lambda u, v: (m.Cylinder.uv_func(actual, u, v)),
            u_range=(0, m.TAU), v_range=(-1, 1), resolution=(9, 5))
        other = m.Point([1, 0, 0])
        expected.add(other)
        expected.scale(.5).set_depth(3., stretch=True).apply_matrix(m.z_to_vector(m.RIGHT))
        np.testing.assert_array_equal(actual.get_points(), expected.get_points())
        np.testing.assert_array_equal(child.get_points(), other.get_points())

    def test_replacement_points_and_custom_records_are_not_overwritten(self):
        for base in (m.Cylinder, m.Cone, m.Line3D):
            child = m.Circle()
            class Replacement(base):
                data_dtype = [*m.Surface.data_dtype, ('tag', np.float32, (1,))]
                def init_data(self):
                    super().init_data()
                    self.add(child)
                def init_points(self):
                    self.set_points([[-1, -1, 0], [-1, 1, 0], [1, -1, 0], [1, 1, 0]])
                    self.data['d_normal_point'][:] = self.get_points() + .001 * m.OUT
                    self.data['tag'][:] = 17
                def uv_func(self, u, v):
                    raise AssertionError('replacement init_points must bypass UV sampling')
            args = ([2, 0, -1], [2, 0, 1]) if base is m.Line3D else ()
            actual = Replacement(*args, resolution=(2, 2))
            self.assertIs(actual.submobjects[0], child)
            self.assertEqual(m._surface_grid_resolution(actual), (2, 2))
            np.testing.assert_array_equal(actual.data['tag'], 17)
            expected = np.array([[-1, -1, 0], [-1, 1, 0], [1, -1, 0], [1, 1, 0]])
            np.testing.assert_array_equal(actual.get_points(), expected + ([2, 0, 0] if args else 0))
            duplicate = actual.copy()
            self.assertIsNot(duplicate.submobjects[0], child)
            np.testing.assert_array_equal(duplicate.data, actual.data)

    def test_line_midpoint_shift_occurs_after_colors_and_honors_override(self):
        events = []
        class Authored(m.Line3D):
            def init_colors(self):
                events.append(('colors', self.get_center().copy()))
                super().init_colors()
            def shift(self, *vectors):
                events.append(('shift', vectors[0].copy()))
                return super().shift(*vectors)
        actual = Authored([2, 0, -1], [2, 0, 1], resolution=(9, 5))
        self.assertEqual([event[0] for event in events], ['colors', 'shift'])
        np.testing.assert_allclose(events[0][1], [0, 0, 0], atol=1e-6)
        np.testing.assert_array_equal(events[1][1], [2, 0, 0])
        np.testing.assert_allclose(actual.get_center(), [2, 0, 0], atol=1e-6)

    def test_live_rebuild_and_regrid_keep_identity_and_shape_parameters(self):
        for base in (m.Cylinder, m.Cone):
            class Authored(base):
                def uv_func(self, u, v):
                    return (u, v, u + v)
            actual = Authored(height=2., radius=.5, resolution=(4, 5))
            scene = m.Scene()
            scene.add(actual)
            for operation, shape in ((actual.init_points, (4, 5)),
                                     (lambda: actual.set_resolution((5, 6)), (5, 6))):
                actual.height, actual.radius = 3., .75
                actual.axis = m.RIGHT.copy()
                operation()
                expected = Authored(height=3., radius=.75, axis=m.RIGHT, resolution=shape)
                self.assertIs(actual._scene, scene)
                self.assertIs(scene.mobjects[0], actual)
                self.assertEqual(m._surface_grid_resolution(actual), shape)
                np.testing.assert_allclose(actual.get_points(), expected.get_points(), atol=1e-6)

    def test_callback_recipe_mutation_refuses_before_publication(self):
        retained = []
        class Mutating(m.Cylinder):
            def init_data(self):
                super().init_data()
                retained.append(self)
            def uv_func(self, u, v):
                self.axis[0] += 1
                return u, v, u + v
        with self.assertRaisesRegex(RuntimeError, 'shape parameters changed'):
            Mutating(resolution=(3, 4))
        self.assertEqual(retained[0].n_records(), 0)
        actual = m.Cone(resolution=(4, 5))
        before = actual.data.copy()
        actual.uv_func = lambda u, v: (setattr(actual, 'radius', actual.radius + 1) or (u, v, u+v))
        with self.assertRaisesRegex(RuntimeError, 'shape parameters changed'):
            actual.set_resolution((5, 6))
        np.testing.assert_array_equal(actual.data, before)
        self.assertEqual(actual.resolution, (4, 5))

    def test_callback_exception_identity_and_first_failure_stop(self):
        error, calls = ValueError('authored axial surface failed'), []
        class Broken(m.Cone):
            def uv_func(self, u, v):
                calls.append((u, v))
                if len(calls) == 3:
                    raise error
                return u, v, 0
        with self.assertRaises(ValueError) as caught:
            Broken(resolution=(3, 4))
        self.assertIs(caught.exception, error)
        self.assertEqual(len(calls), 3)
        self.assertEqual(m.Cone(resolution=(2, 2)).n_records(), 4)

    def test_invalid_recipe_refuses_before_authored_hooks(self):
        events = []
        class Authored(m.Cylinder):
            def init_data(self):
                events.append('data')
                super().init_data()
        for kwargs in (dict(height=float('inf')), dict(radius=float('nan')),
                       dict(axis=(1, 2)), dict(axis=(0, 0, float('inf'))),
                       dict(resolution=(1000, 1000)), dict(epsilon=0)):
            with self.subTest(kwargs=kwargs), self.assertRaises((ValueError, TypeError)):
                Authored(**kwargs)
        self.assertEqual(events, [])

    def test_bound_reinitialization_and_recursive_initialization_refuse(self):
        actual = m.Cylinder(resolution=(3, 4))
        scene = m.Scene()
        scene.add(actual)
        before = actual.data.copy()
        with self.assertRaisesRegex(RuntimeError, 'detached'):
            actual.__init__(radius=7)
        np.testing.assert_array_equal(actual.data, before)
        class Recursive(m.Cone):
            def init_points(self):
                self.__init__()
        with self.assertRaisesRegex(RuntimeError, 'already in progress'):
            Recursive(resolution=(3, 4))

    def test_authored_geometry_reaches_renderer_with_worker_independence(self):
        for base, kind in ((m.Cylinder, 'cylinder'), (m.Cone, 'cone')):
            class Authored(base):
                def init_points(self):
                    super().init_points()
                    self.shift(2 * m.RIGHT)
            with tempfile.TemporaryDirectory() as directory:
                directory = Path(directory)
                expected = render(directory/'expected.y4m', kernel(kind, radius=.5).shift(2*m.RIGHT), 1)
                negative = render(directory/'negative.y4m', kernel(kind, radius=.5), 1)
                self.assertNotEqual(expected, negative)
                for workers in (1, 4, 16):
                    actual = Authored(radius=.5, resolution=(7, 5))
                    self.assertEqual(render(directory/f'actual-{workers}.y4m', actual, workers), expected)


def flat_kernel(kind, size=1., shape=(9, 7)):
    surface = m.Surface.__new__(m.Surface)
    m._install_live_state(surface)
    ur, vr = ((0., 1.), (0., m.TAU)) if kind == 'disk3d' else ((-1., 1.), (-1., 1.))
    specs = getattr(surface, '_build_' + kind)(m._native_surface_shell_factory,
        size, ur, vr, shape, 1, .001, .001, 0)
    assert not specs
    surface.resolution = shape
    surface.compute_triangle_indices()
    surface.set_color(m.GREY)
    surface.set_shading(.3, .2, .4)
    surface.apply_depth_test()
    return surface


class FlatSolidTests(unittest.TestCase):
    def test_untouched_sphere_normals_are_not_double_rounded_after_publication(self):
        for radius in (.7, 2.3, -.7):
            for clockwise in (False, True):
                expected = m.Surface.__new__(m.Surface)
                m._install_live_state(expected)
                expected._build_sphere(m._native_surface_shell_factory, radius,
                    (0., m.TAU), (0., m.PI), (9, 7), True, clockwise, 1, .001, .001)
                actual = m.Sphere(radius=radius, clockwise=clockwise, resolution=(9, 7))
                # Native world-space publication canonicalizes signed zero.
                # Every nonzero normal coordinate must remain EXACTLY the
                # f64-built native value, not a second calculation from f32.
                np.testing.assert_array_equal(actual.data['d_normal_point'],
                                              expected.data['d_normal_point'])

    def test_stock_sizes_and_degenerate_grids_preserve_native_geometry(self):
        for base, kind, key in ((m.Disk3D, 'disk3d', 'radius'),
                                (m.Square3D, 'square3d', 'side_length')):
            for size in (-.7, 0., .7, 3.):
                for shape in ((0, 4), (4, 0), (1, 4), (4, 1), (9, 7)):
                    with self.subTest(kind=kind, size=size, shape=shape):
                        actual = base(**{key: size}, resolution=shape)
                        expected = flat_kernel(kind, size, shape)
                        for field in ('point', 'd_normal_point', 'rgba'):
                            np.testing.assert_array_equal(actual.data[field], expected.data[field])

    def test_all_hooks_see_unit_geometry_before_the_final_scale(self):
        for base, kind, key, unit in ((m.Disk3D, 'disk3d', 'radius', 1.),
                                     (m.Square3D, 'square3d', 'side_length', 2.)):
            events = []
            class Authored(base):
                def init_data(self):
                    events.append('data')
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
                    self.before_scale = self.get_points().copy()
                def scale(self, factor, **kwargs):
                    events.append(('scale', factor))
                    return super().scale(factor, **kwargs)
            actual = Authored(**{key: 3*unit}, resolution=(9, 7), color=m.RED)
            expected = flat_kernel(kind, unit).shift(m.RIGHT)
            np.testing.assert_array_equal(actual.before_scale, expected.get_points())
            expected.scale(3)
            np.testing.assert_array_equal(actual.get_points(), expected.get_points())
            self.assertEqual(events, ['data', 'points', 'uniforms', 'colors', ('scale', 3)])
            self.assertEqual(actual.get_color(), m.BLUE)

    def test_replacement_geometry_children_and_custom_columns_survive_scaling(self):
        points = np.array([[-1, -1, 2], [-1, 1, 2], [1, -1, 2], [1, 1, 2]])
        for base, kwargs in ((m.Disk3D, dict(radius=2)), (m.Square3D, dict(side_length=4))):
            child = m.Point([1, 2, 3])
            class Replacement(base):
                data_dtype = [*m.Surface.data_dtype, ('temperature', np.float32, (1,))]
                def init_points(self):
                    self.set_points(points)
                    self.data['d_normal_point'][:] = points + .001*m.OUT
                    self.data['temperature'][:] = 23
                def init_colors(self):
                    super().init_colors()
                    self.add(child)
                def uv_func(self, u, v):
                    raise AssertionError('replacement points must not invoke the sampler')
            actual = Replacement(resolution=(2, 2), **kwargs)
            expected = m.ParametricSurface(lambda u, v: (u, v, 2),
                u_range=(-1, 1), v_range=(-1, 1), resolution=(2, 2))
            other = m.Point([1, 2, 3])
            expected.add(other).scale(2)
            self.assertIs(actual.submobjects[0], child)
            np.testing.assert_array_equal(actual.get_points(), expected.get_points())
            np.testing.assert_array_equal(child.get_points(), other.get_points())
            np.testing.assert_array_equal(actual.data['temperature'], 23)
            duplicate = actual.copy()
            self.assertIsNot(duplicate.submobjects[0], child)
            np.testing.assert_array_equal(duplicate.data, actual.data)

    def test_authored_uv_and_live_resolution_changes_keep_dimensions(self):
        for base, kwargs, domains in (
                (m.Disk3D, dict(radius=3), ((0, 1), (0, m.TAU))),
                (m.Square3D, dict(side_length=6), ((-1, 1), (-1, 1)))):
            calls = []
            def uv(u, v):
                return u, v, u + v
            class Authored(base):
                def uv_func(self, u, v):
                    calls.append((u, v))
                    return uv(u, v)
            actual = Authored(resolution=(3, 4), **kwargs)
            scene = m.Scene()
            scene.add(actual)
            for operation, shape in ((lambda: None, (3, 4)), (actual.init_points, (3, 4)),
                                     (lambda: actual.set_resolution((4, 5)), (4, 5))):
                operation()
                self.assertIs(scene.mobjects[0], actual)
                self.assertEqual(len(calls), 3*shape[0]*shape[1])
                calls.clear()
                expected = m.ParametricSurface(uv, u_range=domains[0], v_range=domains[1],
                                               resolution=shape).scale(3)
                np.testing.assert_allclose(actual.get_points(), expected.get_points(), atol=1e-6)
                self.assertEqual(m._surface_grid_resolution(actual), shape)
                self.assertIs(actual._scene, scene)

    def test_constructor_scale_argument_is_not_replaced_by_an_authored_attribute(self):
        class Authored(m.Disk3D):
            def init_data(self):
                super().init_data()
                self.radius = 7
        actual = Authored(radius=3, resolution=(9, 7))
        expected = flat_kernel('disk3d').scale(3)
        np.testing.assert_array_equal(actual.get_points(), expected.get_points())
        self.assertEqual(actual.radius, 7)

    def test_bad_size_and_recursive_hooks_refuse_without_partial_construction(self):
        calls = []
        class Authored(m.Square3D):
            def init_data(self):
                calls.append('data')
                super().init_data()
        for size in (float('nan'), float('inf'), 1e40):
            with self.assertRaises(ValueError):
                Authored(side_length=size)
        self.assertEqual(calls, [])
        class Recursive(m.Disk3D):
            def init_colors(self):
                self.__init__()
        with self.assertRaisesRegex(RuntimeError, 'already in progress'):
            Recursive(resolution=(2, 2))

    def test_changed_size_during_sampling_refuses_before_publication(self):
        roots = []
        class Mutating(m.Square3D):
            def init_data(self):
                super().init_data()
                roots.append(self)
            def uv_func(self, u, v):
                self.side_length += 1
                return u, v, 0
        with self.assertRaisesRegex(RuntimeError, 'shape parameters changed'):
            Mutating(resolution=(3, 4))
        self.assertEqual(roots[0].n_records(), 0)
        self.assertEqual(m.Square3D(resolution=(2, 2)).n_records(), 4)

    def test_authored_flat_geometry_is_visible_and_worker_independent(self):
        for base, kind, key, unit in ((m.Disk3D, 'disk3d', 'radius', 1.),
                                     (m.Square3D, 'square3d', 'side_length', 2.)):
            class Authored(base):
                def init_points(self):
                    super().init_points()
                    self.shift(m.RIGHT)
            with tempfile.TemporaryDirectory() as directory:
                directory = Path(directory)
                expected = flat_kernel(kind, unit).shift(m.RIGHT).scale(1.5)
                reference = render(directory/'reference.y4m', expected, 1)
                self.assertNotEqual(reference, render(directory/'unmodified.y4m',
                    flat_kernel(kind, 1.5*unit), 1))
                for workers in (1, 4, 16):
                    actual = Authored(**{key: 1.5*unit}, resolution=(9, 7))
                    self.assertEqual(reference, render(directory/f'actual-{workers}.y4m', actual, workers))


def run_native_axial_solids():
    suite = unittest.TestSuite(unittest.defaultTestLoader.loadTestsFromTestCase(cls)
        for cls in (AxialSolidTests, FlatSolidTests))
    result = unittest.TextTestRunner(verbosity=2).run(suite)
    if not result.wasSuccessful():
        raise AssertionError('native axial-solid acceptance failed')


if __name__ in ('__main__', '<run_path>'):
    run_native_axial_solids()
