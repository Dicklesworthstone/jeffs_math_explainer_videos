"""Restore wiring over production constructors and fixture object storage."""
import types
import unittest
from test_animation_builder_playback import environment, playback


class RestorePlaybackTests(unittest.TestCase):
    def setUp(self):
        self.native = environment()
        self.original = self.native.Restore
        self.old_init = self.original.__init__
        self.alias = types.SimpleNamespace(Restore=self.original)
        self.transform_init = self.native.Transform.__init__
        playback.install_scene_playback(self.native)
        self.mob = self.native.Mobject(4.)
        self.saved = self.native.Mobject(1.)
        self.mob.saved_state = self.saved
    def restore(self, **kwargs):
        return self.native.Restore(self.mob, **kwargs)
    def test_public_class_and_qualified_alias_keep_identity(self):
        self.assertIs(self.native.Restore, self.original)
        self.assertIs(self.alias.Restore, self.original)
        self.assertTrue(issubclass(self.original, self.native.Transform))
    def test_constructor_captures_saved_target_by_reference(self):
        animation = self.restore()
        self.assertIs(animation.mobject, self.mob)
        self.assertIs(animation.target_mobject, self.saved)
        self.assertEqual(animation._target_attr, "target_mobject")
    def test_later_saved_pointer_rebinding_does_not_retarget_animation(self):
        animation = self.restore()
        self.mob.saved_state = self.native.Mobject(10.)
        self.assertIs(animation.target_mobject, self.saved)
        self.assertIs(self.restore().target_mobject, self.mob.saved_state)
    def test_clearing_saved_pointer_after_construction_keeps_target(self):
        animation = self.restore()
        self.mob.saved_state = None
        self.assertIs(animation.target_mobject, self.saved)
    def test_saved_object_edits_remain_live(self):
        animation = self.restore()
        self.saved.shift(2.)
        self.assertEqual(animation.target_mobject.x, 3.)
    def test_missing_and_none_saved_state_preserve_named_exception(self):
        for missing in (False, True):
            mob = self.native.Mobject()
            if missing:
                del mob.saved_state
            with self.assertRaisesRegex(Exception, "Trying to restore without having saved"):
                self.native.Restore(mob)
    def test_malformed_saved_state_is_rejected(self):
        self.mob.saved_state = object()
        with self.assertRaisesRegex(TypeError, "saved_state must be a Mobject"):
            self.restore()
    def test_arbitrary_path_is_forwarded_unchanged(self):
        path = lambda start, end, alpha: (start, end, alpha)
        animation = self.restore(path_func=path)
        self.assertIs(animation.path_func, path)
        self.assertTrue(self.native._requires_python_animation(animation))
    def test_arc_configuration_keeps_existing_native_parameters(self):
        animation = self.restore(path_arc=.7, path_arc_axis=(1., 0., 0.))
        self.assertEqual(animation._native_params(), {"path_arc": .7, "path_arc_axis": (1., 0., 0.)})
        self.assertFalse(self.native._requires_python_animation(animation))
    def test_default_uses_transform_kernel_not_legacy_saved_state_relinking(self):
        animation = self.restore()
        self.assertEqual(animation._native_kind, "transform")
        self.assertFalse(self.native._requires_python_animation(animation))
    def test_full_animation_configuration_reaches_transform(self):
        rate = lambda alpha: alpha
        animation = self.restore(run_time=3, rate_func=rate, lag_ratio=.1, time_span=(1, 2),
                                 suspend_mobject_updating=True, final_alpha_value=.3, remover=True, name="restore")
        self.assertEqual((animation.run_time, animation.lag_ratio, animation.time_span), (3, .1, (1, 2)))
        self.assertIs(animation.rate_func, rate)
        self.assertTrue(animation.suspend_mobject_updating)
        self.assertEqual(animation.final_alpha_value, .3)
        self.assertTrue(animation.remover)
        self.assertEqual(animation.name, "restore")
    def test_final_alpha_and_remover_select_shared_lifecycle(self):
        for options in ({"final_alpha_value": .4}, {"remover": True}):
            animation = self.restore(**options)
            scene = self.native.Scene()
            scene.play(animation)
            self.assertEqual(scene.routes, [[True]])
            self.assertIs(scene.calls[0][0][0], animation)
    def test_no_copy_or_target_generation_during_construction(self):
        self.mob.copy = self.mob.generate_target = lambda: self.fail("unexpected copy")
        self.saved.copy = lambda: self.fail("unexpected saved copy")
        self.assertIs(self.restore().target_mobject, self.saved)
    def test_transform_constructor_dispatch_is_cooperative(self):
        calls = []
        original = self.transform_init
        def observed(instance, *args, **kwargs):
            calls.append((instance, args, kwargs))
            return original(instance, *args, **kwargs)
        self.native.Transform.__init__ = observed
        animation = self.restore(path_arc=.5)
        self.assertEqual(len(calls), 1)
        self.assertIs(calls[0][0], animation)
        self.assertEqual(calls[0][1], (self.mob, self.saved))
    def test_mixin_after_restore_participates_in_constructor_mro(self):
        calls = []
        class Mixin(self.native.Transform):
            def __init__(self, *args, **kwargs):
                calls.append(self)
                super().__init__(*args, **kwargs)
        class Mixed(self.original, Mixin):
            pass
        animation = Mixed(self.mob)
        self.assertEqual(calls, [animation])
        self.assertIs(animation.target_mobject, self.saved)
    def test_subclass_existing_before_installation_keeps_custom_parameters(self):
        native = environment()
        class BroadcastRestore(native.Restore):
            def _native_params(self):
                return dict(super()._native_params(), remover=self.remover)
        playback.install_scene_playback(native)
        mob = native.Mobject()
        mob.saved_state = native.Mobject(9.)
        animation = BroadcastRestore(mob, remover=True)
        self.assertIsInstance(animation, native.Transform)
        self.assertTrue(native._requires_python_animation(animation))
        self.assertEqual(animation._native_params()["remover"], True)
    def test_transform_lifecycle_is_inherited_without_a_second_implementation(self):
        sentinel = lambda animation: "transform lifecycle"
        self.native.Transform.begin = sentinel
        self.assertIs(self.original.begin, sentinel)
        self.assertEqual(self.restore().begin(), "transform lifecycle")
    def test_authored_restore_hooks_stay_on_the_public_object(self):
        calls = []
        class Custom(self.original):
            def create_target(self):
                calls.append(self)
                return self.target_mobject
        animation = Custom(self.mob)
        self.assertIs(animation.create_target(), self.saved)
        self.assertEqual(calls, [animation])
    def test_inherited_camera_transform_identity(self):
        class CameraFrame(self.native.Mobject):
            pass
        frame = CameraFrame()
        frame.saved_state = CameraFrame()
        animation = self.original(frame)
        self.assertIsInstance(animation, self.native.Transform)
        self.assertIs(animation.mobject, frame)
        self.assertIs(animation.target_mobject, frame.saved_state)
    def test_reinstall_does_not_overwrite_authored_restore_method(self):
        authored = lambda *args, **kwargs: None
        self.original.__init__ = authored
        playback.install_scene_playback(self.native)
        self.assertIs(self.original.__init__, authored)
    def test_constructor_identity_is_public(self):
        method = self.original.__init__
        self.assertEqual(method.__name__, "__init__")
        self.assertEqual(method.__qualname__, self.original.__qualname__ + ".__init__")
        self.assertEqual(method.__module__, self.original.__module__)
    def test_old_constructor_rejects_custom_path_negative_control(self):
        native = environment()
        mob = native.Mobject()
        mob.saved_state = native.Mobject()
        with self.assertRaisesRegex(NotImplementedError, "path_func"):
            native.Restore(mob, path_func=lambda a, b, t: a)
        self.assertFalse(issubclass(native.Restore, native.Transform))


if __name__ == "__main__":
    unittest.main()
