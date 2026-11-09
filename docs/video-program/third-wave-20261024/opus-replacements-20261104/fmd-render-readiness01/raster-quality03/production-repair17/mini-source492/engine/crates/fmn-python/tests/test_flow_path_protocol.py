"""Live movement dispatch and flow replay with fixture storage, not native kernels."""
import types
import unittest

import numpy as np

from movement_protocol_support import environment, movement, point


def field(p):
    return np.array([1., 0., 0.])


class PhaseFlowProtocolTests(unittest.TestCase):
    def setUp(self):
        self.n = environment()
        self.mob = point(self.n, 1.)
    def flow(self, **kwargs):
        kwargs.setdefault("virtual_time", 1.)
        return self.n.PhaseFlow(field, self.mob, **kwargs)
    def test_negative_control_old_rebegin_steps_backwards(self):
        n = environment(False)
        mob = point(n, 1.)
        animation = n.PhaseFlow(field, mob, virtual_time=1.)
        animation.begin()
        animation.interpolate(1.)
        self.assertEqual(mob.points[0,0], 2.)
        animation.begin()
        self.assertEqual(mob.points[0,0], 1.)
    def test_rebegin_starts_at_current_geometry_without_spurious_advection(self):
        animation = self.flow()
        animation.begin()
        animation.interpolate(1.)
        animation.finish()
        animation.begin()
        self.assertEqual(self.mob.points[0,0], 2.)
        animation.interpolate(.25)
        self.assertEqual(self.mob.points[0,0], 2.25)
    def test_explicit_zero_virtual_time_does_not_fall_back_to_runtime(self):
        animation = self.flow(virtual_time=0., run_time=2.)
        animation.begin()
        animation.interpolate(1.)
        animation.finish()
        self.assertEqual(animation.virtual_time, 0.)
        self.assertEqual(self.mob.points[0,0], 1.)
    def test_negative_control_old_constructor_loses_explicit_zero(self):
        n = environment(False)
        self.assertEqual(n.PhaseFlow(field, point(n), virtual_time=0., run_time=2.).virtual_time, 2.)
    def test_default_virtual_time_binds_construction_runtime(self):
        animation = self.n.PhaseFlow(field, self.mob, run_time=2.)
        animation.run_time = 5.
        animation.begin()
        animation.interpolate(.5)
        self.assertEqual(self.mob.points[0,0], 2.)
    def test_raw_alpha_euler_remains_path_dependent(self):
        animation = self.n.PhaseFlow(lambda p:p, self.mob, virtual_time=1.)
        animation.begin()
        animation.interpolate(.25)
        animation.interpolate(.5)
        self.assertEqual(self.mob.points[0,0], 1.25**2)
        animation.interpolate(.25)
        self.assertEqual(self.mob.points[0,0], 1.25**2 * .75)
    def test_rates_lag_and_span_do_not_redefine_flow_integration(self):
        seen = []
        def rate(t):
            seen.append(t)
            return t*t
        animation = self.flow(rate_func=rate, lag_ratio=3., time_span=(1.,2.), run_time=2.)
        animation.begin()
        animation.interpolate(.5)
        self.assertEqual(self.mob.points[0,0], 1.5)
        self.assertEqual(seen, [])
    def test_live_field_replacement_is_used_at_next_step(self):
        animation = self.flow()
        animation.begin()
        animation.interpolate(.25)
        animation.function = lambda p:np.array([0.,2.,0.])
        animation.interpolate(.5)
        np.testing.assert_allclose(self.mob.points, [[1.25,.5,0.]])
    def test_negative_virtual_time_advects_backwards(self):
        animation = self.flow(virtual_time=-2.)
        animation.begin()
        animation.interpolate(.25)
        self.assertEqual(self.mob.points[0,0], .5)
    def test_failure_then_reuse_clears_alpha_and_owned_suspension(self):
        animation = self.flow(suspend_mobject_updating=True)
        animation.begin()
        animation.interpolate(.25)
        error = ValueError("field failed")
        def fail(p):
            raise error
        animation.function = fail
        with self.assertRaises(ValueError) as caught:
            animation.interpolate(.5)
        self.assertIs(caught.exception, error)
        self.assertFalse(self.mob.suspended)
        animation.function = field
        animation.begin()
        self.assertEqual(self.mob.points[0,0], 1.25)
        animation.interpolate(.25)
        self.assertEqual(self.mob.points[0,0], 1.5)
    def test_nested_scene_failure_aborts_flow(self):
        scene = self.n.Scene()
        scene.fail_after = "scene"
        animation = self.flow(suspend_mobject_updating=True)
        with self.assertRaisesRegex(RuntimeError, "scene updater"):
            scene.play(self.n.AnimationGroup(animation))
        self.assertFalse(self.mob.suspended)
        self.assertFalse(self.mob.animating)
    def test_current_field_and_time_are_validated_before_begin(self):
        for value in (None, 4):
            with self.assertRaises(TypeError):
                self.n.PhaseFlow(value, self.mob)
        for value in (float("inf"), float("nan")):
            with self.assertRaises(ValueError):
                self.flow(virtual_time=value)
        animation = self.flow()
        animation.function = None
        with self.assertRaises(TypeError):
            animation.begin()
        self.assertFalse(self.mob.animating)
    def test_partial_final_alpha_and_retained_cleanup(self):
        animation = self.flow(final_alpha_value=.25)
        animation.begin()
        animation.interpolate(.5)
        animation.finish()
        self.assertEqual(self.mob.points[0,0], 1.25)
    def test_finished_flow_failed_rebegin_cannot_claim_previous_completion(self):
        animation = self.flow()
        animation.begin()
        animation.finish()
        animation.function = None
        with self.assertRaises(TypeError):
            animation.begin()
        with self.assertRaisesRegex(RuntimeError, "must begin"):
            animation.finish()


