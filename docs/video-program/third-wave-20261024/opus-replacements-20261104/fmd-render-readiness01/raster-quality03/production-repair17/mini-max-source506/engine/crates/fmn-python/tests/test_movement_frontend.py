"""The actual Scene.play lowering; only the native execution boundary is a double.

This expands the lightweight protocol driver coverage. It does not load Rust
or establish native scheduling, geometry, or pixel-rendering correctness.
"""
import ast
import unittest

import numpy as np

from movement_protocol_support import ROOT, environment, movement, point


def frontend():
    native = environment(False)
    g = vars(native)
    path = ROOT / "python/manimlib_bootstrap.py"
    scene = next(node for node in ast.parse(path.read_text()).body
                 if isinstance(node, ast.ClassDef) and node.name == "Scene")
    play = next(node for node in scene.body if isinstance(node, ast.FunctionDef) and node.name == "play")
    exec(compile(ast.Module([play], []), str(path), "exec"), g)
    native.Scene.play = g["play"]
    old_play = native.Scene.play
    native.Mobject._is_bound = lambda mob:getattr(mob,"_scene",None) is not None
    native.Scene._adopt = lambda scene,mob:setattr(mob,"_scene",scene)
    old_add = native.Scene.add
    def add(scene,*mobs):
        for mob in mobs:
            scene._adopt(mob)
        old_add(scene,*mobs)
    native.Scene.add = add
    native._NativeAnimation._native_params = lambda animation:{}
    native._NativeAnimation._native_target = lambda animation:getattr(animation,"path",None)
    g["_BridgeMobject"] = native.Mobject
    g["CameraFrame"] = type("CameraFrame",(native.Mobject,),{})
    g["_CompositionCallbackDriver"] = type("UnusedCompositionBoundary",(),{})
    def execute(scene,specs,callbacks,camera,run_time,rate_payload,lag):
        scene.specs,scene.callbacks,scene.rate_payload = specs,callbacks,rate_payload
        for callback in callbacks:
            if callback is not None:
                callback.begin()
        for alpha in (.25,.5,.75,1.):
            for callback in callbacks:
                if callback is not None:
                    callback.update_mobjects(.25)
                    callback.interpolate(alpha)
        for callback in callbacks:
            if callback is not None:
                callback.finish()
                callback.clean_up_from_scene(scene)
    native.Scene._play_animations = execute
    movement.install_movement(native)
    return native,old_play


class MovementFrontendTests(unittest.TestCase):
    def setUp(self):
        self.n,self.old_play = frontend()
        self.scene,self.mob = self.n.Scene(),point(self.n)
        self.path = self.n.VMobject(points=[(0.,0.,0.),(2.,0.,0.)])
    def test_negative_control_underlying_frontend_probes_global_callback_rate(self):
        seen=[]
        def rate(t):
            seen.append(t)
            return t
        animation=self.n.MoveAlongPath(self.mob,self.path,rate_func=rate)
        self.old_play(self.scene,animation,rate_func=rate)
        self.assertEqual(len(seen),37)
        self.assertEqual(seen[:31],[i/30 for i in range(31)])
    def test_play_level_rate_reaches_real_callback_without_redundant_global_sampling(self):
        seen=[]
        def rate(t):
            seen.append(t)
            return t*t
        animation=self.n.MoveAlongPath(self.mob,self.path)
        self.scene.play(animation,rate_func=rate)
        self.assertEqual(seen,[0.,.25,.5,.75,1.,1.])
        self.assertIsNone(self.scene.rate_payload)
        self.assertEqual(self.scene.specs[0][0],"python_callback")
        self.assertEqual(self.mob.points[0,0],2.)
    def test_animation_level_unhashable_rate_never_enters_native_payload_sampler(self):
        class Rate:
            __hash__=None
            def __call__(self,t):
                return t
        animation=self.n.MoveAlongPath(self.mob,self.path,rate_func=Rate())
        self.scene.play(animation)
        self.assertEqual(self.scene.specs[0][0],"python_callback")
        self.assertEqual(self.mob.points[0,0],2.)
    def test_stock_movealongpath_still_builds_original_native_spec(self):
        animation=self.n.MoveAlongPath(self.mob,self.path,rate_func=self.n._linear_rate)
        self.scene.play(animation)
        self.assertEqual(self.scene.specs[0][0],"move_along_path")
        self.assertEqual(self.scene.specs[0][4],"linear")
        self.assertIs(self.scene.specs[0][2],self.path)
        self.assertIsNone(self.scene.callbacks[0])
    def test_authored_path_and_scene_remover_reach_real_python_spec(self):
        class Path(self.n.VMobject):
            def point_from_proportion(self,t):
                return np.array([t,2*t,0.])
        animation=self.n.MoveAlongPath(self.mob,Path(points=[(0.,0.,0.)]),
                                        rate_func=self.n._linear_rate,remover=True,final_alpha_value=.5)
        self.scene.play(animation)
        # Python callback lifecycle owns cleanup (cfff2662); native placeholder remover is disabled
        self.assertFalse(self.scene.specs[0][6]["remover"])
        np.testing.assert_allclose(self.mob.points,[[.5,1.,0.]])
        self.assertNotIn(self.mob,self.scene.roots)
    def test_play_override_is_not_lost_for_native_sibling(self):
        class Native(self.n._NativeAnimation):
            _native_kind="fixture_native"
        sibling=Native(point(self.n),run_time=2.)
        rate=lambda t:t*t
        animation=self.n.MoveAlongPath(self.mob,self.path)
        self.scene.play(animation,sibling,rate_func=rate)
        self.assertEqual(self.scene.specs[0][0],"python_callback")
        self.assertEqual(self.scene.specs[1][0],"fixture_native")
        self.assertIsNone(sibling.rate_func)
        self.assertIsNone(self.scene.specs[1][4])
        self.assertEqual(self.scene.rate_payload,[(i/30)**2 for i in range(31)])
        self.assertIs(animation.rate_func,rate)
        self.assertFalse(hasattr(animation,"_movement_force_callback"))
    def test_phaseflow_replay_through_real_frontend_preserves_new_start(self):
        flow=self.n.PhaseFlow(lambda p:np.array([1.,0.,0.]),self.mob,virtual_time=1.)
        self.scene.play(flow)
        self.assertEqual(self.mob.points[0,0],1.)
        self.scene.play(flow)
        self.assertEqual(self.mob.points[0,0],2.)
    def test_rate_failure_on_real_callback_unwinds_suspension(self):
        error=ValueError("rate failed")
        def rate(t):
            if t>0:
                raise error
            return t
        animation=self.n.MoveAlongPath(self.mob,self.path,suspend_mobject_updating=True)
        with self.assertRaises(ValueError) as caught:
            self.scene.play(animation,rate_func=rate)
        self.assertIs(caught.exception,error)
        self.assertFalse(self.mob.suspended)
        self.assertFalse(self.mob.animating)


if __name__ == "__main__":
    unittest.main()
