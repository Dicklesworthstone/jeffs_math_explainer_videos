"""Scene-scoped speed-updater contracts; native cases require FMN_TEST_NATIVE=1."""
from __future__ import annotations

import importlib.util
import os
from pathlib import Path
import types
import unittest

from test_speed import protocol

_SOURCE = Path(__file__).resolve().parents[1] / "python/fmn_python/speed_updaters.py"
_spec = importlib.util.spec_from_file_location("speed_updaters_under_test", _SOURCE)
adapter = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(adapter)


class Mobject:
    def __init__(self, scene):
        self._scene, self.updaters = scene, []

    # Models the portal Mobject.add_updater(updater, index=None, call=True);
    # the Reference names the immediate-call flag `call`.
    def add_updater(self, function, index=None, call=False):
        if index is None:
            self.updaters.append(function)
        else:
            self.updaters.insert(index, function)
        if call:
            function(self, 0.0)
        return self

    def tick(self, dt):
        for function in self.updaters:
            function(self, dt)


class ScopeTests(unittest.TestCase):
    def setUp(self):
        self.g = protocol()
        self.g.Mobject = Mobject
        adapter.install_speed_updaters(self.g)
        self.scene = types.SimpleNamespace()
        self.mob = Mobject(self.scene)
        self.deltas = []
        self.g.ChangeSpeed.add_updater(self.mob, lambda mob, dt: self.deltas.append(dt))
        self.active = []

    def tearDown(self):
        for animation in reversed(self.active):
            try:
                animation.abort()
            except BaseException:
                pass

    def make(self, *, scene=None, child=None, profile=None, **kwargs):
        child = self.g.Animation(duration=2) if child is None else child
        animation = self.g.ChangeSpeed(child, {0: 0, 1: 2} if profile is None else profile, **kwargs)
        animation._composition_scene = self.scene if scene is None else scene
        self.active.append(animation)
        return animation

    def test_preparing_multiple_wrappers_acquires_no_scene_clock(self):
        self.make()
        self.make()
        self.mob.tick(0.1)
        self.assertEqual(self.deltas, [0.1])
        self.assertNotIn(adapter._KEY, vars(self.scene))

    def test_current_child_delta_and_final_frame(self):
        animation = self.make()
        animation.begin()
        for alpha in (0.25, 0.5, 0.75, 1.0):
            animation.interpolate(alpha)
            self.mob.tick(0.5)
        self.assertEqual(self.deltas, [0.125, 0.375, 0.625, 0.875])
        self.assertEqual(sum(self.deltas), 2.0)
        animation.finish()
        self.mob.tick(0.1)
        self.assertEqual(self.deltas[-1], 0.1)
        self.assertNotIn(adapter._KEY, vars(self.scene))

    def test_zero_dt_registration_and_resume_never_move_time(self):
        animation = self.make()
        animation.begin()
        animation.interpolate(0.5)
        self.mob.tick(0.0)
        calls = []
        self.g.ChangeSpeed.add_updater(self.mob, lambda mob, dt: calls.append(dt), call_updater=True)
        self.assertEqual(self.deltas, [0.0])
        self.assertEqual(calls, [0.0])

    def test_ordinary_updaters_keep_wall_clock(self):
        ordinary = []
        self.mob.add_updater(lambda mob, dt: ordinary.append(dt))
        animation = self.make()
        animation.begin()
        animation.interpolate(0.5)
        self.mob.tick(0.25)
        self.assertEqual(ordinary, [0.25])
        self.assertEqual(self.deltas, [0.5])

    def test_other_scene_keeps_wall_clock(self):
        other = Mobject(types.SimpleNamespace())
        other_deltas = []
        self.g.ChangeSpeed.add_updater(other, lambda mob, dt: other_deltas.append(dt))
        animation = self.make()
        animation.begin()
        animation.interpolate(0.5)
        other.tick(0.25)
        self.assertEqual(other_deltas, [0.25])

    def test_two_scenes_may_own_independent_clocks(self):
        other_scene = types.SimpleNamespace()
        first = self.make()
        second = self.make(scene=other_scene, profile={0: 2, 1: 2})
        first.begin()
        second.begin()
        first.interpolate(0.5)
        second.interpolate(0.5)
        first.abort()
        self.assertNotIn(adapter._KEY, vars(self.scene))
        self.assertIn(adapter._KEY, vars(other_scene))
        second.abort()
        self.assertNotIn(adapter._KEY, vars(other_scene))

    def test_conflict_refuses_before_second_child_begin(self):
        first, second = self.make(), self.make()
        first.begin()
        with self.assertRaisesRegex(RuntimeError, "only one ChangeSpeed"):
            second.begin()
        self.assertEqual(second.anim.events, [])
        self.assertIs(vars(self.scene)[adapter._KEY].owner, first)
        first.abort()
        second.begin()
        self.assertIs(vars(self.scene)[adapter._KEY].owner, second)

    def test_non_affecting_wrapper_does_not_conflict(self):
        first, second = self.make(), self.make(affects_speed_updaters=False)
        first.begin()
        second.begin()
        first.interpolate(0.5)
        second.interpolate(0.75)
        self.mob.tick(0.25)
        self.assertEqual(self.deltas, [0.5])
        second.finish()
        self.assertIs(vars(self.scene)[adapter._KEY].owner, first)

    def test_helper_copies_never_consume_previous_frame_delta(self):
        animation = self.make()
        animation.begin()
        animation.interpolate(0.5)
        copied = Mobject(self.scene)
        copied.updaters = list(self.mob.updaters)
        copied.tick(0.125)
        self.assertEqual(self.deltas, [0.125])

    def test_authored_updates_inside_child_interpolation_are_not_retimed(self):
        child = self.g.Animation()
        interpolate = child.interpolate
        def invoke(alpha):
            self.mob.tick(0.125)
            interpolate(alpha)
        child.interpolate = invoke
        animation = self.make(child=child)
        animation.begin()
        animation.interpolate(0.5)
        self.mob.tick(0.25)
        self.assertEqual(self.deltas, [0.125, 0.125, 0.5])

    def test_child_helper_updates_keep_wall_clock(self):
        child = self.g.Animation()
        child.update_mobjects = self.mob.tick
        animation = self.make(child=child)
        animation.begin()
        animation.interpolate(0.5)
        animation.update_mobjects(0.125)
        self.assertEqual(self.deltas, [0.125])

    def test_failed_begin_does_not_leak_scene_clock(self):
        child = self.g.Animation()
        failure = RuntimeError("begin")
        def begin():
            raise failure
        child.begin = begin
        animation = self.make(child=child)
        with self.assertRaises(RuntimeError) as caught:
            animation.begin()
        self.assertIs(caught.exception, failure)
        self.assertNotIn(adapter._KEY, vars(self.scene))
        self.make().begin()

    def test_callback_failure_and_abort_release_scope(self):
        animation = self.make()
        animation.begin()
        animation.anim.failure = RuntimeError("interpolation")
        with self.assertRaises(RuntimeError):
            animation.interpolate(0.5)
        self.assertNotIn(adapter._KEY, vars(self.scene))
        self.mob.tick(0.25)
        self.assertEqual(self.deltas, [0.25])

    def test_failed_finish_releases_scope(self):
        animation = self.make()
        animation.begin()
        def fail():
            raise RuntimeError("finish")
        animation.anim.finish = fail
        with self.assertRaisesRegex(RuntimeError, "finish"):
            animation.finish()
        self.assertNotIn(adapter._KEY, vars(self.scene))

    def test_reuse_resets_delta_baseline(self):
        animation = self.make()
        for _ in range(2):
            animation.begin()
            animation.interpolate(0.5)
            self.mob.tick(0.25)
            animation.finish()
            animation.clean_up_from_scene(self.scene)
        self.assertEqual(self.deltas, [0.5, 0.5])

    def test_one_argument_updater_retains_identity(self):
        function = lambda mob: None
        self.g.ChangeSpeed.add_updater(self.mob, function, index=0)
        self.assertIs(self.mob.updaters[0], function)

    def test_invalid_updater_and_configuration(self):
        for mob, function in ((object(), lambda mob: None), (self.mob, None),
                              (self.mob, lambda: None), (self.mob, lambda mob, *, dt: None)):
            with self.subTest(function=function), self.assertRaises(TypeError):
                self.g.ChangeSpeed.add_updater(mob, function)
        with self.assertRaises(TypeError):
            self.make(affects_speed_updaters=1)

    def test_install_once(self):
        function = self.g.ChangeSpeed.begin
        adapter.install_speed_updaters(self.g)
        self.assertIs(function, self.g.ChangeSpeed.begin)


