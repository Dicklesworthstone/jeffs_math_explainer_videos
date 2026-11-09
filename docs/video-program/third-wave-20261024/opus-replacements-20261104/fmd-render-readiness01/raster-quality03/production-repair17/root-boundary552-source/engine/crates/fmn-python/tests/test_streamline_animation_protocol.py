"""Real streamline controller over explicit storage/flash protocol doubles.

These tests verify dispatch and lifetime, not RK45, native geometry or pixels.
The installed-wheel suite exercises those separate integration boundaries.
"""
from __future__ import annotations

import importlib.util
from pathlib import Path
from types import SimpleNamespace
import unittest

SOURCE = Path(__file__).resolve().parents[1] / "python/fmn_python/streamline_animation.py"
spec = importlib.util.spec_from_file_location("streamline_animation_under_test", SOURCE)
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)


def environment():
    events = []

    class Mobject:
        def __init__(self, *children, **kwargs):
            self.submobjects, self.updaters = list(children), []
            self.suspended = False
            self.kwargs = kwargs
        def __iter__(self):
            return iter(self.submobjects)
        def get_family(self):
            found = []
            def visit(mob):
                if mob not in found:
                    found.append(mob)
                    for child in mob:
                        visit(child)
            visit(self)
            return found
        def update(self, dt=0, recurse=True):
            if self.suspended:
                return self
            if recurse:
                for child in list(self):
                    child.update(dt)
            for updater in tuple(self.updaters):
                updater(self, dt)
            return self
        def add_updater(self, updater, call=True):
            self.updaters.append(updater)
            if call:
                self.update(0)
            return self
        def remove_updater(self, updater):
            self.updaters[:] = [item for item in self.updaters if item is not updater]
            return self
        def clear_updaters(self, recurse=True):
            for member in self.get_family() if recurse else (self,):
                member.updaters.clear()
            return self

    class VGroup(Mobject):
        pass
    class StreamLines(VGroup):
        def __init__(self, times=(2., 4.)):
            super().__init__(*(Mobject() for _ in times))
            self._stream_virtual_times = list(times)
            self._stream_rng_draws = 17
    class AnimatedStreamLines(VGroup):
        def update(self, dt=0):
            raise AssertionError("old synthetic interpolation must not execute")
    class Animation:
        pass
    class Flash(Animation):
        constructions = 0
        def __init__(self, mobject, run_time, **kwargs):
            self.mobject, self.run_time, self.config = mobject, run_time, kwargs
            self.begun, self.aborts, self.alphas = False, 0, []
            self.index = Flash.constructions
            Flash.constructions += 1
            events.append(("construct", self.index))
        def begin(self):
            events.append(("begin", self.index))
            self.begun = True
            if getattr(self, "fail_begin", False):
                raise RuntimeError("begin failure")
        def get_run_time(self):
            return self.run_time
        def update_mobjects(self, dt):
            events.append(("helpers", self.index, dt))
            if callable(getattr(self, "helper", None)):
                self.helper()
        def interpolate(self, alpha):
            events.append(("interpolate", self.index, alpha))
            self.alphas.append(alpha)
            if callable(getattr(self, "hook", None)):
                self.hook()
            if getattr(self, "fail_tick", False):
                raise RuntimeError("tick failure")
        def abort(self):
            events.append(("abort", self.index))
            self.aborts += 1
            self.begun = False
            if getattr(self, "fail_abort", False):
                raise RuntimeError("abort failure")

    class Bridge:
        calls = []
        @staticmethod
        def _stream_line_lag_uniforms(seed, offset, count):
            Bridge.calls.append((seed, offset, count))
            return [0.25] * count
    def linear(alpha):
        return alpha
    native = SimpleNamespace(Mobject=Mobject, VGroup=VGroup, StreamLines=StreamLines,
                             AnimatedStreamLines=AnimatedStreamLines, Animation=Animation,
                             VShowPassingFlash=Flash, _BridgeMobject=Bridge, linear=linear)
    return native, events


