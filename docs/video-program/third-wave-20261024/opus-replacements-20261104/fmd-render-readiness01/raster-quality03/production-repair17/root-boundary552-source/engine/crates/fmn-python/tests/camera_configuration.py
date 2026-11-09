"""Native scene camera configuration, live frame identity and actual pixels."""
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

import numpy as np
import manimlib as m


class CameraConfigurationTests(unittest.TestCase):
    def test_manim_config_carries_the_reference_sections(self):
        # fm-e661: manim_config is default_config.yml at the pin, with the
        # camera layer under the cwd custom_config.yml, read once at import
        # (as in the Reference), so the configured case runs in a child.
        config = m.manim_config
        self.assertEqual(dict(config.camera), {"resolution": (1920, 1080), "fps": 30,
                                               "background_color": "#333333",
                                               "background_opacity": 1.0})
        self.assertEqual(dict(config.vmobject), {"default_stroke_width": 4.0,
                                                 "default_stroke_color": "#DDDDDD",
                                                 "default_fill_color": "#888888"})
        self.assertEqual(config.tex.font_size_for_unit_height, 144)
        self.assertEqual(config.text.font_size_for_unit_height, 144)
        self.assertEqual(config.sizes.frame_height, 8.0)
        self.assertEqual(config.colors.blue_c, "#58C4DD")
        self.assertEqual(dict(config.camera), m.Scene().camera_config)
        probe = ("import manimlib as m; c = m.manim_config.camera; "
                 "print(c.background_color, c.fps, tuple(c.resolution))")
        with tempfile.TemporaryDirectory() as directory:
            Path(directory, "custom_config.yml").write_text(
                'camera:\n  background_color: "#000000"\n  fps: 24\n')
            child = subprocess.run([sys.executable, "-c", probe], cwd=directory,
                                   capture_output=True, text=True, timeout=120)
        self.assertEqual(child.returncode, 0, child.stderr)
        self.assertEqual(child.stdout.split(), ["#000000", "24", "(1920,", "1080)"])

    def test_scene_frame_uses_configured_pose_and_field_of_view(self):
        scene = m.Scene(camera_config={"frame_config": {
            "frame_shape": (10., 6.), "center_point": (2., -1., .5), "fovy": .7,
        }, "resolution": (120, 60)})
        frame = scene.frame
        np.testing.assert_array_equal(frame.get_center(), (2., -1., .5))
        self.assertEqual(frame.get_shape(), (10., 6.))
        self.assertAlmostEqual(frame.get_field_of_view(), .7)
        self.assertIs(scene.camera.frame, frame)
        self.assertEqual(frame.get_shape(), (10., 5.))
        np.testing.assert_array_equal(frame.get_center(), (2., -1., .5))

    def test_nested_constructor_config_merges_with_scene_defaults(self):
        class Configured(m.Scene):
            default_camera_config = {
                "resolution": (120, 80),
                "frame_config": {"frame_shape": (9., 5.), "fovy": .6, "euler_axes": "xyz"},
            }
        override = {"frame_config": {"center_point": (3., 2., 0.)}, "fps": 24}
        scene = Configured(camera_config=override)
        self.assertEqual(scene.camera_config["frame_config"]["frame_shape"], (9., 5.))
        self.assertEqual(scene.camera.get_frame_shape(), (9., 6.))
        self.assertAlmostEqual(scene.frame.get_field_of_view(), .6)
        self.assertEqual(scene.frame._core.euler_axes(), "xyz")
        np.testing.assert_array_equal(scene.frame.get_center(), (3., 2., 0.))
        self.assertEqual(scene.camera.fps, 24)
        self.assertEqual(override, {"frame_config": {"center_point": (3., 2., 0.)}, "fps": 24})
        self.assertNotIn("center_point", Configured.default_camera_config["frame_config"])

    def test_frame_recipe_dictionary_is_not_shared(self):
        options = {"frame_config": {"frame_shape": (9., 5.)}}
        scene = m.Scene(camera_config=options)
        options["frame_config"]["frame_shape"] = (100., 100.)
        self.assertEqual(scene.camera_config["frame_config"]["frame_shape"], (9., 5.))
        self.assertEqual(scene.frame.get_width(), 9.)
        scene.camera_config["frame_config"]["fovy"] = .8
        self.assertNotIn("fovy", options["frame_config"])

    def test_scene_default_orientation_applies_to_configured_frame(self):
        class Configured(m.Scene):
            default_frame_orientation = (25., 40., 10.)
            default_camera_config = {"frame_config": {"frame_shape": (7., 4.), "center_point": (1., 2., 0.)}}
        scene = Configured()
        expected = m.CameraFrame(frame_shape=(7., 4.), center_point=(1., 2., 0.)).reorient(25., 40., 10.)
        np.testing.assert_allclose(scene.frame.get_orientation(), expected.get_orientation())
        scene.frame.reorient(0, 0, 0).to_default_state()
        np.testing.assert_allclose(scene.frame.get_orientation(), expected.get_orientation())

    def test_lazy_capture_retains_preexisting_frame_edits_and_updaters(self):
        scene = m.Scene(camera_config={"resolution": (120, 80), "frame_config": {"frame_shape": (9., 6.)}})
        frame = scene.frame
        frame.shift(m.RIGHT).scale(.5).reorient(10., 25., 0.)
        updater = lambda mob, dt: None
        frame.add_updater(updater)
        before = frame.copy()
        camera = scene.camera
        self.assertIs(camera.frame, frame)
        self.assertIs(scene.frame, frame)
        np.testing.assert_allclose(frame.get_center(), before.get_center())
        np.testing.assert_allclose(frame.get_orientation(), before.get_orientation())
        self.assertEqual(frame.get_shape(), (4.5, 3.))
        self.assertIn(updater, frame.updaters)
        self.assertEqual(scene.mobjects, [])

    def test_aspect_ratio_uses_actual_live_width_not_stale_recipe(self):
        scene = m.Scene(camera_config={"resolution": (120, 60), "frame_config": {"frame_shape": (8., 8.)}})
        scene.frame.set_width(5., stretch=True)
        self.assertEqual(scene.camera.get_frame_shape(), (5., 2.5))
        scene.frame.set_height(7., stretch=True)
        self.assertIs(scene.camera, scene.camera)
        self.assertEqual(scene.frame.get_height(), 7., "repeated access must not reapply the initial correction")

    def test_explicit_frame_before_camera_is_not_discarded(self):
        scene = m.Scene(camera_config={"resolution": (120, 60)})
        frame = m.CameraFrame(frame_shape=(6., 4.), center_point=(3., 0., 0.), fovy=.8)
        scene.frame = frame
        self.assertIs(scene.camera.frame, frame)
        self.assertEqual(frame.get_shape(), (6., 3.))
        self.assertAlmostEqual(frame.get_field_of_view(), .8)

    def test_invalid_frame_config_is_rejected(self):
        for config in (None, 1, [], "bad"):
            with self.subTest(config=config), self.assertRaisesRegex(TypeError, "frame_config must be a dict"):
                m.Scene(camera_config={"frame_config": config})
        with self.assertRaises(ValueError):
            m.Scene(camera_config={"frame_config": {"frame_shape": (0., 4.)}})

    def test_unsupported_capture_option_still_fails_lazily(self):
        scene = m.Scene(camera_config={"background_image": "missing.png", "frame_config": {"center_point": (1, 0, 0)}})
        np.testing.assert_array_equal(scene.frame.get_center(), (1., 0., 0.))
        with self.assertRaises(NotImplementedError):
            _ = scene.camera
        self.assertNotIn("_camera", vars(scene))
        scene.camera_config.pop("background_image")
        self.assertIs(scene.camera.frame, scene.frame)

    def test_independent_scenes_do_not_share_camera_or_frame(self):
        config = {"resolution": (120, 60), "frame_config": {"center_point": (2., 0., 0.)}}
        first, second = m.Scene(camera_config=config), m.Scene(camera_config=config)
        first.frame.shift(m.RIGHT)
        self.assertIsNot(first.camera, second.camera)
        self.assertIsNot(first.frame._core, second.frame._core)
        np.testing.assert_array_equal(second.frame.get_center(), (2., 0., 0.))

    def test_direct_camera_and_scene_camera_capture_equal_pixels(self):
        config = {"resolution": (96, 54), "samples": 1, "frame_config": {
            "frame_shape": (5., 3.), "center_point": (2., 1., 0.), "fovy": .8,
        }}
        # A Scene's camera also inherits manim_config.camera (#333333); a
        # directly constructed Camera keeps the class default, as in the
        # Reference, so state the scene background for a pose-only comparison.
        direct = m.Camera(**config, background_color="#333333")
        scene = m.Scene(camera_config=config)
        obj = m.VGroup(m.Square(side_length=1.3, fill_color=m.RED, fill_opacity=1).move_to((2., 1., 0.)),
                       m.Triangle(fill_color=m.BLUE, fill_opacity=1).scale(.3).move_to((3., 1.5, 0.)))
        expected = direct.capture_snapshot(obj).png()
        actual = scene.camera.capture_snapshot(obj).png()
        self.assertEqual(actual, expected)
        wrong = m.Camera(resolution=(96, 54), samples=1,
                         background_color="#333333").capture_snapshot(obj).png()
        self.assertNotEqual(actual, wrong, "a discarded configured pose must visibly fail")

    def test_configured_frame_animation_matches_manual_pose_at_one_four_threads(self):
        def render(path, threads, configured, wrong=False):
            options = dict(resolution=(96, 54), fps=4, samples=1)
            if configured:
                options['frame_config'] = dict(frame_shape=(5., 3.), center_point=(2., 1., 0.), fovy=.8)
            scene = m.Scene(camera_config=options)
            # Explicit independent pose construction, not the configuration path.
            if not configured and not wrong:
                camera = scene.camera
                scene.frame.set_width(5., stretch=True).set_height(5./(96/54), stretch=True)
                scene.frame.move_to((2., 1., 0.)).set_field_of_view(.8)
            with scene.render_session(path, format='png_sequence', resolution=(96, 54), fps=4, threads=threads):
                square = m.Square(side_length=1.3, fill_color=m.RED, fill_opacity=1).move_to((2., 1., 0.))
                scene.add(square)
                scene.play(scene.frame.animate.shift(.5*m.RIGHT), square.animate.shift(.5*m.UP),
                           run_time=.75, rate_func=m.linear)
            return [p.read_bytes() for p in sorted(path.glob('*.png'))]
        with tempfile.TemporaryDirectory(prefix='fmn-camera-config-') as temp:
            root = Path(temp)
            actual = render(root/'one', 1, True)
            self.assertEqual(len(actual), 3)
            self.assertEqual(len(set(actual)), 3)
            self.assertEqual(actual, render(root/'four', 4, True))
            self.assertEqual(actual, render(root/'manual', 1, False))
            self.assertNotEqual(actual, render(root/'wrong', 1, False, True))


    def test_lazy_camera_does_not_evaluate_frame_recipe_twice(self):
        class Once:
            def __init__(self, value):
                self.value, self.calls = value, 0
            def __float__(self):
                self.calls += 1
                if self.calls != 1:
                    raise RuntimeError("frame recipe evaluated twice")
                return self.value
        fovy, width = Once(.8), Once(5.)
        scene = m.Scene(camera_config={"resolution": (96, 54), "samples": 1,
            "frame_config": {"frame_shape": (width, 3.), "fovy": fovy}})
        frame = scene.frame
        self.assertIs(scene.camera.frame, frame)
        self.assertEqual((fovy.calls, width.calls), (1, 1))
        self.assertEqual(frame.get_width(), 5.)
        self.assertAlmostEqual(frame.get_field_of_view(), .8)

    def test_caller_mutating_consumed_recipe_cannot_poison_capture(self):
        shape = [5., 3.]
        center = np.array([2., 1., 0.])
        scene = m.Scene(camera_config={"resolution": (96, 54), "samples": 1,
            "frame_config": {"frame_shape": shape, "center_point": center, "fovy": .8}})
        shape[:] = [0., float('nan')]
        center[:] = float('nan')
        obj = m.Square(side_length=1., fill_color=m.RED, fill_opacity=1).move_to((2., 1., 0.))
        expected = m.Camera(resolution=(96, 54), samples=1, background_color="#333333",
            frame_config={"frame_shape": (5., 3.), "center_point": (2., 1., 0.), "fovy": .8})
        self.assertEqual(scene.camera.capture_snapshot(obj).png(), expected.capture_snapshot(obj).png())
        np.testing.assert_array_equal(scene.frame.get_center(), (2., 1., 0.))

    def test_retrying_failed_capture_does_not_replay_recipe_callbacks(self):
        calls = []
        class Fovy:
            def __float__(self):
                calls.append("fovy")
                return .8
        scene = m.Scene(camera_config={"resolution": (0, 54),
            "frame_config": {"fovy": Fovy(), "center_point": (2., 1., 0.)}})
        with self.assertRaises(ValueError):
            _ = scene.camera
        self.assertNotIn("_camera", vars(scene))
        self.assertEqual(calls, ["fovy"])
        scene.frame.shift(m.RIGHT)
        scene.camera_config["resolution"] = (96, 54)
        self.assertIs(scene.camera.frame, scene.frame)
        self.assertEqual(calls, ["fovy"])
        np.testing.assert_array_equal(scene.frame.get_center(), (3., 1., 0.))

    def test_consumed_frame_recipe_is_not_used_after_explicit_replacement(self):
        scene = m.Scene(camera_config={"resolution": (96, 54),
            "frame_config": {"frame_shape": (8., 4.)}})
        scene.camera_config["frame_config"] = {"fovy": object()}
        replacement = m.CameraFrame(frame_shape=(6., 4.), center_point=(3., 0., 0.), fovy=.7)
        scene.frame = replacement
        self.assertIs(scene.camera.frame, replacement)
        self.assertEqual(replacement.get_shape(), (6., 6. / (96/54)))
        self.assertAlmostEqual(replacement.get_field_of_view(), .7)


if __name__ == '__main__':
    unittest.main()
