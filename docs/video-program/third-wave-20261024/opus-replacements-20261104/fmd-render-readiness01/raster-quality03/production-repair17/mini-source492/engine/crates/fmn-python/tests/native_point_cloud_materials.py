"""Live public glow edits against native record storage and actual Lumen output."""
from pathlib import Path
import copy
import gc
import tempfile
import unittest
import weakref

import numpy as np
import manimlib as m
from fmn_python import render_session


def render(path, cloud, threads=1):
    scene = m.Scene()
    with render_session(scene, path, resolution=(64, 40), fps=8, threads=threads) as session:
        scene.add(cloud)
        scene.wait(.375)
    assert session.result.frame_count == 3
    return path.read_bytes()


def cloud(glow=0., points=None):
    return m.DotCloud([[0, 0, 0]] if points is None else points,
                      color=m.BLUE, radius=.8, opacity=.7, glow_factor=glow)


class PointCloudMaterialTests(unittest.TestCase):
    def test_direct_uniform_assignment_updates_native_material_only(self):
        dots = cloud(points=[[-1, 0, 0], [1, 0, 0]])
        before = dots.data.copy()
        dots.uniforms['glow_factor'] = 2.
        self.assertEqual(dots.get_glow_factor(), 2.)
        np.testing.assert_array_equal(dots.data['glow_factor'], [[2.], [2.]])
        for name in before.dtype.names:
            if name != 'glow_factor':
                self.assertEqual(dots.data[name].tobytes(), before[name].tobytes())

    def test_mapping_updates_and_public_uniform_setters_share_the_native_path(self):
        dots = cloud()
        for apply in (lambda: dots.uniforms.update(glow_factor=3.),
                      lambda: dots.set_uniform(glow_factor=3.),
                      lambda: dots.set_uniforms({'glow_factor': 3.})):
            dots.set_glow_factor(0.)
            apply()
            np.testing.assert_array_equal(dots.data['glow_factor'], [[3.]])

    def test_public_method_returns_self_and_updates_legacy_attribute(self):
        dots = cloud()
        self.assertIs(dots.set_glow_factor(4.), dots)
        self.assertEqual(dots.glow_factor, 4.)
        self.assertEqual(dots.get_glow_factor(), 4.)
        np.testing.assert_array_equal(dots.data['glow_factor'], [[4.]])

    def test_empty_default_survives_append_copy_and_deepcopy(self):
        for edit in (lambda d: d.set_glow_factor(3.),
                     lambda d: d.uniforms.update(glow_factor=3.)):
            dots = cloud(points=[])
            edit(dots)
            for value in (dots, dots.copy(), copy.deepcopy(dots)):
                with self.subTest(copy=value is not dots):
                    self.assertEqual(value.get_num_points(), 0)
                    value.add_points([[0, 0, 0]])
                    self.assertEqual(value.get_glow_factor(), 3.)
                    np.testing.assert_array_equal(value.data['glow_factor'], [[3.]])

    def test_cleared_cloud_retains_the_edited_material(self):
        dots = cloud(glow=1.)
        dots.clear_points()
        dots.uniforms['glow_factor'] = 4.
        dots.set_points([[-1, 0, 0], [1, 0, 0]])
        np.testing.assert_array_equal(dots.data['glow_factor'], [[4.], [4.]])

    def test_copies_do_not_alias_native_records_or_empty_defaults(self):
        for points in ([], [[0, 0, 0]]):
            dots = cloud(points=points)
            dots.uniforms['glow_factor'] = 2.
            duplicate = dots.copy()
            duplicate.uniforms['glow_factor'] = 5.
            self.assertEqual(dots.get_glow_factor(), 2.)
            np.testing.assert_array_equal(dots._style_data()['glow_factor'], 2.)
            np.testing.assert_array_equal(duplicate._style_data()['glow_factor'], 5.)

    def test_invalid_glow_refuses_before_mapping_records_or_attributes_change(self):
        for points in ([], [[0, 0, 0]]):
            dots = cloud(points=points)
            for value in (-1., float('nan'), float('inf'), -float('inf'), 1e40):
                for method in (False, True):
                    with self.subTest(points=points, value=value, method=method):
                        before = dots._style_data().tobytes(), dict(dots.uniforms), dots.glow_factor
                        with self.assertRaises(ValueError):
                            if method:
                                dots.set_glow_factor(value)
                            else:
                                dots.uniforms['glow_factor'] = value
                        self.assertEqual((dots._style_data().tobytes(), dict(dots.uniforms), dots.glow_factor), before)

    def test_other_mobject_uniforms_and_cloud_extras_are_not_reinterpreted(self):
        shape = m.Circle()
        shape.uniforms['glow_factor'] = 'authored payload'
        self.assertEqual(shape.uniforms['glow_factor'], 'authored payload')
        dots = cloud()
        payload = object()
        dots.uniforms['custom'] = payload
        dots.uniforms['anti_alias_width'] = 3.
        self.assertIs(dots.uniforms['custom'], payload)
        self.assertEqual(dots.uniforms['anti_alias_width'], 3.)

    def test_readonly_field_refusal_leaves_uniform_and_attribute_unchanged(self):
        dots = cloud()
        frozen = dots.data.copy()
        frozen.flags.writeable = False
        dots._style_data = lambda: frozen
        before = dict(dots.uniforms), dots.glow_factor
        with self.assertRaisesRegex(ValueError, 'writable'):
            dots.set_glow_factor(3.)
        self.assertEqual((dict(dots.uniforms), dots.glow_factor), before)
        np.testing.assert_array_equal(dots.data['glow_factor'], [[0.]])

    def test_malformed_field_refuses_without_accepting_a_python_only_uniform(self):
        dots = cloud()
        for dtype in ([('point', np.float32, (3,))],
                      [('glow_factor', np.float64, (1,))],
                      [('glow_factor', np.float32, (2,))]):
            fake = np.zeros(1, dtype=dtype)
            dots._style_data = lambda: fake
            with self.assertRaises(ValueError):
                dots.uniforms['glow_factor'] = 3.
            self.assertEqual(dots.get_glow_factor(), 0.)
        np.testing.assert_array_equal(dots.data['glow_factor'], [[0.]])

    def test_uniform_mapping_keeps_weak_owner_lifetime(self):
        dots = cloud()
        table = dots.uniforms
        ref = weakref.ref(dots)
        dots.uniforms['glow_factor'] = 3.
        del dots
        gc.collect()
        self.assertIsNone(ref())
        with self.assertRaises(ReferenceError):
            table['glow_factor'] = 1.

    def test_record_varying_glow_remains_varying_until_uniform_assignment(self):
        dots = cloud(points=[[-1, 0, 0], [1, 0, 0]])
        dots.data['glow_factor'][:] = [[1.], [3.]]
        duplicate = dots.copy()
        np.testing.assert_array_equal(duplicate.data['glow_factor'], [[1.], [3.]])
        duplicate.uniforms['glow_factor'] = 2.
        np.testing.assert_array_equal(duplicate.data['glow_factor'], [[2.], [2.]])
        np.testing.assert_array_equal(dots.data['glow_factor'], [[1.], [3.]])

    def test_record_interpolation_preserves_per_point_glow_and_scalar_projection(self):
        start = cloud(glow=1., points=[[-1, 0, 0], [1, 0, 0]])
        end = cloud(glow=3., points=[[-1, 1, 0], [1, 1, 0]])
        start.data['glow_factor'][:] = [[1.], [2.]]
        end.data['glow_factor'][:] = [[3.], [6.]]
        result = start.copy()
        result.interpolate(start, end, .5)
        np.testing.assert_array_equal(result.data['glow_factor'], [[2.], [4.]])
        self.assertEqual(result.uniforms['glow_factor'], 2.)
        np.testing.assert_array_equal(start.data['glow_factor'], [[1.], [2.]])
        self.assertEqual(start.uniforms['glow_factor'], 1.)
        result.uniforms['glow_factor'] = 2.
        np.testing.assert_array_equal(result.data['glow_factor'], [[2.], [2.]])

    def test_interpolation_locks_and_authored_paths_remain_live(self):
        start, end = cloud(glow=1.), cloud(glow=3.)
        result, other = cloud(glow=0.), cloud(glow=0.)
        result.locked_data_keys = {'glow_factor'}
        result.locked_uniform_keys = {'glow_factor'}
        def path(a, b, alpha):
            self.assertIn('glow_factor', start.uniforms)
            self.assertIn('glow_factor', result.uniforms)
            other.uniforms['glow_factor'] = 4.
            return (1 - alpha) * a + alpha * b
        result.interpolate(start, end, .5, path)
        np.testing.assert_array_equal(result.data['glow_factor'], [[0.]])
        self.assertEqual(result.get_glow_factor(), 0.)
        np.testing.assert_array_equal(other.data['glow_factor'], [[4.]])

    def test_stock_transform_keeps_native_dispatch(self):
        dots = cloud()
        animation = m.Transform(dots, cloud(glow=3.))
        native = getattr(m, '_native', m)
        self.assertFalse(native._requires_python_animation(animation))

    def test_callback_transform_does_not_flatten_heterogeneous_glow(self):
        start = cloud(glow=1., points=[[-1, 0, 0], [1, 0, 0]])
        end = cloud(glow=3., points=[[-1, 1, 0], [1, 1, 0]])
        start.data['glow_factor'][:] = [[1.], [2.]]
        end.data['glow_factor'][:] = [[3.], [6.]]
        scene = m.Scene()
        with tempfile.TemporaryDirectory() as tmp:
            with render_session(scene, Path(tmp) / 'animated.y4m', resolution=(64, 40), fps=8) as session:
                scene.add(start)
                scene.play(m.Transform(start, end), run_time=.375, rate_func=lambda t: t)
            self.assertEqual(session.result.frame_count, 3)
        np.testing.assert_array_equal(start.data['glow_factor'], [[3.], [6.]])
        self.assertEqual(start.uniforms['glow_factor'], 3.)

    def test_authored_path_exception_keeps_original_error_and_live_maps(self):
        start, end, result = cloud(glow=1.), cloud(glow=3.), cloud(glow=0.)
        error = RuntimeError('authored path refused')
        def path(a, b, alpha):
            self.assertIn('glow_factor', start.uniforms)
            result.uniforms['glow_factor'] = 4.
            raise error
        with self.assertRaises(RuntimeError) as caught:
            result.interpolate(start, end, .5, path)
        self.assertIs(caught.exception, error)
        self.assertEqual(result.get_glow_factor(), 4.)
        np.testing.assert_array_equal(result.data['glow_factor'], [[4.]])
        self.assertEqual(start.get_glow_factor(), 1.)

    def test_direct_uniform_renders_as_independent_constructor_at_all_worker_counts(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            expected = render(root / 'expected.y4m', cloud(glow=2.))
            negative = render(root / 'negative.y4m', cloud(glow=0.))
            self.assertNotEqual(expected, negative)
            for threads in (1, 4, 16):
                dots = cloud()
                dots.uniforms['glow_factor'] = 2.
                actual = render(root / f'actual-{threads}.y4m', dots, threads)
                self.assertEqual(actual, expected)

    def test_live_edits_invalidate_retained_frames_without_rebuilding_geometry(self):
        def sequence(path, setter, threads=1):
            scene, dots = m.Scene(), cloud()
            with render_session(scene, path, resolution=(64, 40), fps=8, threads=threads) as session:
                scene.add(dots)
                for glow in (0., 2., 4.):
                    setter(dots, glow)
                    scene.wait(.125)
            self.assertEqual(session.result.frame_count, 3)
            return path.read_bytes()
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            expected = sequence(root / 'expected.y4m', lambda d, v: d.data['glow_factor'].__setitem__(slice(None), v))
            negative = sequence(root / 'negative.y4m', lambda d, v: None)
            self.assertNotEqual(expected, negative)
            for threads in (1, 4, 16):
                actual = sequence(root / f'actual-{threads}.y4m', lambda d, v: d.uniforms.__setitem__('glow_factor', v), threads)
                self.assertEqual(actual, expected)


def run_native_point_cloud_materials():
    result = unittest.TextTestRunner(verbosity=2).run(
        unittest.defaultTestLoader.loadTestsFromTestCase(PointCloudMaterialTests))
    if not result.wasSuccessful():
        raise AssertionError('native point-cloud material acceptance failed')


if __name__ == '__main__':
    run_native_point_cloud_materials()
