"""Homotopy execution over production protocols, not native geometry."""
import unittest

import numpy as np

from movement_protocol_support import environment, movement, point


def shift(x, y, z, t):
    return x + t, y, z


class HomotopyProtocolTests(unittest.TestCase):
    def setUp(self):
        self.n = environment()
        self.mob = point(self.n)
    def animation(self, mob=None, **kwargs):
        kwargs.setdefault("rate_func", self.n._linear_rate)
        return self.n.Homotopy(shift, self.mob if mob is None else mob, **kwargs)
    def test_negative_control_old_begin_ignores_start_hook_and_suspension(self):
        n = environment(False)
        class Custom(n.Homotopy):
            def create_starting_mobject(self):
                raise AssertionError("must be called")
        mob = point(n)
        animation = Custom(shift, mob, rate_func=n._linear_rate, suspend_mobject_updating=True)
        animation.begin()
        self.assertFalse(mob.suspended)
        self.assertFalse(mob.animating)
        self.assertFalse(hasattr(animation, "families"))
    def test_negative_control_old_interpolation_ignores_family_lag(self):
        n = environment(False)
        a, b = point(n), point(n)
        animation = n.Homotopy(shift, n.Mobject(a,b), rate_func=n._linear_rate, lag_ratio=1.)
        animation.begin()
        animation.interpolate(.5)
        np.testing.assert_allclose([a.points[0,0], b.points[0,0]], [.5,.5])
    def test_family_lag_runs_through_shared_sub_alpha(self):
        a, b = point(self.n), point(self.n)
        animation = self.animation(self.n.Mobject(a,b), lag_ratio=1.)
        animation.begin()
        animation.interpolate(.5)
        np.testing.assert_allclose([a.points[0,0], b.points[0,0]], [.5,0.])
    def test_point_free_roots_and_authored_sub_alpha_hooks_are_observable(self):
        seen = []
        class Custom(self.n.Homotopy):
            def get_sub_alpha(self, alpha, index, count):
                seen.append((index,count))
                return index / count
        root = self.n.Mobject(point(self.n), point(self.n))
        animation = Custom(shift, root)
        animation.begin()
        self.assertEqual(seen, [(0,3),(1,3),(2,3)])
        np.testing.assert_allclose([root[0].points[0,0],root[1].points[0,0]], [1/3,2/3])
    def test_custom_starting_copy_is_created_once(self):
        seen = []
        class Custom(self.n.Homotopy):
            def create_starting_mobject(self):
                seen.append(self)
                return super().create_starting_mobject().shift(np.array([4.,0.,0.]))
        animation = Custom(shift, self.mob, rate_func=self.n._linear_rate)
        animation.begin()
        animation.interpolate(.5)
        self.assertEqual(seen, [animation])
        self.assertAlmostEqual(self.mob.points[0,0], 4.5)
    def test_authored_family_zip_controls_execution(self):
        class Custom(self.n.Homotopy):
            def get_all_families_zipped(self):
                return [(self.mobject[1], self.starting_mobject[1])]
        root = self.n.Mobject(point(self.n),point(self.n))
        animation = Custom(shift, root, rate_func=self.n._linear_rate)
        animation.begin()
        animation.interpolate(.7)
        self.assertEqual(root[0].points[0,0],0.)
        self.assertEqual(root[1].points[0,0],.7)
    def test_reverse_sampling_restores_start_instead_of_accumulating(self):
        self.mob.points[0,0] = 2.
        animation = self.animation()
        animation.begin()
        for alpha in (.7,.2,.9,0.):
            animation.interpolate(alpha)
            self.assertAlmostEqual(self.mob.points[0,0], 2 + alpha)
    def test_live_map_replacement_is_used_without_probe_sampling(self):
        seen=[]
        animation = self.animation()
        animation.begin()
        animation.homotopy = lambda x,y,z,t:(seen.append(t) or (x,2*t,z))
        animation.interpolate(.4)
        self.assertEqual(seen,[.4])
        np.testing.assert_allclose(self.mob.points,[[0.,.8,0.]])
    def test_function_factory_and_apply_config_remain_authored(self):
        class Custom(self.n.Homotopy):
            def function_at_time_t(self, t):
                return lambda p:p + np.array([0.,t,0.])
        config = {"make_smooth":True,"about_point":(1,2,3)}
        animation = Custom(shift,self.mob,rate_func=self.n._linear_rate,apply_function_config=config)
        animation.begin()
        animation.interpolate(.3)
        self.assertEqual(self.mob.calls[-1],("apply",config))
        self.assertEqual(config,{"make_smooth":True,"about_point":(1,2,3)})
        np.testing.assert_allclose(self.mob.points,[[0.,.3,0.]])
    def test_smoothed_subclass_keeps_native_smoothing_request(self):
        animation = self.n.SmoothedVectorizedHomotopy(shift,self.mob,rate_func=self.n._linear_rate)
        animation.begin()
        self.assertEqual(self.mob.calls[-1],("apply",{"make_smooth":True}))
    def test_complex_map_preserves_z(self):
        self.mob.points[:] = [[2.,3.,7.]]
        animation = self.n.ComplexHomotopy(lambda z,t:z*(1+t),self.mob,rate_func=self.n._linear_rate)
        animation.begin()
        animation.interpolate(.5)
        np.testing.assert_allclose(self.mob.points,[[3.,4.5,7.]])
    def test_time_span_normalizes_before_rate_and_lag(self):
        animation = self.animation(run_time=2.,time_span=(.5,1.5))
        animation.begin()
        for alpha, expected in ((.1,0.),(.25,0.),(.5,.5),(.75,1.)):
            animation.interpolate(alpha)
            self.assertAlmostEqual(self.mob.points[0,0],expected)
    def test_final_alpha_and_remover_use_shared_protocol(self):
        scene = self.n.Scene()
        animation = self.animation(final_alpha_value=.25,remover=True)
        scene.play(animation)
        self.assertEqual(self.mob.points[0,0],.25)
        self.assertNotIn(self.mob,scene.roots)
    def test_presuspended_descendant_stays_suspended_on_finish(self):
        a,b = point(self.n),point(self.n)
        b.suspend_updating()
        root = self.n.Mobject(a,b)
        animation = self.animation(root,suspend_mobject_updating=True)
        animation.begin()
        self.assertTrue(all(m.suspended for m in root.get_family()))
        animation.finish()
        self.assertFalse(root.suspended)
        self.assertFalse(a.suspended)
        self.assertTrue(b.suspended)
    def test_successful_resume_runs_one_zero_update_and_finish_is_idempotent(self):
        calls=[]
        self.mob.updaters.append(lambda mob,dt:calls.append((mob,dt)))
        animation = self.animation(suspend_mobject_updating=True)
        animation.begin()
        animation.finish()
        animation.finish()
        self.assertEqual(calls,[(self.mob,0.)])
    def test_helper_copy_updater_precedes_deformation(self):
        animation = self.animation(suspend_mobject_updating=True)
        animation.begin()
        animation.starting_mobject.updaters.append(lambda mob,dt:mob.shift(np.array([2.,0.,0.])))
        animation.update_mobjects(.1)
        animation.interpolate(.5)
        self.assertEqual(self.mob.points[0,0],2.5)
    def test_direct_map_failure_unwinds_without_running_live_updaters(self):
        calls=[]
        self.mob.updaters.append(lambda mob,dt:calls.append(dt))
        error=RuntimeError("authored map")
        animation=self.animation(suspend_mobject_updating=True)
        animation.begin()
        def fail(*args):
            raise error
        animation.homotopy=fail
        with self.assertRaises(RuntimeError) as caught:
            animation.interpolate(.5)
        self.assertIs(caught.exception,error)
        self.assertFalse(self.mob.suspended)
        self.assertFalse(self.mob.animating)
        self.assertEqual(calls,[])
    def test_direct_helper_failure_aborts(self):
        animation=self.animation(suspend_mobject_updating=True)
        animation.begin()
        def fail(mob,dt):
            raise ValueError("helper")
        animation.starting_mobject.updaters.append(fail)
        with self.assertRaisesRegex(ValueError,"helper"):
            animation.update_mobjects(.1)
        self.assertFalse(self.mob.suspended)
    def test_scene_failure_unwinds_nested_animations(self):
        scene=self.n.Scene()
        scene.fail_after="scene"
        animation=self.animation(suspend_mobject_updating=True)
        with self.assertRaisesRegex(RuntimeError,"scene updater"):
            scene.play(self.n.AnimationGroup(self.n.AnimationGroup(animation)))
        self.assertFalse(self.mob.suspended)
        self.assertFalse(self.mob.animating)
    def test_authored_override_failure_outside_super_is_unwound_by_scene(self):
        class Custom(self.n.Homotopy):
            def interpolate(self,alpha):
                if alpha > 0:
                    raise ValueError("outside super")
                return super().interpolate(alpha)
        animation=Custom(shift,self.mob,suspend_mobject_updating=True)
        with self.assertRaisesRegex(ValueError,"outside super"):
            self.n.Scene().play(animation)
        self.assertFalse(self.mob.suspended)
    def test_sibling_begin_failure_unwinds_active_movement(self):
        class Failing(self.n.Animation):
            def begin(self):
                raise ValueError("sibling")
        animation=self.animation(suspend_mobject_updating=True)
        with self.assertRaisesRegex(ValueError,"sibling"):
            self.n.Scene().play(animation,Failing(point(self.n)))
        self.assertFalse(self.mob.suspended)
    def test_abort_is_idempotent_and_preserves_preexisting_suspension(self):
        self.mob.suspend_updating()
        animation=self.animation(suspend_mobject_updating=True)
        animation.begin()
        animation.abort()
        animation.abort()
        self.assertTrue(self.mob.suspended)
        self.assertFalse(self.mob.animating)
    def test_rebegin_takes_fresh_start_and_releases_previous_suspension(self):
        animation=self.animation(suspend_mobject_updating=True)
        animation.begin()
        animation.interpolate(.4)
        animation.begin()
        animation.interpolate(.2)
        self.assertAlmostEqual(self.mob.points[0,0],.6)
        animation.finish()
        self.assertFalse(self.mob.suspended)
    def test_bad_callable_refuses_before_native_object_mutation(self):
        animation=self.n.Homotopy(None,self.mob)
        with self.assertRaisesRegex(TypeError,"callable"):
            animation.begin()
        self.assertFalse(self.mob.animating)
        self.assertEqual(self.mob.calls,[])
    def test_rate_catalog_resolves_and_unknown_name_refuses(self):
        animation=self.animation(rate_func="linear")
        animation.begin()
        animation.interpolate(.4)
        self.assertEqual(self.mob.points[0,0],.4)
        other=self.animation(rate_func="does-not-exist")
        with self.assertRaisesRegex(ValueError,"unknown rate"):
            other.begin()
    def test_class_identity_aliases_and_later_replacements_survive_reinstall(self):
        cls=self.n.Homotopy
        self.assertTrue(issubclass(self.n.ComplexHomotopy,cls))
        self.assertEqual(cls.begin.__qualname__,cls.__qualname__+".begin")
        replacement=lambda self:None
        cls.begin=replacement
        movement.install_movement(self.n)
        self.assertIs(self.n.Homotopy,cls)
        self.assertIs(cls.begin,replacement)
    def test_builder_product_identity_is_preserved_and_built_once(self):
        animation=self.animation(suspend_mobject_updating=True)
        builder=self.n._AnimationBuilder(animation)
        self.assertEqual(self.n.Scene().play(builder),"played")
        self.assertEqual(builder.count,1)
        self.assertTrue(animation._movement_finished)
    def test_cleanup_never_removes_unbegun_or_aborted_movement(self):
        animation=self.animation(remover=True)
        scene=self.n.Scene()
        scene.add(self.mob)
        animation.clean_up_from_scene(scene)
        animation.begin()
        animation.abort()
        animation.clean_up_from_scene(scene)
        self.assertIn(self.mob,scene.roots)
    def test_composition_cycle_refuses_before_begin(self):
        group=self.n.AnimationGroup(self.animation())
        group.animations.append(group)
        with self.assertRaisesRegex(ValueError,"cycle"):
            self.n.Scene().play(group)
        self.assertFalse(self.mob.animating)
    def test_later_base_begin_override_is_called_through_super(self):
        original=self.n.Animation.begin
        calls=[]
        def authored(animation):
            calls.append(animation)
            original(animation)
        self.n.Animation.begin=authored
        animation=self.animation()
        animation.begin()
        self.assertEqual(calls,[animation])
    def test_no_movement_exports_are_invented_for_reduced_namespace(self):
        from types import SimpleNamespace
        native=SimpleNamespace(Scene=object)
        movement.install_movement(native)
        self.assertEqual(vars(native),{"Scene":object})


if __name__ == "__main__":
    unittest.main()
