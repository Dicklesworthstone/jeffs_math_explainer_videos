"""Execution ownership over an explicit native-loop fixture, not native output.

The fixture preserves the production boundary's ordering: all callback begins
precede native construction, then helper/interpolation, finish and cleanup.
The companion installed-wheel suite is required to validate real native state.
"""
from __future__ import annotations

import importlib.util
from pathlib import Path
import types
import unittest

MODULE = Path(__file__).resolve().parents[1] / "python/fmn_python/scene_execution.py"
spec = importlib.util.spec_from_file_location("_scene_execution_under_test", MODULE)
execution = importlib.util.module_from_spec(spec)
spec.loader.exec_module(execution)


def environment(install=True):
    class Mob:
        def __init__(self, *children):
            self.submobjects = list(children)
            self.parents = []
            for child in children:
                child.parents.append(self)
            self.suspended = False
            self._is_animating = False
            self.locked_data_keys = {"retained"}
            self.const_data_keys = set()
            self.locked_uniform_keys = {"retained-uniform"}
            self.resume_calls = []
        def get_family(self):
            result, pending, seen = [], [self], set()
            while pending:
                mob = pending.pop()
                if id(mob) in seen:
                    continue
                seen.add(id(mob)); result.append(mob)
                pending.extend(reversed(mob.submobjects))
            return result
        def _is_updating_suspended(self):
            return self.suspended
        def suspend_updating(self, recurse=True):
            for mob in self.get_family() if recurse else [self]:
                mob.suspended = True
        def resume_updating(self, recurse=True, call_updater=True):
            self.resume_calls.append((recurse, call_updater))
            for mob in self.get_family() if recurse else [self]:
                mob.suspended = False
        def set_animating_status(self, flag):
            pending, seen = self.get_family(), set()
            while pending:
                mob = pending.pop()
                if id(mob) in seen:
                    continue
                seen.add(id(mob)); mob._is_animating = flag
                pending.extend(mob.parents)
    class Animation:
        def __init__(self, mob, fail=None, error=None):
            self.mobject, self.fail = mob, fail
            self.error = error if error is not None else LookupError("primary")
            self.events = []
            self.on_interpolate = None
        def update_rate_info(self, **kwargs):
            self.__dict__.update(kwargs)
        def event(self, name):
            self.events.append(name)
            if name == self.fail:
                raise self.error
        def begin(self):
            self.mobject.set_animating_status(True)
            self.mobject.suspend_updating()
            self.mobject.locked_data_keys = {"animation-lock"}
            self.mobject.const_data_keys.add("animation-constant")
            self.event("begin")
        def update_mobjects(self, dt):
            self.event(("helpers", dt))
        def interpolate(self, alpha):
            self.event("interpolate")
            if self.on_interpolate:
                self.on_interpolate()
        def finish(self):
            self.event("finish")
            self.mobject.set_animating_status(False)
            self.mobject.resume_updating(call_updater=False)
        def clean_up_from_scene(self, scene):
            self.event("cleanup")
    class AnimationGroup(Animation):
        def __init__(self, *animations):
            super().__init__(Mob(*(anim.mobject for anim in animations)))
            self.animations = animations
    class Scene:
        default_wait_time = 1.0
        def pre_play(self):
            pass
        def post_play(self):
            self.num_plays += 1
        def __init__(self):
            self.native_failure = None
            self.outer_failure = None
            self.events = []
            self.num_plays = 0
            self._last_specs = None
        def _play_animations(self, specs, callbacks, camera, run_time, rate_func, lag_ratio):
            self._last_specs = specs
            self.events.append("core")
            for callback in callbacks:
                if callback is not None:
                    callback.begin()
            if self.native_failure is not None:
                raise self.native_failure
            for callback in callbacks:
                if callback is not None:
                    if hasattr(callback, "update_mobjects"):
                        callback.update_mobjects(0.25)
                    callback.interpolate(0.25)
            for callback in callbacks:
                if callback is not None:
                    callback.finish()
                    if hasattr(callback, "clean_up_from_scene"):
                        callback.clean_up_from_scene(self)
            return [0.25]
        def play(self, *animations, **kwargs):
            if not animations:
                return None
            try:
                return self._play_animations(animations, list(animations), None, None, None, None)
            except BaseException:
                if self.outer_failure is not None:
                    raise self.outer_failure
                raise
        def wait(self, duration=1, stop_condition=None, **kwargs):
            self.events.append(("wait", duration))
            if stop_condition:
                stop_condition()
            if self.native_failure:
                raise self.native_failure
    class _AnimationBuilder:
        def __init__(self, mobject):
            self.mobject = mobject
        def build(self):
            return Animation(self.mobject)
    def prepare_animation(item):
        if isinstance(item, _AnimationBuilder):
            return item.build()
        if not isinstance(item, Animation):
            raise TypeError("expected Animation")
        return item
    g = types.SimpleNamespace(Scene=Scene, Mobject=Mob, Animation=Animation,
                              AnimationGroup=AnimationGroup, _AnimationBuilder=_AnimationBuilder,
                              prepare_animation=prepare_animation)
    if install:
        execution.install_scene_execution(g)
    return g


