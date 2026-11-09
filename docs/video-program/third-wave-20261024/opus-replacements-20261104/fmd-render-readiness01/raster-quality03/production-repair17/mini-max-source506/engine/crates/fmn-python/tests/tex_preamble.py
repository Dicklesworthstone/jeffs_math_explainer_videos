"""Preambles and packs through real native constructors, selectors and copies."""
import copy
import pickle
import unittest

import manimlib as m
from manimlib.mobject.svg.old_tex_mobject import SingleStringTex
import numpy as np

PREAMBLE = r"\newcommand{\half}[1]{\frac{#1}{2}}\newcommand{\sq}[1]{#1^2}"


class TexPreambleTests(unittest.TestCase):
    def assertGeometryEqual(self, actual, expected):
        np.testing.assert_allclose(actual.get_all_points(), expected.get_all_points(), atol=1e-6)

    def test_native_macro_expansion_matches_unfolded_geometry(self):
        for source, expanded in ((r"\half{x}", r"\frac{x}{2}"),
                                 (r"\sq{x}+\half{y}", r"x^2+\frac{y}{2}")):
            with self.subTest(source=source):
                actual = m.Tex(source, additional_preamble=PREAMBLE)
                self.assertGeometryEqual(actual, m.Tex(expanded))
                self.assertEqual(actual.get_tex(), source)
                self.assertEqual(actual.additional_preamble, PREAMBLE)
                self.assertTrue(actual.family_members_with_points())

    def test_arguments_and_generated_ink_keep_original_source_coordinates(self):
        source = r"\half{x}+\sq{y}"
        mob = m.Tex(source, additional_preamble=PREAMBLE, isolate=["x", "y"],
                    tex_to_color_map={"x": m.RED, "y": m.BLUE})
        for token, color in (("x", m.RED), ("y", m.BLUE)):
            part = mob.get_part_by_tex(token)
            self.assertTrue(part.family_members_with_points())
            self.assertTrue(all(leaf.get_fill_color() == color for leaf in part.family_members_with_points()))
        for start, end in mob._string_sub_spans:
            self.assertTrue(0 <= start <= end <= len(source.encode()))
        self.assertTrue(mob.get_part_by_tex(r"\half{x}").family_members_with_points())
        self.assertFalse(mob.get_parts_by_tex("newcommand").family_members_with_points())

    def test_text_mainland_unicode_and_macro_math_islands(self):
        source = "é area $\\sq{x}$"
        actual = m.TexText(source, additional_preamble=PREAMBLE)
        self.assertGeometryEqual(actual, m.TexText("é area $x^2$"))
        self.assertEqual(actual.get_tex(), source)
        self.assertTrue(actual.get_part_by_tex("é").family_members_with_points())
        self.assertTrue(actual.get_part_by_tex("x").family_members_with_points())
        for start, end in actual._string_sub_spans:
            source.encode()[:start].decode()
            source.encode()[:end].decode()

    def test_multiple_parts_select_flat_glyphs_using_formula_spans_not_definition_bytes(self):
        mob = m.Tex(r"\sq{x}", "+", r"\half{y}", additional_preamble=PREAMBLE)
        # x and its square, plus, y and the denominator/rule: six native
        # primitives. Logical argument selection does not add family nodes.
        self.assertEqual(len(mob), 6)
        self.assertTrue(all(child.has_points() and not child.submobjects for child in mob))
        self.assertTrue(all(len(path) == 1 for path in mob._string_sub_paths))
        for token in (r"\sq{x}", "+", r"\half{y}"):
            self.assertTrue(mob.get_part_by_tex(token).family_members_with_points(), token)
        self.assertEqual(mob.get_tex(), r"\sq{x} + \half{y}")

    def test_macro_definitions_are_per_object_and_cache_separates_them(self):
        first = m.Tex(r"\term", additional_preamble=r"\newcommand{\term}{x}")
        second = m.Tex(r"\term", additional_preamble=r"\newcommand{\term}{y^2}")
        self.assertGeometryEqual(first, m.Tex("x"))
        self.assertGeometryEqual(second, m.Tex("y^2"))
        repeated = m.Tex(r"\term", additional_preamble=r"\newcommand{\term}{x}")
        self.assertGeometryEqual(first, repeated)
        with self.assertRaises(ValueError):
            m.Tex(r"\term")

    def test_known_packs_have_native_semantics_without_fallback(self):
        plain = m.Tex("x")
        for template in ("", "default", "basic", "empty"):
            self.assertGeometryEqual(m.Tex("x", template=template), plain)
        self.assertGeometryEqual(m.Tex(r"x\minus y", template="default"), m.Tex("x-y"))
        for template in ("basic", "empty"):
            # The symbol vocabulary also includes minus. Pack identity is
            # observable through macro shadowing, not by denying that symbol.
            self.assertGeometryEqual(m.Tex(r"x\minus y", template=template), m.Tex("x-y"))
            self.assertGeometryEqual(m.Tex(r"\minus", template=template,
                additional_preamble=r"\newcommand{\minus}{+}"), m.Tex("+"))
            with self.assertRaisesRegex(ValueError, "not defined"):
                m.Tex(r"\minus", template=template,
                      additional_preamble=r"\renewcommand{\minus}{+}")
        with self.assertRaises(ValueError):
            m.Tex("x", template="default", additional_preamble=r"\newcommand{\minus}{+}")
        self.assertGeometryEqual(m.Tex(r"\minus", additional_preamble=r"\renewcommand{\minus}{+}"), m.Tex("+"))

    def test_legacy_leaf_and_matrix_configuration_use_the_same_native_path(self):
        leaf = SingleStringTex(r"\half{x}", additional_preamble=PREAMBLE, template="basic")
        self.assertGeometryEqual(leaf, SingleStringTex(r"\frac{x}{2}"))
        matrix = m.TexMatrix([[r"\sq{x}", r"\half{y}"]],
                             tex_config=dict(additional_preamble=PREAMBLE, template="empty"))
        self.assertEqual(len(matrix.get_entries()), 2)
        for cell, token in zip(matrix.get_entries(), ("x", "y")):
            self.assertTrue(cell.get_part_by_tex(token).family_members_with_points())

    def test_copy_deepcopy_and_pickle_preserve_configuration_and_selection(self):
        mob = m.Tex(r"\sq{x}+1", additional_preamble=PREAMBLE, template="empty", isolate=["x"])
        before = mob.get_all_points().copy()
        for clone in (mob.copy(), copy.deepcopy(mob), pickle.loads(pickle.dumps(mob))):
            self.assertEqual(clone.get_tex(), mob.get_tex())
            self.assertEqual(clone.template, "empty")
            self.assertEqual(clone.additional_preamble, PREAMBLE)
            self.assertEqual(clone._string_sub_spans, mob._string_sub_spans)
            clone.set_color_by_tex("x", m.BLUE).shift(m.RIGHT)
            self.assertTrue(clone.get_part_by_tex("x").family_members_with_points())
            np.testing.assert_array_equal(mob.get_all_points(), before)

    def test_formula_errors_report_original_offsets(self):
        source = r"x+\notanativecommand"
        messages = []
        for preamble in ("", "% é comment\n" + PREAMBLE):
            try:
                m.Tex(source, additional_preamble=preamble)
            except ValueError as error:
                messages.append(str(error))
            else:
                self.fail("unknown native command was accepted")
        self.assertEqual(*messages)

    def test_invalid_preambles_refuse_before_installing_live_state(self):
        for preamble in ("x", r"\quad", r"\usepackage{amsmath}", r"\input{secret.tex}",
                         r"\write18{command}", r"\newcommand{\bad}[1]{#2}", "%" * 65537):
            for cls in (m.Tex, m.TexText, SingleStringTex):
                with self.subTest(preamble=preamble[:50], cls=cls):
                    obj = cls.__new__(cls)
                    with self.assertRaisesRegex(ValueError, "additional_preamble"):
                        cls.__init__(obj, "x", additional_preamble=preamble)
                    self.assertNotIn("_submobjects", vars(obj))
                    self.assertFalse(hasattr(obj, "submobjects"))

    def test_unsupported_templates_and_wrong_option_types_are_named(self):
        for template in ("legacy", "ctex", "comic_sans"):
            with self.subTest(template=template), self.assertRaisesRegex(ValueError, "template.*available packs"):
                m.Tex("x", template=template)
        for options in ({"template": None}, {"template": {}}, {"additional_preamble": []}):
            with self.subTest(options=options), self.assertRaises(TypeError):
                m.Tex("x", **options)

    def test_comments_source_nested_calls_and_recursion_use_the_native_expander(self):
        preamble = PREAMBLE + "% end"
        self.assertGeometryEqual(m.Tex(r"\half{\sq{x}}", additional_preamble=preamble), m.Tex(r"\frac{x^2}{2}"))
        with self.assertRaisesRegex(ValueError, "loop"):
            m.Tex(r"\loop", additional_preamble=r"\newcommand{\loop}{\loop}")

    def test_definition_nested_macros_keep_call_site_and_argument_provenance(self):
        # UPSTREAM_LEDGER #13, resolved at the e911be2a pin: a macro whose
        # definition calls another macro lays out as its full expansion,
        # generated ink keeps the call site's span, and a forwarded argument
        # keeps its own source bytes. No span escapes the formula.
        for preamble, source, expansion in (
            (r"\newcommand{\inner}{x}\newcommand{\outer}{\inner}", r"\outer", "x"),
            (r"\newcommand{\sq}[1]{#1^2}\newcommand{\outer}[1]{\sq{#1}}", r"\outer{x}", "x^2"),
        ):
            with self.subTest(source=source):
                formula = m.Tex(source, additional_preamble=preamble)
                self.assertGeometryEqual(formula, m.Tex(expansion))
                self.assertTrue(all(
                    0 <= start <= end <= len(source.encode())
                    for start, end in formula._string_sub_spans
                ))
        forwarded = m.Tex(r"\outer{x}", additional_preamble=(
            r"\newcommand{\sq}[1]{#1^2}\newcommand{\outer}[1]{\sq{#1}}"
        ))
        self.assertIn((7, 8), [tuple(span) for span in forwarded._string_sub_spans])
        self.assertEqual(len(forwarded["x"].family_members_with_points()), 1)

    def test_live_number_inside_macro_argument_preserves_generated_ink(self):
        for preamble, source in (
            (r"\newcommand{\coefficient}[1]{z = #1}", r"\coefficient{1.00}"),
            (r"\newcommand{\half}[1]{\frac{#1}{2}}", r"\half{1.00}"),
        ):
            with self.subTest(source=source):
                formula = m.Tex(source, additional_preamble=preamble)
                # Whole-call generated ink (z, =, denominator and rule) must
                # survive replacing only the literal numeric argument.
                generated = []
                for span, path in zip(formula._string_sub_spans, formula._string_sub_paths):
                    if span == (0, len(source)):
                        part = formula
                        for index in path:
                            part = part.submobjects[index]
                        generated.append(part)
                self.assertTrue(generated)
                before = [part.data.copy() for part in generated]
                number = formula.make_number_changeable("1.00")
                number.set_value(2.5)
                self.assertEqual(number.get_value(), 2.5)
                self.assertIs(formula.get_part_by_tex(r"\decimalmob")[0], number)
                for part, data in zip(generated, before):
                    self.assertIn(part, formula.get_family())
                    np.testing.assert_array_equal(part.data, data)

    def test_repeated_macro_argument_refuses_numeric_collapse_without_mutation(self):
        preamble = r"\newcommand{\twice}[1]{#1+#1}"
        for inline in (False, True):
            with self.subTest(inline=inline):
                source = r"\twice{1.00}"
                formula = (m.Tex(preamble + source) if inline else
                           m.Tex(source, additional_preamble=preamble))
                self.assertEqual(len(formula.get_part_by_tex("1.00")), 8)
                scene = m.Scene()
                scene.add(formula)
                def snapshot():
                    return (formula.string, tuple(formula._string_sub_spans),
                            tuple(map(tuple, formula._string_sub_paths)),
                            tuple((id(part), part.data.tobytes()) for part in formula.get_family()))
                before = snapshot()
                for replace_all in (False, True):
                    with self.assertRaisesRegex(ValueError, "multiple native occurrences"):
                        formula.make_number_changeable("1.00", replace_all=replace_all)
                    self.assertEqual(snapshot(), before)
                    self.assertEqual(list(scene.mobjects), [formula])


if __name__ in ("__main__", "<run_path>"):
    result = unittest.TextTestRunner(verbosity=2).run(
        unittest.defaultTestLoader.loadTestsFromTestCase(TexPreambleTests))
    if not result.wasSuccessful():
        raise AssertionError("native TeX preamble acceptance failed")
