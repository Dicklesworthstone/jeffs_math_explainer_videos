"""Public Scene hooks over the production adapter and explicit native-loop fixture.

These tests verify orchestration, not native clock/rendering. Installed-wheel
acceptance separately exercises the bootstrap's real hooks and native state.
"""
from __future__ import annotations

import unittest
from test_scene_execution_protocol import environment, execution


class HookTests(unittest.TestCase):
    def setUp(self):
        self.g = environment()
        self.scene = self.g.Scene()
        self.mob = self.g.Mobject()
        self.events = []
        self.scene.pre_play = lambda: self.events.append("pre")
        def post():
            self.events.append("post")
            self.scene.num_plays += 1
        self.scene.post_play = post

    def anim(self, **kwargs):
        return self.g.Animation(self.mob, **kwargs)

    def test_prepare_rate_info_and_hooks_surround_actual_animation_lifecycle(self):
        anim = self.anim()
        prepare = self.g.prepare_animation
        def prepare_logged(item):
            self.events.append("prepare")
            return prepare(item)
        self.g.prepare_animation = prepare_logged
        anim.update_rate_info = lambda **kw: self.events.append(("rate-info", kw))
        anim.event = self.events.append
        self.assertEqual(self.scene.play(anim, run_time=.25, lag_ratio=0), [.25])
        self.assertEqual(self.events, [
            "prepare", ("rate-info", {"run_time": .25, "lag_ratio": 0.0}), "pre",
            "begin", ("helpers", .25), "interpolate", "finish", "cleanup", "post",
        ])
        self.assertEqual(self.scene.num_plays, 1)

    def test_builder_is_resolved_once_before_pre_hook_edits_product(self):
        builder, anim = self.g._AnimationBuilder(self.mob), self.anim()
        calls = []
        def prepare(item):
            calls.append(item)
            if item is not builder:
                self.fail("builder product reconstructed")
            return anim
        self.g.prepare_animation = prepare
        self.scene.pre_play = lambda: setattr(anim, "on_interpolate", lambda: self.events.append("edited"))
        self.scene.play(builder)
        self.assertEqual(calls, [builder])
        self.assertIs(self.scene._last_specs[0], anim)
        self.assertEqual(self.events, ["edited", "post"])

    def test_live_base_hook_changes_and_cooperative_subclasses_are_observed(self):
        events, Base = self.events, self.g.Scene
        class Authored(Base):
            def pre_play(self):
                events.append("authored-pre")
                super().pre_play()
            def post_play(self):
                events.append("authored-post")
                super().post_play()
        scene = Authored()
        Base.pre_play = lambda self: events.append("patched-base")
        scene.play(self.anim())
        self.assertEqual(events, ["authored-pre", "patched-base", "authored-post"])
        self.assertEqual(scene.num_plays, 1)

    def test_failed_execution_does_not_post_or_increment_count(self):
        with self.assertRaises(LookupError):
            self.scene.play(self.anim(fail="interpolate"))
        self.assertEqual(self.events, ["pre"])
        self.assertEqual(self.scene.num_plays, 0)

    def test_failed_pre_hook_never_begins_animation_or_runs_post(self):
        error, anim = LookupError("pre refused"), self.anim()
        def pre():
            raise error
        self.scene.pre_play = pre
        with self.assertRaises(LookupError) as result:
            self.scene.play(anim)
        self.assertIs(result.exception, error)
        self.assertEqual(self.scene.events, [])
        self.assertEqual(anim.events, [])
        self.assertEqual(self.events, [])
        self.assertNotIn(execution._OWNER_KEY, vars(self.scene))

    def test_post_hook_failure_is_not_retried_or_turned_into_animation_abort(self):
        anim, error = self.anim(), LookupError("post refused")
        anim.abort = lambda: self.fail("completed animation aborted")
        def post():
            self.events.append("failed-post")
            raise error
        self.scene.post_play = post
        with self.assertRaises(LookupError) as result:
            self.scene.play(anim)
        self.assertIs(result.exception, error)
        self.assertEqual(self.events, ["pre", "failed-post"])
        self.assertEqual(anim.events.count("finish"), 1)
        self.assertEqual(anim.events.count("cleanup"), 1)
        self.assertFalse(self.mob.suspended)

    def test_empty_play_is_not_a_segment(self):
        self.assertIsNone(self.scene.play())
        self.assertEqual(self.events, [])
        self.assertEqual(self.scene.events, [])
        self.assertEqual(self.scene.num_plays, 0)

    def test_wait_and_early_condition_each_count_one_completed_segment(self):
        predicate_calls = []
        self.scene.play(self.anim())
        self.scene.wait(.1, stop_condition=lambda: predicate_calls.append(1) or True)
        self.assertEqual(predicate_calls, [1])
        self.assertEqual(self.scene.num_plays, 2)
        self.assertEqual(self.events, ["pre", "post", "pre", "post"])

    def test_exclusive_end_gate_can_stop_play_and_wait_before_backend_work(self):
        class EndScene(Exception):
            pass
        def pre():
            if self.scene.num_plays >= 2:
                raise EndScene("end reached")
        self.scene.pre_play = pre
        self.scene.play(self.anim())
        self.scene.wait(.1)
        before = list(self.scene.events)
        for operation in (lambda: self.scene.play(self.anim()), lambda: self.scene.wait(.1)):
            with self.assertRaises(EndScene):
                operation()
        self.assertEqual(self.scene.events, before)
        self.assertEqual(self.scene.num_plays, 2)

    def test_invalid_play_options_and_animation_types_are_rejected_before_hooks(self):
        cases = [((), {"unknown": 1}), ((object(),), {}),
                 ((self.anim(),), {"run_time": -1}), ((self.anim(),), {"run_time": float("nan")}),
                 ((self.anim(),), {"lag_ratio": float("inf")}), ((self.anim(),), {"rate_func": 7})]
        for args, kwargs in cases:
            with self.subTest(kwargs=kwargs), self.assertRaises((TypeError, ValueError, NotImplementedError)):
                self.scene.play(*args, **kwargs)
        self.assertEqual(self.events, [])
        self.assertEqual(self.scene.events, [])

    def test_invalid_wait_arguments_are_rejected_before_hooks(self):
        for kwargs in ({"duration": -1}, {"duration": float("inf")}, {"stop_condition": 1}, {"unknown": 1}):
            with self.subTest(kwargs=kwargs), self.assertRaises((TypeError, ValueError, NotImplementedError)):
                self.scene.wait(**kwargs)
        self.assertEqual(self.events, [])
        self.assertEqual(self.scene.events, [])

    def test_internal_controller_keyword_names_are_not_public_options(self):
        for key in ("hooks", "prepare", "operation", "args", "kwargs"):
            with self.subTest(key=key), self.assertRaises(TypeError):
                self.scene.play(self.anim(), **{key: False})
            with self.subTest(key=key), self.assertRaises(NotImplementedError):
                self.scene.wait(**{key: False})
        self.assertEqual(self.events, [])

    def test_wait_default_and_note_options_reach_the_original_operation(self):
        g = environment(False)
        calls = []
        def original(self, *args, **kwargs):
            calls.append((args, kwargs))
            return "wait result"
        g.Scene.wait = original
        execution.install_scene_execution(g)
        scene = g.Scene()
        scene.default_wait_time = .75
        marker, condition = object(), lambda: True
        self.assertEqual(scene.wait(stop_condition=condition, note=marker, ignore_presenter_mode=True), "wait result")
        self.assertEqual(calls, [((.75, condition), {"note": marker, "ignore_presenter_mode": True})])
        self.assertEqual(scene.num_plays, 1)

    def test_private_native_entry_remains_hook_free(self):
        anim = self.anim()
        self.scene._play_animations([object()], [anim], None, None, None, None)
        self.assertEqual(self.events, [])
        self.assertEqual(self.scene.num_plays, 0)

    def test_invalid_builder_product_never_enters_pre_hook(self):
        self.g.prepare_animation = lambda item: None
        with self.assertRaisesRegex(TypeError, "must return an Animation"):
            self.scene.play(self.g._AnimationBuilder(self.mob))
        self.assertEqual(self.events, [])
        self.assertNotIn(execution._OWNER_KEY, vars(self.scene))

    def test_same_scene_hook_reentry_is_rejected_but_other_scene_is_allowed(self):
        self.scene.pre_play = lambda: self.scene.wait(.1)
        with self.assertRaisesRegex(RuntimeError, "another play/wait"):
            self.scene.play(self.anim())
        self.assertEqual(self.scene.events, [])
        other = self.g.Scene()
        self.scene.pre_play = lambda: other.wait(.1)
        self.scene.play(self.anim())
        self.assertEqual(other.num_plays, 1)
        self.assertEqual(self.scene.num_plays, 1)

    def test_authored_rate_info_failure_propagates_without_running_pre_hook(self):
        anim, error = self.anim(), ValueError("rate-info refused")
        def update(**kwargs):
            raise error
        anim.update_rate_info = update
        with self.assertRaises(ValueError) as result:
            self.scene.play(anim)
        self.assertIs(result.exception, error)
        self.assertEqual(self.events, [])
        self.assertEqual(self.scene.events, [])

    def test_wait_failure_preserves_exception_without_post_hook(self):
        error = KeyboardInterrupt()
        def predicate():
            raise error
        with self.assertRaises(KeyboardInterrupt) as result:
            self.scene.wait(.1, stop_condition=predicate)
        self.assertIs(result.exception, error)
        self.assertEqual(self.events, ["pre"])
        self.assertEqual(self.scene.num_plays, 0)
        self.assertNotIn(execution._OWNER_KEY, vars(self.scene))


if __name__ == "__main__":
    unittest.main()
