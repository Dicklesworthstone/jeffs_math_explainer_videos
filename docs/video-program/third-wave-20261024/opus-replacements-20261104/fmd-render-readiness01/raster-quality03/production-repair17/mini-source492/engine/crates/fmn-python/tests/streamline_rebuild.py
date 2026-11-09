"""Native live StreamLines reintegration and last-good family preservation."""
import gc
import random
import unittest

import manimlib as m
import numpy as np


def horizontal(rows):
    return np.tile([1., 0.], (len(rows), 1))


def vertical(rows):
    return np.tile([0., 1.], (len(rows), 1))


class StreamlineRebuildTests(unittest.TestCase):
    def make(self, **kw):
        axes = m.Axes(x_range=(-1, 1, 1), y_range=(-1, 1, 1), width=2, height=2)
        config = dict(solution_time=.2, dt=.05, noise_factor=0,
                      n_samples_per_line=4, stroke_width=6, taper_stroke_width=True)
        config.update(kw)
        return m.StreamLines(horizontal, axes, **config)

    def test_detached_rebuild_replaces_curves_not_container(self):
        lines = self.make()
        original = tuple(lines)
        lines.func, lines.solution_time = vertical, .4
        lines.label = 'kept'
        self.assertIsNone(lines.draw_lines())
        self.assertEqual(lines.label, 'kept')
        self.assertIs(lines.func, vertical)
        self.assertEqual(len(lines), len(original))
        self.assertTrue(all(old is not new for old, new in zip(original, lines)))
        for line in lines:
            np.testing.assert_allclose(line.get_end() - line.get_start(), [0, .35, 0], atol=1e-6)
            self.assertEqual(line.virtual_time, .4)
            self.assertEqual(line.parents, [lines])
        self.assertTrue(all(lines not in old.parents for old in original))
        self.assertEqual(lines._stream_virtual_times, [.4] * len(lines))

    def test_scene_bound_rebuild_keeps_roots_updaters_and_old_views_valid(self):
        scene, lines = m.Scene(), self.make()
        scene.add(lines)
        updater = lambda current, dt: None
        lines.add_updater(updater)
        old = tuple(lines)
        views, values = [line.get_points() for line in old], [line.get_points().copy() for line in old]
        lines.func = vertical
        lines.draw_lines()
        self.assertIs(scene.mobjects[0], lines)
        self.assertEqual(lines.updaters, [updater])
        self.assertTrue(lines._is_bound())
        self.assertTrue(all(line._is_bound() for line in lines))
        for line, view, value in zip(lines, views, values):
            np.testing.assert_array_equal(view, value)
            self.assertEqual(line.parents, [lines])
            view[:] = -20
            self.assertFalse(np.any(line.get_points() == -20))

    def test_rebuild_uses_native_jitter_without_global_python_rng(self):
        lines = self.make(noise_factor=.2, n_repeats=2)
        expected = [line.data.copy() for line in lines]
        draws = lines._stream_rng_draws
        py_state, np_state = random.getstate(), np.random.get_state()
        for _ in range(2):
            lines.draw_lines()
            self.assertEqual(lines._stream_rng_draws, draws)
            for line, data in zip(lines, expected):
                np.testing.assert_array_equal(line.data, data)
        self.assertEqual(random.getstate(), py_state)
        final = np.random.get_state()
        self.assertEqual(final[0], np_state[0])
        np.testing.assert_array_equal(final[1], np_state[1])
        self.assertEqual(final[2:], np_state[2:])

    def test_late_field_error_preserves_last_good_family_and_metadata(self):
        lines = self.make()
        scene = m.Scene()
        scene.add(lines)
        old, records = tuple(lines), [line.data.copy() for line in lines]
        times, draws = list(lines._stream_virtual_times), lines._stream_rng_draws
        calls = []
        def broken(rows):
            calls.append(len(rows))
            if len(calls) > 3:
                raise LookupError('authored field refused')
            return vertical(rows)
        lines.func = broken
        with self.assertRaisesRegex(LookupError, 'authored field refused'):
            lines.draw_lines()
        self.assertEqual(tuple(lines), old)
        self.assertEqual(lines._stream_virtual_times, times)
        self.assertEqual(lines._stream_rng_draws, draws)
        for line, original in zip(lines, records):
            np.testing.assert_array_equal(line.data, original)
        lines.func = vertical
        lines.draw_lines()
        self.assertIs(scene.mobjects[0], lines)

    def test_invalid_geometry_and_paint_controls_never_publish(self):
        lines = self.make()
        for name, bad in [('density', 1e12), ('dt', -1), ('n_repeats', 1.5),
                          ('magnitude_range', (2, 1)), ('stroke_opacity', 2),
                          ('stroke_width', 1e100)]:
            before, value = tuple(lines), getattr(lines, name)
            records = [line.data.copy() for line in lines]
            setattr(lines, name, bad)
            with self.subTest(name=name), self.assertRaises((ValueError, TypeError, OverflowError)):
                lines.draw_lines()
            self.assertEqual(tuple(lines), before)
            for line, original in zip(lines, records):
                np.testing.assert_array_equal(line.data, original)
            setattr(lines, name, value)

    def test_transformed_axes_are_reconsulted_for_native_geometry(self):
        lines = self.make()
        cs = lines.coordinate_system
        cs.rotate(m.PI / 2).shift(2 * m.RIGHT).stretch(1.5, 1)
        expected = cs.c2p(.15, 0) - cs.get_origin()
        lines.draw_lines()
        for line in lines:
            np.testing.assert_allclose(line.get_end() - line.get_start(), expected, atol=1e-6)

    def test_active_flashes_refuse_before_field_and_release_allows_rebuild(self):
        lines = self.make()
        flow = m.AnimatedStreamLines(lines, lag_range=0)
        before = tuple(lines)
        def forbidden(rows):
            self.fail('active animation must refuse before evaluating the field')
        lines.func = forbidden
        with self.assertRaisesRegex(RuntimeError, 'released animations'):
            lines.draw_lines()
        self.assertEqual(tuple(lines), before)
        flow.clear_updaters()
        self.assertTrue(flow._streamline_controller.closed)
        lines.func, lines.solution_time = vertical, .3
        lines.draw_lines()
        replacement = m.AnimatedStreamLines(lines, lag_range=0)
        try:
            replacement.update(.05)
            self.assertEqual(replacement._line_run_times, [.3] * len(lines))
        finally:
            replacement.clear_updaters()

    def test_zero_repeats_can_remove_and_restore_all_lines(self):
        lines = self.make()
        scene = m.Scene()
        scene.add(lines)
        lines.n_repeats = 0
        lines.draw_lines()
        self.assertEqual(len(lines), 0)
        self.assertEqual(lines._stream_virtual_times, [])
        self.assertEqual(lines._stream_rng_draws, 0)
        lines.n_repeats = 1
        lines.draw_lines()
        self.assertEqual(len(lines), 9)
        self.assertIs(scene.mobjects[0], lines)

    def test_callback_family_edits_invalidate_the_candidate(self):
        lines = self.make()
        new_child = m.Line()
        edited = []
        def field(rows):
            if not edited:
                edited.append(True)
                lines.add(new_child)
            return vertical(rows)
        original = tuple(lines)
        lines.func = field
        with self.assertRaisesRegex(RuntimeError, 'family changed'):
            lines.draw_lines()
        # User side effects are not rolled back, but no candidate is published.
        self.assertEqual(tuple(lines), original + (new_child,))

    def test_recursive_rebuild_refuses_and_next_rebuild_succeeds(self):
        lines = self.make()
        def recursive(rows):
            lines.draw_lines()
            return rows
        before = tuple(lines)
        lines.func = recursive
        with self.assertRaisesRegex(RuntimeError, 'reenter'):
            lines.draw_lines()
        self.assertEqual(tuple(lines), before)
        lines.func = vertical
        lines.draw_lines()

    def test_copy_reintegrates_independently(self):
        lines = self.make()
        records = [line.data.copy() for line in lines]
        other = lines.copy()
        other.func = vertical
        other.draw_lines()
        for original, data, new in zip(lines, records, other):
            np.testing.assert_array_equal(original.data, data)
            self.assertFalse(np.array_equal(original.get_points(), new.get_points()))

    def test_authored_frame_updater_can_rebuild_bound_geometry(self):
        scene, lines = m.Scene(), self.make()
        scene.add(lines)
        def rebuild(current, dt):
            current.func = vertical
            current.draw_lines()
        lines.add_updater(rebuild)
        lines.update(.1)
        np.testing.assert_allclose(lines[0].get_end() - lines[0].get_start(), [0, .15, 0], atol=1e-6)
        self.assertEqual(lines.updaters, [rebuild])

    def test_rebuilt_native_pixels_change_and_remain_repeatable(self):
        # The native solver samples [0, solution_time), as the Reference does.
        # Exercise the renderer's supported endpoint-ramp lane here; arbitrary
        # interior stroke profiles remain a separate native renderer gap.
        lines = self.make(stroke_width=12, taper_stroke_width=False)
        camera = m.Camera()
        camera.reset_pixel_shape(160, 90)
        first = camera.capture_snapshot(lines).pixels()
        lines.func = vertical
        lines.draw_lines()
        second = camera.capture_snapshot(lines).pixels()
        self.assertTrue(first != second, "rebuilt geometry must change native pixels")
        self.assertEqual(second, camera.capture_snapshot(lines).pixels())
        lines.draw_lines()
        self.assertEqual(second, camera.capture_snapshot(lines).pixels())


suite = unittest.defaultTestLoader.loadTestsFromTestCase(StreamlineRebuildTests)
assert suite.countTestCases() == 13, 'native streamline rebuild inventory changed'
result = unittest.TextTestRunner(verbosity=2).run(suite)
gc.collect()
if not result.wasSuccessful():
    raise AssertionError('native streamline rebuilding failed')
