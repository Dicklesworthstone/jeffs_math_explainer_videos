"""Live text selection over production animation code and fixture glyph records.

The companion native subset suite covers actual Scribe/Marionette integration;
these fixtures are not a rendering or installed-wheel acceptance claim.
"""
import math
import unittest

import numpy as np

from test_text_reveal_lifecycle_protocol import environment


class TextSelectorTests(unittest.TestCase):
    def setUp(self):
        self.g, self.Node = environment()
        self.glyphs = [self.Node(letter) for letter in "abcd"]
        # Native painter order deliberately differs from source span order.
        self.part = self.Node("part", *reversed(self.glyphs))
        self.root = self.Node("root", self.part)
        self.root.string = "ab cd"
        self.root._byte_span = lambda span: span
        self.root._string_sub_spans = [(0, 1), (1, 2), (3, 4), (4, 5)]
        self.root._string_sub_paths = [(0, index) for index in (3, 2, 1, 0)]
        self.family = tuple(self.root.get_family())

    def animation(self, cls="AddTextWordByWord", **kwargs):
        return self.g[cls](self.root, **kwargs)

    def assert_prefix(self, count):
        visible = self.root.get_family()
        self.assertEqual([node for node in self.glyphs if node in visible], self.glyphs[:count])
        self.assertEqual(self.part.submobjects, self.glyphs[:count][::-1])

    def assert_released(self):
        self.assertFalse(any(node.suspended for node in self.family))
        self.assertFalse(any(node._is_animating for node in self.family))

    def test_word_selector_receives_word_count_and_keeps_nested_native_order(self):
        calls = []
        def select(value):
            calls.append(value)
            return math.floor(value)
        animation = self.animation(int_func=select)
        self.assertIs(animation.int_func, select)
        self.assertEqual(calls, [])
        animation.interpolate(.4)
        self.assert_prefix(0)
        animation.interpolate(.6)
        self.assert_prefix(2)
        self.assertEqual(calls, [.8, 1.2])
        animation.interpolate(1)
        self.assert_prefix(4)

    def test_letter_selector_receives_glyph_count(self):
        calls = []
        animation = self.animation("AddTextLetterByLetter", int_func=lambda v: calls.append(v) or math.floor(v))
        animation.interpolate(.6)
        self.assertEqual(calls, [2.4])
        self.assert_prefix(2)
        animation.interpolate(.9)
        self.assert_prefix(3)

    def test_rate_and_selector_are_called_once_with_raw_time_spanned_alpha(self):
        calls = []
        def rate(alpha):
            calls.append(("rate", alpha))
            return alpha / 2
        def select(value):
            calls.append(("select", value))
            return math.ceil(value)
        animation = self.animation(int_func=select, rate_func=rate, time_span=(2, 4), run_time=4)
        animation.interpolate(.5)
        self.assertEqual(calls, [("rate", .5), ("select", .5)])
        self.assert_prefix(2)

    def test_callable_instance_and_reassignment_are_live(self):
        class Selector:
            count = 0
            def __call__(self, value):
                self.count += 1
                return self.count
        selector = Selector()
        animation = self.animation(int_func=selector)
        animation.interpolate(.5)
        self.assert_prefix(2)
        animation.interpolate(.5)
        self.assert_prefix(4)
        animation.int_func = lambda _: 0
        animation.interpolate(.5)
        self.assert_prefix(0)
        self.assertEqual(selector.count, 2)

    def test_default_selector_remains_numpy_round_ties_to_even(self):
        animation = self.animation()
        self.assertIs(animation.int_func, np.round)
        animation.interpolate(.25)
        self.assert_prefix(0)
        animation.interpolate(.75)
        self.assert_prefix(4)

    def test_custom_prefix_indices_include_negative_and_huge_values(self):
        for kind, units, stride in (("AddTextWordByWord", 2, 2), ("AddTextLetterByLetter", 4, 1)):
            animation = self.animation(kind)
            for index in (-10**100, -4, -2, -1, 0, 1, 2, 3, 4, 10**100):
                animation.int_func = lambda _, index=index: index
                animation.interpolate(.5)
                self.assert_prefix(len(range(units)[:index]) * stride)
            animation._nested_plan.restore()

    def test_public_selection_hook_is_live_even_on_identical_frames(self):
        calls = []
        class Authored(self.g["AddTextWordByWord"]):
            def update_submobject_list(self, index):
                calls.append(index)
                super().update_submobject_list(index)
        animation = Authored(self.root, int_func=math.floor)
        animation.interpolate(.5)
        animation.interpolate(.5)
        self.assertEqual(calls, [1, 1])
        self.assert_prefix(2)

    def test_direct_selection_hook_maps_units_through_source_spans(self):
        animation = self.animation()
        animation.update_submobject_list(-1)
        self.assert_prefix(2)
        animation.update_submobject_list(10)
        self.assert_prefix(4)
        animation.update_submobject_list(0)
        self.assert_prefix(0)

    def test_non_callable_is_rejected_before_glyph_mutation(self):
        for value in (None, "floor", 0, object()):
            with self.assertRaisesRegex(TypeError, "int_func must be callable"):
                self.animation(int_func=value)
            self.assert_prefix(4)
            self.assert_released()

    def test_failing_selector_restores_every_hidden_glyph_and_error_identity(self):
        failure = LookupError("authored text selection")
        def select(value):
            if value > 0:
                raise failure
            return 0
        animation = self.animation(int_func=select, suspend_mobject_updating=True)
        animation.begin()
        self.assert_prefix(0)
        with self.assertRaises(LookupError) as caught:
            animation.interpolate(.5)
        self.assertIs(caught.exception, failure)
        self.assert_prefix(4)
        self.assert_released()

    def test_invalid_selector_result_is_transactional(self):
        for value in (float("nan"), float("inf"), object()):
            animation = self.animation(suspend_mobject_updating=True)
            animation.begin()
            animation.int_func = lambda _: value
            with self.assertRaises((ValueError, OverflowError, TypeError)):
                animation.finish()
            self.assert_prefix(4)
            self.assert_released()

    def test_selection_hook_failure_after_mutation_is_transactional(self):
        failure = RuntimeError("authored selection hook")
        class Authored(self.g["AddTextWordByWord"]):
            def update_submobject_list(self, index):
                super().update_submobject_list(index)
                if index > 0:
                    raise failure
        animation = Authored(self.root, suspend_mobject_updating=True)
        animation.begin()
        with self.assertRaises(RuntimeError) as caught:
            animation.interpolate(.5)
        self.assertIs(caught.exception, failure)
        self.assert_prefix(4)
        self.assert_released()

    def test_interpolation_override_failure_after_super_is_transactional(self):
        failure = RuntimeError("authored interpolation hook")
        class Authored(self.g["AddTextWordByWord"]):
            def interpolate_mobject(self, alpha):
                super().interpolate_mobject(alpha)
                if alpha > 0:
                    raise failure
        animation = Authored(self.root, suspend_mobject_updating=True)
        animation.begin()
        with self.assertRaises(RuntimeError) as caught:
            animation.interpolate(.5)
        self.assertIs(caught.exception, failure)
        self.assert_prefix(4)
        self.assert_released()

    def test_selector_is_reused_on_replay_and_partial_finish_releases_all(self):
        animation = self.animation(int_func=math.floor, final_alpha_value=.6,
                                   suspend_mobject_updating=True)
        for _ in range(2):
            animation.begin()
            animation.finish()
            self.assert_prefix(2)
            self.assert_released()
        animation.int_func = math.ceil
        animation.begin()
        animation.finish()
        self.assert_prefix(4)
        self.assert_released()


if __name__ == "__main__":
    unittest.main()
