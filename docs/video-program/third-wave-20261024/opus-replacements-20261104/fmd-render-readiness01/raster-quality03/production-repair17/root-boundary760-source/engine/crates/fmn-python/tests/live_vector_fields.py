"""Actual installed-portal field geometry, updater and render acceptance.

No geometry or native publication substitutes. Missing manimlib is an error.
Registered in scripts/check_portal_runtime.sh.
"""
from __future__ import annotations

import unittest

import manimlib as m
import numpy as np


class LiveVectorFieldAcceptance(unittest.TestCase):
    def field(self, function=None, **kwargs):
        axes = m.Axes(x_range=(-2, 2, 1), y_range=(-2, 2, 1), width=4, height=4)
        values = dict(sample_coords=np.array([[-1., 0.], [1., 0.]]),
                      max_vect_len=1., color=m.WHITE)
        values.update(kwargs)
        field = m.VectorField(function or (lambda xs: np.tile([1., 0.], (len(xs), 1))), axes, **values)
        return axes, field

    def test_bound_resampling_and_live_axes_use_real_native_arrow_geometry(self):
        axes, field = self.field()
        scene = m.Scene()
        scene.add(field)
        root = field
        samples = np.array([[-1., -1.], [0., 0.], [1., 1.]])
        field.set_sample_coords(samples).update_vectors()
        self.assertIs(scene.mobjects[0], root)
        self.assertEqual(field.get_num_points(), 23)
        np.testing.assert_allclose(field.get_points()[0::8], axes.c2p(*samples.T), atol=2e-5)
        origin = axes.c2p(0., 0.)
        vector = axes.c2p(1., 0.) - origin
        expected = field.max_displayed_vect_len * np.tanh(np.linalg.norm(vector) / field.max_displayed_vect_len)
        np.testing.assert_allclose(np.linalg.norm(field.get_points()[6::8] - field.get_points()[0::8], axis=1),
                                   expected, atol=2e-5)
        axes.rotate(m.PI / 2).shift(m.RIGHT)
        field.update_vectors()
        np.testing.assert_allclose(field.get_points()[0::8], axes.c2p(*samples.T), atol=2e-5)
        vector = axes.c2p(1., 0.) - axes.c2p(0., 0.)
        expected_vectors = np.tile(expected * vector / np.linalg.norm(vector), (3, 1))
        np.testing.assert_allclose(field.get_points()[6::8] - field.get_points()[0::8], expected_vectors, atol=2e-5)
        field.set_stroke_width(5)
        self.assertEqual(len(field.base_stroke_width_array), 23)
        np.testing.assert_allclose(np.asarray(field.get_stroke_widths()).reshape(-1)[4::8], 20)
        field.set_sample_coords([[-1., 0.], [1., 0.]]).update_vectors()
        self.assertEqual(field.get_num_points(), 15)

    def test_zero_field_and_constant_color_range_remain_finite(self):
        axes, field = self.field(lambda xs: np.zeros_like(xs), color=None)
        self.assertEqual(field.magnitude_range, (0., 0.))
        for _ in range(3):
            field.update_vectors()
            self.assertTrue(np.isfinite(field.data["stroke_rgba"]).all())
            self.assertTrue(np.isfinite(field.get_points()).all())
            np.testing.assert_array_equal(field.get_points()[6::8], field.get_points()[0::8])

    def test_style_callback_failure_preserves_native_records_and_recovers(self):
        axes, field = self.field()
        scene = m.Scene()
        scene.add(field)
        before = np.array(field.data, copy=True)
        field.color_map = lambda alphas: np.tile([1., 0., 0., 1.], (len(alphas), 1))
        def failed(norms):
            raise LookupError("opacity failure")
        field.norm_to_opacity_func = failed
        field.set_sample_coords([[0., 0.], [1., 0.], [2., 0.]])
        with self.assertRaisesRegex(LookupError, "opacity failure"):
            field.update_vectors()
        np.testing.assert_array_equal(field.data, before)
        field.norm_to_opacity_func = lambda norms: np.full((len(norms), 1), .3)
        field.update_vectors()
        self.assertEqual(field.get_num_points(), 23)
        np.testing.assert_allclose(field.data["stroke_rgba"][:, :3], np.tile([1., 0., 0.], (23, 1)))
        np.testing.assert_allclose(field.data["stroke_rgba"][:, 3], .3)

    def test_authored_helpers_dispatch_and_native_snapshot_observes_new_samples(self):
        calls = []
        class Custom(m.VectorField):
            def update_sample_points(self):
                calls.append(self)
                return super().update_sample_points()
        axes = m.Axes(x_range=(-2, 2, 1), y_range=(-2, 2, 1), width=4, height=4)
        field = Custom(lambda xs: np.tile([1., 0.], (len(xs), 1)), axes,
                       sample_coords=[[-1., 0.], [1., 0.]], max_vect_len=1., color=m.WHITE,
                       stroke_width=8.)
        camera = m.Camera()
        camera.reset_pixel_shape(160, 90)
        first = camera.capture_snapshot(field).pixels()
        field.set_sample_coords([[-1., 1.], [0., 1.], [1., 1.]]).update_vectors()
        second = camera.capture_snapshot(field).pixels()
        self.assertNotEqual(first, second)
        self.assertTrue(all(receiver is field for receiver in calls))
        self.assertGreaterEqual(len(calls), 2)


suite = unittest.defaultTestLoader.loadTestsFromTestCase(LiveVectorFieldAcceptance)
if not suite.countTestCases():
    raise AssertionError("no native vector-field acceptance tests selected")
result = unittest.TextTestRunner(verbosity=2).run(suite)
if not result.wasSuccessful():
    raise AssertionError("native vector-field acceptance failed")
