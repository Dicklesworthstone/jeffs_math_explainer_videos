"""Native-family protocol tests; record fixtures are not renderer acceptance."""
import ast
import importlib.util
from pathlib import Path
import re
import types
import unittest

import numpy as np

MODULE = Path(__file__).resolve().parents[1] / "python/fmn_python/text_reveal.py"
spec = importlib.util.spec_from_file_location("_text_reveal_under_test", MODULE)
reveal = importlib.util.module_from_spec(spec)
spec.loader.exec_module(reveal)


class Node:
    def __init__(self, name, *children, records=0):
        self.name, self.submobjects = name, list(children)
        self.data = np.ones(records, dtype=[("point", "f8", (3,)), ("rgba", "f8", (4,))])
        self.calls = []
        self.fail = None

    def set_submobjects(self, children):
        self.calls.append(tuple(child.name for child in children))
        self.submobjects[:] = children
        if self.fail is not None:
            error, self.fail = self.fail, None
            raise error
        return self

    def set_data(self, data):
        self.data = data.copy()
        return self


def string(*children, paths, records=0):
    root = Node("root", *children, records=records)
    root._string_sub_paths = paths
    root._string_sub_spans = [(i, i + 1) for i in range(len(paths))]
    return root


def names(node):
    return [child.name for child in node.submobjects]


class GlyphTreeTests(unittest.TestCase):
    def test_deep_mixed_nesting_preserves_exact_glyph_identities(self):
        a, b, c = (Node(name, records=3) for name in "abc")
        inner, part = Node("inner", a), Node("part", b)
        root = string(Node("outer", inner), part, c, paths=[(0, 0, 0), (1, 0), (2,)])
        plan = reveal._GlyphRevealPlan(root)
        plan.apply(1)
        self.assertEqual(names(root), ["outer"])
        self.assertIs(root.submobjects[0].submobjects[0].submobjects[0], a)
        plan.apply(2)
        self.assertEqual(names(root), ["outer", "part"])
        plan.restore()
        self.assertEqual(names(root), ["outer", "part", "c"])
        self.assertIs(root.submobjects[-1], c)

    def test_reading_order_is_independent_of_painter_order(self):
        a, b, c, d = (Node(name) for name in "abcd")
        left, right = Node("left", a, b), Node("right", c, d)
        root = string(left, right, paths=[(1, 0), (0, 1), (1, 1), (0, 0)])
        plan = reveal._GlyphRevealPlan(root)
        plan.apply(1)
        self.assertEqual(names(root), ["right"])
        self.assertEqual(names(right), ["c"])
        plan.apply(2)
        self.assertEqual(names(root), ["left", "right"])
        self.assertEqual(names(left), ["b"])
        plan.restore()
        self.assertEqual(names(left), ["a", "b"])

    def test_ornaments_follow_completed_parent_and_retain_order(self):
        part = Node("part", Node("before"), Node("a"), Node("after"))
        root = string(part, Node("b"), Node("root ornament"), paths=[(0, 1), (1,)])
        plan = reveal._GlyphRevealPlan(root)
        plan.apply(0)
        self.assertEqual(names(root), [])
        self.assertEqual(names(part), [])
        plan.apply(1)
        self.assertEqual(names(root), ["part"])
        self.assertEqual(names(part), ["before", "a", "after"])
        plan.restore()
        self.assertEqual(names(root), ["part", "b", "root ornament"])

    def test_backward_seek_and_repeated_count_do_not_accumulate_mutations(self):
        children = [Node(str(i)) for i in range(4)]
        part = Node("part", *children)
        root = string(part, paths=[(0, i) for i in range(4)])
        plan = reveal._GlyphRevealPlan(root)
        for count in (0, 3, 1, 4, 2, 0, 4):
            plan.apply(count)
            self.assertEqual(names(part), [str(i) for i in range(count)])
        calls = len(root.calls), len(part.calls)
        plan.apply(4)
        self.assertEqual((len(root.calls), len(part.calls)), calls)
        self.assertTrue(all(a is b for a, b in zip(part.submobjects, children)))

    def test_source_glyph_may_be_a_complete_live_numeric_family(self):
        digit = Node("digit", records=3)
        number = Node("number", digit)
        root = string(Node("part", number), paths=[(0, 0)])
        plan = reveal._GlyphRevealPlan(root)
        plan.apply(0)
        digit.data["point"] = 7
        plan.restore()
        self.assertIs(root.submobjects[0].submobjects[0], number)
        np.testing.assert_array_equal(digit.data["point"], 7)

    def test_whole_entry_root_records_and_children_hide_and_restore(self):
        root = string(Node("rule"), paths=[()], records=3)
        expected = root.data.copy()
        plan = reveal._GlyphRevealPlan(root)
        plan.apply(0)
        self.assertEqual((len(root.data), names(root)), (0, []))
        plan.restore()
        np.testing.assert_array_equal(root.data, expected)
        self.assertEqual(names(root), ["rule"])

    def test_parent_record_ornament_is_not_visible_before_its_glyphs(self):
        part = Node("part", Node("a"), Node("b"), records=3)
        root = string(part, paths=[(0, 0), (0, 1)])
        plan = reveal._GlyphRevealPlan(root)
        plan.apply(1)
        self.assertEqual(len(part.data), 0)
        plan.apply(2)
        self.assertEqual(len(part.data), 3)

    def test_invalid_maps_refuse_before_any_mutation(self):
        for paths in ([], [(0,), (0,)], [(0,), (0, 0)], [(2,)], [(-1,)], [(True,)]):
            with self.subTest(paths=paths):
                root = string(Node("a"), paths=paths)
                with self.assertRaises((ValueError, TypeError)):
                    reveal._GlyphRevealPlan(root)
                self.assertEqual(root.calls, [])
                self.assertEqual(names(root), ["a"])

    def test_mismatched_span_count_refuses(self):
        root = string(Node("a"), paths=[(0,)])
        root._string_sub_spans = []
        with self.assertRaises(ValueError):
            reveal._GlyphRevealPlan(root)

    def test_cycles_and_shared_mutable_parents_refuse_before_edits(self):
        root = string(paths=[(0, 0)])
        root.submobjects.append(root)
        with self.assertRaisesRegex(ValueError, "cyclic"):
            reveal._GlyphRevealPlan(root)
        part = Node("part", Node("a"), Node("b"))
        root = string(part, part, paths=[(0, 0), (1, 1)])
        with self.assertRaisesRegex(ValueError, "alias"):
            reveal._GlyphRevealPlan(root)
        self.assertEqual(part.calls, [])

    def test_setter_failure_after_mutation_rolls_back_entire_frame(self):
        part = Node("part", Node("a"), Node("b"))
        root = string(part, paths=[(0, 0), (0, 1)], records=3)
        plan = reveal._GlyphRevealPlan(root)
        error = LookupError("authored setter failed")
        root.fail = error
        with self.assertRaises(LookupError) as caught:
            plan.apply(0)
        self.assertIs(caught.exception, error)
        self.assertEqual(names(part), ["a", "b"])
        self.assertEqual(names(root), ["part"])
        self.assertEqual(len(root.data), 3)
        plan.apply(0)
        self.assertEqual(names(root), [])
        plan.restore()
        self.assertEqual(len(root.data), 3)

    def test_failure_restores_previous_partial_state_not_a_different_prefix(self):
        part = Node("part", Node("a"), Node("b"))
        root = string(part, paths=[(0, 0), (0, 1)])
        plan = reveal._GlyphRevealPlan(root)
        plan.apply(1)
        root.fail = RuntimeError("stop")
        with self.assertRaises(RuntimeError):
            plan.apply(0)
        self.assertEqual(names(root), ["part"])
        self.assertEqual(names(part), ["a"])

    def test_foreign_plan_is_rejected(self):
        root = string(Node("a"), paths=[(0,)])
        other = string(Node("b"), paths=[(0,)])
        with self.assertRaisesRegex(ValueError, "different"):
            reveal._apply_glyph_reveal(other, reveal._GlyphRevealPlan(root), 0)
        self.assertEqual(names(other), ["b"])

    def test_counts_clamp_and_accept_numpy_integers(self):
        root = string(Node("a"), paths=[(0,)])
        plan = reveal._GlyphRevealPlan(root)
        plan.apply(np.int64(-2))
        self.assertEqual(names(root), [])
        plan.apply(100)
        self.assertEqual(names(root), ["a"])
        with self.assertRaises(TypeError):
            plan.apply(.5)


