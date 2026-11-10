"""Native bar geometry through zero, sign changes, affine charts and playback."""
import copy
import itertools
from pathlib import Path
import tempfile
import unittest

import manimlib as m
import numpy as np


# Independent rectangle vertex/handle order, including the closing anchor.
UV = np.array(((1, 1), (.5, 1), (0, 1), (0, .5), (0, 0),
               (.5, 0), (1, 0), (1, .5), (1, 1)))


def chart(values=(.5, .25), **kwargs):
    return m.BarChart(values, label_y_axis=False, **kwargs)


def geometry(left, right, up, value, maximum=1.):
    return left + UV[:, :1] * (right - left) + UV[:, 1:] * (value / maximum) * up


def snapshots(obj):
    return [bar.get_points().copy() for bar in obj.bars]


class LiveBarChartTests(unittest.TestCase):
    def test_bars_recover_from_zero_and_cross_sign_without_baseline_drift(self):
        obj = chart()
        before = snapshots(obj)
        for value in (0., .75, -.5, 0., -.25, .5):
            with self.subTest(value=value):
                self.assertIsNone(obj.change_bar_values((value, .25)))
                np.testing.assert_allclose(obj.bars[0].get_points(),
                    geometry(before[0][4], before[0][6], np.array((0., 4., 0.)), value), atol=2e-7)
                np.testing.assert_array_equal(obj.bars[1].get_points(), before[1])

    def test_initially_zero_bars_are_not_permanently_collapsed(self):
        obj = chart((0., 0.))
        before = snapshots(obj)
        obj.change_bar_values((.5, -.5))
        for bar, old, value in zip(obj.bars, before, (.5, -.5)):
            np.testing.assert_allclose(bar.get_points(),
                geometry(old[4], old[6], np.array((0., 4., 0.)), value), atol=2e-7)

    def test_automatic_scale_supports_negative_and_all_zero_initial_data(self):
        for values, maximum in [((0., 0.), 1.), ((-2., 1.), 2.), ((-2., -4.), 4.)]:
            with self.subTest(values=values):
                obj = chart(values, max_value=None)
                self.assertEqual(obj.max_value, maximum)
                baseline = obj.x_axis.get_start()[1]
                for bar, value in zip(obj.bars, values):
                    points = bar.get_points()
                    self.assertAlmostEqual(points[4, 1], baseline, places=6)
                    self.assertAlmostEqual(points[0, 1] - points[4, 1], 4 * value / maximum, places=6)

    def test_live_maximum_changes_scale_without_changing_baseline(self):
        obj = chart((.5, .5))
        before = snapshots(obj)
        obj.max_value = 2.
        obj.change_bar_values((1., 0.))
        np.testing.assert_array_equal(obj.bars[0].get_points(), before[0])
        obj.max_value = .5
        obj.change_bar_values((.5, .5))
        for bar, old in zip(obj.bars, before):
            np.testing.assert_allclose(bar.get_points(), geometry(old[4], old[6], np.array((0., 4., 0.)), 1.), atol=2e-7)

    def test_affine_transformed_chart_uses_axis_direction_not_screen_extents(self):
        transforms = [np.array(((0, -1, 0), (1, 0, 0), (0, 0, 1.))),
                      np.array(((-2, .75, 0), (0, .5, 0), (.25, 1, 1.)))]
        for matrix in transforms:
            with self.subTest(matrix=matrix):
                obj = chart()
                before = snapshots(obj)
                shift = np.array((1., -2., .5))
                obj.apply_matrix(matrix).shift(shift)
                obj.change_bar_values((0., 0.))
                obj.change_bar_values((-.5, .75))
                for bar, old, value in zip(obj.bars, before, (-.5, .75)):
                    expected = geometry(old[4], old[6], np.array((0., 4., 0.)), value)
                    expected = expected @ matrix.T + shift
                    np.testing.assert_allclose(bar.get_points(), expected, atol=1e-6)

    def test_direct_bar_width_and_position_edits_are_retained(self):
        obj = chart()
        bar = obj.bars[0]
        bar.stretch(2., 0).shift((.25, 0., 1.))
        before = bar.get_points().copy()
        obj.change_bar_values((0.,))
        obj.change_bar_values((.75,))
        np.testing.assert_allclose(bar.get_points(),
            geometry(before[4], before[6], np.array((0., 4., 0.)), .75), atol=2e-7)

    def test_prefix_and_empty_updates_preserve_other_bars(self):
        obj = chart((.5, .25, .75))
        before = snapshots(obj)
        self.assertIsNone(obj.change_bar_values(()))
        obj.change_bar_values(iter((0.,)))
        for bar, old in zip(tuple(obj.bars)[1:], before[1:]):
            np.testing.assert_array_equal(bar.get_points(), old)

    def test_scene_bar_identities_children_updaters_views_and_styles_survive(self):
        obj = chart(bar_names=('a', 'b'), bar_colors=(m.RED, m.BLUE), bar_stroke_width=5)
        bars = tuple(obj.bars)
        oldstyles = [bar.data['fill_rgba'].copy() for bar in bars]
        labels = [label.get_all_points().copy() for label in obj.bar_labels]
        dot = m.Dot()
        bars[0].add(dot)
        calls = []
        callback = lambda mob, dt: calls.append(dt)
        bars[0].add_updater(callback, call=False)
        scene = m.Scene()
        scene.add(obj)
        # Binding is a separate native generation transition. Capture the live
        # views of the scene-owned records, not the now-detached nursery views.
        views = [bar.get_points() for bar in bars]
        time = scene.get_time()
        obj.change_bar_values((0., -.5))
        obj.change_bar_values((.75, .5))
        self.assertEqual(tuple(obj.bars), bars)
        self.assertEqual(tuple(scene.mobjects), (obj,))
        self.assertIs(obj._scene, scene)
        self.assertIs(bars[0][0], dot)
        self.assertIs(bars[0].updaters[0], callback)
        self.assertEqual(calls, [])
        self.assertEqual(scene.get_time(), time)
        for bar, view, style in zip(bars, views, oldstyles):
            np.testing.assert_array_equal(view, bar.get_points())
            np.testing.assert_array_equal(bar.data['fill_rgba'], style)
            np.testing.assert_allclose(bar.get_stroke_widths(), 5.)
        for label, old in zip(obj.bar_labels, labels):
            np.testing.assert_array_equal(label.get_all_points(), old)

    def test_copies_and_saved_states_retain_independent_bars(self):
        obj = chart((0., .25))
        before = snapshots(obj)
        obj.save_state()
        for duplicate in (obj.copy(), copy.copy(obj), copy.deepcopy(obj)):
            duplicate.change_bar_values((.5, .75))
            self.assertIsNot(duplicate.bars[0], obj.bars[0])
            self.assertGreater(duplicate.bars[0].get_height(), 0.)
            for bar, old in zip(obj.bars, before):
                np.testing.assert_array_equal(bar.get_points(), old)
        obj.change_bar_values((.5, .75))
        for bar, old in zip(obj.saved_state.bars, before):
            np.testing.assert_array_equal(bar.get_points(), old)
        obj.restore()
        for bar, old in zip(obj.bars, before):
            np.testing.assert_array_equal(bar.get_points(), old)
        obj.change_bar_values((.75, .5))
        self.assertAlmostEqual(obj.bars[0].get_height(), 3.)

    def test_native_builder_animates_zero_to_nonzero_and_back(self):
        obj = chart((0., .25))
        bar = obj.bars[0]
        before = bar.get_points().copy()
        scene = m.Scene()
        scene.add(obj)
        for value in (.75, 0., -.5, .5):
            scene.play(obj.animate.change_bar_values((value, .25)), run_time=.125, rate_func=m.linear)
            self.assertIs(obj.bars[0], bar)
            np.testing.assert_allclose(bar.get_points(),
                geometry(before[4], before[6], np.array((0., 4., 0.)), value), atol=2e-7)

    def test_invalid_later_input_refuses_before_any_geometry_is_published(self):
        for values in ((.75, np.nan), (.75, np.inf), (.75, 1e100), (.75, .5, .25)):
            obj = chart()
            before = snapshots(obj)
            with self.assertRaises((ValueError, TypeError)):
                obj.change_bar_values(values)
            for bar, old in zip(obj.bars, before):
                np.testing.assert_array_equal(bar.get_points(), old)
            obj.change_bar_values((.5, .5))

    def test_conversion_exception_identity_and_reentry_cleanup(self):
        obj = chart()
        for error in (LookupError('value'), KeyboardInterrupt('cancel')):
            before = snapshots(obj)
            class Bad:
                def __float__(self):
                    raise error
            with self.assertRaises(type(error)) as caught:
                obj.change_bar_values((.75, Bad()))
            self.assertIs(caught.exception, error)
            for bar, old in zip(obj.bars, before):
                np.testing.assert_array_equal(bar.get_points(), old)
        class Reentrant:
            def __float__(self):
                obj.change_bar_values((.75,))
                return .5
        with self.assertRaisesRegex(RuntimeError, 'reenter'):
            obj.change_bar_values((Reentrant(),))
        self.assertIsNone(obj.change_bar_values((.5, .5)))

    def test_unsupported_topology_or_schema_refuses_before_partial_update(self):
        for kind in ('points', 'schema'):
            obj = chart()
            if kind == 'points': obj.bars[1].insert_n_curves(1)
            else: obj.bars[1].pointlike_data_keys = ['point', 'other']
            before = snapshots(obj)
            with self.assertRaises((TypeError, ValueError)):
                obj.change_bar_values((.75, .5))
            for bar, old in zip(obj.bars, before):
                np.testing.assert_array_equal(bar.get_points(), old)

    def test_constructor_bounds_iterables_and_admits_values_once(self):
        seen = []
        def values():
            for number in (0., .5):
                seen.append(number)
                yield number
        obj = chart(values())
        self.assertEqual(seen, [0., .5])
        obj.change_bar_values((.75, .5))
        self.assertAlmostEqual(obj.bars[0].get_height(), 3.)
        for bad in ((), itertools.repeat(1.), (np.nan, .5)):
            with self.assertRaises((TypeError, ValueError)):
                chart(bad)
        for kwargs in ({'max_value': 0}, {'height': np.nan}, {'width': -1}):
            with self.assertRaises(ValueError):
                chart(**kwargs)

    def test_bound_reconstruction_is_refused(self):
        obj = chart()
        scene = m.Scene()
        scene.add(obj)
        before = snapshots(obj)
        with self.assertRaisesRegex(RuntimeError, 'detached'):
            obj.__init__((1., 1.), label_y_axis=False)
        self.assertIs(obj._scene, scene)
        for bar, old in zip(obj.bars, before):
            np.testing.assert_array_equal(bar.get_points(), old)

    def test_tracker_render_matches_independent_signed_rectangles_at_all_threads(self):
        class Live(m.Scene):
            def construct(self):
                level = m.ValueTracker(.5)
                obj = chart((.5, .25), height=2., width=4., bar_stroke_width=0)
                obj.add_updater(lambda current: current.change_bar_values((level.get_value(), .25)), call=False)
                self.add(obj)
                self.play(level.animate.set_value(-.5), run_time=1., rate_func=m.linear)
        class Reference(m.Scene):
            def construct(self):
                level = m.ValueTracker(.5)
                obj = chart((.5, .25), height=2., width=4., bar_stroke_width=0)
                p = obj.bars[0].get_points().copy()
                obj.add_updater(lambda current: current.bars[0].set_points(
                    geometry(p[4], p[6], np.array((0., 2., 0.)), level.get_value())), call=False)
                self.add(obj)
                self.play(level.animate.set_value(-.5), run_time=1., rate_func=m.linear)
        with tempfile.TemporaryDirectory(prefix='fmn-live-bars-') as directory:
            results = []
            for cls in (Live, Reference):
                for threads in (1, 4):
                    path = Path(directory) / f'{cls.__name__}-{threads}.y4m'
                    receipt = cls().render(path, format='y4m', resolution=(96, 54), fps=8, threads=threads)
                    self.assertEqual(receipt.frame_count, 8)
                    results.append(path.read_bytes())
            self.assertTrue(all(result == results[0] for result in results))
            body = results[0].split(b'\n', 1)[1]
            stride = 6 + 96*54*3//2
            self.assertNotEqual(body[:stride], body[-stride:])


if __name__ == '__main__':
    unittest.main()
