"""Surface subclasses must construct, regenerate and render as native UV grids."""
import copy
from pathlib import Path
import tempfile
import unittest

import numpy as np
import manimlib as m
from fmn_python import render_session


def native_candidate(function, shape=(3, 4), **kwargs):
    """Independent Atlas entry point: never use a public Surface constructor."""
    candidate = m.Surface.__new__(m.Surface)
    m._install_live_state(candidate)
    specs = candidate._build_parametric_surface(
        m._native_surface_shell_factory, function, kwargs.get('u_range', (0., 1.)),
        kwargs.get('v_range', (0., 1.)), shape, .001, .001)
    assert not specs
    return candidate


class SurfaceLifecycleTests(unittest.TestCase):
    def test_all_hooks_follow_cooperative_mro_once(self):
        for base in (m.Surface, m.ParametricSurface):
            with self.subTest(base=base.__name__):
                events = []
                class Hooks:
                    def init_data(self):
                        events.append('data')
                        self.assert_recipe = self.resolution
                        super().init_data()
                    def init_points(self):
                        events.append('points')
                        super().init_points()
                        self.shift(m.RIGHT)
                    def init_uniforms(self):
                        events.append('uniforms')
                        super().init_uniforms()
                        self.set_shading(.1, .2, .3)
                    def init_colors(self):
                        events.append('colors')
                        super().init_colors()
                        self.set_color(m.BLUE)
                class Authored(Hooks, base):
                    pass
                args = (lambda u, v: (u, v, u*v),) if base is m.ParametricSurface else ()
                surface = Authored(*args, resolution=(3, 4), color=m.RED)
                self.assertEqual(events, ['data', 'points', 'uniforms', 'colors'])
                self.assertEqual(surface.assert_recipe, (3, 4))
                self.assertEqual(m._surface_grid_resolution(surface), (3, 4))
                expected = native_candidate(lambda u, v: (u, v, u*v if args else 0))
                expected.shift(m.RIGHT)
                np.testing.assert_allclose(surface.get_points(), expected.get_points(), atol=1e-7)
                np.testing.assert_allclose(surface.get_unit_normals(), expected.get_unit_normals(), atol=1e-5)
                self.assertEqual(surface.get_color(), m.BLUE)
                np.testing.assert_allclose(surface.get_shading(), (.1, .2, .3), atol=1e-7)

    def test_uv_override_on_parametric_surface_is_not_bypassed(self):
        calls = []
        class Authored(m.ParametricSurface):
            def uv_func(self, u, v):
                calls.append((u, v))
                return super().uv_func(u, v) + np.array([0, 0, 2])
        surface = Authored(lambda u, v: np.array([u, v, 0]), resolution=(3, 4))
        self.assertEqual(len(calls), 36)
        np.testing.assert_array_equal(surface.get_points()[:, 2], 2.)
        calls.clear()
        surface.init_points()
        self.assertEqual(len(calls), 36)
        np.testing.assert_array_equal(surface.get_points()[:, 2], 2.)

    def test_replaced_point_hook_does_not_invoke_the_uv_function(self):
        calls = []
        class Authored(m.ParametricSurface):
            def init_points(self):
                self.set_points([[-1, -1, 0], [-1, 1, 0], [1, -1, 0], [1, 1, 0]])
                self.data['d_normal_point'][:] = self.get_points() + .001*m.OUT
        surface = Authored(lambda u, v: calls.append((u, v)), resolution=(2, 2))
        self.assertEqual(calls, [])
        self.assertEqual(m._surface_grid_resolution(surface), (2, 2))
        np.testing.assert_array_equal(surface.get_triangle_indices(), [0, 2, 1, 1, 2, 3])
        np.testing.assert_allclose(surface.get_unit_normals(), [[0, 0, 1]]*4)

    def test_custom_schema_children_and_live_identity_survive(self):
        class Authored(m.ParametricSurface):
            data_dtype = [*m.Surface.data_dtype, ('tag', np.float32, (1,))]
            def init_data(self):
                super().init_data()
                self.resize(1)
                self.data['tag'][:] = 7
                self.child = m.Dot()
                self.add(self.child)
        surface = Authored(lambda u, v: (u, v, 0), resolution=(3, 4), color=m.BLUE)
        np.testing.assert_array_equal(surface.data['tag'], [[7]]*12)
        child = surface.child
        scene = m.Scene()
        scene.add(surface)
        surface.passed_uv_func = lambda u, v: (u, v, 2)
        surface.init_points()
        self.assertIs(surface._scene, scene)
        self.assertIs(surface.submobjects[0], child)
        np.testing.assert_array_equal(surface.get_points()[:, 2], 2)
        np.testing.assert_array_equal(surface.data['tag'], [[7]]*12)
        surface.set_resolution((4, 5))
        self.assertIs(surface._scene, scene)
        self.assertIs(surface.submobjects[0], child)
        np.testing.assert_array_equal(surface.data['tag'], [[7]]*20)
        for duplicate in (surface.copy(), copy.deepcopy(surface)):
            self.assertIsInstance(duplicate, Authored)
            self.assertIsNot(duplicate.submobjects[0], child)
            np.testing.assert_array_equal(duplicate.data, surface.data)

    def test_empty_strip_and_reversed_domain_native_records_are_unchanged(self):
        for shape in ((0, 0), (0, 5), (5, 0), (1, 1), (1, 4), (4, 1), (3, 4)):
            for domain in ((0, 1), (1, -1), (2, 2)):
                with self.subTest(shape=shape, domain=domain):
                    function = lambda u, v: (u, v, u*v)
                    surface = m.ParametricSurface(function, resolution=shape, u_range=domain)
                    expected = native_candidate(function, shape, u_range=domain)
                    np.testing.assert_array_equal(surface.data, expected.data)
                    self.assertEqual(dict(surface.uniforms), dict(expected.uniforms))
        group = m.SGroup(m.Sphere(resolution=(3, 4)))
        self.assertEqual(group.n_records(), 0)
        self.assertEqual(len(group.submobjects), 1)

    def test_empty_containers_do_not_gain_a_fake_grid(self):
        for factory in (m.Surface, m.SGroup):
            with self.subTest(factory=factory.__name__):
                left = factory(**({'resolution': (0, 0)} if factory is m.Surface else {}))
                right = left.copy()
                self.assertIsNone(m._surface_grid_resolution(left))
                left.become(right)
                scene = m.Scene()
                scene.add(left)
                scene.play(m.Transform(left, right), run_time=.1)
                self.assertEqual(left.n_records(), 0)
                self.assertIsNone(m._surface_grid_resolution(left))
        first = m.ParametricSurface(lambda u, v: (u, v, 0), resolution=(2, 3))
        second = m.ParametricSurface(lambda u, v: (u, v, 1), resolution=(3, 4))
        left, right = m.SGroup(first), m.SGroup(second)
        scene = m.Scene()
        scene.add(left)
        scene.play(m.Transform(left, right), run_time=.1)
        self.assertIsNone(m._surface_grid_resolution(left))
        self.assertEqual(m._surface_grid_resolution(left[0]), (3, 4))
        np.testing.assert_array_equal(left[0].get_points()[:, 2], 1)

    def test_callback_error_preserves_identity_and_stops_execution(self):
        for kind in (RuntimeError, KeyboardInterrupt, SystemExit):
            error, calls, roots = kind('surface sampling failed'), [], []
            class Authored(m.ParametricSurface):
                def init_data(self):
                    super().init_data()
                    roots.append(self)
                    self.resize(1)
                    self.data['point'][:] = [8, 9, 10]
            def function(u, v):
                calls.append((u, v))
                if len(calls) == 4:
                    raise error
                return u, v, 0
            with self.assertRaises(kind) as caught:
                Authored(function, resolution=(3, 4))
            self.assertIs(caught.exception, error)
            self.assertEqual(len(calls), 4)
            np.testing.assert_array_equal(roots[0].get_points(), [[8, 9, 10]])
            # Failure cannot leave a global constructor or sampling lock.
            self.assertEqual(m.Surface(resolution=(2, 2)).n_records(), 4)

    def test_callback_mutation_is_not_overwritten(self):
        for mutation in ('records', 'schema', 'bind', 'recipe', 'family', 'lock'):
            with self.subTest(mutation=mutation):
                roots, scene = [], m.Scene()
                class Authored(m.ParametricSurface):
                    def init_data(self):
                        super().init_data()
                        roots.append(self)
                        self.resize(1)
                changed = [False]
                def function(u, v):
                    target = roots[0]
                    if not changed[0]:
                        changed[0] = True
                        if mutation == 'records':
                            target.data['point'][:] = [7, 8, 9]
                        elif mutation == 'schema':
                            target.pointlike_data_keys = ['point']
                        elif mutation == 'bind':
                            scene.add(target)
                        elif mutation == 'recipe':
                            target.passed_uv_func = lambda u, v: (u, v, 3)
                        elif mutation == 'lock':
                            target.lock_data(['point'])
                        else:
                            target.add(m.Dot())
                    return u, v, 0
                with self.assertRaises((RuntimeError, ValueError)):
                    Authored(function, resolution=(3, 4))
                self.assertEqual(roots[0].n_records(), 1)
                if mutation == 'records':
                    np.testing.assert_array_equal(roots[0].get_points(), [[7, 8, 9]])
                elif mutation == 'bind':
                    self.assertIs(roots[0]._scene, scene)
                elif mutation == 'family':
                    self.assertEqual(len(roots[0].submobjects), 1)
                elif mutation == 'lock':
                    self.assertIn('point', roots[0].locked_data_keys)

    def test_raw_initializer_refuses_invalid_requests_without_mutation(self):
        surface = m.Surface(resolution=(2, 2)).shift(m.RIGHT)
        before, view = surface.data.copy(), surface.get_points()
        for shape in ((3, 4), (2**62, 2), (0, 2**62)):
            with self.assertRaises(ValueError):
                m._initialize_surface_grid(surface, shape)
            np.testing.assert_array_equal(surface.data, before)
            np.testing.assert_array_equal(view, before['point'])
        scene = m.Scene()
        scene.add(surface)
        with self.assertRaisesRegex(RuntimeError, 'detached'):
            m._initialize_surface_grid(surface, (2, 2))
        np.testing.assert_array_equal(surface.data, before)

    def test_recursive_sampling_and_invalid_authored_topology_fail_closed(self):
        roots = []
        class Recursive(m.Surface):
            def init_data(self):
                roots.append(self)
                super().init_data()
            def uv_func(self, u, v):
                self.init_points()
                return u, v, 0
        with self.assertRaisesRegex(RuntimeError, 'already in progress'):
            Recursive(resolution=(2, 2))
        class Bad(m.Surface):
            def init_points(self):
                self.set_points([[0, 0, 0]])
        with self.assertRaisesRegex(ValueError, 'match resolution'):
            Bad(resolution=(2, 2))
        self.assertEqual(m.Surface(resolution=(2, 2)).n_records(), 4)

    def test_rendered_subclass_surface_matches_independent_native_geometry(self):
        class Authored(m.ParametricSurface):
            def init_points(self):
                super().init_points()
                self.shift(m.RIGHT)
        function = lambda u, v: (2*u-1, 2*v-1, .2*u*v)
        def render(path, surface, threads):
            scene = m.Scene()
            with render_session(scene, path, resolution=(48, 32), fps=24, threads=threads) as session:
                scene.add(surface)
                scene.wait(.125)
            self.assertEqual(session.result.frame_count, 3)
            return path.read_bytes()
        with tempfile.TemporaryDirectory() as tmp:
            tmp = Path(tmp)
            expected = native_candidate(function, (3, 4)).shift(m.RIGHT)
            expected.resolution = (3, 4)
            expected.compute_triangle_indices()
            reference = render(tmp/'reference.y4m', expected, 1)
            for threads in (1, 4):
                surface = Authored(function, resolution=(3, 4))
                self.assertEqual(render(tmp/f'authored-{threads}.y4m', surface, threads), reference)


    # fm-5wq.13. The stock solids build their geometry with specialized native
    # constructors; a subclass that authors a Surface hook is constructed in
    # Reference order instead. Each expectation is an independently sampled
    # ParametricSurface plus the Reference constructor's own post-steps.
    SOLIDS = (
        (m.Sphere, dict(radius=2, resolution=(9, 7), true_normals=False), (0, m.TAU), (0, m.PI),
         lambda s: s),
        (m.Torus, dict(resolution=(8, 6)), (0, m.TAU), (0, m.TAU), lambda s: s),
        (m.Cylinder, dict(height=3, radius=.5, axis=np.array([1., 1, 0]), resolution=(9, 5)),
         (0, m.TAU), (-1, 1),
         lambda s: s.scale(.5).set_depth(3, stretch=True).apply_matrix(m.z_to_vector([1, 1, 0]))),
        (m.Cone, dict(radius=2, resolution=(9, 5)), (0, m.TAU), (0, 1),
         lambda s: s.scale(2).set_depth(2, stretch=True)),
        (m.Line3D, dict(start=np.array([0., 0, 0]), end=np.array([2., 1, 0]), resolution=(7, 5)),
         (0, m.TAU), (-1, 1),
         lambda s: s.scale(.025).set_depth(np.sqrt(5), stretch=True)
                    .apply_matrix(m.z_to_vector([2, 1, 0])).shift([1, .5, 0])),
        (m.Disk3D, dict(radius=3, resolution=(3, 12)), (0, 1), (0, m.TAU), lambda s: s.scale(3)),
        (m.Square3D, dict(side_length=4, resolution=(3, 3)), (-1, 1), (-1, 1), lambda s: s.scale(2)),
    )

    @staticmethod
    def wave(u, v):
        return np.array([np.cos(u) + .1 * v, np.sin(u), .25 * v * v])

    def test_authored_uv_func_on_every_solid_is_sampled_at_construction(self):
        for cls, kwargs, u_range, v_range, place in self.SOLIDS:
            with self.subTest(solid=cls.__name__):
                calls = []
                wave = self.wave

                class Authored(cls):
                    def uv_func(self, u, v):
                        calls.append(1)
                        return wave(u, v)
                surface = Authored(**kwargs)
                shape = kwargs['resolution']
                # The Reference samples each grid point and its two nudges.
                self.assertEqual(len(calls), 3 * shape[0] * shape[1])
                expected = place(m.ParametricSurface(wave, u_range=u_range, v_range=v_range,
                                                     resolution=shape))
                np.testing.assert_allclose(surface.get_points(), expected.get_points(), atol=2e-5)
                # Unmodified subclasses keep the specialized native solid.
                plain = type('Plain' + cls.__name__, (cls,), {})(**kwargs)
                np.testing.assert_array_equal(plain.get_points(), cls(**kwargs).get_points())

    def test_sphere_true_normals_follow_the_reference_post_step(self):
        class Squashed(m.Sphere):
            def uv_func(self, u, v):
                return self.radius * np.array([np.cos(u) * np.sin(v), .5 * np.sin(u) * np.sin(v), -np.cos(v)])
        surface = Squashed(radius=2, resolution=(9, 7))
        points, normals = surface.data['point'], surface.data['d_normal_point']
        # The authored chart is sampled: y spans 2, half a radius-2 sphere's 4.
        self.assertAlmostEqual(float(np.ptp(points[:, 1])), 2.0, places=5)
        # ...and the Reference's true_normals post-step rescales that chart.
        np.testing.assert_allclose(normals, points * ((2 + surface.normal_nudge) / 2), rtol=1e-6)

    def test_point_hook_on_every_solid_is_dispatched_once(self):
        for cls, kwargs, *_ in self.SOLIDS:
            with self.subTest(solid=cls.__name__):
                calls = []

                class Hooked(cls):
                    def init_points(self):
                        calls.append(1)
                        super().init_points()
                        self.shift(m.RIGHT)
                surface = Hooked(**kwargs)
                self.assertEqual(len(calls), 1)
                expected = cls(**kwargs).shift(m.RIGHT)
                np.testing.assert_allclose(surface.get_points(), expected.get_points(), atol=2e-5)

    def test_rendered_authored_solid_matches_the_parametric_surface(self):
        wave = self.wave

        class Authored(m.Sphere):
            def uv_func(self, u, v):
                return wave(u, v)

        def render(path, surface):
            scene = m.Scene()
            with render_session(scene, path, resolution=(48, 32), fps=24, threads=2):
                scene.add(surface)
                scene.wait(.125)
            return path.read_bytes()
        with tempfile.TemporaryDirectory() as tmp:
            tmp = Path(tmp)
            options = dict(u_range=(0, m.TAU), v_range=(0, m.PI), resolution=(9, 7))
            expected = render(tmp / 'parametric.y4m', m.ParametricSurface(wave, **options))
            options.pop('u_range'), options.pop('v_range')
            self.assertEqual(render(tmp / 'authored.y4m', Authored(true_normals=False, **options)), expected)


def run_native_surface_lifecycle():
    result = unittest.TextTestRunner(verbosity=2).run(
        unittest.defaultTestLoader.loadTestsFromTestCase(SurfaceLifecycleTests))
    if not result.wasSuccessful():
        raise AssertionError('native surface lifecycle acceptance failed')
    return result.testsRun


if __name__ in ('__main__', '<run_path>'):
    run_native_surface_lifecycle()