@unittest.skipUnless(os.environ.get("FMN_TEST_NATIVE") == "1", "requires installed native portal; set FMN_TEST_NATIVE=1")
class NativeSpeedUpdaterTests(unittest.TestCase):
    def test_native_child_clock_drives_updater_and_then_restores_dt(self):
        import manimlib as m
        from manimlib.animation.speed import ChangeSpeed
        scene, anchor, following = m.Scene(), m.Dot(), m.Dot()
        scene.add(anchor, following)
        values = []
        ChangeSpeed.add_updater(following, lambda mob, dt: values.append(dt))
        scene.play(ChangeSpeed(m.Animation(anchor, run_time=2), {0: 2, 1: 2}))
        self.assertAlmostEqual(sum(values), 2.0, places=6)
        self.assertNotIn(adapter._KEY, vars(scene))
        before = sum(values)
        scene.wait(0.5)
        self.assertAlmostEqual(sum(values) - before, 0.5, places=6)

    def test_native_updater_failure_does_not_poison_next_play(self):
        import manimlib as m
        from manimlib.animation.speed import ChangeSpeed
        scene, anchor = m.Scene(), m.Dot()
        scene.add(anchor)
        failure = RuntimeError("authored speed updater")
        def failing(mob, dt):
            if dt:
                raise failure
        ChangeSpeed.add_updater(anchor, failing)
        with self.assertRaises(RuntimeError) as caught:
            scene.play(ChangeSpeed(m.Animation(anchor), {0: 2, 1: 2}))
        self.assertIs(caught.exception, failure)
        self.assertNotIn(adapter._KEY, vars(scene))
        anchor.clear_updaters()
        scene.play(ChangeSpeed(m.Animation(anchor), {0: 2, 1: 2}))


if __name__ == "__main__":
    unittest.main()
