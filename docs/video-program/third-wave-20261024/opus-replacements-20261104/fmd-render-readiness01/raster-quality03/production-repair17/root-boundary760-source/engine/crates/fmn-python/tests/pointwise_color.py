"""Real native records, family semantics, animation and pixels for color callbacks."""
from __future__ import annotations

import gc
import unittest

import numpy as np
import manimlib as m
from manimlib.mobject.mobject import Mobject as QualifiedMobject
from manimlib.mobject.types.vectorized_mobject import VMobject as QualifiedVMobject


class PointwiseColorTests(unittest.TestCase):
    def square(self):
        return m.Square(fill_color="#ff0000", fill_opacity=1, stroke_color="#ff0000", stroke_width=20)

    def assert_lanes(self, mob, rgba):
        for field in ("fill_rgba", "stroke_rgba") if isinstance(mob, m.VMobject) else ("rgba",):
            expected = np.broadcast_to(np.asarray(rgba), mob.data[field].shape)
            np.testing.assert_allclose(mob.data[field], expected, atol=1e-7)

    def test_rgb_callback_colors_both_vector_paint_lanes(self):
        mob = self.square()
        expected = np.tile([0., 0., 1., .4], (mob.get_num_points(), 1))
        self.assertIs(mob.set_color_by_rgb_func(lambda points: expected[:, :3], opacity=.4), mob)
        self.assert_lanes(mob, expected)

    def test_rgba_callback_keeps_per_record_alpha(self):
        mob = self.square()
        expected = np.zeros((mob.get_num_points(), 4))
        expected[:, 1] = 1
        expected[:, 3] = np.linspace(0., 1., len(expected))
        self.assertIs(mob.set_color_by_rgba_func(lambda points: expected), mob)
        self.assert_lanes(mob, expected)

    def test_points_and_surfaces_use_rgba_without_inventing_vector_lanes(self):
        for mob in (m.Mobject().set_points([[0, 0, 0], [1, 0, 0]]),
                    m.DotCloud([[0, 0, 0], [1, 0, 0]]),
                    m.ParametricSurface(lambda u, v: [u, v, 0.], resolution=(3, 3))):
            with self.subTest(kind=type(mob).__name__):
                mob.set_color_by_rgba_func(lambda points: np.tile([0., 1., 0., .5], (len(points), 1)))
                self.assert_lanes(mob, [0., 1., 0., .5])

    def test_mixed_family_callbacks_have_native_point_counts_and_keep_order(self):
        square, cloud = self.square(), m.DotCloud([[0, 0, 0], [1, 0, 0]])
        family, visits = m.Group(square, m.Group(cloud)), []
        def color(points):
            visits.append(len(points))
            return np.tile([0., 1., 0.], (len(points), 1))
        family.set_color_by_rgb_func(color, opacity=.3)
        self.assertEqual(visits, [square.get_num_points(), cloud.get_num_points()])
        self.assert_lanes(square, [0., 1., 0., .3])
        self.assert_lanes(cloud, [0., 1., 0., .3])

    def test_shared_descendants_are_evaluated_once_before_paint_publication(self):
        shared = self.square()
        family = m.Group(m.Group(shared), m.Group(shared))
        seen = []
        def color(points):
            seen.append(shared.data["fill_rgba"][0].copy())
            return np.tile([0., 0., 1., .5], (len(points), 1))
        family.set_color_by_rgba_func(color)
        self.assertEqual(len(seen), 1)
        np.testing.assert_allclose(seen[0], [1., 0., 0., 1.])
        self.assert_lanes(shared, [0., 0., 1., .5])

    def test_empty_containers_and_recurse_false_do_not_invoke_callback(self):
        square = self.square()
        family = m.Group(square)
        original = square.data.copy()
        def forbidden(points):
            raise AssertionError("an empty container has no point colors")
        self.assertIs(family.set_color_by_rgb_func(forbidden, recurse=False), family)
        m.Mobject().set_color_by_rgba_func(forbidden)
        np.testing.assert_array_equal(square.data, original)
        square.add(m.Circle())
        child = square[0]
        original = child.data.copy()
        square.set_color_by_rgb_func(lambda points: np.zeros((len(points), 3)), recurse=False)
        np.testing.assert_array_equal(child.data, original)

    def test_single_color_broadcast_and_strided_callback_outputs(self):
        for value in ([0., 1., 0.], [[0., 1., 0.]]):
            mob = self.square()
            mob.set_color_by_rgb_func(lambda points: value)
            self.assert_lanes(mob, [0., 1., 0., 1.])
        mob = self.square()
        source = np.arange(mob.get_num_points() * 8, dtype=float).reshape((-1, 8)) / 100
        expected = source[::-1, ::2].copy()
        mob.set_color_by_rgba_func(lambda points: source[::-1, ::2])
        self.assert_lanes(mob, expected)

    def test_bad_callback_results_do_not_write_either_current_lane(self):
        mob = self.square()
        saved = mob.data.copy()
        n = mob.get_num_points()
        bad = ([0., 1.], np.zeros((n + 1, 4)), np.zeros((n, 4, 1)),
               np.full((n, 4), 1j), np.full((n, 4), np.nan),
               np.full((n, 4), np.inf), np.full((n, 4), 1e100),
               [["bad"] * 4] * n)
        for values in bad:
            with self.subTest(shape=np.shape(values)):
                with self.assertRaises((TypeError, ValueError)):
                    mob.set_color_by_rgba_func(lambda points: values)
                np.testing.assert_array_equal(mob.data, saved)
        mob.set_color_by_rgba_func(lambda points: [0., 0., 1., 1.])
        self.assert_lanes(mob, [0., 0., 1., 1.])

    def test_invalid_opacity_is_rejected_before_callback(self):
        mob = self.square()
        before = mob.data.copy()
        for alpha in (np.nan, np.inf, 1e100):
            with self.assertRaises(ValueError):
                mob.set_color_by_rgb_func(lambda points: self.fail("invalid alpha ran callback"), opacity=alpha)
            np.testing.assert_array_equal(mob.data, before)

    def test_late_failure_preserves_the_complete_family_and_exception(self):
        first, last = self.square(), self.square()
        family = m.Group(first, last)
        saved = last.data.copy()
        count = 0
        def color(points):
            nonlocal count
            count += 1
            if count == 2:
                raise LookupError("authored field failure")
            return np.tile([0., 0., 1.], (len(points), 1))
        with self.assertRaisesRegex(LookupError, "authored field failure"):
            family.set_color_by_rgb_func(color)
        np.testing.assert_array_equal(first.data, saved)
        np.testing.assert_array_equal(last.data, saved)
        family.set_color_by_rgb_func(lambda points: [0., 1., 0.])
        self.assert_lanes(last, [0., 1., 0., 1.])

    def test_callback_geometry_side_effect_invalidates_paint_but_keeps_scene_ownership(self):
        scene, mob = m.Scene(), self.square()
        scene.add(mob)
        points = mob.get_points()
        before = points.copy()
        def color(values):
            values[:, 0] += 1
            return np.tile([0., 0., 1., 1.], (len(values), 1))
        with self.assertRaisesRegex(RuntimeError, "geometry changed"):
            mob.set_color_by_rgba_func(color)
        np.testing.assert_array_equal(points[:, 0], before[:, 0] + 1)
        np.testing.assert_array_equal(mob.get_points(), points)
        self.assertIs(scene.mobjects[0], mob)
        self.assert_lanes(mob, [1., 0., 0., 1.])

    def test_resizing_callback_rejects_the_obsolete_paint_plan(self):
        mob = m.Mobject().set_points([[0, 0, 0]])
        old_color = mob.data["rgba"][0].copy()
        def color(points):
            mob.set_points([[0, 0, 0], [1, 0, 0]])
            return [[1., 0., 0., 1.], [0., 0., 1., .5]]
        with self.assertRaises(ValueError):
            mob.set_color_by_rgba_func(color)
        np.testing.assert_array_equal(mob.get_points(), [[0, 0, 0], [1, 0, 0]])
        self.assert_lanes(mob, old_color)

    def test_public_setter_overrides_dispatch_and_cannot_alias_later_lanes(self):
        calls = []
        class Observed(m.Square):
            def set_rgba_array(self, values, name="rgba", recurse=False):
                calls.append((name, recurse))
                result = super().set_rgba_array(values, name=name, recurse=recurse)
                values[:] = 0
                return result
        mob = Observed()
        calls.clear()
        source = np.tile([0., 0., 1., .5], (mob.get_num_points(), 1))
        expected = source.copy()
        mob.set_color_by_rgba_func(lambda points: source)
        self.assertEqual(calls, [("fill_rgba", False), ("stroke_rgba", False)])
        self.assert_lanes(mob, expected)
        np.testing.assert_array_equal(source, expected)

    def test_views_copies_and_class_exports_keep_their_native_identity(self):
        self.assertIs(m.Mobject, QualifiedMobject)
        self.assertIs(m.VMobject, QualifiedVMobject)
        mob = self.square()
        view, points = mob.data["fill_rgba"], mob.get_points()
        before = points.copy()
        clone = mob.copy()
        clone.set_color_by_rgb_func(lambda values: [0., 1., 0.])
        self.assert_lanes(mob, [1., 0., 0., 1.])
        mob.set_color_by_rgba_func(lambda values: [0., 0., 1., .5])
        np.testing.assert_allclose(view, np.tile([0., 0., 1., .5], (len(view), 1)))
        np.testing.assert_array_equal(points, before)
        self.assert_lanes(clone, [0., 1., 0., 1.])

    def test_real_camera_pixels_change_and_repeat_without_stale_views(self):
        mob, camera = self.square(), m.Camera()
        camera.reset_pixel_shape(160, 90)
        first = camera.capture_snapshot(mob).pixels()
        mob.set_color_by_rgb_func(lambda points: [0., 0., 1.])
        second = camera.capture_snapshot(mob).pixels()
        self.assertNotEqual(first, second)
        self.assertEqual(second, camera.capture_snapshot(mob).pixels())
        pixels = np.frombuffer(second, dtype=np.uint8).reshape((90, 160, 4))
        self.assertGreater(int(pixels[:, :, 2].sum()), 0)
        self.assertEqual(int(pixels[:, :, 0].sum()), 0)

    def test_authored_pointwise_gradient_reaches_interior_native_stroke_pixels(self):
        path = m.VMobject(stroke_width=60)
        path.set_points_as_corners([[-3., 0., 0.], [0., 0., 0.], [3., 0., 0.]])
        def color(points):
            red = np.abs(points[:, 0]) / 3.0
            return np.column_stack((red, np.zeros(len(points)), 1.0 - red))
        path.set_color_by_rgb_func(color)
        expected = np.column_stack((color(path.get_points()), np.ones(path.get_num_points())))
        self.assert_lanes(path, expected)
        camera = m.Camera()
        camera.reset_pixel_shape(160, 90)
        pixels = np.frombuffer(camera.capture_snapshot(path).pixels(), dtype=np.uint8).reshape((90, 160, 4))
        self.assertGreater(int(pixels[:, :, 0].sum()), 0)
        self.assertGreater(int(pixels[:, :, 2].sum()), 0)
        self.assertEqual(int(pixels[:, :, 1].sum()), 0)
        before = pixels.copy()
        path.set_color_by_rgb_func(lambda points: [1., 0., 0.])
        after = np.frombuffer(camera.capture_snapshot(path).pixels(), dtype=np.uint8).reshape((90, 160, 4))
        self.assertFalse(np.array_equal(before, after))
        self.assertEqual(int(after[:, :, 2].sum()), 0)

    def test_native_animate_builder_interpolates_pointwise_target_colors(self):
        scene, mob = m.Scene(), self.square()
        scene.add(mob)
        scene.play(mob.animate.set_color_by_rgb_func(lambda points: [0., 0., 1.]), run_time=.1)
        self.assert_lanes(mob, [0., 0., 1., 1.])
        self.assertFalse(mob._is_updating_suspended())

    def test_frame_updater_can_recolor_native_geometry_without_replacing_it(self):
        scene, mob = m.Scene(), self.square()
        scene.add(mob)
        calls = []
        def update(current, dt):
            calls.append(dt)
            current.set_color_by_rgb_func(lambda points: [0., 1., 0.])
        mob.add_updater(update)
        scene.wait(.1)
        self.assertTrue(calls)
        self.assertIs(scene.mobjects[0], mob)
        self.assert_lanes(mob, [0., 1., 0., 1.])
        mob.clear_updaters()


suite = unittest.defaultTestLoader.loadTestsFromTestCase(PointwiseColorTests)
assert suite.countTestCases() == 18
result = unittest.TextTestRunner(verbosity=2).run(suite)
gc.collect()
if not result.wasSuccessful():
    raise AssertionError("native pointwise-color acceptance failed")
