"""Source-level native-span planning tests, not native rendering acceptance."""
import ast
import difflib
import itertools
from pathlib import Path
import unittest
from test_matching_parts_protocol import environment, module, SOURCE


def strings_environment():
    g = environment()
    class StringMobject(g["VMobject"]):
        def __init__(self, text, tokens=None, shape=None):
            tokens = list(text) if tokens is None else tokens
            super().__init__(*(g["VMobject"](shape=token if shape is None else shape) for token in tokens))
            self.text, self._string_sub_spans = text, []
            offset = 0
            for token in tokens:
                end = offset + len(token.encode("utf-8"))
                self._string_sub_spans.append((offset, end))
                offset = end
        def get_string(self):
            return self.text
        def _string_submobject(self, ordinal):
            return self.children[ordinal]
    class TransformMatchingStrings(g["NativeAnimation"]):
        _native_kind = "transform_matching_strings"
        _match_by_blocks = True
    class TransformMatchingTex(TransformMatchingStrings):
        _native_kind = "transform_matching_tex"
        _match_by_blocks = False
    # Execute the unchanged production UTF-8 span helper rather than copy its
    # algorithm into a test double. No geometry/storage is supplied by it.
    path = SOURCE.parent.parent / "manimlib_bootstrap.py"
    tree = ast.parse(path.read_text())
    cls = next(node for node in tree.body if isinstance(node, ast.ClassDef) and node.name == "TransformMatchingStrings")
    helper = next(node for node in cls.body if isinstance(node, ast.FunctionDef) and node.name == "_native_span_keys")
    namespace = {"_TexError": ValueError}
    exec(compile(ast.Module(body=[helper], type_ignores=[]), str(path), "exec"), namespace)
    TransformMatchingStrings._native_span_keys = namespace["_native_span_keys"]
    g.update(StringMobject=StringMobject, TransformMatchingStrings=TransformMatchingStrings,
             TransformMatchingTex=TransformMatchingTex, _TexError=ValueError, _difflib=difflib)
    module._install_matching_parts(g)
    module._install_matching_strings(g)
    return g


