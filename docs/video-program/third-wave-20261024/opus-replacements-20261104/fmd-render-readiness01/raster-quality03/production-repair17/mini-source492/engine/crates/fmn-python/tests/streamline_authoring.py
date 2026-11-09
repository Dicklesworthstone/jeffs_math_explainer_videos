"""Real native flow geometry, paint, coordinate callbacks and rendering.

No integrator, record storage or render doubles. This suite requires the built
portal and is also suitable for an installed wheel outside the source tree.
"""
from __future__ import annotations

import gc
import unittest

import manimlib as m
import numpy as np
from manimlib.mobject.vector_field import StreamLines as QualifiedStreamLines


class StreamlineAuthoringTests(unittest.TestCase):
    def lines(self, function=None, **kwargs):
        axes = m.Axes(x_range=(-1, 1, 1), y_range=(-1, 1, 1), width=2, height=2)
        config = dict(noise_factor=0, solution_time=.2, dt=.05, arc_len=1,
                      n_samples_per_line=4, stroke_width=8)
        config.update(kwargs)
        function = function or (lambda xs: np.tile([1., 0.], (len(xs), 1)))
        return m.StreamLines(function, axes, **config)

    def test_two_dimensional_callback_keeps_public_identity_and_configuration(self):
        calls = []
        def field(xs):
            self.assertEqual(xs.shape[1], 2)
            calls.append(xs.copy())
            return np.tile([1., 0.], (len(xs), 1))
        lines = self.lines(field)
        self.assertTrue(calls)
        self.assertIs(lines.func, field)
        self.assertIs(m.StreamLines, QualifiedStreamLines)
        self.assertEqual(lines.n_repeats, 1)
        self.assertEqual(lines.dt, .05)
        self.assertEqual(lines.n_samples_per_line, 4)
        self.assertEqual(lines.noise_factor, 0)
        for line, duration in zip(lines, lines._stream_virtual_times):
            self.assertEqual(line.virtual_time, duration)

    def test_native_magnitude_colors_and_taper_reach_live_records(self):
        lines = self.lines(taper_stroke_width=True, stroke_opacity=.4)
        expected = m.get_vectorized_rgb_gradient_function(0, 2, "3b1b_colormap")([1.])[0]
        for line in lines:
            np.testing.assert_allclose(line.data["stroke_rgba"][:, :3],
                                       np.tile(expected, (line.get_num_points(), 1)))
            np.testing.assert_allclose(line.get_stroke_opacities(), .4)
            widths = line.get_stroke_widths()
            self.assertEqual(widths[0], 0)
            self.assertEqual(widths[-1], 0)
            self.assertGreater(max(widths), 7.9)

    def test_taper_measures_unequal_curves_instead_of_record_indices(self):
        lines = self.lines(taper_stroke_width=True, color_by_magnitude=False)
        line = lines[0]
        line.set_points_as_corners([[0., 0., 0.], [1., 0., 0.], [4., 0., 0.]])
        lines.init_style()
        np.testing.assert_allclose(line.get_stroke_widths(), [0., 2., 4., 6., 0.], atol=1e-6)

    def test_restyle_preserves_identity_geometry_and_live_views(self):
        lines = self.lines()
        scene = m.Scene()
        scene.add(lines)
        children, views = tuple(lines), [line.get_points() for line in lines]
        points = [view.copy() for view in views]
        paints = [line.data["stroke_rgba"] for line in lines]
        lines.func = lambda xs: np.tile([2., 0.], (len(xs), 1))
        lines.stroke_opacity = .25
        self.assertIsNone(lines.init_style())
        self.assertIs(scene.mobjects[0], lines)
        self.assertEqual(tuple(lines), children)
        for line, view, original, paint in zip(lines, views, points, paints):
            np.testing.assert_array_equal(view, original)
            np.testing.assert_array_equal(line.get_points(), original)
            np.testing.assert_allclose(paint[:, 3], .25)
        views[0][0] += .125
        np.testing.assert_array_equal(lines[0].get_points()[0], views[0][0])

    def test_late_callback_failure_does_not_partly_recolor_the_family(self):
        lines = self.lines()
        saved = [line.data.copy() for line in lines]
        calls = []
        def field(xs):
            calls.append(len(xs))
            if len(calls) == 2:
                raise LookupError("late field failure")
            return np.tile([2., 0.], (len(xs), 1))
        lines.func = field
        with self.assertRaisesRegex(LookupError, "late field failure"):
            lines.init_style()
        for line, original in zip(lines, saved):
            np.testing.assert_array_equal(line.data, original)
        lines.func = lambda xs: np.tile([2., 0.], (len(xs), 1))
        lines.init_style()
        self.assertFalse(np.array_equal(lines[0].data, saved[0]))

    def test_malformed_nonfinite_and_overflowing_style_is_rejected_before_write(self):
        lines = self.lines()
        original_func = lines.func
        bad_fields = [lambda xs: np.zeros((len(xs) + 1, 2)),
                      lambda xs: np.full((len(xs), 2), np.nan),
                      lambda xs: np.full((len(xs), 2), 1j),
                      lambda xs: np.full((len(xs), 2), 1e300)]
        for field in bad_fields:
            saved = [line.data.copy() for line in lines]
            lines.func = field
            with self.assertRaises((TypeError, ValueError)):
                lines.init_style()
            for line, original in zip(lines, saved):
                np.testing.assert_array_equal(line.data, original)
        lines.func = original_func
        for key, value in (("stroke_width", -1), ("stroke_width", 1e100),
                           ("stroke_opacity", 2), ("magnitude_range", (3, 1))):
            old = getattr(lines, key)
            saved = [line.data.copy() for line in lines]
            setattr(lines, key, value)
            with self.assertRaises(ValueError):
                lines.init_style()
            for line, original in zip(lines, saved):
                np.testing.assert_array_equal(line.data, original)
            setattr(lines, key, old)

    def test_recursive_style_callback_is_refused_and_next_edit_can_succeed(self):
        lines = self.lines()
        old_func = lines.func
        def field(xs):
            lines.init_style()
            return xs
        lines.func = field
        with self.assertRaisesRegex(RuntimeError, "reenter"):
            lines.init_style()
        lines.func = old_func
        lines.init_style()

    def test_callback_geometry_mutation_invalidates_the_plan_before_paint(self):
        lines = self.lines()
        paints = [line.data["stroke_rgba"].copy() for line in lines]
        def field(xs):
            lines[0].shift(m.RIGHT)
            return np.tile([2., 0.], (len(xs), 1))
        lines.func = field
        with self.assertRaisesRegex(RuntimeError, "geometry changed"):
            lines.init_style()
        for line, old in zip(lines, paints):
            np.testing.assert_array_equal(line.data["stroke_rgba"], old)

    def test_point_func_uses_live_transformed_axes_and_accepts_single_or_batch(self):
        lines = self.lines()
        axes = lines.coordinate_system
        axes.rotate(m.PI / 2).shift(2 * m.RIGHT).stretch(1.5, 1)
        expected = axes.c2p(1., 0.) - axes.get_origin()
        point = axes.c2p(0., 0.)
        np.testing.assert_allclose(lines.point_func(point), expected, atol=1e-6)
        np.testing.assert_allclose(lines.point_func([point, point + m.UP]),
                                   np.tile(expected, (2, 1)), atol=1e-6)
        self.assertEqual(lines.point_func(np.empty((0, 3))).shape, (0, 3))

    def test_three_dimensional_fields_keep_the_third_coordinate(self):
        axes = m.ThreeDAxes(x_range=(-1, 1, 2), y_range=(-1, 1, 2), z_range=(-1, 1, 2))
        def field(xs):
            self.assertEqual(xs.shape[1], 3)
            return np.tile([0., 0., 1.], (len(xs), 1))
        lines = m.StreamLines(field, axes, noise_factor=0, solution_time=.1,
                              dt=.05, n_samples_per_line=3, color_by_magnitude=False)
        vector = axes.c2p(0., 0., 1.) - axes.get_origin()
        np.testing.assert_allclose(lines.point_func([axes.get_origin()]), [vector])

    def test_uniform_style_never_calls_field_and_constructor_shorthand_wins(self):
        lines = self.lines(color_by_magnitude=False, stroke_color=m.RED, color=m.BLUE)
        np.testing.assert_allclose(lines[0].data["stroke_rgba"][:, :3],
                                   np.tile(m.color_to_rgb(m.BLUE), (lines[0].get_num_points(), 1)))
        def forbidden(xs):
            raise AssertionError("uniform restyling must not evaluate the field")
        lines.func = forbidden
        lines.stroke_color = m.GREEN
        lines.init_style()
        np.testing.assert_allclose(lines[0].data["stroke_rgba"][:, :3],
                                   np.tile(m.color_to_rgb(m.GREEN), (lines[0].get_num_points(), 1)))

    def test_empty_and_stationary_paths_have_finite_zero_tapers(self):
        lines = self.lines(lambda xs: np.zeros_like(xs), taper_stroke_width=True)
        lines[0].clear_points()
        lines.init_style()
        self.assertEqual(lines[0].get_points().shape, (0, 3))
        for line in list(lines)[1:]:
            np.testing.assert_array_equal(line.get_stroke_widths(), np.zeros(line.get_num_points()))
            self.assertTrue(np.isfinite(line.data["stroke_rgba"]).all())

    def test_animation_cleanup_restores_the_new_taper_and_keeps_colors(self):
        lines = self.lines(taper_stroke_width=True)
        widths = [line.get_stroke_widths().copy() for line in lines]
        colors = [line.data["stroke_rgba"].copy() for line in lines]
        flow = m.AnimatedStreamLines(lines, lag_range=0)
        flow.update(.07)
        self.assertTrue(any(not np.array_equal(line.get_stroke_widths(), old)
                            for line, old in zip(lines, widths)))
        flow.clear_updaters()
        for line, width, color in zip(lines, widths, colors):
            np.testing.assert_array_equal(line.get_stroke_widths(), width)
            np.testing.assert_array_equal(line.data["stroke_rgba"], color)
            self.assertFalse(line._is_updating_suspended())

    def test_native_pixels_change_with_magnitude_but_repeat_identically(self):
        lines = self.lines(stroke_width=16)
        camera = m.Camera()
        camera.reset_pixel_shape(160, 90)
        first = camera.capture_snapshot(lines).pixels()
        self.assertEqual(first, camera.capture_snapshot(lines).pixels())
        lines.func = lambda xs: np.tile([2., 0.], (len(xs), 1))
        lines.init_style()
        second = camera.capture_snapshot(lines).pixels()
        self.assertNotEqual(first, second)
        self.assertEqual(second, camera.capture_snapshot(lines).pixels())

    def test_subclass_style_hook_is_dispatched_on_the_original_identity(self):
        calls = []
        class Custom(m.StreamLines):
            def init_style(self):
                calls.append(self)
                return super().init_style()
        axes = m.Axes(x_range=(-1, 1, 1), y_range=(-1, 1, 1))
        lines = Custom(lambda xs: np.zeros_like(xs), axes, solution_time=.1, dt=.05)
        self.assertEqual(calls, [lines])
        lines.init_style()
        self.assertEqual(calls, [lines, lines])


suite = unittest.defaultTestLoader.loadTestsFromTestCase(StreamlineAuthoringTests)
assert suite.countTestCases() == 15, "native streamline-authoring inventory changed"
result = unittest.TextTestRunner(verbosity=2).run(suite)
gc.collect()
if not result.wasSuccessful():
    raise AssertionError("native streamline-authoring acceptance failed")