class PathMotionProtocolTests(unittest.TestCase):
    def setUp(self):
        self.n = environment()
        self.mob = point(self.n)
        self.path = self.n.VMobject(points=[(0.,0.,0.),(2.,0.,0.)])
    def animation(self, path=None, mob=None, **kwargs):
        kwargs.setdefault("rate_func", self.n._linear_rate)
        return self.n.MoveAlongPath(self.mob if mob is None else mob,
                                   self.path if path is None else path, **kwargs)
    def requires(self, animation):
        return self.n._requires_python_animation(animation)
    def test_stock_path_motion_keeps_native_execution(self):
        animation = self.animation()
        self.assertFalse(self.requires(animation))
        self.n.Scene().play(animation)
        self.assertFalse(hasattr(animation, "_movement_active"))
    def test_negative_control_old_classifier_silently_ignores_path_override(self):
        n = environment(False)
        class Authored(n.VMobject):
            def point_from_proportion(self, alpha):
                raise AssertionError("must be dispatched")
        animation = n.MoveAlongPath(point(n), Authored(points=[(0.,0.,0.)]))
        self.assertFalse(n._requires_python_animation(animation))
    def test_authored_path_sampler_runs_at_each_actual_alpha_without_probes(self):
        seen = []
        class Authored(self.n.VMobject):
            def point_from_proportion(self, alpha):
                seen.append(alpha)
                return np.array([alpha*alpha, 3*alpha, 0.])
        animation = self.animation(path=Authored(points=[(0.,0.,0.)]))
        self.assertTrue(self.requires(animation))
        self.assertEqual(seen, [])
        animation.begin()
        animation.interpolate(.3)
        self.assertEqual(seen, [0.,.3])
        np.testing.assert_allclose(self.mob.points, [[.09,.9,0.]])
    def test_live_instance_sampler_replacement_is_observed(self):
        animation = self.animation()
        self.path.point_from_proportion = lambda t:np.array([9*t,0.,0.])
        self.assertTrue(self.requires(animation))
        animation.begin()
        animation.interpolate(.5)
        self.assertEqual(self.mob.points[0,0], 4.5)
    def test_later_base_sampler_override_is_observed(self):
        animation = self.animation()
        original = self.n.VMobject.point_from_proportion
        self.n.VMobject.point_from_proportion = lambda path,t:original(path,t)+[0.,t,0.]
        self.assertTrue(self.requires(animation))
        animation.begin()
        animation.interpolate(.5)
        np.testing.assert_allclose(self.mob.points, [[1.,.5,0.]])
    def test_bound_method_replacement_is_unwrapped_for_dispatch(self):
        animation = self.animation()
        def sampler(path,t):
            return np.array([5*t,0.,0.])
        self.path.point_from_proportion = types.MethodType(sampler,self.path)
        self.assertTrue(self.requires(animation))
        animation.begin()
        animation.interpolate(.2)
        self.assertEqual(self.mob.points[0,0],1.)
    def test_custom_move_to_and_shift_hooks_run(self):
        seen=[]
        class Moving(self.n.VMobject):
            def move_to(self,point):
                seen.append("move")
                return super().move_to(point)
            def shift(self,amount):
                seen.append("shift")
                return super().shift(amount)
        mob = Moving(points=[(0.,0.,0.)])
        animation=self.animation(mob=mob)
        self.assertTrue(self.requires(animation))
        animation.begin()
        animation.interpolate(.5)
        self.assertEqual(seen,["move","shift","move","shift"])
        self.assertEqual(mob.points[0,0],1.)
    def test_animation_subclass_lifecycle_is_not_silently_lowered(self):
        seen=[]
        class Custom(self.n.MoveAlongPath):
            def begin(self):
                seen.append(self)
                return super().begin()
        animation=Custom(self.mob,self.path,rate_func=self.n._linear_rate)
        self.assertTrue(self.requires(animation))
        self.n.Scene().play(animation)
        self.assertEqual(seen,[animation])
    def test_later_animation_base_hook_runs_via_super(self):
        original=self.n.Animation.begin
        seen=[]
        def begin(animation):
            seen.append(animation)
            original(animation)
        self.n.Animation.begin=begin
        animation=self.animation()
        self.assertTrue(self.requires(animation))
        animation.begin()
        self.assertEqual(seen,[animation])
    def test_custom_rate_is_not_presampled_or_interpolated(self):
        seen=[]
        def rate(t):
            seen.append(t)
            return 1. if .3<t<.31 else 0.
        animation=self.animation(rate_func=rate)
        self.assertTrue(self.requires(animation))
        self.assertEqual(seen,[])
        animation.begin()
        animation.interpolate(.305)
        self.assertEqual(seen,[0.,.305])
        self.assertEqual(self.mob.points[0,0],2.)
    def test_custom_play_rate_reaches_animation_before_dispatch(self):
        seen=[]
        animation=self.animation()
        self.n.Scene().play(animation,rate_func=lambda t:(seen.append(t) or t*t))
        self.assertEqual(seen,[0.,.25,.5,.75,1.,1.])
        self.assertFalse(hasattr(animation,"_movement_force_callback"))
        self.assertEqual(self.mob.points[0,0],2.)
    def test_repeated_animation_identity_does_not_leave_temporary_route_flags(self):
        animation=self.animation()
        self.n.Scene().play(animation,animation,rate_func=lambda t:t)
        self.assertFalse(hasattr(animation,"_movement_force_callback"))
    def test_catalog_names_and_published_rates_keep_native_route(self):
        for rate in (None,"linear",self.n._linear_rate,self.n.smooth):
            self.assertFalse(self.requires(self.animation(rate_func=rate)))
    def test_raw_alpha_bypasses_lag_and_time_span_like_native_kernel(self):
        animation=self.animation(time_span=(1.,2.),run_time=2.,lag_ratio=3.,final_alpha_value=.5)
        animation.begin()
        animation.interpolate(.25)
        self.assertEqual(self.mob.points[0,0],.5)
        animation.finish()
        self.assertEqual(self.mob.points[0,0],1.)
    def test_retained_fractional_endpoint_and_remover_are_routed(self):
        animation=self.animation(final_alpha_value=.5,remover=True)
        self.assertTrue(self.requires(animation))
        scene=self.n.Scene()
        scene.play(animation)
        self.assertEqual(self.mob.points[0,0],1.)
        self.assertNotIn(self.mob,scene.roots)
    def test_path_is_read_live_between_interpolations(self):
        animation=self.animation(final_alpha_value=.5)
        animation.begin()
        self.path.shift(np.array([3.,1.,0.]))
        animation.interpolate(.5)
        np.testing.assert_allclose(self.mob.points,[[4.,1.,0.]])
    def test_path_only_updaters_are_not_silently_added_to_helper_set(self):
        seen=[]
        self.path.updaters.append(lambda mob,dt:seen.append(dt))
        animation=self.animation(final_alpha_value=.5)
        animation.begin()
        animation.update_mobjects(.1)
        self.assertEqual(seen,[])
    def test_mutated_empty_or_foreign_path_refuses_before_callback_motion(self):
        animation=self.animation(final_alpha_value=.5)
        self.path.points=np.empty((0,3))
        with self.assertRaisesRegex(ValueError,"nonempty"):
            animation.begin()
        self.path.points=np.zeros((1,3))
        self.path._scene=object()
        self.mob._scene=object()
        with self.assertRaisesRegex(ValueError,"another Scene"):
            animation.begin()
        self.assertFalse(self.mob.animating)
    def test_authored_path_exception_releases_only_acquired_suspension(self):
        a,b=point(self.n),point(self.n)
        b.suspend_updating()
        root=self.n.Mobject(a,b)
        animation=self.animation(mob=root,final_alpha_value=.5,suspend_mobject_updating=True)
        animation.begin()
        def fail(t):
            raise ValueError("path callback")
        self.path.point_from_proportion=fail
        with self.assertRaisesRegex(ValueError,"path callback"):
            animation.interpolate(.5)
        self.assertFalse(root.suspended)
        self.assertFalse(a.suspended)
        self.assertTrue(b.suspended)
    def test_nested_scene_failure_cleans_up_callback_path_motion(self):
        scene=self.n.Scene()
        scene.fail_after="scene"
        animation=self.animation(final_alpha_value=.5,suspend_mobject_updating=True)
        with self.assertRaisesRegex(RuntimeError,"scene updater"):
            scene.play(self.n.AnimationGroup(animation))
        self.assertFalse(self.mob.suspended)
        self.assertFalse(self.mob.animating)
    def test_authored_failure_outside_super_is_unwound(self):
        class Custom(self.n.MoveAlongPath):
            def interpolate(self,alpha):
                if alpha>0:
                    raise ValueError("outside super")
                return super().interpolate(alpha)
        animation=Custom(self.mob,self.path,suspend_mobject_updating=True)
        with self.assertRaisesRegex(ValueError,"outside super"):
            self.n.Scene().play(animation)
        self.assertFalse(self.mob.suspended)
    def test_play_rate_failure_leaves_no_temporary_dispatch_flag(self):
        animation=self.animation(suspend_mobject_updating=True)
        def fail(t):
            if t>0:
                raise ValueError("rate")
            return t
        with self.assertRaisesRegex(ValueError,"rate"):
            self.n.Scene().play(animation,rate_func=fail)
        self.assertFalse(hasattr(animation,"_movement_force_callback"))
        self.assertFalse(self.mob.suspended)
    def test_static_reflection_does_not_evaluate_user_sampler_property(self):
        seen=[]
        class Custom(self.n.VMobject):
            @property
            def point_from_proportion(self):
                seen.append("property")
                return lambda t:np.array([t,0.,0.])
        animation=self.animation(path=Custom(points=[(0.,0.,0.)]))
        self.assertTrue(self.requires(animation))
        self.assertEqual(seen,[])
        animation.begin()
        self.assertEqual(seen,["property"])
    def test_unhashable_callable_rates_can_use_callback_route(self):
        class Rate:
            __hash__=None
            def __call__(self,t):
                return t
        animation=self.animation(rate_func=Rate())
        self.assertTrue(self.requires(animation))
        animation.begin()
        animation.interpolate(.25)
        self.assertEqual(self.mob.points[0,0],.5)
    def test_direct_begin_then_play_reenters_the_same_owned_lifecycle(self):
        animation=self.animation()
        self.assertFalse(self.requires(animation))
        animation.begin()
        self.assertTrue(self.requires(animation))
        self.n.Scene().play(animation)
        self.assertTrue(animation._movement_finished)
    def test_reinstall_preserves_authored_path_methods(self):
        previous=self.n.MoveAlongPath.interpolate_mobject
        self.n.MoveAlongPath.interpolate_mobject=lambda animation,t:previous(animation,t)
        replacement=self.n.MoveAlongPath.interpolate_mobject
        movement.install_movement(self.n)
        self.assertIs(self.n.MoveAlongPath.interpolate_mobject,replacement)
        self.assertTrue(self.requires(self.animation()))


if __name__ == "__main__":
    unittest.main()