class StreamlineProtocolTests(unittest.TestCase):
    def setUp(self):
        self.native, self.events = environment()
        self.original = self.native.AnimatedStreamLines
        module.install_streamline_animation(self.native)
    def make(self, **kwargs):
        lines = self.native.StreamLines()
        return lines, self.native.AnimatedStreamLines(lines, **kwargs)
    def test_actual_animation_objects_and_original_lines_are_published(self):
        lines, animated = self.make(lag_range=0, rate_multiple=2, color="white")
        self.assertIs(animated.stream_lines, lines)
        self.assertEqual(list(animated), list(lines))
        for line, duration in zip(lines, (1, 2)):
            self.assertIs(line.anim.mobject, line)
            self.assertEqual(line.anim.run_time, duration)
            self.assertTrue(line.anim.begun)
        self.assertEqual(animated.kwargs, {"color": "white"})
    def test_all_flashes_are_constructed_before_begin(self):
        self.make()
        self.assertEqual(self.events[:4], [("construct", 0), ("construct", 1), ("begin", 0), ("begin", 1)])
    def test_native_rng_offset_and_signed_phase_are_preserved(self):
        lines, animated = self.make()
        self.assertEqual(self.native._BridgeMobject.calls, [(0, 17, 2)])
        self.assertEqual([line.time for line in lines], [-1, -1])
        self.assertEqual([line.anim.alphas[-1] for line in lines], [.5, .75])
        animated.update(.5)
        self.assertEqual([line.anim.alphas[-1] for line in lines], [.75, .875])
    def test_arbitrary_rate_and_flash_options_forward_without_sampling(self):
        class Rate:
            __hash__ = None
            def __call__(self, alpha):
                raise AssertionError("controller must not pre-sample rates")
        rate = Rate()
        config = dict(rate_func=rate, taper_width=.2, time_width=.6, lag_ratio=.3,
                      time_span=(.2, 1), suspend_mobject_updating=True, remover=False)
        lines, _ = self.make(line_anim_config=config)
        self.assertEqual(lines.submobjects[0].anim.config, config)
        self.assertIs(lines.submobjects[0].anim.config["rate_func"], rate)
        self.assertNotIn("run_time", config)
    def test_additional_updaters_and_recurse_flag_work(self):
        lines, animated = self.make(lag_range=0)
        events = []
        lines.submobjects[0].add_updater(lambda _, dt: events.append(("child", dt)), call=False)
        animated.add_updater(lambda _, dt: events.append(("root", dt)), call=False)
        self.assertIs(animated.update(.25), animated)
        self.assertEqual(events, [("child", .25), ("root", .25)])
        events.clear()
        animated.update(.25, recurse=False)
        self.assertEqual(events, [("root", .25)])
    def test_suspension_freezes_phase_and_resume_continues(self):
        lines, animated = self.make(lag_range=0)
        animated.suspended = True
        animated.update(3)
        self.assertEqual([line.time for line in lines], [0, 0])
        animated.suspended = False
        animated.update(.5)
        self.assertEqual([line.time for line in lines], [.5, .5])
    def test_helper_update_precedes_each_interpolation(self):
        _, animated = self.make(lag_range=0)
        self.events.clear()
        animated.update(.5)
        self.assertEqual(self.events, [("helpers", 0, .5), ("interpolate", 0, .25),
                                       ("helpers", 1, .5), ("interpolate", 1, .125)])
    def test_runtime_and_public_time_edits_are_live(self):
        lines, animated = self.make(lag_range=0)
        first = lines.submobjects[0]
        first.time, first.anim.run_time = 4.5, 3
        animated.update(.75)
        self.assertEqual(first.time, 5.25)
        self.assertEqual(first.anim.alphas[-1], .75)
    def test_zero_virtual_time_is_safe(self):
        lines = self.native.StreamLines((0, 2))
        animated = self.native.AnimatedStreamLines(lines)
        animated.update(100)
        self.assertEqual(lines.submobjects[0].anim.alphas[-1], 0)
    def test_empty_stream_is_valid(self):
        lines = self.native.StreamLines(())
        animated = self.native.AnimatedStreamLines(lines)
        animated.update(2).clear_updaters()
        self.assertEqual(list(animated), [])
    def test_copy_callback_cannot_drive_or_cancel_its_source(self):
        lines, animated = self.make(lag_range=0)
        copied = self.native.VGroup()
        copied.updaters = list(animated.updaters)
        copied.update(1)
        copied.clear_updaters()
        self.assertEqual([line.time for line in lines], [0, 0])
        self.assertFalse(animated._streamline_controller.closed)
    def test_remove_updater_aborts_all_owned_flashes_once(self):
        lines, animated = self.make()
        updater = animated._streamline_controller.update
        animated.remove_updater(updater)
        animated.remove_updater(updater)
        self.assertFalse(animated.updaters)
        self.assertEqual([line.anim.aborts for line in lines], [1, 1])
    def test_recursive_parent_clear_aborts_descendant_stream(self):
        lines, animated = self.make()
        self.native.VGroup(animated).clear_updaters()
        self.assertTrue(animated._streamline_controller.closed)
        self.assertEqual([line.anim.aborts for line in lines], [1, 1])
    def test_nonrecursive_parent_clear_leaves_descendant_active(self):
        _, animated = self.make()
        self.native.VGroup(animated).clear_updaters(recurse=False)
        self.assertFalse(animated._streamline_controller.closed)
    def test_tick_failure_cancels_every_line_and_preserves_original(self):
        lines, animated = self.make()
        lines.submobjects[0].anim.fail_tick = True
        lines.submobjects[1].anim.fail_abort = True
        with self.assertRaisesRegex(RuntimeError, "tick failure") as caught:
            animated.update(.25)
        self.assertIn("cleanup failed", caught.exception.__notes__[0])
        self.assertFalse(animated.updaters)
        self.assertEqual([line.anim.aborts for line in lines], [1, 1])
    def test_partial_begin_failure_aborts_started_flashes(self):
        original = self.native.VShowPassingFlash
        class Bad(original):
            def begin(self):
                self.fail_begin = self.index == 1
                super().begin()
        self.native.VShowPassingFlash = Bad
        lines = self.native.StreamLines()
        with self.assertRaisesRegex(RuntimeError, "begin failure"):
            self.native.AnimatedStreamLines(lines)
        self.assertEqual([line.anim.aborts for line in lines], [1, 1])
    def test_reentrant_cancellation_defers_release_until_callback_returns(self):
        lines, animated = self.make()
        first = lines.submobjects[0].anim
        def helper():
            animated.clear_updaters()
            self.assertTrue(first.begun)
        first.helper = helper
        first.alphas.clear()
        animated.update(.5)
        self.assertFalse(first.begun)
        self.assertEqual(first.alphas, [])
    def test_begin_cancellation_does_not_start_later_flashes(self):
        _, animated = self.make()
        owner = animated._streamline_controller
        owner.cancel()
        animations = [self.native.VShowPassingFlash(self.native.Mobject(), 1)
                      for _ in range(2)]
        owner = module._StreamlineController(animated, [], animations)
        def begin():
            owner.cancel()
            self.assertEqual(animations[0].aborts, 0)
        animations[0].begin = begin
        owner.start()
        self.assertEqual([animation.aborts for animation in animations], [1, 0])
        self.assertFalse(animations[1].begun)
        self.assertNotIn(owner.update, animated.updaters)
    def test_invalid_durations_do_not_advance_any_counter(self):
        lines, animated = self.make()
        before = [line.time for line in lines]
        lines.submobjects[1].anim.run_time = float("nan")
        with self.assertRaises(ValueError):
            animated.update(.5)
        self.assertEqual([line.time for line in lines], before)
    def test_invalid_config_fails_before_animation_creation(self):
        for kwargs in ({"rate_multiple": 0}, {"rate_multiple": float("inf")},
                       {"lag_range": -1}, {"line_anim_config": []},
                       {"line_anim_config": {"run_time": 2}}):
            with self.assertRaises((ValueError, TypeError)):
                self.make(**kwargs)
        self.assertEqual(self.native.VShowPassingFlash.constructions, 0)
    def test_metadata_cardinality_error_precedes_begin(self):
        lines = self.native.StreamLines()
        lines._stream_virtual_times.pop()
        with self.assertRaisesRegex(ValueError, "metadata"):
            self.native.AnimatedStreamLines(lines)
        self.assertEqual(self.events, [])
    def test_installer_is_idempotent_and_class_identity_is_preserved(self):
        before = self.original.__init__
        module.install_streamline_animation(self.native)
        self.assertIs(self.original, self.native.AnimatedStreamLines)
        self.assertIs(before, self.original.__init__)
        self.assertEqual(before.__qualname__, self.original.__qualname__ + ".__init__")
    def test_inherited_update_dispatch_remains_live(self):
        _, animated = self.make()
        calls = []
        original = self.native.VGroup.update
        def override(self, dt=0, recurse=True):
            calls.append(dt)
            return original(self, dt, recurse)
        self.native.VGroup.update = override
        animated.update(.25)
        self.assertEqual(calls, [.25])


if __name__ == "__main__":
    unittest.main()
