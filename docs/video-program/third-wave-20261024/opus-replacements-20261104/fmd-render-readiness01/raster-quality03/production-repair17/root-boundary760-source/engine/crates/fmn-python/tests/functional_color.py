"""Actual native record schemas, author callbacks, family failure and pixels."""
from pathlib import Path
import gc
import unittest
import numpy as np
import manimlib as m


class FunctionalColorTests(unittest.TestCase):
    def field(self, points):
        return np.column_stack((.5 + .2 * points[:, 0], .5 + .2 * points[:, 1],
                                np.full(len(points), .25)))

    def test_vector_shapes_write_fill_and_stroke_not_a_missing_rgba_column(self):
        for mob in (m.Square(), m.Circle(), m.Line(m.LEFT, m.RIGHT)):
            points = mob.get_points().copy()
            self.assertIs(mob.set_color_by_rgb_func(self.field, opacity=.4), mob)
            expected = np.column_stack((self.field(points), np.full(len(points), .4)))
            for name in ("fill_rgba", "stroke_rgba"):
                np.testing.assert_allclose(mob.data[name], expected, rtol=1e-6)
            np.testing.assert_array_equal(mob.get_points(), points)

    def test_surface_and_dot_cloud_records_keep_the_native_rgba_lane(self):
        for mob in (m.DotCloud([[0., 0., 0.], [1., 1., 0.]]), m.Sphere(resolution=(3, 3))):
            points = mob.get_points().copy()
            mob.set_color_by_rgb_func(lambda p: [.2, .4, .6], opacity=.7)
            np.testing.assert_allclose(mob.data['rgba'],
                np.tile([.2, .4, .6, .7], (mob.get_num_points(), 1)))
            np.testing.assert_array_equal(mob.get_points(), points)

    def test_point_records_keep_the_existing_rgba_semantics(self):
        mob = m.Point([2., 4., 6.])
        self.assertIs(mob.set_color_by_rgb_func(lambda p: p * .25, opacity=.5), mob)
        np.testing.assert_array_equal(mob.data['rgba'][0], [.5, 1., 1.5, .5])
        mob.set_color_by_rgba_func(lambda p: np.hstack((p * .5, np.ones((len(p), 1)))))
        np.testing.assert_array_equal(mob.data['rgba'][0], [1., 2., 3., 1.])

    def test_empty_groups_skip_callbacks_but_recurse_through_mixed_records(self):
        left, right = m.Point(), m.Square()
        group = m.Group(m.Mobject(), left, m.VGroup(right))
        calls = []
        self.assertIs(group.set_color_by_rgba_func(
            lambda p: calls.append(len(p)) or [1., 0., 0., .5]), group)
        self.assertEqual(calls, [left.get_num_points(), right.get_num_points()])
        np.testing.assert_array_equal(left.data['rgba'], [[1., 0., 0., .5]])
        np.testing.assert_array_equal(right.data['fill_rgba'],
                                      np.tile([1., 0., 0., .5], (right.get_num_points(), 1)))
        calls.clear()
        group.set_color_by_rgb_func(lambda p: calls.append(1), recurse=False)
        self.assertEqual(calls, [])

    def test_late_callback_failure_preserves_the_entire_family_paint(self):
        group = m.VGroup(m.Square(), m.Circle())
        saved = [mob.data.copy() for mob in group]
        calls = []
        error = LookupError('second field failed')
        def field(points):
            calls.append(len(points))
            if len(calls) == 2:
                raise error
            return [0., 1., 0.]
        with self.assertRaises(LookupError) as caught:
            group.set_color_by_rgb_func(field)
        self.assertIs(caught.exception, error)
        for mob, before in zip(group, saved):
            np.testing.assert_array_equal(mob.data, before)
        group.set_color_by_rgb_func(lambda p: [0., 0., 1.])
        self.assertFalse(np.array_equal(group[0].data, saved[0]))

    def test_bad_color_outputs_never_partly_update_records(self):
        mob = m.Square()
        for result in ([[1, 2]], [1j, 0, 0], [np.nan, 0, 0], [np.inf, 0, 0],
                       [1e200, 0, 0], ['red', 0, 0]):
            saved = mob.data.copy()
            with self.subTest(result=result), self.assertRaises((ValueError, TypeError)):
                mob.set_color_by_rgb_func(lambda p: result)
            np.testing.assert_array_equal(mob.data, saved)
        for alpha in (None, np.nan, np.inf, 1j, 1e200):
            with self.subTest(alpha=alpha), self.assertRaises((ValueError, TypeError)):
                mob.set_color_by_rgb_func(lambda p: self.fail('invalid alpha called field'), alpha)

    def test_live_output_alias_is_frozen_before_the_next_callback(self):
        first, second = m.Square(), m.Square()
        group = m.VGroup(first, second)
        values = np.tile([1., 0., 0., .4], (first.get_num_points(), 1))
        calls = []
        def field(points):
            if calls:
                values[:] = [0., 0., 1., 1.]
            calls.append(1)
            return values
        group.set_color_by_rgba_func(field)
        np.testing.assert_allclose(first.data['fill_rgba'], np.tile([1.,0.,0.,.4],(len(values),1)))
        np.testing.assert_allclose(second.data['fill_rgba'], np.tile([0.,0.,1.,1.],(len(values),1)))

    def test_reentrant_family_color_is_refused_then_ownership_is_released(self):
        child, group = m.Square(), m.VGroup()
        group.add(child)
        before = child.data.copy()
        def field(points):
            child.set_color_by_rgb_func(lambda p: [0., 1., 0.])
            return points
        with self.assertRaisesRegex(RuntimeError, 'reenter'):
            group.set_color_by_rgb_func(field)
        np.testing.assert_array_equal(child.data, before)
        child.set_color_by_rgb_func(lambda p: [0., 1., 0.])
        self.assertNotIn('_fmn_functional_color_busy', vars(child))

    def test_callback_geometry_side_effect_is_not_rolled_back_or_recolored(self):
        mob = m.Square()
        points, paint = mob.get_points().copy(), mob.data['fill_rgba'].copy()
        def field(p):
            mob.shift(m.RIGHT)
            return np.zeros_like(p)
        with self.assertRaisesRegex(RuntimeError, 'geometry changed'):
            mob.set_color_by_rgb_func(field)
        np.testing.assert_allclose(mob.get_points(), points + m.RIGHT)
        np.testing.assert_array_equal(mob.data['fill_rgba'], paint)

    def test_bound_records_keep_existing_views_and_scene_identity(self):
        scene, mob = m.Scene(), m.Square()
        scene.add(mob)
        view = mob.data['fill_rgba']
        points = mob.get_points()
        before = points.copy()
        mob.set_color_by_rgb_func(self.field)
        np.testing.assert_allclose(view[:, :3], self.field(before), rtol=1e-6)
        np.testing.assert_array_equal(points, before)
        self.assertIs(scene.mobjects[0], mob)
        view[0, 0] = .1
        self.assertAlmostEqual(mob.data['fill_rgba'][0,0], .1)

    def test_authored_method_override_and_qualified_class_identity_survive(self):
        from manimlib.mobject.mobject import Mobject
        self.assertIs(Mobject, m.Mobject)
        calls = []
        class Custom(m.Square):
            def set_color_by_rgb_func(self, func, opacity=1, recurse=True):
                calls.append(self)
                return super().set_color_by_rgb_func(func, opacity, recurse)
        mob = Custom()
        self.assertIs(mob.set_color_by_rgb_func(self.field), mob)
        self.assertEqual(calls, [mob])

    def test_authored_color_setters_still_receive_native_schema_fields(self):
        calls = []
        class Point(m.Point):
            def set_rgba_array(self, value):
                calls.append((self, "rgba"))
                return super().set_rgba_array(value)
        class Square(m.Square):
            def set_rgba_array(self, value, name="rgba", recurse=False):
                calls.append((self, name))
                return super().set_rgba_array(value, name, recurse)
        point, square = Point(), Square()
        calls.clear()
        m.Group(point, square).set_color_by_rgb_func(lambda p: [0., 1., 0.])
        self.assertEqual(calls, [(point, "rgba"), (square, "fill_rgba"), (square, "stroke_rgba")])

    def test_family_mutation_invalidates_paint_without_rewinding_authored_edits(self):
        child, added = m.Square(), m.Circle()
        group = m.VGroup(child)
        before = child.data.copy()
        def field(points):
            group.add(added)
            return [0., 1., 0.]
        with self.assertRaisesRegex(RuntimeError, 'family changed'):
            group.set_color_by_rgb_func(field)
        self.assertIn(added, group.submobjects)
        np.testing.assert_array_equal(child.data, before)
        group.set_color_by_rgb_func(lambda p: [0., 1., 0.])

    def test_shared_child_is_evaluated_once_in_family_order(self):
        child = m.Square()
        left, right = m.VGroup(child), m.VGroup(child)
        group, calls = m.VGroup(left, right), []
        group.set_color_by_rgb_func(lambda p: calls.append(len(p)) or [1., 0., 0.])
        self.assertEqual(calls, [child.get_num_points()])

    def test_native_stroke_pixels_observe_callable_interior_color(self):
        mob = m.Line(3*m.LEFT, 3*m.RIGHT, stroke_width=80)
        camera = m.Camera()
        camera.reset_pixel_shape(96, 64)
        before = camera.capture_snapshot(mob).pixels()
        mob.set_color_by_rgba_func(lambda p: np.column_stack((
            np.zeros(len(p)), 1-np.abs(p[:,0])/3, np.abs(p[:,0])/3, np.ones(len(p)))))
        after = camera.capture_snapshot(mob).pixels()
        self.assertNotEqual(before, after)
        pixels = np.frombuffer(after, np.uint8).reshape(64,96,4)
        self.assertGreater(int(pixels[:,:,1].sum()), 0)
        self.assertEqual(after, camera.capture_snapshot(mob).pixels())


suite = unittest.defaultTestLoader.loadTestsFromTestCase(FunctionalColorTests)
assert suite.countTestCases() == 15
result = unittest.TextTestRunner(verbosity=2).run(suite)
gc.collect()
if not result.wasSuccessful():
    raise AssertionError('native functional-color acceptance failed')
