"""Two-stage drawing against production interpolation and composition code."""
import ast
import unittest
import numpy as np
from creation_protocol_support import environment, line, semantics, SOURCE


class BorderWriteTests(unittest.TestCase):
    def setUp(self):
        self.g = environment()
        semantics._install_partial_reveals(self.g)
        self.Border, self.Write = self.g["DrawBorderThenFill"], self.g["Write"]
        self.original = (self.Border, self.Write)
        semantics._install_border_write(self.g)
    def curve(self):
        return line(self.g)
    def drawing(self, **kwargs):
        kwargs.setdefault("rate_func", self.g["linear"])
        return self.Border(self.curve(), **kwargs)
    def test_shared_install_calls_border_protocol(self):
        install = next(n for n in ast.parse(SOURCE.read_text()).body if isinstance(n, ast.FunctionDef) and n.name == "install")
        self.assertTrue(any(isinstance(n, ast.Call) and isinstance(n.func, ast.Name)
                            and n.func.id == "_install_border_write" for n in ast.walk(install)))
    def test_public_class_identities_are_unchanged(self):
        self.assertEqual((self.g["DrawBorderThenFill"], self.g["Write"]), self.original)
    def test_first_phase_draws_only_the_outline(self):
        animation = self.drawing()
        animation.begin()
        animation.interpolate(.25)
        self.assertEqual(animation.mobject.get_points()[-1, 0], .5)
        self.assertEqual(animation.mobject.fill_opacity, 0.)
        self.assertEqual(animation.mobject.stroke_width, 2.)
        self.assertEqual(animation.sm_to_index[hash(animation.mobject)], 0)
    def test_fill_phase_restores_full_path_and_interpolates_style(self):
        animation = self.drawing()
        animation.begin()
        animation.interpolate(.25)
        animation.interpolate(.75)
        self.assertEqual(animation.mobject.get_points()[-1, 0], 1.)
        self.assertAlmostEqual(animation.mobject.fill_opacity, .35)
        self.assertAlmostEqual(animation.mobject.stroke_width, 3.5)
        self.assertEqual(animation.sm_to_index[hash(animation.mobject)], 1)
    def test_finish_restores_original_records_and_refreshes_joints(self):
        animation = self.drawing()
        before = animation.mobject.data.copy()
        animation.begin()
        animation.finish()
        np.testing.assert_array_equal(animation.mobject.data, before)
        self.assertIn(("joints",), animation.mobject.calls)
        self.assertFalse(animation.mobject.animating)
    def test_custom_outline_hook_is_evaluated_at_begin_not_constructor(self):
        seen = []
        base = self.Border
        class Authored(base):
            def get_outline(self):
                seen.append(self.mobject.stroke_width)
                return super().get_outline().set_stroke(width=9.)
        animation = Authored(self.curve(), rate_func=self.g["linear"])
        self.assertEqual(seen, [])
        animation.mobject.stroke_width = 7.
        self.assertTrue(self.g["_requires_python_animation"](animation))
        animation.begin()
        self.assertEqual(seen, [7.])
        animation.interpolate(.25)
        self.assertEqual(animation.mobject.stroke_width, 9.)
        animation.finish()
        self.assertEqual(animation.mobject.stroke_width, 7.)
    def test_default_native_routes_stay_native(self):
        self.assertFalse(self.g["_requires_python_animation"](self.Border(self.curve())))
        self.assertFalse(self.g["_requires_python_animation"](self.Write(self.curve())))
    def test_late_class_and_instance_hooks_force_python(self):
        animation = self.drawing()
        original = animation.get_outline
        animation.get_outline = lambda: original()
        self.assertTrue(self.g["_requires_python_animation"](animation))
        self.Border.get_outline = lambda self: self.mobject.copy().set_fill(opacity=0)
        self.assertTrue(self.g["_requires_python_animation"](self.drawing()))
    def test_mobject_interpolate_override_is_called(self):
        base = self.g["VMobject"]
        class AuthoredObject(base):
            def interpolate(self, start, end, alpha, path_func=None):
                self.calls.append(("authored-interpolate", alpha))
                return super().interpolate(start, end, alpha, path_func)
        curve = AuthoredObject(points=[[0,0,0],[1,0,0]])
        animation = self.Border(curve, rate_func=self.g["linear"])
        self.assertTrue(self.g["_requires_python_animation"](animation))
        self.g["Scene"]().play(animation)
        self.assertIn(("authored-interpolate", .5), curve.calls)
    def test_nonmonotonic_easing_can_reenter_outline_phase(self):
        animation = self.drawing()
        animation.begin()
        animation.interpolate(.8)
        self.assertGreater(animation.mobject.fill_opacity, 0)
        animation.interpolate(.2)
        self.assertEqual(animation.mobject.fill_opacity, 0)
        self.assertEqual(animation.mobject.stroke_width, 2.)
        self.assertEqual(animation.mobject.get_points()[-1, 0], .4)
        animation.interpolate(.75)
        self.assertAlmostEqual(animation.mobject.fill_opacity, .35)
    def test_nonzero_rate_at_begin_is_not_overwritten(self):
        animation = self.drawing(rate_func=lambda t:1-t)
        animation.begin()
        self.assertAlmostEqual(animation.mobject.fill_opacity, .7)
        self.assertEqual(animation.mobject.stroke_width, 5.)
    def test_outline_and_starting_helpers_keep_independent_data(self):
        animation = self.drawing()
        animation.begin()
        self.assertIsNot(animation.outline, animation.mobject)
        self.assertIsNot(animation.starting_mobject, animation.mobject)
        helpers = animation.get_all_mobjects_to_update()
        self.assertEqual(helpers, [animation.starting_mobject, animation.outline])
        animation.outline.uniforms["pulse"] = 3.
        self.assertNotIn("pulse", animation.mobject.uniforms)
        animation.interpolate(.25)
        self.assertEqual(animation.mobject.uniforms["pulse"], 3.)
    def test_outline_helper_updaters_run_on_shared_lifecycle(self):
        animation = self.drawing()
        animation.begin()
        ticks = []
        animation.outline.updaters.append(lambda mob,dt:ticks.append(dt))
        animation.update_mobjects(.125)
        self.assertEqual(ticks, [.125])
    def test_family_lag_runs_both_phases_per_member(self):
        root = self.g["VGroup"](self.curve(), self.curve())
        animation = self.Border(root, lag_ratio=.5, rate_func=self.g["linear"])
        animation.begin()
        animation.interpolate(.375)
        # Root alpha=.75, first child=.25, second child=0.
        self.assertEqual(root.submobjects[0].get_points()[-1,0], .5)
        self.assertEqual(root.submobjects[1].get_points()[-1,0], 0.)
        animation.finish()
        self.assertTrue(all(mob.fill_opacity == .7 for mob in root.submobjects))
    def test_time_span_and_final_alpha(self):
        animation = self.drawing(run_time=2, time_span=(.5, 1.5), final_alpha_value=.625)
        animation.begin()
        animation.finish()
        self.assertAlmostEqual(animation.mobject.fill_opacity, .35)
    def test_write_defaults_and_constructor_timing_hooks(self):
        animation = self.Write(self.curve())
        self.assertIs(animation.rate_func, self.g["linear"])
        self.assertEqual(animation.run_time, 1)
        self.assertEqual(animation.lag_ratio, .2)
        self.assertEqual(animation.stroke_color, animation.mobject.get_color())
        class SlowWrite(self.Write):
            def compute_run_time(self, family_size, run_time):
                return 7.
            def compute_lag_ratio(self, family_size, lag_ratio):
                return .4
        custom = SlowWrite(self.curve())
        self.assertEqual((custom.run_time, custom.lag_ratio), (7., .4))
    def test_border_default_rate_is_double_smooth(self):
        animation = self.Border(self.curve())
        animation._ensure_runtime_defaults()
        self.assertIs(animation.rate_func, self.g["double_smooth"])
    def test_write_nondefault_border_width_is_not_silently_dropped(self):
        animation = self.Write(self.curve(), stroke_width=11.)
        self.assertTrue(self.g["_requires_python_animation"](animation))
        animation.begin()
        animation.interpolate(.25)
        self.assertEqual(animation.mobject.stroke_width, 11.)
    def test_native_get_outline_bypass_negative_control(self):
        g = environment()
        class OldOutline(g["DrawBorderThenFill"]):
            def get_outline(self):
                raise AssertionError("hook must be reached")
        old = OldOutline(line(g))
        self.assertFalse(g["_requires_python_animation"](old))
        semantics._install_border_write(g)
        class Authored(g["DrawBorderThenFill"]):
            def get_outline(self):
                raise AssertionError("hook must be reached")
        with self.assertRaisesRegex(AssertionError, "hook must be reached"):
            g["Scene"]().play(Authored(line(g)))
    def test_invalid_outline_fails_without_stranding_state(self):
        for invalid in (None, "not a mobject"):
            animation = self.drawing(suspend_mobject_updating=True)
            animation.get_outline = lambda:invalid
            with self.assertRaisesRegex(TypeError, "must return a VMobject"):
                self.g["Scene"]().play(animation)
            self.assertFalse(animation.mobject.animating)
            self.assertFalse(animation.mobject.suspended)
    def test_aliased_outline_is_rejected(self):
        animation = self.drawing()
        animation.get_outline = lambda: animation.mobject
        with self.assertRaisesRegex(ValueError, "must not alias"):
            animation.begin()
        self.assertFalse(animation.mobject.animating)
    def test_callback_or_scene_failure_releases_owned_suspension(self):
        for fault in ("fail_prologue", "fail_updater"):
            scene = self.g["Scene"]()
            setattr(scene, fault, True)
            animation = self.drawing(suspend_mobject_updating=True, final_alpha_value=.75)
            with self.assertRaises(ValueError):
                scene.play(self.g["AnimationGroup"](animation))
            self.assertFalse(animation.mobject.animating)
            self.assertFalse(animation.mobject.suspended)
            animation.abort()
            self.assertEqual(animation.mobject.calls.count(("resume",)), 1)
    def test_preexisting_suspension_is_not_released(self):
        animation = self.drawing(suspend_mobject_updating=True, final_alpha_value=.75)
        animation.mobject.suspend_updating()
        scene = self.g["Scene"]()
        scene.fail_updater = True
        with self.assertRaises(ValueError):
            scene.play(animation)
        self.assertTrue(animation.mobject.suspended)
    def test_helper_failure_keeps_primary_exception(self):
        animation = self.drawing(suspend_mobject_updating=True)
        animation.begin()
        def fail(mob,dt):
            raise RuntimeError("outline-helper-failure")
        animation.outline.updaters.append(fail)
        with self.assertRaisesRegex(RuntimeError, "outline-helper-failure"):
            animation.update_mobjects(.1)
        self.assertFalse(animation.mobject.suspended)
    def test_remover_and_final_alpha_force_callback(self):
        for kwargs in ({"remover":True}, {"final_alpha_value":.75}):
            animation = self.drawing(**kwargs)
            self.assertTrue(self.g["_requires_python_animation"](animation))
            scene = self.g["Scene"]()
            scene.play(animation)
            if animation.remover:
                self.assertNotIn(animation.mobject, scene.roots)
            else:
                self.assertAlmostEqual(animation.mobject.fill_opacity, .35)
    def test_scene_rate_override_is_normalized(self):
        animation = self.drawing(final_alpha_value=.75)
        self.g["Scene"]().play(animation, rate_func="linear")
        self.assertAlmostEqual(animation.mobject.fill_opacity, .35)
    def test_rebegin_resets_phase_bookkeeping(self):
        animation = self.drawing()
        animation.begin()
        animation.interpolate(.75)
        animation.begin()
        self.assertEqual(animation.sm_to_index[hash(animation.mobject)], 0)
        animation.abort()
    def test_nonfinite_phase_refuses_without_contaminating_records(self):
        animation = self.drawing(suspend_mobject_updating=True)
        animation.begin()
        before = animation.mobject.data.copy()
        with self.assertRaisesRegex(ValueError, "alpha must be finite"):
            animation.interpolate(float("nan"))
        np.testing.assert_array_equal(animation.mobject.data, before)
        self.assertFalse(animation.mobject.suspended)
    def test_write_rejects_non_vector_input_by_name(self):
        for value in (None, object(), self.g["Mobject"]()):
            with self.assertRaisesRegex(TypeError, "Write requires a VMobject"):
                self.Write(value)
    def test_outline_geometry_is_not_replaced_by_original_during_first_phase(self):
        animation = self.drawing()
        old_outline = animation.get_outline
        def outline():
            result = old_outline()
            result.data["point"][:, 0] += 2.
            return result
        animation.get_outline = outline
        animation.begin()
        animation.interpolate(.25)
        np.testing.assert_allclose(animation.mobject.get_points()[[0, -1], 0], [2., 2.5])
        animation.interpolate(.75)
        np.testing.assert_allclose(animation.mobject.get_points()[[0, -1], 0], [1., 2.])
        animation.finish()
        np.testing.assert_allclose(animation.mobject.get_points()[[0, -1], 0], [0., 1.])
    def test_empty_group_has_complete_lifecycle(self):
        animation = self.Border(self.g["VGroup"]())
        animation.begin()
        animation.finish()
        self.assertFalse(animation.mobject.animating)


if __name__ == "__main__":
    unittest.main()
