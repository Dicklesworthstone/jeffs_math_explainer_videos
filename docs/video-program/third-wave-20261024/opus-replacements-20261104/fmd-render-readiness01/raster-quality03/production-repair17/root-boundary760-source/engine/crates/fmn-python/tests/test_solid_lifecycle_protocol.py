"""Exercise real lifecycle adapters with explicit in-memory native seam doubles.

These tests prove dispatch, ownership checks, and error handling, not native
geometry or rendering. native_solid_lifecycle.py supplies that acceptance suite.
"""
from pathlib import Path
import sys
from types import SimpleNamespace
import unittest

import numpy as np

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / 'python'))
from fmn_python.surface_lifecycle import install_surface_lifecycle
from fmn_python.solid_lifecycle import install_solid_lifecycle


def portal():
    calls = []
    dtype = [('point', np.float32, (3,)), ('d_normal_point', np.float32, (3,)),
             ('rgba', np.float32, (4,))]

    def install_live(self):
        self.submobjects = []
        self.locked_data_keys = set()
        self.color, self.opacity = 'white', 1.

    class Mobject:
        data_dtype = dtype
        def __init__(self, **kwargs):
            install_live(self)
            vars(self).update(kwargs)
            self.data = np.zeros(0, dtype=self.data_dtype)
            self.init_data()
            self.init_points()
            self.init_uniforms()
        def _is_bound(self):
            return vars(self).get('bound', False)
        def init_data(self):
            pass
        def init_uniforms(self):
            pass
        def init_colors(self):
            self.set_color(self.color)
        def set_color(self, value):
            self.color = value
        def get_family(self):
            return [self, *self.submobjects]
        def get_points(self):
            return self.data['point']
        def get_height(self):
            return float(np.ptp(self.get_points()[:, 1])) if len(self.data) else 0.
        def set_z_index(self, value):
            self.z_index = value
        def set_shading(self, *values):
            self.shading = values
        def apply_depth_test(self):
            self.depth_test = True
        def deactivate_depth_test(self):
            self.depth_test = False
        def set_points(self, points):
            records = np.zeros(len(points), dtype=self.data_dtype)
            points = np.asarray(points).reshape((-1, 3))
            records['point'] = points
            records['d_normal_point'] = points + (0, 0, .001)
            self.data = records
        def shift(self, delta):
            for name in ('point', 'd_normal_point'):
                self.data[name] += delta
        def _build_parametric_surface(self, factory, function, ur, vr, shape, epsilon, nudge):
            calls.append('generic')
            self.set_points([function(u, v) for u in np.linspace(*ur, shape[0])
                             for v in np.linspace(*vr, shape[1])])
            return []
        def _build_torus(self, factory, r1, r2, ur, vr, shape, axis, epsilon, nudge, z):
            calls.append(('torus', r1, r2))
            # Deliberately recognizable fixtures, NOT a replacement UV kernel.
            self.set_points([(r1, r2, k) for k in range(shape[0]*shape[1])])
            return []

        def _build_sphere(self, factory, radius, ur, vr, shape, true_normals,
                          clockwise, axis, epsilon, nudge):
            calls.append(('sphere', radius, true_normals, clockwise))
            self.set_points([(radius, k, 0) for k in range(shape[0]*shape[1])])
            if true_normals and radius:
                self.data['d_normal_point'] = self.data['point'] * ((radius+nudge)/radius)
            return []

    class Surface(Mobject):
        def uv_func(self, u, v):
            return u, v, 0.
        def init_points(self):
            calls.append('regenerate')
        def compute_triangle_indices(self):
            self.triangles_computed = True

    class ParametricSurface(Surface):
        def uv_func(self, u, v):
            return self.passed_uv_func(u, v)

    class Torus(Surface):
        def uv_func(self, u, v):
            raise AssertionError('stock Torus must select its specialized native builder')

    class Sphere(Surface):
        def uv_func(self, u, v):
            raise AssertionError('stock Sphere must select its specialized native builder')

    def publish(self, shape, candidate=None):
        count = shape[0]*shape[1]
        if candidate is not None:
            records = np.zeros(count, dtype=self.data_dtype)
            keep = min(len(self.data), count)
            records[:keep] = self.data[:keep]
            for key in ('point', 'd_normal_point'):
                records[key] = candidate.data[key]
            self.data = records
        if len(self.data) != count:
            raise ValueError('record count must match resolution')
        self.native_shape = shape

    def refuse(where, flags):
        if flags:
            raise TypeError(where + ': unexpected keyword')

    def refuse_surface_keywords(where, kwargs):
        refuse(where, list(kwargs))

    native = SimpleNamespace(Mobject=Mobject, Surface=Surface,
        ParametricSurface=ParametricSurface, Torus=Torus, Sphere=Sphere, _np=np, GREY='grey',
        _refuse_unrouted=refuse, _refuse_surface_keywords=refuse_surface_keywords,
        _install_live_state=install_live,
        _initialize_surface_grid=publish, _native_surface_shell_factory=None)
    install_surface_lifecycle(native)
    install_solid_lifecycle(native)
    return native, calls


