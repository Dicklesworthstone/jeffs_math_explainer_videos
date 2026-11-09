"""Live numeric TeX acceptance through real native mobjects and rendering."""
from __future__ import annotations

import unittest

import manimlib as m
import numpy as np


class JoinedTex(m.Tex):
    _tex_arg_separator = ""


def identities(tex):
    return [tex._string_submobject(i) for i in range(len(tex._string_sub_paths))]


def metadata(tex):
    return (tex.string, tex.tex_string, list(tex.tex_strings),
            list(tex._string_sub_spans), [list(p) for p in tex._string_sub_paths])


def nest(tex):
    """An authored wrapper around existing native glyphs, not fake glyph data."""
    old = list(tex.submobjects)
    wrapper = m.VGroup(m.VGroup(*old))
    tex.set_submobjects([wrapper])
    tex._string_sub_paths = [[0, 0, *p] for p in tex._string_sub_paths]
    return wrapper


def group_at(tex, split):
    """Author two real native groups for cross-parent splice/rollback tests.

    Modern Tex no longer inserts argument containers. These tests deliberately
    exercise edited, nested families, independently of constructor grouping.
    """
    children = list(tex.submobjects)
    assert all(len(path) == 1 for path in tex._string_sub_paths)
    tex.set_submobjects([m.VGroup(*children[:split]), m.VGroup(*children[split:])])
    tex._string_sub_paths = [
        [0, path[0]] if path[0] < split else [1, path[0] - split]
        for path in tex._string_sub_paths
    ]
    return tex


