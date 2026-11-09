"""In-place procedural surface resolution changes on the real native engine."""
import copy
import io
import itertools
from pathlib import Path
import tempfile
import unittest

import numpy as np
import manimlib as m


def surface(shape=(3, 4), amplitude=1.0, **kwargs):
    return m.ParametricSurface(lambda u, v: (u, v, amplitude * (u*u - v*v)),
                              u_range=(-1, 1), v_range=(-1, 1), resolution=shape,
                              shading=(0, 0, 0), **kwargs)


def analytic(shape, amplitude=1.0):
    u, v = np.meshgrid(np.linspace(-1, 1, shape[0]), np.linspace(-1, 1, shape[1]), indexing='ij')
    return np.stack((u, v, amplitude * (u*u-v*v)), axis=-1).reshape(-1, 3)


class SurfaceRegriddingTests(unittest.TestCase):
    def topology(self, obj, shape):
        self.assertEqual(obj.resolution, shape)
        self.assertEqual(m._surface_grid_resolution(obj), shape)
        self.assertEqual(obj.n_records(), shape[0]*shape[1])
        indices = obj.get_triangle_indices()
        self.assertEqual(len(indices), 6*(shape[0]-1)*(shape[1]-1))
        self.assertEqual(int(indices.max()), obj.n_records()-1)
        self.assertEqual(obj.get_uv_grid().shape, (*shape, 2))

    def test_refinement_and_coarsening_sample_the_actual_recipe(self):
        obj = surface()
        for shape in ((7, 9), (4, 3), (2, 2), (5, 6)):
            self.assertIs(obj.set_resolution(shape), obj)
            self.topology(obj, shape)
            np.testing.assert_allclose(obj.get_points(), analytic(shape), atol=2e-6)
            direct = surface(shape)
            np.testing.assert_array_equal(obj.data['d_normal_point'], direct.data['d_normal_point'])
        # A new interior sample is NOT the bilinear interpolant of a 2x2 saddle.
        self.assertNotEqual(float(obj.get_points()[7, 2]), 0.)

    def test_equal_count_different_topology_updates_records_and_queries(self):
        obj = surface((2, 6))
        view = obj.get_points()
        obj.set_resolution((3, 4))
        self.topology(obj, (3, 4))
        np.testing.assert_allclose(view, analytic((3, 4)), atol=2e-6)
        np.testing.assert_allclose(obj.uv_to_point(0, -1), (0, -1, -1), atol=2e-6)

    def test_same_shape_retains_live_views_and_authored_triangle_order(self):
        state = [1.]
        obj = m.ParametricSurface(lambda u, v: (u, v, state[0]), resolution=(3, 4))
        view, indices = obj.get_points(), obj.get_triangle_indices()
        indices[:] = indices[::-1].copy()
        order = indices.copy()
        state[0] = 2.
        obj.set_resolution((3, 4))
        self.assertIs(obj.get_triangle_indices(), indices)
        np.testing.assert_array_equal(indices, order)
        np.testing.assert_array_equal(view[:, 2], 2.)

    def test_resizing_detaches_old_views_and_preserves_saved_state_and_owners(self):
        obj, child = surface(), m.Dot()
        obj.add(child)
        calls = []
        callback = lambda s, dt: calls.append(dt)
        obj.add_updater(callback, call=False)
        scene = m.Scene(); scene.add(obj)
        obj.save_state()
        old_view = obj.get_points(); before = old_view.copy()
        obj.set_resolution((7, 5))
        self.assertIs(obj._scene, scene)
        self.assertEqual(tuple(scene.mobjects), (obj,))
        self.assertEqual(tuple(obj.submobjects), (child,))
        self.assertIs(obj.updaters[0], callback)
        self.assertEqual(calls, [])
        self.assertEqual(scene.get_time(), 0.)
        np.testing.assert_array_equal(old_view, before)
        old_view[0] = (99, 99, 99)
        np.testing.assert_allclose(obj.get_points(), analytic((7, 5)), atol=2e-6)
        self.topology(obj.saved_state, (3, 4))
        obj.restore()
        self.topology(obj, (3, 4))
        np.testing.assert_allclose(obj.get_points(), before, atol=2e-6)

    def test_paint_fields_follow_normalized_uv_not_flat_record_order(self):
        obj = surface((2, 3))
        p = obj.get_points().copy()
        obj.data['rgba'][:] = np.column_stack(((p[:, 0]+1)/2, (p[:, 1]+1)/2,
                                               np.full(len(p), .25), np.full(len(p), .4)))
        obj.set_resolution((5, 7))
        p = analytic((5, 7))
        expected = np.column_stack(((p[:, 0]+1)/2, (p[:, 1]+1)/2,
                                    np.full(len(p), .25), np.full(len(p), .4)))
        np.testing.assert_allclose(obj.data['rgba'], expected, atol=2e-7)

    def test_solid_geometry_parameters_and_true_normals_are_resampled(self):
        for cls, options in ((m.Sphere, dict(radius=2, true_normals=True)),
                             (m.Torus, dict(r1=2, r2=.3)),
                             (m.Cylinder, dict(radius=.7, height=3, axis=(1, 2, 1))),
                             (m.Cone, dict(radius=1.2, height=2)),
                             (m.Disk3D, dict(radius=2)),
                             (m.Square3D, dict(side_length=3))):
            with self.subTest(cls=cls.__name__):
                obj = cls(resolution=(3, 4), **options)
                obj.set_resolution((7, 6))
                reference = cls(resolution=(7, 6), **options)
                self.topology(obj, (7, 6))
                np.testing.assert_array_equal(obj.get_points(), reference.get_points())
                np.testing.assert_array_equal(obj.data['d_normal_point'], reference.data['d_normal_point'])

    def test_metadata_is_not_temporarily_changed_during_authored_sampling(self):
        obj = surface()
        observed = []
        def recipe(u, v):
            observed.append((obj.resolution, m._surface_grid_resolution(obj)))
            return u, v, 2
        obj.passed_uv_func = recipe
        obj.set_resolution((5, 6))
        self.assertTrue(observed)
        self.assertTrue(all(pair == ((3, 4), (3, 4)) for pair in observed))
        self.topology(obj, (5, 6))

    def test_axes_owned_recipes_regrid_in_the_current_rotated_sheared_chart(self):
        for method in ('get_graph', 'get_parametric_surface'):
            with self.subTest(method=method):
                axes = m.ThreeDAxes(x_range=(-2, 2), y_range=(-2, 2), z_range=(-2, 2))
                state = [0.]
                xyz = lambda u, v: (u, v, u*u - v*v + state[0])
                fn = (lambda u, v: xyz(u, v)[2]) if method == 'get_graph' else xyz
                obj = getattr(axes, method)(fn, u_range=(-1, 1), v_range=(-1, 1), resolution=(2, 3))
                recipe = obj.passed_uv_func
                axes.apply_matrix([[1, .5, 0], [0, 2, .25], [.5, 0, -1]], about_point=m.ORIGIN)
                axes.rotate(.4, axis=m.RIGHT).shift((.5, -1, 2))
                state[0] = .75
                obj.set_resolution((5, 7))
                expected = [axes.c2p(*xyz(u, v)) for u in np.linspace(-1, 1, 5)
                            for v in np.linspace(-1, 1, 7)]
                np.testing.assert_allclose(obj.get_points(), expected, atol=3e-6, rtol=0)
                self.topology(obj, (5, 7))
                self.assertIs(obj.passed_uv_func, recipe)

    def test_chart_edits_during_sampling_refuse_the_candidate_not_the_edit(self):
        axes = m.ThreeDAxes(x_range=(-2, 2), y_range=(-2, 2), z_range=(-2, 2))
        edit = [False]
        def fn(u, v):
            if edit[0]:
                edit[0] = False
                axes.shift((.5, 0, 0))
            return u, v, u*v
        obj = axes.get_parametric_surface(fn, resolution=(3, 4))
        before = obj.data.copy()
        origin = axes.c2p(0, 0, 0).copy()
        edit[0] = True
        with self.assertRaisesRegex(RuntimeError, 'axes changed during sampling'):
            obj.set_resolution((5, 7))
        self.topology(obj, (3, 4))
        np.testing.assert_array_equal(obj.data, before)
        np.testing.assert_allclose(axes.c2p(0, 0, 0), origin + (.5, 0, 0), atol=1e-7)
        obj.set_resolution((5, 7))
        self.topology(obj, (5, 7))

    def test_failure_and_cancellation_preserve_original_exception_and_old_grid(self):
        for error in (ValueError('sample failed'), KeyboardInterrupt('cancel')):
            with self.subTest(error=type(error).__name__):
                obj = surface(); before = obj.data.copy(); calls = []
                def fail(u, v):
                    calls.append((u, v))
                    raise error
                obj.passed_uv_func = fail
                with self.assertRaises(type(error)) as caught:
                    obj.set_resolution((7, 8))
                self.assertIs(caught.exception, error)
                self.assertEqual(len(calls), 1)
                self.topology(obj, (3, 4))
                np.testing.assert_array_equal(obj.data, before)
                obj.passed_uv_func = lambda u, v: (u, v, 1)
                obj.set_resolution((5, 6))
                self.topology(obj, (5, 6))

    def test_reentrant_refresh_refuses_and_recovers(self):
        obj = surface(); before = obj.data.copy()
        def recipe(u, v):
            obj.set_resolution((2, 2))
            return u, v, 0
        obj.passed_uv_func = recipe
        with self.assertRaisesRegex(RuntimeError, 'already in progress'):
            obj.set_resolution((7, 8))
        self.topology(obj, (3, 4))
        np.testing.assert_array_equal(obj.data, before)
        obj.passed_uv_func = lambda u, v: (u, v, 0)
        obj.set_resolution((2, 2))
        self.topology(obj, (2, 2))

    def test_authored_record_edit_is_never_overwritten(self):
        obj = surface()
        def recipe(u, v):
            obj.data['rgba'][0] = (.2, .3, .4, .5)
            return u, v, 2
        obj.passed_uv_func = recipe
        with self.assertRaisesRegex(RuntimeError, 'changed during regeneration'):
            obj.set_resolution((7, 8))
        self.topology(obj, (3, 4))
        np.testing.assert_allclose(obj.data['rgba'][0], (.2, .3, .4, .5), atol=1e-7)
        np.testing.assert_allclose(obj.get_points(), analytic((3, 4)), atol=2e-6)

    def test_authored_binding_or_family_edits_refuse_without_undoing_side_effects(self):
        for change in ('binding', 'child'):
            obj, scene, child = surface(), m.Scene(), m.Dot()
            def recipe(u, v):
                if change == 'binding':
                    scene.add(obj)
                else:
                    obj.add(child)
                return u, v, 2
            obj.passed_uv_func = recipe
            with self.assertRaisesRegex(RuntimeError, 'changed during regeneration'):
                obj.set_resolution((7, 8))
            self.topology(obj, (3, 4))
            if change == 'binding':
                self.assertIs(obj._scene, scene)
            else:
                self.assertIn(child, obj.submobjects)

    def test_invalid_shapes_are_bounded_before_uv_callbacks(self):
        obj = surface(); before = obj.data.copy(); calls = []
        obj.passed_uv_func = lambda u, v: calls.append((u, v)) or (u, v, 0)
        for shape in ((0, 3), (1, 3), (-1, 4), (513, 513), (3.5, 4), (2,), (2, 3, 4), itertools.repeat(2)):
            with self.subTest(shape=shape):
                with self.assertRaises((TypeError, ValueError)):
                    obj.set_resolution(shape)
                self.topology(obj, (3, 4))
                np.testing.assert_array_equal(obj.data, before)
        self.assertEqual(calls, [])

    def test_active_animation_and_builder_resolution_changes_refuse(self):
        obj = surface(); before = obj.data.copy()
        obj.lock_data(['point'])
        with self.assertRaisesRegex(RuntimeError, 'active animation'):
            obj.set_resolution((5, 6))
        obj.unlock_data()
        with self.assertRaisesRegex(Exception, 'resolution changes are discrete'):
            obj.animate.set_resolution((5, 6))
        np.testing.assert_array_equal(obj.data, before)
        self.topology(obj, (3, 4))

    def test_copies_regrid_independently_and_can_still_transform(self):
        obj = surface()
        for other in (obj.copy(), copy.deepcopy(obj)):
            other.set_resolution((5, 6))
            self.topology(obj, (3, 4))
            self.topology(other, (5, 6))
        scene = m.Scene(); scene.add(obj)
        obj.set_resolution((5, 6))
        scene.play(obj.animate.shift((1, 0, 0)), run_time=.1, rate_func=m.linear)
        np.testing.assert_allclose(obj.get_points(), analytic((5, 6)) + (1, 0, 0), atol=2e-6)

    def test_copies_taken_during_sampling_do_not_inherit_a_permanent_operation_lock(self):
        obj = surface()
        copies = []
        def recipe(u, v):
            if not copies:
                copies.extend((obj.copy(), copy.deepcopy(obj)))
                obj.save_state()
                copies.append(obj.saved_state)
            return u, v, .5
        obj.passed_uv_func = recipe
        obj.set_resolution((7, 5))
        self.assertEqual(len(copies), 3)
        for clone in copies:
            clone.passed_uv_func = lambda u, v: (u, v, 2)
            clone.init_points()
            clone.set_resolution((4, 3))
            self.topology(clone, (4, 3))
            np.testing.assert_array_equal(clone.get_points()[:, 2], 2.)
        self.topology(obj, (7, 5))
        np.testing.assert_array_equal(obj.get_points()[:, 2], .5)

    def test_textures_keep_pixels_and_uvs_while_following_regridded_source(self):
        source = surface((2, 3))
        light = np.full((2, 2, 4), (255, 0, 0, 255), dtype=np.uint8)
        dark = np.full((2, 2, 4), (0, 0, 255, 255), dtype=np.uint8)
        obj = m.TexturedSurface(source, light, dark)
        p = obj.get_points().copy()
        obj.data['im_coords'][:] = (p[:, :2]+1)/2
        before = obj.data.copy()
        with self.assertRaisesRegex(ValueError, 'matching native UV topology'):
            obj.set_resolution((5, 7))
        np.testing.assert_array_equal(obj.data, before)
        self.topology(obj, (2, 3))
        source.set_resolution((5, 7))
        obj.set_resolution((5, 7))
        self.topology(obj, (5, 7))
        np.testing.assert_allclose(obj.data['im_coords'], (analytic((5, 7))[:, :2]+1)/2, atol=1e-7)
        np.testing.assert_array_equal(obj.get_pixel_array(), light)
        np.testing.assert_array_equal(obj.get_pixel_array(dark=True), dark)
        self.assertIs(obj.uv_surface, source)
        self.assertEqual(obj.num_textures, 2)

    def test_live_wireframe_follows_changed_source_grid_without_replacing_wires(self):
        obj = surface((3, 4))
        mesh = m.SurfaceMesh(obj, resolution=(3, 4))
        wires = tuple(mesh.submobjects)
        scene = m.Scene(); scene.add(obj, mesh)
        obj.set_resolution((7, 9)); mesh.init_points()
        self.assertEqual(tuple(mesh.submobjects), wires)
        reference = m.SurfaceMesh(surface((7, 9)), resolution=(3, 4))
        for a, b in zip(mesh, reference):
            np.testing.assert_array_equal(a.get_points(), b.get_points())
        self.assertEqual(tuple(scene.mobjects), (obj, mesh))

    def test_raw_bridge_keeps_source_and_destination_material_owners(self):
        target, source = surface((2, 3), color=m.RED), surface((5, 4), 2, color=m.BLUE)
        source.shift((1, 2, 3))
        source_before = source.data.copy()
        old_color = target.data['rgba'][0].copy()
        m._regrid_surface_geometry(target, source)
        self.assertEqual(m._surface_grid_resolution(target), (5, 4))
        np.testing.assert_array_equal(target.get_points(), source.get_points())
        np.testing.assert_array_equal(target.data['rgba'], np.tile(old_color, (20, 1)))
        np.testing.assert_array_equal(source.data, source_before)

    def test_raw_bridge_rejects_invalid_source_without_publishing(self):
        target, source = surface(), surface((5, 6))
        before = target.data.copy()
        source.data['d_normal_point'][0, 0] = np.nan
        with self.assertRaises(ValueError):
            m._regrid_surface_geometry(target, source)
        np.testing.assert_array_equal(target.data, before)
        self.topology(target, (3, 4))

    def test_frame_output_matches_independent_geometry_at_one_and_four_threads(self):
        class Regrid(m.Scene):
            manual = False
            def construct(self):
                tracker = m.ValueTracker(0)
                obj = surface((3, 4), color=m.BLUE)
                self.add(obj)
                def update(s, dt):
                    alpha = tracker.get_value()
                    shape = (3, 4) if alpha < .5 else (7, 5)
                    amp = 1-alpha
                    if self.manual:
                        candidate = surface(shape, 0, color=m.BLUE)
                        candidate.set_points(analytic(shape, amp))
                        s.become(candidate)
                    else:
                        s.passed_uv_func = lambda u, v: (u, v, amp*(u*u-v*v))
                        s.set_resolution(shape)
                obj.add_updater(update, call=False)
                self.play(tracker.animate.set_value(1), run_time=.5, rate_func=m.linear)
        class Reference(Regrid):
            manual = True
        with tempfile.TemporaryDirectory(prefix='fmn-regrid-') as directory:
            outputs = []
            for cls, threads in ((Regrid, 1), (Regrid, 4), (Reference, 1)):
                path = Path(directory) / f'{cls.__name__}-{threads}.y4m'
                receipt = cls().render(path, format='y4m', resolution=(96, 54), fps=8, threads=threads)
                self.assertEqual(receipt.frame_count, 4)
                outputs.append(path.read_bytes())
            self.assertEqual(outputs[0], outputs[1])
            self.assertEqual(outputs[0], outputs[2])


def run_surface_regridding_acceptance():
    stream = io.StringIO()
    result = unittest.TextTestRunner(stream=stream, verbosity=2).run(
        unittest.defaultTestLoader.loadTestsFromTestCase(SurfaceRegriddingTests))
    if not result.wasSuccessful():
        raise AssertionError(stream.getvalue())
    print(stream.getvalue())


if __name__ == '__main__':
    run_surface_regridding_acceptance()
