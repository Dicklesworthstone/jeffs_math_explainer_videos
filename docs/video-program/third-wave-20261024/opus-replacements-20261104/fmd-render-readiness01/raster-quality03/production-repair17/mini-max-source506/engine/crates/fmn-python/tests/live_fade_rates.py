"""Installed-extension fade easing over the production renderer, not a 30-Hz table.

Run against the freshly built wheel: python crates/fmn-python/tests/live_fade_rates.py
Missing native dependencies are errors, not skipped acceptance cases.
"""
from pathlib import Path
import tempfile
import unittest

import numpy as np
import manimlib as m
from fmn_python import render_session


class LiveFadeRates(unittest.TestCase):
    def test_stock_fades_sample_the_actual_frame_clock(self):
        for fps in (24, 60, 120):
            for kind in (m.FadeIn, m.FadeOut, m.FadeInFromLarge):
                with self.subTest(fps=fps, kind=kind.__name__), tempfile.TemporaryDirectory() as tmp:
                    scene = m.Scene()
                    square = m.Square(fill_opacity=.8, stroke_opacity=.6)
                    animation = kind(square, run_time=.5)
                    sampled, observed = [], []

                    def rate(t):
                        sampled.append(float(t))
                        return t ** 3

                    animation.rate_func = rate
                    probe = m.Mobject()
                    probe.add_updater(lambda obj, dt: observed.append(
                        (square.get_fill_opacity(), square.get_stroke_opacity())) if dt > 0 else None)
                    with render_session(scene, Path(tmp) / "fade.y4m", fps=fps,
                                        resolution=(32, 18), threads=1) as output:
                        scene.add(square, probe)
                        scene.play(animation)
                    self.assertEqual(output.result.frame_count, fps // 2)
                    self.assertEqual(len(observed), fps // 2)
                    expected = (np.arange(1, fps // 2 + 1) / (fps / 2)) ** 3
                    if kind is m.FadeOut:
                        expected = 1 - expected
                    np.testing.assert_allclose(observed, expected[:, None] * [.8, .6], atol=2e-6)
                    self.assertTrue(any(abs(t - 2 / fps) < 1e-13 for t in sampled), sampled)
                    self.assertFalse(square._is_updating_suspended())
                    self.assertEqual(square in scene.mobjects, kind is not m.FadeOut)

    def test_global_curve_is_live_for_a_mixed_fade_and_transform(self):
        for fps in (24, 60, 120):
            with self.subTest(fps=fps), tempfile.TemporaryDirectory() as tmp:
                scene, fading, moving = m.Scene(), m.Square(fill_opacity=1), m.Square()
                moving.shift(3 * m.LEFT)
                observed = []
                probe = m.Mobject()
                probe.add_updater(lambda obj, dt: observed.append(
                    (fading.get_fill_opacity(), moving.get_center()[0])) if dt > 0 else None)
                with render_session(scene, Path(tmp) / "mixed.y4m", fps=fps,
                                    resolution=(32, 18), threads=1):
                    scene.add(fading, moving, probe)
                    scene.play(m.FadeIn(fading), m.Transform(moving, moving.copy().shift(m.RIGHT)),
                               run_time=.5, rate_func=lambda t: t ** 3)
                expected = (np.arange(1, fps // 2 + 1) / (fps / 2)) ** 3
                np.testing.assert_allclose(observed, np.column_stack((expected, -3 + expected)), atol=2e-6)


if __name__ == "__main__":
    unittest.main()