class MatchingStringsProtocolTests(unittest.TestCase):
    def setUp(self):
        self.g = strings_environment()
        self.text = self.g["StringMobject"]
        self.Strings = self.g["TransformMatchingStrings"]
        self.Tex = self.g["TransformMatchingTex"]

    def transforms(self, animation):
        return [child for child in animation.animations if child._native_kind == "transform"]

    def test_shared_initializer_calls_string_installer(self):
        tree = ast.parse(SOURCE.read_text())
        install = next(node for node in tree.body if isinstance(node, ast.FunctionDef) and node.name == "install")
        self.assertTrue(any(isinstance(node, ast.Call) and isinstance(node.func, ast.Name)
                            and node.func.id == "_install_matching_strings" for node in ast.walk(install)))

    def test_reference_inheritance_and_callback_dispatch(self):
        self.assertTrue(issubclass(self.Strings, self.g["TransformMatchingParts"]))
        self.assertTrue(issubclass(self.Tex, self.Strings))
        self.assertIsNone(self.Strings._native_kind)
        self.assertIsNone(self.Tex._native_kind)
        self.assertEqual(self.Strings(self.text("ab"), self.text("ba"))._native_params(), {})

    def test_override_controls_selected_parts(self):
        calls = []
        class Authored(self.Strings):
            def matching_blocks(self, source, target, matched_keys, key_map):
                calls.append((source, target, matched_keys, key_map))
                return [(source.children[0], target.children[1])]
        source, target = self.text("ab"), self.text("xy")
        animation = Authored(source, target)
        transforms = self.transforms(animation)
        self.assertEqual(len(calls), 1)
        self.assertEqual(len(transforms), 1)
        self.assertIs(transforms[0].mobject, source.children[0])
        self.assertIs(transforms[0].target_mobject, target.children[1])

    def test_reordered_blocks_transform_instead_of_fading(self):
        animation = self.Strings(self.text("abCD"), self.text("CDab"))
        transforms = self.transforms(animation)
        self.assertEqual(len(animation.animations), 2)
        self.assertEqual([len(child.mobject.family_members_with_points()) for child in transforms], [2, 2])

    def test_unicode_keys_use_byte_spans(self):
        source, target = self.text("αβ😀"), self.text("😀βα")
        animation = self.Strings(source, target)
        transforms = self.transforms(animation)
        self.assertEqual(len(transforms), 3)
        self.assertEqual(sum(len(child.mobject.family_members_with_points()) for child in transforms), 3)
        self.assertEqual([key for _, key in animation._native_span_keys(source)], ["α", "β", "😀"])

    def test_explicit_pairs_win_and_input_collections_unchanged(self):
        source, target = self.text("abc"), self.text("xyz")
        pairs = [(source.children[0], target.children[2])]
        mapping = {"a": "x"}
        animation = self.Strings(source, target, matched_pairs=pairs, key_map=mapping)
        transforms = self.transforms(animation)
        self.assertEqual(len(transforms), 1)
        self.assertIs(transforms[0].target_mobject, target.children[2])
        self.assertEqual(pairs, [(source.children[0], target.children[2])])
        self.assertEqual(mapping, {"a": "x"})
        self.assertEqual(animation.matched_pairs, pairs)

    def test_multi_glyph_key_map_and_repeated_occurrences(self):
        animation = self.Strings(self.text("foo+foo"), self.text("bar+bar"), key_map={"foo": "bar"})
        transforms = self.transforms(animation)
        self.assertEqual(len(transforms), 3)
        self.assertEqual([len(child.mobject.family_members_with_points()) for child in transforms], [3, 3, 1])
        self.assertEqual(len(animation.animations), 3)

    def test_grouped_explicit_pair_claims_all_native_descendants(self):
        source, target = self.text("abc"), self.text("xyz")
        pair = (self.g["VGroup"](*source.children[:2]), self.g["VGroup"](*target.children[:2]))
        animation = self.Strings(source, target, matched_pairs=[pair], key_map={"b": "z"})
        self.assertEqual(len(self.transforms(animation)), 1)
        self.assertEqual(len(animation.animations), 3)

    def test_masks_do_not_create_false_adjacent_blocks(self):
        source, target = self.text("aXa"), self.text("aaY")
        animation = self.Strings(source, target, matched_pairs=[(source.children[1], target.children[2])])
        transforms = self.transforms(animation)
        self.assertEqual(len(transforms), 3)
        self.assertTrue(all(len(child.mobject.family_members_with_points()) == 1 for child in transforms))

    def test_native_semantics_not_glyph_shape(self):
        animation = self.Strings(self.text("a", shape="same"), self.text("b", shape="same"))
        self.assertEqual(self.transforms(animation), [])
        self.assertEqual(len(animation.animations), 2)

    def test_tex_keys_and_whitelist(self):
        source, target = self.text("x+x"), self.text("x+x")
        self.assertEqual(len(self.transforms(self.Tex(source, target))), 3)
        selected = self.Tex(source, target, matched_keys=("x",))
        self.assertEqual(len(self.transforms(selected)), 2)
        self.assertEqual(len(selected.animations), 4)

    def test_tex_key_map_precedes_whitelist(self):
        animation = self.Tex(self.text("x+x"), self.text("y+y"), matched_keys=("x",), key_map={"x": "y"})
        self.assertEqual(len(self.transforms(animation)), 2)
        self.assertEqual(len(animation.animations), 4)

    def test_explicit_duplicate_and_foreign_parts_refuse(self):
        source, target = self.text("ab"), self.text("xy")
        with self.assertRaisesRegex(ValueError, "same part twice"):
            self.Strings(source, target, matched_pairs=[(source.children[0], target.children[0]),
                                                       (source.children[0], target.children[1])])
        with self.assertRaisesRegex(ValueError, "live span-map part"):
            self.Strings(source, target, matched_pairs=[(self.text("a").children[0], target.children[0])])

    def test_utf8_and_partial_native_span_refusals(self):
        source, target = self.text("α"), self.text("α")
        source._string_sub_spans = [(0, 1)]
        with self.assertRaisesRegex(ValueError, "UTF-8 code point"):
            self.Strings(source, target)
        source, target = self.text(r"\alpha", tokens=[r"\alpha"]), self.text("a")
        with self.assertRaisesRegex(ValueError, "splits a native source-span"):
            self.Strings(source, target, key_map={"a": "a"})

    def test_literal_reference_sentinels_are_ordinary_text(self):
        source, target = self.text("Null1Null2"), self.text("Null2Null1")
        animation = self.Strings(source, target)
        transforms = self.transforms(animation)
        sources = [id(obj) for child in transforms for obj in child.mobject.family_members_with_points()]
        targets = [id(obj) for child in transforms for obj in child.target_mobject.family_members_with_points()]
        self.assertEqual(len(sources), len(set(sources)))
        self.assertEqual(set(sources), {id(obj) for obj in source.children})
        self.assertEqual(set(targets), {id(obj) for obj in target.children})

    def test_all_reorderings_preserve_occurrences_once(self):
        for permutation in sorted(set(itertools.permutations("abbc"))):
            source, target = self.text("abbc"), self.text("".join(permutation))
            animation = self.Strings(source, target)
            transforms = self.transforms(animation)
            source_ids, target_ids = [], []
            for child in transforms:
                left = child.mobject.family_members_with_points()
                right = child.target_mobject.family_members_with_points()
                self.assertEqual([part.shape for part in left], [part.shape for part in right])
                source_ids.extend(id(part) for part in left)
                target_ids.extend(id(part) for part in right)
            self.assertEqual(len(source_ids), 4)
            self.assertEqual(len(set(source_ids)), 4)
            self.assertEqual(len(target_ids), 4)
            self.assertEqual(len(set(target_ids)), 4)

    def test_aliased_native_parts_are_not_claimed_twice(self):
        source, target = self.text("aa"), self.text("aa")
        source.children[1] = source.children[0]
        animation = self.Strings(source, target, matched_keys=("a",))
        transforms = self.transforms(animation)
        self.assertEqual(len(transforms), 1)
        self.assertEqual(len(animation.animations), 2)

    def test_point_free_native_span_cannot_stall_matching(self):
        source, target = self.text("ab"), self.text("ab")
        source.children[0].shape = target.children[0].shape = None
        animation = self.Strings(source, target)
        self.assertEqual(len(self.transforms(animation)), 1)
        self.assertEqual(len(self.transforms(animation)[0].mobject.family_members_with_points()), 1)

    def test_live_protocol_and_cleanup_for_text(self):
        source, target = self.text("ab"), self.text("ba")
        animation = self.Strings(source, target)
        scene = self.g["Scene"]()
        scene.add(source)
        scene.play(animation)
        self.assertEqual(scene.roots, [target])
        self.assertEqual(len(scene.cores), 2)
        self.assertTrue(all(("interpolate", .5) in core.events for core in scene.cores))

    def test_abort_releases_drivers_and_is_idempotent(self):
        animation = self.Strings(self.text("ab"), self.text("ba"))
        scene = self.g["Scene"]()
        scene.add(animation.mobject)
        animation.begin()
        animation.abort()
        animation.abort()
        self.assertIsNone(animation._matching_driver)
        self.assertTrue(all(core.events.count(("abort",)) == 1 for core in scene.cores))

    def test_rebegin_unwinds_previous_active_drivers(self):
        animation = self.Strings(self.text("ab"), self.text("ba"))
        scene = self.g["Scene"]()
        scene.add(animation.mobject)
        animation.begin()
        animation.begin()
        self.assertIn(("abort",), scene.cores[0].events)
        self.assertEqual(len(scene.cores), 4)

    def test_invalid_key_types_and_empty_native_map_refuse(self):
        with self.assertRaisesRegex(TypeError, "keys must be strings"):
            self.Strings(self.text("a"), self.text("b"), key_map={1: "b"})
        with self.assertRaisesRegex(ValueError, "non-empty native span maps"):
            self.Strings(self.text(""), self.text("b"))


if __name__ == "__main__":
    unittest.main(verbosity=2)
