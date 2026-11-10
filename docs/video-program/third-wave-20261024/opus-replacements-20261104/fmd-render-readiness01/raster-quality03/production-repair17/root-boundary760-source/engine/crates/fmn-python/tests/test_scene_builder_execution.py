"""Full .animate options cross the real playback and execution adapters.

Only storage and the terminal Scene call are fixtures. Builder construction,
normalization, validation and execution ownership are production code.
"""
import unittest

from test_animation_builder_playback import environment, playback
from fmn_python.scene_execution import install_scene_execution


class SceneBuilderExecutionTests(unittest.TestCase):
    def setUp(self):
        self.native = environment()
        native = self.native
        # This fixture tests admission, not interpolation or a native clock.
        native.Scene._play_animations = lambda *args: self.fail("unexpected native execution")
        native.Scene.wait = lambda *args, **kwargs: None
        native.Scene.pre_play = lambda scene: scene.hooks.append("pre")
        native.Scene.post_play = lambda scene: scene.hooks.append("post")
        def update_rate_info(animation, **options):
            for key, value in options.items():
                if value is not None:
                    setattr(animation, key, value)
        native.Animation.update_rate_info = update_rate_info
        playback.install_scene_playback(native)
        install_scene_execution(native)
        self.scene, self.mob = native.Scene(), native.Mobject()
        self.scene.hooks = []

    def builder(self, **options):
        return self.native._AnimationBuilder(self.mob)(**options).shift(4)

    def test_full_constructor_options_reach_the_real_method_animation(self):
        path = lambda a, b, alpha: a
        animation, = self.scene.play(self.builder(
            path_func=path, time_span=(.25, .75), suspend_mobject_updating=True,
            final_alpha_value=.5, name="authored", remover=True,
            run_time=2., lag_ratio=.2,
        ))[0]
        self.assertIsInstance(animation, self.native._MethodAnimation)
        self.assertIs(animation.path_func, path)
        self.assertEqual(animation.time_span, (.25, .75))
        self.assertTrue(animation.suspend_mobject_updating)
        self.assertTrue(animation.remover)
        self.assertEqual(animation.final_alpha_value, .5)
        self.assertEqual(animation.name, "authored")
        self.assertEqual(animation.run_time, 2.)
        self.assertEqual(animation.lag_ratio, .2)
        self.assertEqual(animation.target_mobject.x, 4)
        self.assertEqual(self.scene.routes, [[True]])
        self.assertEqual(self.scene.hooks, ["pre", "post"])
        self.assertNotIn("_fmn_scene_execution", vars(self.scene))

    def test_unknown_builder_option_is_rejected_by_constructor_before_hooks(self):
        with self.assertRaises((TypeError, NotImplementedError)):
            self.scene.play(self.builder(unknown_option=True))
        self.assertEqual(self.scene.calls, [])
        self.assertEqual(self.scene.hooks, [])
        self.assertNotIn("_fmn_scene_execution", vars(self.scene))

    def test_builder_override_owns_its_options_and_product_is_not_rebuilt(self):
        native, seen = self.native, []
        product = native.Animation(self.mob, run_time=3.)
        class Builder(native._AnimationBuilder):
            def build(this):
                seen.append(dict(this.anim_args))
                product.payload = this.anim_args["custom_payload"]
                return product
        marker = object()
        builder = Builder(self.mob)(custom_payload=marker)
        result = self.scene.play(builder)
        self.assertEqual(seen, [{"custom_payload": marker}])
        self.assertIs(result[0][0], product)
        self.assertIs(product.payload, marker)
        self.assertEqual(self.scene.hooks, ["pre", "post"])

    def test_invalid_override_product_cannot_start_a_segment(self):
        class Builder(self.native._AnimationBuilder):
            def build(this):
                return object()
        with self.assertRaisesRegex(TypeError, "must return an Animation"):
            self.scene.play(Builder(self.mob))
        self.assertEqual(self.scene.calls, [])
        self.assertEqual(self.scene.hooks, [])
        self.assertNotIn("_fmn_scene_execution", vars(self.scene))

    def test_later_builder_failure_preserves_error_and_does_not_begin_earlier_product(self):
        error, seen, Base = LookupError("builder failed"), [], self.native._AnimationBuilder
        class First(Base):
            def build(this):
                seen.append("first")
                return super().build()
        class Second(Base):
            def build(this):
                seen.append("second")
                raise error
        with self.assertRaises(LookupError) as caught:
            self.scene.play(First(self.mob)(time_span=(0., 1.)), Second(self.mob))
        self.assertIs(caught.exception, error)
        self.assertEqual(seen, ["first", "second"])
        self.assertEqual(self.scene.calls, [])
        self.assertEqual(self.scene.hooks, [])
        self.assertNotIn("_fmn_scene_execution", vars(self.scene))
        self.scene.play(self.builder(final_alpha_value=.5))
        self.assertEqual(self.scene.hooks, ["pre", "post"])

    def test_play_level_timing_overrides_reach_the_prepared_object(self):
        rate = lambda alpha: alpha * alpha
        result = self.scene.play(self.builder(run_time=3., lag_ratio=.4, final_alpha_value=.5),
                                 run_time=2., rate_func=rate, lag_ratio=0.)
        animation = result[0][0]
        self.assertEqual(animation.run_time, 2.)
        self.assertIs(animation.rate_func, rate)
        self.assertEqual(animation.lag_ratio, 0.)
        self.assertEqual(animation.final_alpha_value, .5)

    def test_unknown_play_options_do_not_reach_builder_construction(self):
        class Builder(self.native._AnimationBuilder):
            def build(this):
                self.fail("invalid Scene.play options reached authored build")
        with self.assertRaisesRegex(TypeError, "unexpected keyword"):
            self.scene.play(Builder(self.mob), path_func=lambda *args: None)
        self.assertEqual(self.scene.calls, [])
        self.assertEqual(self.scene.hooks, [])

    def test_default_method_animation_still_selects_native_lowering(self):
        self.scene.play(self.builder())
        self.assertEqual(self.scene.routes, [[False]])


if __name__ == "__main__":
    unittest.main()
