"""Real native surface admission; no replacement sampler or renderer."""
import inspect
from pathlib import Path
import tempfile
import unittest

import numpy as np
import manimlib as m
from fmn_python.surface_admission import grid_shape, sampling_options, install_surface_admission


class SurfaceAdmissionTests(unittest.TestCase):
    def test_exception_is_preserved_and_callback_stops_at_each_probe(self):
        for stop in (1, 2, 3, 7, 12):
            for error_type in (RuntimeError, KeyboardInterrupt, SystemExit):
                with self.subTest(stop=stop, error_type=error_type):
                    calls = []
                    error = error_type('original failure')
                    def sample(u, v):
                        calls.append((u, v))
                        if len(calls) == stop:
                            raise error
                        return u, v, u * v
                    with self.assertRaises(error_type) as caught:
                        m.ParametricSurface(sample, resolution=(2, 2))
                    self.assertIs(caught.exception, error)
                    self.assertEqual(len(calls), stop)

    def test_subclass_uv_failures_are_also_first_error(self):
        calls = []
        class Authored(m.Surface):
            def uv_func(self, u, v):
                calls.append((u, v))
                raise LookupError('authored')
        with self.assertRaisesRegex(LookupError, 'authored'):
            Authored(resolution=(3, 4))
        self.assertEqual(len(calls), 1)

    def test_conversion_failure_is_not_repeated(self):
        calls = []
        error = OSError('conversion failure')
        class Result:
            def __array__(self, *args, **kwargs):
                calls.append('convert')
                raise error
        def sample(u, v):
            calls.append('callback')
            return Result()
        with self.assertRaises(OSError) as caught:
            m.ParametricSurface(sample, resolution=(2, 2))
        self.assertIs(caught.exception, error)
        self.assertEqual(calls, ['callback', 'convert'])

    def test_exactly_three_finite_real_coordinates(self):
        for result in ((0, 0), (0, 0, 0, 0), ((0, 0, 0),),
                       (0j, 0j, 0j), ('0', '0', '0'),
                       (float('nan'), 0, 0), (float('inf'), 0, 0), (1e100, 0, 0)):
            with self.subTest(result=result):
                calls = []
                def sample(u, v):
                    calls.append((u, v))
                    return result
                with self.assertRaises(ValueError):
                    m.ParametricSurface(sample, resolution=(2, 2))
                self.assertEqual(len(calls), 1)

    def test_valid_probe_order_and_callable_identity_are_unchanged(self):
        calls = []
        def sample(u, v):
            calls.append((u, v))
            return np.array((u, v, u * v))
        obj = m.ParametricSurface(sample, resolution=(2, 2), epsilon=.125)
        self.assertIs(obj.passed_uv_func, sample)
        self.assertEqual(calls, [(u + du, v + dv) for u in (0., 1.) for v in (0., 1.)
                                 for du, dv in ((0, 0), (.125, 0), (0, .125))])
        np.testing.assert_array_equal(obj.get_points(), [(0, 0, 0), (0, 1, 0), (1, 0, 0), (1, 1, 1)])

    def test_invalid_controls_do_not_evaluate_user_code(self):
        for options in ({'epsilon': 0}, {'epsilon': -1}, {'epsilon': float('nan')},
                        {'normal_nudge': -1}, {'normal_nudge': float('inf')},
                        {'normal_nudge': 1e100}, {'u_range': (0, float('inf'))},
                        {'u_range': (-1e308, 1e308)},
                        {'u_range': (1e308, 1e308), 'epsilon': 1e308},
                        {'preferred_creation_axis': 2}, {'preferred_creation_axis': .5}):
            with self.subTest(options=options):
                calls = []
                with self.assertRaises((ValueError, TypeError)):
                    m.ParametricSurface(lambda u, v: calls.append((u, v)) or (u, v, 0),
                                        resolution=(2, 2), **options)
                self.assertEqual(calls, [])

    def test_reversed_and_collapsed_finite_domains_remain_supported(self):
        obj = m.ParametricSurface(lambda u, v: (u, v, 0), resolution=(2, 3),
                                  u_range=(2, -1), v_range=(4, 4))
        np.testing.assert_array_equal(obj.get_points(), [(u, 4, 0) for u in (2, -1) for _ in range(3)])

    def test_empty_and_strip_grids_remain_supported(self):
        for shape in ((0, 0), (0, 5), (5, 0), (1, 1), (1, 4), (4, 1)):
            with self.subTest(shape=shape):
                calls = []
                obj = m.ParametricSurface(lambda u, v: calls.append((u, v)) or (u, v, 0), resolution=shape)
                self.assertEqual(obj.get_num_points(), shape[0] * shape[1])
                self.assertEqual(len(calls), 3 * shape[0] * shape[1])
                self.assertEqual(len(obj.compute_triangle_indices()), 0)
        group = m.SGroup(m.Sphere(resolution=(3, 3)))
        self.assertEqual(len(group), 1)
        self.assertEqual(group.get_num_points(), 0)

    def test_all_public_stock_grids_are_bounded_before_allocation(self):
        constructors = [m.Surface, m.Sphere, m.Torus, m.Cylinder, m.Cone,
                        m.Disk3D, m.Square3D,
                        lambda **kwargs: m.Line3D((0, 0, 0), (1, 1, 1), **kwargs)]
        for constructor in constructors:
            for shape in ((2**62, 2), (0, 2**62), (513, 512), (-1, 2), (2.5, 2)):
                with self.subTest(constructor=constructor, shape=shape):
                    with self.assertRaises((ValueError, TypeError)):
                        constructor(resolution=shape)

    def test_cube_budget_is_for_all_six_faces(self):
        # 6 * 43690 <= 262144 < 6 * 43691.
        self.assertEqual(grid_shape((1, 43690), copies=6), (1, 43690))
        with self.assertRaises(ValueError):
            grid_shape((1, 43691), copies=6)
        with self.assertRaisesRegex(ValueError, 'aggregate'):
            m.Cube(square_resolution=(209, 210))
        cube = m.Cube(square_resolution=(3, 4))
        self.assertEqual([face.get_num_points() for face in cube], [12] * 6)

    def test_grid_budget_boundary_is_exact(self):
        # Atlas's 262144-point UV budget (fm-1rb8): 301x301 is admitted.
        self.assertEqual(grid_shape((301, 301)), (301, 301))
        self.assertEqual(grid_shape((512, 512)), (512, 512))
        self.assertEqual(grid_shape((0, 262144)), (0, 262144))
        for shape in ((512, 513), (0, 262145), (1000, 1000)):
            with self.assertRaises(ValueError):
                grid_shape(shape)

    def test_reference_dense_grid_constructs_aligns_meshes_and_renders(self):
        # _2025/laplace get_complex_graph authors resolution=(301, 301).
        domain = dict(u_range=(-2, 2), v_range=(-2, 2))
        dense = m.ParametricSurface(
            lambda u, v: (u, v, np.sin(3 * u) * np.cos(3 * v) / 4), resolution=(301, 301), **domain)
        self.assertEqual((dense.resolution, dense.get_num_points()), ((301, 301), 301 * 301))
        mesh = m.SurfaceMesh(dense, resolution=(21, 11))
        self.assertEqual(len(mesh), 32)
        coarse = m.ParametricSurface(lambda u, v: (u, v, 0), resolution=(101, 101), **domain)

        class Morph(m.Scene):
            def construct(self):
                self.camera.frame.set_euler_angles(phi=.7)
                self.add(coarse, mesh)
                self.play(m.Transform(coarse, dense), run_time=.5, rate_func=m.linear)
        with tempfile.TemporaryDirectory(prefix='fmn-dense-surface-') as directory:
            path = Path(directory) / 'morph.y4m'
            result = Morph().render(path, format='y4m', resolution=(96, 54), fps=8)
            self.assertEqual(result.frame_count, 4)
            payload = path.read_bytes().split(b'\n', 1)[1]
        frame_size = 96 * 54 * 3 // 2
        frames = [payload[i + 6:i + 6 + frame_size] for i in range(0, len(payload), frame_size + 6)]
        self.assertNotEqual(frames[0], frames[-1])
        self.assertEqual((coarse.resolution, coarse.get_num_points()), ((301, 301), 301 * 301))
        np.testing.assert_allclose(coarse.get_points(), dense.get_points(), atol=1e-5)

    def test_shape_iterables_are_read_only_to_the_admission_bound(self):
        reads = []
        def endless():
            while True:
                reads.append(1)
                yield 2
        with self.assertRaises(ValueError):
            m.Sphere(resolution=endless())
        self.assertEqual(len(reads), 3)

    def test_authored_dimension_conversion_is_not_truncated_or_repeated(self):
        calls = []
        class Dimension:
            def __index__(self):
                calls.append(1)
                return 2
        obj = m.Sphere(resolution=(Dimension(), Dimension()))
        self.assertEqual(obj.get_num_points(), 4)
        self.assertEqual(len(calls), 2)
        with self.assertRaises(TypeError):
            m.Sphere(resolution=(2.5, 2))

    def test_nonfinite_stock_dimensions_are_refused(self):
        requests = [(m.Sphere, {'radius': float('nan')}),
                    (m.Torus, {'r1': float('inf')}),
                    (m.Cylinder, {'height': 1e100}),
                    (m.Cone, {'radius': float('-inf')}),
                    (m.Disk3D, {'radius': float('nan')}),
                    (m.Square3D, {'side_length': float('inf')}),
                    (m.Cube, {'side_length': float('nan')}),
                    (m.Prism, {'width': float('inf')})]
        for cls, options in requests:
            with self.subTest(cls=cls, options=options):
                with self.assertRaises(ValueError):
                    cls(**options)

    def test_line_endpoints_and_axes_are_admitted(self):
        with self.assertRaises(ValueError):
            m.Line3D((0, 0, 0), (float('nan'), 0, 0))
        with self.assertRaises(ValueError):
            m.Cylinder(axis=(0, float('inf'), 1))
        line = m.Line3D((1, 2, 3), (2, 3, 4), resolution=(3, 4))
        self.assertTrue(np.isfinite(line.get_points()).all())

    def test_scene_bound_reconstruction_refuses_before_resetting_state(self):
        for cls, args in ((m.Surface, ()), (m.Sphere, ()), (m.Cylinder, ()), (m.Cube, ()),
                          (m.ParametricSurface, (lambda u, v: (u, v, 0),))):
            with self.subTest(cls=cls):
                obj = cls(*args)
                scene = m.Scene()
                scene.add(obj)
                before = obj.data.copy()
                children = tuple(obj.submobjects)
                with self.assertRaisesRegex(RuntimeError, 'detached target'):
                    cls.__init__(obj, *args)
                self.assertTrue(obj._is_bound())
                self.assertIs(obj._scene, scene)
                self.assertEqual(tuple(obj.submobjects), children)
                np.testing.assert_array_equal(obj.data, before)

    def test_direct_private_grid_seam_is_also_bounded(self):
        obj = m.Surface(resolution=(2, 2))
        before = obj.data.copy()
        calls = []
        with self.assertRaises(ValueError):
            obj._build_parametric_surface(m._native_surface_shell_factory,
                lambda u, v: calls.append((u, v)) or (u, v, 0), (0, 1), (0, 1), (2**62, 2), .001, .001)
        self.assertEqual(calls, [])
        np.testing.assert_array_equal(obj.data, before)

    def test_bound_private_build_refuses_before_authored_callbacks(self):
        obj = m.Surface(resolution=(2, 2))
        scene = m.Scene()
        scene.add(obj)
        before = obj.data.copy()
        calls = []
        with self.assertRaisesRegex(RuntimeError, 'detached target'):
            obj._build_parametric_surface(m._native_surface_shell_factory,
                lambda u, v: calls.append((u, v)) or (u, v, 0),
                (0, 1), (0, 1), (2, 2), .001, .001)
        self.assertEqual(calls, [])
        self.assertIs(obj._scene, scene)
        np.testing.assert_array_equal(obj.data, before)

    def test_failed_private_build_does_not_publish_replacement_geometry(self):
        obj = m.Surface(resolution=(2, 2)).shift((2, 3, 4))
        before = obj.data.copy()
        calls = []
        error = RuntimeError('stopped')
        def sample(u, v):
            calls.append((u, v))
            if len(calls) == 5:
                raise error
            return u, v, 0
        with self.assertRaises(RuntimeError) as caught:
            obj._build_parametric_surface(m._native_surface_shell_factory, sample,
                                         (0, 1), (0, 1), (3, 4), .001, .001)
        self.assertIs(caught.exception, error)
        self.assertEqual(len(calls), 5)
        np.testing.assert_array_equal(obj.data, before)

    def test_recursive_construction_has_independent_failure_state(self):
        calls = []
        def sample(u, v):
            calls.append(1)
            try:
                m.ParametricSurface(lambda x, y: (_ for _ in ()).throw(ValueError('inner')), resolution=(1, 1))
            except ValueError:
                pass
            return u, v, 0
        obj = m.ParametricSurface(sample, resolution=(2, 2))
        self.assertEqual(len(calls), 12)
        self.assertEqual(obj.get_num_points(), 4)

    def test_regeneration_and_plot_recipes_use_the_same_guard(self):
        axes = m.ThreeDAxes()
        obj = axes.get_parametric_surface(lambda u, v: (u, v, u*v), resolution=(2, 2))
        before = obj.data.copy()
        calls = []
        error = RuntimeError('live recipe')
        def fail(u, v):
            calls.append((u, v))
            raise error
        obj.passed_uv_func = fail
        with self.assertRaises(RuntimeError) as caught:
            obj.init_points()
        self.assertIs(caught.exception, error)
        self.assertEqual(len(calls), 1)
        np.testing.assert_array_equal(obj.data, before)

    def test_installer_is_idempotent_and_signatures_are_preserved(self):
        method = m.Mobject._build_parametric_surface
        before = inspect.signature(method)
        install_surface_admission(m._FMN_ROOT if hasattr(m, '_FMN_ROOT') else m)
        self.assertIs(m.Mobject._build_parametric_surface, method)
        self.assertEqual(inspect.signature(method), before)
        self.assertEqual(tuple(before.parameters), ('self', 'factory', 'uv_func', 'u_range', 'v_range',
                                                   'resolution', 'epsilon', 'normal_nudge'))

    def test_successful_geometry_and_pixels_match_unguarded_native_builder(self):
        def formula(u, v):
            return u, v, u*v/4
        obj = m.ParametricSurface(formula, resolution=(4, 5), u_range=(-1, 1), v_range=(-1, 1))
        expected = m.Surface.__new__(m.Surface)
        m._install_live_state(expected)
        raw = m._FMN_ADMITTED_ORIGINAL_build_parametric_surface
        specs = raw(expected, m._native_surface_shell_factory, formula, (-1, 1), (-1, 1), (4, 5), .001, .001)
        m._hang_native_children(expected, specs)
        for field in ('point', 'd_normal_point', 'rgba'):
            np.testing.assert_array_equal(obj.data[field], expected.data[field])
        results = []
        with tempfile.TemporaryDirectory(prefix='fmn-admitted-') as directory:
            for index, source in enumerate((obj, expected)):
                for threads in (1, 4):
                    class Render(m.Scene):
                        def construct(self):
                            self.add(source.copy())
                    scene = Render()
                    path = Path(directory) / f'{index}-{threads}.png'
                    scene.render(path, format='png', resolution=(64, 36), threads=threads)
                    results.append(path.read_bytes())
        self.assertEqual(len(set(results)), 1)


def run_surface_admission_acceptance():
    result = unittest.TextTestRunner(verbosity=2).run(
        unittest.defaultTestLoader.loadTestsFromTestCase(SurfaceAdmissionTests))
    if not result.wasSuccessful():
        raise AssertionError('native surface admission acceptance failed')


if __name__ in ('__main__', '<run_path>'):
    run_surface_admission_acceptance()