class LiveTexAcceptance(unittest.TestCase):
    def assert_map(self, tex):
        payload = tex.string.encode("utf-8")
        self.assertEqual(len(tex._string_sub_spans), len(tex._string_sub_paths))
        self.assertEqual(tex.get_tex(), tex.string)
        for i, (start, end) in enumerate(tex._string_sub_spans):
            self.assertTrue(0 <= start <= end <= len(payload), (start, end, payload))
            payload[:start].decode("utf-8")
            payload[:end].decode("utf-8")
            self.assertIsInstance(tex._string_submobject(i), m.VMobject)

    def test_multiple_independent_numbers_can_be_made_live_sequentially(self):
        tex = m.Tex("a=1.00+b=2.00+c=3.00")
        values = [tex.make_number_changeable(v) for v in ("1.00", "2.00", "3.00")]
        self.assertTrue(all(isinstance(v, m.DecimalNumber) for v in values))
        self.assertEqual(tex.string, r"a=\decimalmob+b=\decimalmob+c=\decimalmob")
        self.assertEqual(len(tex.get_parts_by_tex(r"\decimalmob")), 3)
        self.assert_map(tex)

    def test_index_selects_the_matching_source_occurrence(self):
        tex = m.Tex("1.00+1.00+1.00")
        first = list(tex.get_part_by_tex("1.00", 0))
        decimal = tex.make_number_changeable("1.00", index=1)
        self.assertIsInstance(decimal, m.DecimalNumber)
        self.assertEqual(tex.string, r"1.00+\decimalmob+1.00")
        self.assertEqual(list(tex.get_part_by_tex("1.00", 0)), first)
        self.assert_map(tex)

    def test_negative_index_and_later_conversion(self):
        tex = m.Tex("1.00+1.00")
        last = tex.make_number_changeable("1.00", index=-1)
        first = tex.make_number_changeable("1.00")
        self.assertIsInstance(first, m.DecimalNumber)
        self.assertIsNot(first, last)
        self.assertEqual(tex.string, r"\decimalmob+\decimalmob")
        self.assert_map(tex)

    def test_replace_all_retains_independent_live_values(self):
        tex = m.Tex("1.00+1.00=2.00")
        decimals = tex.make_number_changeable("1.00", replace_all=True)
        self.assertEqual(len(decimals), 2)
        decimals[0].set_value(8.25)
        self.assertEqual(decimals[1].get_value(), 1)
        self.assertIsInstance(tex.make_number_changeable("2.00"), m.DecimalNumber)
        self.assert_map(tex)

    def test_native_utf8_spans_remain_selectable_after_replacement(self):
        tex = m.TexText("é $1.00$ then $2.00$ τέλος")
        tex.make_number_changeable("1.00")
        self.assertIsInstance(tex.make_number_changeable("2.00"), m.DecimalNumber)
        self.assertTrue(len(tex.get_part_by_tex("é")))
        self.assertTrue(len(tex.get_part_by_tex("τέλος")))
        self.assert_map(tex)

    def test_nested_part_identity_and_unmapped_decorations_survive(self):
        tex = group_at(m.Tex("y =", "1.00 x + 2.00"), 2)
        roots = list(tex.submobjects)
        dot = m.Dot().shift(3 * m.UP)
        roots[-1].add(dot)
        wrapper = nest(tex)
        first = tex.make_number_changeable("1.00")
        second = tex.make_number_changeable("2.00")
        self.assertIsInstance(first, m.DecimalNumber)
        self.assertIsInstance(second, m.DecimalNumber)
        self.assertIs(tex.submobjects[0], wrapper)
        self.assertEqual(list(wrapper[0]), roots)
        self.assertIn(dot, roots[-1].submobjects)
        self.assert_map(tex)

    def test_number_crossing_argument_groups_is_replaced_once(self):
        tex = group_at(JoinedTex("a=1", "2.50", "+3.00"), 3)
        roots = list(tex.submobjects)
        number = tex.make_number_changeable("12.50")
        self.assertIsInstance(number, m.DecimalNumber)
        self.assertEqual(number.get_value(), 12.5)
        self.assertEqual(list(tex.submobjects), roots)
        self.assertEqual(tex.tex_strings, [r"a=\decimalmob", "", "+3.00"])
        self.assertIsInstance(tex.make_number_changeable("3.00"), m.DecimalNumber)
        self.assertEqual(identities(tex).count(number), 1)
        self.assert_map(tex)

    def test_native_matrix_tex_entry_can_become_a_live_number(self):
        matrix = m.TexMatrix([["12.50", "x"]])
        entry = matrix.get_mob_matrix()[0][0]
        self.assertEqual(entry._string_sub_paths, [[]])
        before = entry.get_bounding_box().copy()
        number = entry.make_number_changeable("12.50")
        self.assertIs(matrix.get_mob_matrix()[0][0], entry)
        self.assertIsInstance(number, m.DecimalNumber)
        self.assertEqual(entry._string_sub_paths, [[0]])
        np.testing.assert_allclose(number.get_center(), (before[0] + before[2]) / 2, atol=2e-5)
        self.assert_map(entry)

    def test_preserves_live_scene_identity_and_style(self):
        scene = m.Scene()
        tex = m.Tex("x=1.00+2.00", color=m.RED)
        scene.add(tex)
        before = list(scene.mobjects)
        first = tex.make_number_changeable("1.00")
        second = tex.make_number_changeable("2.00")
        self.assertEqual(list(scene.mobjects), before)
        self.assertEqual(first.get_color(), m.RED)
        scene.play(m.ChangeDecimalToValue(first, 4.25),
                   m.ChangeDecimalToValue(second, 8.5), run_time=2 / 30)
        self.assertAlmostEqual(first.get_value(), 4.25)
        self.assertAlmostEqual(second.get_value(), 8.5)
        self.assert_map(tex)

    def test_copy_remaps_live_number_paths_without_mutating_original(self):
        tex = m.Tex("1.00+2.00")
        original = tex.make_number_changeable("1.00")
        duplicate = tex.copy()
        copied = duplicate.get_part_by_tex(r"\decimalmob")[0]
        self.assertIsNot(copied, original)
        copied.set_value(9)
        self.assertEqual(original.get_value(), 1)
        self.assertIsInstance(duplicate.make_number_changeable("2.00"), m.DecimalNumber)
        self.assertIn("2.00", tex.string)
        self.assert_map(duplicate)

    def test_invalid_configuration_leaves_source_and_topology_unchanged(self):
        tex = m.Tex("1.00+1.00")
        before = metadata(tex), identities(tex)
        with self.assertRaises((TypeError, ValueError, RuntimeError)):
            tex.make_number_changeable("1.00", replace_all=True, definitely_unknown=True)
        self.assertEqual((metadata(tex), identities(tex)), before)

    def test_cross_part_splice_failure_rolls_back_both_native_parents(self):
        tex = group_at(JoinedTex("1", "2.50+3.00"), 1)
        scene = m.Scene()
        scene.add(tex)
        before = metadata(tex), identities(tex), list(scene.mobjects)
        parents = list(tex.submobjects)
        old_children = [list(parent.submobjects) for parent in parents]
        native_set = parents[1].set_submobjects
        sentinel = RuntimeError("authored splice failed after native mutation")
        calls = []

        def failing_once(children):
            result = native_set(children)
            calls.append(tuple(children))
            if len(calls) == 1:
                raise sentinel
            return result

        parents[1].set_submobjects = failing_once
        self.addCleanup(parents[1].__dict__.pop, "set_submobjects", None)
        with self.assertRaises(RuntimeError) as caught:
            tex.make_number_changeable("12.50")
        self.assertIs(caught.exception, sentinel)
        self.assertEqual((metadata(tex), identities(tex), list(scene.mobjects)), before)
        self.assertEqual([list(parent.submobjects) for parent in parents], old_children)
        self.assertEqual(len(calls), 2)
        self.assertIsInstance(tex.make_number_changeable("12.50"), m.DecimalNumber)
        self.assert_map(tex)

    def test_whole_matrix_entry_failure_restores_native_points(self):
        entry = m.TexMatrix([["12.50"]]).get_mob_matrix()[0][0]
        before, points, children = metadata(entry), entry.get_points().copy(), list(entry.submobjects)
        native_set = entry.set_submobjects
        sentinel = RuntimeError("matrix splice failed")
        failed = False

        def failing_once(values):
            nonlocal failed
            result = native_set(values)
            if not failed:
                failed = True
                raise sentinel
            return result

        entry.set_submobjects = failing_once
        self.addCleanup(entry.__dict__.pop, "set_submobjects", None)
        with self.assertRaises(RuntimeError) as caught:
            entry.make_number_changeable("12.50")
        self.assertIs(caught.exception, sentinel)
        self.assertEqual(metadata(entry), before)
        self.assertEqual(list(entry.submobjects), children)
        np.testing.assert_array_equal(entry.get_points(), points)
        self.assertIsInstance(entry.make_number_changeable("12.50"), m.DecimalNumber)

    def test_same_number_replaced_in_reverse_order_preserves_every_span(self):
        tex = m.Tex("x=1.00+1.00+1.00+1.00")
        decimals = [tex.make_number_changeable("1.00", index=i) for i in (3, 1, 1, 0)]
        self.assertTrue(all(isinstance(decimal, m.DecimalNumber) for decimal in decimals))
        self.assertEqual(len(set(map(id, decimals))), 4)
        self.assertEqual(tex.string, r"x=\decimalmob+\decimalmob+\decimalmob+\decimalmob")
        self.assertEqual(len(tex.get_parts_by_tex(r"\decimalmob")), 4)
        self.assert_map(tex)

    def test_missing_and_past_end_are_empty_without_mutation(self):
        tex = m.Tex("1.00")
        before = metadata(tex)
        for value, index in (("9.99", 0), ("1.00", 4)):
            result = tex.make_number_changeable(value, index=index)
            self.assertIsInstance(result, m.VMobject)
            self.assertEqual(len(result), 0)
        self.assertEqual(metadata(tex), before)

    def test_infix_fraction_family_indices_remain_live_after_copy(self):
        import copy
        for bound in (False, True):
            for duplicate in (lambda obj: obj.copy(), copy.deepcopy):
                with self.subTest(bound=bound, duplicate=duplicate.__name__):
                    tex = m.Tex(r"1 \over 2")
                    first = tex.make_number_changeable("1")
                    last = tex.make_number_changeable("2")
                    self.assertIs(tex[0], first)
                    self.assertIs(tex[2], last)
                    if bound:
                        scene = m.Scene()
                        scene.add(tex)
                    copied = duplicate(tex)
                    self.assertIsInstance(copied[0], m.DecimalNumber)
                    self.assertIsInstance(copied[2], m.DecimalNumber)
                    copied[0].set_value(3)
                    copied[2].set_value(7)
                    self.assertEqual((first.get_value(), last.get_value()), (1, 2))
                    self.assertEqual((copied[0].get_value(), copied[2].get_value()), (3, 7))
                    self.assert_map(copied)

    def test_infix_rule_order_keeps_native_ordinals_and_span_selection(self):
        tex = m.Tex(r"12 \over 34")
        self.assertEqual(list(tex.get_part_by_tex("12")), list(tex[:2]))
        self.assertEqual(list(tex.get_part_by_tex(r"\over")), [tex[2]])
        self.assertEqual(list(tex.get_part_by_tex("34")), list(tex[3:]))
        # Native primitives follow TeX's emission order since the 3ed1d332
        # repin (fm-aia1): numerator, bar, denominator.
        self.assertEqual(tex._string_sub_spans, [(0, 1), (1, 2), (3, 8), (9, 10), (10, 11)])
        self.assertEqual(tex._string_sub_paths, [[0], [1], [2], [3], [4]])
        self.assert_map(tex)

    def test_nested_infix_fraction_rules_are_inserted_at_their_own_denominators(self):
        for source, expected in ((r"{1 \over 2} \over 3", ("1", r"\over", "2", r"\over", "3")),
                                 (r"1 \over {2 \over 3}", ("1", r"\over", "2", r"\over", "3")),
                                 (r"x+{1 \over 2}+{3 \over 4}+y",
                                  ("x", "+", "1", r"\over", "2", "+", "3", r"\over", "4", "+", "y"))):
            with self.subTest(source=source):
                tex = m.Tex(source)
                payload = source.encode()
                observed = [None] * len(tex)
                for span, path in zip(tex._string_sub_spans, tex._string_sub_paths):
                    self.assertEqual(len(path), 1)
                    observed[path[0]] = payload[slice(*span)].decode()
                self.assertEqual(tuple(observed), expected)
                self.assert_map(tex)

    def test_infix_rule_order_is_safe_in_legacy_part_groups(self):
        from manimlib.mobject.svg.old_tex_mobject import OldTex
        tex = OldTex(r"{1 \over 2}", "+", r"{3 \over 4}")
        self.assertEqual(len(tex), 3)
        for group in (tex[0], tex[2]):
            self.assertGreater(group[0].get_center()[1], group[1].get_center()[1])
            self.assertGreater(group[1].get_center()[1], group[2].get_center()[1])
        self.assert_map(tex)

    def test_infix_utf8_and_authored_decorations_preserve_selection_and_regeneration(self):
        class Decorated(m.TexText):
            def init_data(self):
                super().init_data()
                self.dot = m.Dot().shift(m.DOWN)
                self.add(self.dot)
        tex = Decorated(r"é $1 \over 2$")
        for _ in range(2):
            self.assertIs(tex[0], tex.dot)
            self.assertIs(tex.get_part_by_tex("1")[0], tex[2])
            self.assertIs(tex.get_part_by_tex(r"\over")[0], tex[3])
            self.assertIs(tex.get_part_by_tex("2")[0], tex[4])
            self.assert_map(tex)
            tex.init_points()
        scene = m.Scene()
        scene.add(tex)
        tex.init_points()
        self.assertIs(tex[0], tex.dot)
        self.assertIs(tex.get_part_by_tex("2")[0], tex[4])

    def test_infix_fraction_copy_save_restore_keeps_typed_denominator(self):
        tex = m.Tex(r"1 \over 2")
        tex.make_number_changeable("1")
        tex.make_number_changeable("2")
        tex.save_state()
        denominator = tex[2]
        self.assertIsInstance(denominator, m.DecimalNumber)
        denominator.set_value(9)
        tex.restore()
        self.assertIs(tex[2], denominator)
        # Reference become restores data/uniforms, not the Python `number`
        # attribute (numbers.py:get_value). Its public numeric API stays live.
        self.assertEqual(denominator.get_value(), 9)
        denominator.set_value(7)
        self.assertEqual(denominator.get_value(), 7)
        self.assertEqual(tex[0].get_value(), 1)
        self.assert_map(tex)

    def test_infix_fraction_indexed_animation_matches_selector_driven_native_frames(self):
        import tempfile
        from pathlib import Path
        root = Path(tempfile.mkdtemp(prefix="fmn-infix-fraction-"))
        print("retaining infix fraction frame evidence:", root)
        def render(path, indices, workers):
            scene = m.Scene()
            with scene.render_session(path, format="y4m", resolution=(96, 54), fps=8, threads=workers):
                tex = m.Tex(r"1 \over 2").scale(4)
                numerator = tex.make_number_changeable("1")
                denominator = tex.make_number_changeable("2")
                scene.add(tex)
                if indices:
                    numerator, denominator = tex[0], tex[2]
                scene.play(m.ChangeDecimalToValue(numerator, 5),
                           m.ChangeDecimalToValue(denominator, 7), run_time=0.5, rate_func=m.linear)
            return path.read_bytes()
        expected = render(root / "control.y4m", False, 1)
        for workers in (1, 4, 16):
            self.assertEqual(render(root / f"indexed-{workers}.y4m", True, workers), expected)
        header, frames = expected.split(b"\n", 1)
        frame_size = 96 * 54 * 3 // 2 + len(b"FRAME\n")
        self.assertEqual(len(frames), 4 * frame_size)
        self.assertNotEqual(frames[:frame_size], frames[-frame_size:])

    def test_infix_reordering_does_not_change_plain_or_prefix_tex_construction(self):
        for source in ("x+y", r"\frac{1}{2}", r"\sqrt{x}"):
            tex = m.Tex(source)
            self.assertEqual(tex._string_sub_paths, [[i] for i in range(len(tex._string_sub_spans))])
            self.assert_map(tex)


suite = unittest.defaultTestLoader.loadTestsFromTestCase(LiveTexAcceptance)
result = unittest.TextTestRunner(verbosity=2).run(suite)
assert result.wasSuccessful(), "native live-TeX acceptance failed"
