"""Execute the real shared span-block planner with explicit Scribe doubles.

The full checkout's installer is AST-selected, not replaced by a test matcher.
The doubles below own only test glyph identities/spans; no native-render claim.
"""
import ast
import difflib
from pathlib import Path
import unittest

from test_matching_authoring_protocol import matching, namespace


def string_namespace():
    g = namespace()
    class StringMobject(g.Mobject):
        def __init__(self, text, *, grouped=False):
            super().__init__(None)
            self.text = text
            self.children = [g.Mobject(text)] if grouped else [g.Mobject(char) for char in text]
            self._string_sub_spans = []
            offset = 0
            for child in self.children:
                end = offset + len(child.shape.encode("utf-8"))
                self._string_sub_spans.append((offset, end))
                offset = end
        def get_string(self):
            return self.text
        def _string_submobject(self, ordinal):
            return self.children[ordinal]
    class Group(g.Mobject):
        def __init__(self, *parts):
            super().__init__(None, parts)
    class Strings(g.Animation):
        _match_by_blocks = True
        def __init__(self, *args, **kwargs):
            raise NotImplementedError("native-only legacy string construction")
        def _native_span_keys(self, mobject):
            # Explicit native-span gateway double, including its UTF-8 refusal.
            encoded = mobject.get_string().encode("utf-8")
            result = []
            for ordinal, (start, end) in enumerate(mobject._string_sub_spans):
                if not 0 <= start < end <= len(encoded):
                    raise ValueError("invalid native span")
                try:
                    key = encoded[start:end].decode("utf-8")
                except UnicodeDecodeError as error:
                    raise ValueError("native span split a UTF-8 code point") from error
                result.append((mobject._string_submobject(ordinal), key))
            return result
    class Tex(Strings):
        _match_by_blocks = False
    g.StringMobject, g.Group, g.VGroup, g.VMobject = StringMobject, Group, Group, g.Mobject
    g.TransformMatchingStrings, g.TransformMatchingTex = Strings, Tex
    g._difflib, g._TexError = difflib, ValueError
    path = Path(__file__).resolve().parents[1] / "python/manimlib/_animation_semantics.py"
    nodes = [node for node in ast.parse(path.read_text()).body
             if isinstance(node, ast.FunctionDef) and node.name == "_install_matching_strings"]
    if len(nodes) != 1:
        raise AssertionError("expected exactly one shared string-matching installer")
    scope = {}
    exec(compile(ast.Module(body=nodes, type_ignores=[]), str(path), "exec"), scope)
    scope["_install_matching_strings"](vars(g))
    matching.install_matching(g)
    matching.install_matching_strings(g)
    return g


def child_text(mobject):
    return "".join(piece.shape for piece in mobject.family_members_with_points())


