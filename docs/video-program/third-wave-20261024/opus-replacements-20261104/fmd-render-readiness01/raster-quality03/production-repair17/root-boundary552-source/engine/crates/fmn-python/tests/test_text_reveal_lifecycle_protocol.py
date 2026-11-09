"""Real Animation/execution-owner code over explicit native-record fixtures."""
import ast
import copy
import importlib.util
import re
import types
import unittest

import numpy as np

from creation_protocol_support import BOOTSTRAP, environment as animation_environment
from test_text_reveal_protocol import MODULE, reveal, names

spec = importlib.util.spec_from_file_location("_reveal_execution", MODULE.with_name("scene_execution.py"))
execution = importlib.util.module_from_spec(spec)
spec.loader.exec_module(execution)


def environment():
    g = animation_environment()

    class Glyph(g["Mobject"]):
        def __init__(self, name, *children):
            super().__init__(*children, points=[])
            self.name, self.parents, self.resume_calls = name, [], []
            self._is_animating = False
            self.fail = None
            for child in children:
                child.parents.append(self)

        def set_submobjects(self, children):
            for child in self.submobjects:
                child.parents[:] = [parent for parent in child.parents if parent is not self]
            self.submobjects[:] = children
            for child in children:
                if all(parent is not self for parent in child.parents):
                    child.parents.append(self)
            if self.fail is not None:
                error, self.fail = self.fail, None
                raise error
            return self

        def suspend_updating(self, recurse=True):
            for node in self.get_family() if recurse else [self]:
                node.suspended = True
            return self

        def resume_updating(self, recurse=True, call_updater=True):
            self.resume_calls.append((recurse, call_updater))
            for node in self.get_family() if recurse else [self]:
                node.suspended = False
            if call_updater:
                self.update(0)
            return self

        def update(self, dt):
            for node in self.get_family():
                if not node.suspended:
                    for updater in node.updaters:
                        updater(node, dt)

        def set_animating_status(self, value):
            for node, _, _ in reveal._transients(self):
                node._is_animating = value
            return self

        def copy(self):
            # Detached native copies do not own a copy of the live ancestors.
            return copy.deepcopy(self, {id(self.parents): [], id(self._scene): self._scene})

    g.update(StringMobject=Glyph, _re=re, _FMN_ROOT=types.SimpleNamespace(linear=lambda t: t))
    wanted = {"_string_word_groups", "_string_glyph_reveal_plan", "_apply_glyph_reveal",
              "AddTextWordByWord", "AddTextLetterByLetter"}
    nodes = [node for node in ast.parse(BOOTSTRAP.read_text()).body
             if isinstance(node, (ast.ClassDef, ast.FunctionDef)) and node.name in wanted]
    exec(compile(ast.Module(nodes, []), str(BOOTSTRAP), "exec"), g)
    class Namespace:
        @property
        def __dict__(self):
            return g
    reveal.install_text_reveal(Namespace())
    return g, Glyph


