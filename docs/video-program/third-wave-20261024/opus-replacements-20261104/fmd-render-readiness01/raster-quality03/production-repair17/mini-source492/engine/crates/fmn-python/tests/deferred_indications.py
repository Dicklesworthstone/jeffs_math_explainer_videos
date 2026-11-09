"""Real-native deferred grow/indication targets and sampled-frame witnesses.

Use the installed portal. No native geometry, driver, or renderer is doubled.
"""
from pathlib import Path
import tempfile
import unittest

import manimlib as m
from manimlib.animation.growing import SpinInFromNothing
import numpy as np


class DeferredIndications(unittest.TestCase):
    def assert_points(self, actual, expected):
        np.testing.assert_allclose(actual, expected, atol=3e-5, rtol=3e-5)

    def scene(self, obj, fps=8):
        scene = m.Scene(camera_config=dict(fps=fps))
        scene.add(obj)
        return scene

    def test_all_grow_spellings_observe_predecessor_geometry(self):
        cases = (
            (m.GrowFromPoint, (3 * m.LEFT,)),
            (m.GrowFromCenter, ()),
            (m.GrowFromEdge, (m.DOWN,)),
            (SpinInFromNothing, ()),
        )
        for cls, args in cases:
            with self.subTest(animation=cls.__name__):
                obj = m.Square()
                scene = self.scene(obj)
                effect = cls(obj, *args, run_time=.25, rate_func=m.linear)
                anchor = effect.point.copy()
                expected = obj.copy().shift(2 * m.RIGHT).scale(2)
                scene.play(m.Succession(
                    obj.animate(run_time=.25).shift(2 * m.RIGHT).scale(2), effect))
                self.assert_points(obj.get_points(), expected.get_points())
                self.assert_points(effect.point, anchor)

    def test_grow_arrow_keeps_authored_anchor_and_live_target(self):
        obj = m.Arrow(m.LEFT, m.RIGHT, buff=0)
        scene = self.scene(obj)
        effect = m.GrowArrow(obj, run_time=.25, rate_func=m.linear)
        anchor = effect.point.copy()
        expected = obj.copy().shift(2 * m.UP)
        scene.play(m.Succession(obj.animate(run_time=.25).shift(2 * m.UP), effect))
        self.assert_points(obj.get_points(), expected.get_points())
        self.assert_points(effect.point, anchor)

    def test_indicate_uses_live_size_placement_and_paint(self):
        obj = m.Square(fill_opacity=.4)
        scene = self.scene(obj)
        effect = m.Indicate(obj, scale_factor=1.5, color=m.YELLOW,
                            run_time=.25, rate_func=m.linear)
        expected = obj.copy().shift(2 * m.RIGHT).scale(2).scale(1.5).set_color(m.YELLOW)
        scene.play(m.Succession(
            obj.animate(run_time=.25).shift(2 * m.RIGHT).scale(2).set_color(m.GREEN), effect))
        self.assert_points(obj.get_points(), expected.get_points())
        self.assertEqual(obj.get_color(), m.YELLOW)
        self.assertAlmostEqual(obj.get_fill_opacity(), .4, places=5)

    def test_default_indicate_returns_to_live_begin_state(self):
        obj = m.Square()
        scene = self.scene(obj)
        effect = m.Indicate(obj, run_time=.25)
        expected = obj.copy().shift(2 * m.RIGHT).scale(2).set_color(m.GREEN)
        scene.play(m.Succession(
            obj.animate(run_time=.25).shift(2 * m.RIGHT).scale(2).set_color(m.GREEN), effect))
        self.assert_points(obj.get_points(), expected.get_points())
        self.assertEqual(obj.get_color(), m.GREEN)

    def test_inside_out_reverses_live_points(self):
        obj = m.VMobject().set_points_as_corners([m.LEFT, m.UP, m.RIGHT])
        expected = (obj.get_points().copy() + 2 * m.RIGHT)[::-1]
        scene = self.scene(obj)
        effect = m.TurnInsideOut(obj, path_arc=0, run_time=.25, rate_func=m.linear)
        scene.play(m.Succession(obj.animate(run_time=.25).shift(2 * m.RIGHT), effect))
        self.assert_points(obj.get_points(), expected)

    def test_target_hook_runs_once_at_begin_not_during_lowering(self):
        obj, calls = m.Square(), []
        scene = self.scene(obj)
        class Authored(m.GrowFromCenter):
            def create_target(self):
                calls.append(self.mobject.get_center().copy())
                return super().create_target()
        effect = Authored(obj, run_time=.25, rate_func=m.linear)
        self.assertEqual(calls, [])
        scene.play(m.Succession(obj.animate(run_time=.25).shift(2 * m.RIGHT), effect))
        self.assertEqual(len(calls), 1)
        self.assert_points(calls[0], 2 * m.RIGHT)

    def test_replayed_effect_rebuilds_target_despite_retained_target_attribute(self):
        for cls in (m.GrowFromCenter, m.Indicate, m.TurnInsideOut):
            with self.subTest(animation=cls.__name__):
                obj, calls = m.Square(), []
                scene = self.scene(obj)
                class Authored(cls):
                    def create_target(self):
                        calls.append(self.mobject.get_center().copy())
                        return super().create_target()
                effect = Authored(obj, run_time=.25)
                scene.play(effect)
                first = effect.target_mobject
                obj.shift(3 * m.RIGHT)
                scene.play(effect)
                self.assertEqual(len(calls), 2)
                self.assert_points(calls[1], 3 * m.RIGHT)
                self.assertIsNot(first, effect.target_mobject)
                self.assert_points(obj.get_center(), 3 * m.RIGHT)

    def test_nested_group_lag_and_partial_endpoint_use_shared_transform(self):
        obj = m.VGroup(m.Square().shift(m.LEFT), m.Circle().shift(m.RIGHT))
        scene = self.scene(obj)
        expected = obj.copy().shift(m.UP)
        effect = m.GrowFromPoint(obj, m.ORIGIN, final_alpha_value=.5,
                                 run_time=.25, lag_ratio=.2, rate_func=m.linear)
        # A direct invocation of the same public lifecycle is an independent
        # scheduling witness, not a replacement interpolation implementation.
        direct = m.GrowFromPoint(expected, m.ORIGIN, final_alpha_value=.5,
                                 run_time=.25, lag_ratio=.2, rate_func=m.linear)
        direct.begin()
        direct.finish()
        scene.play(m.Succession(
            obj.animate(run_time=.25).shift(m.UP), m.AnimationGroup(effect)))
        self.assert_points(obj.get_all_points(), expected.get_all_points())

    def test_remover_cleanup_and_previous_suspension_survive(self):
        obj = m.Square()
        obj.suspend_updating()
        scene = self.scene(obj)
        effect = m.GrowFromCenter(obj, run_time=.25, remover=True,
                                  suspend_mobject_updating=True)
        scene.play(effect)
        self.assertNotIn(obj, scene.mobjects)
        self.assertTrue(obj._is_updating_suspended())

    def test_failed_target_does_not_execute_future_targets_or_undo_predecessor(self):
        obj, calls = m.Square(), []
        scene = self.scene(obj)
        failure = ValueError("deferred indication target failed")
        class Broken(m.GrowFromCenter):
            def create_target(self):
                calls.append("broken")
                raise failure
        class Future(m.Indicate):
            def create_target(self):
                calls.append("future")
                return super().create_target()
        with self.assertRaises(ValueError) as caught:
            scene.play(m.Succession(obj.animate(run_time=.25).shift(m.RIGHT),
                Broken(obj, run_time=.25), Future(obj, run_time=.25)))
        self.assertIs(caught.exception, failure)
        self.assertEqual(calls, ["broken"])
        self.assert_points(obj.get_center(), m.RIGHT)
        self.assertFalse(obj._is_updating_suspended())
        scene.play(m.GrowFromCenter(obj, run_time=.25))
        self.assert_points(obj.get_center(), m.RIGHT)

    def test_persistent_succession_uses_the_same_deferred_protocol(self):
        obj = m.Square()
        scene = self.scene(obj)
        sequence = m.Succession(obj.animate(run_time=.25).shift(2 * m.RIGHT),
                                m.GrowFromCenter(obj, run_time=.25))
        anchor = m.turn_animation_into_updater(sequence)
        scene.add(anchor)
        scene.wait(.75)
        self.assert_points(obj.get_center(), 2 * m.RIGHT)
        self.assert_points(obj.get_width(), 2)
        self.assertFalse(anchor.updaters)

    def test_authored_rate_evaluated_at_actual_samples_at_24_30_60_fps(self):
        for fps in (24, 30, 60):
            for cls in (m.GrowFromCenter, m.Indicate, m.TurnInsideOut):
                with self.subTest(fps=fps, animation=cls.__name__):
                    obj, calls, seen = m.Square(), [], []
                    scene = m.Scene()
                    class Sampled(cls):
                        def interpolate_submobject(self, current, start, target, alpha):
                            if current is self.mobject:
                                seen.append(float(alpha))
                            return super().interpolate_submobject(current, start, target, alpha)
                    effect = Sampled(obj, run_time=.5)
                    class Rate:
                        __hash__ = None
                        def __call__(self, alpha):
                            self_marker = getattr(effect, "starting_mobject", None)
                            assert self_marker is not None, "rate sampled before begin"
                            calls.append(float(alpha))
                            return alpha ** 7
                    effect.rate_func = Rate()
                    directory = Path(tempfile.mkdtemp(prefix="fmn-indication-rate-"))
                    with scene.render_session(directory / "motion.y4m", format="y4m",
                                              fps=fps, resolution=(32, 18), threads=1):
                        scene.add(obj)
                        scene.play(effect)
                    alphas = [0, *[i / (fps * .5) for i in range(1, fps // 2 + 1)], 1]
                    self.assert_points(calls, alphas)
                    self.assert_points(seen, np.asarray(alphas) ** 7)

    def test_actual_grow_frames_match_independently_positioned_geometry(self):
        root = Path(tempfile.mkdtemp(prefix="fmn-deferred-grow-"))
        def render(destination, threads, use_effect):
            scene = m.Scene()
            with scene.render_session(destination, format="png_sequence",
                                      resolution=(96, 54), fps=4, threads=threads):
                obj = m.Square(side_length=.5, fill_opacity=1, stroke_width=0)
                scene.add(obj)
                if use_effect:
                    effect = m.GrowFromCenter(obj, run_time=1, rate_func=m.linear)
                    scene.play(m.Succession(
                        obj.animate(run_time=1, rate_func=m.linear).shift(2*m.RIGHT).scale(2), effect))
                else:
                    # Use the same shared Succession clock, including the frame
                    # at the child boundary (the next child's begin has run).
                    # The geometry witness is independent of Grow/Transform.
                    class Expected(m.Animation):
                        def interpolate_mobject(self, alpha):
                            self.mobject.become(m.Square(side_length=1, fill_opacity=1,
                                stroke_width=0).scale(alpha).move_to(2 * alpha * m.RIGHT))
                    scene.play(m.Succession(
                        obj.animate(run_time=1, rate_func=m.linear).shift(2*m.RIGHT).scale(2),
                        Expected(obj, run_time=1)))
            return [p.read_bytes() for p in sorted(destination.glob("*.png"))]
        actual = render(root / "one", 1, True)
        self.assertEqual(actual, render(root / "four", 4, True))
        self.assertEqual(actual, render(root / "expected", 1, False))
        self.assertEqual(len(actual), 8)
        self.assertGreater(len(set(actual[4:])), 2)
        print("retained deferred-indication frames:", root)


if __name__ == "__main__":
    unittest.main()
else:
    result = unittest.TextTestRunner(verbosity=2).run(
        unittest.defaultTestLoader.loadTestsFromTestCase(DeferredIndications))
    if not result.wasSuccessful():
        raise AssertionError("deferred indication acceptance failed")
