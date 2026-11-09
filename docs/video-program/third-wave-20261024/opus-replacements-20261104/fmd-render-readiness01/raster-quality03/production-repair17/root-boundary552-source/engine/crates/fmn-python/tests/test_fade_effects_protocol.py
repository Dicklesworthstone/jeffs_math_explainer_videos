"""Authored fade routing and state changes with fixture storage and playback."""
import unittest

from test_fading_protocol import environment
from fmn_python.fading import install_fading


class FadeEffectTests(unittest.TestCase):
    def setUp(self):
        self.g = environment()
        self.original_out = self.g.FadeOut.__init__
        self.original_vout = self.g.VFadeOut
        install_fading(self.g)
        self.mob = self.g.VMobject(x=2., width=3., opacity=.8)
        self.mob.stroke_opacity = .6
    def requires(self, animation):
        return self.g._requires_python_animation(animation)
    def test_vector_negative_control_was_an_inert_interpolation(self):
        old = environment()
        mob = old.VMobject(opacity=.8)
        animation = old.VFadeIn(mob)
        animation.begin(); animation.interpolate(.25)
        self.assertEqual(mob.opacity, .8)
        new = self.g.VFadeIn(self.mob)
        new.begin(); new.interpolate(.25)
        self.assertAlmostEqual(self.mob.opacity, .2)
    def test_fadeout_negative_control_rejected_retained_endpoints(self):
        animation = self.g.FadeOut.__new__(self.g.FadeOut)
        with self.assertRaisesRegex(NotImplementedError, "unrouted"):
            self.original_out(animation, self.mob, remover=False)
    def test_vector_out_lineage_keeps_public_identity(self):
        self.assertIs(self.g.VFadeOut, self.original_vout)
        self.assertTrue(issubclass(self.g.VFadeOut, self.g.VFadeIn))
    def test_default_vector_fades_keep_native_selection(self):
        for cls in (self.g.VFadeIn, self.g.VFadeOut, self.g.VFadeInThenOut):
            self.assertFalse(self.requires(cls(self.mob)))
    def test_vector_opacity_changes_without_overwriting_geometry_or_color(self):
        animation = self.g.VFadeIn(self.mob)
        animation.begin()
        self.mob.x, self.mob.width = 12., 5.
        self.mob.uniforms["weight"] = 9.
        animation.interpolate(.25)
        self.assertAlmostEqual(self.mob.opacity, .2)
        self.assertAlmostEqual(self.mob.stroke_opacity, .15)
        self.assertEqual((self.mob.x, self.mob.width, self.mob.uniforms["weight"]), (12., 5., 9.))
    def test_starting_opacity_is_read_at_begin_not_construction(self):
        animation = self.g.VFadeIn(self.mob)
        self.mob.opacity = .4
        animation.begin(); animation.interpolate(.5)
        self.assertAlmostEqual(self.mob.opacity, .2)
    def test_vector_out_reverses_only_rated_alpha(self):
        animation = self.g.VFadeOut(self.mob, rate_func=lambda a:a*a)
        animation.begin(); animation.interpolate(.5)
        self.assertAlmostEqual(self.mob.opacity, .6)
        self.assertAlmostEqual(self.mob.stroke_opacity, .45)
        animation.finish()
        self.assertAlmostEqual(self.mob.opacity, .8)
    def test_vector_in_then_out_preserves_its_final_alpha_default(self):
        animation = self.g.VFadeInThenOut(self.mob)
        animation.begin(); animation.interpolate(.25)
        self.assertAlmostEqual(self.mob.opacity, .4)
        animation.interpolate(.75)
        self.assertAlmostEqual(self.mob.opacity, .4)
        animation.finish()
        self.assertAlmostEqual(self.mob.opacity, .8)
    def test_family_lag_does_not_recursively_overwrite_descendants(self):
        group = self.g.VMobject(self.mob, opacity=.3)
        animation = self.g.VFadeIn(group, lag_ratio=1.)
        animation.begin(); animation.interpolate(.25)
        self.assertAlmostEqual(group.opacity, .15)
        self.assertAlmostEqual(self.mob.opacity, 0.)
        animation.interpolate(.75)
        self.assertAlmostEqual(group.opacity, .3)
        self.assertAlmostEqual(self.mob.opacity, .4)
    def test_time_span_and_final_alpha_use_shared_family_protocol(self):
        animation = self.g.VFadeIn(self.mob, run_time=2., time_span=(.5, 1.5), final_alpha_value=.5)
        animation.begin(); animation.interpolate(.125)
        self.assertEqual(self.mob.opacity, 0.)
        animation.finish()
        self.assertAlmostEqual(self.mob.opacity, .4)
    def test_animation_override_selects_and_executes_python(self):
        calls = []
        class Custom(self.g.VFadeIn):
            def interpolate_submobject(self, current, start, alpha):
                calls.append(alpha)
                super().interpolate_submobject(current, start, alpha)
        animation = Custom(self.mob)
        self.assertTrue(self.requires(animation))
        scene = self.g.Scene(); scene.play(self.g.AnimationGroup(animation))
        self.assertGreater(len(calls), 4)
        self.assertAlmostEqual(self.mob.opacity, .8)
    def test_live_instance_method_replacement_is_detected(self):
        animation = self.g.VFadeIn(self.mob)
        self.assertFalse(self.requires(animation))
        original = animation.get_sub_alpha
        animation.get_sub_alpha = lambda a,i,n:.25
        self.assertTrue(self.requires(animation))
        animation.begin()
        self.assertAlmostEqual(self.mob.opacity, .2)
        animation.get_sub_alpha = original
        self.assertFalse(self.requires(animation))
    def test_live_class_method_replacement_is_detected(self):
        animation = self.g.VFadeOut(self.mob)
        original = self.g.VFadeIn.interpolate_submobject
        self.g.VFadeIn.interpolate_submobject = lambda self,a,b,t: original(self,a,b,.5*t)
        self.assertTrue(self.requires(animation))
        animation.begin(); animation.interpolate(.5)
        self.assertAlmostEqual(self.mob.opacity, .2)
    def test_shared_base_begin_replacement_is_not_hidden_by_vector_wrapper(self):
        calls = []
        original = self.g.Animation.begin
        def begin(animation):
            calls.append(animation)
            return original(animation)
        self.g.Animation.begin = begin
        animation = self.g.VFadeIn(self.mob)
        self.assertTrue(self.requires(animation))
        animation.begin()
        self.assertEqual(calls, [animation])
    def test_object_setter_override_drives_callback_route(self):
        calls = []
        class Custom(self.g.VMobject):
            def set_fill(self, **kwargs):
                calls.append(kwargs.copy())
                return super().set_fill(**kwargs)
        mob = Custom(opacity=.4)
        animation = self.g.VFadeIn(mob)
        self.assertTrue(self.requires(animation))
        animation.begin(); animation.interpolate(.5)
        self.assertEqual(calls[-1], {"opacity": .2, "recurse": False})
    def test_object_getter_override_is_executed(self):
        class Custom(self.g.VMobject):
            def get_stroke_opacity(self):
                return .2
        mob = Custom()
        animation = self.g.VFadeIn(mob)
        self.assertTrue(self.requires(animation))
        animation.begin(); animation.interpolate(.5)
        self.assertAlmostEqual(mob.stroke_opacity, .1)
    def test_vfadein_remover_selects_callback_cleanup(self):
        scene = self.g.Scene()
        animation = self.g.VFadeIn(self.mob, remover=True)
        self.assertTrue(self.requires(animation))
        scene.play(animation)
        self.assertNotIn(self.mob, scene.mobjects)
    def test_vfadeout_retention_is_not_forced_to_python(self):
        animation = self.g.VFadeOut(self.mob, remover=False, final_alpha_value=.5)
        self.assertFalse(self.requires(animation))
        animation.begin(); animation.finish()
        self.assertAlmostEqual(self.mob.opacity, .4)
    def test_direct_failure_releases_owned_suspension(self):
        failure = RuntimeError("alpha zero")
        class Broken(self.g.VFadeIn):
            def interpolate_submobject(self, *args):
                raise failure
        animation = Broken(self.mob, suspend_mobject_updating=True)
        with self.assertRaises(RuntimeError) as raised:
            animation.begin()
        self.assertIs(raised.exception, failure)
        self.assertFalse(self.mob.suspended)
        self.assertFalse(self.mob.animating)
        animation.abort()
    def test_scene_failure_cleans_up_authored_vector_suspension(self):
        class Custom(self.g.VFadeIn):
            def get_sub_alpha(self, *args):
                return super().get_sub_alpha(*args)
        animation = Custom(self.mob, suspend_mobject_updating=True)
        scene = self.g.Scene(); scene.fail_updater = True
        with self.assertRaisesRegex(RuntimeError, "scene updater"):
            scene.play(self.g.AnimationGroup(animation))
        self.assertFalse(self.mob.suspended)
        self.assertFalse(self.mob.animating)
    def test_begin_override_failure_outside_super_also_unwinds(self):
        class Broken(self.g.VFadeIn):
            def begin(self):
                self.mobject.suspend_updating()
                raise ValueError("outside super")
        animation = Broken(self.mob, suspend_mobject_updating=True)
        with self.assertRaisesRegex(ValueError, "outside super"):
            self.g.Scene().play(animation)
        self.assertFalse(self.mob.suspended)
    def test_prior_suspension_survives_finish_and_abort(self):
        self.mob.suspended = True
        group = self.g.VMobject(self.mob)
        animation = self.g.VFadeIn(group, suspend_mobject_updating=True)
        for finish in (True, False):
            animation.begin()
            animation.finish() if finish else animation.abort()
            self.assertTrue(self.mob.suspended)
            self.assertFalse(group.suspended)
    def test_finish_and_abort_are_idempotent(self):
        animation = self.g.VFadeIn(self.mob)
        animation.abort()
        with self.assertRaises(RuntimeError): animation.finish()
        animation.begin(); animation.finish(); animation.finish(); animation.abort()
        self.assertAlmostEqual(self.mob.opacity, .8)
    def test_finish_failure_preserves_original_exception(self):
        failure = ValueError("finish")
        animation = self.g.VFadeIn(self.mob, suspend_mobject_updating=True)
        animation.begin()
        animation.interpolate = lambda a: (_ for _ in ()).throw(failure)
        with self.assertRaises(ValueError) as raised: animation.finish()
        self.assertIs(raised.exception, failure)
        self.assertFalse(self.mob.suspended)
    def test_fadeout_defaults_keep_native_route_and_remover_identity(self):
        animation = self.g.FadeOut(self.mob)
        self.assertTrue(animation.remover)
        self.assertEqual(animation.final_alpha_value, 0.)
        self.assertFalse(self.requires(animation))
    def test_fadeout_retained_endpoint_uses_shared_transform(self):
        animation = self.g.FadeOut(self.mob, shift=(4.,0.,0.), scale=2., remover=False, final_alpha_value=.5)
        self.assertTrue(self.requires(animation))
        scene = self.g.Scene(); scene.play(animation)
        self.assertIn(self.mob, scene.mobjects)
        self.assertAlmostEqual(self.mob.x, 4.)
        self.assertAlmostEqual(self.mob.width, 4.5)
        self.assertAlmostEqual(self.mob.opacity, .4)
    def test_geometric_fadein_endpoint_and_remover_are_not_dropped(self):
        for kwargs in ({"remover": True}, {"final_alpha_value": .25}):
            animation = self.g.FadeIn(self.mob, **kwargs)
            self.assertTrue(self.requires(animation))
        self.assertFalse(self.requires(self.g.FadeIn(self.mob)))
    def test_geometric_callback_failure_releases_owned_suspension(self):
        animation = self.g.FadeOut(self.mob, remover=False, suspend_mobject_updating=True)
        scene = self.g.Scene(); scene.fail_updater = True
        with self.assertRaisesRegex(RuntimeError, "scene updater"):
            scene.play(animation)
        self.assertFalse(self.mob.suspended)
        self.assertIn(("unlock",), self.mob.events)
    def test_unrelated_native_specializations_keep_original_routing(self):
        animation = self.g.FadeOut(self.mob, remover=False)
        animation._native_kind = "other-specialization"
        self.assertFalse(self.requires(animation))
    def test_reinstall_does_not_reset_authored_vector_methods(self):
        method = lambda self, a,b,t: None
        self.g.VFadeOut.interpolate_submobject = method
        install_fading(self.g)
        self.assertIs(self.g.VFadeOut.interpolate_submobject, method)
    def test_catalog_rate_on_direct_vector_lifecycle(self):
        linear = lambda t:t
        self.g._RATE_FUNC_NAMES[linear] = "linear"
        animation = self.g.VFadeIn(self.mob, rate_func="linear")
        animation.begin(); animation.interpolate(.5)
        self.assertIs(animation.rate_func, linear)
        self.assertAlmostEqual(self.mob.opacity, .4)
    def test_unknown_catalog_rate_has_no_live_mutation(self):
        animation = self.g.VFadeIn(self.mob, rate_func="unknown")
        with self.assertRaisesRegex(ValueError, "unknown rate"):
            animation.begin()
        self.assertFalse(self.mob.animating)
        self.assertAlmostEqual(self.mob.opacity, .8)
    def test_geometric_callback_catalog_rate_uses_same_registered_function(self):
        linear = lambda t:t
        self.g._RATE_FUNC_NAMES[linear] = "linear"
        animation = self.g.FadeOut(self.mob, remover=False, final_alpha_value=.5, rate_func="linear")
        self.g.Scene().play(animation)
        self.assertIs(animation.rate_func, linear)
        self.assertAlmostEqual(self.mob.opacity, .4)
    def test_derived_vector_class_inherits_cooperative_initializer(self):
        class Custom(self.g.VFadeOut):
            pass
        animation = Custom(self.mob, remover=False, name="kept")
        self.assertIsInstance(animation, self.g.VFadeIn)
        self.assertFalse(animation.remover)
        self.assertEqual(animation.name, "kept")


if __name__ == "__main__":
    unittest.main()