class LifecycleTests(unittest.TestCase):
    def setUp(self):
        self.g, self.Glyph = environment()
        self.a, self.b = self.Glyph("a"), self.Glyph("b")
        self.part = self.Glyph("part", self.a, self.b)
        self.root = self.Glyph("root", self.part)
        self.root.string = "a b"
        self.root._byte_span = lambda span: span
        self.root._string_sub_spans = [(0, 1), (2, 3)]
        self.root._string_sub_paths = [(0, 0), (0, 1)]
        self.family = tuple(self.root.get_family())

    def animation(self, name="AddTextWordByWord", **kwargs):
        return self.g[name](self.root, **kwargs)

    def assert_original_tree(self):
        self.assertEqual(names(self.root), ["part"])
        self.assertEqual(names(self.part), ["a", "b"])
        self.assertIs(self.part.submobjects[0], self.a)
        self.assertIs(self.part.submobjects[1], self.b)

    def assert_released(self):
        self.assertFalse(any(node._is_animating for node in self.family))
        self.assertFalse(any(node.suspended for node in self.family))

    def test_abort_restores_hidden_families_and_releases_detached_glyphs(self):
        for kind in ("AddTextWordByWord", "AddTextLetterByLetter"):
            animation = self.animation(kind, suspend_mobject_updating=True)
            animation.begin()
            animation.interpolate(.5)
            self.assertEqual(names(self.part), ["a"])
            self.assertTrue(self.b.suspended)
            animation.abort()
            self.assert_original_tree()
            self.assert_released()
            calls = [list(node.resume_calls) for node in self.family]
            animation.abort()
            self.assertEqual([node.resume_calls for node in self.family], calls)
            self.assertTrue(all(call == (False, False) for row in calls for call in row))

    def test_partial_final_alpha_releases_even_hidden_glyphs_without_revealing_them(self):
        animation = self.animation(final_alpha_value=.5, suspend_mobject_updating=True)
        animation.begin()
        animation.finish()
        self.assertEqual(names(self.part), ["a"])
        self.assert_released()
        animation.abort()
        self.assertEqual(names(self.part), ["a"])
        animation.begin()
        animation.final_alpha_value = 1
        animation.finish()
        self.assert_original_tree()

    def test_preexisting_suspension_and_outside_ancestor_state_are_preserved(self):
        parent = self.Glyph("ancestor", self.root)
        parent._is_animating = True
        self.b.suspend_updating()
        calls = []
        self.b.updaters.append(lambda mob, dt: calls.append(dt))
        animation = self.animation(suspend_mobject_updating=True)
        animation.begin()
        animation.finish()
        self.assert_original_tree()
        self.assertTrue(parent._is_animating)
        self.assertTrue(self.b.suspended)
        self.assertFalse(self.a.suspended)
        self.assertFalse(self.b._is_animating)
        self.assertEqual(calls, [])

    def test_absent_animating_projection_is_absent_again_after_abort(self):
        del self.b._is_animating
        animation = self.animation()
        animation.begin()
        animation.abort()
        self.assertNotIn("_is_animating", vars(self.b))

    def test_success_calls_resumed_updaters_once_but_abort_does_not(self):
        calls = []
        self.a.updaters.append(lambda mob, dt: calls.append(dt))
        animation = self.animation(suspend_mobject_updating=True)
        animation.begin()
        animation.finish()
        self.assertEqual(calls, [0])
        animation.begin()
        animation.abort()
        self.assertEqual(calls, [0])

    def test_begin_copy_failure_preserves_primary_and_prior_tree(self):
        animation = self.animation(suspend_mobject_updating=True)
        error = LookupError("authored copy")
        def fail_copy():
            raise error
        animation.create_starting_mobject = fail_copy
        with self.assertRaises(LookupError) as result:
            animation.begin()
        self.assertIs(result.exception, error)
        self.assert_original_tree()
        self.assert_released()

    def test_frame_setter_failure_aborts_whole_animation_not_only_the_frame(self):
        animation = self.animation(suspend_mobject_updating=True)
        animation.begin()
        animation.interpolate(.5)
        error = LookupError("authored setter")
        self.part.fail = error
        with self.assertRaises(LookupError) as result:
            animation.interpolate(1)
        self.assertIs(result.exception, error)
        self.assert_original_tree()
        self.assert_released()

    def test_helper_failure_aborts_and_preserves_exception_identity(self):
        animation = self.animation(suspend_mobject_updating=True)
        animation.begin()
        error = LookupError("helper")
        def fail_update(dt):
            raise error
        animation.starting_mobject.update = fail_update
        with self.assertRaises(LookupError) as result:
            animation.update_mobjects(.1)
        self.assertIs(result.exception, error)
        self.assert_original_tree()
        self.assert_released()

    def test_scene_execution_owner_aborts_reveal_after_unrelated_native_failure(self):
        animation = self.animation(suspend_mobject_updating=True)
        owner = execution._Execution(self.g)
        handle, = owner.wrap([animation])
        handle.begin()
        handle.interpolate(.5)
        owner.unwind(RuntimeError("native scene or updater failure"))
        self.assert_original_tree()
        self.assert_released()

    def test_cleanup_failure_is_not_allowed_to_replace_the_primary_exception(self):
        animation = self.animation(suspend_mobject_updating=True)
        animation.begin()
        primary = LookupError("rate callback failed")
        self.root.fail = RuntimeError("restore also failed")
        def bad_rate(alpha):
            raise primary
        animation.rate_func = bad_rate
        with self.assertRaises(LookupError) as result:
            animation.interpolate(.5)
        self.assertIs(result.exception, primary)
        self.assertIn("text reveal rollback also failed: RuntimeError", primary.__notes__)
        self.assert_released()
        animation.abort()
        self.assert_original_tree()

    def test_flat_paths_obey_source_order_not_sibling_index(self):
        self.root.set_submobjects([self.b, self.a])
        self.root._string_sub_paths = [(1,), (0,)]
        for kind in ("AddTextWordByWord", "AddTextLetterByLetter"):
            animation = self.animation(kind)
            animation.begin()
            animation.interpolate(.5)
            self.assertEqual(names(self.root), ["a"])
            animation.finish()
            self.assertEqual(names(self.root), ["b", "a"])

    def test_begin_uses_regrouped_live_spans_and_current_parent_records(self):
        animation = self.animation()
        inner = self.Glyph("inner", self.a, self.b)
        self.part.set_submobjects([inner])
        self.root._string_sub_paths = [(0, 0, 0), (0, 0, 1)]
        animation.begin()
        animation.interpolate(.5)
        self.assertIs(self.root.submobjects[0].submobjects[0].submobjects[0], self.a)
        animation.abort()
        self.assertEqual(names(inner), ["a", "b"])

    def test_rate_sees_raw_alpha_exactly_once_and_rounds_ties_to_even(self):
        samples = []
        animation = self.animation(run_time=4, time_span=(1, 3),
                                   rate_func=lambda alpha: samples.append(alpha) or alpha)
        animation.begin()
        samples.clear()
        animation.interpolate(.25)
        self.assertEqual(names(self.root), [])
        animation.interpolate(.75)
        self.assert_original_tree()
        self.assertEqual(samples, [.25, .75])
        animation.abort()

    def test_nonfinite_rate_or_alpha_refuses_without_leaking_hidden_state(self):
        for value in (float("nan"), float("inf"), -float("inf")):
            for bad_rate in (False, True):
                with self.subTest(value=value, bad_rate=bad_rate):
                    animation = self.animation(suspend_mobject_updating=True)
                    animation.begin()
                    if bad_rate:
                        animation.rate_func = lambda alpha: value
                    with self.assertRaisesRegex(ValueError, "finite"):
                        animation.interpolate(.5 if bad_rate else value)
                    self.assert_original_tree()
                    self.assert_released()

    def test_duration_rate_validation_and_public_defaults(self):
        self.assertEqual(self.animation().run_time, .4)
        self.assertEqual(self.animation("AddTextLetterByLetter").run_time, .2)
        for kwargs in ({"run_time": float("nan")}, {"run_time": float("inf")},
                       {"time_per_word": -1}, {"rate_func": 7}):
            with self.subTest(kwargs=kwargs):
                with self.assertRaises((ValueError, TypeError)):
                    self.animation(**kwargs)
                self.assert_original_tree()


if __name__ == "__main__":
    unittest.main()
