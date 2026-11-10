"""Surrounding and background rectangle lifecycle over actual native records."""
import copy
import os
from pathlib import Path
import tempfile
import unittest

import numpy as np
import manimlib as m
from manimlib.mobject.shape_matchers import SurroundingRectangle as QualifiedSurrounding


class NativeShapeMatcherLifecycle(unittest.TestCase):
    def assert_points(self, actual, expected):
        np.testing.assert_allclose(actual, expected, rtol=2e-5, atol=2e-5)

    def test_hooks_run_once_before_virtual_surround(self):
        events = []
        class Authored(m.SurroundingRectangle):
            def init_data(self):
                events.append("data")
                super().init_data()
            def init_points(self):
                events.append("points")
                super().init_points()
            def init_uniforms(self):
                events.append("uniforms")
                super().init_uniforms()
                self.uniforms["author_uniform"] = 7.0
            def init_colors(self):
                events.append("colors")
                super().init_colors()
            def surround(self, target, buff=None):
                events.append("surround")
                return super().surround(target, buff)
        target = m.Square().shift(m.UP)
        obj = Authored(target, buff=.3)
        self.assertEqual(events, ["data", "points", "uniforms", "colors", "surround"])
        self.assertEqual(obj.uniforms["author_uniform"], 7)
        self.assertIs(QualifiedSurrounding, m.SurroundingRectangle)
        self.assertAlmostEqual(obj.get_width(), 2.6, places=5)
        self.assertAlmostEqual(obj.get_height(), 2.6, places=5)
        self.assert_points(obj.get_center(), target.get_center())

    def test_authored_rounded_outline_is_preserved_by_padding_changes(self):
        calls = []
        class Authored(m.SurroundingRectangle):
            def init_points(self):
                calls.append("points")
                self.set_points(m.RoundedRectangle(width=4, height=2, corner_radius=.4).get_points())
        target = m.Rectangle(width=2, height=1).shift(m.RIGHT)
        obj = Authored(target)
        expected = m.RoundedRectangle(width=4, height=2, corner_radius=.4)
        expected.surround(target, .1)
        self.assert_points(obj.get_points(), expected.get_points())
        count = obj.get_num_points()
        obj.set_buff(.5)
        expected.surround(target, .5)
        self.assert_points(obj.get_points(), expected.get_points())
        self.assertEqual(obj.get_num_points(), count)
        self.assertEqual(calls, ["points"])

    def test_custom_dtype_children_and_updaters_survive_live_resize(self):
        class Authored(m.SurroundingRectangle):
            data_dtype = m.VMobject.data_dtype + [("heat", 1)]
            def init_data(self):
                self.resize(1)
                self.set_field("heat", 0, [.75])
                self.marker = m.VectorizedPoint(m.ORIGIN)
                self.add(self.marker)
        target = m.Square()
        obj = Authored(target, color=m.BLUE)
        scene = m.Scene()
        scene.add(obj)
        ticks = []
        def updater(current, dt): ticks.append(dt)
        obj.add_updater(updater, call=False)
        identity = hash(obj)
        obj.data["heat"][:] = .25
        obj.uniforms["author_uniform"] = 3.0
        target.shift(m.UP).stretch(2, 0)
        self.assertIs(obj.surround(target, .4), obj)
        self.assertEqual(hash(obj), identity)
        self.assertIs(scene.mobjects[0], obj)
        self.assertIs(obj[0], obj.marker)
        self.assertIs(obj.updaters[0], updater)
        self.assert_points(obj.data["heat"], .25)
        self.assertEqual(obj.uniforms["author_uniform"], 3.0)
        self.assertEqual(obj.get_stroke_color(), m.BLUE)
        self.assert_points(obj.get_center(), target.get_center())

    def test_background_resize_preserves_actual_nonblack_paint(self):
        for bound in (False, True):
            with self.subTest(bound=bound):
                target = m.Square()
                obj = m.BackgroundRectangle(target, color=m.BLUE, fill_opacity=.6)
                # Read native lanes, not BackgroundRectangle.get_fill_color's
                # saved constructor attribute, which could hide black records.
                obj.set_fill(color=m.GREEN, opacity=.4)
                before = obj.data["fill_rgba"].copy()
                scene = m.Scene()
                if bound: scene.add(obj)
                target.scale(2).shift(m.RIGHT)
                obj.surround(target)
                np.testing.assert_array_equal(obj.data["fill_rgba"], before)
                obj.set_buff(.25)
                np.testing.assert_array_equal(obj.data["fill_rgba"], before)
                self.assertEqual(obj.original_fill_opacity, .6)

    def test_explicit_style_override_still_has_reference_black_behavior(self):
        obj = m.BackgroundRectangle(m.Square(), color=m.BLUE)
        obj.set_style(fill_opacity=.2)
        self.assert_points(obj.data["fill_rgba"][:, :3], 0)
        self.assert_points(obj.data["fill_rgba"][:, 3], .2)
        self.assertEqual(obj.get_stroke_width(), 0)

    def test_default_color_is_the_configured_camera_background(self):
        # fm-78b6: the Reference reads manim_config.camera.background_color,
        # so a cwd custom_config.yml background (the corpus sets black)
        # colors the rectangle; #333333 is only the pinned default.
        self.assertEqual(m.BackgroundRectangle(m.Square()).get_fill_color(), "#333333")
        original = Path.cwd()
        with tempfile.TemporaryDirectory() as directory:
            Path(directory, "custom_config.yml").write_text('camera:\n  background_color: "#123456"\n')
            try:
                os.chdir(directory)
                obj = m.BackgroundRectangle(m.Square())
            finally:
                os.chdir(original)
        self.assertEqual((obj.get_fill_color(), obj.get_stroke_color()), ("#123456", "#123456"))
        self.assertEqual(obj.get_fill_colors()[0], "#123456")

    def test_set_buff_dispatches_a_single_argument_subclass_hook(self):
        calls = []
        class Authored(m.SurroundingRectangle):
            def surround(self, target):
                calls.append(self.buff)
                return super().surround(target)
        obj = Authored(m.Square())
        self.assertIs(obj.set_buff(.5), obj)
        self.assertEqual(calls, [.1, .5])
        self.assertAlmostEqual(obj.get_width(), 3.0, places=5)

    def test_fixed_in_frame_and_explicit_channel_colors_survive(self):
        target = m.Square().fix_in_frame()
        obj = m.SurroundingRectangle(target, color=m.BLUE, stroke_color=m.RED,
                                     fill_color=m.GREEN, fill_opacity=.3)
        self.assertTrue(obj.is_fixed_in_frame())
        self.assertEqual(obj.get_stroke_color(), m.RED)
        self.assertEqual(obj.get_fill_color(), m.GREEN)
        obj.set_buff(.4)
        self.assertTrue(obj.is_fixed_in_frame())
        self.assertEqual(obj.get_stroke_color(), m.RED)
        self.assertEqual(obj.get_fill_color(), m.GREEN)

    def test_copies_keep_authored_type_and_independent_targets(self):
        calls = []
        class Authored(m.SurroundingRectangle):
            def init_points(self):
                calls.append("points")
                super().init_points()
        target = m.Square()
        obj = Authored(target)
        before = obj.get_points().copy()
        for clone in (obj.copy(), copy.deepcopy(obj)):
            self.assertIs(type(clone), Authored)
            other = m.Square().shift(2*m.UP)
            clone.surround(other, .4)
            self.assertIs(clone.mobject, other)
        self.assertIs(obj.mobject, target)
        self.assertEqual(calls, ["points"])
        np.testing.assert_array_equal(obj.get_points(), before)

    def test_hook_failure_is_not_swallowed_or_followed_by_sizing(self):
        seen, error = [], RuntimeError("authored background failure")
        class Authored(m.BackgroundRectangle):
            def init_colors(self):
                seen.append("colors")
                raise error
            def surround(self, *args, **kwargs): seen.append("surround")
        with self.assertRaises(RuntimeError) as caught: Authored(m.Square())
        self.assertIs(caught.exception, error)
        self.assertEqual(seen, ["colors"])

    def test_nonfinite_padding_cannot_corrupt_bound_geometry(self):
        target = m.Square()
        obj = m.SurroundingRectangle(target)
        scene = m.Scene()
        scene.add(obj)
        before = obj.get_points().copy()
        for value in (float("nan"), float("inf"), -float("inf")):
            with self.assertRaises(ValueError): obj.set_buff(value)
            with self.assertRaises(ValueError): obj.surround(m.Square(), value)
        np.testing.assert_array_equal(obj.get_points(), before)
        self.assertIs(obj.mobject, target)
        self.assertEqual(obj.buff, .1)

    def test_live_surround_matches_native_affine_oracle_at_one_four_threads(self):
        directory = Path(tempfile.mkdtemp(prefix="fmn-matcher-lifecycle-"))
        def render(name, threads, public):
            scene = m.Scene()
            destination = directory/name
            with scene.render_session(destination, format="png_sequence", fps=4,
                                      resolution=(96,54), threads=threads):
                target = m.Square().shift(m.LEFT)
                if public:
                    class Authored(m.SurroundingRectangle):
                        def init_points(self):
                            self.set_points(m.RoundedRectangle(corner_radius=.4).get_points())
                    obj = Authored(target, color=m.GREEN, fill_opacity=.25)
                else:
                    obj = m.RoundedRectangle(corner_radius=.4, color=m.GREEN, fill_opacity=.25)
                    obj.surround(target, .1)
                scene.add(obj)
                for padding in (.1, .2, .3, .4):
                    target.shift(.25*m.RIGHT)
                    obj.surround(target, padding)
                    scene.wait(.25)
            return [p.read_bytes() for p in sorted(destination.glob("*.png"))]
        frames = render("one", 1, True)
        self.assertEqual(len(frames), 4)
        self.assertEqual(frames, render("four", 4, True))
        self.assertEqual(frames, render("oracle", 1, False))
        self.assertGreater(len(set(frames)), 1)
        print("retained surrounding rectangle frames:",directory)


if __name__ == "__main__":
    unittest.main()
else:
    result = unittest.TextTestRunner(verbosity=2).run(
        unittest.defaultTestLoader.loadTestsFromTestCase(NativeShapeMatcherLifecycle))
    if not result.wasSuccessful():
        raise AssertionError("native shape-matcher lifecycle acceptance failed")