class StringPlanningTests(unittest.TestCase):
    def setUp(self):
        self.g = string_namespace()
    def make(self, source, target, **kwargs):
        return self.g.TransformMatchingStrings(self.g.StringMobject(source), self.g.StringMobject(target), **kwargs)
    def pairs(self, animation):
        return [(child_text(child.mobject), child_text(child.target_mobject))
                for child in animation.animations if child._native_kind == "transform"]

    def test_real_marker_and_correct_public_lineage(self):
        self.assertEqual(self.g.__engine__, "FrankenManim")
        animation = self.make("ab", "ba")
        self.assertIsInstance(animation, self.g.TransformMatchingParts)
        self.assertIsInstance(animation, self.g.AnimationGroup)
        self.assertEqual(animation._native_kind, "transform_matching_strings")
        self.assertTrue(self.g._requires_python_animation(animation))
        self.assertEqual(animation.matched_pairs, [])

    def test_public_override_is_used_once_without_replanning(self):
        calls = []
        class Authored(self.g.TransformMatchingStrings):
            def matching_blocks(self, source, target, matched_keys, key_map):
                calls.append((source, target, matched_keys, key_map))
                yield source.children[0], target.children[0]
        source, target = self.g.StringMobject("ab"), self.g.StringMobject("ba")
        animation = Authored(source, target)
        self.assertEqual(len(calls), 1)
        self.assertEqual(self.pairs(animation), [("a", "b")])
        self.assertEqual([a._native_kind for a in animation.animations],
                         ["transform", "fade_out_to_point", "fade_in_from_point"])
        animation.begin()
        animation.finish()
        self.assertEqual(len(calls), 1)

    def test_multi_glyph_rename_reuses_real_shared_block_planner(self):
        animation = self.make("foo+foo", "bar+bar", key_map={"foo": "bar"})
        self.assertEqual(self.pairs(animation), [("foo", "bar"), ("foo", "bar"), ("+", "+")])
        self.assertEqual(animation.matched_pairs, [])
        again = animation.matching_blocks(animation.source, animation.target, (), animation.key_map)
        self.assertEqual([(child_text(s), child_text(t)) for s, t in again], self.pairs(animation))

    def test_reordered_blocks_are_not_lost(self):
        self.assertEqual(self.pairs(self.make("ab+cd", "cd+ab")), [("ab", "ab"), ("cd", "cd"), ("+", "+")])

    def test_unicode_renames_use_byte_spans(self):
        animation = self.make("é+é", "λ+λ", key_map={"é": "λ"})
        self.assertEqual(self.pairs(animation), [("é", "λ"), ("é", "λ"), ("+", "+")])

    def test_explicit_group_claim_masks_all_its_glyphs(self):
        source, target = self.g.StringMobject("ab+ab"), self.g.StringMobject("xy+xy")
        pair = (self.g.VGroup(*source.children[:2]), self.g.VGroup(*target.children[:2]))
        animation = self.g.TransformMatchingStrings(source, target, matched_pairs=[pair], key_map={"ab": "xy"})
        self.assertEqual(self.pairs(animation), [("ab", "xy"), ("ab", "xy"), ("+", "+")])
        self.assertEqual(animation.matched_pairs, [pair])

    def test_duplicate_explicit_claim_refuses_before_hook(self):
        calls = []
        class Authored(self.g.TransformMatchingStrings):
            def matching_blocks(self, *args):
                calls.append("hook")
                return []
        source, target = self.g.StringMobject("ab"), self.g.StringMobject("cd")
        with self.assertRaisesRegex(ValueError, "same part twice"):
            Authored(source, target, matched_pairs=[(source, target), (source.children[0], target.children[0])])
        self.assertEqual(calls, [])

    def test_masked_slots_do_not_invent_adjacent_substrings(self):
        source, target = self.g.StringMobject("aXb"), self.g.StringMobject("aYb")
        animation = self.g.TransformMatchingStrings(source, target,
            matched_pairs=[(source.children[1], target.children[1])])
        self.assertEqual(self.pairs(animation), [("X", "Y"), ("a", "a"), ("b", "b")])

    def test_tex_uses_semantic_keys_not_outline_equality(self):
        source, target = self.g.StringMobject("x+y"), self.g.StringMobject("y+x")
        animation = self.g.TransformMatchingTex(source, target)
        self.assertEqual(self.pairs(animation), [("x", "x"), ("+", "+"), ("y", "y")])
        # Give unrelated glyphs the same shape without changing their source keys.
        source, target = self.g.StringMobject("x"), self.g.StringMobject("z")
        source.children[0].shape = target.children[0].shape = "same outline"
        animation = self.g.TransformMatchingTex(source, target)
        self.assertEqual([a._native_kind for a in animation.animations], ["fade_out_to_point", "fade_in_from_point"])

    def test_tex_pinned_keys_filter_leftovers(self):
        animation = self.g.TransformMatchingTex(self.g.StringMobject("x+y"), self.g.StringMobject("y+x"), matched_keys=["x"])
        self.assertEqual(self.pairs(animation), [("x", "x")])

    def test_custom_factory_reaches_string_children(self):
        calls = []
        def factory(source, target, **kwargs):
            calls.append((source, target))
            return self.g.Transform(source, target, **kwargs)
        animation = self.make("foo", "bar", key_map={"foo": "bar"}, mismatch_animation=factory)
        self.assertEqual(len(calls), 1)
        self.assertIs(calls[0][0], animation.animations[0].mobject)

    def test_foreign_override_output_refuses(self):
        class Authored(self.g.TransformMatchingStrings):
            def matching_blocks(inner, source, target, *args):
                return [(self.g.Mobject("foreign"), target.children[0])]
        with self.assertRaisesRegex(ValueError, "foreign source"):
            Authored(self.g.StringMobject("a"), self.g.StringMobject("a"))

    def test_split_utf8_refuses_before_custom_hook(self):
        calls = []
        class Authored(self.g.TransformMatchingStrings):
            def matching_blocks(self, *args):
                calls.append("hook")
                return []
        source, target = self.g.StringMobject("é"), self.g.StringMobject("é")
        source._string_sub_spans = [(0, 1)]
        with self.assertRaisesRegex(ValueError, "UTF-8"):
            Authored(source, target)
        self.assertEqual(calls, [])

    def test_key_cannot_split_a_native_grouped_part(self):
        source, target = self.g.StringMobject("ab", grouped=True), self.g.StringMobject("cd", grouped=True)
        with self.assertRaisesRegex(ValueError, "splits a native source-span"):
            self.g.TransformMatchingStrings(source, target, key_map={"a": "c"})

    def test_invalid_keys_and_missing_provenance_refuse(self):
        for options in ({"matched_keys": [1]}, {"key_map": {"a": 1}}, {"key_map": []}):
            with self.subTest(options=options), self.assertRaises(TypeError):
                self.make("a", "a", **options)
        source = self.g.StringMobject("a")
        source._string_sub_spans = []
        with self.assertRaisesRegex(ValueError, "point-bearing strings require a native span map"):
            self.g.TransformMatchingStrings(source, self.g.StringMobject(""))

    def test_zero_ink_strings_are_timed_native_compositions(self):
        for source, target, kinds in (("", "a", ["fade_in_from_point"]),
                                      ("a", "", ["fade_out_to_point"]),
                                      ("", "", [])):
            with self.subTest(source=source, target=target):
                animation = self.make(source, target, run_time=3)
                self.assertEqual([child._native_kind for child in animation.animations], kinds)
                self.assertEqual(animation.group_options["run_time"], 3)
                self.assertEqual(animation.matched_pairs, [])
                animation.begin()
                animation.finish()

    def test_idempotent_install_keeps_helper_identity(self):
        helper, constructor = self.g.TransformMatchingStrings.matching_blocks, self.g.TransformMatchingStrings.__init__
        matching.install_matching_strings(self.g)
        self.assertIs(helper, self.g.TransformMatchingStrings.matching_blocks)
        self.assertIs(constructor, self.g.TransformMatchingStrings.__init__)


if __name__ == "__main__":
    unittest.main()