class SolidLifecycleProtocol(unittest.TestCase):
    def test_hooks_are_cooperative_once_and_native_sampling_is_selected(self):
        m, calls = portal()
        events = []
        class Hooks:
            def init_data(self):
                events.append(('data', self.r1, self.r2, self.resolution))
                super().init_data()
            def init_points(self):
                events.append('points')
                super().init_points()
                self.shift((1, 0, 0))
            def init_uniforms(self):
                events.append('uniforms')
                super().init_uniforms()
                self.set_shading(.2, .3, .4)
            def init_colors(self):
                events.append('colors')
                super().init_colors()
                self.set_color('blue')
        class Authored(Hooks, m.Torus):
            pass
        surface = Authored(r1=2, r2=.5, resolution=(2, 3), color='red', z_index=7)
        self.assertEqual(events, [('data', 2., .5, (2, 3)), 'points', 'uniforms', 'colors'])
        self.assertEqual(calls, [('torus', 2., .5)])
        self.assertEqual(surface.color, 'blue')
        self.assertEqual(surface.shading, (.2, .3, .4))
        self.assertEqual(surface.z_index, 7)
        self.assertEqual(surface.native_shape, (2, 3))
        np.testing.assert_array_equal(surface.get_points()[:, 0], 3.)

    def test_authored_uv_is_evaluated_not_replaced_by_native_shape(self):
        m, calls = portal()
        visits = []
        class Authored(m.Torus):
            def uv_func(self, u, v):
                visits.append((u, v))
                return u, v, self.r1 - self.r2
        surface = Authored(r1=2, r2=.5, resolution=(3, 4))
        self.assertEqual(len(visits), 12)
        self.assertEqual(calls, ['generic'])
        np.testing.assert_array_equal(surface.get_points()[:, 2], 1.5)

    def test_replacement_point_hook_never_samples_uv(self):
        m, calls = portal()
        class Authored(m.Torus):
            def init_points(self):
                self.set_points([(1, 2, 3)] * 4)
        surface = Authored(resolution=(2, 2))
        self.assertEqual(calls, [])
        self.assertEqual(surface.native_shape, (2, 2))
        np.testing.assert_array_equal(surface.get_points(), [(1, 2, 3)] * 4)

    def test_initialization_preserves_authored_schema_and_child_identity(self):
        m, _ = portal()
        child = object()
        class Authored(m.Torus):
            data_dtype = [*m.Surface.data_dtype, ('tag', np.float32, (1,))]
            def init_data(self):
                super().init_data()
                self.set_points([(0, 0, 0)] * 4)
                self.data['tag'] = 23
                self.submobjects.append(child)
        surface = Authored(resolution=(2, 2))
        self.assertIs(surface.submobjects[0], child)
        np.testing.assert_array_equal(surface.data['tag'], 23)

    def test_shape_recipe_mutation_is_not_published(self):
        m, _ = portal()
        roots = []
        class Authored(m.Torus):
            def init_data(self):
                roots.append(self)
            def uv_func(self, u, v):
                self.r1 += 1
                return u, v, 0
        with self.assertRaisesRegex(RuntimeError, 'shape parameters changed'):
            Authored(resolution=(2, 2))
        self.assertEqual(len(roots[0].data), 0)

    def test_callback_exception_identity_and_next_constructor_remains_usable(self):
        m, _ = portal()
        error = RuntimeError('authored failure')
        class Authored(m.Torus):
            def uv_func(self, u, v):
                raise error
        with self.assertRaises(RuntimeError) as caught:
            Authored(resolution=(2, 2))
        self.assertIs(caught.exception, error)
        self.assertEqual(m.Torus(resolution=(2, 2)).native_shape, (2, 2))

    def test_bound_and_recursive_construction_refuse_before_recipe_changes(self):
        m, _ = portal()
        surface = m.Torus(r1=4, resolution=(2, 2))
        surface.bound = True
        with self.assertRaisesRegex(RuntimeError, 'detached'):
            m.Torus.__init__(surface, r1=99)
        self.assertEqual(surface.r1, 4)
        class Recursive(m.Torus):
            def init_data(self):
                with self_test.assertRaisesRegex(RuntimeError, 'already in progress'):
                    m.Torus.__init__(self, r1=99)
                self_test.assertEqual(self.r1, 4)
        self_test = self
        Recursive(r1=4, resolution=(2, 2))

    def test_admission_rejects_bad_dimensions_and_resolution_before_hooks(self):
        m, _ = portal()
        events = []
        class Authored(m.Torus):
            def init_data(self):
                events.append('data')
        for kwargs in ({'r1': float('nan')}, {'r2': float('inf')}, {'r1': 1e40},
                       {'resolution': (2.5, 2)}, {'resolution': (1000, 1000)},
                       {'unsupported': 1}):
            with self.assertRaises((TypeError, ValueError)):
                Authored(**kwargs)
        self.assertEqual(events, [])

    def test_class_monkey_patch_is_not_mistaken_for_the_stock_uv(self):
        m, calls = portal()
        m.Torus.uv_func = lambda self, u, v: (u, v, 9)
        surface = m.Torus(resolution=(2, 2))
        self.assertEqual(calls, ['generic'])
        np.testing.assert_array_equal(surface.get_points()[:, 2], 9)

    def test_installer_is_idempotent_and_plain_surface_keeps_its_sampler(self):
        m, calls = portal()
        constructor = m.Torus.__init__
        install_solid_lifecycle(m)
        self.assertIs(m.Torus.__init__, constructor)
        surface = m.Surface(resolution=(2, 2))
        self.assertEqual(calls, ['generic'])
        surface.init_points()
        self.assertEqual(calls[-1], 'regenerate')


