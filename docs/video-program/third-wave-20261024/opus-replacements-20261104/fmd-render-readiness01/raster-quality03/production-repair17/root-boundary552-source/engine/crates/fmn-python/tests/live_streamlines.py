"""Real native RK45 redraws, authored seed hooks and failure-safe scene edits."""
import gc
import itertools
from pathlib import Path
import tempfile
import unittest

import numpy as np
import manimlib as m
from manimlib.mobject.vector_field import StreamLines as QualifiedStreamLines


def right(rows):
    return np.tile([1., 0.], (len(rows), 1))


class LiveStreamlineTests(unittest.TestCase):
    def axes(self):
        return m.Axes(x_range=(-1, 1, 1), y_range=(-1, 1, 1))

    def lines(self, function=right, cls=m.StreamLines, **kwargs):
        options = dict(noise_factor=0, solution_time=.5, dt=.125, n_samples_per_line=5,
                       arc_len=10, color_by_magnitude=False, stroke_width=4)
        options.update(kwargs)
        return cls(function, self.axes(), **options)

    def state(self, lines):
        return (tuple(lines), [line.data.copy() for line in lines],
                list(lines._stream_virtual_times), lines._stream_rng_draws)

    def unchanged(self, lines, before):
        self.assertEqual(tuple(lines), before[0])
        for line, original in zip(lines, before[1]):
            np.testing.assert_array_equal(line.data, original)
        self.assertEqual(lines._stream_virtual_times, before[2])
        self.assertEqual(lines._stream_rng_draws, before[3])
        self.assertNotIn("_fmn_streamline_authoring_busy", vars(lines))
        self.assertNotIn("_fmn_streamline_sample_draws", vars(lines))

    def test_public_hooks_execute_once_on_the_original_receiver(self):
        calls = []
        class Custom(m.StreamLines):
            def get_sample_coords(self):
                calls.append((self, "seeds"))
                return iter([[0., 0.], [-1., 1.]])
            def draw_lines(self):
                calls.append((self, "draw"))
                return super().draw_lines()
            def init_style(self):
                calls.append((self, "style"))
                return super().init_style()
        lines = self.lines(cls=Custom)
        self.assertEqual(calls, [(lines, "draw"), (lines, "seeds"), (lines, "style")])
        self.assertEqual(len(lines), 2)
        self.assertEqual(lines._stream_rng_draws, 0)
        for line, seed in zip(lines, [[0., 0.], [-1., 1.]]):
            cs = lines.coordinate_system
            np.testing.assert_allclose(line.get_points()[0], cs.c2p(*seed), atol=1e-6)
            np.testing.assert_allclose(line.get_points()[-1], cs.c2p(seed[0] + .375, seed[1]), atol=1e-6)
        self.assertIs(QualifiedStreamLines, m.StreamLines)
        self.assertIs(lines.func, right)

    def test_seed_query_is_deterministic_without_field_or_global_rng_effects(self):
        lines = self.lines(noise_factor=.25, n_repeats=2)
        draws = lines._stream_rng_draws
        def forbidden(rows):
            raise AssertionError("seed query called the vector field")
        lines.func = forbidden
        random_state = np.random.get_state()
        first = lines.get_sample_coords()
        np.testing.assert_array_equal(first, lines.get_sample_coords())
        self.assertEqual(first.shape, (18, 2))
        self.assertEqual(draws, 36)
        first[:] = -10
        self.assertFalse(np.any(lines.get_sample_coords() == -10))
        self.assertEqual(draws, lines._stream_rng_draws)
        after = np.random.get_state()
        self.assertEqual(random_state[0], after[0])
        np.testing.assert_array_equal(random_state[1], after[1])
        self.assertEqual(random_state[2:], after[2:])

    def test_sample_coords_follow_the_reference_grid_repeats_and_noise_bound(self):
        # vector_field.py:119/406: each repeat walks the whole grid (every
        # range's step divided by density, np.arange(min, max + step, step)),
        # each coordinate displaced by U[0, noise_factor); the default
        # noise_factor is half an x unit divided by density.
        density, repeats = 2.0, 3
        lines = self.lines(noise_factor=None, density=density, n_repeats=repeats)
        coordinates = lines.coordinate_system
        ranges = [np.arange(low, high + step / density, step / density)
                  for low, high, step in coordinates.get_all_ranges()]
        grid = np.array(list(itertools.product(*ranges)))
        self.assertEqual(grid.shape, (25, 2))
        seeds = lines.get_sample_coords()
        self.assertEqual(seeds.shape, (repeats * len(grid), 2))
        offsets = seeds.reshape(repeats, len(grid), 2) - grid
        noise = coordinates.x_axis.get_unit_size() / density * 0.5
        self.assertTrue(np.all(offsets >= 0.0) and np.all(offsets < noise), offsets)
        self.assertGreater(offsets.max(), noise / 2)
        self.assertFalse(np.allclose(offsets[0], offsets[1]))
        exact = self.lines(noise_factor=0, density=density, n_repeats=2).get_sample_coords()
        np.testing.assert_allclose(exact, np.tile(grid, (2, 1)))

    def test_redraw_updates_field_geometry_but_preserves_root_scene_and_updaters(self):
        lines = self.lines()
        scene, unrelated = m.Scene(), m.Square().shift(4 * m.RIGHT)
        scene.add(lines, unrelated)
        old_children = tuple(lines)
        views = [line.get_points() for line in lines]
        old_points = [view.copy() for view in views]
        ticks = []
        updater = lambda owner, dt: ticks.append(dt) if owner is lines else None
        lines.add_updater(updater, call=False)
        lines.func = lambda rows: np.tile([0., 1.], (len(rows), 1))
        self.assertIsNone(lines.draw_lines())
        self.assertEqual(scene.mobjects, [lines, unrelated])
        self.assertIn(updater, lines.updaters)
        self.assertTrue(all(new is not old for new, old in zip(lines, old_children)))
        for line, view, before in zip(lines, views, old_points):
            np.testing.assert_array_equal(view, before)
            self.assertFalse(np.array_equal(line.get_points(), before))
            self.assertAlmostEqual(line.virtual_time, lines._stream_virtual_times[0])
        scene.wait(.125)
        self.assertTrue(any(dt > 0 for dt in ticks))

    def test_later_field_failure_preserves_complete_family_and_primary_exception(self):
        class Custom(m.StreamLines):
            def get_sample_coords(self):
                return [[0., 0.], [0., 2.]]
        lines = self.lines(cls=Custom)
        before = self.state(lines)
        failure, hits = LookupError("second seed fails"), []
        def broken(rows):
            if rows[0, 1] > 1:
                hits.append(1)
                raise failure
            return right(rows)
        lines.func = broken
        with self.assertRaises(LookupError) as caught:
            lines.draw_lines()
        self.assertIs(caught.exception, failure)
        self.assertEqual(hits, [1], "poisoned native solve reinvoked the failed callback")
        self.unchanged(lines, before)
        lines.func = right
        lines.draw_lines()
        self.assertTrue(all(new is not old for new, old in zip(lines, before[0])))

    def test_invalid_and_oversized_seed_tables_refuse_before_field(self):
        lines = self.lines()
        before = self.state(lines)
        calls = []
        lines.func = lambda rows: calls.append(rows) or right(rows)
        invalid = ([[np.nan, 0.]], [[np.inf, 0.]], [[1j, 0.]], [[0.]],
                   [[[0., 0.]]], itertools.repeat([0., 0.]))
        for seeds in invalid:
            lines.get_sample_coords = lambda seeds=seeds: seeds
            with self.subTest(seeds=type(seeds).__name__), self.assertRaises((ValueError, TypeError)):
                lines.draw_lines()
            self.unchanged(lines, before)
        self.assertEqual(calls, [])

    def test_empty_seeds_produce_empty_family_and_safe_animation(self):
        lines = self.lines()
        def forbidden(rows):
            raise AssertionError("empty stream family called its field")
        lines.func = forbidden
        lines.get_sample_coords = lambda: []
        lines.draw_lines()
        self.assertEqual(len(lines), 0)
        self.assertEqual(lines._stream_virtual_times, [])
        self.assertEqual(lines._stream_rng_draws, 0)
        animation = m.AnimatedStreamLines(lines)
        animation.update(.125)
        animation.clear_updaters()
        self.assertEqual(len(animation), 0)

    def test_repeats_zero_and_empty_range_do_not_panic(self):
        lines = self.lines(n_repeats=0)
        self.assertEqual(lines.get_sample_coords().shape, (0, 2))
        lines.coordinate_system.get_all_ranges = lambda: [(2, -2, 1), (-1, 1, 1)]
        lines.n_repeats = 1
        lines.draw_lines()
        self.assertEqual(len(lines), 0)

    def test_native_taper_survives_geometry_rebuild_and_live_restyling(self):
        lines = self.lines(taper_stroke_width=True, color_by_magnitude=True)
        lines.func = lambda rows: np.tile([0., 2.], (len(rows), 1))
        lines.stroke_width, lines.stroke_opacity = 8., .25
        lines.draw_lines()
        for line in lines:
            np.testing.assert_allclose(line.get_stroke_opacities(), .25)
            self.assertEqual(line.get_stroke_widths()[0], 0)
            self.assertEqual(line.get_stroke_widths()[-1], 0)
            self.assertGreater(max(line.get_stroke_widths()), 7.9)
        members = tuple(lines)
        lines.stroke_width = 4.
        lines.init_style()
        self.assertEqual(tuple(lines), members)
        self.assertLessEqual(max(lines[0].get_stroke_widths()), 4.)

    def test_invalid_control_does_not_invoke_field_or_discard_geometry(self):
        lines = self.lines()
        before = self.state(lines)
        calls = []
        lines.func = lambda rows: calls.append(rows) or right(rows)
        for key, value in (("density", 0), ("dt", 0), ("solution_time", -1),
                           ("arc_len", np.nan), ("n_repeats", 1.5), ("stroke_opacity", 2),
                           ("magnitude_range", (2, 1))):
            old = getattr(lines, key)
            setattr(lines, key, value)
            with self.subTest(key=key), self.assertRaises((TypeError, ValueError)):
                lines.draw_lines()
            setattr(lines, key, old)
            self.unchanged(lines, before)
        self.assertEqual(calls, [])

    def test_restylers_may_fail_without_publishing_new_geometry(self):
        lines = self.lines(color_by_magnitude=True)
        before = self.state(lines)
        failure = RuntimeError("paint evaluation failed")
        def function(rows):
            if len(rows) > 1:
                raise failure
            return right(rows)
        lines.func = function
        with self.assertRaises(RuntimeError) as caught:
            lines.draw_lines()
        self.assertIs(caught.exception, failure)
        self.unchanged(lines, before)

    def test_active_flashes_refuse_redraw_until_explicitly_stopped(self):
        lines = self.lines()
        animation = m.AnimatedStreamLines(lines, lag_range=0)
        before = self.state(lines)
        with self.assertRaisesRegex(RuntimeError, "stop active|released animations"):
            lines.draw_lines()
        self.unchanged(lines, before)
        animation.clear_updaters()
        lines.draw_lines()
        self.assertTrue(all(new is not old for new, old in zip(lines, before[0])))

    def test_recursive_redraw_preserves_family_and_recovers(self):
        lines = self.lines()
        before = self.state(lines)
        def field(rows):
            lines.draw_lines()
            return right(rows)
        lines.func = field
        with self.assertRaisesRegex(RuntimeError, "reenter"):
            lines.draw_lines()
        self.unchanged(lines, before)
        lines.func = right
        lines.draw_lines()

    def test_live_axes_transforms_and_three_dimensional_fields_remain_authoritative(self):
        axes = m.ThreeDAxes(x_range=(0, 1, 1), y_range=(0, 1, 1), z_range=(0, 1, 1))
        class Custom(m.StreamLines):
            def get_sample_coords(self):
                return [[0., 0., 0.]]
        def field(rows):
            self.assertEqual(rows.shape[1], 3)
            return np.tile([0., 0., 1.], (len(rows), 1))
        lines = Custom(field, axes, noise_factor=0, solution_time=.5, dt=.125,
                       n_samples_per_line=5, color_by_magnitude=False)
        axes.rotate(m.PI / 2).shift(2 * m.RIGHT)
        lines.draw_lines()
        np.testing.assert_allclose(lines[0].get_points()[0], axes.c2p(0, 0, 0), atol=1e-6)
        np.testing.assert_allclose(lines[0].get_points()[-1], axes.c2p(0, 0, .375), atol=1e-6)

    def test_callback_geometry_edits_are_not_silently_overwritten(self):
        lines = self.lines()
        members = tuple(lines)
        changed = []
        def field(rows):
            if not changed:
                lines[0].shift(m.UP)
                changed.append(1)
            return right(rows)
        lines.func = field
        with self.assertRaisesRegex(RuntimeError, "(?:family|geometry) changed"):
            lines.draw_lines()
        self.assertEqual(tuple(lines), members)
        self.assertNotIn("_fmn_streamline_authoring_busy", vars(lines))

    def test_seed_query_then_redraw_preserves_named_rng_offset(self):
        lines = self.lines(noise_factor=.2)
        first = [line.get_points().copy() for line in lines]
        draws = lines._stream_rng_draws
        for _ in range(3):
            lines.get_sample_coords()
        lines.draw_lines()
        self.assertEqual(lines._stream_rng_draws, draws)
        for line, points in zip(lines, first):
            np.testing.assert_array_equal(line.get_points(), points)

    def test_live_redraw_render_frames_match_independent_static_solutions(self):
        def render(path, threads, static=None):
            scene, angle = m.Scene(), [0.]
            def field(rows):
                return np.tile([np.cos(angle[0]), np.sin(angle[0])], (len(rows), 1))
            lines = self.lines(field, stroke_width=12)
            with scene.render_session(path, format="png_sequence", resolution=(160, 90), fps=4, threads=threads):
                scene.add(lines)
                if static is None:
                    for radians in (0., .5, 1., 1.5):
                        angle[0] = radians
                        lines.draw_lines()
                        scene.wait(.25)
                else:
                    angle[0] = static
                    lines.draw_lines()
                    scene.wait(.25)
            return [file.read_bytes() for file in sorted(Path(path).glob("*.png"))]
        with tempfile.TemporaryDirectory(prefix="fmn-live-streamlines-") as directory:
            root = Path(directory)
            frames = render(root / "one", 1)
            self.assertEqual(len(frames), 4)
            self.assertEqual(len(set(frames)), 4)
            self.assertEqual(frames, render(root / "four", 4))
            for index, angle in enumerate((0., .5, 1., 1.5)):
                self.assertEqual(frames[index], render(root / f"reference-{index}", 1, angle)[0])


suite = unittest.defaultTestLoader.loadTestsFromTestCase(LiveStreamlineTests)
assert suite.countTestCases() == 17, "native live-streamlines inventory changed"
result = unittest.TextTestRunner(verbosity=2).run(suite)
gc.collect()
if not result.wasSuccessful():
    raise AssertionError("native live-streamlines acceptance failed")
