"""Native authored fill profiles through animation, updaters and frame emission."""
from pathlib import Path
import tempfile
import unittest
import gc

import numpy as np
import manimlib as m


class FillProfileAnimation(unittest.TestCase):
    def square(self):
        return m.Square(side_length=3, stroke_width=0, fill_opacity=1, fill_color=m.RED)

    def capture(self, *mobs):
        camera = m.Camera()
        camera.reset_pixel_shape(96, 64)
        return camera.capture_snapshot(*mobs).pixels()

    def render_frames(self, destination, threads, action):
        scene = m.Scene()
        with scene.render_session(destination, format="png_sequence",
                                  resolution=(96, 64), fps=8, threads=threads):
            action(scene)
        return [file.read_bytes() for file in sorted(Path(destination).glob("*.png"))]

    def test_public_multicolor_fill_does_not_collapse_equal_endpoints(self):
        mob = self.square()
        flat = self.capture(mob)
        mob.set_fill([m.RED, m.BLUE, m.RED], opacity=[0, 1, 0])
        painted = self.capture(mob)
        self.assertNotEqual(flat, painted)
        pixels = np.frombuffer(painted, np.uint8).reshape(64, 96, 4)
        self.assertGreater(int(pixels[:, :, 2].sum()), 0)
        self.assertGreater(int(pixels[:, :, 0].sum()), 0)
        self.assertEqual(painted, self.capture(mob))

    def test_transform_interpolates_interior_paint_as_real_distinct_frames(self):
        def action(scene):
            mob = self.square()
            scene.add(mob)
            target = mob.copy().set_fill([m.RED, m.BLUE, m.RED])
            scene.play(m.Transform(mob, target, rate_func=m.linear), run_time=.5)
            np.testing.assert_array_equal(mob.data['fill_rgba'], target.data['fill_rgba'])
            self.assertFalse(mob._is_updating_suspended())
        with tempfile.TemporaryDirectory(prefix="fmn-fill-transform-") as directory:
            root = Path(directory)
            one = self.render_frames(root / "one", 1, action)
            self.assertEqual(len(one), 4)
            self.assertEqual(len(set(one)), 4, "native Transform discarded interior paint")
            self.assertEqual(one, self.render_frames(root / "four", 4, action))

    def test_live_view_updater_frames_match_independently_authored_references(self):
        def action(scene, static=None):
            back, front = self.square().set_fill(m.BLUE), self.square()
            scene.add(back, front)
            front.set_fill(m.RED, opacity=0)
            view = front.data['fill_rgba']
            index = len(view) // 2
            if static is not None:
                view[index, 3] = static
                scene.wait(.125)
                return
            time = [0.]
            def paint(owner, dt):
                time[0] += dt
                view[index, 3] = min(1., time[0] * 2)
            front.add_updater(paint, call=False)
            scene.wait(.5)
        with tempfile.TemporaryDirectory(prefix="fmn-fill-updater-") as directory:
            root = Path(directory)
            one = self.render_frames(root / "one", 1, action)
            self.assertEqual(len(one), 4)
            self.assertEqual(len(set(one)), 4, "live fill-alpha updates vanished")
            self.assertEqual(one, self.render_frames(root / "four", 4, action))
            for index, alpha in enumerate((.25, .5, .75, 1.)):
                reference = self.render_frames(root / str(index), 1,
                    lambda scene, alpha=alpha: action(scene, alpha))
                self.assertEqual(one[index], reference[0])

    def test_saved_state_restores_full_profile_and_captured_pixels(self):
        mob = self.square().set_fill([m.RED, m.BLUE, m.RED], opacity=[.2, 1, .2])
        scene = m.Scene()
        scene.add(mob)
        original = mob.data.copy()
        expected = self.capture(mob)
        mob.save_state()
        mob.set_fill([m.GREEN, m.RED, m.GREEN], opacity=[1, .2, 1]).shift(m.RIGHT)
        self.assertNotEqual(expected, self.capture(mob))
        scene.play(m.Restore(mob, rate_func=m.linear), run_time=.125)
        np.testing.assert_array_equal(mob.data, original)
        self.assertEqual(self.capture(mob), expected)
        self.assertIn(mob, scene.mobjects)


suite = unittest.defaultTestLoader.loadTestsFromTestCase(FillProfileAnimation)
assert suite.countTestCases() == 4
result = unittest.TextTestRunner(verbosity=2).run(suite)
gc.collect()
if not result.wasSuccessful():
    raise AssertionError('native animated fill-profile acceptance failed')
