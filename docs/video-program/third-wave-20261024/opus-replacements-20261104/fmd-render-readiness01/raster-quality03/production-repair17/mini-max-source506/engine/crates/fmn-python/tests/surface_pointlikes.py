"""Real Atlas/Marionette/Lumen acceptance for 3D pointlike record geometry."""
import unittest
import numpy as np
import manimlib as m


class SurfacePointlikeTests(unittest.TestCase):
    def surface(self):
        return m.ParametricSurface(lambda u, v: (u, v, 0.0),
                                   u_range=(-1, 1), v_range=(-1, 1), resolution=(5, 7))

    def assert_normals(self, surface, normal):
        expected = np.broadcast_to(normal, (surface.n_records(), 3))
        np.testing.assert_allclose(surface.get_unit_normals(), expected, atol=3e-4)

    def test_shift_readback_does_not_turn_normal_seeds_into_directions(self):
        for bound in (False, True):
            with self.subTest(bound=bound):
                surface = self.surface()
                scene = m.Scene()
                if bound:
                    scene.add(surface)
                surface.shift((2, -1, 0))
                self.assert_normals(surface, (0, 0, 1))
                self.assertEqual(surface._is_bound(), bound)

    def test_affine_edits_update_already_exported_normal_views(self):
        for bound in (False, True):
            with self.subTest(bound=bound):
                surface = self.surface()
                if bound:
                    scene = m.Scene()
                    scene.add(surface)
                records = surface.data
                points = records['point']
                normals = records['d_normal_point']
                original = np.array(normals - points, dtype=float)
                surface.shift((2, 1, 0)).scale(2, about_point=m.ORIGIN)
                np.testing.assert_allclose(normals - points, 2 * original, atol=2e-6)
                self.assert_normals(surface, (0, 0, 1))
                np.testing.assert_array_equal(normals, surface.data['d_normal_point'])

    def test_rotation_and_stretch_keep_whole_record_geometry(self):
        surface = self.surface()
        surface.rotate(np.pi / 2, axis=m.RIGHT, about_point=m.ORIGIN)
        self.assert_normals(surface, (0, -1, 0))
        surface.stretch(3, 0, about_point=m.ORIGIN)
        self.assert_normals(surface, (0, -1, 0))
        surface.shift((1, 2, 3))
        self.assert_normals(surface, (0, -1, 0))

    def test_reading_a_placed_surface_does_not_change_its_render(self):
        camera = m.Camera(resolution=(96, 64), background_opacity=0)
        surface = self.surface().shift((1, 0, 0))
        before = camera.capture_snapshot(surface).pixels()
        surface.get_points()
        surface.get_unit_normals()
        surface.data.copy()
        after = camera.capture_snapshot(surface).pixels()
        self.assertEqual(before, after)
        self.assertTrue(np.frombuffer(after, dtype=np.uint8).reshape(-1, 4)[:, 3].any())

    def test_shared_family_transform_does_not_apply_twice(self):
        surface = self.surface()
        group = m.Group(m.Group(surface), m.Group(surface))
        points = np.array(surface.get_points())
        normals = surface.data['d_normal_point']
        seed = np.array(normals)
        group.shift((2, -1, 0))
        np.testing.assert_allclose(surface.get_points(), points + (2, -1, 0), atol=1e-6)
        np.testing.assert_allclose(normals, seed + (2, -1, 0), atol=1e-6)

    def test_animation_readbacks_keep_normals_and_native_output(self):
        surface = self.surface()
        scene = m.Scene()
        scene.add(surface)
        target = surface.copy().shift((2, 0, 0))
        animation = m.Transform(surface, target, rate_func=m.linear)
        animation.begin()
        for alpha in (.25, .5, .75, 1.0):
            animation.interpolate(alpha)
            self.assert_normals(surface, (0, 0, 1))
        animation.finish()
        np.testing.assert_allclose(surface.get_center(), (2, 0, 0), atol=1e-5)
        camera = m.Camera(resolution=(96, 64), background_opacity=0)
        self.assertTrue(np.frombuffer(camera.capture_snapshot(surface).pixels(), dtype=np.uint8).any())


if __name__ == '__main__':
    unittest.main()
