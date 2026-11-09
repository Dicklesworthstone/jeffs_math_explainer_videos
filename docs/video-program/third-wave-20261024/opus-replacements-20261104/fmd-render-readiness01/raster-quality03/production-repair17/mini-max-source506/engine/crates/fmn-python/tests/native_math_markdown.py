"""Real-native mathematical document construction, without host layout doubles."""
from __future__ import annotations

import unittest
import numpy as np
import manimlib as m


def build(source, **options):
    root = m.VMobject()
    specs, catalog = m._build_math_markdown(root, m._native_shell_factory, source, **options)
    m._hang_native_children(root, specs)
    return root, catalog


def shape(obj):
    runs = [np.asarray(part.get_points()).copy() for part in obj.family_members_with_points()]
    values = np.concatenate(runs) if runs else np.empty((0, 3))
    return values - values[0] if len(values) else values


class NativeMarkdownTests(unittest.TestCase):
    def test_original_utf8_block_catalog_and_native_formula_geometry(self):
        source = '# Ω\n\nBefore $x_i+y_j$ after.\n\n```math\n\\frac{1}{x^2}\n```\n'
        root, catalog = build(source, font_size=48)
        self.assertEqual(len(root), 3)
        self.assertEqual([entry[2] for entry in catalog], ['heading', 'paragraph', 'math'])
        encoded = source.encode('utf-8')
        self.assertIn('Ω', encoded[catalog[0][0]:catalog[0][1]].decode('utf-8'))
        self.assertIn('$x_i+y_j$', encoded[catalog[1][0]:catalog[1][1]].decode('utf-8'))
        np.testing.assert_allclose(shape(root[1][1]), shape(m.Tex('x_i+y_j')), atol=2e-6)
        np.testing.assert_allclose(shape(root[2]), shape(m.Tex(r'\displaystyle\frac{1}{x^2}')), atol=2e-6)

    def test_wrapped_formula_atoms_remain_native_and_do_not_overlap(self):
        text = r'alpha $\frac{1}{x}$ beta $\sqrt{x}$ gamma'
        wide, _ = build(text, font_size=48)
        narrow, _ = build(text, font_size=48, line_width=1.5)
        self.assertGreater(narrow.get_height(), wide.get_height())
        self.assertEqual(len(wide[0]), len(narrow[0]))
        for left, right in zip(wide[0], narrow[0]):
            np.testing.assert_allclose(shape(left), shape(right), atol=2e-6)

    def test_code_is_literal_and_tables_and_nested_blocks_have_geometry(self):
        source = '`$x_i$ <a>&b`\n\n- outer\n  - inner\n\n> ## Quote\n>\n> second paragraph\n\n| x | y |\n|---|---|\n| 1 | 2 |\n'
        root, catalog = build(source)
        self.assertEqual([entry[2] for entry in catalog], ['paragraph', 'list', 'quote', 'table'])
        self.assertTrue(all(part.get_all_points().size for part in root))
        for a, b in zip(root, root.submobjects[1:]):
            self.assertGreater(a.get_bottom()[1], b.get_top()[1])
        literal, _ = build(r'\$unpaired')
        self.assertGreater(literal.get_width(), 0)

    def test_invalid_documents_leave_an_existing_native_receiver_unchanged(self):
        root = m.Square()
        before = shape(root).copy()
        for source, options in [(r'$\unknownmarkdowncommand{x}$', {}),
                                ('![x](https://example.invalid/x.png)', {}),
                                ('x' * 32769, {}), ('x', {'font_size': -1}),
                                ('x', {'line_width': float('nan')}),
                                ('x', {'block_gap': -1}), ('x', {'code_style': 'not-a-theme'})]:
            with self.subTest(source=source[:60], options=options):
                with self.assertRaises((ValueError, RuntimeError)):
                    m._build_math_markdown(root, m._native_shell_factory, source, **options)
                np.testing.assert_array_equal(shape(root), before)
                self.assertEqual(len(root), 0)

    def test_display_lines_and_reference_links_preserve_native_formula_layout(self):
        root, _ = build(r'before $$\frac{1}{x}$$ after', font_size=48)
        self.assertEqual(len(root[0]), 3)
        self.assertGreater(root[0][0].get_bottom()[1], root[0][1].get_top()[1])
        self.assertGreater(root[0][1].get_bottom()[1], root[0][2].get_top()[1])
        reference, _ = build('[details](https://example.invalid/) $x_i$')
        actual, catalog = build('[details][info] $x_i$\n\n[info]: https://example.invalid/\n')
        np.testing.assert_allclose(shape(actual), shape(reference), atol=3e-6)
        self.assertEqual(len(catalog), 1)
        code, _ = build('`unclosed $x_i$', font_size=48)
        np.testing.assert_allclose(shape(code[0][-1]), shape(m.Tex('x_i')), atol=3e-6)

    def test_empty_source_is_an_empty_native_document(self):
        root, catalog = build(' \n\n')
        self.assertEqual(catalog, [])
        self.assertEqual(len(root), 0)
        self.assertEqual(len(shape(root)), 0)


if __name__ == '__main__':
    unittest.main(verbosity=2)
