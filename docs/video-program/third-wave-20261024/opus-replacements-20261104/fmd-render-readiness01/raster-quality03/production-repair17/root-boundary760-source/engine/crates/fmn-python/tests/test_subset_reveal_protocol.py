"""Production selection/lifecycle code over explicit native-object fixtures.

No fixture storage or Python clock here is claimed as native pixel evidence.
The companion subset_reveal.py exercises the actual installed extension.
"""
import ast
import math
import types
import unittest

import numpy as np

from creation_protocol_support import BOOTSTRAP
from test_text_reveal_lifecycle_protocol import environment as text_environment
from fmn_python.subset_reveal import install_subset_reveal


def environment():
    g, Node = text_environment()
    names = {"ShowIncreasingSubsets", "ShowSubmobjectsOneByOne"}
    nodes = [node for node in ast.parse(BOOTSTRAP.read_text()).body
             if isinstance(node, ast.ClassDef) and node.name in names]
    exec(compile(ast.Module(nodes, []), str(BOOTSTRAP), "exec"), g)
    class Namespace:
        @property
        def __dict__(self):
            return g
    native = Namespace()
    install_subset_reveal(native)
    return g, Node, native


class SubsetTests(unittest.TestCase):
    def setUp(self):
        self.g, self.Node, self.native = environment()
        self.children = [self.Node(str(i)) for i in range(4)]
        self.root = self.Node("root", *self.children)
        self.cls = self.g["ShowIncreasingSubsets"]

    def animation(self, **kwargs):
        kwargs.setdefault("rate_func", lambda t: t)
        return self.cls(self.root, **kwargs)

    def assert_restored(self):
        self.assertEqual(list(self.root.submobjects), self.children)
        for child in self.children:
            self.assertEqual(child.parents, [self.root])

    def assert_released(self):
        for child in [self.root, *self.children]:
            self.assertFalse(child._is_updating_suspended())
            self.assertFalse(child._is_animating)

    def test_arbitrary_callable_is_not_probed_and_receives_rated_child_count(self):
        calls = []
        def rate(t):
            calls.append(("rate", t))
            return .5 * t
        def select(count):
            calls.append(("select", count))
            return math.ceil(count)
        animation = self.animation(rate_func=rate, int_func=select)
        self.assertEqual(calls, [])
        self.assertIs(animation.int_func, select)
        self.assertEqual(animation._native_params(), {})
        animation.interpolate(.6)
        self.assertEqual(calls, [("rate", .6), ("select", 1.2)])
        self.assertEqual(self.root.submobjects, self.children[:2])

    def test_callable_object_and_live_function_reassignment(self):
        class Selector:
            def __init__(self):
                self.count = 0
            def __call__(self, value):
                self.count += 1
                return self.count
        selector = Selector()
        animation = self.animation(int_func=selector)
        animation.interpolate(.5)
        self.assertEqual(self.root.submobjects, self.children[:1])
        animation.interpolate(.5)
        self.assertEqual(self.root.submobjects, self.children[:2])
        animation.int_func = lambda _: 3
        animation.interpolate(.5)
        self.assertEqual(self.root.submobjects, self.children[:3])
        self.assertEqual(selector.count, 2)

    def test_public_all_submobs_can_be_reordered_without_copying(self):
        animation = self.animation(int_func=math.floor)
        animation.all_submobs.reverse()
        animation.interpolate(.5)
        self.assertEqual(self.root.submobjects, self.children[::-1][:2])
        self.assertIs(self.root.submobjects[0], self.children[3])

    def test_selection_freezes_construction_time_children_not_begin_time(self):
        animation = self.animation()
        added = self.Node("later")
        self.root.set_submobjects([*self.children, added])
        animation.begin()
        animation.finish()
        self.assert_restored()
        self.assertNotIn(added, self.root.submobjects)
        self.assertFalse(added._is_animating)
        self.assertFalse(added.suspended)

    def test_rate_receives_raw_alpha_despite_time_span(self):
        animation = self.animation(int_func=math.floor, time_span=(2, 4), run_time=4)
        animation.interpolate(.5)
        self.assertEqual(self.root.submobjects, self.children[:2])

    def test_round_ceil_and_python_int_conversion_agree_with_reference(self):
        for function in (np.round, np.ceil, np.floor, abs, lambda value: value - .9):
            animation = self.animation(int_func=function)
            for alpha in (-2., -.1, 0., .125, .375, .5, .9, 1., 2.):
                animation.interpolate(alpha)
                index = int(function(alpha * len(self.children)))
                self.assertEqual(self.root.submobjects, self.children[:index])
            self.root.set_submobjects(self.children)

    def test_stock_native_diagnostic_rules_remain_accurate(self):
        animation = self.animation()
        self.assertIs(animation.int_func, np.round)
        self.assertEqual(animation._native_params(), {"int_round": "round"})
        animation.int_func = np.ceil
        self.assertEqual(animation._native_params(), {"int_round": "ceil"})
        animation.int_func = lambda value: np.ceil(value)
        self.assertEqual(animation._native_params(), {})

    def test_empty_group_remains_a_valid_empty_reveal(self):
        for name in ("ShowIncreasingSubsets", "ShowSubmobjectsOneByOne"):
            root = self.Node("empty")
            calls = []
            animation = self.g[name](root, int_func=lambda n: calls.append(n) or 0)
            animation.begin()
            animation.finish()
            self.assertEqual(root.submobjects, [])
            self.assertEqual(calls, [0., 0.])
            self.assertFalse(root._is_animating)

    def test_one_by_one_preserves_pinned_endpoint_convention(self):
        animation = self.g["ShowSubmobjectsOneByOne"](self.root, rate_func=lambda t: t)
        self.assertIs(animation.int_func, np.ceil)
        for index in (-5, 0, 1, 2, 3, 4, 100):
            animation.update_submobject_list(index)
            clipped = min(max(index, 0), len(self.children) - 1)
            desired = [] if clipped == 0 else [self.children[clipped - 1]]
            self.assertEqual(self.root.submobjects, desired)

    def test_authored_list_hook_is_invoked_even_when_selection_is_unchanged(self):
        calls = []
        class Authored(self.cls):
            def update_submobject_list(self, index):
                calls.append(index)
                return super().update_submobject_list(index)
        animation = Authored(self.root, rate_func=lambda t: t, int_func=lambda _: 2)
        animation.interpolate(.3)
        animation.interpolate(.3)
        self.assertEqual(calls, [2, 2])

    def test_non_callable_is_rejected_before_mutating_group(self):
        for invalid in (0, "round", object()):
            with self.assertRaisesRegex(TypeError, "int_func must be callable"):
                self.animation(int_func=invalid)
            self.assert_restored()
        with self.assertRaisesRegex(TypeError, "requires a Mobject family"):
            self.cls(object())

    def test_begin_partial_finish_and_replay_release_all_hidden_candidates(self):
        animation = self.animation(final_alpha_value=.5, suspend_mobject_updating=True)
        animation.begin()
        self.assertEqual(self.root.submobjects, [])
        self.assertTrue(all(child.suspended for child in self.children))
        animation.finish()
        self.assertEqual(self.root.submobjects, self.children[:2])
        self.assert_released()
        animation.abort()
        self.assertEqual(self.root.submobjects, self.children[:2])
        animation.begin()
        animation.final_alpha_value = 1
        animation.finish()
        self.assert_restored()
        self.assert_released()

    def test_one_by_one_finish_releases_non_visible_members(self):
        animation = self.g["ShowSubmobjectsOneByOne"](
            self.root, suspend_mobject_updating=True)
        animation.begin()
        animation.finish()
        self.assertEqual(self.root.submobjects, [self.children[-2]])
        self.assert_released()

    def test_abort_restores_entry_topology_and_is_idempotent(self):
        animation = self.animation(suspend_mobject_updating=True)
        self.root.set_submobjects(self.children[1:])
        animation.begin()
        animation.interpolate(.5)
        animation.abort()
        self.assertEqual(self.root.submobjects, self.children[1:])
        self.assert_released()
        calls = [list(node.resume_calls) for node in self.children]
        animation.abort()
        self.assertEqual(calls, [node.resume_calls for node in self.children])

    def test_prior_suspension_and_ancestor_flags_survive_success(self):
        ancestor = self.Node("ancestor", self.root)
        ancestor._is_animating = True
        child = self.children[-1]
        child.suspend_updating()
        del self.children[0]._is_animating
        animation = self.animation(final_alpha_value=.5, suspend_mobject_updating=True)
        animation.begin()
        animation.finish()
        self.assertTrue(ancestor._is_animating)
        self.assertTrue(child.suspended)
        self.assertNotIn("_is_animating", vars(self.children[0]))
        self.assertFalse(self.children[1].suspended)

    def test_success_runs_resumed_visible_updaters_once_abort_runs_none(self):
        calls = []
        self.children[0].updaters.append(lambda node, dt: calls.append(dt))
        animation = self.animation(suspend_mobject_updating=True)
        animation.begin()
        animation.finish()
        self.assertEqual(calls, [0])
        animation.begin()
        animation.abort()
        self.assertEqual(calls, [0])

    def test_selector_exception_restores_source_and_preserves_exact_error(self):
        failure = LookupError("authored selector")
        def select(value):
            if value > 0:
                raise failure
            return 0
        animation = self.animation(int_func=select, suspend_mobject_updating=True)
        animation.begin()
        with self.assertRaises(LookupError) as caught:
            animation.interpolate(.5)
        self.assertIs(caught.exception, failure)
        self.assert_restored()
        self.assert_released()

    def test_authored_interpolate_override_failure_is_unwound(self):
        failure = RuntimeError("authored interpolate")
        class Authored(self.cls):
            def interpolate_mobject(self, alpha):
                super().interpolate_mobject(alpha)
                if alpha > 0:
                    raise failure
        animation = Authored(self.root, suspend_mobject_updating=True)
        animation.begin()
        with self.assertRaises(RuntimeError) as caught:
            animation.interpolate(.5)
        self.assertIs(caught.exception, failure)
        self.assert_restored()
        self.assert_released()

    def test_failed_native_setter_rolls_back_even_after_mutation(self):
        animation = self.animation(suspend_mobject_updating=True)
        animation.begin()
        failure = RuntimeError("native child splice")
        self.root.fail = failure
        with self.assertRaises(RuntimeError) as caught:
            animation.interpolate(.5)
        self.assertIs(caught.exception, failure)
        self.assert_restored()
        self.assert_released()

    def test_invalid_live_callback_result_releases_state(self):
        for invalid in (float("nan"), float("inf"), object()):
            animation = self.animation(suspend_mobject_updating=True)
            animation.begin()
            animation.int_func = lambda _: invalid
            with self.assertRaises((ValueError, OverflowError, TypeError)):
                animation.finish()
            self.assert_restored()
            self.assert_released()

    def test_invalid_alpha_releases_state_without_calling_selector(self):
        for invalid in (float("nan"), float("inf")):
            calls = []
            animation = self.animation(int_func=lambda n: calls.append(n) or 0)
            animation.begin()
            with self.assertRaisesRegex(ValueError, "alpha must be finite"):
                animation.interpolate(invalid)
            self.assertEqual(calls, [0.])
            self.assert_restored()
            self.assert_released()

    def test_begin_copy_error_restores_all_transients(self):
        animation = self.animation(suspend_mobject_updating=True)
        failure = ValueError("native copy")
        def fail():
            raise failure
        animation.create_starting_mobject = fail
        with self.assertRaises(ValueError) as caught:
            animation.begin()
        self.assertIs(caught.exception, failure)
        self.assert_restored()
        self.assert_released()

    def test_snapshot_updater_error_unwinds(self):
        animation = self.animation(suspend_mobject_updating=True)
        animation.begin()
        failure = RuntimeError("snapshot updater")
        def fail(dt):
            raise failure
        animation.starting_mobject.update = fail
        with self.assertRaises(RuntimeError) as caught:
            animation.update_mobjects(.25)
        self.assertIs(caught.exception, failure)
        self.assert_restored()
        self.assert_released()

    def test_abort_failure_is_not_hidden_and_can_be_retried(self):
        animation = self.animation(suspend_mobject_updating=True)
        animation.begin()
        failure = RuntimeError("restore setter")
        self.root.fail = failure
        with self.assertRaises(RuntimeError) as caught:
            animation.abort()
        self.assertIs(caught.exception, failure)
        self.assertTrue(animation._subset_active)
        animation.abort()
        self.assertFalse(animation._subset_active)
        self.assert_restored()
        self.assert_released()

    def test_cleanup_failure_does_not_replace_primary_error(self):
        animation = self.animation(suspend_mobject_updating=True)
        animation.begin()
        primary, cleanup = LookupError("select"), RuntimeError("restore")
        def fail(_):
            self.root.fail = cleanup
            raise primary
        animation.int_func = fail
        with self.assertRaises(LookupError) as caught:
            animation.interpolate(.5)
        self.assertIs(caught.exception, primary)
        self.assertTrue(primary.__notes__)
        animation.abort()
        self.assert_restored()

    def test_installer_is_idempotent_and_keeps_public_class_and_method_identity(self):
        old = self.cls.__init__
        install_subset_reveal(self.native)
        self.assertIs(self.cls.__init__, old)
        self.assertIs(self.g["ShowIncreasingSubsets"], self.cls)
        self.assertEqual(old.__qualname__, self.cls.__qualname__ + ".__init__")
        self.assertEqual(old.__module__, self.cls.__module__)


if __name__ == "__main__":
    unittest.main()