class SphereLifecycleProtocol(unittest.TestCase):
    def test_hooks_run_once_and_radial_correction_follows_the_color_hook(self):
        m, calls = portal()
        events = []
        class Authored(m.Sphere):
            def init_data(self):
                events.append(('data', self.radius, self.clockwise))
                super().init_data()
            def init_points(self):
                events.append('points')
                super().init_points()
                self.shift((1, 0, 0))
            def init_uniforms(self):
                events.append('uniforms')
                super().init_uniforms()
            def init_colors(self):
                events.append('colors')
                super().init_colors()
                self.data['d_normal_point'][:] = 0
                self.normal_nudge = .25
        actual = Authored(radius=2, clockwise=True, resolution=(2, 2))
        self.assertEqual(events, [('data', 2., True), 'points', 'uniforms', 'colors'])
        self.assertEqual(calls, [('sphere', 2., True, True)])
        np.testing.assert_array_equal(actual.get_points()[:, 0], 3.)
        np.testing.assert_array_equal(actual.data['d_normal_point'], actual.get_points() * 1.125)

    def test_untouched_stock_columns_are_not_double_rounded(self):
        m, _ = portal()
        original, captured = m.Mobject._build_sphere, []
        def native_with_rounding(self, *args):
            result = original(self, *args)
            self.data['d_normal_point'] = np.nextafter(self.data['d_normal_point'], np.float32(1.))
            captured.append(self.data['d_normal_point'].tobytes())
            return result
        m.Mobject._build_sphere = native_with_rounding
        class Forwarding(m.Sphere):
            def init_points(self):
                super().init_points()
        actual = Forwarding(radius=.7, resolution=(2, 2))
        self.assertEqual(actual.data['d_normal_point'].tobytes(), captured[0])

    def test_false_true_normals_preserves_authored_normal_column(self):
        m, calls = portal()
        class Authored(m.Sphere):
            def init_colors(self):
                super().init_colors()
                self.data['d_normal_point'][:] = (7, 8, 9)
        actual = Authored(resolution=(2, 2), true_normals=False)
        self.assertEqual(calls, [('sphere', 1., False, False)])
        np.testing.assert_array_equal(actual.data['d_normal_point'], [(7, 8, 9)] * 4)

    def test_uv_override_uses_generic_sampling_and_final_radial_normals(self):
        m, calls = portal()
        class Authored(m.Sphere):
            def uv_func(self, u, v):
                return u, v, self.radius
        actual = Authored(radius=2, resolution=(2, 2))
        self.assertEqual(calls, ['generic'])
        expected = (actual.get_points().astype(np.float64) * 1.0005).astype(np.float32)
        np.testing.assert_array_equal(actual.data['d_normal_point'], expected)

    def test_replaced_points_skip_all_sampling_and_still_get_final_normals(self):
        m, calls = portal()
        class Authored(m.Sphere):
            def init_points(self):
                self.set_points([(1, 2, 3)] * 4)
        actual = Authored(radius=2, resolution=(2, 2), normal_nudge=.5)
        self.assertEqual(calls, [])
        np.testing.assert_array_equal(actual.data['d_normal_point'], [(1.25, 2.5, 3.75)] * 4)

    def test_init_data_radius_edits_do_not_replace_constructor_normal_argument(self):
        m, calls = portal()
        class Authored(m.Sphere):
            def init_data(self):
                self.radius = 3
        actual = Authored(radius=2, resolution=(2, 2), normal_nudge=.5)
        self.assertEqual(calls, [('sphere', 3., False, False)])
        np.testing.assert_array_equal(actual.data['d_normal_point'], actual.get_points() * 1.25)

    def test_empty_and_zero_radius_shapes_are_legal(self):
        m, _ = portal()
        empty = m.Sphere(resolution=(0, 3))
        self.assertEqual(len(empty.data), 0)
        zero = m.Sphere(radius=0, resolution=(2, 2))
        self.assertTrue(np.isfinite(zero.data['d_normal_point']).all())

    def test_recipe_changes_fail_before_publishing_an_authored_sample(self):
        m, _ = portal()
        roots = []
        class Authored(m.Sphere):
            def init_data(self):
                roots.append(self)
            def uv_func(self, u, v):
                self.clockwise = not self.clockwise
                self.radius += 1
                return u, v, 0
        with self.assertRaisesRegex(RuntimeError, 'shape parameters changed'):
            Authored(resolution=(2, 2))
        self.assertEqual(len(roots[0].data), 0)

    def test_live_authored_candidate_receives_radial_correction_before_publish(self):
        m, _ = portal()
        class Authored(m.Sphere):
            def uv_func(self, u, v):
                return u, v, 2
        actual = Authored(radius=2, resolution=(2, 2), normal_nudge=.5)
        before = actual.data.copy()
        def sample():
            candidate = m.Surface.__new__(m.Surface)
            candidate.set_points([(1, 2, 3)] * 4)
            return candidate
        result = m._fmn_build_solid_candidate(actual, {'normal_nudge': .5}, sample)
        np.testing.assert_array_equal(result.data['d_normal_point'], [(1.25, 2.5, 3.75)] * 4)
        np.testing.assert_array_equal(actual.data, before)

    def test_late_invalid_normals_refuse_before_writing_the_normal_column(self):
        m, _ = portal()
        roots = []
        class Authored(m.Sphere):
            def init_colors(self):
                roots.append(self)
                self.data['d_normal_point'][:] = 42
                self.normal_nudge = -1
        with self.assertRaisesRegex(ValueError, 'normal_nudge'):
            Authored(resolution=(2, 2))
        np.testing.assert_array_equal(roots[0].data['d_normal_point'], 42)
        self.assertEqual(m.Sphere(resolution=(2, 2)).native_shape, (2, 2))

    def test_final_normals_use_actual_point_records_not_a_custom_getter(self):
        m, _ = portal()
        calls = []
        class Authored(m.Sphere):
            def init_colors(self):
                def getter():
                    calls.append('getter')
                    raise AssertionError('normal correction owns the actual point column')
                self.get_points = getter
                self.data['d_normal_point'][:] = 42
        actual = Authored(resolution=(2, 2), radius=2, normal_nudge=.5)
        self.assertEqual(calls, [])
        np.testing.assert_array_equal(actual.data['d_normal_point'], actual.data['point'] * 1.25)

    def test_late_animation_lock_is_respected_before_normal_writes(self):
        m, _ = portal()
        roots = []
        class Authored(m.Sphere):
            def init_colors(self):
                roots.append(self)
                self.data['d_normal_point'][:] = 42
                self.locked_data_keys.add('d_normal_point')
        with self.assertRaisesRegex(RuntimeError, 'active animation'):
            Authored(resolution=(2, 2))
        np.testing.assert_array_equal(roots[0].data['d_normal_point'], 42)

    def test_candidate_is_released_after_constructor_success_and_failure(self):
        import gc
        import weakref
        m, _ = portal()
        original, candidates = m.Mobject._build_sphere, []
        def capture(self, *args):
            candidates.append(weakref.ref(self))
            return original(self, *args)
        m.Mobject._build_sphere = capture
        m.Sphere(resolution=(2, 2))
        class Broken(m.Sphere):
            def init_colors(self):
                raise ValueError('late hook failure')
        with self.assertRaisesRegex(ValueError, 'late hook failure'):
            Broken(resolution=(2, 2))
        gc.collect()
        self.assertEqual([ref() for ref in candidates], [None, None])



if __name__ == '__main__':
    unittest.main()
