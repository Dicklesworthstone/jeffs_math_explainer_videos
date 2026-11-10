"""Pointwise authoring through complete native fills, glyphs and frame output.

Requires the retained fill implementation in main as well as the public callback
adapter. There are no renderer, record-buffer or animation doubles in this suite.
"""
import gc
from pathlib import Path
import tempfile
import unittest

import numpy as np
import manimlib as m


class PointwisePaintIntegration(unittest.TestCase):
    def setUp(self):
        self.camera = m.Camera()
        self.camera.reset_pixel_shape(128, 96)

    def square(self):
        return m.Square(side_length=4, fill_color="#ff0000", fill_opacity=1, stroke_width=0)

    def pixels(self, *objects):
        return np.frombuffer(self.camera.capture_snapshot(*objects).pixels(), np.uint8).reshape(96, 128, 4).copy()

    def test_interior_callback_colors_reach_pixels_when_both_endpoints_match(self):
        square = self.square()
        geometry = square.get_points().copy()
        first = self.pixels(square)
        def colors(points):
            rgba = np.tile([1., 0., 0., 1.], (len(points), 1))
            rgba[4] = [0., 0., 1., 1.]
            return rgba
        square.set_color_by_rgba_func(colors)
        after = self.pixels(square)
        self.assertFalse(np.array_equal(first, after))
        self.assertGreater(int(after[:, :, 2].sum()), 0)
        np.testing.assert_array_equal(geometry, square.get_points())
        np.testing.assert_array_equal(after, self.pixels(square))
        square.set_color_by_rgb_func(lambda points: [1., 0., 0.])
        np.testing.assert_array_equal(first, self.pixels(square))

    def test_callback_alpha_does_not_allow_opaque_endpoint_occlusion(self):
        background, foreground = self.square(), self.square()
        def colors(points):
            rgba = np.tile([0., 0., 1., 0.], (len(points), 1))
            rgba[[0, -1], 3] = 1.
            return rgba
        foreground.set_color_by_rgba_func(colors)
        combined = self.pixels(background, foreground)
        # The entire interior is translucent, not just one isolated color knot.
        # Endpoint-only visibility pruning incorrectly makes this exactly zero.
        self.assertGreater(int(combined[48, 64, 0]), 150)
        self.assertGreater(int(combined[:, :, 2].sum()), 0)
        np.testing.assert_array_equal(combined, self.pixels(background, foreground))

    def test_real_text_and_math_glyphs_accept_vectorized_color_callbacks(self):
        for text in (m.Text("AB"), m.Tex("x+y")):
            with self.subTest(kind=type(text).__name__):
                members = tuple(text.get_family())
                geometry = [mob.get_points().copy() for mob in members]
                calls = []
                def blue(points):
                    calls.append(points.shape)
                    return np.tile([0., 0., 1.], (len(points), 1))
                self.assertIs(text.set_color_by_rgb_func(blue), text)
                self.assertEqual(len(calls), len(text.family_members_with_points()))
                self.assertTrue(calls)
                self.assertEqual(tuple(text.get_family()), members)
                for member, points in zip(members, geometry):
                    np.testing.assert_array_equal(member.get_points(), points)
                pixels = self.pixels(text)
                self.assertGreater(int(pixels[:, :, 2].sum()), 0)
                self.assertEqual(int(pixels[:, :, 0].sum()), 0)
                self.assertEqual(int(pixels[:, :, 1].sum()), 0)

    def test_interior_only_live_fill_updates_render_identically_across_threads(self):
        class PaintFrames(m.Scene):
            default_camera_config = dict(resolution=(96, 54), fps=4)
            def construct(self):
                phase = m.ValueTracker(0)
                square = m.Square(side_length=4, fill_opacity=1, stroke_width=0)
                def paint(points):
                    values = np.tile([1., 0., 0., 1.], (len(points), 1))
                    alpha = phase.get_value()
                    values[4] = [1. - alpha, 0., alpha, 1.]
                    return values
                square.add_updater(lambda mob: mob.set_color_by_rgba_func(paint))
                self.add(phase, square)
                self.play(phase.animate.set_value(1), run_time=1., rate_func=m.linear)
                square.clear_updaters()
        root = Path(tempfile.mkdtemp(prefix="fmn-pointwise-fill-"))
        outputs = []
        for threads in (1, 4):
            receipt = PaintFrames().render(root / str(threads), threads=threads)
            frames = sorted(receipt.destination.glob("*.png"))
            self.assertEqual(len(frames), 4)
            self.assertEqual(receipt.frame_count, 4)
            payloads = [frame.read_bytes() for frame in frames]
            self.assertTrue(all(p.startswith(b"\x89PNG\r\n\x1a\n") for p in payloads))
            self.assertEqual(len(set(payloads)), 4)
            outputs.append(payloads)
        self.assertEqual(outputs[0], outputs[1])
        print(f"retained pointwise-fill output: {root}")


suite = unittest.defaultTestLoader.loadTestsFromTestCase(PointwisePaintIntegration)
assert suite.countTestCases() == 4
result = unittest.TextTestRunner(verbosity=2).run(suite)
gc.collect()
if not result.wasSuccessful():
    raise AssertionError("native pointwise-paint integration failed")