class ExecutionTests(unittest.TestCase):
    def setUp(self):
        self.g = environment()
        self.scene, self.mob = self.g.Scene(), self.g.Mobject()
    def animation(self, *args, **kwargs):
        return self.g.Animation(self.mob, *args, **kwargs)
    def assert_released(self, mob=None):
        mob = self.mob if mob is None else mob
        self.assertFalse(mob.suspended)
        self.assertFalse(mob._is_animating)
        self.assertEqual(mob.locked_data_keys, {"retained"})
        self.assertEqual(mob.const_data_keys, set())
        self.assertEqual(mob.locked_uniform_keys, {"retained-uniform"})
    def test_success_retains_order_arguments_and_result(self):
        anim = self.animation()
        self.assertEqual(self.scene.play(anim), [0.25])
        self.assertEqual(anim.events, ["begin", ("helpers", .25), "interpolate", "finish", "cleanup"])
        self.assertIs(self.scene._last_specs[0], anim)
        self.assertNotIn(execution._OWNER_KEY, vars(self.scene))
    def test_begin_failure_releases_its_own_partial_state(self):
        anim = self.animation(fail="begin")
        with self.assertRaises(LookupError) as result:
            self.scene.play(anim)
        self.assertIs(result.exception, anim.error)
        self.assert_released()
        self.assertNotIn("finish", anim.events)
        self.assertNotIn("cleanup", anim.events)
    def test_later_begin_failure_recovers_all_begun_callbacks(self):
        first = self.animation()
        second = self.g.Animation(self.g.Mobject(), fail="begin")
        third = self.g.Animation(self.g.Mobject())
        with self.assertRaises(LookupError):
            self.scene.play(first, second, third)
        self.assert_released()
        self.assert_released(second.mobject)
        self.assertEqual(third.events, [])
    def test_native_construction_failure_after_begins_is_covered(self):
        self.scene.native_failure = ValueError("native lowering")
        with self.assertRaisesRegex(ValueError, "native lowering"):
            self.scene.play(self.animation())
        self.assert_released()
    def test_interpolate_failure_restores_original_set_objects(self):
        locked = self.mob.locked_data_keys
        with self.assertRaises(LookupError):
            self.scene.play(self.animation(fail="interpolate"))
        self.assert_released()
        self.assertIs(self.mob.locked_data_keys, locked)
    def test_helper_failure_is_covered(self):
        anim = self.animation(fail=("helpers", .25))
        with self.assertRaises(LookupError):
            self.scene.play(anim)
        self.assert_released()
        self.assertNotIn("interpolate", anim.events)
    def test_finish_failure_does_not_finish_again_or_cleanup(self):
        anim = self.animation(fail="finish")
        with self.assertRaises(LookupError):
            self.scene.play(anim)
        self.assert_released()
        self.assertEqual(anim.events.count("finish"), 1)
        self.assertNotIn("cleanup", anim.events)
    def test_cleanup_failure_restores_prior_suspended_descendant(self):
        child = self.g.Mobject(); child.suspended = True
        root = self.g.Mobject(child)
        anim = self.g.Animation(root, fail="cleanup")
        with self.assertRaises(LookupError):
            self.scene.play(anim)
        self.assertFalse(root.suspended)
        self.assertTrue(child.suspended)
    def test_shared_descendant_uses_pre_batch_ownership(self):
        shared = self.g.Mobject()
        left, right = self.g.Mobject(shared), self.g.Mobject(shared)
        first = self.g.Animation(left)
        second = self.g.Animation(right, fail="begin")
        with self.assertRaises(LookupError):
            self.scene.play(first, second)
        for mob in (shared, left, right):
            self.assert_released(mob)
        self.assertEqual(shared.resume_calls, [(False, False)])
    def test_ancestor_animating_state_and_siblings_are_preserved(self):
        sibling = self.g.Mobject()
        parent = self.g.Mobject(self.mob, sibling)
        parent._is_animating = True
        with self.assertRaises(LookupError):
            self.scene.play(self.animation(fail="begin"))
        self.assertTrue(parent._is_animating)
        self.assertFalse(sibling._is_animating)
        self.assert_released()
    def test_abort_called_in_reverse_order_never_finishes(self):
        order = []
        first = self.animation()
        second = self.g.Animation(self.g.Mobject(), fail="begin")
        first.abort = lambda: order.append("first")
        second.abort = lambda: order.append("second")
        with self.assertRaises(LookupError):
            self.scene.play(first, second)
        self.assertEqual(order, ["second", "first"])
        self.assert_released()
    def test_abort_failure_does_not_mask_primary_or_stop_recovery(self):
        anim = self.animation(fail="begin")
        def abort():
            raise RuntimeError("secondary")
        anim.abort = abort
        with self.assertRaises(LookupError) as result:
            self.scene.play(anim)
        self.assertIs(result.exception, anim.error)
        self.assertIn("RuntimeError", " ".join(result.exception.__notes__))
        self.assert_released()
    def test_outer_legacy_cleanup_cannot_replace_execution_failure(self):
        anim = self.animation(fail="interpolate")
        self.scene.outer_failure = RuntimeError("old wrapper cleanup")
        with self.assertRaises(LookupError) as result:
            self.scene.play(anim)
        self.assertIs(result.exception, anim.error)
        self.assert_released()
    def test_interrupts_and_system_exit_are_preserved(self):
        for error in (KeyboardInterrupt(), SystemExit(7)):
            anim = self.animation(fail="begin", error=error)
            with self.assertRaises(type(error)) as result:
                self.scene.play(anim)
            self.assertIs(result.exception, error)
            self.assert_released()
    def test_private_core_call_has_its_own_failure_owner(self):
        anim = self.animation(fail="begin")
        with self.assertRaises(LookupError):
            self.scene._play_animations([object()], [anim], None, None, None, None)
        self.assert_released()
        self.assertNotIn(execution._OWNER_KEY, vars(self.scene))
    def test_second_play_after_failure_is_not_blocked(self):
        with self.assertRaises(LookupError):
            self.scene.play(self.animation(fail="begin"))
        self.assertEqual(self.scene.play(self.animation()), [.25])
    def test_reentrant_same_scene_play_and_wait_fail_before_native_work(self):
        for operation in (lambda: self.scene.play(self.animation()), lambda: self.scene.wait(.1)):
            self.scene.events.clear()
            anim = self.animation(); anim.on_interpolate = operation
            with self.assertRaisesRegex(RuntimeError, "another play/wait"):
                self.scene.play(anim)
            self.assertEqual(self.scene.events, ["core"])
            self.assert_released()
    def test_nested_other_scene_is_independent(self):
        other = self.g.Scene()
        anim = self.animation(); anim.on_interpolate = lambda: other.wait(.1)
        self.assertEqual(self.scene.play(anim), [.25])
        self.assertEqual(other.events, [("wait", .1)])
    def test_native_only_slots_remain_none_and_specs_unchanged(self):
        specs = [object(), object()]
        self.assertEqual(self.scene._play_animations(specs, [None, None], None, None, None, None), [.25])
        self.assertIs(self.scene._last_specs, specs)
    def test_no_abort_for_callback_that_finished_before_later_failure(self):
        first, second = self.animation(), self.g.Animation(self.g.Mobject(), fail="finish")
        first.abort = lambda: self.fail("completed animation was aborted")
        with self.assertRaises(LookupError):
            self.scene.play(first, second)
        self.assert_released()
    def test_no_recovery_updater_ticks(self):
        with self.assertRaises(LookupError):
            self.scene.play(self.animation(fail="begin"))
        self.assertEqual(self.mob.resume_calls, [(False, False)])
    def test_old_loop_demonstrates_the_partial_begin_leak(self):
        old = environment(False)
        mob = old.Mobject()
        with self.assertRaises(LookupError):
            old.Scene().play(old.Animation(mob, fail="begin"))
        self.assertTrue(mob.suspended)
        self.assertTrue(mob._is_animating)
    def test_private_camera_clock_exposes_only_its_public_animation_roots(self):
        class Clock:
            def __init__(self, animations):
                self.animations = tuple(animations)
        self.g._fmn_camera_clock_driver_type = Clock
        camera = self.g.Mobject()
        first, second = self.animation(), self.g.Animation(camera)
        group = self.g.AnimationGroup(first, second)
        clock = Clock([group, first])
        roots = list(execution._roots(clock, vars(self.g)))
        self.assertEqual(roots, [group.mobject, self.mob, camera])

        class Unrelated:
            @property
            def animations(self):
                self.fail("arbitrary animation-like attributes were traversed")
        self.assertEqual(list(execution._roots(Unrelated(), vars(self.g))), [])

    def test_installation_is_idempotent(self):
        play = self.g.Scene.play
        execution.install_scene_execution(self.g)
        self.assertIs(self.g.Scene.play, play)


if __name__ == "__main__":
    unittest.main()
