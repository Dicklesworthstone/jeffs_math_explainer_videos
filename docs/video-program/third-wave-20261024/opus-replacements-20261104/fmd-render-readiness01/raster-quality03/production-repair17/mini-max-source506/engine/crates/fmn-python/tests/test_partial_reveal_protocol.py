"""Live reveal hooks against production class/lifecycle/driver definitions."""
import ast
import math
import types
import unittest
import numpy as np
from creation_protocol_support import environment, line, semantics, SOURCE


class PartialRevealTests(unittest.TestCase):
    def setUp(self):
        self.g = environment()
        self.old_params = self.g["ShowPartial"]._native_params
        self.identities = {name: self.g[name] for name in ("ShowPartial", "ShowCreation", "Uncreate", "ShowPassingFlash")}
        semantics._install_partial_reveals(self.g)
        self.Partial, self.Creation = self.g["ShowPartial"], self.g["ShowCreation"]
    def curve(self):
        return line(self.g)
    def reveal(self, rule=lambda t: (t / 3, t * t), base=None, **kwargs):
        parent = base or self.Partial
        class Authored(parent):
            def get_bounds(self, alpha):
                return rule(alpha)
        return Authored(self.curve(), rate_func=self.g["linear"], **kwargs)
    def test_installer_is_called_by_both_front_doors_shared_install(self):
        install = next(n for n in ast.parse(SOURCE.read_text()).body if isinstance(n, ast.FunctionDef) and n.name == "install")
        self.assertTrue(any(isinstance(n, ast.Call) and isinstance(n.func, ast.Name)
                            and n.func.id == "_install_partial_reveals" for n in ast.walk(install)))
    def test_class_identity_and_passing_flash_hierarchy(self):
        for name, original in self.identities.items():
            self.assertIs(self.g[name], original)
        self.assertTrue(issubclass(self.g["ShowPassingFlash"], self.Partial))
        self.assertTrue(issubclass(self.g["ShowCreationThenDestruction"], self.Partial))
        with self.assertRaises(TypeError):
            self.Partial(self.curve())
    def test_four_probe_negative_control(self):
        probes = self.Partial._BOUNDS_PROBES
        def rule(t):
            return (0, t + 8 * math.prod(t - point for point in probes))
        animation = self.reveal(rule)
        self.assertEqual(self.old_params(animation), {"bounds_kind": "creation"})
        self.assertTrue(self.g["_requires_python_animation"](animation))
        animation.begin()
        animation.interpolate(.5)
        self.assertAlmostEqual(animation.mobject.get_points()[-1, 0], rule(.5)[1])
        self.assertNotEqual(rule(.5)[1], .5)
    def test_arbitrary_bounds_call_native_partial_method(self):
        animation = self.reveal()
        animation.begin()
        animation.interpolate(.75)
        self.assertEqual(animation.mobject.calls[-1][1:3], (.25, .5625))
        np.testing.assert_allclose(animation.mobject.get_points()[[0, -1], 0], [.25, .5625])
    def test_live_closure_is_not_sampled_during_lowering(self):
        state, calls = {"end": .2}, []
        def bounds(t):
            calls.append(t)
            return (0, state["end"])
        animation = self.reveal(bounds)
        self.assertTrue(self.g["_requires_python_animation"](animation))
        self.assertEqual(calls, [])
        animation.begin()
        state["end"] = .9
        animation.interpolate(.4)
        self.assertEqual(calls, [0., .4])
        self.assertEqual(animation.mobject.get_points()[-1, 0], .9)
    def test_stock_native_paths_are_retained(self):
        for cls in (self.Creation, self.g["Uncreate"], self.g["ShowPassingFlash"], self.g["ShowCreationThenDestruction"]):
            animation = cls(self.curve())
            self.assertFalse(self.g["_requires_python_animation"](animation), cls)
    def test_subclass_override_of_stock_bounds_dispatches(self):
        animation = self.reveal(base=self.Creation)
        self.assertTrue(self.g["_requires_python_animation"](animation))
        self.g["Scene"]().play(animation)
        self.assertTrue(any(row[:3] == ("partial", 1/6, .25) for row in animation.mobject.calls))
    def test_instance_monkeypatch_is_live(self):
        animation = self.Creation(self.curve(), rate_func=self.g["linear"])
        animation.get_bounds = lambda t: (.1, .9)
        self.assertTrue(self.g["_requires_python_animation"](animation))
        animation.begin()
        self.assertEqual(animation.mobject.calls[-1][1:3], (.1, .9))
    def test_class_monkeypatch_after_install_is_detected(self):
        self.Creation.get_bounds = lambda self, t: (.2, .8)
        animation = self.Creation(self.curve())
        self.assertTrue(self.g["_requires_python_animation"](animation))
    def test_mobject_native_partial_override_is_called(self):
        base = self.g["VMobject"]
        class CustomCurve(base):
            def pointwise_become_partial(self, source, lower, upper):
                self.authored_called = True
                return super().pointwise_become_partial(source, lower, upper)
        curve = CustomCurve(points=[[0,0,0],[1,0,0]])
        animation = self.Creation(curve)
        self.assertTrue(self.g["_requires_python_animation"](animation))
        self.g["Scene"]().play(animation)
        self.assertTrue(curve.authored_called)
    def test_surface_preserves_its_partial_protocol_and_metadata(self):
        surface = self.g["Surface"](points=[[0,0,0]] * 4)
        animation = self.Creation(surface)
        self.assertFalse(self.g["_requires_python_animation"](animation))
        self.assertEqual(animation._native_params()["surface_resolution"], (2, 2))
        animation.get_bounds = lambda t: (.2, .8)
        self.g["Scene"]().play(animation)
        self.assertEqual(surface.calls[-1], ("surface_partial", .2, .8, 1))
    def test_surface_passing_flash_is_not_lowered_to_vector_schema(self):
        animation = self.g["ShowPassingFlash"](self.g["Surface"](points=[[0,0,0]] * 4))
        self.assertTrue(self.g["_requires_python_animation"](animation))
        self.g["Scene"]().play(animation)
        self.assertEqual(animation.mobject.calls[-1][:3], ("surface_partial", 0., 1.))
    def test_shared_family_lag_and_time_span(self):
        root = self.g["VGroup"](self.curve(), self.curve())
        animation = self.Creation(root, run_time=2, time_span=(.5, 1.5), lag_ratio=.5, rate_func=self.g["linear"])
        animation.begin()
        animation.interpolate(.5)
        self.assertEqual([mob.calls[-1][2] for mob in root.get_family()], [1., .5, 0.])
    def test_final_alpha_is_honored_and_routes_to_callback(self):
        animation = self.Creation(self.curve(), final_alpha_value=.3, rate_func=self.g["linear"])
        self.assertTrue(self.g["_requires_python_animation"](animation))
        self.g["Scene"]().play(animation)
        self.assertEqual(animation.mobject.get_points()[-1, 0], .3)
    def test_uncreate_default_is_reverse_smooth_with_optional_removal(self):
        for remover in (False, True):
            animation = self.g["Uncreate"](self.curve(), remover=remover)
            scene = self.g["Scene"]()
            scene.add(animation.mobject)
            animation.begin()
            self.assertEqual(animation.mobject.get_points()[-1, 0], 1.)
            animation.interpolate(.25)
            self.assertAlmostEqual(animation.mobject.get_points()[-1, 0], self.g["smooth"](.75))
            animation.finish()
            self.assertEqual(animation.mobject.get_points()[-1, 0], 0.)
            animation.clean_up_from_scene(scene)
            self.assertEqual(animation.mobject in scene.roots, not remover)
    def test_passing_finish_restores_full_geometry_and_remover_flag(self):
        for remover in (False, True):
            animation = self.g["ShowPassingFlash"](self.curve(), remover=remover, rate_func=self.g["linear"])
            expected = animation.mobject.data.copy()
            animation.begin()
            animation.interpolate(.5)
            self.assertNotEqual(animation.mobject.get_points()[0, 0], 0.)
            animation.finish()
            np.testing.assert_array_equal(animation.mobject.data, expected)
            self.assertEqual(animation.remover, remover)
    def test_scene_named_rate_override_reaches_python(self):
        animation = self.reveal(lambda t:(0,t))
        self.g["Scene"]().play(animation, rate_func="linear")
        self.assertIs(animation.rate_func, self.g["linear"])
    def test_unknown_rate_refuses(self):
        animation = self.reveal()
        with self.assertRaisesRegex(ValueError, "unknown rate"):
            self.g["Scene"]().play(animation, rate_func="does-not-exist")
        self.assertFalse(animation.mobject.animating)
    def test_malformed_bounds_do_not_mutate_points(self):
        for result in ((0, float("nan")), (0, float("inf")), (0,), (0, 1, 2)):
            animation = self.reveal(lambda t:result, suspend_mobject_updating=True)
            expected = animation.mobject.data.copy()
            with self.assertRaises((ValueError, TypeError)):
                animation.begin()
            np.testing.assert_array_equal(animation.mobject.data, expected)
            self.assertFalse(animation.mobject.animating)
            self.assertFalse(animation.mobject.suspended)
    def test_authored_exception_remains_primary_during_cleanup_failure(self):
        class CallbackError(Exception):
            pass
        def fail(t):
            raise CallbackError("authored bounds")
        animation = self.reveal(fail, suspend_mobject_updating=True)
        animation.mobject.resume_updating = lambda: (_ for _ in ()).throw(ValueError("cleanup"))
        with self.assertRaisesRegex(CallbackError, "authored bounds"):
            animation.begin()
    def test_scene_failure_releases_owned_suspension(self):
        for name in ("fail_updater", "fail_prologue"):
            scene = self.g["Scene"]()
            setattr(scene, name, True)
            animation = self.reveal(suspend_mobject_updating=True)
            with self.assertRaises(ValueError):
                scene.play(animation)
            self.assertFalse(animation.mobject.animating)
            self.assertFalse(animation.mobject.suspended)
            animation.abort()
            self.assertEqual(animation.mobject.calls.count(("resume",)), 1)
    def test_preexisting_suspension_survives_abort(self):
        animation = self.reveal(suspend_mobject_updating=True)
        animation.mobject.suspend_updating()
        scene = self.g["Scene"]()
        scene.fail_updater = True
        with self.assertRaises(ValueError):
            scene.play(animation)
        self.assertTrue(animation.mobject.suspended)
    def test_lifecycle_override_outside_super_still_unwinds(self):
        animation = self.reveal(suspend_mobject_updating=True)
        base_begin = animation.begin
        def begin():
            base_begin()
            raise ValueError("after super")
        animation.begin = begin
        with self.assertRaisesRegex(ValueError, "after super"):
            self.g["Scene"]().play(animation)
        self.assertFalse(animation.mobject.animating)
        self.assertFalse(animation.mobject.suspended)
    def test_helper_failure_aborts(self):
        animation = self.reveal(suspend_mobject_updating=True)
        animation.begin()
        animation.starting_mobject.updaters.append(lambda mob,dt: (_ for _ in ()).throw(ValueError("helper")))
        animation.starting_mobject.suspended = False
        with self.assertRaisesRegex(ValueError, "helper"):
            animation.update_mobjects(.1)
        self.assertFalse(animation.mobject.animating)
    def test_rebegin_uses_current_geometry_and_unwinds_previous_lifetime(self):
        animation = self.reveal(lambda t:(0,t), suspend_mobject_updating=True)
        animation.begin()
        animation.interpolate(.5)
        animation.begin()
        self.assertEqual(animation.starting_mobject.get_points()[-1, 0], .5)
        animation.abort()
        self.assertEqual(animation.mobject.calls.count(("resume",)), 2)
    def test_nested_succession_uses_production_interval_driver(self):
        first = self.reveal(lambda t:(0,t), run_time=1.)
        second = self.reveal(lambda t:(0,t*t), run_time=1.)
        group = self.g["Succession"](first, second)
        self.g["Scene"]().play(group)
        self.assertEqual(first.mobject.get_points()[-1, 0], 1.)
        self.assertTrue(any(c[:3] == ("partial", 0, .25) for c in second.mobject.calls))
    def test_nested_scene_failure_unwinds_reveal(self):
        animation = self.reveal(suspend_mobject_updating=True)
        group = self.g["AnimationGroup"](animation)
        scene = self.g["Scene"]()
        scene.fail_updater = True
        with self.assertRaises(ValueError):
            scene.play(group)
        self.assertFalse(animation.mobject.suspended)
    def test_override_animate_builder_resolves_once(self):
        animation = self.reveal()
        builder = self.g["_AnimationBuilder"](animation)
        seen = []
        builder.build = lambda: (seen.append(1) or animation)
        self.g["Scene"]().play(builder)
        self.assertEqual(seen, [1])
    def test_invalid_window_width_is_rejected(self):
        for width in (-1, float("nan"), float("inf")):
            with self.assertRaises(ValueError):
                self.g["ShowPassingFlash"](self.curve(), time_width=width)
    def test_empty_point_family_is_safe(self):
        animation = self.Creation(self.g["VGroup"](), rate_func=self.g["linear"])
        animation.begin()
        animation.finish()
        self.assertFalse(animation.mobject.animating)
    def test_nonfinite_alpha_cannot_escape(self):
        animation = self.reveal(suspend_mobject_updating=True)
        animation.begin()
        with self.assertRaisesRegex(ValueError, "alpha must be finite"):
            animation.interpolate(float("nan"))
        self.assertFalse(animation.mobject.suspended)


if __name__ == "__main__":
    unittest.main()