class ExistingAnimationIntegrationTests(unittest.TestCase):
    def environment(self):
        native = types.ModuleType("text_reveal_fixture")
        class Animation:
            def __init__(self, mobject, run_time, rate_func, **kwargs):
                self.mobject, self.run_time, self.rate_func = mobject, run_time, rate_func
        native.__dict__.update(Animation=Animation, StringMobject=Node, _np=np, _re=re,
                               _FMN_ROOT=types.SimpleNamespace(linear=lambda t: t))
        source = MODULE.parent.parent / "manimlib_bootstrap.py"
        names = {"_string_word_groups", "_string_glyph_reveal_plan", "_apply_glyph_reveal",
                 "AddTextWordByWord", "AddTextLetterByLetter"}
        nodes = [node for node in ast.parse(source.read_text()).body
                 if isinstance(node, (ast.ClassDef, ast.FunctionDef)) and node.name in names]
        exec(compile(ast.Module(nodes, []), str(source), "exec"), native.__dict__)
        return native

    def test_existing_word_and_letter_constructors_reach_installed_plan(self):
        for name in ("AddTextWordByWord", "AddTextLetterByLetter"):
            with self.subTest(animation=name):
                native = self.environment()
                cls = getattr(native, name)
                a, b = Node("a"), Node("b")
                root = string(Node("part", Node("inner", a, b)), paths=[(0, 0, 0), (0, 0, 1)])
                root.string = "a b"
                root._string_sub_spans = [(0, 1), (2, 3)]
                root._byte_span = lambda span: span
                with self.assertRaises(NotImplementedError):
                    cls(root)
                reveal.install_text_reveal(native)
                reveal.install_text_reveal(native)
                self.assertIs(getattr(native, name), cls)
                animation = cls(root)
                animation.interpolate_mobject(0)
                self.assertEqual(names(root), [])
                animation.interpolate_mobject(.5)
                self.assertIs(root.submobjects[0].submobjects[0].submobjects[0], a)
                animation.interpolate_mobject(1)
                self.assertEqual(names(root.submobjects[0].submobjects[0]), ["a", "b"])

    def test_shared_native_initializer_owns_installer(self):
        source = MODULE.with_name("initialization.py")
        tree = ast.parse(source.read_text())
        steps = next(node.value for node in tree.body if isinstance(node, ast.Assign)
                     and any(isinstance(target, ast.Name) and target.id == "_STEPS"
                             for target in node.targets))
        self.assertIn(("text_reveal", "install_text_reveal"), ast.literal_eval(steps))


if __name__ == "__main__":
    unittest.main()
