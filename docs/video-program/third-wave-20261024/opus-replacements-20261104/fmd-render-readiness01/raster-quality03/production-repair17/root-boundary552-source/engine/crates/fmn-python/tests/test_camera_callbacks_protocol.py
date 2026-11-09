"""Authored camera playback using production protocols and fixture native storage."""
import ast
import unittest

import numpy as np

from test_camera_choreography_protocol import BOOTSTRAP, environment
from fmn_python.camera_callbacks import install_camera_callbacks


class CameraCallbackTests(unittest.TestCase):
    def setUp(self):
        self.g = environment()
        # Compile the actual public callback classes, not stand-in algorithms.
        tree = ast.parse(BOOTSTRAP.read_text())
        names = {"UpdateFromFunc", "UpdateFromAlphaFunc"}
        nodes = [node for node in tree.body
                 if isinstance(node, ast.ClassDef) and node.name in names]
        self.assertEqual({node.name for node in nodes}, names)
        exec(compile(ast.Module(body=nodes, type_ignores=[]), str(BOOTSTRAP), "exec"), self.g)
        # Preserve the same namespace captured by the production driver
        # factories; copying it would miss dynamic factory replacement.
        class Native:
            pass
        self.native = Native()
        self.native.__dict__ = self.g
        install_camera_callbacks(self.native)
        self.g = vars(self.native)
        self.scene = self.g["Scene"]()
        self.Animation, self.Group = self.g["Animation"], self.g["AnimationGroup"]

    def callback(self, function, mob=None, **kwargs):
        return self.g["UpdateFromAlphaFunc"](
            self.scene.frame if mob is None else mob, function,
            run_time=kwargs.pop("run_time", 1.),
            rate_func=kwargs.pop("rate_func", self.g["linear"]), **kwargs,
        )

    def move(self, x, **kwargs):
        def update(frame, alpha):
            frame.shift(np.array([x * alpha, 0., 0.]) - frame.get_center())
        return self.callback(update, **kwargs)

    def xs(self):
        return [float(position[0]) for _, position, _ in self.scene.samples]

    def assert_unmarked(self, *animations):
        for animation in animations:
            self.assertNotIn("_fmn_allow_camera_callback", vars(animation))

    def test_alpha_callback_uses_one_clock_and_retains_camera_identity(self):
        frame, core = self.scene.frame, self.scene.frame._core
        animation = self.move(4.)
        self.scene.play(animation)
        self.assertEqual(self.xs(), [1., 2., 3., 4.])
        self.assertEqual(len(self.scene.calls), 1)
        self.assertEqual(self.scene.calls[0][0], "native_clock")
        self.assertIs(self.scene.frame, frame)
        self.assertIs(frame._core, core)
        self.assertFalse(frame._is_bound())
        self.assertTrue(all(not roots for _, _, roots in self.scene.samples))
        self.assertEqual(self.scene.roots, [])
        self.assert_unmarked(animation)

    def test_callback_rate_time_window_and_final_alpha(self):
        alphas = []
        animation = self.callback(lambda frame, alpha: alphas.append(alpha),
                                  rate_func=lambda t: t * t, time_span=(.25, .75),
                                  final_alpha_value=.5)
        self.scene.play(animation)
        np.testing.assert_allclose(alphas, [0., 0., .25, 1., 1., .25])

    def test_catalog_rate_names_are_normalized_by_the_existing_camera_driver(self):
        animation = self.move(4., rate_func="linear")
        self.scene.play(animation)
        self.assertEqual(self.xs(), [1., 2., 3., 4.])
        self.assertIs(animation.rate_func, self.g["linear"])

    def test_update_from_func_tracks_a_live_drawable_in_argument_order(self):
        dot = self.g["Mobject"]()
        self.scene.add(dot)
        move = self.g["Transform"](dot, dot.copy().shift((4., 0., 0.)),
                                   run_time=1., rate_func=self.g["linear"])
        follow = self.g["UpdateFromFunc"](
            self.scene.frame,
            lambda frame: frame.shift(dot.get_center() - frame.get_center()),
            run_time=1., rate_func=self.g["linear"],
        )
        self.scene.play(move, follow)
        self.assertEqual(self.xs(), [1., 2., 3., 4.])
        self.assertEqual(self.scene.roots, [dot])
        self.assert_unmarked(follow)

    def test_nested_succession_keeps_callbacks_live_at_child_boundaries(self):
        first = self.move(2., run_time=.5)
        def update(frame, alpha):
            frame.shift(np.array([2. + 2. * alpha, 0., 0.]) - frame.get_center())
        second = self.callback(update, run_time=.5)
        nested = self.Group(self.g["Succession"](first, self.Group(second)))
        self.scene.play(nested)
        self.assertEqual(self.xs(), [1., 2., 3., 4.])
        self.assert_unmarked(first, second)

    def test_custom_lifecycle_keeps_authored_receiver_and_cleanup_order(self):
        events, Base = [], self.Animation
        class Authored(Base):
            def begin(this):
                events.append((this, "begin"))
                super().begin()
            def interpolate_mobject(this, alpha):
                events.append((this, "interpolate"))
                this.mobject.shift((alpha, 0., 0.))
            def finish(this):
                events.append((this, "finish"))
                super().finish()
            def clean_up_from_scene(this, scene):
                events.append((this, "cleanup"))
                super().clean_up_from_scene(scene)
        animation = Authored(self.scene.frame, run_time=.5, rate_func=self.g["linear"])
        self.scene.play(animation)
        self.assertTrue(all(receiver is animation for receiver, _ in events))
        self.assertEqual(events[0][1], "begin")
        self.assertEqual(events[-1][1], "cleanup")
        self.assertEqual(sum(phase == "finish" for _, phase in events), 1)
        self.assert_unmarked(animation)

    def test_builder_override_is_constructed_once(self):
        builder = self.g["Builder"](self.scene.frame, self.scene.frame.copy())
        animation = self.move(4.)
        builder.overridden_animation = animation
        self.scene.play(builder)
        self.assertEqual(builder.calls, 1)
        self.assertEqual(self.xs(), [1., 2., 3., 4.])
        self.assert_unmarked(animation)

    def test_driver_factory_also_accepts_callbacks_without_scene_play(self):
        animation = self.move(4.)
        driver = self.g["_fmn_make_camera_driver"](self.scene, animation)
        self.assert_unmarked(animation)
        driver.begin()
        driver.update_mobjects(.5)
        driver.interpolate(.5)
        np.testing.assert_allclose(self.scene.frame.get_center(), (2., 0., 0.))
        driver.finish()
        self.assertFalse(self.scene.frame._is_bound())

    def test_later_foreign_camera_is_rejected_before_any_begin(self):
        seen = []
        first = self.callback(lambda *args: seen.append(args))
        second = self.callback(lambda *args: seen.append(args), mob=self.g["CameraFrame"]())
        second._fmn_allow_camera_callback = True
        with self.assertRaisesRegex(ValueError, "this Scene.frame"):
            self.scene.play(first, self.Group(second))
        self.assertEqual(seen, [])
        self.assertEqual(self.scene.calls, [])
        self.assertEqual(self.scene.roots, [])
        self.assert_unmarked(first)
        self.assertTrue(second._fmn_allow_camera_callback)

    def test_foreign_driver_cannot_reuse_an_admission_flag(self):
        animation = self.move(4.)
        animation._fmn_allow_camera_callback = True
        with self.assertRaisesRegex(ValueError, "this Scene.frame"):
            self.g["_fmn_make_camera_driver"](self.g["Scene"](), animation)
        self.assertTrue(animation._fmn_allow_camera_callback)

    def test_removal_and_replacement_are_refused_before_callbacks(self):
        for attribute in ("remover", "replace_mobject_with_target_in_scene"):
            with self.subTest(attribute=attribute):
                animation = self.move(4.)
                setattr(animation, attribute, True)
                with self.assertRaisesRegex(NotImplementedError, "remove or replace"):
                    self.scene.play(animation)
                self.assert_unmarked(animation)
        self.assertEqual(self.scene.calls, [])

    def test_native_only_drawable_effect_cannot_be_misclassified_as_camera_callback(self):
        class Unsupported(self.g["_NativeAnimation"]):
            _native_kind = "draw_border_then_fill"
        animation = Unsupported(self.scene.frame)
        animation._fmn_allow_camera_callback = True
        with self.assertRaisesRegex(NotImplementedError, "no camera-pose"):
            self.scene.play(animation)
        self.assertEqual(self.scene.calls, [])

    def test_cycles_do_not_leave_admission_flags_on_earlier_children(self):
        animation = self.move(4.)
        group = self.Group(animation)
        group.animations.append(group)
        with self.assertRaisesRegex(ValueError, "cycle"):
            self.scene.play(group)
        self.assertEqual(self.scene.calls, [])
        self.assert_unmarked(animation)

    def test_primary_callback_exception_and_prior_flag_are_preserved(self):
        failure = LookupError("authored failure")
        def update(frame, alpha):
            if alpha > 0:
                raise failure
        animation = self.callback(update)
        prior = object()
        animation._fmn_allow_camera_callback = prior
        with self.assertRaises(LookupError) as caught:
            self.scene.play(animation)
        self.assertIs(caught.exception, failure)
        self.assertIs(animation._fmn_allow_camera_callback, prior)
        self.assertEqual(self.scene.roots, [])
        self.assertFalse(self.scene._fmn_camera_play_active)
        self.scene.play(self.move(2., run_time=.5))
        self.assertEqual(self.xs(), [1., 2.])

    def test_empty_play_and_drawable_only_play_use_the_original_path(self):
        self.assertEqual(self.scene.play(), "original")
        obj = self.g["Mobject"]()
        animation = self.callback(lambda *args: None, mob=obj)
        self.assertEqual(self.scene.play(animation), "original")
        self.assertIs(self.scene.calls[-1][1][0], animation)
        self.assert_unmarked(animation)

    def test_repeated_installation_retains_classes_and_wrappers(self):
        play = self.g["Scene"].play
        driver = self.g["_fmn_make_camera_driver"]
        install_camera_callbacks(self.native)
        self.assertIs(self.g["Scene"].play, play)
        self.assertIs(self.g["_fmn_make_camera_driver"], driver)
        self.assertIs(self.g["Animation"], self.Animation)


if __name__ == "__main__":
    unittest.main()
