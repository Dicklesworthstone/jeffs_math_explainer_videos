"""Full per-record fill profiles through real native camera pixels and live views."""
import gc
import unittest
import numpy as np
import manimlib as m


class FillProfilePixels(unittest.TestCase):
    def camera(self):
        camera = m.Camera()
        camera.reset_pixel_shape(128, 96)
        return camera

    def shape(self):
        return m.Square(side_length=4, stroke_width=0, fill_color=m.RED, fill_opacity=1)

    def pixels(self, camera, *mobs):
        return np.frombuffer(camera.capture_snapshot(*mobs).pixels(), np.uint8).reshape(96, 128, 4).copy()

    def test_interior_color_changes_actual_pixels_with_identical_endpoints(self):
        mob, camera = self.shape(), self.camera()
        before = self.pixels(camera, mob)
        mob.data['fill_rgba'][4, :3] = [0, 0, 1]
        after = self.pixels(camera, mob)
        self.assertFalse(np.array_equal(before, after))
        self.assertGreater(int(after[:, :, 2].sum()), 0)
        np.testing.assert_array_equal(after, self.pixels(camera, mob))

    def test_transparent_endpoints_do_not_cull_opaque_interior(self):
        mob, camera = self.shape(), self.camera()
        mob.data['fill_rgba'][:] = [0, 0, 1, 0]
        mob.data['fill_rgba'][4, 3] = 1
        self.assertGreater(int(self.pixels(camera, mob)[:, :, :3].sum()), 0)

    def test_writable_fill_view_invalidates_pixels_without_geometry_change(self):
        mob, camera = self.shape(), self.camera()
        points = mob.get_points().copy()
        view = mob.data['fill_rgba']
        before = self.pixels(camera, mob)
        view[4] = [0, 1, 0, 1]
        after = self.pixels(camera, mob)
        self.assertFalse(np.array_equal(before, after))
        np.testing.assert_array_equal(mob.get_points(), points)
        view[4] = view[0]
        np.testing.assert_array_equal(before, self.pixels(camera, mob))

    def test_transparent_middle_reveals_underlying_object(self):
        back, front, camera = self.shape(), self.shape(), self.camera()
        back.set_fill(m.BLUE, 1)
        # The endpoints remain opaque; occlusion pruning must read all alpha.
        front.data['fill_rgba'][:] = [1, 0, 0, 0]
        front.data['fill_rgba'][[0, -1], 3] = 1
        pixels = self.pixels(camera, back, front)
        self.assertGreater(int(pixels[:, :, 2].sum()), 0)
        self.assertGreater(int(pixels[:, :, 0].sum()), 0)

    def test_rotation_into_world_plane_keeps_interior_colors(self):
        mob, camera = self.shape(), self.camera()
        mob.data['fill_rgba'][4, :3] = [0, 0, 1]
        mob.rotate(m.PI / 4, axis=m.RIGHT)
        pixels = self.pixels(camera, mob)
        self.assertGreater(int(pixels[:, :, 2].sum()), 0)
        np.testing.assert_array_equal(pixels, self.pixels(camera, mob))

    def test_copy_retains_profile_and_original_live_edits_remain_independent(self):
        mob, camera = self.shape(), self.camera()
        mob.data['fill_rgba'][4, :3] = [0, 0, 1]
        other = mob.copy()
        before = self.pixels(camera, other)
        np.testing.assert_array_equal(before, self.pixels(camera, mob))
        mob.data['fill_rgba'][4, :3] = [0, 1, 0]
        np.testing.assert_array_equal(before, self.pixels(camera, other))
        self.assertFalse(np.array_equal(before, self.pixels(camera, mob)))


suite = unittest.defaultTestLoader.loadTestsFromTestCase(FillProfilePixels)
assert suite.countTestCases() == 6
result = unittest.TextTestRunner(verbosity=2).run(suite)
gc.collect()
if not result.wasSuccessful():
    raise AssertionError('native fill-profile acceptance failed')
