"""Animation-to-updater production orchestration; native storage is a double."""
import ast
import copy
import importlib.util
import math
from pathlib import Path
import sys
import types
import unittest
from unittest.mock import patch
import numpy as np

ROOT = Path(__file__).resolve().parents[1]
PATH = ROOT / "python/fmn_python/animation_updaters.py"
SPEC = importlib.util.spec_from_file_location("fmn_python.animation_updaters", PATH)
adapter = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(adapter)


def environment():
    native = types.ModuleType("manimlib.test_persistent_native")
    g = vars(native)
    class Mobject:
        def __init__(self, value=0., *children):
            self.value = value
            self.submobjects = list(children)
            self.updaters = []
            self.suspended = self.animating = False
            self._scene = None
            self.events = []
        def get_family(self):
            result = [self]
            for child in self.submobjects:
                for member in child.get_family():
                    if all(member is not old for old in result):
                        result.append(member)
            return result
        def copy(self):
            result = copy.copy(self)
            result.submobjects = [child.copy() for child in self.submobjects]
            result.updaters, result.events = list(self.updaters), []
            return result
        def add_updater(self, updater, call=True):
            self.updaters.append(updater)
            if call:
                self.update(0.)
            return self
        def remove_updater(self, updater):
            self.updaters[:] = [old for old in self.updaters if old is not updater]
        def update(self, dt=0., recurse=True):
            if self.suspended:
                return self
            if recurse:
                for child in list(self.submobjects):
                    child.update(dt)
            for updater in list(self.updaters):
                updater(self, dt)
            return self
        def _is_updating_suspended(self):
            return self.suspended
        def suspend_updating(self, recurse=True):
            for member in self.get_family() if recurse else [self]:
                member.suspended = True
        def resume_updating(self, recurse=True, call_updater=True):
            for member in self.get_family() if recurse else [self]:
                member.suspended = False
            if call_updater:
                self.update(0., recurse)
        def set_animating_status(self, value):
            self.animating = value
        def unlock_data(self):
            self.events.append("unlock")
    class Animation:
        _native_kind = None
    class Transform(Animation):
        pass
    class AnimationGroup(Animation):
        pass
    def old_turn(animation, cycle=False, **kwargs):
        if type(animation).interpolate_mobject is Animation.interpolate_mobject:
            raise NotImplementedError("requires a Python-driven animation")
    old_turn.__module__ = "manimlib.mobject.mobject_update_utils"
    g.update(Mobject=Mobject, Animation=Animation, Transform=Transform, AnimationGroup=AnimationGroup,
             turn_animation_into_updater=old_turn, cycle_animation=lambda *a, **k: None,
             smooth=lambda t:t, smooth_rate=lambda t:t, np=np)
    g["g"] = g
    source = ROOT / "python/manimlib/_animation_semantics.py"
    tree = ast.parse(source.read_text())
    install = next(node for node in tree.body if isinstance(node, ast.FunctionDef) and node.name == "install")
    functions = {node.name:node for node in install.body if isinstance(node, ast.FunctionDef)}
    mapping = {
        "__init__":"animation_init", "_validate_input_type":"validate_input_type",
        "_ensure_runtime_defaults":"ensure_runtime_defaults", "begin":"animation_begin",
        "finish":"animation_finish", "create_starting_mobject":"create_starting_mobject",
        "get_all_mobjects":"get_all_mobjects", "get_all_families_zipped":"get_all_families_zipped",
        "get_all_mobjects_to_update":"get_all_mobjects_to_update", "update_mobjects":"update_mobjects",
        "interpolate":"animation_interpolate", "interpolate_mobject":"interpolate_mobject",
        "interpolate_submobject":"interpolate_submobject", "time_spanned_alpha":"time_spanned_alpha",
        "get_sub_alpha":"get_sub_alpha", "get_run_time":"get_run_time", "update_rate_info":"update_rate_info",
    }
    for name, function in mapping.items():
        exec(compile(ast.Module([functions[function]], []), str(source), "exec"), g)
        setattr(Animation, name, g[function])
    class PerMember(Animation):
        _native_kind = "fixture_native"
        def __init__(self, mob, **kwargs):
            super().__init__(mob, **kwargs)
            self.events = []
        def interpolate_submobject(self, current, start, alpha):
            current.value = start.value + alpha
            self.events.append(("interpolate", current, float(alpha)))
        def update_mobjects(self, dt):
            self.events.append(("helpers", dt))
            super().update_mobjects(dt)
        def finish(self):
            self.events.append(("finish",))
            super().finish()
    native.PerMember = PerMember
    return native


