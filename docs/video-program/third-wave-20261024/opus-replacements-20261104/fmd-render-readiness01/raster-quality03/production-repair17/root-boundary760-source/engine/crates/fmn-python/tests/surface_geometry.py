"""Actual native sampler, live records, scene clock, texture and output witnesses."""
import io
from pathlib import Path
import tempfile
import unittest
import numpy as np
import manimlib as m


class SurfaceGeometryTests(unittest.TestCase):
    def make(self, controls=None):
        controls = [0.0] if controls is None else controls
        return m.ParametricSurface(lambda u, v: (u, v, controls[0] * u * v),
                                   u_range=(-1, 1), v_range=(-1, 1), resolution=(5, 7))

    def test_detached_rebuild_does_not_invent_scene_membership(self):
        surface = self.make()
        self.assertNotIn("_scene", vars(surface))
        surface.passed_uv_func = lambda u, v: (u, v, 3)
        surface.init_points()
        self.assertNotIn("_scene", vars(surface))
        self.assertFalse(surface._is_bound())
        np.testing.assert_array_equal(surface.get_points()[:, 2], 3)

    def test_callback_adoption_refuses_publication_but_preserves_authored_effect(self):
        surface, scene = self.make(), m.Scene()
        before = surface.data.copy()
        def sample(u, v):
            if not surface._is_bound():
                scene.add(surface)
            return u, v, 4
        surface.passed_uv_func = sample
        with self.assertRaisesRegex(RuntimeError, 'changed during regeneration'):
            surface.init_points()
        self.assertEqual(tuple(scene.mobjects), (surface,))
        self.assertIs(surface._scene, scene)
        np.testing.assert_array_equal(surface.data, before)
        surface.passed_uv_func = lambda u, v: (u, v, 4)
        surface.init_points()
        np.testing.assert_array_equal(surface.get_points()[:, 2], 4)

    def test_replacing_the_dispatched_uv_method_is_detected(self):
        surface = self.make()
        before = surface.data.copy()
        replacement = lambda u, v: (u, v, 9)
        def sample(u, v):
            surface.uv_func = replacement
            return u, v, 2
        surface.passed_uv_func = sample
        with self.assertRaisesRegex(RuntimeError, 'UV function changed'):
            surface.init_points()
        self.assertIs(surface.uv_func, replacement)
        np.testing.assert_array_equal(surface.data, before)
        surface.init_points()
        np.testing.assert_array_equal(surface.get_points()[:, 2], 9)

    def test_new_animation_lock_during_callback_is_not_overwritten(self):
        surface = self.make()
        before = surface.data.copy()
        def sample(u, v):
            surface.lock_data(["point"])
            return u, v, 2
        surface.passed_uv_func = sample
        with self.assertRaisesRegex(RuntimeError, 'active animation'):
            surface.init_points()
        np.testing.assert_array_equal(surface.data, before)
        surface.unlock_data()
        surface.passed_uv_func = lambda u, v: (u, v, 2)
        surface.init_points()
        np.testing.assert_array_equal(surface.get_points()[:, 2], 2)

    def test_native_regeneration_updates_positions_and_normals(self):
        controls = [0.0]
        surface = self.make(controls)
        controls[0] = 2.0
        self.assertIsNone(surface.init_points())
        points = np.asarray(surface.get_points(), dtype=float)
        np.testing.assert_allclose(points[:, 2], 2 * points[:, 0] * points[:, 1], atol=1e-6)
        expected = np.column_stack((-2 * points[:, 1], -2 * points[:, 0], np.ones(len(points))))
        expected /= np.linalg.norm(expected, axis=1)[:, None]
        np.testing.assert_allclose(surface.get_unit_normals(), expected, atol=3e-4)
        self.assertFalse(surface._is_bound())

    def test_bound_identity_children_callbacks_and_views_survive(self):
        controls = [0.0]
        surface = self.make(controls)
        child = m.Dot()
        surface.add(child)
        scene = m.Scene()
        scene.add(surface)
        callbacks = []
        callback = lambda obj, dt: callbacks.append(dt)
        surface.add_updater(callback, call=False)
        records = surface.data
        points = records['point']
        colors = np.array(records['rgba'])
        roots = tuple(scene.mobjects)
        controls[0] = .5
        surface.init_points()
        self.assertIs(surface.submobjects[0], child)
        self.assertIs(surface._scene, scene)
        self.assertEqual(tuple(scene.mobjects), roots)
        self.assertIs(surface.updaters[0], callback)
        self.assertEqual(callbacks, [])
        self.assertEqual(scene.get_time(), 0)
        np.testing.assert_array_equal(points, surface.get_points())
        self.assertTrue(np.any(points[:, 2] != 0))
        np.testing.assert_array_equal(records['rgba'], colors)

    def test_callback_error_publishes_no_geometry_and_can_retry(self):
        surface = self.make()
        original = surface.data.copy()
        called = []
        failure = LookupError('authored surface failure')
        def broken(u, v):
            called.append((u, v))
            if len(called) == 4:
                raise failure
            return u, v, 5
        surface.passed_uv_func = broken
        with self.assertRaises(LookupError) as caught:
            surface.init_points()
        self.assertIs(caught.exception, failure)
        self.assertEqual(len(called), 4)
        np.testing.assert_array_equal(surface.data, original)
        surface.passed_uv_func = lambda u, v: (u, v, 2)
        surface.init_points()
        np.testing.assert_array_equal(surface.get_points()[:, 2], 2)

    def test_invalid_samples_refuse_before_publication(self):
        surface = self.make()
        original = surface.data.copy()
        for sample in ((0, 0), (0, 0, float('nan')), (1e50, 0, 0), (1j, 0, 0)):
            surface.passed_uv_func = lambda u, v: sample
            with self.subTest(sample=sample), self.assertRaises((TypeError, ValueError)):
                surface.init_points()
            np.testing.assert_array_equal(surface.data, original)

    def test_reentrant_regeneration_refuses_without_recursing(self):
        surface = self.make()
        original = surface.data.copy()
        def nested(u, v):
            surface.init_points()
            return u, v, 0
        surface.passed_uv_func = nested
        with self.assertRaisesRegex(RuntimeError, 'already in progress'):
            surface.init_points()
        np.testing.assert_array_equal(surface.data, original)

    def test_authored_geometry_changes_are_not_overwritten(self):
        surface = self.make()
        changed = False
        def mutate(u, v):
            nonlocal changed
            if not changed:
                changed = True
                surface.shift((1, 0, 0))
            return u, v, 5
        surface.passed_uv_func = mutate
        with self.assertRaisesRegex(RuntimeError, 'changed during regeneration'):
            surface.init_points()
        np.testing.assert_allclose(surface.get_center(), (1, 0, 0), atol=1e-6)
        np.testing.assert_array_equal(surface.get_points()[:, 2], 0)

    def test_new_recipe_installed_during_sampling_is_not_silently_overwritten(self):
        surface = self.make()
        original = surface.data.copy()
        new = lambda u, v: (u, v, 3)
        def mutate(u, v):
            surface.passed_uv_func = new
            return u, v, 1
        surface.passed_uv_func = mutate
        with self.assertRaisesRegex(RuntimeError, 'UV function changed'):
            surface.init_points()
        np.testing.assert_array_equal(surface.data, original)
        self.assertIs(surface.passed_uv_func, new)

    def test_invalid_controls_and_same_count_reshapes_never_call_uv(self):
        surface = self.make()
        called = []
        surface.passed_uv_func = lambda u, v: called.append((u, v)) or (u, v, 0)
        for key, value in (('resolution', (7, 5)), ('resolution', (1000, 1000)),
                           ('u_range', (0, 0)), ('epsilon', 0), ('normal_nudge', float('inf'))):
            old = getattr(surface, key)
            setattr(surface, key, value)
            with self.subTest(key=key, value=value), self.assertRaises(ValueError):
                surface.init_points()
            setattr(surface, key, old)
        self.assertEqual(called, [])

    def test_uv_domains_can_change_without_losing_native_topology(self):
        surface = self.make([1.0])
        surface.u_range, surface.v_range = (-2, 2), (-3, 3)
        surface.init_points()
        np.testing.assert_allclose(surface.uv_to_point(2, 3), (2, 3, 6), atol=1e-5)
        self.assertEqual(m._surface_grid_resolution(surface), (5, 7))

    def test_subclass_uv_override_uses_the_existing_native_sampler(self):
        class Wave(m.Surface):
            height = 0
            def uv_func(self, u, v):
                return u, v, self.height * u
        surface = Wave(resolution=(5, 7))
        surface.height = 3
        surface.init_points()
        points = surface.get_points()
        np.testing.assert_allclose(points[:, 2], 3 * points[:, 0], atol=1e-6)

    def test_become_replaces_topology_and_python_uv_projection(self):
        for bound in (False, True):
            with self.subTest(bound=bound):
                surface = self.make()
                target = m.ParametricSurface(lambda u, v: (u, v, u + v),
                                             u_range=(-2, 2), v_range=(-3, 3), resolution=(7, 8))
                if bound:
                    scene = m.Scene()
                    scene.add(surface)
                self.assertIs(surface.become(target), surface)
                self.assertEqual(surface.resolution, (7, 8))
                self.assertEqual(m._surface_grid_resolution(surface), (7, 8))
                np.testing.assert_allclose(surface.uv_to_point(2, 3), (2, 3, 5), atol=1e-5)
                self.assertEqual(len(surface.get_triangle_indices()), 6 * 6 * 7)
                self.assertLess(surface.get_triangle_indices().max(), surface.n_records())

    def test_group_become_updates_nested_surface_projections(self):
        left, right = self.make(), m.ParametricSurface(lambda u, v: (u, v, 1), resolution=(3, 4))
        group = m.Group(left)
        group.become(m.Group(right))
        self.assertIs(group.submobjects[0], left)
        self.assertEqual(left.resolution, (3, 4))
        np.testing.assert_allclose(left.uv_to_point(.5, .5), (.5, .5, 1), atol=1e-6)

    def test_native_geometry_copy_refuses_mismatched_topology_atomically(self):
        surface, source = self.make(), m.ParametricSurface(lambda u, v: (u, v, 3), resolution=(7, 5))
        before = surface.data.copy()
        with self.assertRaisesRegex(ValueError, 'matching native UV topology'):
            m._copy_surface_geometry(surface, source)
        np.testing.assert_array_equal(surface.data, before)

    def test_native_geometry_copy_keeps_styles_and_copies_across_arenas(self):
        surface, source = self.make(), self.make([2.0])
        a, b = m.Scene(), m.Scene()
        a.add(surface)
        b.add(source)
        colors = surface.data['rgba'].copy()
        source.shift((1, 0, 0))
        m._copy_surface_geometry(surface, source)
        np.testing.assert_array_equal(surface.data['point'], source.data['point'])
        np.testing.assert_array_equal(surface.data['d_normal_point'], source.data['d_normal_point'])
        np.testing.assert_array_equal(surface.data['rgba'], colors)
        self.assertIs(surface._scene, a)
        self.assertIs(source._scene, b)

    def test_textured_surface_refresh_retains_uvs_opacity_and_native_images(self):
        camera = m.Camera(resolution=(16, 16))
        payload = camera.capture_snapshot(m.Square(fill_opacity=1, fill_color=m.RED)).png()
        with tempfile.TemporaryDirectory(prefix='fmn-live-surface-') as directory:
            texture = Path(directory) / 'texture.png'
            texture.write_bytes(payload)
            controls = [0.0]
            source = self.make(controls)
            surface = m.TexturedSurface(source, str(texture))
            surface.set_opacity(.4)
            before = surface.data.copy()
            controls[0] = 2.0
            source.init_points()
            surface.init_points()
            np.testing.assert_array_equal(surface.data['point'], source.data['point'])
            np.testing.assert_array_equal(surface.data['im_coords'], before['im_coords'])
            np.testing.assert_array_equal(surface.data['opacity'], before['opacity'])
            self.assertTrue(np.frombuffer(camera.capture_snapshot(surface).pixels(), dtype=np.uint8).any())

    def test_specialized_solid_rebuilds_keep_shape_parameters_and_normal_rules(self):
        cases = (
            (m.Sphere, dict(radius=2, true_normals=True, clockwise=False)),
            (m.Sphere, dict(radius=2, true_normals=True, clockwise=True)),
            (m.Sphere, dict(radius=2, true_normals=False, clockwise=True)),
            (m.Torus, dict(r1=2, r2=.5)),
            (m.Cylinder, dict(radius=2, height=3, axis=(1, 2, 3))),
            (m.Cone, dict(radius=2, height=3, axis=(1, 2, 3))),
            (m.Disk3D, dict(radius=3)),
            (m.Square3D, dict(side_length=5)),
        )
        for cls, parameters in cases:
            with self.subTest(solid=cls.__name__, parameters=parameters):
                surface = cls(resolution=(5, 7))
                for name, value in parameters.items():
                    setattr(surface, name, value)
                colors = surface.data['rgba'].copy()
                expected = cls(resolution=(5, 7), **parameters)
                surface.init_points()
                np.testing.assert_array_equal(surface.data['point'], expected.data['point'])
                np.testing.assert_array_equal(surface.data['d_normal_point'], expected.data['d_normal_point'])
                np.testing.assert_array_equal(surface.data['rgba'], colors)
                self.assertEqual(surface._solid_params, expected._solid_params)

    def test_solid_subclass_uv_override_still_executes_authored_callback(self):
        class Custom(m.Sphere):
            def uv_func(self, u, v):
                return u, v, 7
        surface = Custom(resolution=(5, 7))
        surface.init_points()
        np.testing.assert_array_equal(surface.data['point'][:, 2], 7)

    def test_tracker_driven_surface_writes_real_animation_frames(self):
        from fmn_python import render_scene
        class LiveSurface(m.Scene):
            def construct(self):
                tracker = m.ValueTracker(0)
                self.surface = m.ParametricSurface(lambda u, v: (u, v, tracker.get_value() * u * v),
                                    u_range=(-2, 2), v_range=(-2, 2), resolution=(5, 7))
                self.surface.add_updater(lambda obj: obj.init_points(), call=False)
                self.add(self.surface)
                self.play(tracker.animate.set_value(1), run_time=.5, rate_func=m.linear)
        with tempfile.TemporaryDirectory(prefix='fmn-live-surface-render-') as directory:
            destination = Path(directory) / 'surface.y4m'
            result = render_scene(LiveSurface, destination, format='y4m', resolution=(96, 64), fps=8)
            raw = destination.read_bytes()
            self.assertGreaterEqual(result.frame_count, 4)
            chunks = raw.split(b'FRAME\n')[1:]
            self.assertGreater(len(set(chunks)), 1)


def run_surface_geometry_acceptance():
    stream = io.StringIO()
    result = unittest.TextTestRunner(stream=stream, verbosity=2).run(
        unittest.defaultTestLoader.loadTestsFromTestCase(SurfaceGeometryTests))
    if not result.wasSuccessful():
        raise AssertionError(stream.getvalue())
    return result.testsRun


if __name__ == '__main__':
    unittest.main()