class PersistentProtocolTests(unittest.TestCase):
    def setUp(self):
        self.native = environment()
        self.old = self.native.turn_animation_into_updater
        adapter.install_animation_updaters(self.native)
        self.mob = self.native.Mobject()
    def attach(self, **kwargs):
        animation = self.native.PerMember(self.mob, **kwargs)
        result = self.native.turn_animation_into_updater(animation)
        self.assertIs(result, self.mob)
        return animation
    def test_per_member_protocol_no_longer_rejected(self):
        animation = self.native.PerMember(self.mob)
        with self.assertRaises(NotImplementedError):
            self.old(animation)
        self.native.turn_animation_into_updater(animation)
        self.mob.update(.5).update(0.)
        self.assertEqual(self.mob.value, .5)
    def test_helper_updates_follow_interpolation(self):
        animation = self.attach()
        animation.events.clear()
        self.mob.update(.2)
        self.assertEqual([event[0] for event in animation.events], ["interpolate", "helpers"])
        self.assertEqual(animation.events[-1][1], .2)
    def test_starting_copy_helper_really_updates(self):
        animation = self.attach()
        seen = []
        animation.starting_mobject.add_updater(lambda m,dt:seen.append(dt), call=False)
        self.mob.update(.25)
        self.assertEqual(seen, [.25])
    def test_time_span_uses_public_effective_duration(self):
        animation = self.attach(run_time=1., time_span=(1., 3.))
        self.mob.update(2.).update(0.)
        self.assertEqual(animation.run_time, 3.)
        self.assertEqual(self.mob.value, .5)
        self.assertTrue(self.mob.updaters)
    def test_family_lag_is_preserved(self):
        child = self.native.Mobject()
        self.mob.submobjects.append(child)
        self.attach(lag_ratio=1.)
        self.mob.update(.5).update(0.)
        self.assertEqual(self.mob.value, 1.)
        self.assertEqual(child.value, 0.)
    def test_finish_once_uses_final_alpha(self):
        animation = self.attach(final_alpha_value=.3)
        self.mob.update(1.).update(0.).update(5.)
        self.assertEqual(self.mob.value, .3)
        self.assertEqual(animation.events.count(("finish",)), 1)
        self.assertEqual(self.mob.updaters, [])
        self.assertFalse(self.mob.animating)
    def test_remover_does_not_trigger_scene_cleanup(self):
        animation = self.attach(remover=True)
        animation.clean_up_from_scene = lambda scene:self.fail("helper must not perform Scene cleanup")
        self.mob.update(1.).update(0.)
        self.assertEqual(self.mob.value, 1.)
    def test_cycles_without_finish_or_second_clock(self):
        animation = self.native.PerMember(self.mob)
        self.native.cycle_animation(animation)
        self.mob.update(2.25).update(0.)
        self.assertEqual(self.mob.value, .25)
        self.assertNotIn(("finish",), animation.events)
        self.assertEqual(animation.total_time, 2.25)
    def test_runtime_edits_are_observed(self):
        animation = self.attach()
        self.mob.update(.5)
        animation.run_time = 2.
        self.mob.update(0.)
        self.assertEqual(self.mob.value, .25)
    def test_rate_override_is_used(self):
        animation = self.native.PerMember(self.mob)
        self.native.turn_animation_into_updater(animation, rate_func=lambda t:t*t, run_time=2.)
        self.mob.update(1.).update(0.)
        self.assertEqual(self.mob.value, .25)
    def test_custom_rate_info_hook_is_called_once(self):
        animation = self.native.PerMember(self.mob)
        seen = []
        original = animation.update_rate_info
        animation.update_rate_info = lambda **kwargs:(seen.append(kwargs), original(**kwargs))[-1]
        self.native.turn_animation_into_updater(animation, run_time=2.)
        self.assertEqual(seen, [{"run_time":2.}])
    def test_copied_updater_never_advances_original(self):
        animation = self.attach()
        duplicate = self.mob.copy()
        duplicate.update(2.)
        self.assertEqual(animation.total_time, 0.)
        self.assertEqual(self.mob.value, 0.)
    def test_reentrant_original_update_is_ignored(self):
        animation = self.attach()
        animation.starting_mobject.add_updater(lambda m,dt:self.mob.update(dt), call=False)
        self.mob.update(.25)
        self.assertEqual(animation.total_time, .25)
    def test_suspension_pauses_updater_time(self):
        animation = self.attach(suspend_mobject_updating=True)
        self.assertFalse(animation.suspend_mobject_updating)
        self.mob.suspend_updating()
        self.mob.update(2.)
        self.assertEqual(animation.total_time, 0.)
        self.mob.resume_updating(call_updater=False)
        self.mob.update(.5)
        self.assertEqual(animation.total_time, .5)
    def test_preexisting_child_suspension_is_not_changed(self):
        child = self.native.Mobject()
        child.suspend_updating()
        self.mob.submobjects.append(child)
        self.attach()
        self.mob.update(1.).update(0.)
        self.assertTrue(child.suspended)
    def test_zero_duration_finishes_during_registration(self):
        animation = self.attach(run_time=0., final_alpha_value=.8)
        self.assertEqual(self.mob.value, .8)
        self.assertEqual(self.mob.updaters, [])
        self.assertEqual(animation.events.count(("finish",)), 1)
    def test_zero_duration_cycle_refuses_before_begin(self):
        animation = self.native.PerMember(self.mob, run_time=0.)
        with self.assertRaisesRegex(ValueError, "positive"):
            self.native.cycle_animation(animation)
        self.assertFalse(self.mob.animating)
        self.assertEqual(self.mob.updaters, [])
    def test_bad_duration_is_rejected_before_begin(self):
        for duration in (-1., math.nan, math.inf):
            animation = self.native.PerMember(self.mob, run_time=duration)
            with self.assertRaises(ValueError):
                self.native.turn_animation_into_updater(animation)
        self.assertEqual(self.mob.updaters, [])
    def test_nonfinite_dt_aborts_and_detaches(self):
        animation = self.attach()
        with self.assertRaises(ValueError):
            self.mob.update(math.nan)
        self.assertEqual(self.mob.updaters, [])
        self.assertFalse(self.mob.animating)
        self.assertEqual(animation.total_time, 0.)
    def test_begin_failure_preserves_original_error(self):
        animation = self.native.PerMember(self.mob)
        error = RuntimeError("begin failed")
        def begin():
            self.mob.set_animating_status(True)
            raise error
        animation.begin = begin
        with self.assertRaises(RuntimeError) as caught:
            self.native.turn_animation_into_updater(animation)
        self.assertIs(caught.exception, error)
        self.assertFalse(self.mob.animating)
        self.assertEqual(self.mob.updaters, [])
    def test_interpolation_error_aborts_and_detaches(self):
        animation = self.attach()
        error = RuntimeError("interpolate failed")
        animation.interpolate = lambda alpha:(_ for _ in ()).throw(error)
        with self.assertRaises(RuntimeError) as caught:
            self.mob.update(.1)
        self.assertIs(caught.exception, error)
        self.assertEqual(self.mob.updaters, [])
    def test_helper_error_aborts_and_detaches(self):
        animation = self.attach()
        animation.update_mobjects = lambda dt:(_ for _ in ()).throw(KeyboardInterrupt())
        with self.assertRaises(KeyboardInterrupt):
            self.mob.update(.1)
        self.assertFalse(self.mob.animating)
        self.assertEqual(self.mob.updaters, [])
    def test_finish_error_aborts_and_detaches(self):
        animation = self.attach()
        self.mob.update(1.)
        animation.finish = lambda:(_ for _ in ()).throw(RuntimeError("finish"))
        with self.assertRaisesRegex(RuntimeError, "finish"):
            self.mob.update(0.)
        self.assertEqual(self.mob.updaters, [])
        self.assertFalse(self.mob.animating)
    def test_abort_error_cannot_replace_callback_error(self):
        animation = self.attach()
        error = RuntimeError("original")
        animation.abort = lambda:(_ for _ in ()).throw(ValueError("abort"))
        animation.interpolate = lambda alpha:(_ for _ in ()).throw(error)
        with self.assertRaises(RuntimeError) as caught:
            self.mob.update(0.)
        self.assertIs(caught.exception, error)
        self.assertTrue(error.__notes__)
        self.assertEqual(self.mob.updaters, [])
    def test_old_registration_only_detaches_its_own_updater(self):
        self.attach()
        other = lambda m,dt:None
        self.mob.add_updater(other, call=False)
        self.mob.update(1.).update(0.)
        self.assertEqual(self.mob.updaters, [other])
    def test_live_method_replacement_is_observed(self):
        animation = self.attach()
        seen = []
        animation.interpolate = lambda alpha:seen.append(alpha)
        self.mob.update(.2).update(0.)
        self.assertEqual(seen, [0., .2])
    def test_invalid_animation_and_unknown_kwargs_refuse(self):
        with self.assertRaises(TypeError):
            self.native.turn_animation_into_updater(object())
        with self.assertRaises(TypeError):
            self.native.turn_animation_into_updater(self.native.PerMember(self.mob), unknown=True)
    def test_native_only_detached_animation_has_explicit_refusal(self):
        class NativeOnly(self.native.Animation):
            _native_kind = "v_show_passing_flash"
        with self.assertRaisesRegex(NotImplementedError, "scene-bound"):
            self.native.turn_animation_into_updater(NativeOnly(self.mob))
    def test_aliases_and_later_user_replacements_survive(self):
        native = environment()
        module = types.ModuleType("manimlib.mobject.mobject_update_utils")
        module.turn_animation_into_updater = native.turn_animation_into_updater
        module.other_alias = native.turn_animation_into_updater
        with patch.dict(sys.modules, {module.__name__:module}):
            adapter.install_animation_updaters(native)
            self.assertIs(module.other_alias, native.turn_animation_into_updater)
            self.assertEqual(native.turn_animation_into_updater.__module__, module.__name__)
            replacement = lambda:None
            native.turn_animation_into_updater = replacement
            adapter.install_animation_updaters(native)
            self.assertIs(native.turn_animation_into_updater, replacement)


if __name__ == "__main__":
    unittest.main()
